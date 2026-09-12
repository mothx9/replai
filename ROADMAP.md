# Project status

## At a Glance / Current Snapshot

| Axis | Current truth |
| --- | --- |
| Project target | Embeddable command-line interaction infrastructure: a simple entry that can grow into rich, long-lived host-driven interfaces over one engine. |
| Current selected engineering boundary | **RELEASE.CANDIDATE.0 — SELECTED_NOT_STARTED**: E3 public API/ABI freeze candidate and V0 frozen-source release qualification; requires separate authorization. |
| Latest major completed boundary | RELEASE.PACKAGING.0: crates.io-ready Rust package, versioned C source SDK/CMake, MSRV, independent debugger consumer and executed cookbook qualification. |
| Most important structural gap | The E3 public-surface freeze and V0 integrated release-candidate audit remain. I4/I5 and broader output/platform work stay deferred from v0.1. |
| Executable foundation | Platform-neutral engine; bounded Unicode/grapheme editor; history navigation; completion requests; paste, interrupts/EOF, resize, safe output and exact restoration. |
| Qualified platforms | Linux/macOS: real Rust/C terminal runtime. Windows: portable engine/document tests only, no terminal backend. |
| Current Rust surface | Editor/Interaction, blocking results, session events, portable wake/deadline/admission types, borrowed POSIX readiness, revision/snapshot/stale outcomes, bounded completion candidates/selection, submission requests/dispositions, diagnostics, editor analysis spans/hints, prompts/themes and structured documents. Pre-release, without API freeze. |
| Current C surface | ABI 1: POSIX descriptor binding, static/shared artifacts, caller-owned buffers and plain coordinated output. No structured-document, revision-aware analysis, rich-candidate or validation C interface. |
| Performance posture | P0/P1/P2 preserved at matched workloads; Q2 freezes 32 bounded latency/allocation/byte workloads with preregistered noise rules. This is not a universal latency SLA or ranking. |
| Presentation posture | Safe spans, headings, facts, lists, responsive tables/status, composed prompts/themes and deterministic plain output. Bounded completion and validated multiline interaction are qualified; bounded host editor spans/hints are qualified; broader visual refinement remains separate. |
| Consumer posture | Packaged Rust and moved-prefix C/C++ consumers are qualified independently. Product consumers remain external owners; producer metadata assigns no migrations or repins. |
| Public-release posture | Pre-release. The v0.1 candidate envelope and compatibility requirements are selected in [release scope](docs/release-scope.md); no freeze, package publication or current stability promotion. |
| Next decision point | Authorize RELEASE.CANDIDATE.0 separately. One engineering wave plus one separately authorized publication wave remains; no new runtime feature is required by the selected scope. |

This is the sole authority for **public macro state, maturity, strategic programs,
dependency ordering and release progression**. [README](README.md) owns first use;
[architecture][architecture] and contracts own implementation truth; engineering
dossiers own bounded evidence; Git owns chronology. Planned properties are not
APIs. F1/P3/F2 are complete at their stated scope. I0 is complete at its stated scope; later boundaries require separate authorization.

