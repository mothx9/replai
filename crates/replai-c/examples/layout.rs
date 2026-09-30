//! Executable Rust side of the C/Rust ABI layout comparison.
use replai_c::*;
fn main() {
    println!("REPLAI_C_ABI_VERSION={}", REPLAI_C_ABI_VERSION);
    println!("REPLAI_OK={}", REPLAI_OK);
    println!("REPLAI_INVALID_ARGUMENT={}", REPLAI_INVALID_ARGUMENT);
    println!("REPLAI_INVALID_UTF8={}", REPLAI_INVALID_UTF8);
    println!("REPLAI_INVALID_RANGE={}", REPLAI_INVALID_RANGE);
    println!("REPLAI_CAPACITY={}", REPLAI_CAPACITY);
    println!("REPLAI_INVALID_STATE={}", REPLAI_INVALID_STATE);
    println!("REPLAI_UNSUITABLE_TERMINAL={}", REPLAI_UNSUITABLE_TERMINAL);
    println!("REPLAI_IO={}", REPLAI_IO);
    println!("REPLAI_BUFFER_TOO_SMALL={}", REPLAI_BUFFER_TOO_SMALL);
    println!("REPLAI_ABI_MISMATCH={}", REPLAI_ABI_MISMATCH);
    println!("REPLAI_BUSY={}", REPLAI_BUSY);
    println!("REPLAI_INTERNAL={}", REPLAI_INTERNAL);
    println!("REPLAI_INVALID_TEXT={}", REPLAI_INVALID_TEXT);
    println!("REPLAI_HISTORY_DISABLED={}", REPLAI_HISTORY_DISABLED);
    println!("REPLAI_INVALID_SEQUENCE={}", REPLAI_INVALID_SEQUENCE);
    println!("REPLAI_EVENT_NONE={}", REPLAI_EVENT_NONE);
    println!("REPLAI_EVENT_SUBMITTED={}", REPLAI_EVENT_SUBMITTED);
    println!("REPLAI_EVENT_INTERRUPTED={}", REPLAI_EVENT_INTERRUPTED);
    println!("REPLAI_EVENT_END_OF_INPUT={}", REPLAI_EVENT_END_OF_INPUT);
    println!(
        "REPLAI_EVENT_COMPLETION_REQUESTED={}",
        REPLAI_EVENT_COMPLETION_REQUESTED
    );
    println!("REPLAI_EVENT_EDIT_REJECTED={}", REPLAI_EVENT_EDIT_REJECTED);
    println!("REPLAI_ROLE_DEFAULT={}", REPLAI_ROLE_DEFAULT);
    println!("REPLAI_ROLE_STRONG={}", REPLAI_ROLE_STRONG);
    println!("REPLAI_ROLE_ACCENT={}", REPLAI_ROLE_ACCENT);
    println!("REPLAI_ROLE_DIM={}", REPLAI_ROLE_DIM);
    println!("REPLAI_ROLE_SUCCESS={}", REPLAI_ROLE_SUCCESS);
    println!("REPLAI_ROLE_WARNING={}", REPLAI_ROLE_WARNING);
    println!("REPLAI_ROLE_ERROR={}", REPLAI_ROLE_ERROR);
    println!(
        "REPLAI_PRESENTATION_VERSION={}",
        REPLAI_PRESENTATION_VERSION
    );
    println!("REPLAI_BLOCK_PARAGRAPH={}", REPLAI_BLOCK_PARAGRAPH);
    println!("REPLAI_BLOCK_HEADING={}", REPLAI_BLOCK_HEADING);
    println!("REPLAI_BLOCK_FIELD={}", REPLAI_BLOCK_FIELD);
    println!("REPLAI_BLOCK_LIST={}", REPLAI_BLOCK_LIST);
    println!("REPLAI_BLOCK_LITERAL={}", REPLAI_BLOCK_LITERAL);
    println!("REPLAI_BLOCK_STATUS={}", REPLAI_BLOCK_STATUS);
    println!("REPLAI_BLOCK_SPACER={}", REPLAI_BLOCK_SPACER);
    println!("REPLAI_BLOCK_TABLE_HEADER={}", REPLAI_BLOCK_TABLE_HEADER);
    println!("REPLAI_BLOCK_TABLE_ROW={}", REPLAI_BLOCK_TABLE_ROW);
    println!("REPLAI_APPLIED={}", REPLAI_APPLIED);
    println!("REPLAI_STALE={}", REPLAI_STALE);
    println!("REPLAI_WAKE_INPUT={}", REPLAI_WAKE_INPUT);
    println!("REPLAI_WAKE_RESIZE={}", REPLAI_WAKE_RESIZE);
    println!("REPLAI_WAKE_DEADLINE={}", REPLAI_WAKE_DEADLINE);
    println!("REPLAI_WAIT_READY={}", REPLAI_WAIT_READY);
    println!("REPLAI_WAIT_INPUT={}", REPLAI_WAIT_INPUT);
    println!("replai_config.size={}", std::mem::size_of::<ReplaiConfig>());
    println!(
        "replai_config.align={}",
        std::mem::align_of::<ReplaiConfig>()
    );
    println!(
        "replai_config.struct_size={}",
        std::mem::offset_of!(ReplaiConfig, struct_size)
    );
    println!(
        "replai_config.abi_version={}",
        std::mem::offset_of!(ReplaiConfig, abi_version)
    );
    println!(
        "replai_config.max_input_bytes={}",
        std::mem::offset_of!(ReplaiConfig, max_input_bytes)
    );
    println!(
        "replai_config.history_entries={}",
        std::mem::offset_of!(ReplaiConfig, history_entries)
    );
    println!(
        "replai_config.reserved={}",
        std::mem::offset_of!(ReplaiConfig, reserved)
    );
    println!("replai_event.size={}", std::mem::size_of::<ReplaiEvent>());
    println!("replai_event.align={}", std::mem::align_of::<ReplaiEvent>());
    println!(
        "replai_event.struct_size={}",
        std::mem::offset_of!(ReplaiEvent, struct_size)
    );
    println!(
        "replai_event.abi_version={}",
        std::mem::offset_of!(ReplaiEvent, abi_version)
    );
    println!(
        "replai_event.kind={}",
        std::mem::offset_of!(ReplaiEvent, kind)
    );
    println!(
        "replai_event.status={}",
        std::mem::offset_of!(ReplaiEvent, status)
    );
    println!(
        "replai_event.text_bytes={}",
        std::mem::offset_of!(ReplaiEvent, text_bytes)
    );
    println!(
        "replai_event.cursor_bytes={}",
        std::mem::offset_of!(ReplaiEvent, cursor_bytes)
    );
    println!(
        "replai_event.reserved={}",
        std::mem::offset_of!(ReplaiEvent, reserved)
    );
    println!("replai_span.size={}", std::mem::size_of::<ReplaiSpan>());
    println!("replai_span.align={}", std::mem::align_of::<ReplaiSpan>());
    println!(
        "replai_span.struct_size={}",
        std::mem::offset_of!(ReplaiSpan, struct_size)
    );
    println!(
        "replai_span.extension_version={}",
        std::mem::offset_of!(ReplaiSpan, extension_version)
    );
    println!(
        "replai_span.text={}",
        std::mem::offset_of!(ReplaiSpan, text)
    );
    println!(
        "replai_span.text_bytes={}",
        std::mem::offset_of!(ReplaiSpan, text_bytes)
    );
    println!(
        "replai_span.role={}",
        std::mem::offset_of!(ReplaiSpan, role)
    );
    println!(
        "replai_span.reserved={}",
        std::mem::offset_of!(ReplaiSpan, reserved)
    );
    println!("replai_text.size={}", std::mem::size_of::<ReplaiText>());
    println!("replai_text.align={}", std::mem::align_of::<ReplaiText>());
    println!(
        "replai_text.struct_size={}",
        std::mem::offset_of!(ReplaiText, struct_size)
    );
    println!(
        "replai_text.extension_version={}",
        std::mem::offset_of!(ReplaiText, extension_version)
    );
    println!(
        "replai_text.spans={}",
        std::mem::offset_of!(ReplaiText, spans)
    );
    println!(
        "replai_text.span_count={}",
        std::mem::offset_of!(ReplaiText, span_count)
    );
    println!(
        "replai_text.reserved={}",
        std::mem::offset_of!(ReplaiText, reserved)
    );
    println!("replai_block.size={}", std::mem::size_of::<ReplaiBlock>());
    println!("replai_block.align={}", std::mem::align_of::<ReplaiBlock>());
    println!(
        "replai_block.struct_size={}",
        std::mem::offset_of!(ReplaiBlock, struct_size)
    );
    println!(
        "replai_block.extension_version={}",
        std::mem::offset_of!(ReplaiBlock, extension_version)
    );
    println!(
        "replai_block.kind={}",
        std::mem::offset_of!(ReplaiBlock, kind)
    );
    println!(
        "replai_block.level={}",
        std::mem::offset_of!(ReplaiBlock, level)
    );
    println!(
        "replai_block.text={}",
        std::mem::offset_of!(ReplaiBlock, text)
    );
    println!(
        "replai_block.label={}",
        std::mem::offset_of!(ReplaiBlock, label)
    );
    println!(
        "replai_block.cells={}",
        std::mem::offset_of!(ReplaiBlock, cells)
    );
    println!(
        "replai_block.cell_count={}",
        std::mem::offset_of!(ReplaiBlock, cell_count)
    );
    println!(
        "replai_block.reserved={}",
        std::mem::offset_of!(ReplaiBlock, reserved)
    );
    println!("replai_render.size={}", std::mem::size_of::<ReplaiRender>());
    println!(
        "replai_render.align={}",
        std::mem::align_of::<ReplaiRender>()
    );
    println!(
        "replai_render.struct_size={}",
        std::mem::offset_of!(ReplaiRender, struct_size)
    );
    println!(
        "replai_render.extension_version={}",
        std::mem::offset_of!(ReplaiRender, extension_version)
    );
    println!(
        "replai_render.columns={}",
        std::mem::offset_of!(ReplaiRender, columns)
    );
    println!(
        "replai_render.indent={}",
        std::mem::offset_of!(ReplaiRender, indent)
    );
    println!(
        "replai_render.styled={}",
        std::mem::offset_of!(ReplaiRender, styled)
    );
    println!(
        "replai_render.reserved={}",
        std::mem::offset_of!(ReplaiRender, reserved)
    );
    println!(
        "replai_candidate.size={}",
        std::mem::size_of::<ReplaiCandidate>()
    );
    println!(
        "replai_candidate.align={}",
        std::mem::align_of::<ReplaiCandidate>()
    );
    println!(
        "replai_candidate.struct_size={}",
        std::mem::offset_of!(ReplaiCandidate, struct_size)
    );
    println!(
        "replai_candidate.extension_version={}",
        std::mem::offset_of!(ReplaiCandidate, extension_version)
    );
    println!(
        "replai_candidate.start={}",
        std::mem::offset_of!(ReplaiCandidate, start)
    );
    println!(
        "replai_candidate.end={}",
        std::mem::offset_of!(ReplaiCandidate, end)
    );
    println!(
        "replai_candidate.insertion={}",
        std::mem::offset_of!(ReplaiCandidate, insertion)
    );
    println!(
        "replai_candidate.insertion_bytes={}",
        std::mem::offset_of!(ReplaiCandidate, insertion_bytes)
    );
    println!(
        "replai_candidate.label={}",
        std::mem::offset_of!(ReplaiCandidate, label)
    );
    println!(
        "replai_candidate.label_bytes={}",
        std::mem::offset_of!(ReplaiCandidate, label_bytes)
    );
    println!(
        "replai_candidate.annotation={}",
        std::mem::offset_of!(ReplaiCandidate, annotation)
    );
    println!(
        "replai_candidate.annotation_bytes={}",
        std::mem::offset_of!(ReplaiCandidate, annotation_bytes)
    );
    println!(
        "replai_candidate.reserved={}",
        std::mem::offset_of!(ReplaiCandidate, reserved)
    );
    println!(
        "replai_interest.size={}",
        std::mem::size_of::<ReplaiInterest>()
    );
    println!(
        "replai_interest.align={}",
        std::mem::align_of::<ReplaiInterest>()
    );
    println!(
        "replai_interest.struct_size={}",
        std::mem::offset_of!(ReplaiInterest, struct_size)
    );
    println!(
        "replai_interest.extension_version={}",
        std::mem::offset_of!(ReplaiInterest, extension_version)
    );
    println!(
        "replai_interest.kind={}",
        std::mem::offset_of!(ReplaiInterest, kind)
    );
    println!(
        "replai_interest.input_fd={}",
        std::mem::offset_of!(ReplaiInterest, input_fd)
    );
    println!(
        "replai_interest.timeout_ms={}",
        std::mem::offset_of!(ReplaiInterest, timeout_ms)
    );
    println!(
        "replai_interest.reserved={}",
        std::mem::offset_of!(ReplaiInterest, reserved)
    );
    println!(
        "replai_interest.deadline_ticket={}",
        std::mem::offset_of!(ReplaiInterest, deadline_ticket)
    );
}
