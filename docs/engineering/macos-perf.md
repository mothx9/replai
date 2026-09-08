# macOS runtime and common performance closure

This dossier owns evidence for MACOS.RUNTIME.PERFORMANCE.CLOSURE.0.
[ROADMAP](../../ROADMAP.md) alone owns current status and authorization.
The wave first qualifies a native macOS checkpoint without changing editor,
layout, render or decoding algorithms, records its native P0 baseline, then
redesigns the common implementation and measures same-machine convergence.

## Starting identity and scope

Canonical `mothx9/replai` master was clean and equal to origin at
`8210c93d484d50c74d3e02d70e881c3e8d8a1c56`, tree
`9045f787191f4e859673b961eb4a69ecf9a70615`.
[P0](p0.md) and its results remain historical evidence, not a native macOS
comparison baseline. Neither consumer repository nor its external pin is changed.
Public F1/F2, public P3 driving, I/O/U work and Windows runtime remain outside
this wave. The neutral [demo](../../examples/demo.rs) remains the manual host.

## POSIX checkpoint architecture and dependency review

`system.rs` stays one resource implementation. Both operating systems share
TTY checks (`isatty`, matching device identity), descriptor duplication,
termios capture/raw/restore, terminal dimensions, read/write, failure cleanup,
exclusive lease and protocol lifecycle. Readiness is the actual OS difference:
Linux retains rustix poll; macOS uses one owned kqueue/read registration. The
Darwin `/dev/tty` alias rejects that registration, so it uses safe select on the
owned descriptor instead. This fallback explicitly rejects descriptors outside
FD_SETSIZE and restores the terminal on failure. Registration happens after raw
mode: native queued-input tests exposed a missed readiness notification when a
canonical registration survived the ICANON change. No
signals or background threads are installed. Editor, engine, decoder, layout,
renderer, VT encoding and output coordination are unchanged at the checkpoint.

