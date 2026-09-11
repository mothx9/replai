# First public release scope

This is the adopted **v0.1 candidate contract**, not a release announcement or a
claim that its remaining gates have passed. [ROADMAP](../ROADMAP.md#first-release-scope)
owns release classification, maturity, selection and execution order. This
document owns the support envelope, decision rationale and acceptance gates;
[architecture](architecture.md), [interaction](interaction.md), [presentation](presentation.md)
and [C ABI](c-api.md) continue to own implemented behavior.

## Reconciled basis

Decision basis: master `8d758142d7649cf9a422ad8025c9ba62093fc47d`, tree
`8638889c27d8c56b22b1068cf58766e9a38d3fcd`.
That documentation checkpoint preserves the runtime at
`da16302c33cdce5ef40978e02e9d8dff045c93f9`, including I0/I1/I2/I3, U0/U1/U2,
F1/P3/F2 and fixed continuation indentation. Its
[12-job CI](https://github.com/mothx9/replai/actions/runs/34516163472) is existing
boundary qualification, not evidence for the release gates defined below.
No runtime, API, ABI, dependency or producer-capability changes accompany this decision.

The first release is useful now as an embeddable line-oriented interaction
library. No missing interaction feature is necessary to make that bounded
contract coherent. The remaining mandatory work is adversarial qualification,
package/install machinery, independent consumer proof and a reviewed compatibility
commitment. A discovered correctness defect blocks release; it does not justify
silently expanding the feature scope.

## Candidate support envelope

| Surface | v0.1 commitment upon release | Explicit exclusion |
| --- | --- | --- |
| Native Rust | Blocking, session and driven Interaction over one engine; standalone Editor | Host execution, parser, analysis scheduling or application cancellation |
| Analysis | Immutable revisioned snapshots; stale-safe replacement; rich candidates; optional validated submission; host style spans and non-canonical hints | Discovery/ranking, grammar, semantic context identity, parsing or analysis cache |
| Editing | Bounded Unicode/grapheme editing, in-memory history, fixed bindings, admitted paste and validated multiline navigation/indentation | General search/provider, configurable keymap, undo/redo, secret input mode |
| Presentation | Composed prompts/themes, completion/diagnostics/hints and responsive standalone/coordinated documents | Full-screen UI, transient surfaces, universal font/emulator agreement |
| Output | Synchronous, host-serialized plain/document transactions with active draft restoration; ordinary host output after close | Sustained streaming service, bounded asynchronous queue, independent writers or fairness guarantee |
| C | ABI 1 POSIX session interface, existing replacement/direct submission and plain coordinated output | Rust/C feature parity, new ABI 1 methods, driven handles, rich analysis/documents |
| Platforms | Linux GNU x86_64 and aarch64; macOS aarch64 runtime; Windows x86_64 portable Rust core/models | Windows terminal runtime/C acquisition; Intel macOS, musl and other targets as release-qualified promises |
| Distribution | crates.io Rust package plus versioned C SDK source bundle and documented staged static/shared installation | Required BOUNDARY checkout, downstream repository, daemon or prebuilt native binary matrix |

The target triples are `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
`aarch64-apple-darwin`, and portable-only `x86_64-pc-windows-msvc`.
These are **candidate release targets**, not a claim that every target's new
release campaign is already qualified. G1/G2/G5 require native execution at the
claimed scope. Cross-compilation cannot replace it. Intel macOS or musl may work;
this decision does not certify them or remove existing source portability.

V0 must record exact OS/kernel, libc or SDK/deployment target, architecture,
compiler and terminal profiles for each claimed target. The release supports
source builds at those recorded environments; it makes no blanket promise for
all historical Linux distributions or macOS versions. The source-only native
SDK avoids implying a universal binary/loader baseline. Expanding that envelope
requires new qualification and a separately reviewed scope change.

Interactive admission still requires matching TTYs, restorable modes, usable
geometry and admitted cursor/erase mechanics. Simple/driven conservative defaults
and legacy session/C VT assumptions remain distinguishable. NO_COLOR is policy;
standalone plain documents are valid on captured streams. Optional paste loss
must remain explicit: unframed multiline input is not an atomic paste contract.

## Scope classes and decision rule

The [roadmap classification table](../ROADMAP.md#first-release-scope) includes every
non-established maturity row. Classification does not promote maturity.

- `MUST_V0_1`: release is blocked until the bounded acceptance requirement passes.
- `SHOULD_V0_1`: desirable refinement; a recorded exclusion reason is permitted
  if mandatory safety/usability gates remain satisfied. It cannot hide a defect.
- `V0_2`: first post-v0.1 planning pool, not a promised v0.2 delivery or selection.
- `LATER`: outside both the first-release gate and the immediate post-release pool.
- `OUT_OF_SCOPE`: belongs to the host or a different kind of library.

The test is whether absence makes the advertised contract incomplete, misleading,
unsafe or impractical. Competitor feature inventories and roadmap numbering are
not dependency arguments. Existing established properties included above must
also survive release qualification; they are not exempt because their row is green.

### History

Bounded in-memory navigation and restoration of the unsent draft are sufficient.
The host retains its own records and can re-admit them with `admit_history`;
there is no public history-export/provider API. Reverse search and a provider abstraction are useful post-release
interaction work, not prerequisites for a usable command loop. No persistence
backend, file format or history database is part of v0.1.

The cookbook must show host admission, reload/export responsibilities and privacy
limits without implying a storage/search API. Persistent application history,
retention and encryption policy remain `OUT_OF_SCOPE` for REPLAI ownership.

### Editing policy and visual refinement

The current fixed bindings are sufficient for the selected line editor:
grapheme arrows/deletion, whole-draft Home/End, history, completion, and validated
multiline movement/continuation indentation. Their direct/validated differences
must be documented. In particular, existing Ctrl-A/Ctrl-E/Ctrl-C/Ctrl-D behavior
is not an Emacs mode claim.

| Addition | Release class | Reason |
| --- | --- | --- |
| Word movement/deletion | V0_2 | Useful efficiency, not required to reach/edit any admitted draft position |
| Configurable bindings | V0_2 | Fixed documented policy is a coherent v0.1 contract |
| Undo/redo | V0_2 | No undo promise is advertised; a separate state/history contract is needed |
| Broader Emacs-like bindings | V0_2 | Do not turn a few familiar control keys into compatibility-mode claims |
| Vi mode | LATER | Independent modal contract and qualification; no first-release dependency |
| Sensitive input | LATER | Masking, leakage and secret/history posture need explicit separate design |
| General U3 visual/accessibility refinement | SHOULD_V0_1 | Improve coherence if available; do not make an unbounded polish program a gate |

A **minimum existing-surface usability review is mandatory** in G2: keyboard-only
accept/dismiss, visible selection without color, readable continuation/diagnostics,
non-canonical hint distinction, and narrow/wide clipping. A failure that confuses
submitted bytes or hides required actions is a blocker, not waivable U3 polish.
Passing that bounded review does not close all U3 or claim screen-reader certification.
Applications needing secret collection must use a separate suitable mechanism;
v0.1 does not present the ordinary editor as a password entry facility.

### Output, model clients and long-lived hosts

O1 and O2 are **not required** by the selected first-release output contract.
This is an explicit product constraint, not a claim that they already exist.

| Host pattern | v0.1 support | Host obligation |
| --- | --- | --- |
| Turn-by-turn CLI/model chat | Supported | Read/close, execute or stream with host I/O, then reopen; REPLAI owns no closed-interval stream |
| Debugger/build notifications while typing | Supported | Reactor delivers complete bounded notices through serialized output calls |
| Input while network/model events arrive | Supported with synchronous transactions | Host buffers/coalesces data and owns backpressure/cancellation; each output call may block |
| Token-by-token rendering while editing with latency/fairness guarantees | Not a v0.1 promise; O1 is V0_2 | Do not market repeated transactions as a qualified sustained streaming API |
| Threads/processes writing independently to the same active terminal | Unsupported; O2 is V0_2 | Marshal into the single owner; no direct concurrent stdout/stderr writers |
| In-place progress/expiry/replacement | Not provided; O3 is V0_2 | Use persistent status output or host output while REPLAI is closed |

Application event multiplexing already exists through P3. It is not terminal
writer arbitration. No maximum producer rate, asynchronous cancellation of a
blocked write, throughput SLA or hard event-latency guarantee is promised.
G2/G4 must exercise finite repeated output mixed with input and delayed analysis;
that proves safe serialized use, not an endless stream service. If a proposed
release description requires sustained active-edit streaming, O1 becomes a new
scope decision rather than a hidden dependency or accidental promotion.

## Compatibility policy to be ratified before tagging

These are selected release requirements. Current pre-release contracts remain
unchanged until the freeze review and actual publication.

| Contract | Selected v0.1 policy |
| --- | --- |
| Rust | Preserve source compatibility and documented behavior across 0.1.x, including exhaustive enums, trait bounds, lifetimes and error semantics. Breaking changes require 0.2 or later, with migration notes. |
| C | Preserve ABI 1 records, layouts, symbol signatures, numeric values and documented behavior. Newer 0.1.x libraries must run qualified old ABI 1 clients without recompilation. Never reinterpret ABI 1, even in a later package minor release. A breaking C contract needs a separately designed ABI identity. |
| SemVer | 0.1.0 is the first bounded commitment, not 1.0. Patch versions fix compatible behavior; breaking Rust evolution cannot be hidden in 0.1.x. No stability promise for private internals or exact non-contractual renderer bytes. |
| MSRV | Select Rust **1.98.1** as the initial supported floor, matching the reconciled working compiler. Qualify it on all selected Rust targets and declare rust-version before release. Keep that floor throughout 0.1.x; raising it requires a new minor release. |
| Deprecation | Prefer a documented replacement and at least one 0.1.x deprecation notice before removal in a later minor. No removal in 0.1.x. Security/correctness exceptions require explicit advisory and migration evidence, not silent scope redefinition. |
| Platforms | Retain the named 0.1.x target/profile envelope; no silent dropping of a claimed target in a patch. Newly qualified targets may be additive. Unsupported terminals still refuse according to the documented admission policy. |
| Support | Reproducible bug reports and public release notes; no response-time, LTS duration or service SLA is implied. |

The selected MSRV is a conservative support floor, not a claim that older Rust
cannot compile the code. Current manifests declare no MSRV. G3 must prove this
floor with packaged sources and clean consumers on the exact toolchain; inability
to obtain/qualify it blocks the gate and requires an explicit scope revision,
not a silently raised requirement. Test both the release lock graph and a fresh
consumer resolution. Dependency MSRV changes must not silently break 0.1.x.

Cargo treats compatible 0.1.x dependencies differently from a blanket “anything
before 1.0 may break” posture; the selected policy follows the
[Cargo compatibility guidance](https://doc.rust-lang.org/cargo/reference/semver.html)
and [version requirements](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html).
The [rust-version field](https://doc.rust-lang.org/stable/cargo/reference/rust-version.html)
will declare the tested floor. This decision does not edit Cargo metadata now.

## Distribution and installation gates

### Rust package

crates.io publication of **replai 0.1.0 is mandatory**; Git-only distribution is
not the chosen public release. An exact Git pin remains useful for prerelease
or unreleased work. E0 must remove publish=false only in its authorized wave,
audit metadata/license/repository/readme/categories, and produce a verified
`.crate` with every required source file and no checkout-relative dependency.
The separate C producer crate need not be published to crates.io.

G3 requires `cargo package` verification, unpacked-package compilation/tests,
clean external Cargo consumers on MSRV/current stable, and a docs.rs-compatible
rustdoc build. Apart from ordinary dependency acquisition, no network access or
build-time tool/script outside declared Cargo inputs may be required by the
library. Resolve package-name/maintainer
publication authority before declaring publication ready; this scope review
neither reserves a registry name nor publishes a package.

Rustdoc must expose the Linux native API and make target-gated differences
visible. Use explicit docs.rs target metadata if needed; test it against
[docs.rs build/metadata rules](https://docs.rs/about/metadata). Windows docs must
not advertise unavailable terminal methods. G6 verifies the actual hosted docs,
not just a successful local rustdoc run.

### C SDK

Mandatory distribution is a versioned, checksummed **C SDK source bundle** from
the same release revision, with Cargo lock/source, public header, build/staging
tools, license, concise consumer examples and installation instructions. A
producer builds native static/shared artifacts; installed C/C++ consumers do not
need Rust or access to REPLAI sources. The SDK producer does need the declared
Rust toolchain and build prerequisites. State that distinction on the download page.

The installed contract must include:

```text
include/replai.h
lib/libreplai_c.a
lib/libreplai_c.so or libreplai_c.dylib
lib/pkgconfig/replai.pc
lib/cmake/replai/  (package configuration and imported targets)
share/licenses/replai/LICENSE
```

E0 adds minimal CMake **consumption**, not a second Rust build system:
`find_package(replai CONFIG REQUIRED)` and explicit `replai::static` /
`replai::shared` targets. Test C11 and C++ consumers through both pkg-config and
CMake against moved install prefixes, including transitive system link libraries,
loader lookup and no accidental global REPLAI resolution. Shared and static
linkage are separate oracles. No bundled C rich-analysis expansion is needed.

Prebuilt native SDK archives, OS package managers, universal/fat macOS binaries
and Windows C installation are deferred (`V0_2` distribution evaluation).
They cannot be implied by “static/shared available”: v0.1 guarantees documented
source production and installation of those artifacts, not a universal download.

## Consumer evidence and cookbook

Read-only remote observation on 2026-09-10 found:

- YAI master `a08ede230705e3d68e69fb44dfb7eb73c2860f2b`, tree
  `095d770ad65f4dd7f7e90ccc35cea3e35b78bcff`: its
  [Rust manifest](https://github.com/yailabs/yai/blob/a08ede230705e3d68e69fb44dfb7eb73c2860f2b/cmd/yai/Cargo.toml)
  pins REPLAI `6365f84e12865871bf26ecf0d984b48213d81ebc`.
- YVEX's default main `3f4a1c182d35e5a0e163adb81008ae7a366efcc6` is not evidence
  for the current C integration. The separately inspected published models2
  `991fa5a1fdd61a49667a297f7439787ae52b898a`, tree
  `8a64a564931024c4d312903e536c6339e5be8d72`, has an explicit
  [C ABI 1 dependency descriptor](https://github.com/yailabs/yvex/blob/991fa5a1fdd61a49667a297f7439787ae52b898a/config/replai.json)
  at the same `6365f84…` checkpoint and tree
  `2ff0954473fc642f571bf78424278d9019e29338`.

These are bounded dependency observations, not fresh consumer execution,
authoritative consumer manifests or qualifications of the newer analysis APIs.
Both products supply useful language/integration evidence, but they share a
model-client lineage and remain on an older surface. They are **not sufficient
alone for E1 at the selected release scope**. No consumer change or repin is required.

Existing [simple](../examples/simple.rs), [console](../examples/console.rs),
[driven](../examples/driven.rs), [query](../examples/query.rs) and
[C](../examples/c/demo.c) hosts already supply much of the mechanics. G4 must add
or adapt **one independently built debugger-style deterministic console** outside
the checkout during qualification, consuming packaged/staged public artifacts.
It owns a small command catalog and state machine, an independent timer/socket
notification, delayed analysis and finite batched notices while a draft remains
active. It must not depend on a model service, donor source or private REPLAI modules.
Changing the prompt of the existing model-like demo is not independent semantics.
A bounded fixture is sufficient; an external commercial adopter is not a gate.

The release consumer matrix requires: a tiny Rust blocking CLI; that distinct
session/driven host exercising I0/I1/I2/I3/output; standalone captured documents;
and installed ABI 1 C/C++ static/shared clients. Record exact artifact hashes and
host semantic assertions. YAI/YVEX need not adopt to let REPLAI ship independently.

E2 derives recipes from those executed fixtures: simple read/evaluate, validated
multiline, delayed analysis, reactor notifications, chat streaming **after close**,
host-coalesced output while editing, history admission/persistence ownership,
plain reports, and C installation. Existing [use cases](use-cases.md) are the owner;
do not rewrite the README again. Release-time documentation changes are limited
to versioned installation, compatibility/MSRV, API docs, support matrix, release
notes and migration from the final pre-release pin. A broader cookbook may remain
partial after this bounded release gate passes.

## Required evidence before freeze and tagging

The quantities below are **planned minimum campaign budgets and acceptance
rules**, not fabricated measurements. Retain tools/seeds/source/environment and
all failures. A passing old boundary suite does not establish these new gates.
A missing tool/platform or unqualified threshold is BLOCKED, not silently skipped.

### G1 — Q0 adversarial state qualification

Build five bounded fuzz/property targets: (1) VT/UTF-8/paste decoding, (2) editor
and history actions, (3) revision-bound completion/validation/I2 payloads and
lifecycle, (4) documents/prompt/width layout, and (5) legal C ABI call sequences
with valid pointer/buffer ownership, malformed lengths/states and record versions.
The C target must not call outside documented pointer preconditions and label
resulting host UB a library bug.

Each target needs at least **60 CPU-minutes of coverage-guided execution** on a
recorded Linux environment before freeze, with bounded input sizes, watchdogs
and memory instrumentation appropriate to the target. Replay the retained corpus
on Linux, macOS and portable Windows where the contract exists. Native C fuzzing
is not a Windows runtime claim. Add at least **100,000 seeded generated transition
sequences** (up to 128 operations each) against a deterministic semantic oracle.

Acceptance: zero unexplained panic/crash/hang, invalid indexing, terminal injection,
stale mutation/output, broken grapheme boundary or content loss. Every finding
must become a minimized regression and be rerun through its affected native
boundary. Archive corpus identity and execution/coverage reports; campaign time
alone is not a proof of robustness or a security certification.

### G2 — Q1 resource stress and baseline usability

On each claimed native target, run **1,000 acquire/edit/close cycles per Rust tier**
and ABI 1 lifecycle, then a mixed **10,000-event** resize/input/serialized-output
sequence. Exercise 20/40/80/132 columns, changing heights, Unicode, paste, history,
candidate/diagnostic/I2 combinations and stale delayed results. Include admitted
plain/no-paste profiles and negative TERM/non-TTY/missing-mechanic cases.

Cover resource acquisition, read, write and restoration failures with fault
injection and native disconnected/closed descriptors; at least **100 repeated
failure/cleanup cycles per failure class**. Descriptor exhaustion uses isolated
child limits, not changes to the test host. Sweep deterministic virtual failure
positions and distinguish those from real PTY evidence. Require exact restoration
when the OS permits it, explicit cleanup-error reporting otherwise, stable FD and
lease ownership, no growing retained terminal/payload state, and no test hang.
Linux Valgrind must show no invalid access or definite/indirect/possible loss;
macOS native leak checks must show no attributable leaks. Pre-existing runtime
allocations require separately evidenced attribution, not a blanket suppression.

Allocator exhaustion is not a promise of recoverable allocation everywhere:
ordinary Rust allocation failure/abort and SIGKILL are outside graceful cleanup.
Test configured capacity refusal and failed system resource acquisition, and
state these termination limits. The keyboard/plain/narrow usability review above
is part of G2. General U3 remains separately classified.

### G3 — E0 installability and toolchain

Produce the Rust package and C source SDK/install tree described above, with
checksums, license and source identity. Verify MSRV 1.98.1 and current stable,
locked producer and fresh external dependency resolution, package contents,
rustdoc/docs.rs configuration, moved prefixes, pkg-config, CMake and static/shared
C11/C++ use without repository/private-header access. Installation must be usable
from release instructions alone. No package publication occurs in this gate.

### G4 — E1/E2 independent integration

Execute the consumer matrix above against G3 artifacts on native Linux/macOS,
with platform-neutral portions on Windows. Assert host decisions as well as
terminal cells: accepted input, rejected/stale results, exact cursor/draft,
current menu/diagnostic/hint restoration, and no hint in submitted bytes. Supply
copyable recipes and a clear unsupported-streaming/secret-input posture. External
consumer adoption is useful corroboration, never a hidden release blocker.

### G5 — Q2 policy, E3 freeze and V0 release qualification

Q2 must establish the measured policy **before API freeze**. Freeze deterministic
allocation, encoded-byte and write-count expectations for named fixtures where
those are actual contracts; unexpected changes need a reviewed cause, not an
automatic baseline overwrite. Idle driven no-periodic-wake/no-I/O interest remains
an exact oracle. Do not turn incidental ANSI encodings into public ABI guarantees.

For latency, record at least **five control batches** and **31 measured repetitions
per workload**, with preparation outside timing, warmups, machine/power identity,
and separate allocation instrumentation. Register thresholds before comparing
candidate changes: median regression allowance is the greater of **15% of baseline,
5× control-batch MAD and 2× measured timer resolution**; p95 uses **25%**, the
corresponding control-p95 MAD and the same timer floor. Exceedance in two independent
alternating before/after runs blocks promotion pending investigation. These are
selected release engineering rules, not existing measured performance bounds.
If noise cannot support a meaningful threshold, that metric remains unqualified;
use a controlled runner rather than declare a noisy hosted pass. Hosted CI retains
integrity and deterministic resource gates, not fabricated latency precision.

Required workloads: isolated append/cursor, 1000-byte edit/submission, paste,
history, direct replacement, menu show/navigation/accept, validation, I2 install,
serialized output, resize, and 10/100/1000-line plus admitted 64 KiB/1 MiB cases.
Name prefix-traversal debt and host-output blocking explicitly. This freezes a
bounded regression policy, not a universal latency SLA or general performance win.

After G1–G4 and the Q2 policy pass, E3 audits the **entire existing public surface**:
Rust exports/exhaustiveness, lifecycle/errors, limits, text safety, cross-language
asymmetry, auto-traits/borrowing, and terminal/width assumptions. No new feature
is required. If a real compatibility defect requires code repair, it needs a
bounded authorized correction and affected gate replay before freeze.

V0 then qualifies one frozen candidate source/tree plus exact crate/SDK digests:
full current CI and native suites; G1 corpus/campaign and G2 stress on that source;
G3/G4 installed-artifact tests; Q2 paired measurements; ABI 1 symbol/layout and
old-client/new-library checks; README/docs/API/migration and release-note checks.
Use a pre-freeze ABI 1 client as the initial executable baseline, and retain the
0.1.0 client as the patch-series baseline after release. Document residual limits.
No source-changing fix may borrow the previous candidate's green receipt.

### G6 — V1 publication

A separate explicit release authorization is required after V0. Publish the
qualified crate, source SDK, versioned notes and checksums; create the release tag
on the exact qualified source, with no post-qualification rebuild silently
substituted. Verify installed/downloaded artifacts and hosted Rust docs resolve
the same release identity. Package names, publisher access and release destinations
must be confirmed before tagging; missing access keeps publication blocked.
No release tag, crates.io upload, C ABI bump or consumer migration is authorized
by FIRST.RELEASE.SCOPE.0.

## Remaining work and change control

[The roadmap sequence](../ROADMAP.md#current-execution-sequence) contains exactly
**two remaining engineering waves followed by one separately authorized publication wave**.
G1/G2/Q2 are complete at their bounded scope; G3/G4 establish installability and
independent adoption; E3/V0 audit and qualify the final candidate; G6 publishes it.

Required new work is packaging/install metadata, the independent fixture and
release recipes, followed by candidate freeze/qualification. There is **no selected
new runtime feature**. Existing candidate APIs still need audit and regression
qualification. SHOULD refinements cannot delay the mandatory sequence indefinitely;
record exclusions before freeze. A blocking defect is repaired/requalified within
the affected authorized boundary, or this scope/order is explicitly revised in
ROADMAP if a genuinely new boundary is necessary. The count is the adopted plan,
not a promise that undiscovered defects cannot change it.

ROADMAP keeps all deferred properties and original maturity. Passing a release
gate promotes no wider Q/E/U program automatically: a scoped installed-consumer
matrix is not every possible consumer, and five fuzz targets are not universal
proof. Each promotion still requires its row's exact evidence and an explicit
control update. No subsequent engineering wave starts through this document.
