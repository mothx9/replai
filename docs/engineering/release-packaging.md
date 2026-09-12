# RELEASE.PACKAGING.0 evidence

This dossier records G3/G4 qualification for the unpublished `replai 0.1.0`
candidate. [ROADMAP](../../ROADMAP.md) owns maturity and selection;
[release scope](../release-scope.md) owns the v0.1 envelope. No crate, tag,
GitHub Release or binary SDK was published.

## Source and artifact identities

- Wave baseline: `3a0cb44ca17712712652dbbadc61f84c45e247c5`, tree
  `63ba9a716a6ef005c2120242875e358e56f5f0d8`.
- Qualified packaging source: `14cacba16f3cf2a00448d720502e1cfe975f2e83`,
  tree `1a49972a3fb9a623cf6bc3201281192f004ce102`.
- Production `src` tree before and after:
  `12b9cd0e58ef8d2dfe6c1b91185fd21b05a7aa9e`. Runtime behavior and
  the public Rust API did not change.
- C ABI source identity is unchanged: ABI number `1`, header SHA-256
  `c0644f2d541e7537bc5c96f3d70679b2f2f18f7b89af513de96bda2e7cb49bf5`,
  schema SHA-256
  `946189594ef5a57235c2d70580b1a95c6cb3fadecccf7b95f66b00c3f6cfab6b`.
