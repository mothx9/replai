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

The qualified results below separate source identity, runtime execution and
characterization from this contract. A native test does not establish emulator-independent
Unicode appearance. WidthPolicy remains the existing deterministic cell policy.

U2's known prefix geometry scan remains: a bounded visible frame does not imply
constant-time layout for a cursor deep in a large draft. I2 does not redesign
that storage/layout boundary. Host analysis cadence is outside the library;
cursor movement still invalidates spans and may require host reuse/reanalysis.

No I4/I5/U3/output-concurrency or API-freeze claim follows from I2. C ABI 1 keeps
its existing synchronous replacement/direct submission surface; native I0–I3
visibility requires separate E3 cross-language design.

## Qualified results

Implementation: `e0ab06bc2e9111b41968842b459c51a68855aa13`, tree
`3c0479400d02f73fcc3237a026c47deb51a8200e`.
Fixture carrier: `27981f0dbb22305f96a07bb7e4c7fafbbf81e90b`, with unchanged library
source and added delayed-session/active edit-invalidation checks.
[Implementation CI](https://github.com/mothx9/replai/actions/runs/34501515879) and
[fixture CI](https://github.com/mothx9/replai/actions/runs/34502378013) each passed
**12/12 jobs**. The local complete qualifier passed **22 content gates**, including
I2 native/memory, all existing boundaries and C prepare/static/shared/memory/audit;
clean publication is checked separately at closure.

[Native observations](analysis-presentation/platforms.json) retain real Linux and
macOS session/driven results, styled/plain editor cell checks, stale zero-byte
refusal, active completion/history/paste invalidation, output/resize and 12
restoration cycles per profile. Linux Valgrind reports zero errors; macOS
`leaks --atExit` reports zero leaks. The 10/100/1000-line semantic screens match
across Linux and macOS. Windows portable engine/model, docs and benchmark
integrity jobs pass; this remains **no Windows terminal-runtime claim**.

### Performance and memory

[Before](analysis-presentation/before.json) and
[after](analysis-presentation/after.json) are schema-v1 selections from 7,000 and
7,410 full measured/integrity records. Raw timing and allocation streams were
separate. All **3,494 common measured allocation profiles are identical**.
Environment, compiler, sample distributions and source hashes are embedded.

The baseline binaries were prepared from clean `e372f4e…`. Development edits
started while those immutable binaries ran, so run.py's live-source end guard
correctly refused to call that a source-stable run. The baseline envelope was
assembled only after verifying **every executing binary hash** against its clean
pre-mutation preparation receipt; that explicit qualification is recorded in the
artifact. There was no rebuild of those binaries during timing. The after run
passes the source-hash stability guard; its development source fingerprints were
independently matched against every published implementation blob at `e0ab06b…`.
Its recorded dirty preparation identity is retained, not rewritten as clean.

Same Linux aarch64 machine, 1 KiB ASCII editor, median microseconds:

| Operation | Before | After |
| --- | ---: | ---: |
| Append | 0.064 | 0.064 |
| Left | 0.032 | 0.032 |
| Right | 0.064 | 0.064 |
| Direct completion replacement | 0.048 | 0.048 |
| History previous | 0.048 | 0.048 |
| 10-line beginning edit, validation enabled | 16.352 | 16.464 |
| 1000-line beginning edit, validation enabled | 37.728 | 38.256 |

The timer quantizes very small operations at roughly 0.016 µs. The
[alternating primary PTY run](analysis-presentation/paired.json) uses the exact
1000-byte ASCII + Left + X + Enter workload: **248.433 → 165.232 µs median**,
p95 **362.352 → 275.344 µs**, MAD **26.959 / 34.768 µs**. Both emit **1054 bytes**;
all 63 samples per source submit exact bytes and restore terminal state. This
run establishes no observed regression at that workload; it is not a causal
speedup attribution or a general library ranking.

I2 installation includes snapshot-range validation, current-state replacement,
layout and render transition, with the payload already constructed by the host.
Short hint, cursor at end, 80 columns, median milliseconds:

| Spans | 1 KiB | 64 KiB | 1 MiB |
| --- | ---: | ---: | ---: |
| 10 | 0.037552 | 2.169203 | 34.633633 |
| 100 | 0.046960 | 2.268051 | 35.991139 |
| 1000 | 0.039744 | 2.359603 | 36.237203 |

Adjacent equal-role spans need fewer emitted style runs than sparse alternating
ranges, so span count alone is not a latency ranking. The 100-span multiline
fixtures install in **0.033632 / 0.267440 / 2.614420 ms** at 10/100/1000 lines
(690/6900/69000 bytes). No constant-time deep-cursor layout claim is made.
Long 4096-byte and combining/CJK/ZWJ hints are recorded separately in the artifacts.

At 1 KiB/100 spans, forced render (Ctrl-L) is **34.480 µs**, resize 80→40 columns
**35.600 µs**, and explicit display clear **24.240 µs**. Component VT-byte counters
use the harness's plain encoder and are not OS syscall counts. The real native
`bu` fixture records styled/plain bytes: initial install **106/59**, identical
replacement **0/0**, output coordination **168/109**, completion show **278/168**,
menu dismissal **139/92**, display clear **32/18**. These are line-surface damage
transactions, not an alternate-screen or full-terminal menu redraw.

On this 64-bit target, Editor remains **144 bytes**; Interaction changes
**688→704 bytes**. AnalysisSpan is **24 bytes**, Hint **24**, and
AnalysisPresentation **64**. One retained maximum I2 payload is bounded to
4096×24 + 4096 + 64 = **102,464 bytes**, excluding allocator bookkeeping and
existing renderer buffers. The result owns no draft copy. Ordinary editing
without a result allocates no candidate/span/hint state.

Allocation records measure installation/renderer work, excluding the host's
already-built payload: 1 KiB short-hint installs with 10/100/1000 spans make
**170/599/118 allocations**, with **7,581/41,365/5,701 bytes** net retained by the
measured frame/effects transaction. These numbers include temporary render
mutations and are not the size of the semantic payload. Sparse styles and
cross-line layout still create recycled row/run work; optimization beyond this
bounded first contract requires its own measurements.
