# Interaction contract

This document owns behavior shared by the Rust and C interfaces. Individual
Rust methods are documented in rustdoc (`cargo doc --no-deps`); the
[C contract](c-api.md) owns ABI representation and mechanical C ownership.
[Architecture](architecture.md) explains the implementing modules.

## Text and editing boundaries

Storage and capacities are **UTF-8 bytes**. Every public cursor/range endpoint
must lie on an **extended grapheme boundary**, as determined by
`unicode-segmentation`. Left/Right, Backspace/Delete operate on those clusters.
Insertion/deletion can join adjacent clusters; the cursor advances to the next
valid boundary after the edit. Invalid ranges, controls and capacity overflow
leave the original text and cursor unchanged.

Home/End and Ctrl-A/Ctrl-E refer to the whole input, including multiple logical
lines. Up/Down navigate history, not vertical columns. Input is neither trimmed
nor normalized as Unicode. LF and TAB are accepted text; other Unicode controls
are rejected. Bracketed paste performs the separate newline normalization below.

The [presentation contract](presentation.md#prompt-cells-and-redraw) separately
defines terminal cells, ANSI width, tab expansion and emulator limits.

## Bounded input and paste

The decoder consumes bytes in order from bounded transport reads (currently
4 KiB). An advancement stops at the first host-visible event, a drained short read, or
the ready-work budget. Full buffers may continue with zero-wait reads, up to
32 KiB; a 2 ms active-work budget is checked after semantic actions. Neither
budget inserts a collection delay or interrupts one atomic editor operation.
UTF-8 staging holds at most four bytes;
escape staging at most 64. Supported CSI/SS3 sequences cover arrows, Home/End,
Delete and bracketed-paste delimiters; listed control keys include Enter,
Ctrl-A/C/D/E/K/L/R/S/U/W/Y/_, Backspace and Tab. Recognized Meta bindings are
Alt-B, Alt-F, Alt-D and Alt-Backspace. Unknown sequences are rejected. Oversized CSI
or OSC sequences drain to their terminator with constant extra memory. A bad
UTF-8 byte and the incomplete scalar containing it are rejected together; the
offending byte is not replayed as a shortcut.

The adapter expires pending sequences after a 250 ms idle interval, observed
by the selected scheduler: compatibility polling or an explicit host deadline.
Each compatibility poll waits at most 100 ms. This is an idle bound, not a
fixed total sequence length/time guess; tests deliver each byte with a delay
longer than 25 ms. Expired UTF-8/incomplete escape input produces `Event::Rejected` and
keeps the draft. A lone Escape dismisses an active completion or history-search surface; without
one it retains the same rejection. Host starvation can delay observation.

Already-ready semantic edits may share one presentation update. Completion,
submission, interruption, EOF, rejection and Ctrl-L remain observable boundaries;
required output completes before the event returns. A drained short read is
presented before probing for more input, preserving isolated-key responsiveness.
Read-ahead after an event is retained by the Interaction, bounded to one read,
including across close/reopen and closed-editor changes. It is not replayed into
the OS queue. Destroying the Interaction discards its already-consumed read-ahead;
hosts reusing type-ahead should retain the same Interaction/opaque C handle.

Bracketed paste stages one atomic payload, bounded by the editor byte limit,
and recognizes fragmented begin/end markers. CRLF becomes one LF; lone CR and
LF become LF. TAB remains literal text, not completion. Other control characters
reject the **entire** paste; they never execute shortcuts. Oversized payloads
are drained through their end marker, then rejected. The wire-byte bound is
applied before newline normalization, and insertion separately checks remaining
draft capacity. Invalid UTF-8 is rejected. No partial paste is committed.

An incomplete paste or physical EOF during a pending sequence produces a terminal
I/O error and closes/restores the interaction. The host must treat this as a
failed input transaction; automatically reopening on an untrusted remaining
byte stream would erase the framing distinction. Enter outside paste submits
one complete string. Unbracketed LF/CR each mean Enter; automatic detection of
unframed multi-command paste is not promised.

## Terminal and signal ownership

The following resource contract describes the shared Linux/macOS POSIX system façade. The
platform-neutral engine has no resource lease or OS signal policy.

`Interaction::open` validates TTY input and output and requires the same terminal
before changing state. It duplicates the FDs, captures full termios and window
dimensions, enters raw mode with ISIG disabled, enables bracketed paste and draws.
No input queue is flushed. One active system terminal per linked library image is admitted;
other opens fail before mutation. The small atomic lease prevents competing
editors but does not install a process-wide signal policy.

Submit, empty-buffer Ctrl-D/read EOF, interruption and explicit close restore
**exactly the captured termios**, not a guessed cooked mode. Read/write errors,
partial acquisition and ordinary unwinding also attempt restoration. Bracketed
paste is disabled and styling reset during cleanup. A failed output FD does not
skip termios restoration; cleanup also tries the same-terminal input FD when
writable. Explicit errors report cleanup failure along with the original error.
Drop is best effort and never panics or exits the process. Disconnected terminals
may reject restoration syscalls, and no library can promise cleanup on SIGKILL,
abort or process termination that bypasses unwinding.

No handlers, signal masks, signal threads or cancellation threads are installed,
so there is no previous handler state to replace or restore. Keyboard Ctrl-C is
a decoded byte yielding `Interrupted`, with a visible `^C` at the end of the
editing surface. OS SIGINT/SIGTERM/SIGTSTP policy stays with the host. A host may
observe its own signals and call `interrupt` or `close`; suspension requires
closing before suspension and opening again after resumption. Resize is observed
by reading current dimensions on each poll, independent of SIGWINCH handlers;
it works even when the host owns or blocks SIGWINCH. Concurrent direct writers
or externally changing terminal modes during an interaction are unsupported.

## Host lifecycle and events

`Interaction` uses the independently owned composition described in
[architecture](architecture.md#composition-and-public-boundaries). `editor()`
exposes current text and byte cursor; `editor_mut()` permits direct host edits only while closed.
Active edits go through validated completion/output operations so display state
cannot become stale. Terminal closure does not clear the editor or admit history.
The host can retain rejected/interrupted/submitted input and choose its next step.

- `Event::Submitted(String)`: Enter; terminal restored.
- `Event::Interrupted`: decoded Ctrl-C or explicit host call; terminal restored.
- `Event::EndOfInput`: physical EOF or Ctrl-D with empty text; terminal restored.
  Ctrl-D with nonempty text deletes the next grapheme (no-op at end).
- `Event::CompletionRequested`: Tab; active draft remains available.
- `Event::Rejected(EditError)`: recoverable input/capacity failure; editing continues.
- `Error::State`: operation unavailable in the current lifecycle.
- `Error::Busy`: another interaction owns the terminal lease.
- `Error::UnsuitableTerminal`: non-TTY, terminal mismatch or unsupported dimensions.
- `Error::Edit`: invalid host edit/output text; unchanged draft, interaction active.
- `Error::Io`: terminal failure; restoration attempted, interaction unavailable
  after successful restoration. A failed restoration is reported.

## Completion

Completion has no callback registry or candidate type. The host reads text and
cursor, discovers and selects candidates, and calls `complete(range, text)` for
one chosen replacement. Range endpoints must be ordered grapheme boundaries.
Zero candidates, ambiguity, refusal or lookup failure need no mutation. The
same atomic validation applies to Unicode and oversized replacements.

## Revision-aware host analysis

`Editor::analysis_snapshot()` and `Interaction::analysis_snapshot()` return one
coherent `AnalysisSnapshot`: private revision, immutable text and byte cursor,
exposed by accessors. Creation copies text once into `Arc<str>`; cloning shares
that allocation. The snapshot is Send + Sync and survives live edits. The host
owns parsing, derived payloads, context, scheduling and cancellation. REPLAI
runs no analysis callbacks, jobs, futures or cache.

`DraftRevision` supports copy and equality within its originating retained
Editor. It has no public arithmetic, ordering or global uniqueness. A host with
several editors must route each result to the originating editor; replacing an
Editor creates a new domain. Equality between unrelated domains is meaningless.

| Transition | Revision rule |
| --- | --- |
| Successful text or cursor change | New identity, including history recall/return, paste and completion |
| No-op movement/replacement/clear; rejected edit | Retain identity |
| History admission alone | Retain identity; stored history is not snapshot context |
| Submit, editing interrupt, transport EOF or empty Ctrl-D | End the semantic draft and advance identity, even if retained bytes are unchanged |
| Close/reopen, including a different prompt | Retain identity if the draft is unchanged |
| Resize, output, theme, capability, readiness or partial decoder state | Retain identity unless an actual editor transition results |

Returning to earlier bytes/cursor after an intervening change never revives old
analysis. Paste is one atomic editor insertion after successful decoding, not a
revision per protocol byte. The private 128-bit counter uses checked increments;
on exhaustion a state-changing operation panics **before mutation**, never wraps.
No snapshot allocation occurs on ordinary editing paths.

Use `Editor::replace_at(revision, range, text)` or the active native
`Interaction::complete_at(revision, range, text)` to compare, validate and apply
under one exclusive mutable borrow. `AnalysisOutcome::Stale` performs no edit,
history change, redraw or terminal I/O. It is checked before range/text validation.
`Applied` admits a current valid replacement, including a no-op; malformed current
edits retain the existing edit errors. Closed interaction application returns
`Error::State`. A current edit followed by an I/O failure has the existing cleanup
semantics: the edit committed, terminal cleanup is attempted, and the error is
reported. A snapshot is not a transaction rollback mechanism.

Unversioned `complete` remains available for immediate synchronous completion and
C ABI 1. I0 adds no C symbols; C callers retain their synchronous contract. Native
blocking callers need not use snapshots. Session and driven hosts can retain a
snapshot, continue input/deadline/resize/output work, then serially apply a result.
The [external-reactor example](../examples/analysis.rs) demonstrates that flow;
[qualification](engineering/analysis-protocol.md) separates portable and real PTY
proof. Completion, validation, analysis presentation and reversible edits all
reuse this identity; none restores an earlier revision.

## Semantic editing and reversible draft

The terminal decoder recognizes physical keys and paste framing only. `KeyMap`
translates public, normalized `Key` values into the public semantic `Action`
vocabulary; the engine applies those actions to the sole `Editor`. `NamedKey`,
`Control` and `Meta` describe portable input meaning, never escape bytes.
Printable text and bracketed paste remain text transports and cannot be rebound.
Portable `Editor` methods expose word operations, undo/redo and kill/yank without
requiring callers to synthesize terminal bytes.

`KeyMap::new()` installs the qualified compatibility profile. While an
`Interaction` is closed, a host may inspect mappings, bind or replace an action,
unbind one key, reset it to its default, or reset the complete profile. At most
128 custom overrides are retained in a sorted vector; lookup is allocation-free
and bounded `O(log 128)`. Duplicate binds replace atomically. Invalid control or
Meta values and a 129th override return typed `KeyMapError`s. Active mutation is
refused because the terminal driver owns an immutable map clone for the session.
`Interaction::apply_action` sends a semantic action through the same engine and
render path as decoded input; it is available only on native open interactions.

Word movement and deletion use locale-independent Unicode default word boundaries
from `unicode-segmentation` 1.13.3 (UAX #29). A semantic boundary that falls
inside an extended grapheme is snapped outward, so combining sequences, emoji
ZWJ sequences and regional indicators are never split. Movement skips punctuation,
spaces, tabs and LF separators to the previous word start or following word end;
CJK word segments follow the dependency's Unicode tables. A real move advances
`DraftRevision`; a boundary no-op does not. Word deletion is one atomic undo
transaction.

Undo records are deltas: byte start, removed/inserted UTF-8 and cursor before/after.
They never store a revision or clone the complete draft for an ordinary local
edit. Undo and redo each retain at most 256 records and, by default, at most twice
the configured draft capacity in payload bytes. `Editor::with_limits` accepts
explicit `EditorLimits`; `Editor::new(max_bytes, history_entries)` preserves its
existing signature and uses those defaults. Contiguous typing and repeated
Backspace/Delete coalesce up to 4096 payload bytes. Cursor movement, requests and
atomic replacements end a group. A divergent edit clears redo; cursor-only
movement preserves it. Undo/redo restores content and cursor with a fresh
revision, so every old host result remains stale forever.

Completion acceptance, revision-bound replacement, paste, validated continuation
indentation, word deletion, kill, yank and accepted history search use the same
transaction path. `clear`, semantic submit, interrupt and EOF end the logical
draft and clear undo/redo so a prior command cannot reappear. Closing and reopening
the terminal without ending the draft preserves its undo state.

The editor owns one bounded kill register, not a clipboard or Readline kill ring.
It supports killing one word backward/forward or from the cursor to the logical
line start/end, plus yank. Each successful kill replaces the register; consecutive
kills do not concatenate. The default register limit is the smaller of draft
capacity and 64 KiB and is configurable through `EditorLimits`. A failed kill or
yank changes no draft, cursor, revision, undo/redo or prior register. The register
survives clear, submission and close/reopen for a retained `Editor`; the explicit
clear method provides the future sensitive-input boundary a deterministic purge.

Compatibility bindings are Ctrl-_ undo, Alt-B/F word movement, Alt-D word-delete
forward, Alt-Backspace word-delete backward, Ctrl-W word-kill backward, Ctrl-U/K
kill to line start/end and Ctrl-Y yank. Redo and forward word-kill remain available
through `Editor` methods but intentionally have no fixed terminal binding before
I5. Redo and forward word-kill are now bindable without acquiring a terminal.
PageUp/PageDown clamp completion by one visible page. Bracketed paste always
arrives as literal text in one transaction; embedded control bytes never invoke
configured actions.

## Completion helpers, large sets and suggestions

`CompletionSet` remains the low-level revision-bound protocol. Optional helpers
only construct valid sets from host-supplied snapshots, replacement ranges and
sources. `complete_prefix` preserves caller order and duplicates with selectable
case-sensitive or ASCII-insensitive matching. `common_grapheme_prefix` ends at an
extended-grapheme boundary. `complete_fuzzy` uses deterministic Unicode-scalar
subsequence matching, ordered by earliest byte start, then fewer byte gaps, then
caller order; it makes no semantic-ranking claim.

`complete_path` reads one directory level. Relative input resolves under an
explicit base, absolute input remains absolute, results sort by insertion text,
directories gain the platform separator and shell quoting, environment expansion
and execution remain host policy. Hidden entries are optional. Missing or denied
directories fail atomically; non-UTF-8 entry names are skipped rather than
lossily inserted. Helpers inspect at most 4096 source items, accept at most a
4096-byte query and remain inside the existing 4096-candidate, 65,536-byte field
and 4 MiB aggregate `CompletionSet` bounds.

Completion `Next`/`Previous` wrap. `PageNext`/`PagePrevious` move by the current
visible-window size and clamp; `First` and `Last` select the endpoints. Resizing
changes only the visible slice, never candidate identity or selected index. The
menu reports selection and visible range in text, so plain mode remains legible.
Installation validates the full set; navigation retains only an index/window and
does not clone or rescan candidate payloads.

`Suggestion` is one nonempty, control-safe suffix of at most 4096 bytes for an
exact end-of-draft `DraftRevision`. It is distinct from informational `Hint`:
display does not make it canonical. Stale delivery returns `AnalysisOutcome::Stale`
without mutation or output. Acceptance inserts the suffix atomically as one undo
transaction with a fresh revision; dismissal changes neither draft nor revision.
`suggest_from_static` and `suggest_from_history` inspect caller-ordered bounded
sources and return the remaining suffix of the first whole-draft prefix match.
History persistence and scheduling remain host-owned.

The default Right action accepts a visible suggestion only when the cursor is at
the draft end; hosts may bind an explicit `SuggestionAction`. Reverse search owns
the temporary frame, completion suppresses suggestion, invalid diagnostics suppress
suggestion, and a suggestion suppresses an analysis hint. Hidden current state may
return after dismissal; every canonical edit invalidates it. Resize and serialized
host output preserve a current suggestion. No suggestion may enter history or
submitted bytes before explicit acceptance.

## History

History is configured by entry count and input byte bound. Admission is explicit,
with oldest-entry eviction only when the host-selected bound is full. No hardcoded
history size, persistence, deduplication or privacy policy exists. First Up saves
the current draft **and cursor**; Down past the newest entry restores both.
Editing a recalled entry changes only the current edit. Navigating away discards
that recalled edit, without modifying admitted history.

`HistoryProvider` is a synchronous, newest-first loading seam for storage that
remains host-owned. The host invokes `HistorySearchSource::from_provider` outside
the input hot path; REPLAI calls at most the configured entry count and copies at
most the configured retained-byte bound into an immutable search view. The
provider is never retained and cannot receive an `Interaction`, terminal resource
or callback through this API. Database/network I/O must be scheduled by the host;
provider failure, invalid control text or an oversized entry rejects construction
atomically. Existing admitted in-memory history works without a provider and is
searched first; both sources share the same reverse-search mechanics.

Ctrl-R begins reverse incremental search and then selects older matches; Ctrl-S
selects a newer match without wrapping. Matching is case-sensitive literal
substring matching over UTF-8 strings, newest first, with duplicates retained.
The default bounds are 1024 inspected/retained entries, 1 MiB retained entry
payload and a 4096-byte query. Query text and the selected match are derived
presentation: the canonical draft, cursor and revision do not change. Backspace,
Ctrl-W and Ctrl-U edit the query. Enter accepts the selected entry as one atomic,
undoable whole-draft replacement at its end and always creates a fresh revision;
Escape dismisses and restores the exact pre-search draft/cursor without mutation.

Search owns interactive selection while active, so beginning it dismisses a
completion menu and cancels pending validation. Current analysis spans remain
available behind the search frame and reappear after dismissal. A stale completion,
validation or analysis result remains stale and silent. Resize and serialized host
output rebuild the bounded search frame without losing query, match, draft or
cursor. Search ends on accept, dismiss, submit lifecycle, interrupt, EOF or close.

## Coordinated output

The [presentation contract](presentation.md#external-output) owns visible
output transactions and their limits. Host execution may instead write after
submission closes the terminal, then reopen for the next draft. Neither path
installs an application scheduler.

## Embedding tiers and wait ownership

All three native tiers retain one `Interaction`, editor, decoder, renderer and
resource lifecycle. System methods exist on Linux/macOS; the scheduling values
and capability facts compile on every portable core target.

| Tier | Entry | Host responsibility | Deliberate scope |
| --- | --- | --- | --- |
| Simple blocking | `read_line(Prompt)` | Match `ReadOutcome`, execute application code, admit history, clear/retain editor, repeat if wanted | stdin/stdout, no completion discovery; Tab declined; recoverable rejection produces safe warning feedback and editing continues |
| Explicit session | Existing `open` / `poll` / output / completion / close | Dispatch events and decide when to poll | Compatibility scheduler observes dimensions each call, waits at most 100 ms and shortens its wait for a pending internal deadline |
| Driven | `open_driven` or `open_with_config`, `wait_interest`, `advance` | Wait on terminal/application sources and deadlines; deliver resize; serialize mutations | No internal blocking wait, periodic timer, signal handler, thread or executor |

`ReadOutcome::{Submitted, Interrupted, EndOfInput}` returns after restoration;
it never means evaluate, cancel application work or exit the process. The
retained reader preserves history and read-ahead. `editor_mut()` is available
between reads. Fatal errors attempt restoration using the same driver as the
session tier. Drop remains best effort; explicit close reports/retries cleanup.

`WaitInterest::Ready` means the interaction already holds unread input: advance
without waiting for another OS readiness edge. `WaitInterest::Input { deadline }`
means wait for readability and, if present, the monotonic due time. With `None`
there is **no required periodic REPLAI wake**. Interest inspection performs no
I/O, allocation or geometry query. Host application and resize events remain
independent wake sources. Obtain a fresh interest after each operation.

`Wake::InputReady` drains a bounded ready burst with zero-duration readiness
checks and the same observable event ordering described above. Spurious
readiness is normal no-progress. REPLAI is the sole reader of the terminal;
a competing reader invalidates the readiness/read assumption. REPLAI never sets caller file
status flags and O_NONBLOCK is not required. OS bookkeeping on ordinary writes
is separate from host-selected file modes.

`Deadline` is an opaque token with `at() -> Instant`. It identifies one active
resource session and input epoch. Early, superseded and previous-session tokens
are no-ops. A due notification first reconciles input already queued, then
expires pending protocol state if still due. Successful expiry removes that
deadline, so replay does not repeat its outcome. Time is monotonic, never UTC.
The engine itself still has no clock or protocol timeout policy.

`Wake::Resize` asks the owned resource for its dimensions and applies the same
layout/render transition immediately. The library does not install SIGWINCH;
the host converts its own notification into this ordinary serialized call.
Input readiness does not probe dimensions. The compatibility scheduler retains
periodic observation; its idle wakeup behavior is **not** the driven contract.

### POSIX borrowing and serialization

`input_source()` returns `BorrowedFd<'_>` tied to the active interaction. It does
not transfer ownership, keep a closed resource alive or assign restoration to
the host. A live Rust borrow prevents closing/mutating the interaction. Hosts
which copy the descriptor into an OS registration must unregister before close
or reopen: an integer in a reactor is not a Rust lifetime. Never read this
source directly, change its modes or hand it to another terminal owner.

The public values and `Interaction` satisfy Send/Sync auto traits; that does not
permit concurrent mutations. Methods require exclusive mutable access and the
host serializes advancement, completion, interruption and synchronous output.
The examples use one thread. External socket/timer readiness can trigger
`output_document` while a draft exists, preserving exact bytes/cursor. This is
composable event waiting, **not concurrent independent writers**, background
queues or an O2 output service. The current backend's single active terminal
lease remains at resource acquisition.

## Terminal admission and degradation

All tiers share the [capability contract](#terminal-capabilities) below. New
simple/driven defaults use conservative TERM hints; legacy session/C entries
retain an explicit VT compatibility assumption. Neither profile bypasses native
resource verification. See the [simple example](../examples/simple.rs),
[session example](../examples/demo.rs), [external reactor](../examples/driven.rs)
and [embedding qualification](engineering/embedding.md).

## Terminal capabilities

Admission is one contract for every tier, resolved at acquisition rather than per
key, frame or idle query.

| Layer | Public representation | Authority |
| --- | --- | --- |
| Protocol evidence | `TerminalFacts`, `FeatureSupport` | Host evidence/assumption, or TERM hint |
| Resource evidence | `TerminalRealization` | Native acquisition; explicit assertions for virtual resolution |
| Required mechanics | `InteractionRequirements` | Presentation, Editing or Driven |
| Optional policy | `TerminalConfig`, `FeaturePolicy` | Disabled / Preferred / Required, independently for styling and paste |
| Resolved contract | `TerminalCapabilities`, `Degradation` | Original facts plus enabled features and loss reasons |

`Interaction::capabilities()` copies the active snapshot without I/O. It contains
no handles or ownership. Closing invalidates the accessor (`Error::State`); a
previous copy remains historical. Dimensions are last observed at acquisition or
refresh, not continuously discovered.

### Minimum and precedence

Editing requires matching interactive input/output, restorable input mode, at
least two columns and two rows, cursor/erase mechanics and backend readiness.
The cursor profile includes CR/LF, relative/absolute positioning, conventional
cell advance, autowrap and scrolling. Erase includes line/range clearing and
clear-to-home for Ctrl-L. These are protocol **assumptions**, not discoveries made
by `isatty`. Driven additionally requires a host waitable source. Presentation
requires none of these editing mechanics; the document supplies its layout width.

1. Native acquisition verifies resources and obtains dimensions before raw mode.
   Configured protocol support cannot override non-TTY/mismatched endpoints or
   missing dimensions. Refusals emit no prompt/mode bytes.
2. Explicit `TerminalFacts` replace protocol hints. `Supported` is affirmative
   evidence supplied by the host, `Assumed` an accepted hypothesis, `Unknown`
   insufficient evidence and `Unavailable` an explicit negative.
3. `TerminalConfig::from_environment` reads TERM once as a hint: nonempty/non-dumb
   produces `Assumed`; absent/empty/dumb produces `Unknown`. NO_COLOR presence,
   including empty, is `Disabled` styling **policy**, not a negative color fact.
   Replacing protocol hints later does not remove that captured policy.
4. Optional policy resolves independently. An explicitly plain theme also vetoes
   styling. Missing/vetoed `Required` features fail. Unavailable `Preferred`
   features report `Unavailable` degradation; intentional disablement reports
   `Disabled`. Facts themselves are not rewritten.

Fully explicit configuration does not reread environment variables. Hosts own
that choice; starting from `from_environment` retains user policy. `Theme::new`
and `from_environment` remain compatibility projections through the capability
module rather than independently discovering terminal truth.

### Resolution and degradation

`TerminalConfig::resolve_for(realization, requirements)` is pure and allocation-free.
Native opens obtain their own resource observations. Public
`TerminalRealization::interactive((columns, rows))` instead affirms controlled
host/virtual facts; it does not probe an OS. Use `Presentation` and
`TerminalRealization::captured()` to resolve a plain theme without input/cursor
requirements. The old `resolve()` checks only protocol policy and does not prove
resource suitability. `open_with_config` admits the Driven requirements; ordinary
`open` and blocking `read_line` admit Editing. This does not select a scheduler.

Styling loss removes SGR for both prompt and structured output, preserving text
and hierarchy. The current palette needs intensity and 256-color foregrounds;
a host with weaker support can conservatively decline styling.

Paste loss suppresses enable/disable sequences, including output and cleanup.
Ordinary Unicode editing still works. **Unframed multiline paste is not atomic:**
Enter can submit the first line. Already-read trailing bytes survive close/reopen;
a host requiring atomic paste must require that feature. Framed input received
anyway still goes through the bounded, safe paste decoder. Paste policy is
irrelevant to Presentation requirements, which never enable input modes.

`Readiness` distinguishes absent support, backend-managed waiting and a host
waitable source; it contains no FD/HANDLE. POSIX borrowed FDs realize the latter.
Dimensions, `dimension_query` and resize delivery are independent:
`ResizeDelivery::PollOrHostNotification` means poll queries periodically, while
driven callers notify with `Wake::Resize`. REPLAI installs no signals or periodic
driven wake. A controlled fixed surface can resolve to `Fixed` while still being
waitable. Opaque decoder deadlines remain runtime state, not a capability.

### Compatibility and limits

`open`, `open_with_theme` and C ABI 1 use `TerminalConfig::compatibility` internally.
This retains their VT assumption under TERM=dumb, with styling suppressed by the
environment theme. It is not a claim that every dumb terminal supports VT.
Blocking/driven defaults refuse that hint unless given stronger host evidence.

Missing native dimensions now reports `Error::CapabilityMismatch` rather than
`UnsuitableTerminal`, naming the required geometry before any raw mode. Non-TTY
and mismatched endpoints retain `UnsuitableTerminal`; I/O errors retain cleanup
context. C maps both admission categories to its existing unsuitable status;
ABI 1 declarations, records, symbols and numeric identity remain unchanged.
Capability snapshots remain native-only, an explicit E3 cross-language review item.

Linux/macOS share one realization; Windows runs portable model tests without a
terminal backend. No active probing, alternate editor or emulator database is
introduced. [Width policy](presentation.md#display-width-contract) ·
[Profile evidence](engineering/terminal-capabilities.md).

## Revision-bound completion candidates

Native Rust session/driven hosts may return a `CompletionSet` from one
`AnalysisSnapshot`. Candidate discovery, filtering, order, ranking and context
freshness remain host responsibilities. REPLAI owns temporary selection and
atomic application. No provider callback, worker, executor or analysis queue is
installed. The smallest `read_line` tier continues to decline completion requests.

```rust
use replai::{AnalysisOutcome, CompletionCandidate, CompletionSet, Interaction};
# #[cfg(any(target_os = "linux", target_os = "macos"))]
fn deliver(interaction: &mut Interaction) -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = interaction.analysis_snapshot();
    // The host supplies meaning, replacement range and candidate order.
    let candidates = vec![
        CompletionCandidate::new(0..snapshot.text().len(), "build ", "build")?
            .with_annotation("Build the project")?,
        CompletionCandidate::new(0..snapshot.text().len(), "bundle ", "bundle")?
            .with_annotation("Produce a bundle")?,
    ];
    match interaction.present_completions(CompletionSet::new(snapshot.revision(), candidates)?)? {
        AnalysisOutcome::Applied => {},
        AnalysisOutcome::Stale => { /* host discards or schedules fresh discovery */ },
    }
    Ok(())
}
```

Each candidate has its own grapheme-aligned UTF-8 byte range, replacement,
nonempty display label and optional annotation. Display and insertion need not
match. Labels/annotations reject all control characters, including LF/TAB;
replacement admits editor LF/TAB. Candidate fields additionally reject bidi
formatting controls and invisible direction/word markers; ZWJ, variation selectors
and combining marks remain valid Unicode. No field accepts terminal escape control.

Limits are 4,096 candidates, 65,536 bytes per field and 4 MiB combined retained
text, plus bounded candidate metadata. Constructors validate before copying;
installation checks every range and resulting editor capacity before activating
anything. No partial activation or silent semantic truncation occurs. Host order
and exact duplicates are preserved. `CompletionError` distinguishes limits,
invalid display text, edit rejection and ordinary interaction/terminal errors.

`present_completions` first checks lifecycle and revision. A stale set returns
`AnalysisOutcome::Stale` without terminal bytes, rendering, editor mutation or
replacement of an existing valid menu. Zero candidates silently dismiss current
presentation. Every nonempty set, including one candidate, requires acceptance.
This keeps delayed delivery from unexpectedly inserting text.

| Active candidate surface | Operation |
| --- | --- |
| Tab | Select next candidate, wrapping in host order |
| Shift-Tab | Select previous candidate, wrapping |
| Enter | Accept selected candidate; a subsequent Enter submits |
| Escape | Dismiss after the existing 250 ms lone-Escape decoder deadline |
| Up/Down | Existing history navigation; a changed draft dismisses candidates |
| Left/Right, Home/End, insertion/deletion, paste | Existing editing; revision changes dismiss candidates |
| Ctrl-L, resize, serialized external output | Redraw/restore current candidates and selection |
| Ctrl-C, EOF, close, Drop | End the terminal surface and release candidate storage |

`completion_action(Next/Previous/Accept/Dismiss)` provides the same serialized
operations to the host. With no menu it returns `Error::State`.
`completion_selection()` exposes revision, zero-based index and count, with no
resource ownership. Outside an active menu, Tab returns `CompletionRequested`
except for the validated continuation-indentation rule below; lone Escape and Shift-Tab retain sequence-rejection
behavior. Paste contents never invoke menu shortcuts.

Navigation and dismissal do not mutate Editor or advance revision. Acceptance
checks the current revision and uses the existing atomic range replacement;
it dismisses the menu even if the replacement is an I0 no-op. No-op acceptance
retains revision; a real edit advances it once. Failed edits and no-op cursor
commands preserve a still-current menu. External output and resize preserve
selection and revision. Explicit close/reopen preserves I0 draft identity but
ends temporary candidate presentation. Read-ahead retains the existing bounded
ownership and event ordering.

Two results for the same draft revision are both current. Explicit deliveries
replace presentation in **delivery order**, starting selection at index zero.
REPLAI does not infer latest-request preference: hosts must discard superseded
jobs or changed application context before delivery. No second request counter
or hidden common-prefix/fuzzy expansion is introduced.

See the [runnable session host](../examples/completion.rs),
[external reactor](../examples/completion-driven.rs),
[presentation policy](presentation.md#completion-surface) and
[qualification dossier](engineering/completion-contract.md).
C ABI 1 retains synchronous request/replacement only; rich candidates are native
Rust, with a future cross-language design reserved for E3.


## Validated submission and multiline navigation

`SubmissionPolicy::Direct` remains the default. Opt into `Validated` with
`set_submission_policy` while closed; the policy survives reopen. The tiny
`read_line` entry requires Direct and refuses a validated configuration before
acquisition. Session and driven paths share the same validation engine.

Enter emits `Event::SubmissionRequested(AnalysisSnapshot)` and leaves editing
active. The host computes and delivers `ValidationResult` through
`apply_validation`. The result contains the originating DraftRevision and one
`ValidationDisposition`: Complete, Incomplete or Invalid. No parser or callback
is installed. See the [runnable validator](../examples/validation.rs).

| Current pending decision | Atomic effect |
| --- | --- |
| Complete | Submit exact current bytes, end draft identity, restore terminal; return Submitted in ValidationOutcome.event once |
| Incomplete | Insert LF at cursor through Editor, advance revision, retain editing |
| Invalid | Validate all diagnostic ranges, retain text/cursor/revision and display safe explanations |
| Stale revision | Return AnalysisOutcome::Stale, no event, mutation or terminal bytes, including after close |
| No pending Enter | ValidationError::NoRequest; a host cannot submit unsolicited validation |
| Malformed range/capacity | ValidationError::Edit; preserve request and draft so the host can correct its result |

Each successful response consumes the pending request. Repeated Enter at one
revision coalesces; same-revision job preference remains host-owned. There is no
second revision counter. Close/reopen clears pending requests and diagnostics
without changing an otherwise unchanged draft revision. Submit/interrupt/EOF
retain I0's semantic draft termination. Editing remains permitted while validation
runs; any revision change invalidates both pending authority and diagnostics.

`Diagnostic` has a safe nonempty single-line message and optional grapheme-aligned
UTF-8 byte range. Limits: 32 diagnostics, 4096 bytes/message, 65536 total message
bytes. Controls and hidden bidi/control-like characters are rejected before copying;
ZWJ and combining text remain admitted. Invalid is a host semantic result, not
an EditError. No warning ontology, fix-its or general syntax highlighting is added.

The bounded surface shows a textual invalid marker, diagnostic count and up to
four messages; long messages are ellipsized by grapheme and terminal cell width.
Optional ranges display byte offsets. Full content remains available through
`diagnostics()`. Resize and serialized output preserve current explanations;
Escape dismisses presentation only. A successfully installed completion set
replaces diagnostic presentation. Enter with a completion menu accepts a candidate
only; a later Enter requests validation.

Validated Up/Down uses Editor::line_up/line_down: logical LF lines first, history
at first/last line. Columns use the width policy, four-cell tabs and short-line
clamping; soft wraps use Left/Right. Ctrl-A/Ctrl-E retain whole-draft Home/End.
No force-submit bypass or new configurable keymap is supplied. Ordinary
replacement can insert LF explicitly; bracketed multiline paste remains one edit.

With `SubmissionPolicy::Validated`, Tab before the first non-whitespace character
of a continuation line inserts spaces to the next four-cell indentation stop.
The prefix must contain only ASCII spaces/tabs; stops are relative to the logical
line, independent of prompt width and soft wrapping. Existing tabs in the prefix
use the same four-cell stops. Text after the cursor is retained. The first logical
line and positions after non-whitespace still request completion. An active menu
always consumes Tab as next-candidate navigation. Shift-Tab retains its existing
menu binding; there is no unindent binding or grammar-aware automatic indentation.
This rule is implemented in the shared engine for all validated session/driven
hosts. Direct submission (including the simple blocking tier and C ABI 1) retains
Tab completion requests. Pasted tabs remain literal text. Indentation is one
atomic edit: success invalidates old analysis/diagnostics through DraftRevision;
capacity rejection changes neither draft/revision nor current presentation.

Adding SubmissionRequested is a native Rust enum extension: exhaustive matches
must account for it. Existing default behavior is unchanged. ABI 1 exposes neither
this event nor policy; its direct-submit contract, records and symbols remain
unchanged. The [use-case map](use-cases.md) separates deterministic hosts, model
clients, standalone reports and external reactors. [Evidence](engineering/validation-multiline.md)
records native and portable qualification separately.

## Editor analysis presentation

Native session/driven hosts may derive `AnalysisPresentation` from one
`AnalysisSnapshot` and call `Interaction::present_analysis`. REPLAI never calls
an analyzer. The host chooses cadence, parsing, roles, hint text and job/context
freshness. The live interaction remains exclusively owned; immutable snapshots
may leave it for externally scheduled work.

```rust
use replai::{AnalysisPresentation, AnalysisSpan, Hint, Role};
// `snapshot` is retained from this interaction; the host chose these boundaries.
let display = AnalysisPresentation::new(
    snapshot.revision(),
    vec![AnalysisSpan::new(0..2, Role::Accent)?],
    Some(Hint::new("ild", Role::Dim)?),
)?;
let outcome = interaction.present_analysis(display)?;
```

`AnalysisOutcome::Stale` changes nothing and emits no bytes. A current result
requires an active terminal. A malformed current result preserves prior display;
I/O failure uses the normal cleanup path. Same-revision valid deliveries replace
the whole result in delivery order. Empty spans/no hint clear it. Inspect current
data with `analysis_presentation()`; this never grants editor mutation access.

Spans are nonempty, ordered, nonoverlapping UTF-8 byte ranges with grapheme-aligned
endpoints. Adjacent spans are admitted; no merging or precedence by list order.
There are at most 4096 spans and 4096 safe hint bytes. Hints reject newline, tabs,
controls and hidden directional controls; they carry generic roles, not ANSI.
Bounds are checked before retention, and snapshot-dependent ranges before activation.

Editor Default is overridden only within each span. Prompt/continuation,
completion selection and diagnostics retain their own styles. The same frame
builder and damage renderer handle all of them. Styling changes no text geometry.

Hints are **non-canonical**. At end-of-draft they appear in remaining row space as
` [~hint]`, grapheme-clipped without wrapping; fewer than five free cells suppress
them. A non-end cursor or active completion menu suppresses the hint. Delimiters
remain under NO_COLOR; plain spans add no substitute punctuation. Hints never
enter submission/history. There is no hint acceptance key: supply a completion
candidate or existing revision-bound replacement when insertion is desired.

Output and resize preserve current display. Actual text/cursor changes (including
history, paste, completion edits and Incomplete continuation) invalidate it;
no-op/rejected edits do not. Menu dismissal restores a still-current hint;
validation Invalid may coexist with spans and hint. Close, failure and submission
release I2 surface state; close/reopen keeps I0 revision semantics unchanged.

The [example](../examples/analysis-presentation.rs) uses one host parse to derive
I1, I2 and I3 products. The [I2 dossier](engineering/analysis-presentation.md)
records resource limits, native/portable evidence and performance. Blocking
`read_line()` requires no analyzer. C ABI 1 does not expose this native Rust API.
