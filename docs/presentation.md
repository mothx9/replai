# Presentation contract and reference evidence

This document owns the terminal surface and its executable reference evidence.
The historical visual reference is the linear YVEX console at
`3a6520945a5c103365178f48104f0ccdb5154624` (branch `models1`, observed 2026-09-05).
The expected R0 donor was `cb336ad60c12d6fa841dc0715bba9d44aa721846`.
The intervening commits changed source/runtime work, with no changes to the
inspected console editor, palette, stream renderer, completion adapter or PTY
script. No donor source was modified or linked into REPLAI.

## Structured documents and semantic ownership

Hosts classify information; REPLAI lays it out. `Document` contains an ordered,
nonrecursive sequence of `Block`s. `Text` composes validated `Span`s using the
same seven `Role`s as prompts. No schema, command vocabulary, JSON dependency or
terminal escape supplied by the host becomes part of the model.
`Text::new` takes the block's default emphasis. Explicit `Span::new` or
`Text::styled` roles override that emphasis, including `Role::Default` to return
to the terminal foreground and normal intensity within an emphasized block.

| Block | Representation and narrow-width behavior |
| --- | --- |
| Paragraph | Preserves LF; wraps at extended-grapheme boundaries |
| Heading | Levels 1–3 with `#`, `##`, `###` and strong text; hierarchy survives plain output |
| KeyValue | Aligns labels within the group; stacks label/value when two readable columns do not fit |
| List | Ordered or unordered markers; explicit depth 0–8, two-cell indentation, hanging continuation |
| Table | Left/right value alignment, multiline cells, bounded column shrinking; stacks records with column headings when four cells per column plus gaps cannot fit |
| Literal | Preserves spaces and LF, expands TAB, wraps safely under a `| ` gutter; no syntax interpretation |
| Status | Info `[i]`, Success `[ok]`, Warning `[!]`, Error `[error]`; color is supplementary |
| Spacer | One explicit blank line |

Separation is explicit: documents do not insert arbitrary blank lines between
blocks. When a marker/indent leaves less than two cells, the marker occupies its
own wrapped rows and the content uses the full width. No horizontal scrolling,
hidden columns or silent truncation occurs. Paragraph and cell wrapping is
currently grapheme-based, not language-aware word breaking. A grapheme wider
than the available row returns `InvalidRange` rather than being split.

`Document::render(columns, theme)` works without terminal acquisition, returning
LF-terminated text. `write_to` lays out the entire document before touching the
caller-owned writer, then calls `write_all` once; a short-writing transport may
need several writes. The caller chooses captured-output width and resolves its
output capability using `Theme::new` or `Theme::from_environment(is_tty)`.
The same document, width and plain theme produce identical text under non-TTY,
NO_COLOR and TERM=dumb. Styling adds only foreground/emphasis, never a background.

For example, a key/value block needs no host padding:

```rust
use replai::{Block, Document, Text, Theme};
let document = Document::new(vec![
    Block::Heading { level: 2, text: Text::new("Connection")? },
    Block::KeyValue(vec![
        (Text::new("Endpoint")?, Text::new("http://127.0.0.1:18001")?),
        (Text::new("Locality")?, Text::new("loopback")?),
    ]),
])?;
let plain = document.render(60, Theme::new(false, false, None))?;
assert!(plain.contains("Endpoint  http://127.0.0.1:18001"));
# Ok::<(), replai::EditError>(())
```

The [structured example](../examples/structured.rs) combines grouped help,
connection facts, capability notices, a matrix and literal data. Run
`cargo run --locked --example structured -- 60` or set `NO_COLOR=1`; pass `8`
to inspect the same document's narrow fallback. This renderer does not reformat
an application's existing `println!` calls: adopting semantic blocks is a host
integration decision.

### Bounds and failure atomicity

Text admits valid UTF-8, normalizes CRLF, and permits LF/TAB. Other controls,
including CSI/OSC/DCS introductions, C1 controls, NUL and lone CR, are rejected.
TAB expands to four-column stops within its content column. Graphemes crossing
span boundaries remain indivisible and take the first scalar's role. The width
policy and terminal/font limitations below apply to documents as well as drafts.

Each span and combined Text is at most 16 KiB, with at most 1024 spans per Text.
A document admits at most 4096 blocks, 1 MiB aggregate text and 16,384 aggregate
spans/fields (empty fields still count). Tables admit 1–32 columns and at most
4096 rows; lists and fact groups admit at most 4096 entries. Lists are flat
records with depth at most eight; there is no recursive traversal.
Rendering accepts widths 2–4096 and bounds work to 65,536 physical rows and a
conservative 8 MiB text/style encoding budget, including plain rendering.
Excess input/work reports `Capacity`; invalid geometry/structure reports
`InvalidRange`; forbidden text reports `InvalidText`. Nothing is truncated.
Construction cannot validate a future width: render-time rejection is possible.
Rejected output leaves the writer, active surface and editor unchanged. An I/O
failure may have delivered a prefix; the active interaction attempts its normal
terminal cleanup. No output API promises rollback of already-written bytes.

