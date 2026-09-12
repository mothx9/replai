# Development method and qualification

[Contributing](../CONTRIBUTING.md) is the entry point; [AGENTS.md](../AGENTS.md)
contains mandatory agent rules. This document owns the operational workflow.

## Reconcile and define the change

Inspect branch, HEAD/tree, remotes, status and both staged and unstaged diffs
before edits and again before commit. Establish which paths are concurrently
owned. Preserve unrelated work and never reset the checkout to fit an expected
SHA. If a relevant source changes during qualification, reconcile it and rerun
the affected checks before making a claim about the new tree.

Describe the observed problem, its owner, intended behavior and rejection paths.
Use source, tests and the ABI schema as authorities. Choose a small change that
answers a demonstrated integration or correctness need. Host policy, language
evaluation, command catalogs and application storage remain outside this library.

## Tools

Use current stable Rust with rustfmt and Clippy. The unpublished 0.1.0 candidate
declares Rust 1.98.1 as its floor after packaging qualification; record
`rustc --version --verbose` and `cargo --version` with evidence.
Fetch the locked graph with `cargo fetch --locked`; Rust checks may then use
`CARGO_NET_OFFLINE=true`. Commit Cargo.lock for reproducible repository checks.

The documentation lane uses Python 3, Node 22 and a separate locked parser
installation (Mermaid 11.17.2 and jsdom 26.1.0):

```sh
npm ci --prefix tools/docs --ignore-scripts
```

These are documentation-only dependencies. They are not required by Cargo or
installed Rust/C consumers. Mermaid parses actual fenced diagrams using a DOM
provided by jsdom, without a browser, SVG copies or screenshot comparison.

Complete native qualification additionally requires cc/c++, pkg-config,
Valgrind and Landlock on Linux, or native `leaks` and sandbox isolation on
macOS. Missing required memory/isolation tools fail the native gate. Tests use temporary prefixes and never install into system directories.

## Select checks by boundary

| Change | Required evidence |
| --- | --- |
| Markdown, diagrams or documentation checks | Documentation checks below; manual authority/reference review |
| Rust API examples or package inventory | Documentation checks plus doctests and foundation tests |
| Editing/input/lifecycle/presentation | Full Rust checks including real PTYs; C regressions if shared behavior changes |
| C declarations, binding or staging | Generated drift check and complete isolated native qualification |
| Dependency or shared boundary changes | Full qualification and license/source/public-type review |

For the documentation surface:

```sh
python3 tools/check_docs.py
python3 -B tools/test_check_docs.py
python3 tools/generate_abi.py --check
cargo test --doc
cargo test --test foundation
git diff --check
```

