# Review Request: SPIRE DML Frontdoor Raw Boundary

## Summary

This checkpoint addresses the soundness audit's SPIRE DML frontdoor finding.

Changes:

- Marked baserel primitive-plan helpers unsafe because they accept raw
  `PlannerInfo` / `RelOptInfo` pointers from PostgreSQL planner callbacks.
- Marked executor parameter evaluation helpers unsafe because they accept raw
  `ParamListInfo` pointers.
- Added safety acknowledgments in CustomScan planning and DML frontdoor tests
  where those raw planner/executor pointers are known to be valid.

## Validation

- Pass: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Artifact: `artifacts/cargo-check-pg18-bench.log`
  - Note: pre-existing unused-import warning in `src/am/mod.rs`.
- Pass: `git diff --check`
  - Artifact: `artifacts/git-diff-check.log`
- Pass: `make unsafe-block-count`
  - Artifact: `artifacts/unsafe-block-count.log`
  - Summed unsafe count: 1641.

## Reviewer Focus

Please check that the planner and `ParamListInfo` validity contracts are now
visible at the helper boundary and that the added call-site safety comments are
attached to active planner/executor scopes.