### Theme and prompt composition

`Style` maps a role to `Foreground` and bold intensity. The restrained foreground
choices are terminal-default, neutral, cyan, gray, green, amber and red.
`Theme::with_style` customizes individual roles without admitting SGR strings;
Default must remain the reset style. The compatibility palette remains the
initial theme, so old Rust and C consumers retain their presentation. Strong
headings/labels, dim literal data, markers and indentation supply hierarchy.
This is a theme foundation, not the final visual refinement or capability model.

`Prompt::new("demo")` retains exactly `demo> ` and its existing accented style.
For richer prompts, `Prompt::composed(Text::from_spans(...))` takes ordered
host-provided segments, including delimiter and spacing. Context and state have
no library-specific meaning. `with_continuation_text` uses the same span model.
Composed primary prompts allow 3072 bytes/64 spans; continuations allow 1024
bytes/64 spans. Prompts reject all controls, including LF/TAB. Existing simple
label/state/continuation fields retain their 1024-byte limits. `with_state` on a
composed prompt rejects rather than guessing a suffix insertion slot.
Use `Interaction::open_with_theme` to select the same resolved theme for prompt
and coordinated documents. `open` preserves environment-derived default styling.

## Prompt, cells and redraw

Layout uses `unicode-width`'s normal (ambiguous-narrow) cell policy per grapheme.
ANSI style sequences are not text and contribute no cells. Draft TAB expands
to four-column stops. CJK wide characters, accented text, combining clusters,
ordinary emoji and joined emoji are covered by core/layout tests. The independent
VT oracle covers the subset it can model; a font or emulator may render a joined
emoji or ambiguous character differently. Full Unicode terminal equivalence,
bidi layout and all terminal width tables are not claimed.

Simple prompt fields are plain control-free text, at most 1024 bytes each. The label
and optional literal suffix compose as `<Accent>label+suffix><Default> `;
continuations default to `... `. No raw ANSI prompt injection is accepted.
Style roles are Default, Strong, Accent, Dim, Success, Warning and Error. All
SGR values have one authority in the [VT encoder](../src/protocol.rs); the public
`Theme::sequence` compatibility method delegates there. Layout retains semantic
roles and cell geometry before encoding. No background color or alternate
screen is set. Non-TTY output, `NO_COLOR` present (including empty), or
`TERM=dumb` disables styling. Disabling color emits no SGR, including no reset
residue, while preserving text. `TERM=dumb` follows the reference's color rule; it is **not** a
promise to operate on a terminal that lacks the cursor/erase protocol entirely.

