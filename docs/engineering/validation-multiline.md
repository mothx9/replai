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

Measurement and exact CI receipts are recorded at closure. Timing characterizes
this machine/workload, not other operating systems or a general speed ranking.
