# Project status

## At a Glance / Current Snapshot

| Axis | Current truth |
| --- | --- |
| Project target | Embeddable command-line interaction infrastructure: a simple entry that can grow into rich, long-lived host-driven interfaces over one engine. |
| Current selected engineering boundary | **COMPLETION.CONTRACT.0 — ACTIVE**: I1/U1 candidate delivery and presentation are being qualified over I0. |
| Latest major completed boundary | I0 analysis protocol: shared immutable snapshots, draft revision identity and stale-safe replacement; [native/portable and performance qualification][analysis-protocol]. |
| Most important structural gap | Rich completion, hints/highlighting and validation remain open above the qualified shared draft-provenance boundary. |
| Executable foundation | Platform-neutral engine; bounded Unicode/grapheme editor; history navigation; completion requests; paste, interrupts/EOF, resize, safe output and exact restoration. |
| Qualified platforms | Linux/macOS: real Rust/C terminal runtime. Windows: portable engine/document tests only, no terminal backend. |
| Current Rust surface | Editor/Interaction, blocking results, session events, portable wake/deadline/admission types, borrowed POSIX readiness, revision/snapshot/stale outcomes, prompts/themes and structured documents. Pre-release, without API freeze. |
| Current C surface | ABI 1: POSIX descriptor binding, static/shared artifacts, caller-owned buffers and plain coordinated output. No structured-document or revision-aware analysis C interface. |
| Performance posture | P0/P1/P2 preserved at matched workloads; driven idle requires no periodic library wake, while compatibility polling retains its 100 ms resize observation. No universal latency gate. |
| Presentation posture | Safe spans, headings, facts, lists, responsive tables/status, composed prompts/themes and deterministic plain output. Completion UI and broader visual refinement remain incomplete. |
| Consumer posture | Native Rust and C consumers are external owners. Exact pins, adoption, application mappings and publication are their decisions; producer metadata assigns no migrations. |
| Public-release posture | Pre-release; no stable Rust API, ABI longevity, SemVer or MSRV promise and no release date. |
| Next decision point | Assess I1/U1 versus I3/U2 and I2 using concrete host needs; I0 supplies provenance, not feature semantics. No next implementation is selected. |

This is the sole authority for **public macro state, maturity, strategic programs,
dependency ordering and release progression**. [README](README.md) owns first use;
[architecture][architecture] and contracts own implementation truth; engineering
dossiers own bounded evidence; Git owns chronology. Planned properties are not
APIs. F1/P3/F2 are complete at their stated scope. I0 is complete at its stated scope; later boundaries require separate authorization.

