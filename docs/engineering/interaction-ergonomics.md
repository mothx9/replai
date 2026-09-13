# Interaction ergonomics qualification

This dossier records INTERACTION.ERGONOMICS.0. It separates implemented
semantics from campaign evidence and leaves no later keymap/suggestion work
implied.

## Source identities

- Baseline: `aa2a0d747472b78110fcb90263babeff7f28d710`, tree
  `f3163d4c1db196338be92cd936f0dc3a9378629b`, runtime tree
  `12b9cd0e58ef8d2dfe6c1b91185fd21b05a7aa9e`.
- Initial implementation: `5de0965ac2bdf7d928693e4eeffb480695f92fd1`,
  runtime tree `b86ae9147e28ea5a4f49c2e2ea4ccd42accfd80f`.
- Revision-atomicity correction: `370fe9287c595884fd5ad336b51d73aa291b46b9`,
  runtime tree `d3fb692f8b4f495180eb4b667378f576c89e1bd2`.

The local campaign source is `65e3158123a7c90a5d03dd995601326d005672b7`,
tree `b5a5c77b146b865e0d5526ad9243fd5a071c1d1d`; its runtime tree is the final
`d3fb692f8b4f495180eb4b667378f576c89e1bd2`. Native CI receipts are linked
below when the evidence carrier is published.

## Architecture and contracts

The existing byte decoder, fixed keymap, normalized actions, engine and editor
were already separate. This wave expands the single crate-private `EditCommand`
vocabulary and keeps it private until I5 can audit configurable mapping as one
public contract. Public portable `Editor` methods expose the new mechanics;
active input continues through `Engine`, preserving revision invalidation and
render ownership. There is one canonical draft, cursor and `DraftRevision`.

`HistoryProvider` is a synchronous newest-first loading seam. The host invokes it
while constructing a bounded `HistorySearchSource`; REPLAI does not retain the
provider or call it from the terminal input path. The immutable view is limited
by entries and aggregate bytes. Existing admitted history needs no provider and
is searched before external entries. Persistence, I/O, admission, retention,
privacy and encryption remain host-owned.

Reverse search is literal, case-sensitive UTF-8 substring matching, newest first,
duplicate preserving and non-wrapping. Defaults are 1024 entries, 1 MiB retained
entry bytes and 4096 query bytes. Query and match are derived presentation.
Escape restores the exact canonical draft/cursor without a revision change;
Enter applies one whole-draft transaction at the match end with a fresh revision,
even when bytes equal the pre-search draft. Search selection excludes completion
selection; current analysis can return after dismissal. Resize and serialized
output preserve search state.

