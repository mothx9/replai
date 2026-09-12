# First public release scope

This is the active REPLAI v0.1 product contract. It is a plan for the first public
release, not a release announcement and not a claim that unimplemented targets
already exist. [ROADMAP](../ROADMAP.md#first-release-scope) remains the sole
authority for maturity, programs, selection and dependency order. This document
owns the release envelope and acceptance conditions. Architecture and feature
contracts own implementation truth; engineering dossiers own executed evidence;
Git owns chronology.

## Scope supersession

The decision recorded on 2026-09-12 at master
243f1604c39bee349476f01d738533149d761f20, tree
fc978952fbd082349ce66c175f40fe442b274852 supersedes the earlier “minimum
qualified kernel” definition. That earlier definition treated fixed editing,
finite serialized output, Linux/macOS runtime and Windows portable core as
sufficient for v0.1, with history search, configurable editing, sensitive input,
long-lived output, transient presentation and Windows runtime deferred.

The active decision instead requires a coherent daily-driver product suitable
for long-lived applications, selected major desktop platforms, human developers
and coding agents. This changes release sufficiency. It does not retroactively
change what current source implements or invalidate evidence at its recorded
scope.

The packaging implementation source remains
14cacba16f3cf2a00448d720502e1cfe975f2e83, tree
1a49972a3fb9a623cf6bc3201281192f004ce102. The current runtime source tree remains
12b9cd0e58ef8d2dfe6c1b91185fd21b05a7aa9e. This scope refoundation changes no
runtime, Rust API, C ABI, package dependency or producer capability.

## Current foundation and supersession

The following foundations remain established:

- one platform-neutral interaction engine with blocking, session and driven Rust
  integration;
- Linux GNU x86_64/ARM64 and macOS ARM64 native terminal runtime;
- Windows portable editor, analysis and presentation models;
- Unicode/grapheme editing, bounded history navigation, completion, revision-safe
  analysis, validation/multiline, diagnostics and host spans/hints;
- structured documents, themes, terminal capabilities and safe finite coordinated
  output;
- Q0 fuzz/property, Q1 resource/failure and Q2 regression-policy evidence for the
  surfaces recorded by RELEASE.HARDENING.0;
- E0 Rust/C packaging machinery, E1 independent packaged consumers and E2 executed
  recipes recorded by RELEASE.PACKAGING.0;
- an unpublished replai 0.1.0 package candidate, Rust 1.98.1 MSRV machinery,
  docs.rs candidate configuration, deterministic C source SDK, relocatable
  pkg-config/CMake and bounded POSIX C ABI 1.

These results are not reopened. Final v0.1 must replay affected hardening and
artifact qualification against the expanded source. A prior green result cannot
qualify behavior that did not exist when it ran.

## Active v0.1 product envelope

REPLAI v0.1 is ready for candidate freeze only when it can demonstrate all of:

- a coherent line-oriented interaction model with daily-driver editing;
- provider-backed bounded history navigation and reverse search while durable
  history policy remains host-owned;
- configurable common editing actions, Unicode-aware word operations, undo/redo
  and bounded basic kill/yank;
- reusable completion/suggestion helpers, autosuggestion and usable large
  candidate navigation;
- sustained long-lived output, bounded backpressure/producer arbitration and
  transient presentation while editing remains active;
- native Rust terminal runtimes on Linux GNU x86_64/ARM64, macOS ARM64 and
  Windows x86_64;
- leakage-aware sensitive input with explicit limitations;
- coherent prompt, menu, diagnostic, hint, transient, theme, NO_COLOR,
  accessibility and narrow/wide behavior;
- acceptable measured large-draft navigation, geometry, resize and redraw;
- a high-level facade, selected optional reactor adapters, generic helpers and
  production-shaped reference consumers;
- complete task-oriented human documentation;
- an official public integration skill, machine-readable integration contract,
  executable agent recipes, deterministic integration conformance and migration
  guidance;
- independent Rust/C packaging, expanded hardening, final API/ABI audit and one
  exact qualified candidate.

The host continues to own parser, language, command semantics, execution,
application state, scheduling, semantic candidate generation, storage policy and
business meaning. REPLAI owns bounded interaction mechanics, revision provenance,
generic presentation and terminal/resource realization. The expanded scope does
not weaken that boundary.

## Current versus planned surface

| Area | Current established foundation | Required before v0.1 freeze |
| --- | --- | --- |
| Editing | Grapheme editing, multiline movement, bounded history navigation, fixed mappings | Search/provider, words, undo/redo, kill/yank and configurable actions |
| Completion | Revision-bound rich candidates and host hints | Generic helpers, autosuggestion and large-menu paging |
| Output | Finite host-serialized output with draft restoration | Sustained bounded flow, producer arbitration and transient lifetime |
| Platforms | Linux/macOS runtime; Windows portable core | Native Windows x86_64 Rust terminal runtime |
| Sensitive input | No secret-entry contract | Leakage-aware masked/hidden interaction and negative qualification |
| Presentation | Prompts, themes, completion, diagnostics, hints and documents | Coherent U3/accessibility and expanded-surface consistency |
| Performance | Recorded Q2 workloads and known prefix-layout debt | Qualified large-draft scalability and updated regression gates |
| Developer adoption | Three Rust tiers, C ABI 1, examples, packages and cookbook | Facade, helpers, selected adapters and complete reference/docs set |
| Agent adoption | Public docs, ABI schema, producer fragments and recipes | Official skill, public machine contract, recipes, conformance and migration |
| Release | Hardening/packaging foundations | Expanded hardening, E3/V0 freeze and separately authorized V1 |

## Interaction and daily-driver editing

History search must separate mechanics from ownership. REPLAI may request or
consume bounded entries through a provider boundary and provide reverse
incremental search, navigation and exact restoration of the pre-search draft.
The host owns persistence, admission, retention, privacy, encryption and durable
identity. No HistoryStore or database becomes a REPLAI owner.

Word-wise left/right and deletion require an explicit deterministic Unicode
model. The final contract must define punctuation, whitespace, grapheme and
multiline edges rather than accidentally inheriting byte or ASCII behavior.

Undo/redo is a distinct bounded state machine. Its design must reconcile text,
cursor, DraftRevision, invalidation of completion/validation/presentation,
history navigation and rejected/no-op edits. It cannot be hidden inside a
keybinding-only row.

Basic kill/yank is adopted for v0.1 as a bounded ergonomic facility. It does not
promise full GNU Readline kill-ring behavior. The implementation boundary will
decide a small explicit register/lifecycle contract and qualify its relation to
undo, revisions, history and sensitive input.

Configurable keymaps must drive a stable common action vocabulary without moving
terminal decoding or editor storage into user callbacks. Full Vi, Helix or
Kakoune modal compatibility is not required.

## Completion and suggestion ergonomics

The existing completion protocol remains authoritative: the host owns semantics,
ordering and application context; REPLAI owns bounds, selection, revision
binding and safe insertion.

v0.1 adds optional generic helpers for static/list candidates, filesystem paths,
common-prefix operations and bounded fuzzy matching. A filesystem helper owns
filesystem traversal only when explicitly selected. Helpers never acquire the
application grammar or command vocabulary.

Autosuggestion must accept bounded history/static/host-generated sources,
present text as non-canonical and use explicit qualified insertion. A visible
suggestion must never enter submitted bytes accidentally. Large candidate sets
need deterministic paging/scrolling, selection visibility and narrow-terminal
behavior.

## Long-lived output

Long-lived developer tools require more than repeated synchronous notices.
v0.1 must establish:

- sustained host output while editing remains active;
- explicit bounded buffering and backpressure;
- a safe producer/arbitration boundary;
- predictable interaction with input, completion, validation, analysis, resize
  and close;
- generic transient progress/status/notice replacement and removal;
- exact restoration and measured chunk workloads.

Host event multiplexing, serialized Interaction mutation, producer arbitration
and arbitrary direct stdout/stderr writers are different things. The target
supports legitimate host producers without blessing uncontrolled terminal
writes. REPLAI owns mechanics and lifetime; the host owns token/event meaning,
cancellation policy and application scheduling.

Until that boundary is implemented, current source supports only finite
host-serialized output calls while active or host-controlled streaming after the
editor is closed. That present limit is not the final v0.1 envelope.

## Windows runtime

The selected native Rust release targets are:

- x86_64-unknown-linux-gnu;
- aarch64-unknown-linux-gnu;
- aarch64-apple-darwin;
- x86_64-pc-windows-msvc with a real native terminal runtime.

Windows qualification must cover Console and ConPTY choices where applicable,
HANDLE ownership, input-event translation, Unicode, dimensions/resize, terminal
modes, readiness/driven integration, admission, errors and exact cleanup. Native
execution is mandatory; compilation and portable model tests are supplemental.

The current POSIX C ABI 1 release scope remains Linux/macOS. This plan does not
promise Windows C resource binding, Intel macOS, musl, universal binaries or
other targets. Any such expansion needs its own adopted design and evidence.

## Sensitive input

Sensitive input is not ordinary input rendered with an asterisk. The future
contract must explicitly address:

- echo, hiding and optional masking;
- exclusion from history;
- snapshot, completion, analysis and suggestion visibility;
- paste policy;
- external/stream/transient output and redraw leakage;
- interrupt, EOF, failure and restoration;
- process-memory and allocator limitations.

REPLAI must not claim reliable secret erasure from ordinary Rust String or OS
process memory without an independently supportable mechanism. The final docs
must state the defensible protection boundary and residual risks.

## Visual and large-draft quality

U3 requires one coherent line-oriented visual system across prompts,
continuations, completion annotations and large menus, diagnostics, hints,
transient status, structured output and sensitive input. Keyboard-only operation,
NO_COLOR, non-color semantic cues, narrow/wide terminals, accessibility and
cross-platform consistency are release requirements. This does not create a
full-screen widget framework.

Large-draft qualification is a distinct property because current measurements
show prefix traversal for distant multiline layout. The target covers distant
cursor movement, line lookup, viewport-local geometry, resize, redraw, analysis
presentation and 10/100/1000-line plus admitted 64 KiB/1 MiB workloads. The
roadmap selects scalability and evidence; it does not prescribe a rope, piece
table, gap buffer or tree.

## Developer experience

The D program owns adoption ergonomics over the same implementation:

- D0: a high-level facade for common rich integrations;
- D1: bounded generic completion, suggestion and history helpers;
- D2: selected optional reactor adapters while core remains runtime-neutral;
- D3: packaged, production-shaped CLI, debugger/admin, model/network and C/C++
  reference consumers;
- D4: task-oriented documentation sufficient without source archaeology.

A facade or adapter may reduce boilerplate. It must not create a second editor,
runtime engine, parser, hidden scheduler or mandatory Tokio/mio dependency.
Supporting every Rust event ecosystem is not a v0.1 promise.

## Agentic integration

The A program makes the public package integrable by coding agents without
turning REPLAI into an agent framework.

A0 provides an official versioned integration skill covering blocking/session/
driven choice, Rust/C surfaces, ownership, history/editing/keymaps, completion,
validation/multiline, spans/hints, sustained/transient output, sensitive input,
Windows/platform posture, errors/resources, installation and compatibility.
Any knowledge required by the skill must also exist in public documentation.

A1 provides machine-readable public integration information for capabilities,
surfaces, platforms, tiers, examples, compatibility and unsupported combinations.
It must derive from or reconcile mechanically with canonical public contracts.
Private BOUNDARY infrastructure cannot be required, and the file cannot become a
second roadmap or release authority.

A2 provides executable task recipes for blocking, driven reactors, completion,
validated multiline, delayed analysis, long-lived output, sensitive input,
keymaps and Windows. They must remain useful to humans.

A3 qualifies deterministic reference integration tasks using public APIs and
materials only. Accepted solutions compile, pass tests, preserve ownership and
avoid unsupported behavior. This is not a benchmark of a named model.

A4 makes released compatibility and migration boundaries discoverable by humans,
agents and tooling. It does not promise versions that do not exist.

## Documentation closure

README remains the front door rather than the complete manual. Before freeze,
the canonical documentation set must cover architecture; quick start; all Rust
tiers; Rust API and C ABI; history/search; editing/keymaps; completion/helpers;
validation/multiline; sustained/transient output; sensitive input; Windows;
themes/accessibility; performance; safety/resources; package installation;
cookbook/reference applications; agent integration/machine contract; and
compatibility/migration.

Executable examples and packages remain authoritative for recipes. Documentation
closure cannot advertise planned surfaces as implemented, duplicate ROADMAP
state or rely on private knowledge.

## Packaging and hardening evidence

E0, E1 and E2 remain ESTABLISHED. The crate metadata, MSRV machinery, docs.rs
configuration, deterministic C source SDK, pkg-config/CMake installation and
external consumer harnesses are reusable foundations. They do not need to be
reinvented. The final expanded candidate must regenerate and requalify exact
crate and SDK artifacts, fresh Rust resolution, moved-prefix C/C++ consumers,
docs and external reference consumers against final source.

Q0, Q1 and Q2 remain ESTABLISHED for the surfaces and environments in
[RELEASE.HARDENING.0](engineering/release-hardening.md). Future properties do
not inherit that qualification. RELEASE.HARDENING.1 must extend or replay as
applicable:

- decoder/editor/history/search/word/undo/kill/keymap/suggestion state models;
- sustained output, backpressure, producer arbitration and transient lifetime;
- native Windows lifecycle, resource, failure and restoration stress;
- sensitive-input negative leakage properties;
- U3/plain/accessibility and large-draft scaling;
- new facade/helper/adapter/reference/agent conformance surfaces;
- Q2 named workloads and preregistered thresholds;
- C ABI 1 preservation and final package-consumer evidence.

A fixed finding remains in the ledger. Missing native hardware or evidence blocks
the corresponding gate rather than being inferred from another platform.

## Compatibility policy

E3 will ratify the final public contract. Until then REPLAI is unpublished and
the Rust API remains pre-release.

The selected candidate policy is:

| Contract | Intended v0.1 commitment |
| --- | --- |
| Rust | Preserve documented source behavior through compatible 0.1.x releases; breaking evolution requires a later minor and migration guidance. |
| C | Preserve ABI 1 symbols, records, layouts, numeric values and documented behavior; never silently reinterpret ABI 1. |
| SemVer | 0.1.0 is bounded pre-1.0 compatibility, not a 1.0 promise. Patch releases do not hide breaking Rust changes. |
| MSRV | Rust 1.98.1 remains the selected floor and requires replay on final package resolution. |
| Platforms | Do not silently drop a claimed 0.1.x target; additions require qualification. |
| Distribution | crates.io Rust package plus checksummed versioned C source SDK and documented installed artifacts. |
| Deprecation | Prefer replacement guidance before a later minor removes an API; corrections need explicit notes and evidence. |

Package name/publisher authority and actual hosted docs remain publication gates.
No crate or tag is published by this plan.

## Bounded later and out-of-scope

LATER:

- full Vi modal compatibility;
- Helix/Kakoune-style editing;
- advanced mouse interaction;
- system clipboard integration;
- rich C parity or ABI 2;
- exotic terminal protocols unrelated to the selected platform contract.

OUT_OF_SCOPE:

- command language, parser and shell grammar;
- application command router, execution scheduler and durable persistence;
- history database;
- model runtime and agent runtime;
- full-screen TUI framework and product rendering ontology;
- plugin framework.

REPLAI can support applications that own these systems. It does not own them.

## Release sequence and gates

[ROADMAP](../ROADMAP.md#current-execution-sequence) owns the exact ordered
boundaries. The active plan has twelve engineering/design/qualification
boundaries before one separately authorized publication boundary:

1. INTERACTION.ERGONOMICS.0
2. COMPLETION.KEYMAP.SUGGESTION.0
3. OUTPUT.LONG_LIVED.0
4. WINDOWS.RUNTIME.0
5. SENSITIVE.INPUT.0
6. PRESENTATION.UX.0
7. LARGE.DRAFT.PERFORMANCE.0
8. DEVELOPER.EXPERIENCE.0
9. AGENTIC.INTEGRATION.0
10. DOCUMENTATION.CLOSURE.0
11. RELEASE.HARDENING.1
12. RELEASE.CANDIDATE.0
13. RELEASE.PUBLICATION.0

### Final qualification and release

The first boundary is selected but not started. Each feature boundary must
produce its own contract and qualification. Documentation/agent materials follow
stable public surfaces. HARDENING.1 qualifies the expanded surface. E3/V0 then
audits the Rust API and C ABI, ratifies compatibility, regenerates exact
artifacts and freezes one candidate. V1 remains a separate explicit publication
authorization.

Adjacent boundaries may combine only when architecture and evidence close
naturally and review remains bounded. Independent risk may require a split.
Neither adjustment begins work automatically.

The release remains blocked until all MUST_V0_1 rows are ESTABLISHED at their
stated scope, HARDENING.1 passes, final package/SDK consumers pass, E3 ratifies
the complete public surface and V0 records one exact source/artifact receipt.
Publication then verifies crates.io, tag, SDK/checksums, hosted documentation and
downloaded artifacts. No old dossier can be relabeled as proof for a changed
surface.
