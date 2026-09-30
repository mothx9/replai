//! Optional semantic presentation extension. Only public safe producer APIs.
use super::*;
use replai::{Block, Column, CompletionCandidate, CompletionSet, Document, Span, Text, Theme};
use std::{
    os::fd::AsRawFd,
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

unsafe fn extension_record<T: Copy>(p: *const T) -> Result<T, i32> {
    // SAFETY: exact versioned extension record supplied by the caller.
    unsafe { record_version(p, REPLAI_PRESENTATION_VERSION) }
}

static TICKETS: AtomicU64 = AtomicU64::new(1);
fn ticket() -> Result<u64, i32> {
    TICKETS
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .map_err(|_| REPLAI_CAPACITY)
}
unsafe fn array<'a, T>(p: *const T, count: usize, limit: usize) -> Result<&'a [T], i32> {
    if count > limit {
        return Err(REPLAI_CAPACITY);
    }
    let bytes = count
        .checked_mul(mem::size_of::<T>())
        .ok_or(REPLAI_INVALID_ARGUMENT)?;
    span(p.cast(), bytes)?;
    if count == 0 {
        return Ok(&[]);
    }
    aligned(p)?;
    // SAFETY: caller promises live aligned storage for the checked extent. No borrow escapes.
    Ok(unsafe { std::slice::from_raw_parts(p, count) })
}
unsafe fn semantic(t: &ReplaiText) -> Result<Text, i32> {
    // SAFETY: the caller supplies a full live versioned record.
    let t = unsafe { extension_record(t)? };
    if t.reserved != [0; 2] {
        return Err(REPLAI_INVALID_ARGUMENT);
    }
    let mut spans = Vec::new();
    let mut bytes = 0usize;
    // SAFETY: caller supplies the checked span array for this call.
    for p in unsafe { array(t.spans, t.span_count, 1024)? } {
        // SAFETY: each array element is a live record; prefix validation precedes fields.
        let p = unsafe { extension_record(p)? };
        if p.reserved != 0 {
            return Err(REPLAI_INVALID_ARGUMENT);
        }
        bytes = bytes.checked_add(p.text_bytes).ok_or(REPLAI_CAPACITY)?;
        if bytes > 16 * 1024 {
            return Err(REPLAI_CAPACITY);
        }
        // SAFETY: the checked caller span is borrowed only during copying.
        spans.push(Span::new(role(p.role)?, unsafe { text(p.text, p.text_bytes)? }).map_err(edit)?);
    }
    Text::from_spans(spans).map_err(edit)
}
fn empty(t: &ReplaiText) -> bool {
    t.span_count == 0
}
unsafe fn document_budget(records: &[ReplaiBlock]) -> Result<(), i32> {
    let (mut bytes, mut spans) = (0usize, 0usize);
    for b in records {
        // SAFETY: live caller-owned block in the previously bounded array.
        let b = unsafe { extension_record(b)? };
        // SAFETY: cells are live caller-owned records; count is bounded before reading.
        let cells = unsafe { array(b.cells, b.cell_count, 32)? };
        for t in [&b.text, &b.label].into_iter().chain(cells.iter()) {
            // SAFETY: embedded/cell record extent remains live through this call.
            let t = unsafe { extension_record(t)? };
            // SAFETY: bounded array supplied by this semantic-text record.
            let items = unsafe { array(t.spans, t.span_count, 1024)? };
            spans = spans
                .checked_add(items.len().max(1))
                .ok_or(REPLAI_CAPACITY)?;
            for s in items {
                // SAFETY: each element is a live caller-owned versioned span.
                let s = unsafe { extension_record(s)? };
                bytes = bytes.checked_add(s.text_bytes).ok_or(REPLAI_CAPACITY)?;
            }
            if bytes > 1024 * 1024 || spans > 16_384 {
                return Err(REPLAI_CAPACITY);
            }
        }
    }
    Ok(())
}
unsafe fn document(p: *const ReplaiBlock, count: usize) -> Result<Document, i32> {
    // SAFETY: caller provides count readable block records; array enforces the budget.
    let records = unsafe { array(p, count, 4096)? };
    // SAFETY: preflight all extents/payload budgets before allocating copied text.
    unsafe {
        document_budget(records)?;
    }
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < records.len() {
        // SAFETY: this is a live element of the checked array.
        let b = unsafe { extension_record(&records[i])? };
        if b.reserved != [0; 2] {
            return Err(REPLAI_INVALID_ARGUMENT);
        }
        // SAFETY: live embedded versioned text records, copied only during this call.
        let t = unsafe { semantic(&b.text)? };
        // SAFETY: same embedded-record lifetime and extent contract.
        let label = unsafe { semantic(&b.label)? };
        if b.kind != REPLAI_BLOCK_FIELD && !empty(&b.label) {
            return Err(REPLAI_INVALID_ARGUMENT);
        }
        if b.kind != REPLAI_BLOCK_TABLE_HEADER
            && b.kind != REPLAI_BLOCK_TABLE_ROW
            && (b.cell_count != 0 || !b.cells.is_null())
        {
            return Err(REPLAI_INVALID_ARGUMENT);
        }
        let block = match b.kind {
            REPLAI_BLOCK_PARAGRAPH if b.level == 0 => Block::Paragraph(t),
            REPLAI_BLOCK_HEADING if (1..=3).contains(&b.level) => Block::Heading {
                level: b.level as u8,
                text: t,
            },
            REPLAI_BLOCK_LITERAL if b.level == 0 => Block::Literal(t),
            REPLAI_BLOCK_SPACER if b.level == 0 && empty(&b.text) => Block::Spacer,
            REPLAI_BLOCK_STATUS if b.level < 4 => Block::Status {
                severity: match b.level {
                    0 => replai::Severity::Info,
                    1 => replai::Severity::Success,
                    2 => replai::Severity::Warning,
                    _ => replai::Severity::Error,
                },
                text: t,
            },
            REPLAI_BLOCK_LIST if b.level <= 8 => Block::List {
                ordered: false,
                items: vec![replai::ListItem {
                    depth: b.level as u8,
                    text: t,
                }],
            },
            REPLAI_BLOCK_FIELD if b.level == 0 => {
                if let Some(Block::KeyValue(fields)) = blocks.last_mut() {
                    fields.push((label, t));
                    i += 1;
                    continue;
                }
                Block::KeyValue(vec![(label, t)])
            }
            REPLAI_BLOCK_TABLE_HEADER if empty(&b.text) => {
                if b.cell_count < 32 && (b.level >> b.cell_count) != 0 {
                    return Err(REPLAI_INVALID_ARGUMENT);
                }
                // SAFETY: cells are bounded caller-owned records, copied into Text.
                let columns = unsafe { array(b.cells, b.cell_count, 32)? }
                    .iter()
                    .enumerate()
                    .map(|(column, c)| {
                        // SAFETY: live cell in the checked array.
                        Ok(Column {
                            // SAFETY: live cell in the previously checked array.
                            heading: unsafe { semantic(c)? },
                            alignment: if b.level & (1u32 << column) != 0 {
                                replai::Alignment::Right
                            } else {
                                replai::Alignment::Left
                            },
                        })
                    })
                    .collect::<Result<Vec<_>, i32>>()?;
                Block::Table {
                    columns,
                    rows: Vec::new(),
                }
            }
            REPLAI_BLOCK_TABLE_ROW if b.level == 0 && empty(&b.text) => {
                let Some(Block::Table { rows, .. }) = blocks.last_mut() else {
                    return Err(REPLAI_INVALID_ARGUMENT);
                };
                // SAFETY: each bounded cell is copied during this call.
                let row = unsafe { array(b.cells, b.cell_count, 32)? }
                    .iter()
                    .map(|c| unsafe { semantic(c) })
                    .collect::<Result<_, _>>()?;
                rows.push(row);
                i += 1;
                continue;
            }
            _ => return Err(REPLAI_INVALID_ARGUMENT),
        };
        blocks.push(block);
        i += 1;
    }
    Document::new(blocks).map_err(edit)
}
fn completion_error(e: replai::CompletionError) -> i32 {
    match e {
        replai::CompletionError::Limit => REPLAI_CAPACITY,
        replai::CompletionError::InvalidDisplay => REPLAI_INVALID_TEXT,
        replai::CompletionError::Edit(e) => edit(e),
        replai::CompletionError::Interaction(e) => error(e),
    }
}

/// Query the optional extension.
/// # Safety
/// Output is live aligned caller storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_presentation_version(out: *mut u32) -> i32 {
    guard(|| {
        if let Err(e) = aligned(out) {
            return e;
        }
        // SAFETY: aligned writable caller slot.
        unsafe {
            out.write(REPLAI_PRESENTATION_VERSION);
        }
        REPLAI_OK
    })
}
/// Render semantic blocks without acquiring terminal resources.
/// # Safety
/// Records, arrays and output obey the installed header's extent/disjointness contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_document_render(
    blocks: *const ReplaiBlock,
    count: usize,
    options: *const ReplaiRender,
    buffer: *mut u8,
    capacity: usize,
    required: *mut usize,
) -> i32 {
    guard(|| {
        // SAFETY: caller supplies a versioned render-options record.
        let o = match unsafe { extension_record(options) } {
            Ok(o) => o,
            Err(e) => return e,
        };
        if o.styled > 1 || o.reserved != 0 {
            return REPLAI_INVALID_ARGUMENT;
        }
        // SAFETY: caller supplies the bounded complete document for this call.
        let d = match unsafe { document(blocks, count) } {
            Ok(d) => d,
            Err(e) => return e,
        };
        let rendered = match d.render_indented(
            o.columns as usize,
            Theme::new(o.styled == 1, false, None),
            o.indent as usize,
        ) {
            Ok(s) => s,
            Err(e) => return edit(e),
        };
        // SAFETY: exact-size caller-buffer contract is validated by copy_text.
        unsafe { copy_text(&rendered, buffer, capacity, required) }
    })
}
/// Coordinate a structured document with the current editor.
/// # Safety
/// Handle is live/serialized and arrays obey the installed extent contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_document_output(
    h: *mut Handle,
    blocks: *const ReplaiBlock,
    count: usize,
) -> i32 {
    // SAFETY: caller owns and serializes h; document only borrows caller records.
    unsafe {
        with_handle(h, |h| match document(blocks, count) {
            Ok(d) => result(h.interaction.output_document(&d)),
            Err(e) => e,
        })
    }
}
/// Configure composed prompts without leaking terminal escapes.
/// # Safety
/// Handle and versioned text records obey the installed extent contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_prompt_composed(
    h: *mut Handle,
    primary: *const ReplaiText,
    continuation: *const ReplaiText,
) -> i32 {
    // SAFETY: serialized handle and readable full records are caller-owned.
    unsafe {
        with_handle(h, |h| {
            if h.interaction.is_open() {
                return REPLAI_INVALID_STATE;
            }
            let p = match extension_record(primary)
                .and_then(|p| semantic(&p))
                .and_then(|p| Prompt::composed(p).map_err(edit))
            {
                Ok(p) => p,
                Err(e) => return e,
            };
            let p = match extension_record(continuation)
                .and_then(|c| semantic(&c))
                .and_then(|c| p.with_continuation_text(c).map_err(edit))
            {
                Ok(p) => p,
                Err(e) => return e,
            };
            h.prompt = p;
            REPLAI_OK
        })
    }
}
/// Capture a coherent revision-bound draft.
/// # Safety
/// Output slots and buffer are live writable disjoint storage; h is serialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_completion_snapshot(
    h: *mut Handle,
    buffer: *mut u8,
    capacity: usize,
    required: *mut usize,
    cursor: *mut usize,
    out_ticket: *mut u64,
) -> i32 {
    // SAFETY: handle/output storage follow the installed header contract.
    unsafe {
        with_handle(h, |h| {
            if let Err(e) = aligned(cursor)
                .and_then(|()| aligned(out_ticket))
                .and_then(|()| aligned(required))
                .and_then(|()| span(buffer, capacity))
            {
                return e;
            }
            let id = match ticket() {
                Ok(id) => id,
                Err(e) => return e,
            };
            let snapshot = h.interaction.analysis_snapshot();
            let status = copy_text(snapshot.text(), buffer, capacity, required);
            if status != REPLAI_OK && status != REPLAI_BUFFER_TOO_SMALL {
                return status;
            }
            cursor.write(snapshot.cursor());
            out_ticket.write(id);
            h.snapshot = Some((id, snapshot));
            status
        })
    }
}
/// Atomically install a current candidate set or refuse stale computation.
/// # Safety
/// Live serialized handle, disjoint output slot, readable candidate array when current.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_completions_present(
    h: *mut Handle,
    id: u64,
    candidates: *const ReplaiCandidate,
    count: usize,
    disposition: *mut u32,
) -> i32 {
    // SAFETY: only current delivery inspects caller candidate payloads.
    unsafe {
        with_handle(h, |h| {
            if let Err(e) = aligned(disposition) {
                return e;
            }
            let Some((current, snapshot)) = &h.snapshot else {
                return REPLAI_INVALID_ARGUMENT;
            };
            if id != *current {
                return REPLAI_INVALID_ARGUMENT;
            }
            if snapshot.revision() != h.interaction.revision() {
                disposition.write(REPLAI_STALE);
                return REPLAI_OK;
            }
            let mut result = Vec::new();
            let mut bytes = 0usize;
            let items = match array(candidates, count, replai::MAX_COMPLETION_CANDIDATES) {
                Ok(items) => items,
                Err(e) => return e,
            };
            for c in items {
                let c = match extension_record(c) {
                    Ok(c) => c,
                    Err(e) => return e,
                };
                if c.reserved != [0; 2] {
                    return REPLAI_INVALID_ARGUMENT;
                }
                for len in [c.insertion_bytes, c.label_bytes, c.annotation_bytes] {
                    bytes = match bytes.checked_add(len) {
                        Some(n) => n,
                        None => return REPLAI_CAPACITY,
                    };
                    if len > replai::MAX_COMPLETION_FIELD_BYTES
                        || bytes > replai::MAX_COMPLETION_BYTES
                    {
                        return REPLAI_CAPACITY;
                    }
                }
                let candidate = (|| {
                    let insertion = text(c.insertion, c.insertion_bytes)?;
                    let label = text(c.label, c.label_bytes)?;
                    let annotation = text(c.annotation, c.annotation_bytes)?;
                    let mut item = CompletionCandidate::new(c.start..c.end, insertion, label)
                        .map_err(completion_error)?;
                    if !annotation.is_empty() {
                        item = item.with_annotation(annotation).map_err(completion_error)?;
                    }
                    Ok(item)
                })();
                match candidate {
                    Ok(c) => result.push(c),
                    Err(e) => return e,
                };
            }
            let set = match CompletionSet::new(snapshot.revision(), result) {
                Ok(s) => s,
                Err(e) => return completion_error(e),
            };
            match h.interaction.present_completions(set) {
                Ok(outcome) => {
                    disposition.write(if outcome == replai::AnalysisOutcome::Stale {
                        REPLAI_STALE
                    } else {
                        REPLAI_APPLIED
                    });
                    REPLAI_OK
                }
                Err(e) => completion_error(e),
            }
        })
    }
}
/// Obtain host reactor interest without waiting.
/// # Safety
/// h is serialized; out is a zero-initialized versioned writable record.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_wait_interest(h: *mut Handle, out: *mut ReplaiInterest) -> i32 {
    // SAFETY: record prefix and caller-owned writable storage validated first.
    unsafe {
        with_handle(h, |h| {
            let value = match extension_record(out) {
                Ok(v) => v,
                Err(e) => return e,
            };
            if value.kind != 0
                || value.input_fd != 0
                || value.timeout_ms != 0
                || value.reserved != 0
                || value.deadline_ticket != 0
            {
                return REPLAI_INVALID_ARGUMENT;
            }
            let interest = match h.interaction.wait_interest() {
                Ok(i) => i,
                Err(e) => return error(e),
            };
            let fd = match h.interaction.input_source() {
                Ok(fd) => fd.as_raw_fd(),
                Err(e) => return error(e),
            };
            let (kind, deadline) = match interest {
                replai::WaitInterest::Ready => (REPLAI_WAIT_READY, None),
                replai::WaitInterest::Input { deadline } => (REPLAI_WAIT_INPUT, deadline),
            };
            let mut timeout = if kind == REPLAI_WAIT_READY { 0 } else { -1 };
            let mut id = 0;
            if let Some(deadline) = deadline {
                if h.deadline.is_none_or(|(_, old)| old != deadline) {
                    let new = match ticket() {
                        Ok(n) => n,
                        Err(e) => return e,
                    };
                    h.deadline = Some((new, deadline));
                }
                id = h.deadline.expect("deadline installed").0;
                timeout = deadline
                    .at()
                    .saturating_duration_since(Instant::now())
                    .as_millis()
                    .saturating_add(1)
                    .min(i32::MAX as u128) as i32;
            } else {
                h.deadline = None;
            }
            out.write(ReplaiInterest {
                struct_size: mem::size_of::<ReplaiInterest>() as u32,
                extension_version: REPLAI_PRESENTATION_VERSION,
                kind,
                input_fd: fd,
                timeout_ms: timeout,
                reserved: 0,
                deadline_ticket: id,
            });
            REPLAI_OK
        })
    }
}
/// Advance one advisory host notification without blocking.
/// # Safety
/// h is serialized; event follows the unchanged ABI 1 output-record rules.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_advance(
    h: *mut Handle,
    wake: u32,
    id: u64,
    event: *mut ReplaiEvent,
) -> i32 {
    // SAFETY: event validated before driving, and only public safe Interaction is used.
    unsafe {
        with_handle(h, |h| {
            if let Err(e) = event_ready(event) {
                return e;
            }
            let w = match wake {
                REPLAI_WAKE_INPUT if id == 0 => replai::Wake::InputReady,
                REPLAI_WAKE_RESIZE if id == 0 => replai::Wake::Resize,
                REPLAI_WAKE_DEADLINE => match h.deadline {
                    Some((n, d)) if id == n => replai::Wake::Deadline(d),
                    _ => return event_result(h, None, event),
                },
                _ => return REPLAI_INVALID_ARGUMENT,
            };
            match h.interaction.advance(w) {
                Ok(e) => event_result(h, e, event),
                Err(e) => error(e),
            }
        })
    }
}
/// Measure geometry using the producer's qualified Unicode policy.
/// # Safety
/// Input span/output slot are live disjoint caller storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_text_cells(p: *const u8, len: usize, out: *mut usize) -> i32 {
    guard(|| {
        if let Err(e) = aligned(out) {
            return e;
        }
        // SAFETY: caller span is borrowed only for this operation.
        let s = match unsafe { text(p, len) } {
            Ok(s) => s,
            Err(e) => return e,
        };
        match replai::WidthPolicy::UnicodeNarrow.measure(s) {
            // SAFETY: caller-owned aligned output slot.
            Ok(n) => {
                // SAFETY: caller-owned aligned writable output slot.
                unsafe {
                    out.write(n);
                }
                REPLAI_OK
            }
            Err(e) => edit(e),
        }
    })
}