Word operations use `unicode-segmentation` 1.13.3 Unicode default word boundaries
(UAX #29), snapped outward to extended-grapheme boundaries. They are deterministic
and locale independent across punctuation, whitespace, tabs, LF, accented Latin,
combining marks, CJK, emoji, ZWJ, regional indicators and mixed scripts. Movement
allocates nothing; deletion is one atomic transaction.

Undo records store one delta: start, removed/inserted bytes, cursor before/after
and group class. Undo and redo each default to 256 entries and twice the draft
capacity in payload bytes. Contiguous typing and repeated Backspace/Delete group
to 4096 bytes. Cursor movement and atomic actions close a group. A divergent edit
clears redo; cursor-only movement preserves it. Clear and semantic draft end clear
both stacks; resource close/reopen preserves the unfinished draft stack. No record
stores a revision: undo and redo always advance to a fresh identity.

One editor-owned register supports word backward/forward and logical-line
start/end kill plus yank. Each kill replaces the register. The default is bounded
to min(draft capacity, 64 KiB); failed kill/yank is atomic. It survives retained
Editor lifecycle boundaries and has an explicit clear operation for the future
sensitive-input contract. It is not a clipboard or kill ring.

Fixed bindings added: Ctrl-R/Ctrl-S search, Alt-B/F word movement, Alt-D and
Alt-Backspace word deletion, Ctrl-W/U/K kill, Ctrl-Y yank and Ctrl-_ undo. Redo
and forward-word kill remain portable Editor operations without a fixed binding.
The decoder only reports Control/Meta physical keys. Bracketed paste remains one
literal atomic payload and cannot invoke shortcuts.

## Bounds and footprint

On Linux ARM64, `size_of::<Editor>()` changes from 144 to 176 bytes. Undo/redo
payloads, search view/match data and kill contents remain heap-backed and bounded
as described above. The query and search result view have no terminal-resource
lifetime.

## Findings ledger

| ID | Target/input | Finding | Classification | Repair/regression | Status |
| --- | --- | --- | --- | --- | --- |
| E-001 | Generated Unicode state, seed `7632459182231841`, state `4360685392116866849` | A Unicode word boundary could fall inside an extended grapheme and produce an invalid cursor. | Product defect in new ergonomics | Snap word boundaries with `GraphemeCursor`; mixed combining/ZWJ/RI regressions. | Closed |
| E-002 | 1 MiB local insertion allocation probe | Initial grouping could extend a huge initial transaction and reallocate its whole payload. | Product defect in new ergonomics | Cap a coalesced typing/deletion transaction at 4096 bytes; one-byte local undo payload regression. | Closed |
| E-003 | Identical history-search acceptance and exhausted history navigation | Identical accepted bytes could retain revision identity; navigation reset auxiliary undo state before the checked revision commit. | Product defect in new ergonomics | Always advance identical acceptance and order history revision checks before auxiliary mutation; unit regression. | Closed |
| H-001 | First fuzz invocation | A single-worker launch accumulated only 63.20 CPU-seconds before manual interruption to configure four accounted workers. | Harness execution, not product evidence | Excluded from campaign totals; exact four-worker command retained. | Closed |

## Qualification evidence

The final exact-source tables are populated only from retained receipts. The
canonical Linux command sequence is:

```sh
python3 tools/hardening/campaign.py --work /tmp/replai-ergonomics-fuzz \
  --cpu-seconds 1800 --workers 4 --targets editor
python3 tools/ergonomics/corpus.py replay --archive tools/ergonomics/editor-corpus.json.gz \
  --work /tmp/replai-ergonomics-replay
cargo run --locked --release --manifest-path tools/hardening/Cargo.toml \
  --bin ergonomics-sequences -- 100000 7632459182231841
python3 tools/ergonomics/qualify_native.py --work /tmp/replai-ergonomics-native
```

The Linux ARM64 feature campaign used nightly 1.100.0-nightly, cargo-fuzz
0.13.2 and its default address-sanitizer, inline-coverage and trace-comparison
instrumentation. Four accounted workers completed 1809.269397 CPU-seconds in
60 chunks. The initial 31-file corpus digest was
`fa7cba512c02a1ee4a565614adf55af27e8d5a60079ce07e8ecbf61d9857e8e4`;
the final 6104-file corpus digest is
`f13a67d0fdb166715e5956c81b7d3763f715dd86081312662c1e37848e80285b`.
Peak per-worker RSS was 530488 KiB and no finding remained. The deterministic
archive is 874819 bytes with SHA-256
`84c8bf5d43e16f0f94cacd9a1ecbfd79b730544634a26d10304f08fc64ee0969`.

The retained feature corpus and all five RELEASE.HARDENING.0 corpora replayed
cleanly. The latter contained 7442 C-ABI, 7296 editor, 6769 geometry, 1506
protocol and 7587 result inputs. The generated model completed 100000 sequences
of 128 operations from seed `7632459182231841`; final oracle state
`948307739503825185` matched, with 2212 KiB peak RSS. Exact machine-readable
receipts include the [fuzz campaign](../../tools/ergonomics/evidence/fuzz-linux-aarch64.json),
[corpus replay](../../tools/ergonomics/evidence/corpus-replay-linux-aarch64.json),
[semantic model](../../tools/ergonomics/evidence/semantic-sequences-linux-aarch64.json)
and [Q2 registration](../../tools/ergonomics/evidence/q2-linux-aarch64.json).

The native workflow executes the same portable model plus real PTY and Valgrind
or macOS `leaks` on Linux x86_64, Linux ARM64 and macOS ARM64. Windows executes
the portable model only. Its workflow-dispatch source is
[interaction-ergonomics.yml](../../.github/workflows/interaction-ergonomics.yml).

## Q2 and allocation policy

The immutable 32-workload registration remains historical evidence. Because the
new undo contract intentionally retains edit payload, allocation deltas are
rebaselined separately rather than passed against a zero-allocation append oracle.
Cursor and word movement remain zero-allocation. The updated exact gate requires a
warmed one-byte append to perform exactly one one-byte retained allocation. Sixteen
new ergonomics workloads cover words (ASCII/Unicode), undo/redo, replacement undo,
small/1000-entry history search, query refinement, kill and yank.

Alternating historical-control/current-source latency, the new 48-workload
registration, allocation measurements and known large-draft debt are retained in
the final evidence files. The prior 32-workload registration remains immutable.
The only independently repeated historical threshold exceedance is the
1000-input burst: median 44656 ns before and 61232 ns after. Its cause is the
selected undo contract (1118 versus 2127 allocation calls); encoded output
remains exactly 1042 bytes. It is an intentional bounded contract cost rather
than an unexplained regression.

Representative current registrations on Linux ARM64 are:

| Workload | Median / p95 (ns) | Allocations | Requested / retained bytes |
| --- | ---: | ---: | ---: |
| Word backward / forward, 1 KiB ASCII | 8512 / 8560; 6624 / 6640 | 0; 0 | 0 / 0 |
| Delete word backward / forward | 8544 / 8592; 6656 / 6688 | 1; 1 | 1024 / 1024 |
| Undo / redo local insertion | 48 / 64; 48 / 48 | 1; 0 | 320 / 320; 0 / 0 |
| Reverse search, small / 1000 entries | 23776 / 25312; 33712 / 36576 | 125; 1128 | 19293 / 4749; 112326 / 51702 |
| Kill / yank, 1 KiB | 8576 / 8624; 736 / 752 | 2; 2 | 2048 / 2048; 2048 / 1024 |
| One-byte append / cursor movement | 48 / 80; 32 / 48 | 1; 0 | 1 / 1; 0 / 0 |

The 64 KiB and 1 MiB edit probes register medians of 18912 ns and 240097 ns.
Their total retained deltas are 65634 and 1048674 bytes because the probe also
observes the canonical `String`; the undo delta itself is one byte. Existing
synchronous output and large multiline/prefix-layout debt remain outside this
wave and are not relabeled as solved.

## Compatibility and residual scope

`Editor::new(max_bytes, history_entries)` remains source compatible. C ABI 1 adds
no symbol, record, constant or numeric identity. Configurable keymaps, completion
helpers, autosuggestion, large candidate paging, streaming/output arbitration,
Windows runtime, sensitive input, U3 redesign, structural large-draft redesign,
high-level facade, runtime adapters and agent materials remain unimplemented.
COMPLETION.KEYMAP.SUGGESTION.0 is selected only after this dossier closes.
