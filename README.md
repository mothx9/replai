<p align="center">
  <img src="assets/replai-logo-dark.svg#gh-dark-mode-only" alt="REPLAI logo" width="240">
  <img src="assets/replai-logo-light.svg#gh-light-mode-only" alt="REPLAI logo" width="240">
</p>

<h1 align="center">REPLAI</h1>

<p align="center">
  <strong>The application owns the language.<br>REPLAI owns the line.</strong>
</p>
<p align="center">Terminal interaction infrastructure for Rust and C.</p>
<p align="center">Rust · C ABI 1 · Linux / macOS · MIT · pre-release</p>

A small line reader is easy to embed. A long-lived console needs more: editable
Unicode, completion, multiline input, output arriving mid-draft, and a terminal
that is restored when the interaction ends.

REPLAI keeps those mechanics below your application. Bring a parser, a command
catalog, a model client or a debugger. Keep execution and the event loop.
Use a blocking call when that is enough; take explicit control when it isn't.
The editor and renderer stay the same. No full-screen framework is required.

<p align="center">
  <img src="assets/terminal-session.png" alt="Real REPLAI terminal: an indented multiline statement is submitted, then a host result arrives while bu and the selected bundle completion remain active." width="912">
</p>

Real PTY, host-owned semantics: validated multiline input, completion and output
with the active draft, cursor and selection preserved.

## Run it

On Linux or macOS, with Git and a current stable Rust toolchain:

```sh
git clone https://github.com/mothx9/replai.git
cd replai
cargo run --locked --example console
```

1. Type `begin {`, press Enter, then Tab and `task`. Enter continues the draft;
   type `}` and press Enter to submit the complete block.
2. Type `bu`, press Tab to see candidates, then Tab again to select `bundle`.
3. Leave the menu open for two seconds. A host timer delivers a local result;
   your draft and selection remain where you left them.
4. Enter accepts the candidate. A second Enter submits. Ctrl-D on an empty
   draft exits; Ctrl-C interrupts editing.

The [console host](examples/console.rs) uses fixed local data. It runs no build,
contacts no service and shares one host analysis across styles, completion and
brace validation. Pasted multiline text remains one draft.

## The ownership line

```text
YOUR APPLICATION
commands · parser · execution · state · persistence
candidate discovery · validation meaning · event loop
                         │
                  public contracts
                         ▼
REPLAI
Rust: blocking / session / driven    C ABI 1: session
                         │
                one interaction engine
editor · revision provenance · completion mechanics
multiline · generic presentation · terminal lifecycle
                         │
                         ▼
                Linux / macOS terminal
```

**REPLAI owns** editing, history navigation, safe revision-bound application of
host results, candidate navigation, rendering and coordinated output.

**Your application owns** the language, semantic classification, discovery,
execution, persistence and scheduling. REPLAI never decides what a command means
or starts an analysis job on your behalf.

## Keep the small host small

Add the qualified runtime checkpoint to your own Cargo project:

```toml
[dependencies]
replai = { git = "https://github.com/mothx9/replai", rev = "da16302c33cdce5ef40978e02e9d8dff045c93f9" }
```

```rust
use replai::{Editor, Interaction, Prompt, ReadOutcome};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = Interaction::new(Editor::new(65_536, 100));
    loop {
        match input.read_line(Prompt::new("demo")?)? {
            ReadOutcome::Submitted(text) => println!("received: {text}"),
            ReadOutcome::Interrupted => {},
            ReadOutcome::EndOfInput => break,
        }
        input.editor_mut()?.clear();
    }
    Ok(())
}
```

The terminal is restored before the call returns. The host decides what to do
with each outcome. The [retained-reader example](examples/simple.rs) also admits
history explicitly. This is enough for a deterministic CLI or a turn-by-turn
model chat: read input, run your application, show its result, read again.

## Let the host analyze

An explicit session receives `Event::CompletionRequested`. The application
chooses candidates from one immutable snapshot; REPLAI presents and applies them.
For a host that recognizes `bu`:

