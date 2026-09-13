# Completion, keymap and suggestion qualification

This dossier records `COMPLETION.KEYMAP.SUGGESTION.0`. It qualifies the public
Rust surface added above the interaction-ergonomics substrate. It does not claim
long-lived output, a Windows terminal backend, sensitive input, a high-level
facade or modal editing.

## Source and architecture

The requested baseline was `623ebadf29de609d25c72d01ffb1cffe2841b41f`
(tree `07782e0eae6e8a63af816907410e4835cf6acc4e`, runtime tree
`8db955760152f10a7d34056796e2c9f0e1e003af`). Initial implementation was recorded
at `5cc9482cd9a372fecdb1c4d57de96559be310170`; the final qualified runtime is
`ab34d0b78524329a0c2404c8e60a23f5ff5e92f8` (runtime tree
`62596c22ad2568c46c4e742b21136b31f9e1eb96`). Subsequent documentation,
measurement and producer records are carriers. Exact final identities and CI
runs are recorded in the closeout section.

The source retains one path:

```text
bytes -> Decoder -> Key -> KeyMap -> Action -> Engine -> Editor/temporary surfaces
```

`Decoder` recognizes physical protocol input. `KeyMap` assigns policy. `Action`
describes semantic intent. `Engine` executes the same action path for terminal
input and direct native `Interaction::apply_action`. `Editor` remains the sole
canonical text/cursor/revision owner. During archaeology an early draft let
`Engine` import the decoder key type; the architecture regression rejected it
and keymap ownership moved to `Interaction`/`Terminal` before qualification.

## Public action and key contract

Public `EditAction`, `CompletionAction`, `SuggestionAction` and non-exhaustive
`Action` cover the established edits, lifecycle requests, completion request and
navigation, history-search acceptance/dismissal and suggestion actions. Public
`Key` contains `Named`, validated `Control(u8)` and validated `Meta(u8)` forms.
Named keys cover arrows, Home/End, Delete/Backspace, Enter, Tab/BackTab, Escape
and PageUp/PageDown. Raw escape strings, renderer operations, `Input` and paste
are not public binding values.

Printable UTF-8 always remains `Text`; bracketed paste always remains one atomic
text transport. Neither can invoke a custom binding. Multi-key macros, callbacks,
modal state and arbitrary modifier combinations are deliberately absent.

## Keymap bounds and defaults

`KeyMap::new` installs the prior compatibility profile. A closed `Interaction`
can inspect, bind/replace, unbind, reset one key or reset all overrides. Opening
clones the immutable map into the terminal driver; active mutation is refused.
The map retains at most 128 sorted overrides. Duplicate bind replaces, a 129th
entry fails atomically, and binary-search lookup is bounded `O(log 128)` with
zero allocation. Redo and forward-word kill are bindable without new default
shortcuts. PageUp/PageDown default to clamped completion page navigation.

## Completion helpers

The existing revision-bound `CompletionSet` remains authoritative. Helpers accept
a snapshot, host-selected grapheme range and explicit source material:

- prefix matching preserves caller order and duplicates; matching is case
  sensitive or ASCII-insensitive as explicitly selected;
- common prefix returns the longest prefix ending at an extended-grapheme
  boundary;
- fuzzy matching is a Unicode-scalar subsequence policy ordered by earliest byte
  start, fewer byte gaps and stable caller position;
- path matching inspects one directory level, sorts insertion strings, appends a
  platform separator to directories, skips non-UTF-8 names and adds no quoting,
  expansion or shell semantics.

Queries are at most 4096 bytes. Sources/results remain within the existing 4096
candidate, 65,536-byte field and 4 MiB aggregate limits. Filesystem I/O and
validation errors reject the entire helper result.

## Large candidate navigation

Next/previous preserve wrapping. Page next/previous move by the visible window
and clamp; first/last select endpoints. The active state retains one set plus an
index and small viewport geometry. Navigation does not clone or scan payloads.
Resize may change the visible range but preserves selected index and candidate.
Sets above ten candidates add a textual selected/total and visible-range label;
the `>` selection marker remains available in styled and plain modes.

Qualification covers 1, 2, 8, 9, 100, 1000 and 4096 candidates, annotations,
Unicode and clipped labels through unit/model/PTY/fuzz paths.

## Autosuggestion contract

`Suggestion` is exactly one nonempty safe suffix, bounded to 4096 bytes and tied
to an end-of-draft `DraftRevision`. It is not `AnalysisPresentation::Hint`.
Delivery for an obsolete revision returns `Stale` with no mutation or terminal
effects. Acceptance inserts atomically through the established transaction path,
creates a fresh revision and is one undoable edit. Dismissal changes no canonical
state or revision. Static/history helpers inspect at most 4096 values and 4 MiB
of caller-owned source text, returning the suffix of the first caller-ordered
whole-draft prefix match.

Right at draft end is the conservative default acceptance gesture; explicit
suggestion actions are bindable. Precedence is reverse search, completion,
invalid diagnostics, suggestion, then informational hint. Hidden current state
may return after dismissal. Any canonical edit invalidates it; resize and finite
serialized output preserve it.

## Undo, ABI and ownership

Completion and suggestion acceptance both use the existing bounded delta undo
stack. Undo restores pre-accept bytes/cursor with a fresh identity; redo restores
the accepted edit with another fresh identity. No previous analysis identity can
become current. Helper discovery, rank meaning, scheduling, grammar and path
quoting remain host-owned.

C ABI 1 is unchanged: no keymap, helper or suggestion symbol was added. The
header, schema, symbol, layout and numeric comparison is replayed during closeout.

## Property, fuzz and native evidence

