# Project status

## At a Glance / Current Snapshot

| Axis | Current truth |
| --- | --- |
| Project target | Embeddable command-line interaction infrastructure: a simple entry that can grow into rich, long-lived host-driven interfaces over one engine. |
| Current selected engineering boundary | **INTERACTION.ERGONOMICS.0 — SELECTED_NOT_STARTED**: history/search, Unicode-aware word operations, undo/redo, bounded kill/yank and the common editing-action vocabulary; requires separate authorization. |
| Latest major completed boundary | RELEASE.PACKAGING.0: crates.io-ready Rust package, versioned C source SDK/CMake, MSRV, independent debugger consumer and executed cookbook qualification. |
| Most important structural gap | The expanded v0.1 product still needs daily-driver editing, long-lived output, Windows runtime, sensitive input, coherent UX, large-draft scaling, developer experience and agentic integration before final hardening/freeze. |
| Executable foundation | Platform-neutral engine; bounded Unicode/grapheme editor; history navigation; completion requests; paste, interrupts/EOF, resize, safe output and exact restoration. |
| Qualified platforms | Linux/macOS: real Rust/C terminal runtime. Windows: portable engine/document tests only, no terminal backend. |
| Current Rust surface | Editor/Interaction, blocking results, session events, portable wake/deadline/admission types, borrowed POSIX readiness, revision/snapshot/stale outcomes, bounded completion candidates/selection, submission requests/dispositions, diagnostics, editor analysis spans/hints, prompts/themes and structured documents. Pre-release, without API freeze. |
| Current C surface | ABI 1: POSIX descriptor binding, static/shared artifacts, caller-owned buffers and plain coordinated output. No structured-document, revision-aware analysis, rich-candidate or validation C interface. |
| Performance posture | P0/P1/P2 preserved at matched workloads; Q2 freezes 32 bounded latency/allocation/byte workloads with preregistered noise rules. This is not a universal latency SLA or ranking. |
| Presentation posture | Safe spans, headings, facts, lists, responsive tables/status, composed prompts/themes and deterministic plain output. Bounded completion and validated multiline interaction are qualified; bounded host editor spans/hints are qualified; broader visual refinement remains separate. |
| Consumer posture | Packaged Rust and moved-prefix C/C++ consumers are qualified independently. Product consumers remain external owners; producer metadata assigns no migrations or repins. |
| Public-release posture | Pre-release. The expanded v0.1 product envelope and compatibility requirements are selected in [release scope](docs/release-scope.md); adopted targets are not implementation claims. |
| Next decision point | Authorize INTERACTION.ERGONOMICS.0 separately. Twelve engineering boundaries plus a separately authorized publication boundary remain in the current dependency plan. |

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
ESTABLISHED=35 PARTIAL=15 OPEN=9 LATER=4 TOTAL=63
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
| presentation.visual_system | U3 visual refinement | 🟡 PARTIAL | Roles/themes, spacing and plain hierarchy exist; no complete accessibility/UX qualification across future surfaces. | Qualify coherent prompt, continuation, large-menu, diagnostic, hint, transient, theme and plain/accessibility behavior across selected platforms and widths. | U | [Presentation][presentation]; [document tests][documents] |

### Output coordination

| ID | Property | Maturity | Current truth / exact boundary | Promotion condition | Program | Evidence / owner |
| --- | --- | --- | --- | --- | --- | --- |
| output.coordinated | Synchronous and exclusive output | 🟢 ESTABLISHED | Safe text/documents preserve active draft/cursor; submit releases editing before host execution and later reopen. | Retain failure cleanup, exact draft return and Rust/C plain compatibility. | O | [Interaction][interaction]; [PTY][pty]; [C PTY][c-pty] |
| output.streaming | O1 sustained output | 🔴 OPEN | Bounded output measurements exist; no sustained-stream contract or stream-specific optimization qualification. | Measure long-running chunk workloads, bytes/writes/backpressure and restoration with host-owned meaning. | O / P | [P0 output evidence][p0]; [presentation owner][presentation] |
| output.multiplexed | O2 editing with background output | 🔴 OPEN | Serialized transactions do not establish independent concurrent writers or output arbitration. | Qualify bounded producer arbitration, ordering/backpressure and exact draft/cursor preservation while the host retains scheduling. | O / P | [Interaction owner][interaction]; [F0][f0] |
| output.transient | O3 transient feedback | 🔴 OPEN | Persistent status blocks are not transient notices, expiry or replacement surfaces. | Define lifetime/removal and redraw evidence without product rendering semantics. | O / U | [Presentation owner][presentation] |

