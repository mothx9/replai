# Completion contract and presentation evidence

This dossier scopes I1/U1 evidence. [ROADMAP](../../ROADMAP.md) owns promotion;
[interaction](../interaction.md#revision-bound-completion-candidates) and
[presentation](../presentation.md#completion-surface) own behavior.

## Identity and ownership

Baseline master: `fea24ddaf6f987a8966bd149b5bcb16a51e892bd`, tree
`85c252f4a4e2efdcbb3b4e5f3f586a437ffc3261`; its direct predecessor is qualified
I0 `cb79a70bed1ed167aff61e5aed87e35260a255a2`.
Implementation and qualification identities are recorded below.
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
receipt is not proof that an entire supplied sequence has completed. The session
observer must also wait for the new prompt after submission before sending EOF: a
host result line precedes re-acquisition and is not proof that raw editing mode
has reopened. Fragmentation tests enforce this boundary without timing sleeps.

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

The following qualified results supersede the implementation checkpoint
without broadening the feature scope.

## Qualified implementation and platform matrix

Implementation:

```text
HEAD 05d574caf709f596269072ac8348611823555992
TREE e4804b9dcbc3a0067b1f20f0cec9fce9a6f318f8
```

Qualified source and measurement-fixture correction:

```text
HEAD 9f245f115a12d781f1fb279d1b8da08b89f780ec
TREE 400982b62309e94f77304591e3cfdeeb09ca033f
```

[CI 34482778339](https://github.com/mothx9/replai/actions/runs/34482778339)
passed **12/12 jobs**. The previous fixture attempted 1,000 annotations above the
4 MiB admitted aggregate; correct limit rejection stopped the benchmark. The
correction reduces that admitted workload to 128 repetitions per annotation,
without changing runtime limits or behavior. Oversized rejection remains tested.

| Scope | Executed evidence |
| --- | --- |
| Linux aarch64 local | Full `tools/qualify.py` executable gates, real PTY session/driven completion, ABI 1 static/shared/C++ and native Valgrind. The initial development run did not claim the clean-worktree gate; closure repeats it on the published carrier. |
| Linux native CI | Same completion oracle in styled and NO_COLOR modes, delayed stale/fresh result delivery, output/resize, 12 repeated close/reopen cycles per profile with stable FDs, Drop/read-failure and type-ahead. Both completion Valgrind processes pass with zero errors. Existing C memory/static/shared gates pass. |
| macOS native CI | Same external readiness and session oracle, exact draft/cursor/selection, termios/paste restoration and 12 stable lifecycle cycles per profile. Both active completion processes must report native leaks `0 leaks for 0 total leaked bytes`; the existing C static/dylib/leaks gate passes independently. |
| Windows portable | Public candidate safety/bounds, Engine selection/provenance/layout, generated interleavings, virtual transport, existing I0/F2 model and documentation tests pass. No Windows runtime backend is claimed. |
| Existing contracts | Full Rust release/debug, C generation/layout/symbols/static/shared, F2 matrices, embedding deadlines/idle/resize and benchmark-integrity jobs remain green. ABI declaration/header/binding files have no diff from the baseline. |

[Retained native receipts](completion-contract/platforms.json) identify the exact
source, CI and styled/plain/memory scenarios. Subsequent carriers add evidence,
an additional stale-delivery observer check against an already visible fresh
menu, and producer metadata; they do not alter the qualified runtime.

## Performance and resource results

Full suites: **6,088 before / 6,688 after** validated rows. The 600 additions are
300 completion cases in separate latency/allocation modes. All **2,977 existing
measured allocation rows are identical**. Retained
[before](completion-contract/before.json) and [after](completion-contract/after.json)
subsets preserve source/file/environment identities, full counts and raw hashes.
Unselected scales remain in the full local artifacts. Timing is release-mode on
spark-7c3d, Linux aarch64, with an observed 16 ns clock granularity.

| Existing operation | Before median µs | After median µs |
| --- | ---: | ---: |
| Editor append, 1 KiB ASCII | 0.064 | 0.064 |
| Editor Left / Right, 1 KiB | 0.032 / 0.064 | 0.032 / 0.064 |
| Atomic 64 KiB paste insertion | 38.960 | 38.992 |
| History Up / draft restore, 100 entries | 0.048 / 0.032 | 0.048 / 0.032 |
| Engine append with rendering, 1 KiB | 0.224 | 0.224 |
| Engine completion replacement with rendering, 1 KiB | 12.768 | 12.736 |
| Public Editor::replace_at, 1 KiB | 0.032 | 0.032 |
| Public Interaction::complete_at, 1 KiB | 2.128 | 2.176 |

The [direct public API control](completion-contract/direct.json) runs one
[unchanged fixture](../../tools/perf/direct_completion.rs) against exact baseline
and qualified-source archives. Only a benchmark entry is added to each temporary
archive build. Runtime files are unmodified; these are not Git worktrees. The
[driver](../../tools/direct_completion_benchmark.py) records binary and fixture
hashes, 63 samples per source, exact output and separate allocation observations.
Opening/closing/draining is outside the timed replacement; its write is included.
Both sources retain 0 editor-replacement allocations and 21 native-replacement
allocations, with exactly 132 terminal bytes for the native operation.

[Alternating primary control](completion-contract/paired.json): 1,000 ASCII bytes
+ Left + X + submission, **239.665 → 245.153 µs**, 63 samples each and two warmups.
p95 is 367.281→373.056 µs; MAD 22.016→23.360 µs. Terminal output is exactly
**1,054 bytes for both sources**. This includes submission cleanup and host IPC,
not only key-to-visible rendering. The 2.3% median difference is smaller than
within-run dispersion; isolated operations and allocation controls support no
material hot-path regression at the measured scope, not a universal latency
promise or claimed speedup.

The same alternating run retains original exact terminal-byte cases: ASCII
append 50.176→48.096 µs, Unicode append 38.000→42.864 µs, completion
135.249→90.273 µs, 64 KiB paste 2656.885→2701.478 µs. Scheduling-sensitive
individual cases are not treated as causal speed rankings. Compatibility resize
remains about 100.65 ms; the driven contract and its wake-free idle stay separate.

| Short candidates, 80 columns | Build set µs | Retained bytes | Construction allocations | Install µs | Layout µs | Next µs | Accept µs |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.128 | 96 | 7 | 0.784 | 0.560 | 0.624 | 0.560 |
| 10 | 1.056 | 960 | 61 | 2.176 | 1.616 | 1.984 | 0.848 |
| 100 | 13.040 | 9,780 | 601 | 2.944 | 1.776 | 2.032 | 2.016 |
| 1,000 | 126.704 | 99,780 | 6,001 | 8.800 | 1.936 | 2.304 | 13.376 |

Construction includes fixture string formatting and validated copies; it is not
pure discovery-free validation. Retained bytes are candidate metadata plus owned
strings, excluding stack values and allocator rounding/RSS. Installation moves
the set without cloning it. Acceptance includes releasing all retained candidate
storage, explaining its count-dependent cost. Long annotations and Unicode
cases are retained in the same dataset; selected navigation still lays out only
the visible rows. Normal editing allocates no candidate state.

| Ten short candidates, 80 columns | Encoded bytes |
| --- | ---: |
| Initial show | 279 |
| Next / previous (wrap to last page) | 142 / 236 |
| Accept / dismiss | 93 / 87 |
| Resize 80→81 columns | 355 |

These are the shared protocol encoder's exact benchmark counts; native
transaction bytes, resize and Escape-deadline timings are recorded separately in
the platform receipts. Within-page navigation updates changed menu/header rows; a page-size change
may redraw the owned surface, never the entire terminal. The combined frame currently rebuilds bounded visible geometry;
further allocation reduction remains measured optimization work, not a second
renderer. Terminal transports retain the existing batched write path.

| Object, Linux aarch64 | Before bytes | After bytes |
| --- | ---: | ---: |
| Editor | 144 | 144 |
| Interaction | 672 | 688 |
| CompletionCandidate | — | 64 |
| CompletionSet | — | 32 |

## Reproduction and closure carriers

```sh
python3 tools/qualify.py --work /tmp/replai-completion-qualified
python3 tools/perf/run.py --prepare --families components,linux --work /tmp/replai-completion-after
python3 tools/perf/run.py --families components,linux --work /tmp/replai-completion-after
python3 tools/perf/results.py validate /tmp/replai-completion-after/baseline.json
python3 tools/completion_pty.py --memory --work /tmp/replai-completion-memory
```

Run the same full suite on the exact clean baseline before mutation. Preserve its
PTY binary for `tools/analysis_benchmark.py`; the separate direct control takes
`--before REV --after REV --out FILE`. Native CI repeats correctness and memory
oracles on each published carrier. Producer metadata records its own exact
qualified source rather than attempting a self-referential enclosing Git hash.