- Exact native/portable packaging matrix: [GitHub Actions run 34703095088](https://github.com/mothx9/replai/actions/runs/34703095088).
- Exact-source general CI, including Linux Valgrind and macOS native leak gates:
  [GitHub Actions run 34703089136](https://github.com/mothx9/replai/actions/runs/34703089136).
  The evidence/documentation carrier may be newer; it does not rename these
  source-derived bytes.

| Artifact | Files | Uncompressed bytes | Archive bytes | SHA-256 |
| --- | ---: | ---: | ---: | --- |
| `replai-0.1.0.crate` | 91 | 1,674,801 | 984,730 | `7d2219f5055df7e392e00138a3b2b91f8c0787c2bd8ec75f9935288f45f1c705` |
| `replai-c-sdk-0.1.0.tar.gz` | 52 | 445,676 | 114,188 | `b3773ee821ed03c040a089e79f824a9a3cf0acda6f300366effcd7de4ad2a0c4` |

The SDK archive is a source distribution. It needs Rust/Cargo, Python, C/C++
tools and CMake to build the producer artifacts. Consumers of an installed
prefix need neither Rust nor a REPLAI checkout.

## Rust package

The root manifest now declares `replai 0.1.0`, edition 2024, MSRV 1.98.1,
MIT license, repository/readme/documentation links, five crates.io keywords,
two categories and `publish = ["crates-io"]`. The C adapter is also versioned
`0.1.0` and remains `publish = false`. This is publication readiness, not a
publication event.

The anchored Cargo include list retains library sources, selected examples,
consumer documentation, README/LICENSE/CHANGELOG, ABI records and the small
README assets referenced by packaged documentation. It excludes `.boundary`,
CI, packaging/readme generators, hardening corpus bulk, mutable benchmark
samples, repository targets and private release machinery. `cargo package
--list` is the inventory oracle; Cargo's normalized manifest and VCS identity
are included in the counts above.

Executed from the qualified source:

```sh
cargo package -p replai --locked
cargo package -p replai --locked --list
cargo publish -p replai --dry-run --locked
```

The dry run passed without upload. The extracted crate was tested with the
producer checkout outside every dependency path: locked fetch, all applicable
unit/integration/example tests, doctests and rustdoc all passed. A fresh
external Cargo project generated its own lock, then passed `check`, `build`,
`test`, deterministic portable execution and native real-PTY execution against
the extracted package. Its identities are:

- source SHA-256:
  `9e0333675a18301f9e1d018c827d790003e4a7e6e9fcbccd4848602405ab4213`;
- fresh `Cargo.lock` SHA-256:
  `f40b27e72ccd113f53c00275f157eb9eb0805097621f7b3432c229ed1a43ed29`;
- referenced package SHA-256: the `.crate` identity above.

Fresh resolution selected only dependencies compatible with Rust 1.98.1. The
repository lock and independently generated consumer lock both passed at the
floor. Current stable ran independently even though it resolved to the same
current release during this campaign.

### MSRV and documentation matrix

| Environment | Rust | Executed scope | Result |
| --- | --- | --- | --- |
| Ubuntu 24.04 x86_64 native | exact 1.98.1 and stable | package/unpack/docs, Rust consumer PTY, C SDK/install matrix | PASS / PASS |
| Ubuntu 24.04 ARM64 native | exact 1.98.1 and stable | package/unpack/docs, Rust consumer PTY, C SDK/install matrix | PASS / PASS |
| macOS 15 ARM64 native | exact 1.98.1 and stable | package/unpack/docs, Rust consumer PTY, C SDK/install matrix | PASS / PASS |
| Windows x86_64 MSVC portable | exact 1.98.1 | package/unpack, portable tests/consumer and rustdoc; no terminal backend | PASS |

The floor compiler is `rustc 1.98.1 (48a229cea 2026-09-01)`, Cargo
`1.98.1 (797e8a9bc 2026-08-05)`, LLVM 22.1.8. docs.rs metadata uses
`x86_64-unknown-linux-gnu` as default and requests ARM64 GNU Linux, ARM64 macOS
and Windows MSVC documentation. Local docs.rs-like rustdoc passed for all four;
Windows documentation retains target-gated absence of POSIX runtime methods.
Hosted docs.rs remains a publication-time observation.

The crates.io read-only API returned HTTP 404 for `replai` on 2026-09-12.
The name was therefore unregistered at the check, but publisher authority was
not inferred: credentials were neither inspected nor printed. Authentication,
ownership establishment and a fresh name check remain manual
RELEASE.PUBLICATION.0 gates.

## C source SDK and installed contract

`python3 tools/package_sdk.py --revision 14cacba16f3cf2a00448d720502e1cfe975f2e83
--output replai-c-sdk-0.1.0.tar.gz --check-reproducible` generated identical
bytes twice. The archive normalizes path order, timestamps, uid/gid, modes and
gzip metadata. `SDK-METADATA.json` records bundle schema, version, Git revision,
Git tree and ABI identity. The 52-file source set contains the locked Rust
workspace needed by the adapter, ABI header/schema/generator, CMake templates,
stage/qualification tools, C/C++ examples, tests, LICENSE and concise SDK docs;
it contains no `.git`, `target`, cache, credential, fuzz corpus or unrelated
README media.

After standalone extraction the SDK regenerated ABI records, built
`replai-c`, staged prefix A, moved the complete tree to prefix B, made prefix A
unavailable, and qualified only prefix B:

```text
include/replai.h
lib/libreplai_c.a
lib/libreplai_c.so | libreplai_c.dylib
lib/pkgconfig/replai.pc
lib/cmake/replai/replai-config.cmake
lib/cmake/replai/replai-config-version.cmake
lib/cmake/replai/replai-targets.cmake
share/licenses/replai/LICENSE
```

The Linux ARM64 MSRV receipt records these installed identities (native library
bytes vary by platform/toolchain; every job retains its own machine-readable
receipt):

| Installed file | Bytes | SHA-256 |
| --- | ---: | --- |
| `include/replai.h` | 5,645 | `c0644f2d541e7537bc5c96f3d70679b2f2f18f7b89af513de96bda2e7cb49bf5` |
| `lib/libreplai_c.a` | 30,178,226 | `3f6438b22b467485f172e74f6609f1d38cae12ea62458a6a93759c2a10c02a90` |
| `lib/libreplai_c.so` | 6,995,152 | `463de9771d37ee5df90e6af5f2b5e897b2394b87f7f7c85519459c1625c82a06` |
| `lib/pkgconfig/replai.pc` | 286 | `88b01b4aabb444ecfb2c11cc8dd205f40af73071e1c3fb768b99f2c47961d337` |
| `lib/cmake/replai/replai-config.cmake` | 58 | `0952fd6573c1bc90e5b18c816eafc16f07e374ceb62a4924d6d5a7d093e11f8e` |
| `lib/cmake/replai/replai-config-version.cmake` | 406 | `3e83642ed76ebb3f906ad994240d67b8605ed1d8e868ffb5d20b7b3bc3330cf5` |
| `lib/cmake/replai/replai-targets.cmake` | 1,054 | `47ba92a36c02e5c3874a910dcd33c60f7045bba557b6dc9120864e08044ce18d` |
| `share/licenses/replai/LICENSE` | 1,065 | `2bb60eec69be9d7203ba11ccc22a7727e75146ddd5cf6f7ab1a2575e0ee0362f` |

On both Linux architectures and macOS ARM64, at both toolchain selections, the
following external moved-prefix consumers passed:

| Discovery | Language | Static | Shared |
| --- | --- | --- | --- |
| pkg-config | C11 | PASS | PASS |
| pkg-config | C++17 | PASS | PASS |
| `find_package(replai CONFIG REQUIRED)` | C11 | `replai::static` PASS | `replai::shared` PASS |
| `find_package(replai CONFIG REQUIRED)` | C++17 | `replai::static` PASS | `replai::shared` PASS |

`pkg-config --libs` and `--static --libs` expose platform system dependencies;
the CMake imported targets expose include roots, artifact identity and static
transitive requirements. Metadata contains no prefix-A, checkout or temporary
path. Linux `ldd` and macOS `otool`/install-name checks resolve the shared
library from prefix B; static consumers have no dynamic REPLAI dependency.
Existing symbol, numeric, struct-layout, generated-header, old-client,
static/shared PTY and C++ compatibility checks all pass. The exact-source
general CI independently ran Linux Valgrind and macOS native leak qualification
and reported zero attributable errors/leaks. Packaging does not nest those
memory tools inside the extracted-SDK installation matrix.

## Independent debugger consumer

The external `replai-packaged-debugger 0.1.0` is copied to a fresh directory,
rewritten only to point at the extracted `.crate`, locked there and run outside
the producer checkout. It uses the public session tier and owns a deterministic
debugger-like model: breakpoints, frames, inspect/continue vocabulary and a
multiline `watch` expression. It attaches to no process.

The fixture proves all nine semantic assertions:

1. an immutable snapshot produces a stale result after editing and is refused;
2. a fresh analysis result applies;
3. rich completion preserves host order and accepts deliberate selection;
4. host style spans/non-canonical hints remain presentation;
5. Invalid preserves text and shows a diagnostic;
6. corrected Incomplete inserts one continuation newline;
7. Complete submits the exact validated revision;
8. two finite serialized pipe/timer notices preserve draft, cursor and selection;
9. the final host result is a structured `Document`.

The portable oracle uses `Editor`, `AnalysisSnapshot`, all three host result
types and `replace_at` stale refusal; it does not call absent Windows terminal
methods. Native Linux/macOS runs use real PTYs. Deterministic receipt SHA-256 is
`1cb111f7b848beb25734cf535f228d6eba563ee1698ec2dfb42f3e2841e4daa1`;
terminal transcript SHA-256 is
`c8bf4b56c2a77ec2c9bd1afcbeb764a83e019ea94cb0579f3747f1b31b2b2e04`.
The event source has no concurrent terminal writer; the host serializes every
Interaction mutation. This is finite-notice evidence, not O1/O2 qualification.

## Executed cookbook

[Use cases](../use-cases.md) is the canonical recipe owner. The fast recipe
oracle reports `PASS cookbook: 19 selected supported/deferred recipe contracts`.
Each recipe points to a compiled example, packaged fixture or explicit
unsupported posture and labels Rust/C/platform scope. The executed set covers:

- simple read/evaluate, host history admission and host persistence/reload;
- rich completion, validated multiline, delayed results and host spans/hints;
- existing reactors, finite notices and host-coalesced output;
- turn-by-turn model/chat streaming after editor close;
- structured reports, pkg-config, CMake static/shared and C++ consumers;
- NO_COLOR/plain interaction;
- unsupported secret entry and independent concurrent writers.

History remains bounded in-memory mechanics: the host decides admission,
persistence, reload, privacy and retention. Active-edit output remains finite
and host-serialized. Sustained token output is supported after close through
host I/O, followed by reopening; no active-edit streaming/fairness or writer
arbitration contract is claimed.

## Orchestration, isolation and CI

The canonical full gate is:

```sh
python3 tools/qualify_packaging.py --source HEAD \
  --work /tmp/replai-packaging --full
```

It uses isolated extraction, target and consumer directories; generates a JSON
summary and per-phase logs; verifies source/toolchain/artifact identities; and
fails non-zero on any phase. The workflow executes it natively on Linux GNU
x86_64/ARM64 and macOS ARM64 at the MSRV and stable, plus a Windows MSRV portable
job. Normal CI retains the faster package, recipe, SDK reproducibility, CMake
syntax and external-smoke integrity checks without pretending to rerun G3/G4.

The artifact leakage scan found zero checkout path, home path, hostname or
credential-file hits. Environment lookup is restricted to the intended package
or moved prefix. No registry token, SSH key, GitHub token, home configuration or
release binary is retained.

## Findings

| ID | Observation | Classification and repair | Final status |
| --- | --- | --- | --- |
| P001 | Unanchored Cargo include globs admitted repository `tools/docs/node_modules` | Packaging defect; root-anchor the package inventory and add positive/negative oracles | CLOSED; 91-file audited crate |
| P002 | Cargo-normalized target dependency syntax failed a source-text architecture oracle | Harness defect; accept the equivalent normalized manifest form | CLOSED; extracted crate tests pass |
| P003 | Debugger fixture applied stale presentation while Interaction was closed | Harness defect; use the supported stale model and later the platform-neutral Editor oracle | CLOSED; semantic stale assertions pass |
| P004 | PTY fixture omitted `TERM` | Harness defect; run under deterministic `xterm-256color` capability input | CLOSED; Linux/macOS PTYs pass |
| P005 | Fixture accepted the wrong candidate and attempted Interaction output after Complete had closed it | Harness defect; deterministic navigation/edit sequence and standalone Document output | CLOSED; completion/submission assertions pass |
| P006 | Clean extracted package entered offline test before fetching dev dependencies | Harness defect; locked fetch precedes isolated offline testing | CLOSED; fresh runners pass |
| P007 | CI native packaging smoke invoked CMake without installing it | CI environment defect; declare CMake with native tools | CLOSED; CI/native matrix pass |
| P008 | Windows path separators broke package inventory membership | Harness defect; normalize Cargo inventory paths | CLOSED; Windows package oracle passes |
| P009 | MSRV workflow used the minimal toolchain without rustfmt required by ABI regeneration | Qualification environment defect; install exact-version rustfmt explicitly | CLOSED; MSRV ABI check passes |
| P010 | Windows portable qualification tried POSIX examples and a POSIX Interaction consumer | Harness defect; run portable tests and an Editor/result-model consumer while preserving rustdoc | CLOSED; Windows portable job passes |
| P011 | A historical `--source` could be paired with package bytes from the current checkout | Harness defect; require `--source` to resolve to clean checked-out HEAD before any phase | CLOSED; final package identity is source-bound |
| P012 | macOS `leaks` stalled when launched from the Python process that had already driven PTY children | Harness defect; `--phase all` gives prepare/static/shared/memory/audit separate processes | CLOSED; exact-source native macOS CI leak gate passes |
| P013 | Process-isolated C phases could fill the outer qualifier pipe with successful command logs | Harness defect; retain detailed per-phase files and bound successful orchestration output | CLOSED; phase orchestration completes |
| P014 | Nesting the macOS `leaks` phase below extracted-SDK packaging still stalled after both PTY phases had passed | Harness ownership defect; packaging now invokes the process-isolated `installed` contract (prepare/static/shared/audit), while exact-source native CI independently retains the mandatory leak gate | CLOSED; packaging matrix 7/7 and native macOS leak CI pass on the same source |

No runtime or public API defect was established. No source under `src/`, C
header/schema/layout/symbol or ABI number changed. YAI, YVEX and the private
BOUNDARY repository were not modified or repinned.

## Residual publication gates

The package identity, artifacts, MSRV, consumers and recipes are ready for the
later freeze audit. E3/V0 must review and freeze the complete Rust/API/ABI/package
surface on one final candidate. RELEASE.PUBLICATION.0 must authenticate a real
publisher, recheck the crates.io name, publish, verify hosted docs/downloaded
artifacts, create any tag/release and make the actual compatibility commitment.
None of those actions occurred here.
