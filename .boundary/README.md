# Producer contract metadata

REPLAI owns these producer declarations. They describe generic capabilities,
surfaces, qualification and changes; they assign no consumer paths, commands or
migration work. Consumer profiles must be authored by their own repositories.

- [Current producer snapshot](producer.json): qualified runtime/API source
  `b9581102220364b94d2bcdef49f602308d24e6c9`, tree
  `f5ec77d774166605a578951e6ca827c29391b500`.
- [Reconstructed previous checkpoint](checkpoints/6365f84e12865871bf26ecf0d984b48213d81ebc.json):
  source `6365f84e12865871bf26ecf0d984b48213d81ebc`.
- [Consumer-neutral delta](deltas/6365f84-b958110.json): structured documents,
  responsive tables, status, composed prompts and explicit themes added on the
  native Rust surface. C ABI 1 remains unchanged. This is not an assigned
  consumer handoff.

The checkpoint manifest was authored retrospectively from that exact Git source
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
  --previous .boundary/checkpoints/6365f84e12865871bf26ecf0d984b48213d81ebc.json
```

Run from a clean REPLAI checkout with BOUNDARY installed separately. Validation
also checks watched current-HEAD blobs, so unreported public movement fails as
`MANIFEST_DRIFT`. `--revision` selects an exact historical-only source check.
No adjacent checkout or BOUNDARY installation is required by Cargo or REPLAI CI.
No executable source, API, consumer manifest or consumer pin changes with this
metadata publication. [The documentation map](../docs/README.md) retains normal
library contract authority; hashes detect movement but do not infer compatibility.
