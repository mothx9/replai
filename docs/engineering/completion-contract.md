# Completion contract and presentation evidence

This dossier scopes I1/U1 evidence. [ROADMAP](../../ROADMAP.md) owns promotion;
[interaction](../interaction.md#revision-bound-completion-candidates) and
[presentation](../presentation.md#completion-surface) own behavior.

## Identity and ownership

Baseline master: `fea24ddaf6f987a8966bd149b5bcb16a51e892bd`, tree
`85c252f4a4e2efdcbb3b4e5f3f586a437ffc3261`; its direct predecessor is qualified
I0 `cb79a70bed1ed167aff61e5aed87e35260a255a2`.
Implementation and final qualification identities are recorded after the gates.
No consumer checkout, dependency pin, public C declaration or F2 policy changes.

The host supplies candidates from one retained snapshot; REPLAI stores only
bounded safe replacement/display data and selection. Per-candidate ranges admit
alternatives replacing different syntactic regions. A set-wide range was rejected
because that restriction has no terminal or editor justification. Structural
duplicates and host order survive unchanged.

Every nonempty set needs deliberate acceptance. Automatic single-item acceptance
and common-prefix insertion were rejected: asynchronously delivered data should
not edit before the user confirms. There is no preview-through-mutation.
Tab/Shift-Tab select; Enter accepts; Escape uses the existing decoder deadline.
Arrows preserve editing/history bindings. Configurable keymaps remain separate.

No completion request counter is added. Two responses for the same DraftRevision
both describe current editor state; explicitly delivered sets replace in delivery
order. Hosts own request preference and discard superseded jobs before delivery.
This is tested with B then A at the same revision, not called latest-request-wins.

## Qualification shape

- Public candidate tests: distinct display/insertion, duplicate preservation,
  safe Unicode, control/bidi injection and count/field/aggregate limits.
- Deterministic Engine tests: atomic whole-set validation, zero/single/multiple
  candidates, navigation, acceptance, I0 no-op behavior, stale zero-effects,
  history/paste/edit invalidation, output/resize retention, close cleanup,
  20/40/80/132-column layouts and 4,000 generated interleavings.
- Virtual transport: same decoder/engine/renderer, semantic screen/cursor,
  NO_COLOR and injected write failure cleanup; this is portable logic evidence.
- Native [PTY oracle](../../tools/completion_pty.py): actual external wait loop,
  delayed stale and fresh socket results, widths, output restoration, 1,000-item
  viewport, history/paste, decoder Escape deadline, repeated FD/termios lifecycle,
  and a separate synchronous session host. Valgrind/leaks execute active paths.
- Rust PTY regression: read-ahead across completion request/accept/submission and
  reopen, active-menu Drop and read-failure restoration.

The observer waits for complete operation acknowledgements and expected semantic
states. Memory-tool execution can split one ready burst; an intermediate STATE
receipt is not proof that an entire supplied sequence has completed.

## Measurement method

The unchanged before suite produced 6,088 validated results on spark-7c3d,
Linux aarch64. The before executable was retained for alternating same-machine
primary-workload comparisons. Component timing excludes setup and separately
counts allocations; snapshots/discovery are not charged to ordinary editing.

`tools/perf/completions.rs` measures 1/10/100/1,000 candidates with short labels,
long annotations and CJK/combining/ZWJ labels, at 20/80/132 columns. Construction,
validation, installation, layout, next/previous, resize, accept and dismiss are
separate operations. Terminal bytes come from the existing encoder; native PTY
receipts additionally record observed transaction bytes and wall-clock delivery.
Those observer timings include host/process scheduling and are not pure layout
latency. A menu is bounded to eight temporary rows, independent of set size.

## Scope and limits

Rich completion is native Rust only. ABI 1 retains its exact synchronous
completion replacement contract. Windows qualifies portable logic, not a runtime
terminal backend. F2 width assumptions remain: UnicodeNarrow does not guarantee
all emulator/font cell agreement. Long display fields are ellipsized and narrow
annotations omitted; insertion remains exact. Selection layout reconstructs the
bounded combined frame and reuses existing row transitions; no full-screen clear
or second renderer is introduced. I2, I3, I4, I5, O1/O2 and U2 remain outside scope.

Native matrix, measured numbers, implementation commit and final CI are pending
qualification at this implementation checkpoint; this dossier does not yet
assert I1/U1 closure.