Physical rows are explicitly laid out with CR/LF; full-width boundaries and
wide-character gaps are handled without counting scalars as columns. A logical
newline immediately after a full row does not introduce an extra blank row.
Stable geometry permits cursor-only moves and replacement of changed rows or
ASCII suffixes. Other transitions erase the previous editing rows before
redrawing; the logical cursor is restored in either case. Normal short end
insertion appends directly, preserving the compact reference rhythm. Ctrl-L
explicitly clears the visible screen and redraws.
For drafts taller than the terminal, a cursor-following viewport retains at most
height minus one physical rows; hidden text is retained and still submitted.
Minimum dimensions are two columns and two rows. Resize qualification covers
PTY dimension changes and the VT cell model, not every emulator's scrollback
reflow policy; the [executable oracle](#executable-oracle) states the evidence.

## External output

`external_output(role, text)` is a synchronous display transaction: disable paste
framing, clear the editing rows, write validated host text using a generic role,
finish its line, enable paste and redraw draft/cursor. LF/CRLF are normalized;
other controls except TAB reject before any terminal mutation. There is no raw
ANSI passthrough. Input stays raw and queued bytes are retained. The host controls
when output is written; independent concurrent writes must be serialized through
this method. Partial fragments can be sent as separate lines; continuous
no-newline streaming batches and an unrestricted writer guard are not supported APIs.

`Interaction::output_document(&document)` uses the same synchronous surface
transaction and current observed terminal width. It lays out and validates all
blocks before erasing anything. The engine restores the exact draft, cursor,
prompt and continuation; the POSIX driver handles transport and failure cleanup.
Call `poll` to observe a resize before emitting output when the host has changed
terminal dimensions. There is no second editor or independent output writer.
The old plain method projects unwrapped literal text to the same semantic
mutation/encoding and coordination path, preserving TAB, trailing LF and ABI 1
bytes. It intentionally does not acquire document wrapping or size limits.
Structured documents/composed spans are native Rust APIs only; C ABI 1 is
unchanged and retains its existing plain prompt/output functions.

## Structured qualification and characterization

[Public document tests](../tests/document.rs) assert exact plain hierarchy,
responsive rows, style reset, Unicode span boundaries, safe-text rejection,
bounds and writer atomicity. Deterministic engine tests prove failed layout
leaves the editing surface and non-end cursor unchanged. The shared
[POSIX PTY suite](../tests/pty.rs) emits the same document at narrow/wide widths,
with styled, NO_COLOR and dumb policies, during a composed multiline draft.
It compares VT cells/cursor to independent rendering, observes resize/reflow,
default background, balanced paste modes and exact captured termios restoration.
Terminal tests also inject write failure through both output entrypoints.
Linux/macOS run those PTYs; Windows runs the portable document/engine tests.

The existing [performance fixture](../tools/perf/documents.rs) characterizes
paragraphs, 16-field groups, 32-entry lists/tables, status and a 128-paragraph
document at widths 8, 20, 80 and 160. It measures layout separately from
layout+plain/styled encoding+writer delivery. Separate allocation instrumentation
records allocation counts/bytes; output counters record encoded bytes and logical
writer calls, not OS syscall timing. `python3 tools/perf/smoke.py` checks fixture
integrity on all three platforms, without hosted latency thresholds. Existing
editor, render, PTY and comparison measurements remain in that suite.

## Record derivation

The compact [presentation.tsv](../tests/fixtures/presentation.tsv) contains ten
named hex byte streams, 285 bytes of decoded terminal output in total. They
were captured through Linux PTYs from a temporary neutral C probe using the
**actual** `yvex_cli_terminal_style_get`, `repl_columns` and `repl_redraw`
functions extracted from the pinned donor. That probe lived outside this repo;
no donor implementation is shipped here. Its small driver supplied `demo` as
host label and invoked the observed presentation primitives below. It neither
linked a runtime nor exercised the full donor application.

The PTY transport had output postprocessing disabled to record explicit CR/LF
once (and avoid the donor transport's CR/CR/LF artifact). No content, cursor
commands, SGR or spacing was normalized after capture. `TERM=xterm-256color`
was used except for `dumb`; `NO_COLOR` was absent except for the deliberately
empty value in `no_color`. Neutral content is an explicit host substitution,
not a changed terminal style. The probe used this prompt expression:
`accent + "demo>" + reset + " "`.

| Record | Executed/reference operation | Intentional comparison |
| --- | --- | --- |
| `styled_prompt` | Redraw empty input | Exact initial prompt bytes and accented foreground |
| `no_color` | Same with NO_COLOR present and empty | Exact plain bytes, no reset residue |
| `dumb` | Same with TERM=dumb | Exact plain bytes; cursor protocol still used |
| `typed` | Empty redraw, then append `hello` | Text/cursor/style state |
| `left` | Redraw `hello` at byte cursor 4 | One-cell backward cursor motion |
| `history` | Redraw replacement `earlier` | Complete old draft replacement |
| `paste` | Empty redraw, `hello`, one CR/LF plus `... `, `world` | Intended single-newline continuation rhythm |
| `clear` | Clear visible screen/home, redraw `hello` | Explicit Ctrl-L behavior |
| `interrupt` | Redraw `hello`, append `^C` and CR/LF | Visible interrupt line |
| `resize` | Redraw `draft` | Short-input redraw after dimensions change |

Paste and interrupt records combine the actual redraw helper with the literal
control emission at `client.c:1116` and `client.c:1137`; they do not claim a full
live donor input replay. REPLAI's paste test sends CRLF, which intentionally
normalizes to the single logical newline represented here. Donor CRLF doubling
is a deficiency, not parity authority.

Pinned source evidence:
[palette and disable rules](https://github.com/yailabs/yvex/blob/3a6520945a5c103365178f48104f0ccdb5154624/src/cli/io/out.c#L236),
[redraw](https://github.com/yailabs/yvex/blob/3a6520945a5c103365178f48104f0ccdb5154624/src/cli/io/client.c#L983),
[input, continuation and interrupt](https://github.com/yailabs/yvex/blob/3a6520945a5c103365178f48104f0ccdb5154624/src/cli/io/client.c#L1089),
[prompt composition](https://github.com/yailabs/yvex/blob/3a6520945a5c103365178f48104f0ccdb5154624/src/cli/io/client.c#L1552).

## Executable oracle

`cargo test --test pty` launches separate processes with isolated environment
values and actual PTYs. It types the representative input through REPLAI's public
API, changes real PTY dimensions for resize, and compares terminal **cell text,
foreground, weight, default background and cursor** against these byte records
using the independent `vt100` parser. Initial styled/plain prompts also require
exact bytes. No OCR, runtime service, font screenshot or application repository
is needed. Only Rust test-harness chatter outside bracketed-paste lifecycle
markers is excluded from capture. Redraw's extra safe cursor/erase operations
are compared by resulting state, not removed from the stream.

Further PTY assertions cover mixed-width cursor edits, full-width row boundaries,
multiline continuation, tall drafts, height changes and external output with a
non-end cursor. Those are corrected structural contracts with explicit expected
cells; no broken donor snapshot is used as their oracle. There is no donor
active-draft external-output equivalent to copy: the new transaction is qualified
against exact preserved text/cursor and expected visible notice/draft states.

Unit tests require all seven exact SGR sequences from the reference palette and
all disable conditions. PTY cell assertions require the default background and
absence of alternate screen. The normal renderer never sets a background; its
clear commands use the terminal's own background. Ctrl-L is the explicit
full-visible-screen clear action, not a background paint or TUI switch.

## Deliberate differences and qualification limit

Grapheme movement, Unicode cells, multirow erase, viewport layout, proper resize,
CRLF normalization, atomic paste rejection and draft return repair source-derived
deficiencies. Ctrl-D on nonempty text performs forward deletion. Ctrl-C is shown
at the end of the editing surface so it cannot overwrite a draft when the cursor
is in its middle. Admission, repeated-interrupt exit, command lookup and application
labels are host choices. Arbitrary ANSI output is excluded from the safe text API.

This is evidence for the specified line-oriented visual/interaction grammar and
its corrected structural cases. It is not pixel/font equivalence, a full product
runtime transcript comparison or certification of every emulator's emoji widths,
ambiguous-width settings, saved scrollback reflow or terminal multiplexers.

## The same renderer through C

The observed read-only donor at R2 start was
`5b95ee82eee394581521d106c7b1ec479d472448`, branch `models2`, tree
`7f1065cda89b12a54d81591f801f492da70594ca`. The console, palette, stream output
and PTY script are unchanged from the R1 oracle revision above. R2 retains that
record instead of following unrelated runtime work.

`tools/c_pty.py` drives the **external C process** built from an installed header
and release library. It does not call Rust editor functions from the harness.
The dev-only `terminal-state` executable uses the independent vt100 parser to
report cells, foreground, bold weight, default background and cursor. Both
static/shared processes, plus the shared process under Valgrind, run the same
scenarios. Initial styled/NO_COLOR/dumb prompt bytes must equal the R1 record
exactly after the bracketed-paste enable marker. All seven generic style roles
are checked in actual C output. There is only one Rust renderer beneath both APIs.

| C scenario | Observed state required by assertions (zero-based row,column) |
| --- | --- |
| C01 styled / C02 NO_COLOR / TERM=dumb | `demo> `; cursor (0,6); Accent 81 or default; exact prompt bytes |
| C03 UTF-8 edit | `hé界🌍`, Left, Backspace, `X` → `héX🌍`; byte cursor 4, cells (0,9) |
| C04 history / C14 reopen | Submit `earlier`; draft `draft`, Left, Up, Down → original draft and byte cursor 4; cells (3,10) |
| C05 completion | `wor`, Tab → request with `wor`, C chooses `world`; byte cursor 5, cells (0,11) |
| C06 paste | Bracketed `é` CRLF `界` → one `é` LF `界` input; continuation `... `, cursor (1,6) |
| C07 resize | 12 to 9 columns while editing `ab界` LF `line 🌍`; four physical rows, cursor (3,0); inserting X gives `ab界` LF `line X🌍` |
| C08 external output | Dim notice above `demo> ab界`; byte cursor 2 remains at (1,8); X gives `abX界` |
| C09 interrupt | `hello`, Ctrl-C → event 2, visible `demo> hello^C`, cursor (1,0) |
| C10 empty Ctrl-D | Event 3, empty draft, cursor (1,0) |
| C11 nonempty Ctrl-D | `abc`, Ctrl-D at end, Home, Ctrl-D → `bc`, byte cursor 0, cells (0,6); then submit |
| C12 submit | `hello`, Enter → event 1 and exact echo text; cursor (3,0) |
| C13 restoration / C15 destruction | Captured termios before == after; paste disabled; destroy success; process exit 0 |
| Ctrl-L | `draft`, Left, Ctrl-L → same draft, cursor (0,10), cleared visible surface |

Every scenario checks default background and no alternate screen. Resize keeps
the real trailing space on its full-width continuation row; normalization must
not delete a visible cell. PTY JSON retains input hex, complete output bytes,
events, text and cursor observations. These are terminal-state/byte contracts,
not a claim about pixel rendering or arbitrary emulator reflow.