```rust
use replai::{AnalysisOutcome, CompletionCandidate, CompletionError, CompletionSet, Interaction};

fn offer_build(input: &mut Interaction) -> Result<AnalysisOutcome, CompletionError> {
    let snapshot = input.analysis_snapshot();
    let candidate = CompletionCandidate::new(0..snapshot.text().len(), "build ", "build")?
        .with_annotation("Build the project")?;
    input.present_completions(CompletionSet::new(snapshot.revision(), vec![candidate])?)
}
```

Display labels and inserted text can differ. Results may arrive later; if the
draft or cursor changed, delivery returns `Stale` without changing the screen.
The [completion host](examples/completion.rs) demonstrates multiple candidates;
[analysis presentation](examples/analysis-presentation.rs) adds safe style spans
and explicitly non-canonical hints. The host supplies both.

### Enter asks; your parser answers

Opt into `SubmissionPolicy::Validated` while closed. Enter then yields
`Event::SubmissionRequested(snapshot)`. Return your decision against that revision:

```rust
use replai::{AnalysisSnapshot, Interaction, ValidationDisposition, ValidationError,
             ValidationOutcome, ValidationResult};

fn return_decision(input: &mut Interaction, request: AnalysisSnapshot,
                   decision: ValidationDisposition) -> Result<ValidationOutcome, ValidationError> {
    input.apply_validation(ValidationResult::new(request.revision(), decision)?)
}
```

**Complete** submits exactly that draft. **Incomplete** inserts a newline and
continues editing. **Invalid** retains the draft and presents safe diagnostics.
Handle the returned `ValidationOutcome.event`; a completed submission arrives
there. A stale decision does none of these.

```text
demo> begin {
...     task
... }
[ok] host received 18 bytes; nothing executed.
```

The [small validator](examples/support/validation.rs) owns the brace grammar.
REPLAI owns continuation layout, vertical navigation and Tab indentation in
continuation whitespace. Completion-menu acceptance takes precedence over
submission. Direct submission remains the default for small hosts.

### Bring your own reactor

```text
terminal readiness ─┐
network result ─────┼── your reactor ── serialized Interaction calls
resize notification ┤
REPLAI deadline ────┘
```

Use `wait_interest()` for ready work or a deadline, and `input_source()` for the
borrowed POSIX readiness descriptor. Deliver terminal events with
`advance(Wake::InputReady)`, `advance(Wake::Resize)` or `advance(Wake::Deadline(token))`.
Application results use `output_document`, `external_output` or the analysis APIs.

The [driven host](examples/driven.rs) owns the wait. Idle REPLAI requires no
periodic wake without an event or deadline. The simpler session `poll()` keeps
periodic resize observation. Neither tier installs signal handlers, owns an async
runtime or permits concurrent independent writers.

## Give output structure

No ANSI strings, manual padding or application-specific rendering vocabulary:

```rust
use replai::{Block, Document, Severity, Text, Theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let document = Document::new(vec![
        Block::Heading { level: 1, text: Text::new("Workspace")? },
        Block::KeyValue(vec![(Text::new("Mode")?, Text::new("local")?)]),
        Block::Status { severity: Severity::Success, text: Text::new("Ready")? },
    ])?;
    print!("{}", document.render(60, Theme::new(false, false, None))?);
    Ok(())
}
```

```text
# Workspace
Mode  local
[ok] Ready
```

The same document can go through `Interaction::output_document` during editing.
Lists, responsive tables, literal blocks and composed prompts use the same safe
text and cell geometry. NO_COLOR preserves structure; standalone plain rendering
works without a TTY. See the [structured example](examples/structured.rs) and
[standalone report](examples/report.rs).

## C and C++ are consumers of the same engine

ABI 1 exposes opaque handles, explicit events and caller-owned UTF-8 buffers.
For example, a C host can wait for the next observable outcome:

```c
#include "replai.h"

replai_status next_event(replai_handle *input, replai_event *event) {
    replai_status status;
    do {
        *event = (replai_event){.struct_size = sizeof *event,
                               .abi_version = REPLAI_C_ABI_VERSION};
        status = replai_poll(input, 100, event);
    } while (status == REPLAI_OK && event->kind == REPLAI_EVENT_NONE);
    return status;
}
```

