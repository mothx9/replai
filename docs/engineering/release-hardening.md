# Release hardening evidence

[ROADMAP](../../ROADMAP.md) owns maturity and selection. The
[release scope](../release-scope.md) owns the acceptance budgets. This dossier
records the campaign against the existing v0.1 surface; an implemented harness
is not an executed release gate.

## Source and scope

Reconciled baseline: `bfe9d1cef9303371e123d44d07c5d295908968c2`, tree
`9a79c98e79d4b229b0c4f16cc67d7d10182b15f6`. Production source is initially
unchanged. Qualification tooling compiles the same private source modules in an
isolated crate, following the existing performance-fixture approach. C calls
use the actual ABI binding and live caller-owned storage. There is no new
library feature, exported API or dependency in the production workspace.

## Reproduction

From the repository root with Cargo on PATH:

```sh
cargo install cargo-fuzz --locked
rustup toolchain install nightly --profile minimal
cargo build --locked --release --manifest-path tools/hardening/Cargo.toml --bin replay
tools/hardening/target/release/replay generated 100000 1
tools/hardening/target/release/replay faults 100
python3 tools/hardening/campaign.py --work /tmp/replai-hardening-campaign --cpu-seconds 3600 --workers 3
```

The campaign requires Linux, a C++ compiler and a **fresh** output directory.
It builds all five address-sanitized coverage-guided targets, runs them in
parallel, and accumulates `wait4` user + system CPU seconds separately per
target. `--workers 3` runs three independent processes per target over its shared
corpus. Each child has its own `wait4` receipt; only summed user/system CPU
satisfies the budget. Thirty-second wall-limited chunks permit bounded monitoring; wall time
does not satisfy the CPU budget. Every chunk retains arguments, timestamps,
CPU time, peak RSS, exit status and libFuzzer statistics. A finding stops that
target and fails the campaign. It is never counted as a passing budget.

`environment.json` records source/tree, dirty state, OS/libc and compiler/fuzzer.
Each target's `summary.json` records binary hash and initial/final corpus digest
(SHA-256 of sorted content hashes). Initial seeds include fixed protocol/Unicode
cases and 16 reproducible LCG seeds. The replay binary accepts `TARGET CORPUS`
or `TARGET FILE`, using exactly the same oracle as fuzzing. Windows runs the four
portable targets; C requires a native POSIX terminal backend.

## Target inventory

| Target | Ownership boundary and oracle | Initial execution |
| --- | --- | --- |
| protocol | VT/UTF-8 fragmentation, expiry, paste atomicity, safe semantic actions | Short smoke passed; full budget pending |
| editor | Independent whole-string/grapheme model, history/draft restoration, cursor/revision and rejection atomicity | Short smoke and 1,000 generated sequences passed; full budget pending |
| results | Retained snapshots, completion/validation/I2, mutation/lifecycle, stale zero-effects, noncanonical presentation | Short smoke and 1,000 generated sequences passed; full budget pending |
| geometry | Safe host fields, deterministic documents/frames, width and bounded cursor viewport | Short smoke passed; full budget pending |
| cabi | Actual ABI records/handles/buffers, native PTYs, invalid state/UTF-8, restoration | Short smoke passed; full budget pending |

Initial Linux ARM64 smoke used cargo-fuzz 0.13.2 and nightly
1.100.0 (`a36d05efa`, 2026-09-09). This is instrumentation identity, not the
release MSRV. The full campaign must retain its own exact compiler/source
receipt. No macOS/Windows replay or native stress is implied by this smoke.

## Findings ledger

