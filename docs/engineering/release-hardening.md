# Release hardening evidence

This dossier records bounded qualification of the [selected v0.1 surface](../release-scope.md).
[ROADMAP](../../ROADMAP.md) alone owns maturity and the next selected boundary.
No release, package upload, new interaction feature or consumer repin is implied.

The first final-corpus replay exposed H011: the C fixture consumed PTY output
only after synchronous calls returned. A Darwin queue could fill during one
completion redraw, leaving the fixture blocked in `write`. This is terminal-peer
backpressure, not a promise that synchronous output is nonblocking. A dedicated
fixture reader now drains the PTY concurrently; it owns no library state or writer.
The retained reproducer, a fresh 3,668-second C campaign, full native-memory
matrix and cross-platform corpus replay all pass after that harness repair.

## Source and evidence identities

- Wave baseline: `bfe9d1cef9303371e123d44d07c5d295908968c2`, tree
  `9a79c98e79d4b229b0c4f16cc67d7d10182b15f6`.
- Production `src` tree throughout: `12b9cd0e58ef8d2dfe6c1b91185fd21b05a7aa9e`.
  Runtime source, root dependency graph, public Rust API, C headers/schema and
  binding implementation are unchanged from the wave baseline.
- Final expanded fuzz harness: `bd338ead57885257a4bbf554d0ec0c5b22e34a57`, tree
  `24b121dde64dc4bc9f9e7cbd9ac188ec074cc1a4`. Editor's unchanged independent oracle
  retains its completed campaign at `b812afac2706ea11321239e15788cee8ae5c4fdf`,
  tree `65a06ff46aee02ca16a2d00928c74598fe9e5f4f`.
- Final C terminal-peer harness: `4e9f8c1378de36a435007d7f46f94f1ecc6157bc`,
  tree `16f00f092a0d7fc63b411d157bf127692d4d32b0`.
- Final Q2 measurement harness: `6975c0979a1fd13f619f2d079b7494945ca18c5e`, tree
  `71bb4281b44481aecc90196af6ec3202a975f91e`.