The deterministic model uses seed `15579347612220529`, 100,000 sequences and
128 operations per sequence across default/custom/reset mappings, editing,
history search, completion delivery/navigation, suggestion stale/accept/dismiss,
undo/redo, kill/yank and resize. The retained receipt lives under
`tools/completion/evidence/`.

The `surfaces` cargo-fuzz target combines decoder/keymap lookup, bounded helper
construction, completion navigation and suggestion lifecycle. Each input is at
most 4096 bytes. The final Linux ARM64 campaign runs at least 1800 aggregate CPU
seconds with ASan, inline coverage and comparison tracing. The final run recorded
1806.685 CPU seconds, 4189 inputs, corpus digest
`f31f3ca3f30aa2385c73f5b8668dcc5222ed56308558e02564ab143bf6c97fb6`
and zero findings. Its corpus is packed
deterministically and replayed by `tools/completion/corpus.py`. A failed initial
campaign was a harness compile defect (`DismissCompletion` survived the action
refoundation); it was repaired before the budgeted run and is retained in the
findings table.

The native workflow executes real PTYs at 20/40/80/132 columns in styled/plain
mode on Linux x86_64, Linux ARM64 and macOS ARM64. It covers custom binding,
suggestion display/accept/undo, a 1000-candidate page move, history-search output
restoration and exact terminal cleanup. Linux runs Valgrind; macOS uses native
`leaks` through an external PTY controller. Windows executes the portable public
model and corpus only; no terminal runtime is inferred.

## Performance and memory

Existing Q2 allocation/encoded-byte gates are replayed without replacing their
historical registration. The new large-menu range label is emitted only for sets
above ten candidates so the established small-menu deterministic gate remains
exact. New 5-control-batch, 31-repetition characterization covers default/custom/
maximum lookup, map construction, prefix 100/1000/4096, fuzzy 100/1000, Unicode
common prefix, path 100/1000, 4096-candidate install/navigation/resize, suggestion
present/stale/accept/dismiss and history helpers. Allocation measurement is a
separate instrumented build. Exact samples and summaries live under
`tools/completion/evidence/`; these are recorded workloads, not latency SLAs.
Representative medians/p95 are 32/32 ns for default and custom lookup, 64/64 ns
at the 128-entry map bound, 68.2/84.8 µs for 1000 prefix sources, 71.5/73.8 µs
for 1000 fuzzy sources, 24.0/24.9 µs for a 4096-candidate page move and
22.9/23.1 µs for suggestion acceptance on this ARM64 runner.

Retained bounds are 128 custom bindings, one existing completion set plus index,
one 4096-byte suggestion, and helper inputs/results bounded by protocol limits.
Lookup and suggestion dismissal allocate nothing; navigation clones no candidate
payload. Suggestion acceptance allocates only insertion/undo/render state required
by that canonical edit.

## Findings

| ID | Class | Finding | Repair / regression | Status |
| --- | --- | --- | --- | --- |
| A-C01 | New design defect | Early Engine-to-decoder type dependency violated the layer contract. | Keymap ownership moved to Interaction/Terminal; architecture dependency test passes. | Closed |
| H-C01 | Harness defect | Initial fuzz launch referenced removed internal `DismissCompletion`. | Harness maps public `CompletionAction::Dismiss`; failed launch excluded and full budget restarted. | Closed |
| H-C02 | Harness defect | The second macOS `leaks --atExit` pass exhausted a 120-second wrapper budget after the PTY oracle itself had passed. | The unchanged leak oracle retains a finite 300-second tool budget, matching the Linux memory pass; native qualification was restarted. | Closed |
| Q-C01 | Regression finding | Large-menu range prose changed the existing 10-candidate Q2 byte/allocation oracle. | Range prose is limited to sets above ten; unchanged small-menu gate replays exactly. | Closed |
| — | Product findings from final fuzz | No crash, panic, hang, invalid state, control injection, stale mutation or content loss. | Retained corpus replay. | Closed |

## Reproduction

```sh
cargo test --all-targets --locked
cargo run --locked --example configured_completion
cargo run --locked --release --manifest-path tools/hardening/Cargo.toml --bin completion-sequences -- 100000 15579347612220529
python3 tools/hardening/campaign.py --work WORK --cpu-seconds 1800 --workers 4 --targets surfaces
python3 tools/completion/corpus.py replay --archive tools/completion/surfaces-corpus.json.gz --work WORK
python3 tools/ergonomics/qualify_native.py --work WORK
python3 tools/hardening/deterministic.py
cargo run --locked --release --manifest-path tools/hardening/Cargo.toml --bin bench -- 31 5
```

## Closeout

The public crate was packaged as `replai-0.1.0.crate`: 99 files, 1,008,781
bytes compressed, SHA-256
`e4b4e73be42c8c272694ae0a246618fdf18aa057185ab19e6aecdbe5d8507026`.
Unpacked tests/rustdoc and `cargo publish --dry-run --locked` pass. A separate
sparse-registry consumer, with no workspace or path dependency, builds and runs
the custom keymap, prefix helper and autosuggestion under Rust 1.98.1. The
MSRV and current stable are the same qualified 1.98.1 toolchain on this date.

Native workflow `34770458423` passed Linux x86_64, Linux ARM64, macOS ARM64 and
Windows portable jobs at implementation carrier `ab34d0b78524329a0c2404c8e60a23f5ff5e92f8`.
The final carrier reruns CI and this native workflow after harness/evidence and
producer closure. `OUTPUT.LONG_LIVED.0` remains selected and unstarted. C ABI 1,
YAI, YVEX and private BOUNDARY remain untouched.
