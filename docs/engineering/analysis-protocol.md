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

Implementation/measured source:

```text
HEAD 81d102e74e0aa0b42aa3ad569315c00c3d34b675
TREE b4777e327d7ae81ce786f4fd40c8a4f6a0bf0050
```

Qualified source including the Linux signal-observer correction:

```text
HEAD 39d41fef51012a8b4e509928846489026d908654
TREE a3a016655bf1ff7e9fe198601acf7395e3f870d7
```

[Native CI](https://github.com/mothx9/replai/actions/runs/34475749705) passed **12/12
jobs**. Later evidence/roadmap/producer carriers preserve this implementation;
the producer snapshot carries its own exact source revision/tree rather than
claiming that metadata was present in the implementation commit.

| Scope | Actual qualification |
| --- | --- |
| Local Linux aarch64 | Complete `tools/qualify.py --allow-dirty --work /tmp/replai-i0-development2` passed every executable gate; this development run explicitly did not establish the clean-tree gate. Clean published qualification is rerun for closure. |
| Portable model | Five public analysis tests, 20,000 generated transitions, checked exhaustion and common engine/decoder/output conformance. Native Linux/macOS/Windows CI executes the same portable tests. |
| Linux/macOS real analysis | External native wait loop plus independent application socket delivers delayed analysis. Stale results emit zero terminal bytes and preserve exact draft/cursor/screen; fresh results apply. History return, atomic/rejected paste, completion, output, resize, submit/reopen, EOF and 20 repeated lifecycles pass on both OSes. |
| Existing embedding/F2 | Blocking/session/driven PTYs, readiness/deadline/idle/resize, admission profiles and resource cleanup remain green. I0 changes no capability or scheduler policy. |
| Linux C/memory | Exact ABI generation/layout/symbols, isolated static/shared and C++ qualification; 384 open/close transitions with FD count 8→8 and exact termios. Both native Valgrind contract runs report 0 errors. |
| macOS C/memory | Static/shared and native leaks qualification; 384 exact restorations, memory-run FD count 9→9, 0 leaks / 0 leaked bytes. Existing reactor native leak gate also passes. |
| Windows | Portable public snapshot/revision model, engine/document/capability tests, doctests and benchmark integrity pass. No Windows terminal runtime is claimed. |

Two qualification failures were observer defects with direct evidence:

- A pipe read may end midway through `STATE`. The new analysis fixture exposed
  the shared observer accepting that prefix; it now waits for a complete new
  record. `tools/test_analysis_pty.py` tests every fragmentation point, with and
  without an older complete state.
- The first CI's Linux release unwind test read `/proc/self/status`, observing
  the launcher thread's transient blocked-signal mask while libtest created
  threads. Its before/after masks were `0` and `fffffffffffbfeff`. A controlled
  two-thread reproduction changed only the main thread mask: `/proc/self/status`
  reported `0x200`, while `/proc/thread-self/status` remained `0`. The test now
  observes the calling thread, retaining process-wide handler/ignore checks.
  The complete green CI above qualifies this correction; no signal or runtime
  implementation changed.

## Performance and memory observations

Same Linux aarch64 host (`spark-7c3d`, kernel `6.17.0-1021-nvidia`, Rust toolchain
recorded in the data), normal release builds. Full existing components + Linux
families produced **6,068 before / 6,088 after** valid rows. The 20 new rows are
snapshot creation/clone at five sizes in separate timing/allocation modes.
All **2,973 existing allocation result sets are identical**. Retained
[before](analysis-protocol/before.json) and [after](analysis-protocol/after.json)
subsets preserve exact source/file/binary/environment identities, selection rules,
full-result counts and full raw SHA-256. Omitted scales remain in the full local
run artifacts, not hidden substitutions for selected measurements.

| Isolated operation | Before median µs | After median µs |
| --- | ---: | ---: |
| Append to 1 KiB ASCII editor | 0.064 | 0.064 |
| Left / Right in 1 KiB ASCII editor | 0.032 / 0.064 | 0.032 / 0.064 |
| Short completion replacement, 1 KiB | 0.048 | 0.048 |
| Atomic 64 KiB ASCII paste insertion | 39.152 | 38.960 |
| History Up / original-draft return, 100 entries | 0.048 / 0.032 | 0.048 / 0.032 |
| Engine append with render, 1 KiB | 0.224 | 0.224 |
| Engine completion with render, 1 KiB | 12.640 | 12.689 |
| External-output render transition, 64 B | 0.160 | 0.160 |

The clock's observed 16 ns granularity limits interpretation of tiny operations.
No speedup is attributed to I0. Sequential PTY medians were scheduler-sensitive:
ASCII append moved 138→252 µs while isolated editing remained unchanged. We
therefore rebuilt the exact baseline from `git archive` into a temporary source
/build directory and alternated both exact binaries against the same oracle.
This was a build fixture, not a Git worktree or consumer checkout.

[Alternating source control](analysis-protocol/paired.json): the existing P0
**1000-byte ASCII burst + Left + X + submission** workload measured **252.992 →
255.536 µs** (63 samples each, two warm-ups; p95 449.361→434.721 µs). The endpoint
includes rendering, submission cleanup and receipt IPC; it is not key-to-visible.
The same run retains 31-sample pairs for the original exact terminal-byte cases.
One Unicode case remained bimodal, so a separately reported
[same-core control](analysis-protocol/affinity.json) pinned observer and both
children to CPU 0: primary 385.873→387.040 µs; Unicode append 234.912→234.001 µs;
64 KiB paste 4933.303→4937.847 µs. These are separate scheduling conditions,
not a combined ranking. The isolated, allocation and alternating controls support
no material I0 hot-path regression at this scope, not a universal latency bound.

| Snapshot text | Create median µs | Allocations | Retained allocation bytes | Clone allocations |
| --- | ---: | ---: | ---: | ---: |
| Empty | 0.032 | 1 | 16 | 0 |
| 64 B | 0.032 | 1 | 80 | 0 |
| 1 KiB | 0.048 | 1 | 1,040 | 0 |
| 64 KiB | 0.640 | 1 | 65,552 | 0 |
| 1 MiB admitted draft | 16.288 | 1 | 1,048,592 | 0 |

Clone median is 0.032 µs at each size; it copies no text. Retained allocation is
text length plus Arc metadata on this target, excluding allocator rounding/RSS.
The host controls how many independently created snapshots it retains; REPLAI
keeps no job queue/cache. Ordinary editing allocates no snapshot or revision data.

| Object on measured Linux aarch64 | Before bytes | After bytes |
| --- | ---: | ---: |
| Editor | 128 | 144 |
| Interaction | 656 | 672 |
| DraftRevision | — | 16 |
| AnalysisSnapshot | — | 48 |

## Reproduction

```sh
python3 tools/qualify.py --work /tmp/replai-i0-qualified
python3 tools/perf/run.py --prepare --families components,linux --work /tmp/replai-i0-after
python3 tools/perf/run.py --families components,linux --work /tmp/replai-i0-after
python3 tools/perf/results.py validate /tmp/replai-i0-after/baseline.json
```

Run the same P0 commands on the exact baseline before mutation for a sequential
comparison. For the alternating control, archive baseline `d7a0a7f…` into a fresh
temporary directory, build its `tools/perf/Cargo.toml` `pty-host` in release mode,
and run [analysis_benchmark.py](../../tools/analysis_benchmark.py) with explicit
`--before`, `--before-revision`, `--after`, `--after-revision`, `--out` arguments.
The after executable is the release `pty-host` from measured `81d102e…`; the
fixture records binary hashes and samples. The affinity control uses `taskset -c 0`
with otherwise identical arguments. No current-source rebuild is passed off as a
baseline. Native CI preserves the complete existing benchmark integrity suite.

## Limits

Revision equality is scoped to the originating retained editor, not a global
routing identifier. Hosts pair their own semantic context with snapshots. Checked
counter exhaustion panics before mutation; it never wraps. Snapshot creation is
linear in draft bytes and host retention remains host-owned. Live Interaction is
still exclusively mutated; an immutable transferable snapshot does not grant
concurrent writer safety. C ABI 1 does not expose I0. I1 candidate models, I2
hints/highlighting, I3 validation and U1 presentation are not implemented here.