The checker validates local links/anchors, document and asset reachability,
retired-surface absence, the designated project-status owner, roadmap maturity
IDs/states/counts, program references and unique engineering selection, ABI identity/tag
tables against the schema, Markdown table delimiters, and Mermaid syntax. Its negative fixtures must prove
that rejected documents produce a file-specific error. It does not infer prose
accuracy, verify live external URLs or prove capability by counting tests.
Release-scope checks require one valid classification for every non-established
maturity row, exact maturity agreement, evidence links and consistent class counts.
These are documentation controls, not execution of the future
[release qualification gates](release-scope.md#required-evidence-before-freeze-and-tagging).

For Rust behavior:

```sh
cargo fmt --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --doc
cargo test --workspace --release
```

The single full qualification entry point is:

```sh
python3 tools/qualify.py --work /tmp/replai-qualification
```

Choose a fresh work directory. It records each command and gate log, executes
documentation/Rust checks, [real embedding qualification](engineering/embedding.md) and [native C qualification](c-api.md#executable-authority-and-current-limits),
then requires a clean repository. `--allow-dirty` supports development only;
it does not qualify clean closure. CI separates documentation, Rust core/PTY,
release, and C prepare/static/shared/memory/audit gates.

## Evidence and independence

Report the tested revision/tree, toolchain, exact commands and observed property:
termios before/after, submitted bytes, cells/cursor, FD ownership, ABI layout,
staged loader resolution or memory-checker results as appropriate. Test count,
successful compilation and a push are not substitutes for those observations.
Keep raw logs outside Git; retain small enduring fixtures only when they define
a contract. Report local evidence and remote CI separately.

Review dependency sources, crate/features, public names, environment reads,
examples and build scripts. A clean checkout must work without any application
repository or private configuration. Foundation tests enforce dependency-source
and package inventory rules. The native qualification denies repository reads
with Landlock on Linux and a generated sandbox policy on macOS when compiling and running staged consumers. No stale local or
globally installed library may stand in for the qualified artifact.

For ABI changes edit [api/c-abi.json](../api/c-abi.json), run
`python3 tools/generate_abi.py`, and commit generated header, Rust records,
signature assertions and probes together. A changed declaration needs layout,
misuse and real external C evidence. Never weaken core unsafety policy to make
a binding convenient.

## Documentation lifecycle

Admit a document only when a distinct subject, audience or contract needs an
owner. [The map](README.md) assigns those owners. README gives orientation;
architecture gives structure; contracts give behavior; this method gives
procedures; ROADMAP alone gives current project state. CHANGELOG records consumer
capabilities, compatibility and fixes, not editorial or implementation steps.

Transfer surviving facts before retiring a report. Preserve chronology in Git,
with an immutable link where useful, rather than archive folders or duplicate
ledgers. Keep donor names in dedicated historical/reference evidence and the
required license attribution; public API and generic examples remain neutral.
Keep diagrams embedded as Mermaid and edit their surrounding explanation with
them. Review the exact claim supported by each primary reference: a classical
evaluator description does not establish terminal conformance or API compatibility.

Before delivery inspect the actual final diff, rerun affected checks, commit
only the intended paths and use an ordinary push. Verify the remote revision
and actual CI outcomes. Report any remaining failures with causal scope rather
than either hiding them or promoting unrelated failures into library defects.

## Milestone closeout

Every completed milestone ends with an impact classification. This matrix points
to existing authorities; it does not create another project-state ledger.

| Change class | Required closeout review |
| --- | --- |
| Runtime behavior | CHANGELOG; owning interaction/presentation/architecture contract; tests and evidence dossier; ROADMAP maturity; README if first-use or visible behavior changed; producer metadata/delta |
| Public Rust API | Rustdoc; README snippets and examples; CHANGELOG; contract and compatibility notes; consumer impact; producer metadata |
| C ABI change or requalification | `api/c-abi.json`; `include/replai.h`; C API docs/examples; symbol/layout/static/shared/C++ evidence; README surface matrix; producer metadata. A break requires a new explicit ABI identity; ABI 1 is never reinterpreted. |
| Platform support | ROADMAP; release scope; README platform matrix; architecture/capability docs; CI and native evidence; producer metadata |
| Performance change or requalification | Performance dossier; Q2 workload/source identity; benchmark visual only when the represented workload was rerun with comparable qualified evidence; README figures only when their exact source changes |
| Capability added or removed | README capability/API matrices; runnable example; owning contract; ROADMAP; CHANGELOG; producer delta; consumer-impact classification |
| Qualification only | Evidence dossier; producer evidence identity; ROADMAP only when promotion criteria pass; README Verification only when a public support claim changes |
| Documentation or branding only | Affected public surface and its checks. No runtime claim, capability delta or performance qualification. |
| Release-scope change | Explicit authorization; `docs/release-scope.md`; ROADMAP; README release/support matrices; compatibility and support claims |

README changes are required when a milestone changes first-use instructions,
installation, public capabilities or surfaces, integration tiers, platform
support, current release/support status, a major qualified performance claim,
visible terminal behavior, or scope/non-goals. Internal refactors, invisible
optimizations within an established claim, harness-only repairs, documentation
carrier commits and evidence reruns with the same public support normally leave
README unchanged.

A screenshot or GIF is regenerated when the illustrated terminal behavior,
prompt/theme, example API or canonical showcase changes. A new HEAD alone is not
a trigger. A benchmark visual changes only after its exact workload is rerun and
qualified, source identity is recorded and methodology remains comparable.
Historical evidence stays labeled instead of being silently refreshed.

Use this checklist at closeout:

```text
[ ] Runtime and public behavior reconciled
[ ] Owning contract docs updated where applicable
[ ] Tests and qualification executed
[ ] Engineering evidence recorded
[ ] ROADMAP state and unique selection reconciled
[ ] CHANGELOG reviewed
[ ] README impact classified
[ ] Screenshot/GIF impact classified
[ ] Performance visual impact classified
[ ] Producer metadata/delta reconciled
[ ] Consumer impact classified
[ ] CI green
[ ] Worktree clean
[ ] Local/remote HEAD and tree recorded
```

## Embedding qualification

`python3 tools/embedding_pty.py --work /tmp/replai-embedding` builds the real
simple/driven examples and runs an external POSIX reactor over a PTY and an
independent application socket. It records exact drafts/cursors, VT cells,
resize/deadline observations, termios before process exit and repeated lifecycle
resources. Run `--memory` in a fresh directory for Valgrind on Linux or native
`leaks --atExit` on macOS. These complement the full existing session/C gates.
CI runs both native embedding jobs and preserves their observation artifacts.
Portable engine tests exercise idle/no-I/O and stale-deadline contracts without
claiming a Windows terminal backend. Hosted latency is characterization only.

<a id="readme-terminal-preview"></a>

## Public README assets

The signature [PNG](../assets/readme/terminal-showcase.png) and illustrative
[GIF](../assets/readme/terminal-showcase.gif) run the
[showcase](../examples/showcase.rs) through an actual PTY. The host fixture uses
only public REPLAI APIs and verifies output-time preservation of the logical
draft, cursor, revision and completion selection. The script separately observes
the rendered cells, Unicode edit, hint, menu selection, validation, continuation,
structured result, history recall, termios restoration and paste-mode cleanup.

[capture_showcase.py](../tools/readme/capture_showcase.py) decodes actual bytes
with pyte 0.8.2 and rasterizes cells with Pillow 11.3.0 and DejaVu Sans Mono. The
PNG and GIF are 1072 × 891. The committed GIF has 35 encoded frames, 11.63 seconds
of frame timing and is not measurement evidence. The opaque terminal background
works in light and dark GitHub themes; the PNG is the motion-free fallback.

The architecture light/dark SVG pair and Q2 benchmark SVG come from
[render_vectors.py](../tools/readme/render_vectors.py). The benchmark reads the
immutable [Q2 registration](../tools/hardening/evidence/q2-linux-aarch64.json)
and labels its evidence source, machine and recorded-workload limitation. The
narrow [asset manifest](../assets/readme/manifest.json) records file/input hashes,
dimensions and exact commands. It is asset provenance, not project or release status.

Reproduce all public assets on Linux with a current Rust toolchain, Python 3 and
`fonts-dejavu-core`:

```sh
python3 -m venv /tmp/replai-capture-env
/tmp/replai-capture-env/bin/pip install -r tools/docs/capture-requirements.txt
cargo build --locked --example showcase
/tmp/replai-capture-env/bin/python tools/readme/capture_showcase.py
python3 tools/readme/render_vectors.py
python3 tools/readme/build_manifest.py
```

Use the corresponding `--check` form for each script to compare generated bytes
or manifest metadata without modifying tracked files. Raster identity is scoped
to the pinned dependencies/font; native terminal suites remain runtime authority.
The Python packages are documentation dependencies and never enter Cargo or an
installed consumer. `python3 tools/docs/check_readme.py` compiles every README
Rust/C snippet, executes the blocking host in a PTY and checks the exact Document
transcript.

The prior [console capture](../assets/terminal-session.png) remains reproducible
for historical documentation continuity with `cargo build --locked --example
console` followed by `tools/docs/capture_terminal.py --check`. Earlier focused
captures remain available for [query/results](../assets/terminal-results.png),
[standalone report](../assets/terminal-report.png),
[completion](../assets/terminal-completion.png) and
[validation](../assets/terminal-validation.png).

## Analysis protocol qualification

`cargo test --test analysis` runs the portable revision model (including 20,000
generated mutations); library conformance adds semantic lifecycle, decoder and
output invariants. `python3 tools/analysis_pty.py --work /tmp/replai-analysis`
uses the same Linux/macOS external-reactor oracle, with delayed host result
arrival, stale screen preservation, history and repeated resource lifecycles.
These run in the complete qualifier and native CI; Windows executes the portable
model only. Snapshot allocation/clone measurements live in the existing isolated
performance harness under the `analysis/` component filter.


Completion changes additionally run `python3 tools/completion_pty.py --work /tmp/replai-completion`
and its `--memory` variant (Valgrind on Linux, native leaks on macOS). The same
external-reactor observer validates delayed delivery, menu restoration, resize,
read-ahead and termios; the session example validates synchronous delivery.
`tests/completion.rs`, Engine completion tests and virtual transport conformance
execute on the Windows portable lane too. `tools/perf/completions.rs` records
bounded candidate costs separately from ordinary editing benchmarks.


Validation/multiline changes run `cargo test --test validation`, portable Engine
conformance and `python3 tools/validation_pty.py --work /tmp/replai-validation`.
The `--memory` variant executes the same active paths under Valgrind/Linux or
native leaks/macOS. Both are part of the full qualifier and native CI. The
[use-case map](use-cases.md) supplies manual recipes; the
[dossier](engineering/validation-multiline.md) separates timing from correctness.

I2 changes additionally run `cargo test --test analysis_presentation` and
`python3 tools/analysis_presentation_pty.py --work /tmp/replai-analysis-presentation`,
including `--memory` for native Valgrind/leaks. These gates are in the complete
qualifier and native CI. The shared PTY launcher defaults to plain performance
fixtures; correctness oracles explicitly opt into styled output and verify actual
cell styling. Windows executes portable I2 payload/engine/layout tests only.

## Release packaging

`tools/qualify_packaging.py` is the canonical G3/G4 entry point. It requires an
empty evidence directory and a committed source revision because artifact
identity comes from Git, not from an untracked working copy:

```sh
python3 tools/qualify_packaging.py \
  --source <packaging-source-sha> \
  --work /tmp/replai-packaging-$(uname -m) \
  --full
```

The command audits Cargo metadata, creates and unpacks the `.crate`, resolves a
fresh external Rust consumer, builds docs for the selected target set, produces
the reproducible C source SDK, moves the installed prefix, and exercises C11 and
C++17 through pkg-config and CMake. On Linux/macOS `--full` also runs the native
C ABI/PTY/memory matrix. It writes `packaging-summary.json`, command logs and
artifact hashes beneath the evidence directory. Use Rust 1.98.1 exactly for the
MSRV run; current-stable and native target lanes are distinct release evidence.

Fast CI may run package and recipe integrity without retaining release
artifacts. Full native packaging qualification remains a release gate and must
not be inferred from a cross-build.
