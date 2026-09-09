# Terminal capability qualification

This dossier owns bounded F2 engineering evidence. [ROADMAP](../../ROADMAP.md)
owns maturity/promotion; [interaction](../interaction.md#terminal-capabilities)
and [presentation](../presentation.md#display-width-contract) own public semantics.

## Reconciled baseline and scope

Repository `mothx9/replai`, canonical `master`:

```text
HEAD 8d7dc585688d81a7348e025a058a36431ba610e8
TREE 2426ba3cfeabf9aeb1ad1bff693d4cbd2e4deb46
```

The checkout was clean and matched the fetched remote. YAI, YVEX and BOUNDARY
are outside the mutation scope. No consumer pin or application semantics change.
F2 begins from qualified F1/P3 and closes admission for the current native and
portable realizations, not active capability probing or Windows terminal support.

## Ownership and public API

Before, TerminalConfig had four protocol facts, two feature policies and a Theme.
Native session opens bypassed its resolution; native resources verified TTYs and
sizes separately. Theme retained a private styling condition; layout/document
width calls directly selected unicode-width behavior.

After, the same resolver admits all native opens before raw mode. It separates:

| Owner | Contract | Current realization |
| --- | --- | --- |
| Protocol evidence | TerminalFacts / FeatureSupport | Supported versus Assumed/Unknown/Unavailable; TERM only supplies hints |
| Resource evidence | TerminalRealization | Native paired TTYs, captured mode, size, query and waitable support |
| Required mechanics | InteractionRequirements | Presentation, Editing, Driven |
| Optional policy | TerminalConfig / FeaturePolicy | Styling/paste Disabled, Preferred or Required |
| Result | TerminalCapabilities / Degradation | Inspectable retained facts, features and loss reasons |
| Geometry | WidthPolicy::UnicodeNarrow | Shared deterministic non-CJK cell model; no font detection |

`Interaction::capabilities` copies the active snapshot without I/O. No engine,
decoder, renderer, private mutation or OS handle is added to the public model.
Existing TerminalConfig/TerminalFacts struct fields and Error variants remain
source-compatible. The native missing-dimensions error changes from unsuitable
resource to CapabilityMismatch, naming required geometry; C status remains the
same unsuitable integer category.

One resource acquisition owns observed truth. It obtains termios and dimensions,
resolves requirements, and only then enters raw mode and draws. Negative protocol
or dimensions admission releases duplicated descriptors/lease without writing.
The driver stores a resolved theme/features/snapshot. It does not rediscover
facts, read environment variables or resolve policy per input/frame. Refresh
updates only observed dimensions. Editor/engine/keymap behavior is unchanged.

The legacy VT assumption remains explicit through `TerminalConfig::compatibility`.
TERM=dumb suppresses its environment theme but cannot retroactively turn the
assumption into discovery. All profiles remain one editor and lifecycle.

## Profiles and evidence classes

[Portable oracle](../../tests/capabilities.rs) supplies explicit synthetic resource
facts. It tests the support/policy cross-product, required mechanics, stronger
resource negatives, backend-managed versus host-waitable readiness, fixed versus
queryable geometry, standalone plain output and width classes. Isolated child
processes test absent/dumb/normal TERM with NO_COLOR absent/present-empty; explicit
host protocol evidence replaces hints while preserving user policy.

[Native oracle](../../tests/capabilities_pty.rs) uses actual PTYs. Interactive
resource/size/mode properties are observed; protocol support remains explicitly
Assumed or host-configured Unavailable. The test does not claim emulator discovery.

| Profile | Facts / requirement / policy | Admission and observed output |
| --- | --- | --- |
| Full | Real TTY, size; assumed VT; Driven, prefer style/paste | Styled prompt and heading, paste mode enabled |
| No styling | Styling unavailable; prefer styling | Plain heading/text with same cells, no SGR |
| No paste | Paste unavailable; prefer paste | Ordinary editing, no paste enable/disable; unframed Enter submits first line |
| Plain/no paste | Both optional features unavailable | Same editing/output with both degradations recorded |
| Required paste absent | Real resource, absent paste; required | CapabilityMismatch, zero terminal writes, mode unchanged |
| Cursor/erase absent | Real resource, missing required protocol mechanics | CapabilityMismatch, zero writes, mode unchanged |
| Dimensions absent | Real zero-size PTY, affirmative protocol assumptions | CapabilityMismatch; host assumptions cannot override observed geometry |
| TERM=dumb/absent | Unknown hint, default blocking admission | Refusal before raw mode; legacy session profile separately retains Assumed VT |
| NO_COLOR | User policy, not color evidence | Facts remain unchanged; effective styling disabled |
| Captured output | No interactive input/output; Presentation | Deterministic plain document without cursor or size-query requirement |
| Driven | Waitable, queryable dimensions | Host-notified resize; deadlines/idle remain independent |
| Session | Same resource facts, Editing | Compatibility poll observes changed dimensions |

For each admitted native profile the concrete draft is `é!界`, cursor byte 4,
visible column 8 after prompt `caps> `. A structured `# Output` transaction
preserves it. Resize 80→12 columns preserves text/cursor and updates the snapshot;
idle interest then has no deadline. Explicit interrupt/close restores the exact
termios snapshot while the caller still owns the PTY; paste is disabled.

Framed `first\r\nsecond` submits `first\nsecond` atomically. Unframed
`first\rsecond` submits `first`; `second` remains owned read-ahead and is applied
on reopening. This is explicit degradation, not equivalent multiline semantics.
The retained submitted editor is deliberately not cleared by the fixture, so its
reopened text becomes `firstsecond`; clearing submitted text is host policy.

Existing [embedding PTY](../../tools/embedding_pty.py) remains the external-host
oracle for blocking/session/driven input, actual host readiness waiting, opaque
expiry, resize, structured output, 30 lifecycles and native memory tools. Its TERM
refusal case now initializes real dimensions so the failure identifies protocol
admission rather than accidentally failing zero-size geometry.

## Qualification commands and promotion scope

```sh
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo test --test capabilities_pty -- --nocapture
python3 tools/qualify.py --work /tmp/replai-f2-qualified
python3 tools/perf/run.py --prepare --families components --work /tmp/replai-f2-after
python3 tools/perf/run.py --families components --work /tmp/replai-f2-after
python3 tools/perf/results.py validate /tmp/replai-f2-after/baseline.json
```

The existing CI retains Linux/macOS real embedding and C/memory checks and native
Windows portable tests. The embedding jobs additionally print the native profile
observations. No old gate is removed or replaced by synthetic resolver tests.

## Qualified source and platform observations

Implementation and measured source:

```text
HEAD 12cef7608d8243383b432f303fd3a7218cb9b20c
TREE 67570744fca534bd5e2341ee967e67adae310485
```

Full qualification source, including the C PTY observer correction:

```text
HEAD 69205d104a2a2bdd88b9ad03176fc58b5a3e266b
TREE 18e8c687aec9617f999b877c86e237c1dd6a2f78
```

[Native/portable CI](https://github.com/mothx9/replai/actions/runs/34370575960)
passed all 12 jobs. This is qualified implementation evidence; later roadmap and
producer-metadata commits carry this evidence without changing executable source.
The producer snapshot identifies its exact source/document carrier separately.

| Scope | Actual observation |
| --- | --- |
| Local Linux aarch64 | Complete `tools/qualify.py --work /tmp/replai-f2-final-qualified` passed, including clean-tree gate, release, Rust/PTYS, generated ABI/layout/symbols, isolated static/shared/C++ consumers and Valgrind. |
| Linux native memory | Static/shared contract runs: 0 Valgrind errors; 384 open/close transitions preserve exact termios, caller descriptors valid and FD count 8→8. |
| Linux/macOS real profiles | All four admitted profiles and negative cases above passed through actual PTYs with an independent VT screen oracle. The host configures protocol assumptions; OS resource facts are observed. |
| macOS native C/memory | Static/shared C lifecycle: 384 exact termios restorations, FD count 9→9; native `leaks --atExit` reports 0 leaks / 0 leaked bytes in each contract run. |
| macOS external host | 30 reopen cycles: active descriptor observation stays 9; 0.4-second idle requires no REPLAI advancement or deadline. Host resize receipt was 149.042 µs in this CI run, not a latency guarantee. Native embedding leak gate passes. |
| Windows | Native portable editor/engine/document/capability tests and benchmark integrity pass. No Windows terminal runtime is implemented or qualified. |

The first implementation CI run failed in the **C observation harness**, not the
library: its log already contained `EVENT 3` and `DESTROY 0`, with paste disabled,
but the observer raised after draining the exited process without rechecking the
new event. The correction drains before the final event search; three deterministic
observer tests prove late EOF is accepted and missing/wrong/prior events still
fail. The complete native CI above reruns C and memory qualification with that
correction. No runtime change or weakened terminal assertion was needed.

## Performance boundaries

Prepared binaries and all benchmark source hashes were verified before and after
measurement on `spark-7c3d`, Linux aarch64 `6.17.0-1021-nvidia`, Rust 1.98.1
(`48a229cea`), release opt-level 3. CPU parts reported `0xd85`/`0xd87`.
The optional P0_MACHINE free-text label was unset; automatic machine/toolchain,
load, frequency and source records are retained. These are sequential local
component characterizations, not a controlled cross-library or primary real-PTY
burst ranking. No previously qualified 1000-byte end-to-end number is relabeled
as an F2 measurement.

Before/after full component families contain 5,944 / 5,946 validated records
(timing and allocations). Raw evidence remains in `/tmp/replai-f2-before` and
`/tmp/replai-f2-after`; reproducible commands are above. All 2,972 comparable
allocation records are identical, including requested/retained/peak bytes. New
resolution allocates nothing and performs no terminal call. Component median
latency ratios range 0.969–1.005; this supports no observed systematic slowdown,
not a general speedup claim.

| Exact component (80 columns unless indicated) | Before median µs | After median µs |
| --- | --- | --- |
| Interaction append, 64-byte ASCII | 0.112 | 0.112 |
| Middle insert, 64-byte ASCII, engine/layout/render | 1.824 | 1.792 |
| Short completion replacement | 0.624 | 0.576 |
| ASCII paste layout/render, 1024 bytes | 22.832 | 22.512 |
| ASCII end layout, 64 bytes | 1.616 | 1.536 |
| Render cursor-left transition, 64 bytes | 0.096 | 0.096 |
| Coordinated external output, 1024 bytes | 3.216 | 3.216 |
| Idle interest query | 0.032 | 0.032 |
| Pure editing admission, new isolated measurement | — | 0.048 |

Admission is executed once per native open before raw-mode configuration; it is
not a key/frame cost. Native acquisition also avoids the former second initial
size query. No separate syscall-inclusive open-latency claim is made. The engine,
input batching, compatibility wait and external driven wait paths are unchanged.
Virtual idle counters remain zero required wakes and zero transport reads; real
external-loop qualification confirms that admission adds no timer.

On this aarch64 build, `Interaction` grows 608→656 bytes, exactly the 48-byte
snapshot; `TerminalConfig` is 21 bytes. Editor remains 128 bytes, and deadline,
wake and wait-interest representations remain 32 bytes each. Base construction
and resolution have zero heap allocations. These sizes are observations, not ABI.

Exact source-inventory SHA-256 values:

```text
before e4046310486a9fef0eba4d04a6c51b16562ea5a7edcc326492a8486683e5b682
after  2f3ea2ed96dcfb91c98dca7fd2820dcd9659018d9f3229c647da2140caf778ca
```

Raw `baseline.json` SHA-256 values:

```text
before 0c52f658dbf2be819a9a71c0d43c3cc09c8ccb0d46cabc7f918cf4cada765fc4
after  bde12f75bf61398aea5266b4cb17d0fe1d0d9adf3de07d0adfe5612538bf19a6
```

## Deliberate limits

No ConPTY backend or active terminal/emulator probing exists. Styling support
means the currently implemented intensity/256-color palette; weaker color can
conservatively degrade to plain. Width is deterministic but actual font, joined
emoji, bidi and ambiguous-width settings may disagree. Required native resource
facts cannot be configured around. NO_COLOR is honored by environment convenience;
a host constructing a fully explicit configuration owns its policy choices.

C ABI 1 remains an exact session compatibility surface and does not expose the
new snapshot, requirements or width query. This visibility gap belongs to E3,
not an improvised ABI 2. No I0, output concurrency, terminal feature expansion,
consumer migration or automatic next-wave selection is part of this work.