### Command interaction

| ID | Property | Maturity | Current truth / exact boundary | Promotion condition | Program | Evidence / owner |
| --- | --- | --- | --- | --- | --- | --- |
| completion.candidates | I1 rich completion contract | 🟢 ESTABLISHED | Host-ordered bounded candidates bind DraftRevision; whole-set validation and atomic stale refusal/application. Native Rust only; C ABI 1 retains replacement. | Preserve host discovery/context ownership, bounds, rejection atomicity and delivery-order semantics. | I | [I1/U1 dossier][completion-contract]; [interaction][interaction] |
| analysis.hints_highlight | I2 hints and highlighting | 🟢 ESTABLISHED | Native revision-bound ordered editor style spans and non-canonical bounded hints; stale silence, I1/I3 composition, plain degradation, Linux/macOS PTYs/memory and Windows portable models. | Preserve canonical text/revision, safe grapheme ranges, bounded payload and measured ordinary-editing cost. Host analysis and insertion remain separate. | I / U | [I2 dossier][analysis-presentation]; [interaction][interaction] |
| analysis.validation | I3 validation and submission policy | 🟢 ESTABLISHED | Optional host Complete/Incomplete/Invalid over immutable Enter snapshots; atomic stale refusal, exact submission and bounded diagnostics. Native Rust only. | Preserve host grammar/scheduling authority, rejected-result atomicity and completion precedence across platforms. | I | [I3/U2 dossier][validation-multiline]; [interaction][interaction] |
| history.storage_search | I4 history provider/search | 🟡 PARTIAL | Memory navigation exists; no storage-provider or search boundary. | Separate navigation from storage with bounded search, draft return and host retention/privacy policy. | I | [Core][core]; [interaction][interaction] |
| editing.keymap | I5 configurable editing | 🟡 PARTIAL | Normalized actions and a fixed compatibility keymap exist; no configurable modes, general undo or search. | Feed common edit operations from different mappings without changing decoder/storage authority. | I | [Keymap][keymap]; [F0][f0] |
| editing.word_operations | Unicode-aware word operations | 🔴 OPEN | Grapheme movement/deletion exists; there is no public deterministic word-boundary action contract. | Qualify word-wise movement/deletion over an explicit Unicode model, including revision and multiline edges. | I | [Editor owner][core]; [interaction][interaction] |
| editing.undo_redo | Undo and redo | 🔴 OPEN | Canonical edits advance DraftRevision, but no reversible edit state or public undo/redo actions exist. | Define bounded edit transactions and qualify text/cursor/revision, derived-analysis invalidation and history-navigation interaction. | I | [Editor owner][core]; [analysis contract](docs/interaction.md#revision-aware-host-analysis) |
| editing.kill_yank | Bounded kill and yank | 🔴 OPEN | Deletion and replacement primitives exist; no kill/yank register or lifecycle contract exists. | Qualify a bounded useful kill/yank contract without claiming GNU Readline kill-ring compatibility. | I | [Editor owner][core]; [keymap owner][keymap] |
| input.sensitive | I6 sensitive input | ⚪ LATER | No masked/hidden mode or secret-specific history posture exists in the current implementation. | Define and qualify echo/masking, history/snapshot/analysis exclusion, paste/output leakage, interruption and restoration without promising memory erasure. | I / Q | [Interaction owner][interaction]; [development method][development] |

### Completion and suggestion ergonomics

| ID | Property | Maturity | Current truth / exact boundary | Promotion condition | Program | Evidence / owner |
| --- | --- | --- | --- | --- | --- | --- |
| completion.helpers | Generic completion helpers | 🟡 PARTIAL | Revision-bound candidates and small host-owned examples exist; no reusable static/path/common-prefix/fuzzy helper surface exists. | Qualify optional bounded helpers while leaving grammar, semantic discovery and host-specific ranking with the host. | I / D | [Completion contract][completion-contract]; [completion example](examples/completion.rs) |
| completion.large_sets | Large candidate navigation | 🟡 PARTIAL | Bounded candidate menus and selection exist; no independently qualified paging/scrolling contract for large sets exists. | Qualify deterministic paging, scrolling, stable selection and narrow/plain visibility under bounded large candidate sets. | I / U | [I1/U1 dossier][completion-contract]; [presentation][presentation] |
| suggestion.autosuggest | Non-canonical autosuggestion | 🟡 PARTIAL | Revision-bound non-canonical hints can present host text, but no generic suggestion source, acceptance or history-suggestion mechanics exist. | Compose history/static/host suggestions with I1 insertion, stale refusal and explicit acceptance without canonicalizing visible hints. | I / D | [I2 dossier][analysis-presentation]; [interaction][interaction] |

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
| performance.large_draft | Large-draft scalability | 🟡 PARTIAL | Large multiline and 64 KiB/1 MiB workloads are measured; distant cursor/layout work still traverses prefixes. | Qualify scalable distant movement, line lookup, viewport-local geometry, resize, redraw and analysis presentation without prescribing a storage structure. | P / Q | [Hardening dossier][hardening]; [performance tools][perf-tools] |
| robustness.fuzz_property | Q0 general state-machine robustness | 🟢 ESTABLISHED | Five distinct Linux campaigns exceed 60 CPU-minutes each; 100,000 seeded paired semantic sequences and final-corpus replay pass on Linux x86_64/ARM64, macOS ARM64 and Windows portable core. | Retain minimized findings, immutable corpus identity and exact platform limits; new surfaces require new targets/evidence. | Q | [Hardening dossier][hardening]; [final corpus](tools/hardening/evidence/final-corpus.json.gz) |
| robustness.resource_stress | Q1 resource/failure stress | 🟢 ESTABLISHED | Each native release target passed 1,000 cycles per Rust tier and C ABI, 10,000 mixed events, eight failure classes ×100, 100 exhaustion children and native memory tools. | Preserve exact connected restoration, explicit impossible-restoration failure, caller ownership and platform-native resource evidence. | Q | [Hardening dossier][hardening]; [native evidence](tools/hardening/evidence/native-campaign.json.gz) |

### Developer and agent integration

| ID | Property | Maturity | Current truth / exact boundary | Promotion condition | Program | Evidence / owner |
| --- | --- | --- | --- | --- | --- | --- |
| dx.high_level_facade | Common-case high-level facade | 🟡 PARTIAL | Blocking `read_line` is small, while richer hosts assemble lower-level session/analysis configuration directly. | Qualify a common-case facade over the same editor/engine without hidden scheduling, parsing or a second runtime. | D / F | [Interaction][interaction]; [simple example](examples/simple.rs) |
| dx.generic_helpers | Reusable host helpers | 🟡 PARTIAL | Executed examples demonstrate host logic, but reusable completion/suggestion/history helpers are not a public library surface. | Provide bounded opt-in helpers that preserve host semantics, storage and privacy ownership. | D / I | [Use cases](docs/use-cases.md); [packaging dossier][packaging] |
| dx.runtime_adapters | Optional reactor adapters | 🟡 PARTIAL | Driven readiness/deadline primitives are established and runtime-neutral; no optional adapter for a common reactor ecosystem exists. | Qualify selected opt-in adapters without a mandatory Tokio, mio or other scheduler dependency in core. | D / F | [Embedding qualification][embedding]; [driven example](examples/driven.rs) |
| dx.reference_integrations | Production-shaped references | 🟡 PARTIAL | Simple, showcase, C and packaged debugger fixtures prove several surfaces; the expanded long-lived/platform/security product patterns remain uncovered. | Execute a bounded reference set spanning simple CLI, debugger/admin, model/network and native C/C++ use through public packages. | D / E | [Packaging dossier][packaging]; [use cases](docs/use-cases.md) |
| dx.documentation | Adoption documentation | 🟡 PARTIAL | README, contracts, rustdoc, C docs and an executed cookbook exist for the current surface; expanded v0.1 capabilities have no final user/agent documentation yet. | Complete task-oriented human documentation for every selected surface, platform, limit, package and migration path without duplicating authorities. | D / E | [Documentation map](docs/README.md); [use cases](docs/use-cases.md) |
| agent.skill | Official integration skill | 🔴 OPEN | No official versioned REPLAI integration skill exists. | Publish and qualify a skill whose guidance is fully available in public documentation and covers every selected integration boundary. | A / D | [Documentation owner](docs/README.md); [development method][development] |
| agent.machine_contract | Machine-readable integration contract | 🟡 PARTIAL | ABI schema and producer metadata are machine-readable at bounded scopes, but no public consumer-oriented integration contract covers platforms, tiers, examples and unsupported combinations. | Derive a versioned public contract from canonical docs/metadata without requiring private BOUNDARY infrastructure or creating a second roadmap. | A / E | [C ABI schema](api/c-abi.json); [producer metadata][producer] |
| agent.recipes | Agent integration recipes | 🟡 PARTIAL | Nineteen human-executed recipes provide a base, but expanded editing/output/security/Windows tasks and agent-oriented validation are absent. | Execute public-API recipes for the selected agent tasks and keep each equally usable by human integrators. | A / D | [Use cases](docs/use-cases.md); [packaging dossier][packaging] |
| agent.conformance | Deterministic integration conformance | 🔴 OPEN | External consumers are qualified, but no task suite checks generated integrations against ownership and unsupported-behavior rules. | Compile and test deterministic reference tasks using public materials only, without scoring or depending on a particular model. | A / Q | [Development method][development]; [packaging dossier][packaging] |
| agent.migration | Version-aware migration guidance | 🟡 PARTIAL | Changelog and exact producer deltas preserve chronology, but no ratified public migration contract exists before the first freeze. | Make compatibility boundaries and migrations discoverable to humans, agents and tooling, tied to released identities. | A / E | [Changelog](CHANGELOG.md); [producer metadata][producer] |

### Packaging, ecosystem and release

| ID | Property | Maturity | Current truth / exact boundary | Promotion condition | Program | Evidence / owner |
| --- | --- | --- | --- | --- | --- | --- |
| packaging.integration | E0 packaging | 🟢 ESTABLISHED | Unpublished `replai 0.1.0` crate and deterministic C source SDK build outside the checkout; MSRV/stable, docs, relocatable pkg-config/CMake and C11/C++17 static/shared consumers pass on the selected envelope. | Preserve standalone package/SDK identity, relocation and exact native/portable evidence; publication remains separate. | E | [Packaging dossier][packaging]; [C SDK][c-sdk] |
| ecosystem.consumers | E1 consumer diversity | 🟢 ESTABLISHED | Independent packaged debugger host exercises I0/I1/I2/I3, finite serialized notices and structured output on real Linux/macOS PTYs; portable state runs on Windows. | Requalify genuinely distinct external hosts when their consumed surface or package contract changes. | E | [Packaging dossier][packaging]; [debugger fixture](tools/package/debugger-consumer/src/main.rs) |
| ecosystem.cookbook | E2 integration patterns | 🟢 ESTABLISHED | Nineteen executed or explicit-deferred recipes cover three Rust tiers, analysis, history ownership, output limits, Rust/C/CMake and plain mode. | Keep recipes tied to executable examples/artifacts and preserve explicit host ownership and unsupported postures. | E / F | [Use cases](docs/use-cases.md); [Packaging dossier][packaging] |
| ecosystem.contract_handoff | Producer contract publication | 🟢 ESTABLISHED | Repository-owned snapshots/fingerprints and consumer-neutral delta; no runtime dependency or assigned migrations. | Keep metadata aligned with exact qualified source; consumers author their own profiles/receipts. | E | [Producer metadata][producer]; [exact publication][carrier] |
| release.api_freeze | E3 API freeze candidate | ⚪ LATER | Pre-release shapes; first-release audit is mandatory but cannot begin until the expanded product and hardening close. ABI identity is not yet a long-term compatibility promise. | Audit all selected public ownership, lifecycle, errors, language and portability contracts after RELEASE.HARDENING.1. | E / V | [Architecture owner][architecture]; [C contract][c-api] |
| release.qualification | V0 release qualification | ⚪ LATER | Expanded candidate support envelope selected; no integrated frozen-source/package release qualification yet. | Qualify one exact final source and regenerated artifact set across the complete expanded envelope. | V / Q | [Development method][development]; [CI][ci] |
| release.public | V1 first public commitment | ⚪ LATER | No package publication or effective stable compatibility promise; V1 requires separate authorization after V0. | Explicit release decision after V0; publish only the qualified support/compatibility scope. | V | [Release progression](#release-progression); [development][development] |
<!-- maturity:end -->

## Strategic Programs

Programs own durable architectural gaps rather than serial work slots. Program
maturity is not counted again in the row summary. The new D and A programs
organize adoption and agent integration around the same engine and public
contracts; neither creates another runtime, parser, scheduler or project-state
authority.

<!-- programs:start -->
| Program | Target property | Maturity | Completed foundations | Remaining v0.1 gaps | Dependencies | Explicit exclusions |
| --- | --- | --- | --- | --- | --- | --- |
| F | One engine, simple/session/driven embedding | 🟢 ESTABLISHED | F0 engine; F1 embedding; F2 capability/admission | Preserve one engine while editing, output, Windows, facade and adapters expand | P3 delivery; X resources; Q evidence | Host scheduler, parser, second editor/runtime |
| P | Measurable interaction under host driving | 🟡 PARTIAL | P0 baseline; P1/P2 convergence; P3 driving; Q2 policy | Large-draft scaling and new-surface regression workloads | F contracts; Q methodology | Universal ranking, unmeasured redesign |
| I | Daily-driver command interaction from host semantics | 🟡 PARTIAL | Revisions, candidates, validation, spans/hints, history navigation, normalized actions | I4 search/provider; word operations; undo/redo; kill/yank; I5; I6; helpers/suggestions | F delivery/revisions; U presentation; Q bounds | Parser, command language, history database |
| O | Long-lived coordinated output | 🟡 PARTIAL | O0 documents and finite serialized output with restoration | O1 sustained flow/backpressure; O2 producer arbitration; O3 transient lifetime | P3/F2; U geometry; Q stress | Token meaning, uncontrolled terminal writers |
| U | Coherent accessible line-oriented presentation | 🟡 PARTIAL | Prompts, candidates, multiline diagnostics, hints, documents and themes | U3 hierarchy, paging, transient integration, plain/accessibility and cross-platform consistency | I/O semantics; F2 degradation; X parity | Alternate-screen dashboard or widget framework |
| X | Native system realizations below one engine | 🟡 PARTIAL | X0 separation; Linux/macOS runtime; Windows portable core | X2 native Windows Rust runtime and resource/capability parity | F1/F2 resource contract; Q native evidence | Support inferred from compilation; automatic Windows C parity |
| Q | Reproducible correctness/resource/performance promotion | 🟡 PARTIAL | Q0/Q1/Q2 established for the current surface | Extend/replay evidence for every new surface and final artifacts in HARDENING.1 | Accompanies each changed boundary; P0/Q2 baselines | Test-count maturity; inherited evidence claims |
| E | Reproducible independent packaging and compatibility | 🟡 PARTIAL | E0 crate/C SDK; E1 consumers; E2 cookbook; producer metadata | Replay final expanded artifacts and complete E3 freeze audit | Final product/docs; Q/X evidence | Mandatory BOUNDARY, automatic consumer migrations |
| D | Human developer adoption over the same engine | 🟡 PARTIAL | Small blocking API, driven primitives, public examples, package tooling and cookbook | D0 facade; D1 helpers; D2 adapters; D3 references; D4 complete adoption docs | F/I/O/X public contracts; E packages; Q evidence | Hidden scheduler/parser, second engine, every runtime ecosystem |
| A | Agentic integration through public versioned material | 🟡 PARTIAL | Machine-readable ABI/producer fragments, public docs and executed recipes | A0 skill; A1 public machine contract; A2 recipes; A3 conformance; A4 migration guidance | D docs/helpers; E identities; Q conformance | Agent runtime, model benchmark, private knowledge requirement |
| V | First bounded public compatibility commitment | ⚪ LATER | Qualified current implementation, hardening and packaging foundations | Expanded HARDENING.1; E3/V0 frozen candidate; separately authorized V1 | All MUST_V0_1 rows; D/A docs; E/Q/X replay | Date-driven release, claims beyond exact evidence |
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

The earlier “minimum qualified kernel” v0.1 plan is superseded. The active first
release is a coherent daily-driver product for long-lived applications, selected
native platforms, human developers and coding agents. This classification changes
the release gate, not current implementation truth or earlier evidence.
[Release scope](docs/release-scope.md) owns the detailed envelope and acceptance
rules.

Every non-established maturity row is classified below. Existing ESTABLISHED rows
remain part of the release foundation at their exact qualified scope.

<!-- release-counts:start -->
MUST_V0_1=28 SHOULD_V0_1=0 LATER=0 OUT_OF_SCOPE=0 TOTAL=28
<!-- release-counts:end -->

<!-- release-scope:start -->
| Capability | Current maturity | v0.1 class | Rationale | Required evidence |
| --- | --- | --- | --- | --- |
| history.storage_search | 🟡 PARTIAL | MUST_V0_1 | Daily-driver history needs bounded provider/search mechanics while persistence, retention and privacy remain host-owned. | [Interaction scope](docs/release-scope.md#interaction-and-daily-driver-editing); provider, reverse-search and restoration qualification |
| editing.keymap | 🟡 PARTIAL | MUST_V0_1 | Common actions need configurable mappings without moving decoder or editor ownership. | [Interaction scope](docs/release-scope.md#interaction-and-daily-driver-editing); action/mapping and portable usability evidence |
| editing.word_operations | 🔴 OPEN | MUST_V0_1 | Word-wise movement/deletion is required daily-driver ergonomics and needs an explicit Unicode model. | [Interaction scope](docs/release-scope.md#interaction-and-daily-driver-editing); Unicode/property and revision evidence |
| editing.undo_redo | 🔴 OPEN | MUST_V0_1 | Reversible editing owns distinct text/cursor/revision and derived-state invariants. | [Interaction scope](docs/release-scope.md#interaction-and-daily-driver-editing); bounded state-machine qualification |
| editing.kill_yank | 🔴 OPEN | MUST_V0_1 | A bounded basic kill/yank facility completes the adopted editing fundamentals without promising Readline ring parity. | [Interaction scope](docs/release-scope.md#interaction-and-daily-driver-editing); lifecycle, bounds and keymap composition |
| input.sensitive | ⚪ LATER | MUST_V0_1 | The first product needs a defensible leakage-aware input path; masking alone is insufficient. | [Sensitive-input scope](docs/release-scope.md#sensitive-input); negative leakage, lifecycle and restoration campaign |
| presentation.visual_system | 🟡 PARTIAL | MUST_V0_1 | Current primitives need one coherent keyboard/plain/narrow/wide/accessibility system across the expanded surface. | [Visual scope](docs/release-scope.md#visual-and-large-draft-quality); cross-platform U3 qualification |
| output.streaming | 🔴 OPEN | MUST_V0_1 | Long-lived tools need sustained active-edit output with explicit backpressure and bounds. | [Output scope](docs/release-scope.md#long-lived-output); chunk, backpressure, latency and restoration stress |
| output.multiplexed | 🔴 OPEN | MUST_V0_1 | Legitimate producers need bounded arbitration while Interaction mutation remains serialized. | [Output scope](docs/release-scope.md#long-lived-output); producer ordering/fairness/resource evidence |
| output.transient | 🔴 OPEN | MUST_V0_1 | Progress and replaceable/expiring notices need a bounded lifecycle that composes with editing. | [Output scope](docs/release-scope.md#long-lived-output); replacement/removal/redraw qualification |
| completion.helpers | 🟡 PARTIAL | MUST_V0_1 | Common static, path, prefix and fuzzy integrations should not require repeated host boilerplate. | [Completion scope](docs/release-scope.md#completion-and-suggestion-ergonomics); optional helper API and bounds |
| completion.large_sets | 🟡 PARTIAL | MUST_V0_1 | Large candidate sets need usable deterministic paging/scrolling without losing selection or editor context. | [Completion scope](docs/release-scope.md#completion-and-suggestion-ergonomics); large-set narrow/plain/resize qualification |
| suggestion.autosuggest | 🟡 PARTIAL | MUST_V0_1 | History/static/host suggestions need generic non-canonical presentation and explicit acceptance. | [Completion scope](docs/release-scope.md#completion-and-suggestion-ergonomics); stale/plain/acceptance qualification |
| platform.windows_runtime | 🔴 OPEN | MUST_V0_1 | Native Windows Rust terminal use is part of the selected major-desktop product envelope. | [Windows scope](docs/release-scope.md#windows-runtime); real Console/ConPTY resource and PTY-equivalent evidence |
| performance.large_draft | 🟡 PARTIAL | MUST_V0_1 | Measured prefix traversal must be resolved enough for practical large multiline editing. | [Large-draft scope](docs/release-scope.md#visual-and-large-draft-quality); scaling thresholds and workload replay |
| dx.high_level_facade | 🟡 PARTIAL | MUST_V0_1 | Common applications need a smaller rich-integration path over the same engine. | [Developer experience](docs/release-scope.md#developer-experience); public facade consumers and ownership audit |
| dx.generic_helpers | 🟡 PARTIAL | MUST_V0_1 | Reusable helpers must convert proven example logic into bounded opt-in library support. | [Developer experience](docs/release-scope.md#developer-experience); package consumers and no-semantic-ownership tests |
| dx.runtime_adapters | 🟡 PARTIAL | MUST_V0_1 | Runtime-neutral driven core needs selected optional reactor adapters for practical adoption. | [Developer experience](docs/release-scope.md#developer-experience); real host-reactor consumers without core runtime dependency |
| dx.reference_integrations | 🟡 PARTIAL | MUST_V0_1 | Product-shaped reference hosts must cover the expanded use cases through public packages. | [Developer experience](docs/release-scope.md#developer-experience); executed independent reference matrix |
| dx.documentation | 🟡 PARTIAL | MUST_V0_1 | Humans must integrate every selected surface without source archaeology. | [Documentation closure](docs/release-scope.md#documentation-closure); task-oriented docs/API/link validation |
| agent.skill | 🔴 OPEN | MUST_V0_1 | An official public integration skill is a defining first-release adoption surface. | [Agentic integration](docs/release-scope.md#agentic-integration); versioned skill and public-doc parity audit |
| agent.machine_contract | 🟡 PARTIAL | MUST_V0_1 | Agents/tools need a public versioned map of capabilities, platforms, tiers and unsupported combinations. | [Agentic integration](docs/release-scope.md#agentic-integration); schema/derivation consistency checks |
| agent.recipes | 🟡 PARTIAL | MUST_V0_1 | Expanded integration tasks need executable instructions usable by agents and humans. | [Agentic integration](docs/release-scope.md#agentic-integration); packaged recipe execution |
| agent.conformance | 🔴 OPEN | MUST_V0_1 | Deterministic tasks must prove public material is sufficient without scoring a specific model. | [Agentic integration](docs/release-scope.md#agentic-integration); compile/test ownership conformance |
| agent.migration | 🟡 PARTIAL | MUST_V0_1 | Compatibility and upgrade boundaries must be discoverable before the first commitment. | [Agentic integration](docs/release-scope.md#agentic-integration); version-aware human/tool guidance |
| release.api_freeze | ⚪ LATER | MUST_V0_1 | E3 audits the complete expanded public surface only after final hardening. | [Final qualification](docs/release-scope.md#final-qualification-and-release); API/ABI/MSRV/migration review |
| release.qualification | ⚪ LATER | MUST_V0_1 | V0 must qualify one exact expanded source and regenerated artifact set. | [Final qualification](docs/release-scope.md#final-qualification-and-release); frozen-source native/package receipt |
| release.public | ⚪ LATER | MUST_V0_1 | V1 is the separately authorized publication of the qualified candidate. | [Final qualification](docs/release-scope.md#final-qualification-and-release); registry/tag/download/docs verification |
<!-- release-scope:end -->

No active property retains the superseded post-v0.1 classification. SHOULD_V0_1
remains available for independently omissible refinement, but none of the
current incomplete rows meets that test: each now belongs to the explicit
product definition.

The release remains bounded by the following horizon and ownership decisions.
These entries are scope decisions, not maturity rows or hidden implementation
work.

<!-- scope-boundaries:start -->
| Topic | Class | Boundary |
| --- | --- | --- |
| Full Vi modal compatibility | LATER | A distinct modal editing contract; configurable actions do not imply it. |
| Helix/Kakoune-style modes | LATER | Selection-first/modal systems require independent design and evidence. |
| Advanced mouse interaction | LATER | Keyboard-complete line editing is the v0.1 requirement. |
| System clipboard integration | LATER | Platform clipboard ownership is separate from bounded kill/yank. |
| Rich C parity / ABI 2 | LATER | C ABI 1 remains POSIX and bounded unless separately redesigned. |
| Exotic terminal protocol enhancements | LATER | Only mechanisms required by the selected native runtimes are release gates. |
| Command language, parser and shell grammar | OUT_OF_SCOPE | The host owns language and semantic interpretation. |
| Application router, scheduler and persistence | OUT_OF_SCOPE | The host owns execution, scheduling and durable policy. |
| History database | OUT_OF_SCOPE | REPLAI owns bounded navigation/search mechanics, never durable history authority. |
| Model runtime and agent runtime | OUT_OF_SCOPE | Agentic integration is documentation/tooling for consumers, not agent execution. |
| Full-screen TUI and product rendering ontology | OUT_OF_SCOPE | REPLAI remains a line-oriented interaction library. |
| Plugin framework | OUT_OF_SCOPE | Extension systems belong to the embedding application. |
<!-- scope-boundaries:end -->

E0/E1/E2 and Q0/Q1/Q2 remain ESTABLISHED at the source and environments recorded
by their dossiers. They are foundations, not evidence for future surfaces.
RELEASE.HARDENING.1 must extend and replay their relevant campaigns before E3/V0.

## Current Execution Sequence

The previous RELEASE.CANDIDATE.0 selection is removed because it targeted the
superseded smaller release. The sole selected boundary is
INTERACTION.ERGONOMICS.0, selected but not started. The plan contains twelve
engineering/design/qualification boundaries before one separately authorized
publication boundary:

| Order | Boundary | Bounded closure | Dependency / exit |
| ---: | --- | --- | --- |
| 1 | INTERACTION.ERGONOMICS.0 | I4 history/search; word operations; undo/redo; bounded kill/yank; common action vocabulary | Current editor/revision/history foundation; no feature work starts through this roadmap |
| 2 | COMPLETION.KEYMAP.SUGGESTION.0 | I5 keymaps; completion helpers; autosuggestion; large candidate paging/navigation | Stable action/revision semantics from 1 |
| 3 | OUTPUT.LONG_LIVED.0 | O1 sustained output; O2 bounded producer arbitration; coupled O3 transient foundation | Existing serialized output, driven delivery and resource ownership |
| 4 | WINDOWS.RUNTIME.0 | X2 native Windows Rust terminal realization and capability/resource parity | Stable interaction/output contracts; real Windows execution |
| 5 | SENSITIVE.INPUT.0 | I6 leakage-aware sensitive interaction | Editing/history/output/platform contracts established |
| 6 | PRESENTATION.UX.0 | Remaining O3/U3 hierarchy, menus, diagnostics, hints/status, themes and accessibility | All visible interaction surfaces and selected native platforms |
| 7 | LARGE.DRAFT.PERFORMANCE.0 | performance.large_draft scaling redesign and measured qualification | Stable final editing/presentation behaviors |
| 8 | DEVELOPER.EXPERIENCE.0 | D0 facade; D1 helpers; D2 selected adapters; D3 references; D4 adoption docs | Runtime/product surfaces stable enough for ergonomic wrappers |
| 9 | AGENTIC.INTEGRATION.0 | A0 skill; A1 machine contract; A2 recipes; A3 conformance; A4 migration guidance | Public facade/helpers/docs and compatibility vocabulary |
| 10 | DOCUMENTATION.CLOSURE.0 | Complete human/machine integration docs and reconcile examples/reference applications | Feature, DX and agent surfaces complete |
| 11 | RELEASE.HARDENING.1 | Expanded fuzz/state, resource/platform/security/output/large-draft/Q2/package-consumer qualification | All selected product surfaces complete |
| 12 | RELEASE.CANDIDATE.0 | E3 public API/C ABI review, compatibility ratification, artifact replay and V0 exact freeze | HARDENING.1 green; no publication |
| 13 | RELEASE.PUBLICATION.0 | V1 crates.io/tag/SDK/checksums/hosted-doc and downloaded-artifact verification | Separate explicit authorization after V0 |

Adjacent boundaries may later combine only when architecture and qualification
naturally close together and reviewability remains intact. Independently risky
problems may split. Such a control change requires an explicit roadmap update;
it is never inferred from implementation convenience.

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

The expanded [first-release scope](#first-release-scope) defines what must exist;
[release acceptance](docs/release-scope.md#release-sequence-and-gates) defines
when the final candidate can freeze. Current implementation and package evidence
is neither erased nor generalized.

| Phase | Required result |
| --- | --- |
| Product completion | Orders 1–7 establish daily-driver editing, long-lived output, Windows runtime, sensitive input, U3 and large-draft behavior. |
| Adoption completion | Orders 8–10 establish human DX, agentic integration and complete public documentation. |
| Expanded hardening | Order 11 extends Q0/Q1/Q2 and package-consumer evidence to every new surface and selected native target. |
| E3 / V0 | Order 12 audits the whole Rust/C public contract and qualifies one exact regenerated crate/SDK candidate. |
| V1 | Order 13 requires separate publication authorization and verifies registry, tag, downloads, checksums and hosted docs. |

No date is selected. The active target adds native Windows Rust runtime but does
not promise Windows C ABI 1, Intel macOS, musl or universal binaries. Packaging
machinery is established and will be replayed rather than reinvented. A new
surface receives new evidence; an older green dossier is never relabeled.

## Explicit Nonclaims

REPLAI is not a command language, parser, shell framework, full-screen TUI,
application router, scheduler, persistence system, history database, model/agent
runtime, product rendering ontology or plugin framework. Hosts own those systems.

At the current source there is no native Windows terminal runtime, sustained
output/backpressure service, producer arbitration, transient lifecycle,
sensitive-input mode, configurable public keymap, word editing, undo/redo,
generic completion helper layer, official integration skill or frozen Rust API.
Those are adopted targets, not present-tense capability claims. Windows C ABI 1,
full Vi/Helix/Kakoune modes, advanced mouse/clipboard support and rich C parity
are not implied by the expanded plan.

Performance comparisons establish exact recorded workloads, not general
superiority. Existing Q0/Q1/Q2, package, C SDK and consumer evidence applies only
to its recorded source and surface.

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