| ID | Target / reproducer | Observation and cause | Classification / repair | Status |
| --- | --- | --- | --- | --- |
| H001 | results / generated seed 1 | Harness unwrapped a candidate constructor for a deliberately reversed range | Harness defect; respect constructor rejection before attempting installation | Replayed 1,000 sequences; closed |
| H002 | cabi / empty input | Harness expected a second destroy of the nulled owner to succeed | Harness defect; ABI 1 explicitly rejects the null handle with INVALID_ARGUMENT | Empty input and short fuzz replay passed; closed |
| H003 | geometry / retained 11-byte regression | Harness unwrapped rendering when a heading did not fit a narrow geometry | Harness defect; compare deterministic render outcomes, inspect safe bytes only on success | Minimized with libFuzzer; [regression](../../tools/hardening/regressions/geometry/heading-too-narrow) retained; geometric budget restarts |
| H004 | native / PTY hangup | Harness assumed is_open became false before a failed restoration was explicitly closed | Harness defect; is_open documents outstanding cleanup ownership. Assert explicit cleanup failure, close/release and stable descriptors | Native disconnected read/write smoke passed |
| H006 | native / macOS read and write hangup | Harness required Linux-style cleanup failure even when Darwin restores termios successfully | Harness defect; compare actual restored termios when available and require explicit failure when the OS refuses | Full native and leak replay pending |
| H007 | Q2 / append allocation calibration | The initial 1 KiB insertion filled capacity; the next append legitimately grew storage | Harness defect; separate warmed append from append-grow, with preparation outside timing | Initial registration refused before any thresholds were written; fresh controls required |
| H008 | Q2 / first ARM control receipt | A later Cargo build replaced the unpreserved control executable; its hash no longer matched the registration | Harness defect; freeze executables inside each evidence directory and refuse dirty-source qualification | No candidate comparison occurred; unusable calibration retained, new registration required |
| H009 | Q2 / composed 1000-byte submission | Byte accounting retained only the final close effects, omitting the preceding draft flush on SubmissionRequested | Harness defect; retain and count both mutation batches outside timing | New controls and registration required; prior passing receipt does not qualify the corrected byte metric |
| H010 | native instrumentation timeout | Partial stdout was discarded by subprocess timeout, obscuring the macOS leak-phase stall; descendant instrumentation could survive controller timeout | Harness defect; stream receipts to files, sample timed-out Darwin processes and kill the isolated process group | Linux timeout regression passes; macOS diagnosis and replay pending |
| H005 | native / driven under Valgrind | Harness assumed a ready notification consumes the entire write despite the bounded work budget | Harness defect; drain WaitInterest::Ready before evaluating the semantic result | Native memory campaign replay required; no product scheduling change |

100,000 generated seeds 1–100,000 passed on Linux ARM64, with 128 operations per
editor/history sequence and 128 per host-result/lifecycle sequence. The five
long fuzz campaigns started at harness checkpoint
`b812afac2706ea11321239e15788cee8ae5c4fdf`; geometry stopped on H003 at 27.581449
CPU seconds, which does not count as its passing 60-minute campaign.

No product defect has been established by these observations. Subsequent
findings remain in this ledger even after repair; a short clean smoke is not a
security certification or evidence that a longer campaign will find none.

## Resource, usability and regression gates

Initial deterministic virtual fault sweep: 100 repetitions at each selected call
position; observed fault hits: geometry 500, read 400, write 800, restoration 100.
The one-shot restoration failure exercises retry, not an impossible OS recovery
claim. All paths ended closed with restored virtual resources. 100,000 idle
interest queries performed zero transport calls. These are **virtual** observations,
not real PTY resource-stress evidence.

Still required: 1,000 lifecycle cycles per native Rust tier and C target;
10,000 mixed events per native target; native failure/descriptor exhaustion;
Valgrind and macOS leaks; keyboard/plain/narrow review; final corpus replay;
100,000 generated sequences; pre-registered Q2 control batches and candidate
comparisons; full existing CI. Linux x86_64 and ARM64 and macOS ARM64 each need
their own receipts. Missing native evidence blocks closure.