[Rustix 1.1.4 poll documentation](https://docs.rs/rustix/1.1.4/rustix/event/fn.poll.html)
and [Apple poll contract](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/poll.2.html)
identify Darwin device limitations. Rustix select/kevent require unsafe caller
code. The core's unsafe prohibition is preserved by using registry
`nix = 0.31.3`, default features disabled, event and poll only, on macOS. Its MIT license,
`Kqueue(OwnedFd)`, safe `kevent` interface and cfg-only build script were reviewed
from the downloaded exact crate. User data is zero, no user pointer is supplied,
the registration refers to our owned input descriptor, and the queue is closed
with the resource. The queue receives CLOEXEC explicitly. No nix/rustix type
enters the public Rust API. Existing rustix supplies the remaining POSIX work.
The lockfile records registry checksums and transitive features/dependencies.

PTY creation differs only in tests: Linux opens the peer with TIOCGPTPEER;
Darwin uses grant/unlock/ptsname/open through safe rustix calls. The same Rust
PTY assertions execute on both. The C descriptor ABI and generated declarations
remain version 1. Darwin staging sets a relative `@rpath` dylib identity and
platform link flags; header/layout/ownership semantics do not change.

C qualification retains Linux Landlock/Memcheck. Darwin uses a runtime-generated
sandbox policy denying repository/Cargo-source reads, actual static/dylib and
C++ consumers, `nm`/`otool` plus actual dyld resolution, and native `leaks --atExit`.
The native leak tool is not called Valgrind or a complete invalid-access oracle.
Runtime-generated SDK/prefix paths are never committed. Windows only executes
the portable core and benchmark-integrity lanes.

## Reproduction and identity

Use the shared commands on the current platform, from the canonical checkout:

```sh
python3 tools/qualify.py --work /tmp/replai-native-qualification
cargo build --locked --release --example demo
python3 tools/perf/run.py --prepare --work /tmp/replai-native-performance \
  --qualification /tmp/replai-native-qualification
P0_MACHINE='physical host description and constraints' \
  python3 tools/perf/run.py --work /tmp/replai-native-performance \
  --qualification /tmp/replai-native-qualification
```

The existing `tools/perf/linux.py` entry name is retained for compatibility;
it now drives real Linux or Darwin PTYs with the same endpoints and assertions.
Missing Linux strace on Darwin is an explicit unavailable record, never zero
syscalls. Preparation downloads/builds before timing. The pinned competitor
identities and primary 1000-byte ASCII/Left/insertion/submission workload stay
unchanged. Reedline's receipt descriptor path uses `/dev/fd`, valid on both
systems, rather than Linux-only procfs. A separate `/dev/tty` case proves the
controlling-terminal readiness path and caller-FD closure.

Two native Darwin observations require transparent, common harness changes.
First, session-leader exit revokes the PTY; attributes cannot be inspected
afterward. Every reference host and REPLAI now acknowledges completed teardown
through an inherited control pipe before exiting. The parent checks restoration,
releases this exit gate, and drains all terminal output. The gate is outside the
original timed submission/last-output endpoints and applies on both platforms;
Linux additionally retains its post-exit attribute check. No rendering or
submission work is moved after the timing receipt.

Second, Darwin adds the transient line-discipline PENDIN state during a raw to
canonical transition. A non-consuming FIONREAD query settles pending input
before the initial and final full attribute comparisons for every library.
No attribute fields are masked. A native Rust PTY regression queues input in
raw mode, closes, compares all settled attributes, reopens, and verifies exact
submission of those queued bytes. Original correctness suites retain their
existing exact comparisons. Neither correction changes the primary workload,
competitor settings, timer, submitted bytes or complete output accounting.

The enqueue-only transport microbenchmark has no concurrent reader. Native
sampling exposed a blocked 4 KiB write because Darwin's PTY queue filled before
verification could drain it. A separate nonblocking capacity probe now runs on
both platforms before that family; sizes that cannot fit are explicit
unsupported records. End-to-end/output families retain their concurrent parent
drain and measure actual backpressure. This is a fixture bound, not a library
output-size limit.

The physical host is MacBook Pro Mac17,9, Apple M5 Pro (15 cores), 24 GB RAM,
macOS 26.6.2 build 25G83. Native baseline toolchain: Rust 1.96.0
`ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`, aarch64-apple-darwin, LLVM 22.1.2;
Apple clang 21.0.0. No VM is used for native measurements. Frequency and host
scheduler are uncontrolled; every run retains environment and variance.

Checkpoint/final implementation identities are recorded after commits. A
commit cannot contain its own hash in its tree: exact publication HEAD/TREE
are reported in the final receipt; recorded benchmark envelopes identify each
measured checkpoint, and `git log -1 --format='%H %T' -- src tools/perf` recovers
the last implementation/harness change independently of documentation closure.

## Qualified native checkpoint and baseline

The backend checkpoint is `dbd9562b94fa654b5931c5c06d0cc39287ea9cbf`, tree
`1b22125ccab89e67a165cca3f7de4b4196457270`. Subsequent fixture preparation and
test-only oracles produce the measured native checkpoint
`e058e080c2edbac6360ecc78145476ca7dd23b4c`, tree
`ce62960bd15bbd4ade53e9c45e9cac4302174676`. Its production editor, decoder,
layout, renderer and encoding algorithms are still P0's.

`tools/qualify.py` passed on the physical Mac with the clean-worktree gate.
This includes native debug/release Rust PTYs, failure/unwind restoration,
static/dylib C01–C15 scenarios, C11/C++17, ownership/layout/export checks,
staged-loader isolation, and zero leaks in both native C contract runs.
[Checkpoint CI 34226086836](https://github.com/mothx9/replai/actions/runs/34226086836)
passed all ten jobs: Linux Rust/PTY/C/Memcheck, macOS real Rust/C terminals,
Windows portable core, all three benchmark-integrity lanes and documentation.
An earlier macOS foundation failure was an offline missing Linux dependency;
fetching the complete locked target graph before that audit fixed preparation.

[The native pre-optimization selection](macos-perf/before.json) contains 3,336
representative summaries from 5,652 validated full-run records. Full raw timing
and allocation JSONL stays in the runner's work directory. Selection is
reproducible with [compact.py](../../tools/perf/compact.py):

```sh
python3 tools/perf/compact.py /tmp/replai-native-performance/baseline.json \
  docs/engineering/macos-perf/before.json
```

The authoritative run used verified AC power, default power mode 0, no local
compilation during timed families, and clean checkpoint sources. Power was AC
at both recorded endpoints. Earlier partial battery runs were retained only as
diagnostics and excluded after the power source changed. Raw Mach-O `size`
listings include virtual segments; file lengths, not those virtual totals, are
the binary-size measurements. Compact evidence removes machine installation
paths, transient battery IDs and verbose archive-member listings.

| Primary 1000-byte ASCII burst + Left + X + exact submission | Median ms | p95 ms | Terminal bytes, median |
| --- | ---: | ---: | ---: |
| REPLAI | 18.395 | 20.189 | 17,741 |
| linenoise blocking | 5.951 | 6.622 | 569,892 |
| linenoise feed | 6.131 | 6.541 | 569,892 |
| rustyline | 0.806 | 0.835 | 9,096 |
| reedline | 0.457 | 0.480 | 1,045 |

All rows have 31 measured invocations after two warmups, exact submission and
restoration, and complete output draining under the P0 contract. Pins are
unchanged from P0. The native pre-optimization parity band is
`1.25 × 456.5 µs = 570.625 µs`; final convergence must use final same-run
references instead of treating this number as a permanent threshold.

The largest measured editor median is 9.133 ms for a substantial completion
replacement in a 1 MiB mixed draft. Layout reaches 33.304 ms for a 1 MiB
multiline draft. The 1 MiB multiline layout at width 20 makes 399,496 allocation
or reallocation requests, with 12,983,000 peak temporary live bytes, 20,048,300
requested bytes and 3,264 retained bytes. These quantities are distinct.
Native 1 MiB ASCII/prose paste medians are respectively 1,246.82/1,248.22 ms.

The frozen full-layout oracle and generated editor model were qualified before
optimization. They exercise extended-grapheme joins across both replacement
neighbors, long combining/RI/Prepend context, invalid byte offsets, bounded
rejection, and more than 65,000 generated edit operations. The old layout
exists only under the test boundary, not as another production implementation.

## Common redesign and causal checkpoints

The final algorithms are shared by Linux, macOS and the Windows portable core.
Only resource acquisition/readiness and platform qualification utilities have
OS branches. No editor/layout/render/decoder fork, unsafe core code, telemetry,
worker thread, asynchronous output queue or architecture-specific acceleration
was introduced. The native checkpoint's dependency graph is unchanged by the
optimization commits.

The editor keeps `String` and `VecDeque`. Measurements justified removing
unconditional prefix scans before choosing a more expensive storage/index
design. `GraphemeCursor` validates a boundary using its required Unicode context;
endpoints and UTF-8 boundaries are checked explicitly. There is no stale
grapheme index to invalidate. Generated edit/model tests cover both neighboring
joins, ZWJ, combining marks, variation selectors, regional indicators and
Prepend characters. String shifts and growth remain real costs; local boundary
validation does not make arbitrary Unicode context constant-time.

Layout keeps a rolling potential viewport while finding the cursor. Rows recycle
their run/text storage; after the cursor fixes the viewport, layout stops after
its visible suffix. Grapheme width is computed once per layout traversal.
Oversized offscreen row strings are not retained in the recycling pool. A
cursor near the end still requires scanning its prefix; a giant single grapheme
can itself require scanning substantial text. The width policy and frozen
full-document geometry oracle are unchanged.

The renderer retains row widths and source cursor/length metadata. Engine damage
classification permits geometry reuse only for unchanged-text cursor motion or
proven simple ASCII tail insertion/deletion within one physical row. Mixed
damage, changed dimensions, cross-row motion, tabs/newlines, ambiguous wrapping
or other text mutations fall back to fresh layout. Completion invalidates even
pending append damage. Unicode text mutations never borrow ASCII prefix rules.
Ready frames update changed rows, use ASCII common prefixes where valid, and
otherwise anchor with CR and replace the row. Changed geometry retains full
erase/redraw. Ctrl-L still clears the screen. Generated cached-frame equality
and independent VT screen tests cover more than 32,000 editing transitions,
with additional deferred/completion/output/resize invalidation cases.

The POSIX resource reads up to 4 KiB into driver stack storage. It never sets
O_NONBLOCK on the caller's shared open-file description. Full reads can drain
more immediately-ready input, with zero additional wait, up to 32 KiB per poll;
a 2 ms active-work budget is checked after semantic actions. This is a yield
budget, not a debounce or deadline interrupting an atomic operation. A short
read publishes its completed edits immediately. A returned event retains at
most one read of unconsumed bytes in the owning Interaction, including across
close/reopen. Destruction drops already-consumed read-ahead. This ownership
boundary is explicit in the Rust/C interaction documentation.

The engine applies semantic actions in order and marks presentation dirty.
Submission, interrupt, EOF, completion requests, rejection and explicit redraw
flush at their observable boundary. Submission rendering and terminal cleanup
remain before the timed submission receipt. External output remains synchronous
and restores the exact draft/cursor. Input is not concatenated indiscriminately:
sequential insertion around combining boundaries must retain its original
semantics. Paste still goes through bounded framing, UTF-8 validation, newline
normalization, atomic insertion and normal presentation.

The causal selections retain intermediate results, including an unfavorable
checkpoint, rather than presenting only the final improvement:

| Checkpoint | Primary burst median | Change isolated by the checkpoint |
| --- | ---: | --- |
| Native pre-optimization `e058e08` | 18.395 ms | Qualified POSIX Mac, original algorithms |
| [Editor `0046dea`](macos-perf/editor.json) | 15.923 ms | Local Unicode boundary context |
| [Layout `2fd7630`](macos-perf/layout.json) | 10.212 ms | Viewport storage/traversal |
| [Ready input `c486666`](macos-perf/buffered.json) | 0.186 ms | Bounded reads and semantic/presentation separation |
| [Renderer `b89e594`](macos-perf/probe-delay.json) | 0.185 ms | Incremental rows and geometry reuse; isolated-key delay exposed |

The renderer checkpoint still exposed roughly 8–12 microseconds to several isolated
keys because dirty output waited behind a zero-time readiness probe. This was
not accepted as closure. `ec4a3bc` publishes a drained short read before probing
another burst. Its [63-sample diagnostic](macos-perf/visible-flush.json) measured
ASCII append at 23.5 microseconds, Unicode at 24.7, and middle insertion at 26.2.
The final full suite, rather than this selected diagnostic, owns the verdict.
The input size, dimensions, submission, renderer output and comparison endpoints
are unchanged. More intermediate pictures from already-ready input are not
required; the final picture and every host-visible outcome still are.

[Driver counts before buffering](macos-perf/driver-before.json) use the added
`3af57e8` fixture with the unchanged `2fd7630` production artifact. They measure
1,005 poll calls for the primary burst and 1,048,589 for the 1 MiB multiline
paste plus submission. They are poll calls, not syscall counts. The final
structured driver rows retain the corresponding measurements after redesign.

`interaction_edit` adds final-only scaling coverage for complete semantic edit,
layout/render and VT encoding, with setup excluded. Its rows explicitly state
that no original P0 timing pair exists. Existing P0 families remain intact.
`--predict` now uses the same deferred engine/flush boundaries for a ready input
burst; the exact-byte PTY oracle still checks the resulting output. Comparison
timing and receipt/drain rules are unchanged. Final comparison rows additionally
record exact library revisions and executable hashes, and preparation verifies
those hashes before timing. This is provenance, not a competitor setting change.

## Final recorded implementation and measurement conditions

Final measured implementation/harness HEAD: `96f794c7b807636b4fabd3939bbeeab150a87dd1`; TREE: `66ccbecfc7acad17af4c370f35ddf5d7880e3c67`. The worktree was clean at measurement. The final documentation publication is identified separately by the receipt and Git history as explained above.

[The final native selection](macos-perf/after.json) retains 3,605 summaries from 5,921 validated records. Full timing/allocation JSONL and verbose traces stay outside Git in the chosen work directory. The same prepare/run/compact commands above reproduce every family on Linux or macOS. The schema remains version 1; additional comparison revision/hash fields are backwards-compatible metadata.

The pre-optimization run used AC power. The final run recorded battery power at both endpoints (70% to 65%), default power policy unchanged. The hardware, macOS and Rust identities match. This limits attribution of small before/after timing changes; the primary parity comparison uses only references within the final same-machine run. Intermediate AC checkpoints also establish the large pipeline improvements. No compilation or dependency download occurred during timed families. Governor/frequency, scheduler placement and normal desktop background activity remain uncontrolled; this is descriptive physical-host evidence, not a regression-grade isolated lab.

Microbenchmarks retain 63 samples (three groups of 21), two warmups per group, and separate timing/allocation executables. PTY/comparison distributions retain 31 samples after two warmups. No valid samples are removed; p95/p99 use nearest rank. Values near the timer floor are reported here as below 0.1 microsecond, not as supported nanosecond precision. Raw serialized units and group medians remain available.

## Final same-run comparisons and parity

| Primary 1000-byte ASCII + Left + X + submission | Median ms | p95 ms | p99 ms | Terminal bytes |
| --- | ---: | ---: | ---: | ---: |
| replai | 0.194 | 0.204 | 0.210 | 1,054 |
| linenoise-blocking | 5.869 | 6.321 | 6.720 | 569,892 |
| linenoise-feed | 5.899 | 6.222 | 6.258 | 569,892 |
| rustyline | 0.825 | 0.842 | 0.846 | 9,096 |
| reedline | 0.460 | 0.488 | 0.489 | 1,045 |

Primary target: **194.4 µs ≤ 574.5 µs**, where the threshold is 1.25 times the fastest final reference. This passes. REPLAI's median changes from 18.395 to 0.194 ms, and terminal traffic from 17,741 to 1,054 bytes. All scored rows assert exact submitted bytes, full termios restoration and complete PTY output draining. No global winner claim follows from this one burst.

Other directly overlapping workloads, medians in milliseconds:

| Workload | REPLAI | linenoise blocking | linenoise feed | rustyline | reedline |
| --- | ---: | ---: | ---: | ---: | ---: |
| short_ascii | 0.037 | 0.055 | 0.049 | 0.053 | 0.062 |
| long_ascii_4096 | 0.644 | 113.321 | 113.072 | 4.822 | 1.630 |
| unicode | 0.044 | 0.053 | 0.053 | 0.053 | 0.066 |
| combining | 0.038 | 0.044 | 0.045 | 0.047 | 0.062 |
| history | 0.041 | 0.037 | 0.038 | 0.042 | 0.060 |
| completion | 0.044 | 0.044 | 0.041 | 0.046 | 0.079 |
| multiline | 0.039 | 0.048 | 0.045 | 0.041 | 0.061 |

All five fixtures remain pinned to the original identities: linenoise `a473823d74b93eab2ba83480df16ed37617493f2`, rustyline 18.0.0 `1d1aa2f43a8f08092bc925c0cac4afc2a2772b11`, reedline 0.51.0 `3eb99426d4399b18b11a2cf4b5b5fff8c1b404e9`. Revisions and executable hashes are in each final row. Prompt/readiness and per-library output behavior remain visible in the byte totals; this is a tradeoff matrix, not feature-normalized ranking.

Recalculate parity and paired observations without rerunning a timer:

```sh
python3 tools/perf/convergence.py docs/engineering/macos-perf/before.json \
  docs/engineering/macos-perf/after.json > /tmp/replai-convergence.json
```

The derivation rejects mismatched primary dimensions/input size or missing correctness/restoration evidence. CI tests that logic, not live timing thresholds.

## Isolated key-to-visible behavior

| PTY workload | Before median µs | Final median µs | Final p95 µs | Final p99 µs | VT bytes before → final |
| --- | ---: | ---: | ---: | ---: | ---: |
| ascii_append | 26.2 | 22.5 | 25.5 | 26.5 | 1 → 1 |
| unicode_append | 26.7 | 24.3 | 27.8 | 28.3 | 4 → 4 |
| middle_insert | 27.7 | 26.2 | 29.5 | 30.1 | 79 → 38 |
| cursor_movement | 25.6 | 23.5 | 30.1 | 31.9 | 22 → 4 |
| history | 25.0 | 24.5 | 28.2 | 28.4 | 23 → 18 |
| completion | 26.6 | 24.6 | 30.8 | 32.7 | 8 → 8 |
| ctrl_l | 23.2 | 23.4 | 26.8 | 31.5 | 20 → 20 |
| middle_edit | 223.0 | 32.2 | 37.4 | 38.1 | 2653 → 43 |
| long_ascii_append | 155.8 | 24.2 | 28.2 | 28.2 | 1 → 1 |
| resize | 101,040.4 | 100,934.9 | 101,134.5 | 101,215.6 | 158 → 158 |

Ordinary medians are preserved or improved; the 0.2-microsecond Ctrl-L median difference is below the observed spread. There is no collection delay. Resize without input still encounters the 100 ms polling cap. The isolated typing claim does not imply that arbitrary long Unicode re-layout is equally cheap.

## Component scaling, allocations and remaining work

| Operation | Before median µs | Final median µs |
| --- | ---: | ---: |
| Middle insert ASCII 64 KiB | 699.3 | 0.5 |
| Large completion mixed 1 MiB | 9,132.6 | 1.4 |
| Backspace end ASCII 1 MiB | 7,548.4 | <0.1 |
| Layout begin multiline 1 MiB, 20 columns | 33,047.3 | 6.4 |
| Layout middle multiline 1 MiB, 240 columns | 33,303.9 | 8,739.6 |
| Layout end multiline 1 MiB, 240 columns | 33,274.9 | 17,958.5 |
| Layout end ASCII 1 MiB, 80 columns | 26,165.7 | 13,825.0 |
| Right across one combining grapheme, 1 MiB | 2,823.5 | 3,137.7 |

The largest final ordinary editor median is 470.1 microseconds (history admission of a substantial Unicode entry). The adversarial 1 MiB combining grapheme still costs milliseconds to traverse. Neither result is hidden by a global editor score.

| ASCII middle insertion scale | Before µs | Final µs |
| --- | ---: | ---: |
| 64 bytes | 0.8 | <0.1 |
| 4,096 bytes | 44.5 | 0.1 |
| 65,536 bytes | 699.3 | 0.5 |

String insertion/deletion still shifts the suffix; growth is amortized, not worst-case constant. Boundary work depends on relevant grapheme context. Full layout is proportional to scanned prefix plus visible suffix, except that one grapheme can itself be large. Retained layout storage is viewport-oriented; changed-row rendering scans visible content. Homogeneous cursor/ASCII-tail damage can reuse geometry; mixed or ambiguous damage rebuilds. History navigation still clones the selected entry and unsent draft; no history index or persistence was added.

Final-only full engine/VT `cursor_left` on a 1 MiB ASCII draft: 2.2 µs, 4 encoded bytes, 1 mutations, one logical transport write. This excludes OS transport and setup.

Final-only full engine/VT `backspace_end` on a 1 MiB ASCII draft: 2.1 µs, 7 encoded bytes, 2 mutations, one logical transport write. This excludes OS transport and setup.

| Ready-frame ASCII 64-byte transition | Before µs / bytes | Final µs / bytes |
| --- | ---: | ---: |
| append | <0.1 / 1 | <0.1 / 1 |
| cursor_left | 0.2 / 77 | <0.1 / 4 |
| middle_insert | 0.2 / 79 | 0.1 / 38 |
| middle_delete | 0.2 / 77 | <0.1 / 39 |
| history_replace | 0.2 / 25 | <0.1 / 24 |
| resize | 0.2 / 79 | 0.2 / 79 |
| full_redraw | 0.2 / 78 | 0.2 / 78 |

Logical mutations, encoded bytes, logical output operations and actual syscalls remain separate. Native Mac strace/syscall observations are explicitly unavailable. Actual write counts are not inferred from mutation counts. The Linux tracing path remains runnable after a pull; CI is correctness/integrity evidence, not authoritative Linux performance.

| 1 MiB multiline layout, cursor beginning, 20 columns | Before | Final |
| --- | ---: | ---: |
| allocation_calls | 399,496 | 85 |
| requested_bytes | 20,048,300 | 7,176 |
| peak_live_delta_bytes | 12,983,000 | 5,240 |
| retained_delta_bytes | 3,264 | 3,448 |

These are benchmark-process System allocation/reallocation requests and incremental live-byte observations. Peak includes the returned frame; retained delta is reported separately. Allocator metadata, internal realloc overlap and macOS RSS are not inferred. Editor in-object size stays 128 bytes; Interaction grows from 432 to 560 bytes for frame metadata and bounded read-ahead ownership. A poll uses a 4 KiB stack read buffer. Retained read-ahead is bounded to one read. History growth at 100,000 entries of 64 bytes remains 9,545,736 allocated live bytes.

## Paste, idle and synchronous host output

| ASCII bracketed paste | Before PTY median ms | Final PTY median ms |
| --- | ---: | ---: |
| 1,024 bytes | 0.813 | 0.128 |
| 4,096 bytes | 4.431 | 0.361 |
| 16,384 bytes | 18.991 | 1.294 |
| 65,536 bytes | 77.364 | 4.987 |
| 262,144 bytes | 310.828 | 19.774 |
| 1,048,576 bytes | 1246.825 | 79.185 |

Final isolated 1 MiB ASCII paste stages (different timers; they are not an exclusive-time decomposition):

- `paste_decoder`: 4.987 ms.
- `paste_normalization`: 0.781 ms.
- `paste_editor`: 0.412 ms.
- `paste_layout_render`: 13.191 ms.

Final-delimiter normalization is already included in full decoder framing time. The PTY total additionally includes kernel transfer/backpressure, production read/poll coordination and two-process scheduling. The roughly 60 ms gap over isolated decoder/editor/layout work is not an exclusive measured kernel time. It remains an identified pipeline/transport investigation limit. Atomicity, configured capacity, normalization and exact submission remain asserted.

| Production poll-call counts including submission | Before buffering | Final |
| --- | ---: | ---: |
| short_ascii | [8, 8, 8] | [1, 1, 1] |
| unicode | [14, 14, 14] | [1, 1, 1] |
| long_ascii | [1005, 1005, 1005] | [1, 1, 1] |
| paste_multiline_65536 | [65549, 65549, 65549] | [65, 65, 65] |
| paste_multiline_1048576 | [1048589, 1048589, 1048589] | [1027, 1027, 1027] |

Publishing drained short reads increases large-paste poll calls relative to the intermediate deep-drain driver (34–38 for the 1 MiB diagnostic), while removing the isolated-key probe delay. The final count is still about three orders below byte-per-poll P0. This is an explicit latency/driver-work tradeoff; no paste validation is bypassed.

| Idle timeout | Before polls/s | Final polls/s | Before lifecycle CPU ms/s | Final lifecycle CPU ms/s |
| --- | ---: | ---: | ---: | ---: |
| 0 ms | 76709.7 | 74442.9 | 161.61 | 182.59 |
| 1 ms | 797.5 | 882.1 | 7.58 | 8.66 |
| 10 ms | 86.7 | 92.1 | 3.02 | 3.59 |
| 100 ms | 9.9 | 9.9 | 2.00 | 2.17 |
| 1000 ms | 9.9 | 9.9 | 2.07 | 1.71 |

Each production poll requests dimensions once; the measured counts above are driver calls, not independently traced Darwin ioctl/wakeup counts. CPU includes process/open/close cost over the observation window. A normal 100 ms idle wait remains about ten checks per second. Zero timeout is intentionally busy polling. Public P3 still owns host event-loop embedding and resize strategy.

| Synchronous output at 200 chunks/s | Before host-call µs | Final host-call µs | Final p95 µs | Final complete-run VT bytes |
| --- | ---: | ---: | ---: | ---: |
| 80 bytes/chunk | 18.1 | 28.9 | 44.0 | 53,616 |
| 1,024 bytes/chunk | 55.5 | 76.5 | 107.1 | 453,616 |
| 4,096 bytes/chunk | 129.9 | 179.8 | 253.5 | 1,754,416 |
| 16,384 bytes/chunk | 437.2 | 446.7 | 601.8 | 6,958,816 |

The full evidence retains all 5/20/50/100/200 rates and small/large chunk classes, CPU observations and missed-deadline counts. Traffic and exact draft/cursor restoration are preserved. Small-chunk host-call timing is not uniformly better; power-state/scheduling differences limit causal attribution of that variance. Rate sleeps remain outside the host-call timer. No concurrent editing, worker queue or O2 semantics were introduced.

## Embedding and resource cost

| Artifact | Before unstripped / stripped bytes | Final unstripped / stripped bytes |
| --- | ---: | ---: |
| rust_example | 679,256 / 496,784 | 700,104 / 513,344 |
| c_static_consumer | 1,986,496 / 1,345,568 | 2,012,176 / 1,362,160 |
| c_shared_consumer | 35,088 / 35,008 | 35,088 / 35,008 |
| dynamic_library | 600,208 / 436,008 | 620,848 / 452,552 |
| static_archive | 20,424,400 / 12,340,624 | 20,628,984 / 12,374,024 |

The static archive is not a linked consumer. Shared embedding includes both the consumer and dylib; stripped final combined size is 487,560 bytes. The additional algorithm/driver code has a measurable binary cost; no size optimization is claimed. All dependency revisions and feature graphs are retained in the environment and lockfiles.

## Qualification, practical inspection and closure boundary

The final measured candidate passed `tools/qualify.py` with a clean worktree on the physical Mac: Rust debug/release/property/real PTYs, failure/unwind/restoration, C11/C++17, static/dylib ABI 1, exact record layout/symbol/status/buffer rules, opaque/descriptor ownership, staged loader isolation and repeated lifecycle. Native `leaks --atExit` reports zero leaks for both C contract consumers. Linux Memcheck remains the invalid-access/leak correctness oracle; native leaks is not claimed to provide equivalent invalid-access coverage.

[Implementation CI 34235936504](https://github.com/mothx9/replai/actions/runs/34235936504) passed all ten jobs at the measured revision: full Linux Rust/PTY/C/Valgrind, real macOS Rust/C terminals, Windows portable engine, portable benchmark integrity and documentation. Final documentation publication runs the same gates again; its exact CI receipt is recorded with closure. No hosted timing threshold is introduced.

The neutral release `demo` was exercised through a native Darwin interactive PTY: normal/rapid typing, Unicode and middle edit, recalled history, `wor` completion to `world`, CR/LF-normalized multiline paste, an exact 17,200-byte paste, interrupt, reopen, submission and EOF. Automated real PTYs additionally qualify resize, terminal styling/default background, descriptor stability and failure paths. Ghostty UI control was rejected by the computer-use tool for safety reasons; an emulator visual/resize smoke is therefore not claimed. This limitation does not substitute a virtual transport for the authoritative native PTY suite.

X1, P1 and P2 close together on this evidence. String storage remains the measured choice; cursor-local geometry reuse and changed-row rendering satisfy the common performance objective without an OS fork. Remaining costs include long-prefix layout, adversarial grapheme context, large-paste transport coordination, periodic resize latency, modest binary/object growth, and scheduler-dependent output-call variance. F1 must account for bounded read-ahead ownership and observable flush/event boundaries, but no new public API tier is implemented. Public P3 embedding, F1/F2, I/O/U waves, O2, Windows runtime and public release remain unstarted. Neither consumer nor its externally owned pin changed. No next wave is authorized.