Build staged static/shared artifacts and the [complete C host](examples/c/demo.c):

```sh
cargo build --locked --release -p replai-c
python3 tools/stage_c.py --prefix /tmp/replai-install
export PKG_CONFIG_PATH=/tmp/replai-install/lib/pkgconfig
cc examples/c/demo.c $(pkg-config --cflags --libs replai) \
  -Wl,-rpath,/tmp/replai-install/lib -o /tmp/replai-demo
/tmp/replai-demo
```

Use an absent or empty staging directory. Installed consumers need a native
C/C++ toolchain and the artifacts, not Rust or a REPLAI checkout. ABI 1 retains
session polling, synchronous completion replacement, prompts and plain coordinated
output. Rich candidates, analysis, validated submission, structured documents and
driven embedding are currently **Rust-native only**. [Installation and ABI contract](docs/c-api.md).

## Scope and support

REPLAI is not a shell, parser, command framework, async runtime, full-screen TUI
or background application scheduler. Those boundaries keep it embeddable.

| Platform | Current scope |
| --- | --- |
| Linux | Real terminal runtime: Rust blocking/session/driven; C ABI 1 static/shared |
| macOS | Real terminal runtime: same Rust tiers; C ABI 1 static/dylib |
| Windows | Portable editor, engine and presentation tests only; **no terminal backend** |

Interactive defaults require admitted cursor/erase mechanics and matching TTYs;
unknown or `TERM=dumb` defaults refuse before raw mode. The compatibility session/C
entry retains explicit VT assumptions. NO_COLOR is styling policy, not terminal
capability discovery. Unicode width follows a deterministic policy, with known
emulator/font differences. Exact limits belong to the [interaction contract](docs/interaction.md).

Pre-release, MIT licensed. Distribution is pinned Git source and staged C
artifacts. There is no crates.io release, declared MSRV, API freeze or long-term
Rust/ABI compatibility commitment yet.

## Evidence, not adjectives

[CI on master](https://github.com/mothx9/replai/actions/workflows/ci.yml) executes
the portable and native gates. The [qualified runtime checkpoint](https://github.com/mothx9/replai/actions/runs/34509792610)
passed all 12 jobs; this README wave leaves its runtime source unchanged.

| Property | Executed evidence |
| --- | --- |
| Terminal ownership | Real Linux/macOS PTYs: draft/cursor restoration, resize, descriptor lifecycle and failure cleanup |
| Native memory and ABI | Linux Valgrind, macOS native leak checks, record/symbol checks, isolated static/shared C and C++ consumers |
| Host analysis | Revision/stale models plus real delayed-result, completion, validation and output interactions |
| Performance | Separate editor, layout/render, transport, allocation and analysis measurements at recorded source revisions |

Performance dossiers identify the exact workload, machine and pinned source.
The [matched comparison](docs/engineering/macos-perf.md#final-same-run-comparisons-and-parity)
and [analysis costs](docs/engineering/analysis-presentation.md#performance-and-memory)
are bounded measurements, not a universal ranking or a benchmark of every newer
commit. The signature image has an [executable capture and restoration check](docs/development.md#readme-terminal-preview).

## Find the next layer

- **Start:** [examples and use-case recipes](docs/use-cases.md), including deterministic hosts and model clients.
- **Embed:** [interaction](docs/interaction.md), [C installation](docs/c-api.md).
- **Present:** [documents, themes and geometry](docs/presentation.md).
- **Understand:** [architecture](docs/architecture.md), [documentation authorities](docs/README.md).
- **Verify:** [development and qualification](docs/development.md).
- **Track:** [roadmap](ROADMAP.md), [changelog](CHANGELOG.md).

For cross-repository coordination, optional [producer metadata](.boundary/README.md)
records capability changes. It is not a build or runtime dependency.

[Contribute](CONTRIBUTING.md) with a reproduction, terminal/OS details and evidence
at the changed boundary. [Report an issue](https://github.com/mothx9/replai/issues) · [MIT license](LICENSE).
