# Embedding contract qualification

This dossier records bounded implementation and execution evidence for the
[interaction embedding contract](../interaction.md#embedding-tiers-and-wait-ownership).
[ROADMAP](../../ROADMAP.md) alone owns maturity and selection. Measurements here
are workload/environment observations, not generic latency promises.

## Source and ownership

Baseline master HEAD `dfd337e62624e3ea8324ce783c026e2e5737b6ef`, TREE
`3b612fa7e7557ccd1b479986ea1507cc2459c33d`. Implementation HEAD `349d75ebab5dc9dda0c1875d027f62a95bcdd2d9`, TREE
`0bc70ef10e295214c8df43a2fb21f09ad2914e5f`. Native fixture qualification at
`5c042778f453255411943119a35418c230b635e7` preserves those implementation sources;
later documentation/metadata carriers do not change the runtime. No consumer repository or pin is changed.

Before: the public POSIX façade offered open/poll/complete/output/interrupt/close;
private terminal polling combined geometry observation, waiting and advancement.
After: one retained `Interaction` adds blocking reads, explicit terminal
admission, pure wait interest, opaque deadlines, borrowed readiness and typed
advancement. Compatibility polling selects waiting policy over the same bounded
input loop. The editor, engine, decoder, render model and resource lifecycle
are shared, not copied. The engine remains free of clocks/FDs and protocol policy.

| Owner | Contract change |
| --- | --- |
| `driving.rs` | Portable wake, wait-interest, deadline and blocking-result vocabulary; no OS integer handles |
| `terminal.rs` | One advancement loop; session/input deadline validation, resize entry and separate compatibility waiting policy |
| `interaction.rs` | Retained blocking/session/driven façades and resource-borrow lifetimes |
| `capabilities.rs` | Minimal explicit facts versus feature policy; required cursor/erase; optional styling/paste |
| `system.rs` | Borrow existing owned input for registration; native resource/wait/restoration remain here |
| C binding | Maps the new native admission error to existing unsuitable status; ABI records/header/symbol contract unchanged |

`Error::CapabilityMismatch` is additive behavior but a source compatibility
change for exhaustive native error matches. The legacy open/poll/C TERM=dumb
contract is preserved; new simple/driven environment defaults refuse unknown
cursor/erase support before acquisition. This is not full F2 negotiation.

## Alternatives and bounded archaeology

Inspected the same exact implementations used by the qualified comparator hosts:

| Source | Revision | Relevant lesson |
| --- | --- | --- |
| [linenoise header](https://github.com/antirez/linenoise/blob/a473823d74b93eab2ba83480df16ed37617493f2/linenoise.h) | `a473823d…` | Blocking and EditStart/EditFeed/EditStop express different host scheduling over shared edit state; simple embedding need not erase a driven path |
| [rustyline entry](https://github.com/kkawakam/rustyline/blob/1d1aa2f43a8f08092bc925c0cac4afc2a2772b11/src/lib.rs) | `1d1aa2f…` | Retained editor plus blocking result is approachable; helper-based analysis is a separate boundary, not required for a tiny reader |
| [reedline engine](https://github.com/nushell/reedline/blob/3eb99426d4399b18b11a2cf4b5b5fff8c1b404e9/src/engine.rs) | `3eb99426…` | Conditional event polling and external-printer/idle paths show why wait ownership must be explicit when application events coexist |

Existing pinned fixture source under `tools/perf/comparisons` was inspected too.
No competitor dependency is imported into production. A callback framework
would invert application wait ownership; async-first or thread-owned readers
would impose an executor/lifecycle. A raw-byte host interface would export
protocol/resource responsibilities unnecessarily. A global reactor or Darwin
public driver would duplicate scheduling/platform authority. The selected
contract exposes why to wake, leaving how to wait to the host.

## Executable oracles

- Deterministic virtual transport: idle interest does no read/write/size query;
  input advancement supplies only zero waits; early/stale/replayed/cross-session
  deadlines are inert; queued continuation supersedes expiry; events preserve
  read-ahead; optional paste filters every mode transaction; failures restore.
- Real [driven host](../../examples/driven.rs): native POSIX wait on terminal and
  an independent Unix socket. Application output, resize, close/reopen and
  interruption are host decisions. No compatibility poll is used.
- [Blocking host](../../examples/simple.rs): no polling/readiness loop; exact
  Unicode result, original draft return, interrupt and EOF through the executable.
- [PTY runner](../../tools/embedding_pty.py): independent VT oracle, exact termios
  before session-leader exit, read-ahead/event/draft checks, repeated resource
  ownership, deadline scheduling and host-to-redraw timing. Admission refuses
  dumb/empty TERM and non-TTY without acquiring raw ownership.
- Existing session/Rust/C suites remain intact. A compile-fail rustdoc proves
  that an active borrowed readiness source cannot be used after close. Portable
  tests assert actual Send/Sync auto traits without implying concurrent writers.

Local initial Linux observation: external output preserves `é!界`, byte cursor 4
and visible column 10; history returns `draft` to byte cursor 4; completion turns
`he` into `hello`; bracketed paste yields `first\n界 second\nthird`, 22 bytes.
A host resize to 12 columns preserves the same draft. Thirty lifecycle repeats
restore exact termios/paste and retain 7 active process descriptors. Driven idle
has zero advancements/deadlines over the observer interval. Native macOS and Windows results below are executed CI evidence, independently
of this Linux observation.

## Qualification commands and measurement scope

```sh
python3 tools/qualify.py --work /tmp/replai-embedding-qualification
python3 tools/embedding_pty.py --work /tmp/replai-embedding
python3 tools/embedding_pty.py --memory --work /tmp/replai-embedding-memory
python3 tools/perf/run.py --prepare --families components,linux,driver --work /tmp/replai-embedding-after
python3 tools/perf/run.py --families components,linux,driver --work /tmp/replai-embedding-after
```

The complete before run used clean baseline source and separate fresh build/
measurement phases. After measurements verify the exact source and binary hashes
remain unchanged during the run. Timing and allocation modes are separate.
The new idle-interest component measures query cost and allocation count with
acquisition outside the measured region. Object sizes include retained driver
state; pending input remains at most a 4 KiB read tail. Full raw measurements
remain outside Git; a compact same-machine comparison is recorded at closure.

Idle observer CPU includes the host process, not solely library instructions;
zero required library wakeups is established independently by interest and
transport-call oracles. Host reactor blocking syscalls are not REPLAI periodic
polls. Resize timings include application notification and receipt IPC, not
pure render cost. A hosted runner timing is not a performance regression limit.

## Remaining architectural boundaries

The process-wide resource lease and compatibility 100 ms resize observation
remain deliberate backend constraints. Driven resize requires a host notification;
reactor registrations must not outlive borrowed descriptor ownership. O2 concurrent
writers, I0 revision-aware analysis, full F2 discovery, Windows runtime and a
portable driven C contract are not implemented by this work. No new wave begins.

## Native fixture corrections

The first native run exposed two observation defects, with existing macOS C
and portable library tests green. The new flag oracle initially compared
Darwin's kernel `FWASWRITTEN` marker as though it were a caller-selected mode.
[XNU fcntl definitions](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/fcntl.h)
identify `0x00010000` as write bookkeeping; the corrected oracle excludes only
that bit on macOS and still compares every other bit, including O_NONBLOCK.
The native memory runner also waited for process exit before draining its PTY/
pipe, which can block a verbose exit-time leak report. It now drains while
waiting and retains the zero-leak assertion. Neither correction changes the
terminal implementation or relaxes termios restoration.

The blocking-process fixture also waits for the next rendered prompt before
sending editing Ctrl-C/EOF. Waiting only for host echo output could inject Ctrl-C
during the intentionally restored interval between reads and signal the host
session leader. Prompt publication is the raw-editing acquisition barrier; this
keeps editing interruption distinct from host signal policy.

## Qualified observations

[Native matrix run 34353630259](https://github.com/mothx9/replai/actions/runs/34353630259)
passed all 12 jobs at `5c042778f453255411943119a35418c230b635e7`: full Linux
Rust/session/embedding/C static/shared/Valgrind, macOS Rust/embedding/C/native
leaks, Windows portable engine/rustdoc and all benchmark-integrity jobs.
The [compact measurement record](embedding.json) retains source hashes,
environments, observations and the matched comparison samples. Final publication
CI must also pass; its run remains discoverable from the exact published Git head.

| Property | Linux physical host | Native macOS runner |
| --- | --- | --- |
| Unicode, host output | `é!界`, byte cursor 4, visible column 10 before/after document | Same draft/cursor and semantic cells |
| History / completion | Original `draft`, cursor 4; `he` → `hello` | Same |
| Multiline / resize | `first\n界 second\nthird`, 22 bytes; 12-column reflow; Ctrl-L preserves it | Same |
| Deadline | One expiry/rejection; continuation before expiry removes old wake | Same |
| Idle | No deadline, no advances; 0.00 s process CPU at kernel accounting precision over 0.4 s | No deadline or advances; process CPU not measured |
| Syscall attribution | Between observer events: one host blocking wait; zero terminal reads, size queries or REPLAI periodic waits | Exact syscalls not measured; same virtual zero-I/O oracle and native reactor execution |
| Native FD observer | 8 active → 6 closed, stable over 30 repeats | 9 active → 6 closed, stable over 30 repeats |
| Cleanup | Exact termios before process exit, paste disabled at every close; drop/unwind across pending states | Same; native FD oracle distinguishes Darwin write bookkeeping from caller flags |
| Memory | Valgrind: 0 errors, 0 definite/indirect/possible lost bytes; 8,736 reachable bytes in two runtime blocks | `leaks --atExit`: 0 leaks / 0 leaked bytes; memory-instrumented FD observer 10 → 7, stable |

FD observer counts include its directory descriptor and fixture handles. The
parent's independent Linux `/proc` observation records 7 active descriptors.
These are counts at explicit lifecycle states, not a claim that all belong to
REPLAI. A second physical repetition exercised 30 simple process lifecycles too.

On Linux, ready notification → advancement return for the Unicode burst was
94.352 µs; host resize request → completed-write receipt was 215.696 µs.
The macOS observations were 81.125 µs and 441.333 µs respectively. These are
single debug-fixture characterizations, not cross-machine comparisons. Existing
compatibility resize observation remained around 100 ms; the driven entry has
no such discovery floor and waits for host notification instead.

### Performance preservation

Before/after full runs validate 6,069 / 6,071 result records. The two additions
are timing/allocation measurements for pure idle interest. Both runs completed
with their prepared source/binary hashes unchanged. The final measured runtime
inventory hash is `e4046310486a9fef0eba4d04a6c51b16562ea5a7edcc326492a8486683e5b682`.
The Linux environment is `spark-7c3d`, aarch64, kernel `6.17.0-1021-nvidia`,
20 logical CPUs. It is not the historical macOS comparison machine.

| Same-machine observation | Before | After | Endpoint / qualification |
| --- | ---: | ---: | --- |
| 1000 ASCII bytes + middle edit + submit | 353.265 µs | 277.216 µs | 31 retained samples each, alternating binaries; includes PTY scheduling, cleanup and submission receipt |
| Isolated append, matched follow-up | 137.344 µs | 138.336 µs | 63 retained samples each, alternating binaries; exact final VT bytes |
| Completion replacement, matched follow-up | 134.512 µs | 133.328 µs | Same original P0 case and exact-byte oracle |
| Multiline paste + submit | 148.064 µs | 148.912 µs | Alternating burst-to-submission fixture |
| Append frame transition, 64 ASCII bytes | 0.112 µs | 0.112 µs | Separate private component measurement |
| Middle-insert frame, 1024 ASCII bytes | 0.496 µs | 0.512 µs | Separate private component measurement |
| Compatibility idle | 30 calls / ~3 s | 30 calls / ~3 s | Periodic observation deliberately retained |
| Retained Interaction object | 584 bytes | 608 bytes | Linux stack representation; not an ABI layout promise |

The sequential full-run isolated append shifted from 33.792 to 242.880 µs.
That required investigation, not omission: repeating the exact case with
alternating old/new binaries gave 137.344 / 138.336 µs. PTY delivery includes
scheduler/environment variance; the sequential change does not establish a
library regression. Likewise the faster primary median is not a causal speedup
or general ranking. Exact output and restoration pass in both comparisons.

The 1000-byte workload remains one compatibility call and 1,054 terminal bytes;
64 KiB multiline paste remains 17 calls and 403 output bytes. The 1 MiB paste
retains bounded calls (before 257–263, after 258–259), 403 terminal bytes and exact
submission. Synchronous output and all existing component/large-paste cases
remain measured by the complete suite. No editor/render algorithm changed.

The new idle-interest query measured 0.032 µs and **zero allocations**; retained
Interaction construction also remains allocation-free. Deadline, WaitInterest
and Wake are each 32 bytes on this Linux target. Native macOS Interaction is
640 bytes, with its resource realization; its three scheduling values are each
32 bytes. Pending read-ahead remains bounded to one 4 KiB tail. None of these
representations is a public binary layout contract.

### Final observation and documentation carrier

[CI run 34355338986](https://github.com/mothx9/replai/actions/runs/34355338986)
passed all 12 jobs on `7f144d9e98a12bc1d3bbfc09bbcda547cecf2bd5`, tree
`8589809ac8898913af193525f884ff7edf07a204`. Full local
`python3 tools/qualify.py --work /tmp/replai-embedding-final-qualified` passed
on that clean revision, including C static/shared, Valgrind and release tests.
The runtime, binding, manifests and lockfile have no diff from the implementation
revision identified above; the README now includes a real terminal capture.

An intervening macOS run exposed an observer scheduling race: after acknowledging
an incomplete sequence, the Python host deliberately slept 150 ms inside a
250 ms protocol window. CI scheduling allowed the legitimate deadline to expire
before the continuation arrived (`abD!`, rather than `a!b`). The fixture now
sends the continuation immediately after acknowledgement; real expiry remains a
separate PTY case, and exact near-deadline/stale-token cases retain the virtual
clock oracle. This changes qualification timing, not decoder/runtime behavior.
The producer metadata records this watched-input change as qualification-only.
