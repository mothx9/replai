# Project status

This is the sole authority for current project status and future direction.
Implemented contracts live in [the documentation map](docs/README.md);
[CHANGELOG](CHANGELOG.md) and Git retain completed changes. Planned series below
are engineering objectives, not existing features, API declarations, release
promises or authorization to begin the next wave.

## Thesis: simple entry, room to grow

REPLAI is becoming an **embeddable interactive command-line substrate**:
editing, terminal lifecycle, input decoding, history mechanics, host analysis,
rendering and output coordination beneath an application-owned command loop.
It should be immediately useful to a small CLI, then support richer application
shells, database/debugger/compiler frontends and long-lived processes with
background or streaming output without replacing the interaction engine.
Streaming model clients and agents are important consumers of these generic
properties; their commands, execution, state and semantics remain outside the
library. REPLAI remains narrower than a shell framework or a full-screen TUI.

Small embedding cost and simple mental models matter as much as richer behavior.
The conceptual reference is modern linenoise, including its multiplexed editing
and hide/show output lifecycle, not only its original blocking line-read call.
[Linenoise's current documentation](https://github.com/antirez/linenoise/blob/master/README.markdown)
also describes multiline editing, hints, bracketed paste, UTF-8 and masking.
These are comparative design inputs, not borrowed compatibility guarantees.

[Reedline](https://github.com/nushell/reedline) demonstrates separate completion,
hints, validation, edit modes and menus, with an explicitly experimental
concurrent external printer. [Rustyline's helper boundary](https://docs.rs/crate/rustyline/18.0.1/source/src/lib.rs)
combines analysis roles and explicitly raises parsing a draft once for their
shared use. REPLAI should preserve the simplicity of embedding while making
that richer analysis and output coordination coherent. It must measure its own
tradeoffs rather than infer superiority from a feature list.

## Extraction foundation and current status

| Wave | Status and established property |
| --- | --- |
| R0 — Repository genesis | Completed: independent repository, safe Rust foundation and application-neutral ownership |
| R1 — Terminal editor kernel | Completed: Linux grapheme editor, bounded decoder/paste, history, lifecycle and terminal-native presentation |
| R2 — Interaction API / ABI | Completed: movable native ownership, C ABI 1, installed static/shared consumers, real PTY/layout/resource/memory qualification |
| R3 — First consumer | Completed: YVEX consumes the qualified C boundary; product execution and cancellation remain host-owned |
| R4 — Second consumer | Completed: YAI consumes native Rust; transient edits remain distinct from canonical conversation submission |
| R5 — LEGACY.OWNERSHIP.CLOSURE | Consumer editor removal and local qualification established; outstanding YVEX publication reconciliation is externally owned |

Both consumers retain exact qualified revision
`df5538c718b8d068432032e7fb116fb8bfab158e`. Library documentation evolution does
not justify repinning. The remaining consumer publication reconciliation does not block independent
library architecture work unless it demonstrates a reproducible generic contract
defect. This repository does not declare that external publication complete.

**F0 — PUBLIC.ARCHITECTURE is the active library wave.** F1, P0, P3 and all
later implementation waves remain planned and have not started.

System-terminal runtime qualification remains Linux-specific and pre-release. There is no stable API,
ABI, SemVer or MSRV promise. The current implementation has one active terminal
interaction per linked library image, synchronous safe-text output transactions
and host-owned signal meaning. It does not support independent concurrent
writers. See [interaction](docs/interaction.md) and
[presentation](docs/presentation.md) for the actual conditions.

## Ordered development map

```mermaid
flowchart TD
    R[Extraction closure: R5] --> F[General library architecture: F]
    F --> PI[Performance and command interaction: P and I]
    PI --> OU[Streaming output and line-oriented UX: O and U]
    OU --> XQ[Platforms and robustness: X and Q]
    XQ --> E[Ecosystem and embedding: E]
    E --> V[Qualified public release: V]
```

This is dependency order, not a promise to finish a whole series before learning
from another. In particular, the first post-R5 design must consider **API tiers,
performance baselines and event-loop architecture together**. F0 frames that
joint investigation; F1, P0 and P3 retain distinct deliverables. A convenient API
cannot be frozen before understanding its blocking behavior and costs. F0 has explicit task authorization; no later implementation is authorized
by the roadmap or by external consumer closure.

### Foundation — one engine, several embedding levels

| Wave | Property to establish |
| --- | --- |
| F0 — PUBLIC.ARCHITECTURE | **Active.** Re-evaluate ownership as general infrastructure: editor, terminal substrate, interaction, host analysis, render and output boundaries with no donor-specific assumptions |
| F1 — API.TIERS | A small blocking/read-line entry, an explicit host-driven interaction surface, and a driven/multiplexable surface over the same engine; simple programs need no elaborate event loop |
| F2 — TERMINAL.CAPABILITIES | Explicit capabilities and degradation policy, rather than scattered environment checks; request only features needed by the selected line-oriented interaction |

These are responsibilities to test, not a prescribed module tree or object
layout. The lowest integration tier must accept host readiness, resize and
arriving analysis/output without imposing Tokio, another async runtime or a
threading model. The higher tiers must not become separate editors.
Capabilities should distinguish styling, paste, cursor operations and width
assumptions. TERM=dumb needs a deliberate degradation contract; a capability
inventory does not authorize mouse interfaces, alternate screens or a canvas.

### Performance — measure before replacing the kernel

| Wave | Property to establish |
| --- | --- |
| P0 — PERFORMANCE.BASELINE | Reproducible latency, throughput, allocations, memory, bytes, writes/syscalls, idle wakeups and artifact-size baselines; fair comparisons with linenoise, rustyline and reedline where workloads overlap |
| P1 — EDITOR.KERNEL.PERFORMANCE | Choose buffer/index/cache strategy from observed costs and bounded-memory constraints, including long Unicode and multiline drafts |
| P2 — INCREMENTAL.RENDER | Measured frame deltas and batched output that reduce cells mutated, terminal bytes and syscalls while preserving cursor/layout correctness |
| P3 — EVENT.DRIVER | Host event-loop embedding without artificial periodic polling where readiness/resize mechanisms permit it; no prescribed executor |

Today the editor uses String storage and grapheme traversal; the renderer can
append a suffix but otherwise redraws the frame. Polling checks dimensions and
caps waits at 100 ms to avoid taking over resize signals. These are valid
foundation choices, not measured performance conclusions or immutable designs.

P0 must characterize beginning/middle/end edits; ASCII, CJK, combining and emoji
movement; key-to-frame p50/p95/p99; output chunk rates; lifecycle and FD stability;
and substantial pasted SQL, source, JSON or prose. Proposed pressure ranges
include 1 KB–1 MB paste, 100–100k history entries and 10–100k candidates where the
respective interface exists. Unsupported ranges must be labelled, never counted
as current capacity. Compare equivalent behaviors and record platform, terminal,
versions and bounds. Local opt-in counters for frames, redraws, writes, stale
analysis and layout work should explain costs with negligible disabled overhead;
this is development instrumentation, not telemetry. Gap buffers, ropes or piece
tables are possible outcomes, not predetermined upgrades.

### Command interaction — host knowledge, library mechanics

| Wave | Property to establish |
| --- | --- |
| I0 — ANALYSIS.PROTOCOL | Revision-aware shared analysis of draft/cursor/context, so one host parse can drive multiple features and stale results cannot mutate a newer draft |
| I1 — COMPLETION | Host-defined replacement ranges, display/descriptive metadata and acceptance policy; rich candidates independent of a particular menu |
| I2 — HINT.HIGHLIGHT | Generic hints/autosuggestions and styled spans derived from host analysis |
| I3 — VALIDATION.MULTILINE | Host-defined complete/incomplete/invalid outcomes govern submission, continued editing and diagnostics without embedding a language parser |
| I4 — HISTORY.SEARCH | Navigation separated from storage; bounded memory adapters, custom/persistent providers and incremental/prefix/reverse search with host privacy/retention policy |
| I5 — KEYMAP.EDITING | Key sequences map to generic edit commands independently of the decoder: word operations, undo/redo, search and eventual Emacs/Vi/custom bindings |
| I6 — SECRET.INPUT | Explicit masked/hidden behavior with history denied by default and a clear sensitive-input lifecycle |

Analysis results must identify the revision they describe. Completion, hints,
highlighting and validation should reuse the same derived snapshot; slow host
lookups must not overwrite intervening edits. Context fields, candidate tags,
selection and semantic word-boundary shapes remain design questions. REPLAI
never acquires the host parser, command registry, filesystem lookup or history
storage policy merely because it can present their results.

### Output and presentation — long-running terminal-native interaction

| Wave | Property to establish |
| --- | --- |
| O0 — OUTPUT.MODEL | Safe text by default, semantic styled spans, and only a deliberately admitted trusted terminal-output boundary; untrusted OSC/DCS/control data must not acquire terminal authority |
| O1 — STREAMING.OUTPUT | Characterize and optimize sustained chunks from any host, including build logs, debugger events and model text |
| O2 — MULTIPLEXED.INTERACTION | Background/streaming output coexists with an editable draft, preserving cursor, draft and terminal ownership under explicit arbitration |
| O3 — TRANSIENT.FEEDBACK | Generic notices/diagnostics and transient feedback without owning product rendering |
| U0 — PROMPT.LAYOUT | Composable primary/context/state/continuation grammar and optional right-side information with correct width accounting |
| U1 — COMPLETION.UX | Inline, list, cycling and menu strategies over the completion contract |
| U2 — MULTILINE.UX | Substantial multiline continuation, wrapping, cursor geometry and diagnostic presentation |
| U3 — VISUAL.SYSTEM | Coherent semantic roles, themes, spacing, accessibility and terminal-native background behavior |

Exclusive execution remains useful: submit, release editing, emit host output,
then reopen. Concurrent editing is a separate, harder contract. Output arbitration
must coordinate the editing surface, notices and completion presentation without
interpreting the stream's product meaning. Long output and a host emitting
5–200 chunks per second are workloads to characterize, not token semantics or
performance promises. No full-screen dashboard or hidden theme replacement is
implied by a richer line surface.

### Platforms, robustness and a qualified ecosystem

| Wave | Property to establish |
| --- | --- |
| X0 — BACKEND.SEPARATION | Platform-independent editor/render/interaction logic separated from OS terminal resources and protocols |
| X1 — MACOS.QUALIFICATION | A second operating system qualified by executable lifecycle and PTY evidence |
| X2 — WINDOWS.CONPTY.INVESTIGATION | Independent backend feasibility and qualification, without premature compatibility branches in the Unix implementation |
| Q0 — FUZZ.PROPERTY | Decoder/state-machine fuzzing and arbitrary edit-sequence invariants |
| Q1 — RESOURCE.FAILURE.STRESS | Repeated lifecycle, resize/paste/output storms, descriptor exhaustion and controlled I/O/allocation failures where simulatable |
| Q2 — PERFORMANCE.REGRESSION | Stable, meaningful performance properties become guarded regressions after baseline variance is understood |
| E0 — PACKAGING | Reproducible Cargo, C ABI installation, pkg-config, CMake and static/shared external builds |
| E1 — CONSUMER.MATRIX | Small genuinely distinct Rust/C/C++ command, SQL-like, debugger and streaming consumers over the same implementation |
| E2 — INTEGRATION.COOKBOOK | Copyable patterns derived from those executed consumers, rather than speculative wrappers |
| E3 — API.FREEZE.CANDIDATE | Audit ownership, ergonomics, failure/lifetime behavior and cross-language compatibility before any promise |
| V0 — RELEASE.QUALIFICATION | Exact platform, API, ABI, packaging and support claims backed by reproducible evidence |
| V1 — PUBLIC.0.1 | Consider publication and the first compatibility commitment only after release qualification |

Robustness work should accompany changing boundaries even though its complete
qualification series follows them. BSD and further language wrappers are future
investigations, not implied support. The C ABI supplies a language-neutral
embedding route; it does not require publishing wrappers for every language.
Release is the conclusion of qualified ownership and real integration, not a
calendar milestone or a version number already present in Cargo metadata.