Navigate: [maturity](#system-maturity) · [programs](#strategic-programs) ·
[completed boundaries](#completed-boundaries) · [sequence](#current-execution-sequence) ·
[release scope](#first-release-scope) · [release gates](#release-progression) · [promotion](#promotion-and-living-update-discipline).

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
ESTABLISHED=35 PARTIAL=3 OPEN=4 LATER=4 TOTAL=46
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
| presentation.completion_ux | U1 completion presentation | 🟢 ESTABLISHED | Bounded temporary rows; Tab/Shift-Tab selection, explicit accept/dismiss, responsive plain/styled output and shared-frame restoration. Native Linux/macOS; portable model on Windows. | Preserve exact draft/revision/selection across resize and serialized output; retain safe narrow/plain behavior. | U / I | [I1/U1 dossier][completion-contract]; [presentation][presentation] |
| presentation.multiline_ux | U2 substantial multiline UX | 🟢 ESTABLISHED | Validated logical-line navigation, continuation, bounded diagnostic viewport, resize/output preservation; 10/100/1000-line real PTYs on Linux/macOS. | Preserve exact draft/cursor/revision, history edges and measured bounded screen damage; prefix layout traversal remains measured debt. | U / I | [I3/U2 dossier][validation-multiline]; [interaction][interaction] |
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
| completion.candidates | I1 rich completion contract | 🟢 ESTABLISHED | Host-ordered bounded candidates bind DraftRevision; whole-set validation and atomic stale refusal/application. Native Rust only; C ABI 1 retains replacement. | Preserve host discovery/context ownership, bounds, rejection atomicity and delivery-order semantics. | I | [I1/U1 dossier][completion-contract]; [interaction][interaction] |
| analysis.hints_highlight | I2 hints and highlighting | 🟢 ESTABLISHED | Native revision-bound ordered editor style spans and non-canonical bounded hints; stale silence, I1/I3 composition, plain degradation, Linux/macOS PTYs/memory and Windows portable models. | Preserve canonical text/revision, safe grapheme ranges, bounded payload and measured ordinary-editing cost. Host analysis and insertion remain separate. | I / U | [I2 dossier][analysis-presentation]; [interaction][interaction] |
| analysis.validation | I3 validation and submission policy | 🟢 ESTABLISHED | Optional host Complete/Incomplete/Invalid over immutable Enter snapshots; atomic stale refusal, exact submission and bounded diagnostics. Native Rust only. | Preserve host grammar/scheduling authority, rejected-result atomicity and completion precedence across platforms. | I | [I3/U2 dossier][validation-multiline]; [interaction][interaction] |
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
| performance.regression | Q2 performance regression policy | 🟢 ESTABLISHED | Thirty-two named workloads have preregistered median/p95 noise rules plus exact allocation/encoded-byte CI oracles; large multiline traversal and synchronous output remain explicit debt. | Preserve workload/source/machine identity, investigate two-run repeated exceedances, and never tune thresholds after candidate observation. | Q / P | [Hardening dossier][hardening]; [Q2 registration](tools/hardening/evidence/q2-linux-aarch64.json) |
| robustness.fuzz_property | Q0 general state-machine robustness | 🟢 ESTABLISHED | Five distinct Linux campaigns exceed 60 CPU-minutes each; 100,000 seeded paired semantic sequences and final-corpus replay pass on Linux x86_64/ARM64, macOS ARM64 and Windows portable core. | Retain minimized findings, immutable corpus identity and exact platform limits; new surfaces require new targets/evidence. | Q | [Hardening dossier][hardening]; [final corpus](tools/hardening/evidence/final-corpus.json.gz) |
| robustness.resource_stress | Q1 resource/failure stress | 🟢 ESTABLISHED | Each native release target passed 1,000 cycles per Rust tier and C ABI, 10,000 mixed events, eight failure classes ×100, 100 exhaustion children and native memory tools. | Preserve exact connected restoration, explicit impossible-restoration failure, caller ownership and platform-native resource evidence. | Q | [Hardening dossier][hardening]; [native evidence](tools/hardening/evidence/native-campaign.json.gz) |

### Packaging, ecosystem and release

| ID | Property | Maturity | Current truth / exact boundary | Promotion condition | Program | Evidence / owner |
| --- | --- | --- | --- | --- | --- | --- |
| packaging.integration | E0 packaging | 🟢 ESTABLISHED | Unpublished `replai 0.1.0` crate and deterministic C source SDK build outside the checkout; MSRV/stable, docs, relocatable pkg-config/CMake and C11/C++17 static/shared consumers pass on the selected envelope. | Preserve standalone package/SDK identity, relocation and exact native/portable evidence; publication remains separate. | E | [Packaging dossier][packaging]; [C SDK][c-sdk] |
| ecosystem.consumers | E1 consumer diversity | 🟢 ESTABLISHED | Independent packaged debugger host exercises I0/I1/I2/I3, finite serialized notices and structured output on real Linux/macOS PTYs; portable state runs on Windows. | Requalify genuinely distinct external hosts when their consumed surface or package contract changes. | E | [Packaging dossier][packaging]; [debugger fixture](tools/package/debugger-consumer/src/main.rs) |
| ecosystem.cookbook | E2 integration patterns | 🟢 ESTABLISHED | Nineteen executed or explicit-deferred recipes cover three Rust tiers, analysis, history ownership, output limits, Rust/C/CMake and plain mode. | Keep recipes tied to executable examples/artifacts and preserve explicit host ownership and unsupported postures. | E / F | [Use cases](docs/use-cases.md); [Packaging dossier][packaging] |
| ecosystem.contract_handoff | Producer contract publication | 🟢 ESTABLISHED | Repository-owned snapshots/fingerprints and consumer-neutral delta; no runtime dependency or assigned migrations. | Keep metadata aligned with exact qualified source; consumers author their own profiles/receipts. | E | [Producer metadata][producer]; [exact publication][carrier] |
| release.api_freeze | E3 API freeze candidate | ⚪ LATER | Pre-release shapes; first-release audit is mandatory but not started. ABI identity is not yet a long-term compatibility promise. | Audit all selected public ownership, lifecycle, errors, language and portability contracts after E1 evidence. | E / V | [Architecture owner][architecture]; [C contract][c-api] |
| release.qualification | V0 release qualification | ⚪ LATER | Candidate support envelope selected; no integrated frozen-source/package release qualification yet. | Close the release progression below with exact platform/API/ABI/package claims and reproducible gates. | V | [Development method][development]; [CI][ci] |
| release.public | V1 first public commitment | ⚪ LATER | No package publication or effective stable compatibility promise; V1 requires separate authorization after V0. | Explicit release decision after V0; publish only the qualified support/compatibility scope. | V | [Release progression](#release-progression); [development][development] |
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
| I | Rich command interaction from host analysis | 🟡 PARTIAL | Revision-aware shared snapshots, rich candidates, host validation/diagnostics, safe editor spans/hints, history mechanics, fixed normalized actions | I4 storage/search; I5 keymaps; I6 later sensitive input | F1/P3 delivery/revision rules; U presentation; Q bounds | Parser, command language, history database |
| O | Safe output that scales beyond exclusive phases | 🟡 PARTIAL | O0 documents and synchronous surface coordination | O1 sustained output; O2 arbitration; O3 transient lifecycle | P3/F2 delivery and capabilities; U geometry; Q stress | Product streams, token semantics, uncontrolled writers |
| U | Coherent line-oriented interaction presentation | 🟡 PARTIAL | U0 prompts; U1 candidates; U2 validated multiline/diagnostics; documents, tables, status and themes | U3 accessibility/visual system | I1/I3 semantics; F2 degradation; O coordination | Alternate-screen panels, dashboard, product ontology |
| X | System realizations below one generic engine | 🟡 PARTIAL | X0 separation; Linux and X1 macOS runtime; Windows portable core | X2 runtime and its C acquisition design space; other systems later | F1/F2 resource contract; shared Q conformance | Fake support from compilation; speculative OS stubs |
| Q | Reproducible correctness/resource/performance promotion | 🟡 PARTIAL | Q0 five-boundary fuzz/property campaigns; Q1 native resource/failure stress; Q2 bounded preregistered regression policy | Replay/grow evidence for future surfaces and final frozen package/candidate identities | Runs alongside each changed boundary; P0 noise evidence | Test-count maturity; unexecuted platform claims |
| E | Reproducible, understandable independent embedding | 🟡 PARTIAL | Versioned crate/C SDK, relocatable pkg-config/CMake, executed cookbook, independent packaged debugger and producer handoff metadata | E3 public-surface freeze candidate | F1/P3 and real platform claims; Q evidence | Consumer migrations by default; mandatory BOUNDARY dependency |
| V | First defensible public compatibility commitment | ⚪ LATER | Exact pre-release source/ABI qualification and CI; selected v0.1 envelope | V0 integrated candidate/artifact qualification; V1 publication decision | E3 candidate plus Q/X/package qualification | Date-driven release; incidental SemVer/API promises |
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
| I1/U1 COMPLETION.CONTRACT | Implementation `05d574caf709f596269072ac8348611823555992`, qualified measurement-fixture carrier `9f245f115a12d781f1fb279d1b8da08b89f780ec`; [native/portable, memory and performance evidence][completion-contract]. C ABI 1 stays exact. |
| I3/U2 VALIDATION.MULTILINE | Implementation `1da4dbf9162a2fea4d267e1aa473859ee64ae290`; [native/portable, memory and performance evidence][validation-multiline]. C ABI 1 stays exact; Rust Event gains opt-in SubmissionRequested. |
| I2 ANALYSIS.PRESENTATION | Implementation `e0ab06bc2e9111b41968842b459c51a68855aa13`; fixture carrier `27981f0dbb22305f96a07bb7e4c7fafbbf81e90b`; [native/portable, memory and performance evidence][analysis-presentation]. Editor and C ABI 1 unchanged. |
| RELEASE.HARDENING.0 | Five 60-CPU-minute fuzz/property campaigns, 100,000 generated semantic pairs, cross-platform corpus replay, native lifecycle/failure/memory stress and preregistered Q2 policy; [hardening dossier][hardening]. Runtime/API/C ABI unchanged; U3 and deferred features remain open. |
| RELEASE.PACKAGING.0 | Unpublished `replai 0.1.0` crate, deterministic versioned C source SDK, Rust 1.98.1/current-stable qualification, relocatable pkg-config/CMake consumers, independent packaged debugger and 19-recipe cookbook; [packaging dossier][packaging]. Runtime/API/C ABI unchanged; no publication or consumer repin. |

## First Release Scope

The adopted v0.1 candidate is the current Rust interaction/analysis/presentation
surface on Linux/macOS, portable Windows core, and bounded C ABI 1 sessions.
Output is synchronous and host-serialized. No sustained active-edit stream or
independent writer service is promised. [Release scope](docs/release-scope.md)
owns the detailed support, compatibility, packaging and G1–G6 acceptance contract.
It is a release design, not another maturity/status authority.

Every non-established property is classified below.

<!-- release-counts:start -->
MUST_V0_1=3 SHOULD_V0_1=1 V0_2=6 LATER=1 OUT_OF_SCOPE=0 TOTAL=11
<!-- release-counts:end -->

No adopted remaining row is discarded as OUT_OF_SCOPE; host history databases, parser/command semantics and application
scheduling remain architectural exclusions. V0_2 means the first post-release
planning pool, not a delivery promise. Maturity is unchanged by classification.

<!-- release-scope:start -->
| Capability | Current maturity | v0.1 class | Rationale | Required evidence |
| --- | --- | --- | --- | --- |
| history.storage_search | 🟡 PARTIAL | V0_2 | I4: bounded navigation suffices; provider/reverse search is not advertised by v0.1. Persistence policy stays host-owned. | [History decision](docs/release-scope.md#history); later search/provider contract qualification |
| editing.keymap | 🟡 PARTIAL | V0_2 | I5: fixed documented bindings suffice; word editing/configuration/undo can follow. Vi remains later within this broader row. | [Editing decision](docs/release-scope.md#editing-policy-and-visual-refinement); G2 preserves current bindings |
| input.sensitive | ⚪ LATER | LATER | I6: ordinary editing is not secret entry; leakage-safe masked input is separately scoped. | [Sensitive-input exclusion](docs/release-scope.md#editing-policy-and-visual-refinement); separate future leakage/history campaign |
| presentation.visual_system | 🟡 PARTIAL | SHOULD_V0_1 | U3 broad refinement is optional; existing-surface keyboard/plain usability is mandatory G2. | [Usability floor](docs/release-scope.md#g2--q1-resource-stress-and-baseline-usability); record omitted refinements without claiming U3 closure |
| output.streaming | 🔴 OPEN | V0_2 | O1: host streams after close or serializes bounded notices; no sustained active-edit throughput contract. | [Output envelope](docs/release-scope.md#output-model-clients-and-long-lived-hosts); G2/G4 finite serialized-output proof |
| output.multiplexed | 🔴 OPEN | V0_2 | O2: host event multiplexing already exists; independent writers/arbitration are excluded. | [Output envelope](docs/release-scope.md#output-model-clients-and-long-lived-hosts); later arbitration/scheduling contract |
| output.transient | 🔴 OPEN | V0_2 | O3: persistent status output is sufficient; in-place expiry/replacement is not advertised. | [Output envelope](docs/release-scope.md#output-model-clients-and-long-lived-hosts); later lifetime/removal qualification |
| platform.windows_runtime | 🔴 OPEN | V0_2 | X2: deliberate Linux/macOS release; portable Windows tests do not imply runtime support. | [Platform envelope](docs/release-scope.md#candidate-support-envelope); future real Windows acquisition/restoration |
| release.api_freeze | ⚪ LATER | MUST_V0_1 | E3: audit every selected public contract after hardening and independent package consumers; not 1.0. | [G5](docs/release-scope.md#g5--q2-policy-e3-freeze-and-v0-release-qualification): API/ABI/MSRV and migration review |
| release.qualification | ⚪ LATER | MUST_V0_1 | V0: one exact candidate source and artifact set must satisfy the complete envelope. | [G5](docs/release-scope.md#g5--q2-policy-e3-freeze-and-v0-release-qualification): frozen candidate and installed-artifact qualification |
| release.public | ⚪ LATER | MUST_V0_1 | V1: a first public release includes actual publication/verification, not merely a tag proposal. | [G6](docs/release-scope.md#g6--v1-publication): separate authorization, packages/tag and download verification |
<!-- release-scope:end -->

Existing 35 ESTABLISHED rows retain their scoped contracts and release regression
gates. Packaging promotes only E0/E1/E2; the broader E program remains partial
until E3 and no adjacent runtime capability is promoted.
LATER maturity for E3/V0/V1 records unstarted downstream work; its MUST release
classification makes the dependency explicit without claiming implementation.

## Current Execution Sequence

The selected boundary is `RELEASE.CANDIDATE.0`, selected but not started.
Packaging evidence now supports freezing and qualifying the selected interaction
surface before any publication. The exact remaining plan is **one engineering
wave plus one separately authorized publication wave**, in this dependency order:

| Boundary | Bounded closure | Prerequisite / exit |
| --- | --- | --- |
| RELEASE.CANDIDATE.0 | E3 full API/ABI review and V0 frozen candidate qualification | G1–G4 plus Q2 passed; [G5](docs/release-scope.md#g5--q2-policy-e3-freeze-and-v0-release-qualification) on final source/artifact identities, no publication |
| RELEASE.PUBLICATION.0 | V1 public compatibility commitment and package/tag delivery | Separate explicit release authorization after V0; [G6](docs/release-scope.md#g6--v1-publication), verified downloads/docs |

Preparation may overlap only under a later authorized wave's scope; exits are
ordered. A correctness finding requires repair/requalification, not automatic
feature expansion. If it exposes a new architectural dependency, revise this
sequence explicitly rather than pretending the wave count is immutable.
I4/I5/O1–O3/X2 stay visible in the post-release pool; I6 stays later. No next
engineering implementation or consumer repin is authorized by this selection.

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

The [selected first-release scope](#first-release-scope) fixes what v0.1 means;
[release acceptance](docs/release-scope.md#required-evidence-before-freeze-and-tagging)
defines its gates. Current implementation evidence is not a release receipt.

| Gate | Required promotion evidence |
| --- | --- |
| G1/G2 and Q2 | Bounded fuzz/state campaigns, native stress/cleanup and minimum plain/keyboard usability; meaningful regression thresholds before API freeze |
| G3/G4 | crates.io-ready Rust package and versioned C source SDK; MSRV/current stable, relocated CMake/pkg-config, external Rust/C/C++ hosts and executable recipes |
| E3 / G5 | Ratified 0.1.x Rust/ABI/MSRV compatibility, full public-surface audit, exact frozen candidate and artifact-qualified native/portable evidence |
| V1 / G6 | Separate publication authorization; release only the qualified crate/SDK/tag and verify downloaded artifacts and hosted docs |

No date is selected. Linux/macOS runtime plus Windows portable core is deliberate;
C ABI 1 does not need full Rust parity. History search, configurable keymaps,
streaming arbitration and a Windows terminal backend do not block this envelope.
Deferred features are not complete, and a release claim cannot exceed its gates.

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

[completion-contract]: docs/engineering/completion-contract.md

[validation-multiline]: docs/engineering/validation-multiline.md

[analysis-presentation]: docs/engineering/analysis-presentation.md

[hardening]: docs/engineering/release-hardening.md

[packaging]: docs/engineering/release-packaging.md

[c-sdk]: docs/c-sdk.md
