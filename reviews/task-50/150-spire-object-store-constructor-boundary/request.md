# Review Request: SPIRE Object Store Constructor Boundary

## Summary

This checkpoint addresses the soundness audit's SPIRE store cluster for
relation-backed object-store constructors.

Changes:

- Marked `SpireRelationObjectStore::for_index_relation` and
  `for_store_relation_id` unsafe because the stored raw relation must remain
  live for all store operations.
- Marked `SpireRelationObjectStoreSet::for_index_relation_and_config` and
  `for_index_relation_and_placements` unsafe for the same live-relation
  precondition.
- Added explicit safety acknowledgments at SPIRE coordinator, production scan,
  and vacuum call sites that pass through live index relations.

## Validation

- Pass: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Artifact: `artifacts/cargo-check-pg18-bench.log`
  - Note: pre-existing unused-import warning in `src/am/mod.rs`.
- Pass: `git diff --check`
  - Artifact: `artifacts/git-diff-check.log`
- Pass: `make unsafe-block-count`
  - Artifact: `artifacts/unsafe-block-count.log`
  - Summed unsafe count: 1634.

## Reviewer Focus

Please check that the constructor boundary is the right layer for the live
relation precondition and that the added call-site acknowledgments are attached
to genuinely live SPIRE relation scopes.
