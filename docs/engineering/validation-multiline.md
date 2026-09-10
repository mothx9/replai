# Revision-bound submission and multiline qualification

This dossier owns the bounded I3/U2 evidence. [Interaction](../interaction.md)
owns the public contract, [use cases](../use-cases.md) owns runnable integration
recipes, and [ROADMAP](../../ROADMAP.md) owns maturity. No later wave is included.

## Source and design

Baseline: `3fffb40abc7f5d0ac41db3e6cde09413b018cfee`, tree
`334a94d9322ca080437c150070d29fa3ed8b1af8`. The baseline was clean and equal to
origin/master before mutation. Runtime, tests and docs share the existing master
worktree. No consumer or BOUNDARY repository is an input to this implementation.

One Engine gains optional boxed ValidationState. Direct mode allocates no
validation state. The Editor still owns the only DraftRevision; snapshots share
immutable text through the existing Arc. SubmissionRequested carries one coherent
snapshot. The state retains only a pending revision and the current bounded
explanations, never parser results or historic diagnostics.

Repeated Enter at the same unchanged revision coalesces the pending attempt.
The first successfully delivered decision consumes it. Invalid consumes without
changing revision; another Enter is needed to try again. No separate request
counter is added: all decisions about identical draft state are equally eligible
while it has a pending request. Hosts filter superseded jobs/context themselves,
including older decisions delivered after another Enter at that same revision.
Close clears pending authority even if bytes/revision survive; submit/interrupt/EOF
end the semantic draft under I0. Complete cannot bypass an Enter request.

Complete uses the existing Engine finish and Terminal cleanup path. Incomplete
inserts one LF through Editor and advances revision once. Capacity failure retains
the request and text. Invalid validates every range before presentation and retains
the exact draft. Stale comparison happens first, including after terminal closure,
and returns no Effects, allocation for snapshots, or terminal operation.

Up/Down in validated mode move between logical LF lines before history at edges.
Standalone Editor::line_up/line_down use deterministic width and four-cell tab
stops, clamp short lines, and preserve grapheme boundaries. Soft-wrap navigation
uses Left/Right; full keymaps and remembered preferred columns are deferred.
Direct mode and C ABI 1 keep their existing key/submit behavior.

Diagnostics share Frame and Renderer with editing and completion. Five temporary
rows maximum reserve bounded draft space. `! Invalid input` is always textual;
messages are grapheme-ellipsized, with a count and optional byte-range labels.
Full retained messages are inspectable through diagnostics(). They are not syntax
highlights. Escape hides them; successful edits invalidate them; output/resize
preserve them. Installing a valid completion set explicitly replaces diagnostics;
acceptance takes precedence over validation on Enter.

Rejected alternatives: callbacks would acquire analysis execution; an async-first
validator would acquire scheduling; draft-preview mutations would invalidate I0;
a second diagnostics renderer would split terminal ownership; force-submit would
bypass the host's explicit policy. None is introduced.

## Executable oracles

- `tests/validation.rs`: portable bounds, control rejection and vertical geometry.
- `src/validation_tests.rs`: deterministic dispositions, lifecycle, stale silence,
  malformed ranges, capacity rejection, completion/history, 10,000 generated
  transitions and substantial Unicode drafts through 16,000 lines.
- `tools/validation_pty.py`: the same Linux/macOS external reactor and synchronous
  example; real termios, paste, resize, output, delayed results, completion,
  10/100/1000-line paste and repeated FD/close/reopen behavior. Styled and NO_COLOR
  semantic screens are compared. Native memory modes are mandatory qualification.
- Existing I0/I1/F2/embedding and C/static/shared/record/symbol gates remain enabled.
- Windows runs the portable structures/model/layout; no runtime backend is claimed.

## Measurements and publication evidence

