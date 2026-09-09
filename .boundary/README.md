# Producer contract metadata

REPLAI owns these producer declarations. They describe generic capabilities,
surfaces, qualification and changes; they assign no consumer paths, commands or
migration work. Consumer profiles must be authored by their own repositories.

- [Current producer snapshot](producer.json), generation 3: qualified source
  `3c0b14162cd322017bb718b71cc4b433c6100476`, tree
  `4b035dbaedf0387961526568a551cf76e0cd617e`. The source includes the embedding implementation and
  its qualification/documentation carrier; the engine implementation is identified
  separately in the [embedding dossier](../docs/engineering/embedding.md).
- [Previous producer snapshot](checkpoints/b9581102220364b94d2bcdef49f602308d24e6c9.json):
  the immutable generation-2 structured-presentation contract.
- [Current consumer-neutral delta](deltas/b958110-embedding.json): native blocking,
  explicit session, driven scheduling, borrowed readiness and terminal admission.
  Exhaustive native Rust Error matches must account for CapabilityMismatch.
  C ABI 1 remains unchanged; platform qualification is refreshed. No migration
  is assigned to any consumer.
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
  --previous .boundary/checkpoints/b9581102220364b94d2bcdef49f602308d24e6c9.json
```

Run from a clean REPLAI checkout with BOUNDARY installed separately. Validation
also checks watched current-HEAD blobs, so unreported public movement fails as
`MANIFEST_DRIFT`. `--revision` selects an exact historical-only source check.
No adjacent checkout or BOUNDARY installation is required by Cargo or REPLAI CI.
No executable source, API, consumer manifest or consumer pin changes with this
metadata publication. [The documentation map](../docs/README.md) retains normal
library contract authority; hashes detect movement but do not infer compatibility.
