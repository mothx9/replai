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
