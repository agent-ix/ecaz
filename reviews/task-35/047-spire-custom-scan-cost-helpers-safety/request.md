# Task 35 Packet 047: Spire Custom Scan Cost Helpers Safety

## Code Under Review

- Commit: `7952208943ad1a2e3937f1437b55885507e160e7`
- Scope: `src/am/ec_spire/custom_scan/cost_helpers.rs` and
  `scripts/unsafe_comment_baseline.txt`

## Summary

This slice documents the unsafe boundaries in SPIRE custom-scan cost and
planner-expression helper logic. It covers backend-local planner cost reads,
`PathTarget` width reads, `PlannerInfo` and `RelOptInfo` borrows, Query
`sortClause`/`targetList` traversal, NodeTag-dispatched `OpExpr` and `Var`
casts, and relation/query operand checks for order-by vector-distance
expressions.

Key safety boundaries documented:

- planner cost GUC and `cpu_tuple_cost` reads during path construction
- live `PathTarget` pointer used for projected tuple width
- live planner root/relation pointers and Query ownership during planning
- PostgreSQL-owned sort and target lists traversed with `PgList`
- NodeTag checks before casting `Expr` to `OpExpr` or `Var`
- two-operand order-by expression matching in normal and reverse operand order

## Baseline Accounting

- Global unsafe-comment baseline: `2564 -> 2545`
- `src/am/ec_spire/custom_scan/cost_helpers.rs`: `19 -> 0`

## Validation

- `artifacts/unsafe-baseline-report-before.log`: before-count report showing
  `2564` global entries and `19 src/am/ec_spire/custom_scan/cost_helpers.rs`.
- `artifacts/spire-custom-scan-cost-helpers-baseline-before.log`: pre-slice
  baseline entry list ending with `entries: 19`.
- `artifacts/unsafe-audit-before-baseline-update.log`: unsafe-comment audit
  completed with exit code 0 before baseline regeneration.
- `artifacts/unsafe-baseline-update.log` and
  `artifacts/unsafe-baseline-update-after-fmt.log`: regenerated baseline logs,
  ending at `2545` entries.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh`
  completed with exit code 0 and no diagnostic output.
- `artifacts/unsafe-baseline-report-after.log`: after-count report showing
  `2545` global entries and no remaining cost-helper entry.
- `artifacts/spire-custom-scan-cost-helpers-baseline-after.log`: after-count
  output showing `entries: 0`.
- `artifacts/unsafe-baseline-after-count.log`: after-count output showing
  `global: 2545` and `src/am/ec_spire/custom_scan/cost_helpers.rs: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` completed with exit code
  0 and no diagnostic output.
- `artifacts/cargo-fmt.log`: `cargo fmt --all` completed with the repository's
  existing stable-rustfmt warnings for unstable rustfmt options.
- `artifacts/cargo-check-pg18-bench.log`: cargo check completed successfully
  with known unrelated warnings.
- `artifacts/final-diff.patch`: final review diff.
