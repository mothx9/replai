# Analysis protocol qualification

This dossier owns bounded I0 evidence. [ROADMAP](../../ROADMAP.md) owns maturity;
[interaction](../interaction.md#revision-aware-host-analysis) owns public semantics.

## Reconciled baseline

Canonical `mothx9/replai`, `master`, clean and equal to fetched origin:

```text
HEAD d7a0a7f070409dfc13c715da7dc0edcead02356c
TREE 2f1bdae65254ef1472e79d571c06380eedc1e9bc
```

The expected `bf76138…` advanced through README screenshot-only work, preserved
here. YAI, YVEX and BOUNDARY remain outside the mutation scope.

## Ownership decision and alternatives

Before: Editor owned text/cursor, the engine coordinated semantic lifecycle, and
completion accepted a range without analysis provenance. After: Editor retains
that state with one opaque checked 128-bit revision; Engine invalidates the same
identity at submit/interrupt/EOF. Interaction forwards snapshots and atomic
revision-bound completion. No terminal-specific revision implementation exists.

An engine-only counter would miss standalone/closed editor mutations. A hash of
text/cursor would revive old analysis after history restoration. A wrapping
counter could alias ancient results. A globally unique/random identity is neither
required nor introduced: revisions are scoped to the originating retained editor.
An AnalysisResult payload envelope is unnecessary for this first contract; hosts
pair the snapshot revision with their own derived payload/context. Future feature
contracts can reuse it without creating another identity system.

Snapshots use one immutable `Arc<str>` allocation/copy at initiation, shared by
clone. Live editing does not allocate snapshots. No storage redesign, parser,
analysis cache, callback, worker, mutex or executor is added. Existing unversioned
completion and C ABI 1 remain unchanged; C does not expose I0 provenance.

## Executable oracles

- `tests/analysis.rs`: immutable retained/shared text, cursor invalidation,
  no-op/rejected edits, history return and 20,000 deterministic generated mutations.
- `src/conformance.rs`: identical generic engine across lifecycle, resize/output,
  decoder partial input, atomic paste and stale effects with no terminal mutation.
- `src/core.rs` / `src/analysis.rs`: checked exhaustion refuses before state change.
- `examples/analysis.rs` / `tools/analysis_pty.py`: real host-owned POSIX wait loop,
  separate application result delivery, delayed stale refusal and fresh success,
  Unicode/cursor/history/paste/output/resize/submit/reopen/EOF and 20 stable cycles.
- `tools/perf/components.rs`: separate creation/clone timing and allocation at
  0, 64, 1024, 65536 and 1048576 bytes; object-size counters.

## Qualification and measurement scope

Qualification is in progress for this implementation revision. Portable smoke,
focused Linux model and real reactor tests have passed locally; final native CI,
full repository gates and before/after measurements will be recorded at closure.
No macOS runtime or Windows execution is inferred from local Linux results.
