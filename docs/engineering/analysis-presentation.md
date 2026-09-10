# I2 analysis presentation evidence

This dossier owns the bounded ANALYSIS.PRESENTATION.0 qualification. The
[interaction contract](../interaction.md#editor-analysis-presentation) owns API
semantics; [ROADMAP](../../ROADMAP.md) alone owns maturity.

## Identity and scope

Baseline: `e372f4eb897c81db24e1f7d489298a529f079f04`, tree
`24ea60d41a1ecd3aaa656fcb5cd449700d16bdc2`. It is a logo-only descendant of
qualified I3/U2 carrier `958881ce9e288a1d1f6af928fb9291531490273f`.

I2 adds native Rust editor display, not parsing, analysis scheduling, insertion,
validation policy or a new terminal renderer. C ABI 1, dependencies and Editor
storage/revision ownership remain unchanged. Consumer repositories and BOUNDARY
are not modified. No subsequent wave is authorized by this work.

## Design decisions and alternatives

| Boundary | Decision and reason |
| --- | --- |
| Provenance | Reuse DraftRevision and AnalysisOutcome. Same-revision deliveries replace in serialized arrival order; host context and job priority remain external. |
| Canonical owner | Editor still owns bytes, cursor, history and revision. Optional boxed surface state owns one AnalysisPresentation; closing drops it. |
| Ranges | Nonempty, ordered, nonoverlapping byte ranges. Every endpoint must be a snapshot grapheme boundary. Adjacent spans are valid; no sorting, merging or cascade. |
| Bounds | 4096 spans, 4096 hint bytes, fixed-size per-span metadata; no retained draft clone. Boxed slice prevents excess Vec capacity being retained. |
| Hint | One safe non-canonical suffix, visible only at end-of-draft and clipped to spare cells without wrapping. `[~...]` identifies derived text with or without color. Suppressed with a menu or a non-end cursor. |
| Insertion | No hint acceptance API/key. Host can derive I1 candidates or use existing revision-bound replacement. |
| Renderer | Frame::analyzed adds generic Run::Style over canonical graphemes. Completion and validation pass the same spans to the same frame builder. Renderer still computes terminal damage; protocol encoding stays separate. |
| Cost | Single monotonic endpoint validation traversal and ordered span cursor during layout. No per-grapheme scan from the first span; no allocation or snapshot creation on ordinary input merely to track I2. |
| Rejected alternatives | Parser/highlighter callbacks would put host execution in rendering. Mutable ghost previews would violate I0. Arbitrary overlays and overlapping style precedence would add an unnecessary rendering framework. |

## Exact lifetime and composition

Installation validates provenance, lifecycle and every range before changing the
active result. Malformed current data retains the old result. Stale data emits
no mutations, even after close. Empty current data clears. Installation does not
change draft revision. No-op/rejected edits retain current display; successful
text or cursor changes discard it automatically. Close/abandon/finish remove
I2 display before leaving the input surface. Reopening the same unchanged draft
preserves its I0 identity but starts without retained I2 display.

Base editor role is Default; a host span overrides only its canonical range.
Prompt/continuation styles, completion selection and validation diagnostics keep
their own roles. A menu suppresses the hint but retains draft spans. Dismissal
restores the still-current hint; acceptance that changes the draft invalidates
I2. Invalid validation may coexist with spans/hint; Incomplete inserts LF and
invalidates them. Output and resize preserve provenance and result data.

Pure spans disappear visually without styling. The hint is explicitly delimited
in all themes, cannot wrap, and cannot become submitted text or history. A hint
is omitted when fewer than five spare cells remain. It is not an autosuggestion
provider, an analysis cache, an executable completion or a language feature.

## Executable evidence

- [Portable public payload tests](../../tests/analysis_presentation.rs): ordering,
  overlap/count limits, empty/reversed spans, Unicode-safe hints and rejection of
  ESC/OSC/CR/LF/TAB/hidden controls; Send/Sync immutable result payload.
- [Deterministic engine tests](../../src/analysis_presentation_tests.rs): atomic
  installation/clear, grapheme range rejection, cursor and identical-text stale
  identities, I1/I3 composition, output/resize, canonical geometry and 10,000
  generated transitions; 10/100/1000/16000-line bounded frame cases.
- [Virtual transport conformance](../../src/conformance.rs): actual encoded screen
  cell colors, plain output, canonical cursor, stale zero writes and restoration
  after a presentation write failure. This is portable logic, not OS support.
- [Native PTY oracle](../../tools/analysis_presentation_pty.py): the same external
  readiness host on Linux/macOS, independent application socket, delayed stale
  and fresh delivery, shared I1/I2/I3 analysis, session submission, styled/plain,
  20/40/80/132 columns, Unicode multiline, output, history, continuation and 12
  repeated FD/termios lifecycles. Hints are excluded from submitted bytes.
- [Host-owned parse](../../examples/support/presented_analysis.rs) produces
  CompletionSet, AnalysisPresentation and ValidationResult from one snapshot.
  Neither this parser nor its command vocabulary is linked into the library.
- [Performance component](../../tools/perf/presented.rs): 10/100/1000 spans,
  1 KiB/64 KiB/1 MiB and 10/100/1000-line fixtures, short/long/Unicode hints;
  separate install, render, resize and clear latency/allocation/encoding records.

Run the existing complete qualifier plus its new I2 native and memory gates:

```sh
python3 tools/qualify.py --work /tmp/replai-i2-qualification
cargo test --locked --test analysis_presentation
python3 tools/analysis_presentation_pty.py --work /tmp/replai-i2-pty
python3 tools/analysis_presentation_pty.py --memory --work /tmp/replai-i2-memory
```

The full qualifier also retains I0/I1/I3/F2/embedding, static/shared C, ABI
layout/symbol/header drift, Valgrind/Linux or native leaks/macOS, documentation
and benchmark integrity. Native CI runs Linux and macOS; Windows runs portable
models only. No Windows terminal backend is added.

## Qualification-harness correction

The inherited performance Session launcher forced NO_COLOR, including when some
consumer PTY wrappers called their scenario “styled”. This wave adds an explicit
styled launch option without changing default benchmark policy. I1/I3/I2 native
oracles now request true styled versus NO_COLOR profiles. The I2 oracle asserts
an actual editor cell's Accent color via the independent VT parser, and Default
under NO_COLOR. Earlier artifact labels alone are not upgraded into styled proof.
The readiness observer recognizes the prompt before an intervening SGR reset.

## Measurements and remaining limits

Final measurement/CI identities are recorded after execution, separately from
this source contract. A native test does not establish emulator-independent
Unicode appearance. WidthPolicy remains the existing deterministic cell policy.

U2's known prefix geometry scan remains: a bounded visible frame does not imply
constant-time layout for a cursor deep in a large draft. I2 does not redesign
that storage/layout boundary. Host analysis cadence is outside the library;
cursor movement still invalidates spans and may require host reuse/reanalysis.

No I4/I5/U3/output-concurrency or API-freeze claim follows from I2. C ABI 1 keeps
its existing synchronous replacement/direct submission surface; native I0–I3
visibility requires separate E3 cross-language design.