Implementation: `1da4dbf9162a2fea4d267e1aa473859ee64ae290`, tree
`abe89cbd73616282b72c453668d8db50ae52a650`. Its
[CI run](https://github.com/mothx9/replai/actions/runs/34494834221) passed 12/12 jobs.
Full local `tools/qualify.py` passed 20 executable gates plus the clean-worktree
gate on that published source. Final carrier identities are recoverable from Git;
subsequent documentation/metadata commits do not replace this implementation identity.

[Native receipts](validation-multiline/platforms.json) retain Linux/macOS styled
and NO_COLOR real-PTY results, including delayed results, Unicode, substantial
paste, mid-draft navigation/output, exact termios and 12 stable lifecycle cycles.
Linux Valgrind reports zero errors in both profiles; macOS native `leaks --atExit`
reports zero leaks. Windows native portable tests and benchmark integrity pass;
no Windows terminal runtime is claimed. C ABI 1 schema, generated records/header,
symbols and static/shared/C++ qualification remain exact. The binding adds only
an unreachable match arm: ABI 1 cannot enable validated submission.

[Before](validation-multiline/before.json) and
[after](validation-multiline/after.json) retain 6,566 and 7,000 validated
characterization rows. All 3,277 matched measured allocation profiles are identical.
The after run records a dirty development source based on the baseline HEAD:
every recorded file hash was independently compared with the committed
implementation above and matched. No dirty snapshot is represented as a clean
published source. Timing and allocation modes use separate binaries.

Machine: spark-7c3d, Linux aarch64. These are exact workload observations, not
macOS timings or a general speed ranking. Timer granularity is 16 ns.

| Existing 1 KiB ASCII editor operation | Before µs | After µs |
| --- | --- | --- |
| Append | 0.064 | 0.064 |
| Left | 0.032 | 0.032 |
| Right | 0.064 | 0.064 |
| Completion short replacement | 0.048 | 0.048 |
| History up | 0.048 | 0.048 |

The [alternating PTY comparison](validation-multiline/paired.json) keeps the exact
1,000-byte ASCII + Left + X + Enter workload. Median 247.745 → 251.456 µs;
p95 370.880 → 368.225 µs; median absolute deviations 20.495 and 17.136 µs.
Both emit 1,054 terminal bytes. The 1.5% median difference is below observed
dispersion; this is evidence of no material regression at this scope, not a
universal latency bound. The same receipt covers visible append, completion,
Ctrl-L, resize, multiline paste and 64 KiB backpressure. Compatibility resize
still observes its approximately 100 ms wait; driven resize has no periodic floor.

New Engine operations below include layout/render transition, exclude acquisition,
transport syscalls and host parsing, use 80 columns with cursor at draft end,
and retain 63 measured samples per operation. Each fixture line is 69 bytes.

| Lines / bytes | Visible append µs | Up µs | Incomplete µs | Invalid µs | Complete µs |
| --- | --- | --- | --- | --- | --- |
| 10 / 690 | 16.288 | 16.208 | 16.976 | 16.848 | 0.256 |
| 100 / 6,900 | 147.648 | 147.600 | 147.888 | 146.384 | 0.496 |
| 1,000 / 69,000 | 1,442.754 | 1,443.091 | 1,442.882 | 1,427.666 | 1.040 |

At 1,000 lines the encoded mutations are: append 1 byte, Up 73, Incomplete 21,
Invalid 108, Complete 7, resize 1,846 and external output 1,875. Actual transport
writes are separately observed by the native PTY fixture; logical mutation counts
are not syscalls. Large drafts retain a bounded viewport, not all historical
screen rows. The inherited fresh geometry traversal still scans the prefix to
the cursor: layout CPU can grow with draft length even when emitted damage is
one byte. This is deliberate measured debt, not a claim of constant-time layout.

Editor remains 144 bytes and Interaction remains 688 bytes on this target.
ValidationResult is 48 bytes; Diagnostic is 40 bytes. Opt-in ValidationState is a
64-byte allocation; Direct has no validation allocation. Maximum retained
messages are 65,536 bytes plus at most 32 diagnostic records and bounded Vec/state
metadata. The pending request retains only a revision; the snapshot text remains
host-owned. Full per-operation timing, allocations, encoded bytes and beginning/
middle/end positions are preserved in the after receipt.

Deliberate limits: native Rust only; fixed logical-line navigation with short-line
clamping, not configurable modes; diagnostic byte-range labels, not editor syntax
highlights; bounded/ellipsized messages with full inspection, not an IDE panel;
serialized current output, not concurrent writers. No later boundary is promoted.