- Final native campaign: `4e9f8c1378de36a435007d7f46f94f1ecc6157bc`;
  [three native jobs](https://github.com/mothx9/replai/actions/runs/34606480203)
  and [full CI](https://github.com/mothx9/replai/actions/runs/34606453021) pass.
  Later evidence/documentation carriers do not denote different runtime implementations.

The isolated [hardening workspace](../../tools/hardening/Cargo.toml) compiles
actual engine/protocol/presentation modules and the real ABI binding. It adds no
production feature flag or API. Independent model state supplies the editor oracle.

## Q0 campaigns and corpus identity

Five distinct ownership boundaries received at least 3,600 CPU seconds each:

| Target | Boundary and principal oracles |
| --- | --- |
| protocol | Fragmented VT/UTF-8 decoding, incomplete expiry, paste atomicity; real generic Terminal through virtual 1/7/4096-byte reads, paste admission/degradation and resize-adjacent state |
| editor | Independent Unicode/grapheme, capacity, revision and bounded-history state machine; accepted/rejected edits and restoration |
| results | Completion, validation, spans/hints, current/stale/malformed results, lifecycle, resize/output; atomic rejection checks canonical and presentation state plus zero effects |
| geometry | All current Document blocks, safe fields, narrow/large dimensions and Unicode; deterministic success or explicit rejection, canonical text/cursor preservation |
| cabi | Real ABI 1 records, legal handle/pointer sequences, versions/sizes, exact capacity, caller-buffer canaries, malformed UTF-8, PTYs and restoration |

Recorded Linux ARM64 environment: Spark `spark-7c3d`, kernel
6.17.0-1021-nvidia, glibc 2.39, cargo-fuzz 0.13.2, libfuzzer-sys 0.4.13,
Rust nightly 1.100.0 (`a36d05efa`, LLVM 23.1.1). Instrumentation is cargo-fuzz
AddressSanitizer, inline coverage and comparison tracing. Nightly is a test-tool
identity, not the release MSRV.

| Target | CPU seconds | Executions including corpus initialization | Final inputs | Corpus SHA-256 |
| --- | --- | --- | --- | --- |
| cabi | 3668.286148 | 2,755,786 | 7,442 | `fb2874dfd96b2cb3684ab61b2aca919ea862cfd6b5c8664fded07ad05c8e65b2` |
| editor | 3601.242122 | 4,790,559 | 7,296 | `48ce910b67ec5cb82fe22ba8a8383b205133ac6b3442175a7b07883703bc322e` |
| geometry | 3616.111162 | 1,934,034 | 6,769 | `aa9893f788d7f2eff93c731b7b953d5e54c153dd65ebfde06d9029162491a7b2` |
| protocol | 3685.213694 | 196,657 | 1,506 | `c090bd35b6df24be850ce09ed74288690b432bab08091ebce4cb5c303a1213b4` |
| results | 3604.104277 | 2,827,876 | 7,587 | `89b2692e1f04bad00c7144bd70797664f54aa1f102d439279a609d8b70fcd99e` |

The [immutable final corpus](../../tools/hardening/evidence/final-corpus.json.gz)
retains exact inputs, seed/corpus digests, source/tree, compiler, binary hashes,
chunk parameters, start/end times, user/system CPU, exit statuses and peak RSS.
Each child is measured with `wait4`; wall time is never substituted for CPU time.
Three workers per expanded target share its corpus; their CPU times are summed.
The scheduler pause used for Q2 does not count as CPU execution. Chunks use
30-second wall bounds, 10-second per-input timeout, 2-GiB RSS limits and 4-KiB
inputs (8 KiB for geometry). The base initial corpus has 29 deterministic seeds;
the final C campaign adds both retained minimized regressions. The unchanged
editor's earlier seed identity is retained independently.

Incomplete development campaigns were superseded after oracle expansion, not
combined into final budgets. Geometry's original H003 stop at 27.581449 CPU
seconds is a failed harness observation; its replacement campaign starts afresh.
No unexplained fuzz panic, crash, hang, stale mutation/output, unsafe terminal
content or broken grapheme invariant remains in the recorded final campaigns.
This is not security certification or proof against arbitrary future inputs.

Final corpus replay uses the same immutable archive on Linux x86_64/ARM64,
macOS ARM64 and Windows x86_64. Windows excludes native C and claims portable
semantics only. The final replay workflow must pass all four jobs against this
post-H011 archive before Q0 promotion.

## Generated semantic sequences

100,000 seeded pairs passed: seeds 1–100,000, at most 128 operations per
editor/history oracle and 128 per host-results/lifecycle oracle. The final
results replay includes malformed-current refusal as well as stale refusal.
The LCG, operation mapping and seed print-before-execution are retained in
[replay.rs](../../tools/hardening/replay.rs); a failing seed is directly replayable.
The model checks exact history/draft restoration, accepted/no-op/rejected revision
rules, cursor-only invalidation, byte-identical later drafts, completion navigation,
non-canonical hints, validation and lifecycle. No parser runs inside REPLAI.

```sh
cargo build --locked --release --manifest-path tools/hardening/Cargo.toml --bin replay
tools/hardening/target/release/replay generated 100000 1
```

## Findings and minimized regressions

No production defect was established; the repairs below change harness assumptions,
accounting or evidence retention. They do not expand the selected v0.1 contract.

| ID | Target / reproducer | Observation and cause | Classification / repair | Final evidence |
| --- | --- | --- | --- | --- |
| H001 | results / generated seed 1 | Harness unwrapped a candidate constructor for a deliberately reversed range | Harness defect; respect constructor rejection before attempting installation | Seeds 1–100,000 and final results corpus pass; closed |
| H002 | cabi / empty input | Harness expected a second destroy of the nulled owner to succeed | Harness defect; ABI 1 explicitly rejects the null handle with INVALID_ARGUMENT | Empty input, final C corpus and native C stress pass; closed |
| H003 | geometry / retained 11-byte regression | Harness unwrapped rendering when a heading did not fit a narrow geometry | Harness defect; compare deterministic render outcomes, inspect safe bytes only on success | Minimized regression and restarted full geometry budget pass; closed |
| H004 | native / PTY hangup | Harness assumed is_open became false before a failed restoration was explicitly closed | Harness defect; is_open documents outstanding cleanup ownership. Assert explicit cleanup failure, close/release and stable descriptors | Native disconnected read/write and repeated cleanup pass; closed |
| H005 | native / driven under Valgrind | Harness assumed a ready notification consumes the entire write despite the bounded work budget | Harness defect; drain WaitInterest::Ready before evaluating the semantic result | Full native memory campaigns pass with bounded Ready draining; closed |
| H006 | native / macOS read and write hangup | Harness required Linux-style cleanup failure even when Darwin restores termios successfully | Harness defect; compare actual restored termios when available and require explicit failure when the OS refuses | Native Darwin records 200 restorable hangups; Linux records 200 refused restorations; closed |
| H007 | Q2 / append allocation calibration | The initial 1 KiB insertion filled capacity; the next append legitimately grew storage | Harness defect; separate warmed append from append-grow, with preparation outside timing | Warmed/growing append are separate; corrected registration and comparisons pass; closed |
| H008 | Q2 / first ARM control receipt | A later Cargo build replaced the unpreserved control executable; its hash no longer matched the registration | Harness defect; freeze executables inside each evidence directory and refuse dirty-source qualification | Final control/candidate executables retained with verified hashes; closed |
| H009 | Q2 / composed 1000-byte submission | Byte accounting retained only the final close effects, omitting the preceding draft flush on SubmissionRequested | Harness defect; retain and count both mutation batches outside timing | Both effect batches counted: 1,042 burst bytes; fresh preregistration/comparison pass; closed |
| H010 | native instrumentation timeout | Partial stdout was discarded by subprocess timeout, obscuring the macOS leak-phase stall; descendant instrumentation could survive controller timeout | Harness defect; stream receipts to files, sample timed-out Darwin processes and kill the isolated process group | Partial-output regression and complete native leak replay pass; original instrumentation timeout retained as failed measurement |
| H011 | C corpus input `cb64c81e4cd085cd552d64d670f422871963fa8b8e89e8dfaa6de0ac19207390` | Darwin stack sample stops in synchronous completion write while the fixture waits to drain output | Harness defect; concurrent bounded terminal-peer reader, joined and released after each case | [Retained reproducer](../../tools/hardening/regressions/cabi/synchronous-output-peer), fresh C budget, native memory and macOS replay pass; closed |

The original macOS instrumentation stall lost partial stdout before H010.
Its runtime root cause cannot be reconstructed from that deficient receipt.
It remains a failed measurement, not a retroactively successful run or an
attributed product crash. The corrected controller writes receipts continuously,
retains timeout diagnostics, samples Darwin processes and kills the isolated
instrumentation group. Every phase of the replacement full native campaign
completed inside a stricter 120-second watchdog, with zero attributable leaks.

## Q1 native lifecycle and failure stress

The [compressed native receipts](../../tools/hardening/evidence/native-campaign.json.gz)
retain machine/kernel/compiler identities, exact binaries, raw memory reports,
per-phase JSON and hashes of terminal transcripts from the final workflow.
Each table cell is independently executed on its stated native architecture.

| Native target | Blocking / session / driven / C cycles | Separate mixed events | Failure classes × repetitions | Exhaustion children | Memory result |
| --- | --- | --- | --- | --- | --- |
| Linux x86_64 GNU | 1000 / 1000 / 1000 / 1000 | 10000 | 8 × 100 | 100 | Valgrind clean |
| Linux aarch64 GNU | 1000 / 1000 / 1000 / 1000 | 10000 | 8 × 100 | 100 | Valgrind clean |
| macOS aarch64 | 1000 / 1000 / 1000 / 1000 | 10000 | 8 × 100 | 100 | native leaks: zero |

These full counts also passed with memory instrumentation. Session/driven each
execute 20,000 serialized actions, separately from the 10,000-event mixed gate.
Memory-tool descriptor bookkeeping is excluded from the separate NOFILE=64
exhaustion child; the qualification controller's limits never change.

Eight native failure classes cover unsuitable admission, geometry/write failure
at acquisition, geometry failure while active, read/write hangup, configured
capacity and malformed host data. Acquisition under descriptor exhaustion refuses,
existing ownership cleans up, caller descriptors remain owned by the caller, and
acquisition can retry after pressure is removed. Descriptor counts and connected
termios match exactly after each lifecycle. Linux observed 200 non-restorable
hangups with explicit errors; Darwin restored all 200 exactly. Cleanup never
fabricates success when the OS refuses restoration.

The separate virtual fault-position sweep repeats each selected position 100
times: geometry 500 actual hits, read 400, write 800, restoration 100. It exercises
retry and resource release without pretending synthetic faults are OS discovery.
100,000 idle interest queries issue zero transport calls; stale analysis issues
zero transport calls; serialized output uses one write transaction.

Linux Valgrind 3.22.0 reports zero invalid accesses, definite/indirect/possible
leaks or suppressions. Reachable allocations belong to Rust's process-lifetime
stack-overflow registry (544 bytes), plus its stdin buffer in blocking hosts
(8192 bytes); retained stack traces identify those runtime owners. Darwin
`leaks` report format 4.0 records zero leaked bytes on Darwin 24.6.0 ARM64.
Arbitrary allocator abort, SIGKILL and process-wide OOM recovery remain unclaimed.

## Minimum usability review

Existing real-PTY cell/cursor oracles were replayed with the hardening source:
[completion](completion-contract.md), [validation/multiline](validation-multiline.md),
[analysis presentation](analysis-presentation.md) and [embedding](embedding.md).
Keyboard-only edit/history, completion request/next/previous/accept/dismiss,
validated continuation and indentation, multiline arrows, submission, interrupt
and EOF remain available. Completion Enter accepts once before validation.

20/40/80/132-column styled and plain/NO_COLOR profiles retain visible selection,
textual diagnostics, continuation prompts and cursor/draft identity. Plain hints
retain their non-canonical marker; hint bytes never enter submitted text.
Resize and serialized output preserve current menus/diagnostics/analysis.
10/100/1000-line scenes retain usable bounded viewport output. This is the
mandatory current-surface review, not general U3 closure or screen-reader certification.

## Q2 registered regression policy

The [registration](../../tools/hardening/evidence/q2-linux-aarch64.json) and
[raw control/comparison samples](../../tools/hardening/evidence/q2-linux-aarch64-samples.json.gz)
freeze 32 workloads. Spark CPU 19, performance governor, Rust 1.98.1/LLVM 22.1.8,
16-ns observed timer floor; preparation/snapshots/verification/byte encoding stay
outside the timed operation. Each control uses five batches of 31 measurements,
three warmups per batch; allocation instrumentation is a separate build.
Executable copies and hashes prevent Cargo rebuilds from changing the control.

Thresholds were recorded before candidate evaluation:
median allowance = max(15% baseline, 5 × control-batch median MAD, 2 × timer floor);
p95 allowance = max(25% baseline, control-batch p95 MAD, 2 × timer floor).
A repeated exceedance in two independent alternating control/candidate runs blocks
promotion. The first final comparison exceeded on the 1-MiB cursor workload;
the second did not reproduce it. No blocking intersection remained; allocation
and byte gates passed without changing thresholds or replacing the baseline.

This compares the unchanged runtime/binary to establish enforcement and a reference,
not a speedup. Hosted CI executes deterministic gates, never precision latency claims.
All values below are nanoseconds; allowances are absolute additions to the baseline.

| Workload / bytes / lines | Median ns | p95 ns | Median / p95 allowance ns | Allocation calls | Encoded bytes |
| --- | --- | --- | --- | --- | --- |
| append/1024/0 | 32 | 48 | 32.0 / 32.0 | 0 | 0 |
| append-grow/1024/0 | 64 | 80 | 32.0 / 32.0 | 1 | 0 |
| cursor/1024/0 | 32 | 48 | 32.0 / 32.0 | 0 | 0 |
| burst/0/0 | 44656 | 45344 | 6698.4 / 11336.0 | 1118 | 1042 |
| paste/1024/0 | 57760 | 59552 | 8664.0 / 14888.0 | 195 | 1956 |
| history/1024/0 | 48 | 64 | 32.0 / 32.0 | 1 | 0 |
| replace/1024/0 | 928 | 1040 | 139.2 / 260.0 | 16 | 139 |
| menu-show/1024/0 | 24752 | 25168 | 3712.8 / 6292.0 | 161 | 1312 |
| menu-next/1024/0 | 24368 | 24736 | 3655.2 / 6184.0 | 132 | 105 |
| menu-accept/1024/0 | 1360 | 1440 | 204.0 / 360.0 | 17 | 211 |
| validation/1024/0 | 23872 | 24304 | 3580.8 / 6076.0 | 123 | 1235 |
| incomplete/1024/0 | 23616 | 23936 | 3542.4 / 5984.0 | 115 | 1182 |
| analysis/1024/0 | 23488 | 24016 | 3523.2 / 6004.0 | 102 | 182 |
| output/1024/0 | 23728 | 23936 | 3559.2 / 5984.0 | 120 | 1228 |
| resize/1024/0 | 23040 | 23520 | 3456.0 / 5880.0 | 86 | 572 |
| edit/640/10 | 15600 | 15840 | 2340.0 / 3960.0 | 73 | 1 |
| vertical/640/10 | 15600 | 15888 | 2340.0 / 3972.0 | 70 | 9 |
| resize/640/10 | 16112 | 16384 | 2416.8 / 4096.0 | 85 | 465 |
| edit/6400/100 | 140240 | 142432 | 21036.0 / 35608.0 | 151 | 1 |
| vertical/6400/100 | 140320 | 141904 | 21048.0 / 35476.0 | 149 | 68 |
| resize/6400/100 | 140128 | 141664 | 21019.2 / 35416.0 | 86 | 573 |
| edit/64000/1000 | 1381825 | 1393153 | 207273.8 / 348288.2 | 151 | 1 |
| vertical/64000/1000 | 1366913 | 1373088 | 205036.9 / 343272.0 | 149 | 68 |
| resize/64000/1000 | 1375905 | 1382625 | 206385.8 / 345656.2 | 86 | 573 |
| edit/65536/0 | 13328 | 23968 | 1999.2 / 5992.0 | 5 | 1 |
| cursor/65536/0 | 48 | 48 | 32.0 / 32.0 | 0 | 0 |
| analysis/65536/0 | 1323600 | 1329185 | 198540.0 / 332296.2 | 159 | 37 |
| resize/65536/0 | 1312177 | 1316721 | 196826.5 / 329180.2 | 86 | 654 |
| edit/1048576/0 | 221552 | 251872 | 33232.8 / 62968.0 | 5 | 1 |
| cursor/1048576/0 | 80 | 112 | 32.0 / 32.0 | 0 | 0 |
| analysis/1048576/0 | 21149211 | 21183499 | 3172381.6 / 5295874.8 | 159 | 37 |
| resize/1048576/0 | 20986987 | 21031067 | 3148048.0 / 5257766.8 | 86 | 654 |

The exact internal allocation and terminal-byte oracles are regression controls,
not public ANSI compatibility. Warmed append/cursor allocate zero; cold growth
is measured separately. The 1000-byte workload includes the intermediate redraw
and final submission effects (H009). Prefix-layout traversal on large multiline
drafts and synchronous host-output blocking remain named debt. This policy is
bounded to recorded workloads, not a universal latency SLA.

## Reproduction and CI ownership

Normal CI runs harness guards, minimized regressions, bounded seed/model smoke
and deterministic allocation/byte checks. Manual campaigns own full CPU budgets,
native counted stress, final corpus replay and controlled latency qualification.
Use fresh evidence directories; commands fail nonzero on violated invariants.

```sh
python3 tools/hardening/test_hardening.py
python3 tools/hardening/smoke.py
python3 tools/hardening/deterministic.py
python3 tools/hardening/campaign.py --work /tmp/replai-fuzz --cpu-seconds 3600 --workers 3
python3 tools/hardening/corpus.py pack --campaign /tmp/replai-fuzz --output /tmp/replai-corpus.json.gz
python3 tools/hardening/corpus.py replay --archive tools/hardening/evidence/final-corpus.json.gz --work /tmp/replai-corpus-replay
python3 tools/hardening/native.py --work /tmp/replai-native
python3 tools/hardening/native.py --memory --work /tmp/replai-native-memory
```

Linux campaigns need nightly with cargo-fuzz; native memory runs require Valgrind
on Linux or `/usr/bin/leaks` on macOS. The [native workflow](../../.github/workflows/hardening.yml)
asserts native architecture. The [corpus workflow](../../.github/workflows/hardening-replay.yml)
replays the immutable archive on all applicable targets. Windows runs portable
oracles only. No adjacent repository or private consumer is needed.

```sh
cargo build --locked --release --manifest-path tools/hardening/Cargo.toml --bin bench
cargo build --locked --release --manifest-path tools/hardening/Cargo.toml --bin bench --features allocations --target-dir tools/hardening/target/allocations
python3 tools/hardening/regression.py register --work /tmp/replai-control --binary tools/hardening/target/release/bench --allocation-binary tools/hardening/target/allocations/release/bench
python3 tools/hardening/regression.py compare --work /tmp/replai-candidate --registration /tmp/replai-control/registered.json --control-binary /tmp/replai-control/binary --binary /tmp/replai-control/binary --allocation-binary /tmp/replai-control/allocation_binary
python3 tools/qualify.py --work /tmp/replai-full-qualification
```

The example compares a frozen unchanged binary. For a runtime repair, preserve
separate qualified control/candidate identities and investigate reproduced
exceedances before accepting a new baseline. Full qualification retains C ABI 1
symbols/layouts, static/shared/C++ consumers, native PTYs and memory checks.
Release packaging, API freeze, public distribution and consumer migrations remain
separate authorizations; no deferred runtime feature is added here.