Q2 thresholds are not registered by this initial dossier. The adopted formula
and workload list remain in [G5](../release-scope.md#g5--q2-policy-e3-freeze-and-v0-release-qualification).
Prefix-layout traversal on large multiline drafts and synchronous host-output
blocking remain explicit limits. General U3 and deferred features are unchanged.

## Native and controlled measurement entry points

```sh
python3 tools/hardening/native.py --work /tmp/replai-native-hardening
python3 tools/hardening/native.py --memory --work /tmp/replai-native-memory
python3 tools/hardening/test_hardening.py
python3 tools/hardening/smoke.py
```

The native command runs 1,000 cycles for each Rust tier and ABI 1, a separately
counted 10,000-event mixture, eight native failure classes repeated 100 times,
and 100 descriptor-exhaustion repetitions in a child with NOFILE=64. The parent
limit never changes. Valgrind/native leaks instrumentation applies to lifecycle,
mixed and failure paths; descriptor exhaustion separately avoids confusing the
memory tool's private descriptors with product resources. Native architecture is
asserted by the manually dispatched
[campaign workflow](../../.github/workflows/hardening.yml). Normal CI executes
only deterministic smoke, regressions and harness guards.

The first Linux ARM64 lifecycle run on harness `0aef173` passed 1,000 blocking,
1,000 session, 1,000 driven and 1,000 ABI cycles; session/driven each exercised
20,000 serialized actions. The separate mixed run passed 10,000 actions.
Caller termios and descriptor counts matched after every connected cycle.
These counts do not establish the still-required native memory/tool receipts.

Q2 uses 32 named workloads in the isolated `bench` binary, five batches of 31
repetitions with three warmups per batch. Setup, snapshot capture, verification
and byte counting stay outside the timed operation. The allocation build uses
the existing counting allocator independently of the latency build. Register
before comparing, using fresh output directories:

```sh
cargo build --locked --release --manifest-path tools/hardening/Cargo.toml --bin bench
cargo build --locked --release --manifest-path tools/hardening/Cargo.toml --bin bench --features allocations --target-dir tools/hardening/target/allocations
python3 tools/hardening/regression.py register --work /tmp/replai-q2-control --binary tools/hardening/target/release/bench --allocation-binary tools/hardening/target/allocations/release/bench
python3 tools/hardening/regression.py compare --work /tmp/replai-q2-candidate --registration /tmp/replai-q2-control/registered.json --control-binary tools/hardening/target/release/bench --binary tools/hardening/target/release/bench --allocation-binary tools/hardening/target/allocations/release/bench
```

The example compares one unchanged binary to exercise the policy. A changed
candidate must use separate preserved control/candidate binaries and source
identities. Threshold exceedances in both alternating runs block investigation;
contemporaneous controls never raise the registered bound. Byte/allocation
oracles are internal regression guards, not exact VT public compatibility.
Hosted CI proves harness integrity, not controlled latency. Real observations, environment and thresholds must be attached before Q2 promotion.


The final scope audit expanded three campaigns: protocol now also advances the
actual generic Terminal with one-byte/seven-byte/4-KiB virtual reads and both
paste admission policies; geometry includes every current Document block;
host results include oversized aggregate/count/field payloads and overlapping
spans. These three targets require new full budgets. Earlier completed campaigns
remain observations of their recorded narrower harness, not evidence for added
branches. The editor target retains its original independently recorded 60-CPU-minute
budget. C was also expanded to exercise exact configured capacity and canaries
at each declared output-buffer boundary. Production sources remain unchanged.
The final parallel campaign supersedes incomplete development runs; interrupted
chunks never count toward its fresh budgets.


## Q2 registered reference and deterministic CI gates

The final reference at `6975c0979a1fd13f619f2d079b7494945ca18c5e` includes
the H009 composed-burst accounting repair: 1,042 encoded bytes, including the
intermediate redraw. The earlier reference remains superseded for that metric.

The corrected Linux ARM64 run on Spark (`spark-7c3d`, kernel
6.17.0-1021-nvidia, Rust 1.98.1) pinned measurements to CPU 19 under the
performance governor. Five control batches of 31 repetitions registered all
32 workloads before two alternating control/candidate comparisons. The first
comparison exceeded the registered bound for the 1-MiB cursor workload; the
second did not reproduce it. No workload exceeded in both independent runs, so
the adopted policy passed without changing thresholds. Allocation and
encoded-byte checks passed. These compare an unchanged runtime/binary: they
establish a regression reference and exercise enforcement, not a speedup claim.
The timer floor, per-workload MADs and allowances are retained verbatim in the
[registration](../../tools/hardening/evidence/q2-linux-aarch64.json); exact raw
samples and comparison receipts are in the
[compressed receipt](../../tools/hardening/evidence/q2-linux-aarch64-samples.json.gz).

```sh
python3 tools/hardening/deterministic.py
```

Normal portable CI executes three repetitions of each allocation/encoded-byte
oracle, with no latency assertion on hosted runners. Warmed append and cursor
movement require zero allocations; cold append growth is a separate measured
operation. The virtual fault gate additionally freezes zero-I/O idle interest,
one serialized output write transaction, and zero transport calls for stale
analysis delivery. These are internal regression fixtures, not a promise of
identical ANSI encodings across future versions.

Final corpus transport uses `tools/hardening/corpus.py pack` with ordered campaign
directories. Only completed budgets are admitted; each input and aggregate corpus
digest is revalidated before replay. A bounded immutable compressed receipt retains
source/toolchain/binary identities, CPU accounting, executions and exact inputs.
The manual final-corpus workflow runs the same archive on Linux x86_64/ARM64,
macOS ARM64 and Windows x86_64; Windows excludes native C. Neither an implemented
replay workflow nor a seed smoke is a completed final-corpus replay.


## Recorded native campaign

The [final native campaign](https://github.com/mothx9/replai/actions/runs/34598637866)
passed on native Linux x86_64, Linux ARM64 and macOS ARM64 at harness
`c68fc3a69763587440b212381c4e27cd8373d443`. Each platform completed 1,000
blocking, session, driven and C lifecycles, a separate 10,000-event mixture,
eight failure classes repeated 100 times and 100 isolated descriptor-exhaustion
children. The full counts also passed under Valgrind/native leaks (exhaustion
runs separately from the memory tool). Exact source, machine and tool receipts,
raw reports and terminal-transcript hashes are retained in the
[compressed native evidence](../../tools/hardening/evidence/native-campaign.json.gz).

The old macOS instrumentation run timed out without preserving partial stdout.
Its runtime root cause cannot be reconstructed from that deficient receipt.
H010 fixes the evidence-loss and descendant-cleanup defects; the replacement
campaign completes every phase within a stricter 120-second watchdog, with
zero attributable native leaks. The earlier timeout is retained as a failed
measurement, not relabeled as a product crash or silently counted as a pass.
