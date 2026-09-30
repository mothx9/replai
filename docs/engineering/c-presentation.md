# Native C presentation extension qualification

## Scope and exact source

`INTERFACES.CLI.REPLAI.PRODUCT.SURFACE.REFOUNDATION.0` establishes optional
presentation extension 1 alongside unchanged C ABI 1. Qualified implementation:
`19845f5ae24fc0b9589a2621d03a8d1ba1ad47c2`, tree
`1e012f04a871bf65c9f05bfb16659419d000eac6`, Rust runtime tree
`7b62ec3136df94a9f906307d2e1e9276ca19530a`.

The [C contract](../c-api.md#presentation-extension-1) owns layouts, bounds and
failure semantics. The extension copies semantic documents/prompts/candidates,
uses the existing safe Rust implementation, and exposes opaque revision/deadline
tickets and a borrowed readiness source. A quiet-output scope shares the existing
exclusive terminal lease, owns replaceable feedback and exact mode restoration,
and leaves signal/cancellation meaning with the host. It is not concurrent writer
arbitration, a shell, a full-screen framework or a general streaming API.

## Native evidence

Linux aarch64, kernel `6.17.0-1021-nvidia`, Rust 1.98.1, Valgrind 3.22.0.
No model, GPU, application repository or private Rust type participates.

| Gate | Independent observation | Expected / observed |
| --- | --- | --- |
| `python3 tools/qualify.py --work <fresh-directory>` | Repository-native complete suite and clean-source gate | All 22 gates PASS, including unit/integration, docs, clippy, doctests, native PTY and memory |
| Installed C11/C++17, static/shared, pkg-config/CMake | Consumer reads isolated with Landlock; moved install prefix | PASS without repository/Cargo-source access; namespace and loaded-library identity checked |
| Existing ABI 1 contracts | C/Rust sizes, offsets, numeric values; 128 owners / 384 opens and closes | PASS; no change to existing records/functions; exact restoration and bounded FD use |
| Extension documents | Installed-header-only C consumer | 117 widths (4–120), plain/styled, responsive records/tables, right alignment, literals, Unicode and exact buffers PASS |
| Extension refusals | Same C consumer, malformed version/tag/control/bounds/row geometry | Fail closed; short buffer unchanged; stale candidate payload not read and no output emitted |
| Completion and driven notification | Real PTY plus independent VT cell oracle in Rust conformance | Menu/annotation/insertion separation, stale draft, fragmented sequence, resize, reopen and restoration PASS |
| Quiet feedback | Real PTYs, narrow/normal/wide, exclusive resource owner | Replacement/clear leaves no old feedback; echo suppressed with ISIG/canonical input retained; cleanup and next editor lifetime PASS |
| Memory | Valgrind, both installed link forms and existing PTY scenarios | Zero errors, zero leaked blocks; no suppressions |
| `python3 tools/qualify_packaging.py --source <exact-source> --work <fresh-directory> --full` | Git-derived crate/SDK, external Rust debugger and moved native C consumers | All 27 software phases PASS; distribution legal gate remains BLOCKED without an exact recipient legal package |

The optional extension exports 13 additional functions (29 total symbols), not a
replacement ABI. Windows remains portable-core-only; this observation does not
qualify a Windows terminal or claim new macOS native execution evidence.

## Latency meaning

Across 32 installed static/shared native PTY cycles, serialized ESC-preview
advance plus immediate drain was **0.012–0.139 ms**, and explicit resize advance
was **0.002–0.003 ms**. These are bounded local call observations, not end-to-end
host-notification latency or a terminal SLA. The host must actually drive resize;
ABI 1 polling remains unchanged.

ESC now provisionally hides the visible menu while retaining candidates and the
full fragmented-sequence deadline. Semantic bare-ESC resolution still waits for
the decoder deadline (approximately 250 ms in the existing compatibility test).
A completed fragmented sequence restores the menu and retains its original
action. Visual responsiveness is not a claim that protocol ambiguity vanished.

## Bounded accounting regression

The newly reachable C document path exposed Memcheck taint at the layout work
budget. Origin/disassembly traced it to vectorized nested sums loading inactive
storage in a private Rust enum, not to a C payload or allocator ownership escape.
This observation alone is not proof of a Rust-language memory-safety violation.
Accounting now uses checked, active-variant increments with immediate budget
refusal before output extension. Both installed link forms pass Memcheck with
unchanged rendering semantics and no suppression or disabled checks.

## Consumer and release limits

The producer is qualified independently before any application repin. Consumers
own candidate discovery, application semantics, reactor/signal integration and
the line-boundary/clear-before-write discipline for feedback. An immutable pin
must authenticate source/tree/archive and the separately queried extension.

Software packaging controls passing does not establish recipient distribution
readiness. No crates.io release, supported release tag, consumer capability or
expanded O2 multi-producer arbitration maturity is promoted here.