/// Acquire the generic quiet-output scope, not an editor.
/// # Safety
/// Handle/FDs are live and serialized as in replai_open.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_output_open(h: *mut Handle, input: i32, output: i32) -> i32 {
    guard(|| {
        // SAFETY: integer-only FD checks; caller retains FD lifetime through duplication.
        if input < 0
            || output < 0
            // SAFETY: integer-only validity check; no pointer is dereferenced.
            || unsafe { libc::fcntl(input, libc::F_GETFD) } < 0
            // SAFETY: same integer-only validity check for the output descriptor.
            || unsafe { libc::fcntl(output, libc::F_GETFD) } < 0
        {
            return REPLAI_INVALID_ARGUMENT;
        }
        // SAFETY: validated descriptors stay live for this call.
        let input = unsafe { BorrowedFd::borrow_raw(input) };
        // SAFETY: same validated descriptor lifetime contract.
        let output = unsafe { BorrowedFd::borrow_raw(output) };
        // SAFETY: caller owns this serialized handle.
        unsafe {
            with_handle(h, |h| {
                if h.interaction.is_open() || h.output.is_some() {
                    return REPLAI_INVALID_STATE;
                }
                match replai::OutputSession::open(&input, &output) {
                    Ok(o) => {
                        h.output = Some(o);
                        REPLAI_OK
                    }
                    Err(e) => error(e),
                }
            })
        }
    })
}
/// Replace or clear bounded feedback without appending progress to scrollback.
/// # Safety
/// Handle and semantic text record obey the extension extent contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_output_feedback(h: *mut Handle, t: *const ReplaiText) -> i32 {
    // SAFETY: caller records are copied; handle is serialized.
    unsafe {
        with_handle(h, |h| {
            let Some(output) = &mut h.output else {
                return REPLAI_INVALID_STATE;
            };
            match extension_record(t).and_then(|t| semantic(&t)) {
                Ok(t) => result(output.feedback(t)),
                Err(e) => e,
            }
        })
    }
}
/// Retire quiet-output scope, explicitly selecting queued-input discard policy.
/// # Safety
/// h is a live serialized handle, including after INTERNAL for cleanup.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_output_close(h: *mut Handle, discard_input: u32) -> i32 {
    guard(|| {
        if let Err(e) = aligned(h) {
            return e;
        }
        if discard_input > 1 {
            return REPLAI_INVALID_ARGUMENT;
        }
        // SAFETY: live exclusive handle; cleanup remains permitted after poisoning.
        let h = unsafe { &mut *h };
        match h.output.as_mut() {
            None => REPLAI_OK,
            Some(o) => match o.close(discard_input == 1) {
                Ok(()) => {
                    h.output = None;
                    REPLAI_OK
                }
                Err(e) => error(e),
            },
        }
    })
}

/// Project producer-authored style for host-owned incremental serialization.
/// # Safety
/// Output buffer follows the exact-size caller-owned buffer contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_role_sequence(
    tag: u32,
    styled: u32,
    buffer: *mut u8,
    capacity: usize,
    required: *mut usize,
) -> i32 {
    guard(|| {
        if styled > 1 {
            return REPLAI_INVALID_ARGUMENT;
        }
        let role = match role(tag) {
            Ok(r) => r,
            Err(e) => return e,
        };
        // SAFETY: copy_text checks the caller's live disjoint buffer/output spans.
        unsafe {
            copy_text(
                Theme::new(styled == 1, false, None).sequence(role),
                buffer,
                capacity,
                required,
            )
        }
    })
}
