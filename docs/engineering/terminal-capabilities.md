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

Local portable/native profile tests and existing Rust all-target regressions pass
at implementation time. Full clean-source qualification, after measurements and
published native CI are required before promotion; the closure record below will
identify their exact source scope.

## Performance boundaries

Baseline measurement completed on the untouched HEAD above at
`/tmp/replai-f2-before`, with 5,944 validated timing/allocation records. The new
component measurement adds isolated capability resolution: acquisition is outside
steady-state editing, and pure admission contains no heap allocation or OS call.
Source inventory includes the new width module and all benchmark inputs.

Before/after object size, resolution timing/allocation results and retained
editor/render/output measurements must be recorded from prepared source identities.
Neither an extra enum nor one benchmark result implies general performance claims.

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