Navigate: [maturity](#system-maturity) · [programs](#strategic-programs) ·
[completed boundaries](#completed-boundaries) · [sequence](#current-execution-sequence) ·
[release](#release-progression) · [promotion](#promotion-and-living-update-discipline).

## System Maturity

| State | Meaning |
| --- | --- |
| 🟢 ESTABLISHED | Generic property implemented and qualified at the exact scope stated in its row. |
| 🟡 PARTIAL | Real foundation exists; the adopted generic boundary or evidence remains incomplete. |
| 🔴 OPEN | Adopted capability or architectural property is absent or unqualified. |
| ⚪ LATER | Deliberately beyond the current dependency horizon; not silently abandoned. |

Maturity is independent of temporal execution status. An OPEN property need not
be a currently blocked task; completing a bounded wave does not establish its
whole program. Counts describe rows, never percentage completion. IDs are stable
control identifiers, not new public API or producer-capability declarations.

<!-- maturity-counts:start -->
ESTABLISHED=24 PARTIAL=11 OPEN=7 LATER=4 TOTAL=46
<!-- maturity-counts:end -->

<!-- maturity:start -->
### Interaction kernel

| ID | Property | Maturity | Current truth / exact boundary | Promotion condition | Program | Evidence / owner |
| --- | --- | --- | --- | --- | --- | --- |
| interaction.editing | Bounded Unicode editing | 🟢 ESTABLISHED | UTF-8 storage and extended-grapheme cursor/atomic replacement; no terminal I/O in the editor. | Preserve rejection atomicity and combining/CJK/ZWJ oracles on supported backends. | F | [Interaction][interaction]; [core tests][core-tests]; [Unicode oracle][unicode-tests] |
| interaction.history_navigation | History navigation | 🟢 ESTABLISHED | Bounded memory history and exact original draft/cursor return; admission is host-owned. | Preserve draft return independently of future storage/search providers. | I | [Interaction][interaction]; [core tests][core-tests] |
| interaction.completion_request | Completion mechanics | 🟢 ESTABLISHED | Request event plus host-selected grapheme-aligned replacement; no candidate discovery. | Preserve validated replacement and refusal through Rust/C. | I | [Interaction][interaction]; [PTY][pty] |
| interaction.outcomes | Submit, interrupt and EOF | 🟢 ESTABLISHED | Distinct outcomes; transport EOF differs from delete-on-nonempty Ctrl-D. Host owns cancellation meaning. | Keep deterministic/backend outcome parity and restoration. | F | [Engine tests][engine-tests]; [PTY][pty] |
| interaction.paste | Multiline and paste framing | 🟢 ESTABLISHED | Bounded atomic paste normalization, no shortcut execution inside paste; Enter submits complete edited text. | Preserve exact multiline bytes and malformed-sequence progress. | I | [Decoder][decoder]; [PTY][pty] |

### Host integration

| ID | Property | Maturity | Current truth / exact boundary | Promotion condition | Program | Evidence / owner |
| --- | --- | --- | --- | --- | --- | --- |
| embedding.session | Explicit interaction ownership | 🟢 ESTABLISHED | Movable state, scoped open/poll/close/reopen and host-owned execution; Linux/macOS acquisition. | Preserve lifecycle/error semantics and public/native C composition. | F | [Interaction][interaction]; [facade][facade]; [C contract][c-api] |
| embedding.tiers | F1 simple and layered embedding | 🟢 ESTABLISHED | One retained Interaction supplies typed blocking reads, explicit sessions and host driving on Linux/macOS; C ABI 1 retains session compatibility. | Preserve all three native facades over one engine, with real acquisition/restoration and explicit cross-language limits. | F / P | [Embedding qualification][embedding]; [interaction][interaction] |
| embedding.driver | P3 public event driver | 🟢 ESTABLISHED | Host-owned readiness wait, opaque monotonic deadlines and explicit resize; no required periodic driven idle wake. Pending input and observable ordering retained. | Preserve real native reactor, deadline, resize, output and resource oracles without acquiring host scheduling. | P / F | [Embedding qualification][embedding]; [driver][driver] |
| analysis.revisions | I0 shared host analysis | 🟢 ESTABLISHED | Editor-owned identity, shared immutable snapshots and atomic stale-safe replacement; native Linux/macOS delayed analysis qualified, portable model on Windows. | Preserve exact text/cursor/lifecycle provenance, stale refusal and allocation-free revision bookkeeping; future feature protocols reuse this identity. | I / F | [I0 qualification][analysis-protocol]; [analysis contract](docs/interaction.md#revision-aware-host-analysis) |

### Presentation

| ID | Property | Maturity | Current truth / exact boundary | Promotion condition | Program | Evidence / owner |
| --- | --- | --- | --- | --- | --- | --- |
| presentation.prompt | U0 composed prompts | 🟢 ESTABLISHED | Safe primary/continuation spans and semantic roles; simple constructor retained. | Preserve cell geometry, default background and simple embedding across qualified terminals. | U | [Presentation][presentation]; [document tests][documents] |
| presentation.structured_output | O0 semantic documents | 🟢 ESTABLISHED | Rust paragraphs/headings, key/value, lists, literal blocks and spacing; safe standalone/coordinated output. | Preserve bounds, no terminal injection and meaningful styled/plain forms. | O / U | [Presentation][presentation]; [document tests][documents] |
| presentation.table | Responsive tables | 🟢 ESTABLISHED | Cell-width layout, wrapped cells and narrow record stacking; no horizontal viewport. | Preserve content and bounded work under narrow/wide Unicode inputs. | U | [Document tests][documents]; [presentation][presentation] |
| presentation.status | Semantic severity | 🟢 ESTABLISHED | Status has textual cues in plain output; color is not the only distinction. | Keep severity legible under NO_COLOR and captured output. | U | [Presentation][presentation]; [document tests][documents] |
| presentation.theme | Theme foundation | 🟢 ESTABLISHED | Explicit role styles and emphasis, with terminal-default background and safe spans. | Preserve style inheritance and explicit default emphasis without ANSI injection. | U | [Presentation][presentation]; [document tests][documents] |
| presentation.completion_ux | U1 completion presentation | 🔴 OPEN | A request/replacement event is not a candidate list, cycling strategy or menu. | Qualify presentation against the rich I1 contract, including narrow/plain behavior. | U / I | [Presentation owner][presentation]; [interaction][interaction] |
| presentation.multiline_ux | U2 substantial multiline UX | 🟡 PARTIAL | Continuation, wrapping, viewport and resize work; richer diagnostics and large-edit UX remain incomplete. | Qualify substantial multiline editing/diagnostics and width changes without losing cursor/content. | U / I | [PTY][pty]; [layout oracle][layout-tests] |
| presentation.visual_system | U3 visual refinement | 🟡 PARTIAL | Roles/themes, spacing and plain hierarchy exist; no complete accessibility/UX qualification across future surfaces. | Review coherent prompt, candidate, diagnostic and output hierarchy across capabilities. | U | [Presentation][presentation]; [document tests][documents] |

### Output coordination

| ID | Property | Maturity | Current truth / exact boundary | Promotion condition | Program | Evidence / owner |
| --- | --- | --- | --- | --- | --- | --- |
| output.coordinated | Synchronous and exclusive output | 🟢 ESTABLISHED | Safe text/documents preserve active draft/cursor; submit releases editing before host execution and later reopen. | Retain failure cleanup, exact draft return and Rust/C plain compatibility. | O | [Interaction][interaction]; [PTY][pty]; [C PTY][c-pty] |
| output.streaming | O1 sustained output | 🔴 OPEN | Bounded output measurements exist; no sustained-stream contract or stream-specific optimization qualification. | Measure long-running chunk workloads, bytes/writes/backpressure and restoration with host-owned meaning. | O / P | [P0 output evidence][p0]; [presentation owner][presentation] |
| output.multiplexed | O2 editing with background output | 🔴 OPEN | Serialized transactions do not establish independent concurrent writers or output arbitration. | Qualify interleaved input/output, bounded scheduling and exact draft/cursor preservation. | O / P | [Interaction owner][interaction]; [F0][f0] |
| output.transient | O3 transient feedback | 🔴 OPEN | Persistent status blocks are not transient notices, expiry or replacement surfaces. | Define lifetime/removal and redraw evidence without product rendering semantics. | O / U | [Presentation owner][presentation] |

### Command interaction

| ID | Property | Maturity | Current truth / exact boundary | Promotion condition | Program | Evidence / owner |
| --- | --- | --- | --- | --- | --- | --- |
| completion.candidates | I1 rich completion contract | 🟡 PARTIAL | Safe replacement mechanics exist; no rich display/description/acceptance candidate protocol. | Qualify host-defined candidates and stale-result handling independently of U1 presentation. | I | [Interaction][interaction]; [architecture owner][architecture] |
| analysis.hints_highlight | I2 hints and highlighting | 🔴 OPEN | Document spans exist; editor hints, autosuggestions and host syntax spans do not. | Reuse revision-bound analysis with safe range/style validation and plain degradation. | I / U | [Presentation owner][presentation] |
| analysis.validation | I3 validation and submission policy | 🔴 OPEN | Multiline bytes can be edited; no host complete/incomplete/invalid submission decision. | Prove continued editing, submit and diagnostics with a language-neutral host validator. | I | [Interaction owner][interaction] |
| history.storage_search | I4 history provider/search | 🟡 PARTIAL | Memory navigation exists; no storage-provider or search boundary. | Separate navigation from storage with bounded search, draft return and host retention/privacy policy. | I | [Core][core]; [interaction][interaction] |
| editing.keymap | I5 configurable editing | 🟡 PARTIAL | Normalized actions and a fixed compatibility keymap exist; no configurable modes, general undo or search. | Feed common edit operations from different mappings without changing decoder/storage authority. | I | [Keymap][keymap]; [F0][f0] |
| input.sensitive | I6 sensitive input | ⚪ LATER | No masked/hidden mode or secret-specific history posture. Deferred behind embedding and editing-policy contracts. | Explicit authorization plus display/history/lifecycle leakage tests before admitting secrets. | I | [Interaction owner][interaction] |

### Terminal and platform

| ID | Property | Maturity | Current truth / exact boundary | Promotion condition | Program | Evidence / owner |
| --- | --- | --- | --- | --- | --- | --- |
| platform.separation | X0 protocol/resource separation | 🟢 ESTABLISHED | One neutral engine and semantic mutations; VT encoding and POSIX resources are separate. Established by F0/shared POSIX work, not a new X0 wave claim. | Add realizations without OS policy entering the editor or forking the engine. | X / F | [Architecture][architecture]; [conformance][conformance] |
| platform.posix_linux | Linux terminal runtime | 🟢 ESTABLISHED | Real Rust/C PTYs, static/shared installation, termios/FD cleanup and Valgrind. | Preserve actual terminal and resource evidence on changed boundaries. | X / Q | [PTY][pty]; [C qualification][c-qualification]; [CI][ci] |
| platform.posix_macos | X1 macOS terminal runtime | 🟢 ESTABLISHED | Shared POSIX runtime with real Rust/C PTYs and native leak qualification. | Preserve native execution and common-engine parity, not compilation alone. | X / Q | [macOS dossier][macos-perf]; [CI][ci] |
| platform.windows_core | Windows portable engine | 🟢 ESTABLISHED | Native deterministic engine/document tests; no Windows terminal acquisition. | Retain portable execution; do not promote this row into runtime support. | X | [Conformance][conformance]; [CI][ci] |
| platform.windows_runtime | X2 Windows runtime | 🔴 OPEN | No Console/ConPTY backend or HANDLE-based acquisition contract. | Qualify real Windows input/output, mode restoration and ownership; resolve future C binding separately. | X / F | [Architecture owner][architecture]; [C contract][c-api] |
| terminal.capabilities | F2 capabilities/degradation | 🟢 ESTABLISHED | Shared admission separates observed resources, protocol assumptions, requirements and optional policy; explicit UnicodeNarrow width. Linux/macOS runtime; Windows portable resolver only. | Preserve negative admission, optional degradation, width and lifecycle evidence when adding realizations; no active probing or universal emulator agreement claimed. | F / X | [Capability contract][interaction]; [F2 evidence][f2] |

### Performance and robustness

| ID | Property | Maturity | Current truth / exact boundary | Promotion condition | Program | Evidence / owner |
| --- | --- | --- | --- | --- | --- | --- |
| performance.baseline | P0 characterization | 🟢 ESTABLISHED | Versioned components, allocations, PTY/idle/output and pinned comparisons at recorded workloads. | Retain workload/toolchain/variance identity; label unsupported workloads and new platforms separately. | P | [P0 dossier][p0]; [performance tools][perf-tools] |
| performance.editor | P1 editor redesign | 🟢 ESTABLISHED | Measured local grapheme-boundary work retains String storage and Unicode oracle parity. | Preserve semantics and repeat matched scaling measurements for changes. | P | [Combined dossier][macos-perf]; [Unicode oracle][unicode-tests] |
| performance.render | P2 common layout/render redesign | 🟢 ESTABLISHED | Viewport storage, geometry reuse and changed-row rendering; bounded fallbacks remain. | Recheck exact workloads, allocations, bytes and isolated-key latency; no universal speed-ranking claim. | P | [Combined dossier][macos-perf]; [layout oracle][layout-tests] |
| performance.regression | Q2 performance regression policy | 🟡 PARTIAL | Benchmark result/driver integrity runs in CI; stable broad latency/resource thresholds are not established. | Admit repeatable metrics with noise budgets and justified failure thresholds. | Q / P | [Performance result tests][perf-tests]; [CI][ci] |
| robustness.fuzz_property | Q0 general state-machine robustness | 🟡 PARTIAL | Deterministic negative tests and generated Unicode/layout oracles exist; no general fuzz campaign qualification. | Exercise arbitrary edit/decoder sequences with reproducible failures and retained invariants. | Q | [Unicode tests][unicode-tests]; [decoder][decoder]; [conformance][conformance] |
| robustness.resource_stress | Q1 resource/failure stress | 🟡 PARTIAL | Exact restoration, repeated FD lifecycle and bounded I/O/memory failures qualified; not a general storm/exhaustion campaign. | Stress resize/paste/output and exhaustion with no stuck mode, leaked ownership or unbounded work. | Q | [C contracts][c-tests]; [C qualification][c-qualification]; [PTY][pty] |

### Packaging, ecosystem and release

| ID | Property | Maturity | Current truth / exact boundary | Promotion condition | Program | Evidence / owner |
| --- | --- | --- | --- | --- | --- | --- |
| packaging.integration | E0 packaging | 🟡 PARTIAL | Cargo and staged C static/shared/pkg-config work; no standardized CMake/public package release surface. | Qualify clean external installations and supported packaging paths without adjacent checkouts. | E | [C contract][c-api]; [foundation tests][foundation] |
| ecosystem.consumers | E1 consumer diversity | 🟡 PARTIAL | Two independent Rust/C product integrations and neutral fixtures; no broad shell/DB/debugger/streaming matrix. | Execute genuinely different hosts against exact contracts, with their own semantic controls. | E | [C example][c-example]; [Rust example][rust-example]; [historical extraction][extraction] |
| ecosystem.cookbook | E2 integration patterns | 🟡 PARTIAL | Executable Rust/C examples and ownership docs exist; qualified three-tier examples exist; a diverse integration cookbook remains incomplete. | Derive copyable recipes from the qualified E1/F1/P3 consumers. | E / F | [Rust example][rust-example]; [C contract][c-api] |
| ecosystem.contract_handoff | Producer contract publication | 🟢 ESTABLISHED | Repository-owned snapshots/fingerprints and consumer-neutral delta; no runtime dependency or assigned migrations. | Keep metadata aligned with exact qualified source; consumers author their own profiles/receipts. | E | [Producer metadata][producer]; [exact publication][carrier] |
| release.api_freeze | E3 API freeze candidate | ⚪ LATER | Pre-release shapes; ABI identity is not a long-term compatibility promise. | Audit all selected public ownership, lifecycle, errors, language and portability contracts after E1 evidence. | E / V | [Architecture owner][architecture]; [C contract][c-api] |
| release.qualification | V0 release qualification | ⚪ LATER | Boundary-specific evidence exists, not a first-release support envelope. | Close the release progression below with exact platform/API/ABI/package claims and reproducible gates. | V | [Development method][development]; [CI][ci] |
| release.public | V1 first public commitment | ⚪ LATER | No package publication or stable compatibility promise is authorized. | Explicit release decision after V0; publish only the qualified support/compatibility scope. | V | [Release progression](#release-progression); [development][development] |
<!-- maturity:end -->

## Strategic Programs

Programs own gaps, not serial time slots. Maturity here describes each program's
whole adopted target; it is not counted again in the row summary. F/P/I work can
co-evolve, Q accompanies every affected boundary, and platform/packaging evidence
can run independently when their prerequisites are explicit.

<!-- programs:start -->
| Program | Target property | Maturity | Completed foundations | Remaining gaps | Dependencies | Explicit non-goals |
| --- | --- | --- | --- | --- | --- | --- |
| F | One engine, simple/session/driven embedding | 🟢 ESTABLISHED | F0 engine; F1 embedding; F2 scoped capability/admission contract | Preserve the same contracts as future platforms and analyses are introduced | Qualified P3 delivery; X resource constraints; P0 evidence | Scheduler, async runtime, cosmetic API churn |
| P | Measurable, efficient interaction under host driving | 🟡 PARTIAL | P0 baseline; P1/P2 convergence; P3 external driving | Broaden measured envelopes and noise policy without generalizing wins | F1/F2 contracts; Q2 variance policy | Intuitive buffer rewrites; fastest-library claims |
| I | Rich command interaction from host analysis | 🟡 PARTIAL | Revision-aware shared snapshots, completion requests, history mechanics, fixed normalized actions | I1–I5 candidates/validation/storage/keymaps; I6 later sensitive input | F1/P3 delivery/revision rules; U presentation; Q bounds | Parser, command language, history database |
| O | Safe output that scales beyond exclusive phases | 🟡 PARTIAL | O0 documents and synchronous surface coordination | O1 sustained output; O2 arbitration; O3 transient lifecycle | P3/F2 delivery and capabilities; U geometry; Q stress | Product streams, token semantics, uncontrolled writers |
| U | Coherent line-oriented interaction presentation | 🟡 PARTIAL | U0 prompts; documents, tables, status and theme foundation | U1 candidates; U2 multiline refinement; U3 accessibility/visual system | I1/I3 semantics; F2 degradation; O coordination | Alternate-screen panels, dashboard, product ontology |
| X | System realizations below one generic engine | 🟡 PARTIAL | X0 separation; Linux and X1 macOS runtime; Windows portable core | X2 runtime and its C acquisition design space; other systems later | F1/F2 resource contract; shared Q conformance | Fake support from compilation; speculative OS stubs |
| Q | Reproducible correctness/resource/performance promotion | 🟡 PARTIAL | Deterministic/PTY/ABI oracles, failure cleanup, memory tools, benchmark integrity | Q0 fuzz/property breadth; Q1 storms/exhaustion; Q2 stable regression thresholds | Runs alongside each changed boundary; P0 noise evidence | Test-count maturity; unexecuted platform claims |
| E | Reproducible, understandable independent embedding | 🟡 PARTIAL | Cargo/C installation, examples, two consumers, producer handoff metadata | E0 CMake/package consolidation; E1 diversity; E2 recipes; E3 freeze later | F1/P3 and real platform claims; Q evidence | Consumer migrations by default; mandatory BOUNDARY dependency |
| V | First defensible public compatibility commitment | ⚪ LATER | Exact pre-release source/ABI qualification and CI | V0 support envelope; V1 publication decision | E3 candidate plus Q/X/package qualification | Date-driven release; incidental SemVer/API promises |
<!-- programs:end -->

## Completed Boundaries

This compact foundation is not a second changelog. Implementation and later
qualification/document carriers are distinct; retained measurements apply only
to their recorded source, workload and environment.

| Boundary | Established foundation and evidence identity |
| --- | --- |
| R0–R5 extraction foundation | Independent library, Rust/C boundaries, two consumer substitutions and local legacy-removal qualification. [Retained extraction control][extraction] records the externally owned publication caveat; this roadmap neither reopens extraction implementation nor certifies external publication. Historical qualified consumer contract: `df5538c718b8d068432032e7fb116fb8bfab158e`. |
| F0 PUBLIC.ARCHITECTURE | Neutral engine/actions/render and system/protocol seams: implementation `a8afb8e…`, closure carrier `4ddd24f…`; [F0 dossier][f0]. |
| P0 PERFORMANCE.BASELINE | Characterization implementation `d800083…` plus corrections; evidence carrier `8210c93…`; [P0 dossier][p0]. |
| X1/P1/P2 combined macOS-performance closure | Shared native POSIX runtime and common redesign; measured source/harness `96f794c7b807636b4fabd3939bbeeab150a87dd1`, closure carrier `6365f84e12865871bf26ecf0d984b48213d81ebc`; [combined dossier][macos-perf]. |
| O0/U0 STRUCTURED.PRESENTATION | Implementation `21e3493…`, correctness closures through `b9581102220364b94d2bcdef49f602308d24e6c9`; [presentation contract][presentation] and [exact qualification][presentation-ci]. Richer U/O work remains separate. |
| F1/P3 EMBEDDING.CONTRACT | Native blocking/session/driven contract and minimal admission; implementation `349d75e…`, native qualification `5c04277…`; [bounded evidence][embedding]. Its minimal admission is completed by F2 below; O2 remains incomplete. |
| F2 TERMINAL.CAPABILITIES | Unified acquisition/admission, inspectable snapshot and deterministic width; implementation `12cef7608d8243383b432f303fd3a7218cb9b20c`, qualification observer correction `69205d104a2a2bdd88b9ad03176fc58b5a3e266b`; [native/portable, memory and performance evidence][f2]. No consumer mutation. |
| BOUNDARY producer metadata | Metadata/README carrier `9d9375db407246a6f4946e20e38f56fe3155e62a` describes qualified source `b958110…` and the delta from `6365f84…`; [producer metadata][producer]. No runtime/API change or consumer repin. |
| I0 ANALYSIS.PROTOCOL | Implementation `81d102e74e0aa0b42aa3ad569315c00c3d34b675`, qualified observer carrier `39d41fef51012a8b4e509928846489026d908654`; [I0 dossier][analysis-protocol]. C ABI 1 remains synchronous. |

## Current Execution Sequence

COMPLETION.CONTRACT.0 combines I1/U1 over qualified I0 provenance and the
F1/P3/F2 embedding foundation. Candidate delivery and temporary selection must
qualify together; neither a data structure alone nor one rendered menu closes
the boundary. I2, I3/U2 and history remain separate, unstarted decisions.

| Dependency frontier | Next decision / evidence | Explicit limit |
| --- | --- | --- |
| F1/P3/F2 qualified foundation | Reuse one engine, host-owned readiness and explicit admission in later contracts. | Linux/macOS qualification does not establish Windows runtime or active capability discovery. |
| I0 qualified provenance | Reuse snapshots and stale-safe application for future host analysis contracts. | No parser, rich completion, hint/highlight or validation semantics follow from revision identity. |
| I1–I5 with U1–U3; O1–O3 | Select bounded analysis/output/presentation intersections, measured against P baselines and Q stress. | An application reactor is not independent concurrent writers or O2 arbitration. |
| X/E/Q and release | Retain native runtime/portable limits, strengthen failure and consumer diversity, then evaluate API freeze. | No Windows runtime, release or consumer repin follows automatically. |

Evidence may reorder these dependencies. No later scope is started or authorized
by this sequence; each needs its own explicit engineering decision.

## Ownership and Consumer Posture

```text
HOST: commands, parser, application semantics, execution,
      candidate discovery, history policy, semantic classification
                         ↓ public REPLAI contracts
REPLAI: editing, interaction mechanics, generic presentation,
        terminal lifecycle, safe output coordination
                         ↓ platform realization
Linux / macOS terminal runtime; future Windows runtime
```

YAI and YVEX own their adapters, frontend decisions, application semantics,
qualification and publication. Consumer repinning requires external authorization
and does not follow every library commit. The producer manifest exposes generic
contract deltas; it assigns no consumer migration. Consumer state blocks an
independent library boundary only when it exposes a **reproducible generic
REPLAI contract defect**, not because a downstream publication is pending.

BOUNDARY metadata concerns cross-repository contract evolution. It is neither
an engine component nor a build/runtime dependency. This roadmap changes no
producer capability meaning, consumer profile, pin or receipt. Control IDs and
maturity describe project direction; a producer manifest describes an exact
implemented source contract. Neither substitutes for the other.

## Release Progression

| Gate | Required promotion evidence |
| --- | --- |
| Embedding and capabilities | F1/P3 ownership and F2 degradation are coherent; selected Rust/C exposure is explicit, with unsupported surfaces named. |
| Robustness and platform envelope | Relevant Q0/Q1 failure/invariant coverage, justified Q2 metrics and real X runtime evidence for every claimed platform. Windows need not block a deliberately narrower release, but must never be implied supported. |
| Reproducible ecosystem | E0 clean packages/installations, E1 different real hosts and E2 usable recipes; no adjacent source or accidental system library. |
| E3 freeze candidate | Review public types, errors, lifetimes, resource ownership, API/ABI evolution and compatibility policy against those consumers. |
| V0 release qualification | Repeat the exact chosen support envelope with pinned artifacts, docs and reproducible CI; record exclusions and residual risks. |
| V1 public commitment | Separate authorization to publish packages and the first compatibility promise. No date or version number alone establishes readiness. |

The adopted capability map is broader than the eventual first release. A smaller
release requires an explicit scope decision here; it cannot erase unresolved
rows or silently treat excluded features as completed.

## Explicit Nonclaims

REPLAI is not a shell framework, full-screen TUI, async runtime, application
command language, parser, agent framework, history database or product rendering
ontology. Hosts own those meanings and systems.

There is **no Windows terminal runtime**, stable Rust API/long-term ABI promise,
universal terminal fallback or support for concurrent independent writers.
Structured Rust documents are not exposed through C ABI 1. Theme/span support
is not syntax analysis, and basic multiline editing is not host validation.
Internal batching is not public event-loop embedding. Existing performance
comparisons establish exact workload results, not general superiority.

## Promotion and Living-update Discipline

Implementation is not generic maturity; test existence is not qualification;
one platform is not cross-platform support; consumer success is not generic
proof; a benchmark win is not general performance superiority; a planned API is
not an implemented contract.

For each promotion, update its stable maturity row, exact scope, promotion
condition, owner/evidence and affected program/dependencies together. Evidence
must identify source, environment, commands and observed outcomes, including
rejections and failure cleanup where relevant. Keep historical dossiers bounded;
do not relabel an old run as qualification of a new source or environment.

Change the selected boundary only with a documented dependency rationale. Keep
one selection, recompute the row counts, and leave chronology to Git. Update
architecture/contracts when implementation truth changes; keep first-use
instructions in README. A future narrower release or deferred gap remains
visible here. New public macro-status pages are not additional authorities.

`python3 tools/check_docs.py` validates maturity IDs/states/counts, program
references, the unique selection and normal documentation links/anchors. Its
negative fixtures run through `python3 -B tools/test_check_docs.py`. These checks
validate control consistency, **not the truth of a maturity promotion**. Follow
[the development method][development] for qualification and publication.

[architecture]: docs/architecture.md
[interaction]: docs/interaction.md
[presentation]: docs/presentation.md
[c-api]: docs/c-api.md
[development]: docs/development.md
[f0]: docs/engineering/f0.md
[p0]: docs/engineering/p0.md
[macos-perf]: docs/engineering/macos-perf.md
[producer]: .boundary/README.md
[carrier]: https://github.com/mothx9/replai/commit/9d9375db407246a6f4946e20e38f56fe3155e62a
[presentation-ci]: https://github.com/mothx9/replai/actions/runs/34251543122
[ci]: .github/workflows/ci.yml
[core]: src/core.rs
[facade]: src/interaction.rs
[engine-tests]: src/engine.rs
[decoder]: src/input.rs
[keymap]: src/keymap.rs
[capabilities]: src/capabilities.rs
[substrate]: src/substrate.rs
[driver]: src/terminal.rs
[conformance]: src/conformance.rs
[core-tests]: tests/core.rs
[unicode-tests]: tests/unicode_model.rs
[documents]: tests/document.rs
[pty]: tests/pty.rs
[layout-tests]: tests/support/layout_reference.rs
[c-pty]: tools/c_pty.py
[c-tests]: tests/c/contracts.c
[c-qualification]: tools/qualify_c.py
[foundation]: tests/foundation.rs
[perf-tools]: tools/perf/run.py
[perf-tests]: tools/perf/test_results.py
[rust-example]: examples/demo.rs
[c-example]: examples/c/demo.c
[extraction]: https://github.com/mothx9/replai/blob/89d36f8433cd109866ae360d8691dac30b7de026/ROADMAP.md

[embedding]: docs/engineering/embedding.md

[f2]: docs/engineering/terminal-capabilities.md

[analysis-protocol]: docs/engineering/analysis-protocol.md
