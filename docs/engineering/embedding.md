# Embedding contract qualification

This dossier records bounded implementation and execution evidence for the
[interaction embedding contract](../interaction.md#embedding-tiers-and-wait-ownership).
[ROADMAP](../../ROADMAP.md) alone owns maturity and selection. Measurements here
are workload/environment observations, not generic latency promises.

## Source and ownership

Baseline master HEAD `dfd337e62624e3ea8324ce783c026e2e5737b6ef`, TREE
`3b612fa7e7557ccd1b479986ea1507cc2459c33d`. Implementation identity is the Git
commit carrying the source below; subsequent qualification/producer carriers
will identify it explicitly. No consumer repository or pin is changed.

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
has zero advancements/deadlines over the observer interval. Native macOS and
Windows matrix results must be recorded from the published CI run, not inferred
from this Linux execution.

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
