# Producer contract metadata

REPLAI owns these producer declarations. They describe generic capabilities,
surfaces, qualification and changes; they assign no consumer paths, commands or
migration work. Consumer profiles must be authored by their own repositories.

- [Current producer snapshot](producer.json), generation 11: documentation/example
  source `fbe80ffc10b518186d7edac55fafda9f3bd226dc`, tree
  `08bb7c903f9ec86436b495d09b7ec29f10212806`.
  [12/12 CI jobs](https://github.com/mothx9/replai/actions/runs/34512238354) passed,
  including the new README snippet and real-PTY capture checks. Runtime source,
  public Rust/C behavior, dependencies and all capability definitions are identical
  to the predecessor; this snapshot refreshes qualification inputs only.
- [README-proof delta](deltas/indentation-readme-proof.json) records
  `QUALIFICATION_CHANGED` for the session reference host. The optional
  [capture method](../docs/development.md#readme-terminal-preview) verifies
  multiline submission, completion, host output and exact restoration from real
  terminal bytes. No feature, compatibility change or consumer migration is added.
- [Previous indentation snapshot](checkpoints/da16302c33cdce5ef40978e02e9d8dff045c93f9.json), generation 10: qualified source
  `da16302c33cdce5ef40978e02e9d8dff045c93f9`, tree
  `0e487a176b025ed5f7b239788d55eff95670ac6c`. The
  [continuation indentation correction](../docs/engineering/validation-multiline.md#continuation-indentation-follow-up)
  belongs to the common engine. [Exact-source CI](https://github.com/mothx9/replai/actions/runs/34509792610)
  covers real Linux/macOS interaction and memory, Windows portable logic and
  benchmark integrity. Local qualification passed all 22 content gates and the
  clean-worktree gate.
- [Indentation delta](deltas/analysis-presentation-indentation.json) records an
  intentional `BEHAVIOR_CHANGED` entry for `presentation.multiline_ux`, classified
  conservatively as `breaking`: validated continuation whitespace consumes Tab
  instead of emitting a completion request. Native signatures/types, direct mode,
  simple blocking and C ABI 1 are unchanged. No consumer migration is assigned.
- [Previous analysis-presentation snapshot](checkpoints/ff5de189aee2f4e5d9b75f35232af6eb719941d4.json), generation 9: source/document carrier
  `ff5de189aee2f4e5d9b75f35232af6eb719941d4`, tree
  `2ac2c730c6c84419fe63aa90862ec88b867de91c`. Library source is identical to
  implementation `e0ab06bc2e9111b41968842b459c51a68855aa13`; fixture carrier
  `27981f0dbb22305f96a07bb7e4c7fafbbf81e90b` passed
  [12/12 CI jobs](https://github.com/mothx9/replai/actions/runs/34502378013), including
  native Linux/macOS PTYs and memory, portable Windows and benchmark integrity.
  The [I2 dossier](../docs/engineering/analysis-presentation.md) separates source,
  qualification, performance and deliberately bounded hint behavior.
- [Analysis-presentation delta](deltas/validation-analysis-presentation.json) adds
  `analysis.hints_highlight`: host-derived revision-bound spans, safe non-canonical
  hints, stale silence and plain degradation. It adds no consumer migration,
  C entry point, parser or scheduling authority. C ABI 1 remains unchanged.
- [Previous validation snapshot](checkpoints/0abed4fa75a5684005c87df3bb36e2f997025d00.json), generation 8: source/document carrier
  `0abed4fa75a5684005c87df3bb36e2f997025d00`, tree
  `254a0c8f254ef79bd45313cba1d4c30709be566b`. Runtime is identical to qualified
  implementation `1da4dbf9162a2fea4d267e1aa473859ee64ae290`;
  [CI](https://github.com/mothx9/replai/actions/runs/34494834221) passed 12/12 jobs,
  including native Linux/macOS PTYs/memory and portable Windows. The
  [I3/U2 dossier](../docs/engineering/validation-multiline.md) records full local
  qualification, clean source, measurements and limitations.
- [Validation/multiline delta](deltas/completion-validation.json) adds
  `analysis.validation` and `presentation.multiline_ux`. It explicitly records
  native source compatibility changes for `embedding.session` and
  `embedding.driven`: exhaustive Rust Event matches must handle the new opt-in
  SubmissionRequested variant. Default direct submission and C ABI 1 are
  unchanged. No consumer migration or repin is assigned.
- [Previous completion snapshot](checkpoints/0fef74edd5d4d6d536560036473980a41e1da3f3.json), generation 7: qualified source/document
  `0fef74edd5d4d6d536560036473980a41e1da3f3`, tree
  `e86c5b385e993f124c7901677d6f6593b0f3fd71`. The [I1/U1 dossier](../docs/engineering/completion-contract.md)
  separates implementation, measurement and qualification sources.
  [Native/portable CI](https://github.com/mothx9/replai/actions/runs/34485342480)
  passed 12/12 jobs; complete local qualification and its clean-worktree gate passed.
- [Completion contract delta](deltas/analysis-completion.json) adds
  `completion.candidates` and `presentation.completion_ux`: host-owned discovery,
  bounded revision-bound delivery, selection/acceptance and safe responsive
  presentation through native Rust. C ABI 1 stays unchanged. No consumer migration
  or repin is assigned.
- [Previous analysis snapshot](checkpoints/aa137e881065f0a0615fa8ae03c84c89f052bea3.json)
  retains generation 6 and its exact source/fingerprints.
- [Analysis protocol delta](deltas/terminal-analysis.json): `analysis.revisions`
  adds shared immutable draft snapshots and atomic stale-result refusal through
  native Rust. C ABI 1 remains unchanged and synchronous. The Linux restoration
  observer now checks the calling thread's signal mask. No consumer action is assigned.
- [Previous terminal-capability snapshot](checkpoints/36fefcaebba8489d672eb1c5c2b4f5cb1d939d86.json)
  retains generation 5 and its exact source/fingerprints.
- [Terminal capability delta](deltas/embedding-terminal-capabilities.json): unified
  admission, portable capability snapshot and deterministic width query. C ABI 1
  remains exact; its qualification is refreshed, including final-event observation.
  These native additions assign no consumer migration.
- [Previous embedding producer snapshot](checkpoints/7f144d9e98a12bc1d3bbfc09bbcda547cecf2bd5.json)
  preserves generation 4 and its exact source/fingerprints.
- [Structured-presentation producer snapshot](checkpoints/b9581102220364b94d2bcdef49f602308d24e6c9.json):
  the immutable generation-2 structured-presentation contract.
- [Embedding consumer-neutral delta](deltas/b958110-embedding.json): native blocking,
  explicit session, driven scheduling, borrowed readiness and terminal admission.
  Exhaustive native Rust Error matches must account for CapabilityMismatch.
  C ABI 1 remains unchanged; platform qualification is refreshed. No migration
  is assigned to any consumer.
- [Previous embedding snapshot](checkpoints/3c0b14162cd322017bb718b71cc4b433c6100476.json)
  and [historical qualification-only delta](deltas/embedding-observation.json) retain
  the real-PTY observation correction. No runtime, Rust API or C ABI changed
  between these two snapshots.
- Historical [6365 checkpoint](checkpoints/6365f84e12865871bf26ecf0d984b48213d81ebc.json)
  and [6365 → b958 presentation delta](deltas/6365f84-b958110.json) remain unchanged.

The 6365 checkpoint manifest was authored retrospectively from that exact Git source
and its retained qualification; the file did not exist at the checkpoint.
Snapshot IDs are deterministic SHA-256 identities of the contract record, not
Git revisions. The commit publishing this metadata is a **carrier**, distinct
from the qualified implementation source it describes. This avoids a manifest
trying to contain the hash of its own enclosing Git commit. Metadata-only
carrier evolution does not imply a new runtime contract or a consumer repin.

[BOUNDARY](https://github.com/mothx9/boundary) schema version 1 defines these
records. It validates the exact Git revision/tree and hashes watched API,
behavior, ABI and documentation inputs. Its external validation command is:

```sh
boundary validate .boundary/producer.json --repo . \
  --previous .boundary/checkpoints/da16302c33cdce5ef40978e02e9d8dff045c93f9.json
```

Run from a clean REPLAI checkout with BOUNDARY installed separately. Validation
also checks watched current-HEAD blobs, so unreported public movement fails as
`MANIFEST_DRIFT`. `--revision` selects an exact historical-only source check.
No adjacent checkout or BOUNDARY installation is required by Cargo or REPLAI CI.
No executable source, API, consumer manifest or consumer pin changes with this
metadata publication. [The documentation map](../docs/README.md) retains normal
library contract authority; hashes detect movement but do not infer compatibility.
