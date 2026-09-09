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
Ctrl-A/C/D/E/L, Backspace and Tab. Unknown sequences are rejected. Oversized CSI
or OSC sequences drain to their terminator with constant extra memory. A bad
UTF-8 byte and the incomplete scalar containing it are rejected together; the
offending byte is not replayed as a shortcut.

The adapter expires pending sequences after a 250 ms idle interval, observed
by the selected scheduler: compatibility polling or an explicit host deadline.
Each compatibility poll waits at most 100 ms. This is an idle bound, not a
fixed total sequence length/time guess; tests deliver each byte with a delay
longer than 25 ms. Expired UTF-8/escape input produces `Event::Rejected` and
keeps the draft. Host starvation can delay observation.

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

## History

History is configured by entry count and input byte bound. Admission is explicit,
with oldest-entry eviction only when the host-selected bound is full. No hardcoded
history size, persistence, deduplication or privacy policy exists. First Up saves
the current draft **and cursor**; Down past the newest entry restores both.
Editing a recalled entry changes only the current edit. Navigating away discards
that recalled edit, without modifying admitted history.

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

`TerminalFacts` distinguishes Unknown, Unavailable, Assumed and Supported
cursor, erase, styling and paste features. `TerminalConfig` separates these facts
from Disabled/Preferred/Required host policy for optional features and a `Theme`.
Cursor and erase are required by this editor. `resolve()` is pure admission;
actual TTY identity/dimensions are still checked separately during acquisition.
`Interaction::features()` reports the admitted styling and paste mode.

`read_line` and `open_driven` use conservative environment convenience: a
nonempty TERM other than `dumb` supplies an explicit VT **assumption**, not
terminal probing. Missing/empty/dumb TERM does not establish cursor/erase and
fails before raw mode. An independently informed host can supply facts using
`open_with_config`; this does not bypass actual TTY validation. NO_COLOR disables
styling; an explicitly plain theme stays plain. Required styling with a plain
theme rejects. Optional unavailable paste may be disabled, in which case no
paste-mode toggles are emitted, including output transactions and cleanup.
Without framing, multiline paste cannot be distinguished from ordinary Enter.

Existing `open`, `open_with_theme` and C ABI 1 retain their qualified explicit VT
compatibility assumption: TERM=dumb disables styling but does not select a new
line editor or suppress their existing cursor/erase protocol. This distinction
preserves existing consumers while making new simple/driven defaults truthful.

Non-TTY or redirected interactive resources reject as `UnsuitableTerminal`.
Standalone `Document::render` remains suitable for plain captured/non-TTY/dumb
output without an interaction. A document renderer does not grant terminal
editing capabilities. No universal capability discovery, Windows backend,
terminal probing, fallback cooked editor or complete F2 negotiation is implied.

`Error::CapabilityMismatch(&'static str)` reports admission failure, separately
from lifecycle, unsuitable resource and I/O errors. Adding this native variant
requires exhaustive Rust error matches to add an arm; existing operations keep
their behavior. C ABI 1 declarations, records, symbols and numeric identity are
unchanged; the binding maps this native category to its existing unsuitable
terminal status. It exposes neither the new blocking nor driven methods. Their
future C/HANDLE contract remains an independent cross-language design boundary.

See the [simple example](../examples/simple.rs), [session example](../examples/demo.rs),
[external reactor](../examples/driven.rs) and [embedding qualification](engineering/embedding.md).
