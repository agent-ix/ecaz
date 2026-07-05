# Review Request: SPIRE Page Relation Boundary

## Summary

This checkpoint addresses the soundness audit's SPIRE page helper cluster.

Changes:

- Marked `SpirePageRelation::new` and relation-backed page helpers unsafe
  because callers must pass live SPIRE relations.
- Marked locked-page tuple visitor helpers unsafe because callers must hold the
  page lock/pin and supply valid page bounds.
- Added safety acknowledgments at publish, CustomScan, diagnostics, coordinator,
  storage, and vacuum call sites.

## Validation

- Pass: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Artifact: `artifacts/cargo-check-pg18-bench.log`
  - Note: pre-existing unused-import warning in `src/am/mod.rs`.
- Pass: `git diff --check`
  - Artifact: `artifacts/git-diff-check.log`
- Pass: `make unsafe-block-count`
  - Artifact: `artifacts/unsafe-block-count.log`
  - Summed unsafe count: 1675.

## Reviewer Focus

Please check that relation liveness and page lock/pin preconditions are now
acknowledged at the right caller boundaries, especially around manifest publish
and locked-page tuple visitation.
