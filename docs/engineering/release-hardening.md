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
python3 tools/hardening/campaign.py --work /tmp/replai-hardening-campaign --cpu-seconds 3600
```

The campaign requires Linux, a C++ compiler and a **fresh** output directory.
It builds all five address-sanitized coverage-guided targets, runs them in
parallel, and accumulates `wait4` user + system CPU seconds separately per
target. Thirty-second wall-limited chunks permit bounded monitoring; wall time
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

No product defect has been established by these initial observations. Subsequent
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
