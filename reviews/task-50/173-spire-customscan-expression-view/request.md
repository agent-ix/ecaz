# Review Request: SPIRE CustomScan Expression View

## Scope

This packet reviews commit `f289973951ee0067e110c6e6fa70b49969ae479c` (`Add SPIRE CustomScan expression view`).

The slice adds `CustomScanExpr<'a>` in `src/am/ec_spire/custom_scan/cost_helpers.rs` and uses it for planner ORDER BY query extraction plus executor query decoding.

## Unsafe Burndown

Removed the old expression helper cluster:

- `custom_scan_expr_is_relation_var`
- `custom_scan_op_expr`
- `custom_scan_var_expr`
- `custom_scan_expr_node_tag`
- `custom_scan_expr_is_query_value`
- `custom_scan_query_values_from_const`

The replacement view validates the PostgreSQL node tag once, then exposes safe methods for:

- `op_expr()`
- `var()`
- `const_expr()`
- `param()`
- `is_relation_var()`
- `query_values_from_const()`
- `is_query_value()`

Unsafe ledger movement:

- previous packet 172 ledger: `1858`
- packet 173 ledger: `1853`
- net reduction: `5`

High-signal file counts from `make unsafe-block-count`:

- `src/am/ec_spire/custom_scan/plan_private.rs`: `22 -> 17`
- `src/am/ec_spire/custom_scan/cost_helpers.rs`: remains `22` because the new view centralizes remaining tag-checked casts there.
- `src/am/ec_spire/custom_scan/dml.rs`: remains `23` while its Const decoding call now goes through `CustomScanExpr`.

## Validation

Packet-local artifacts are under `reviews/task-50/173-spire-customscan-expression-view/artifacts/`.

Passed:

- `cargo-check-pg18-bench.log`
- `cargo-check-pg18-pg-test.log`
- `git-diff-check.log`
- `unsafe-block-count.log`
- `unsafe-ledger-generate.log`
- `unsafe-ledger-check.log`

## Reviewer Focus

Please check that `CustomScanExpr<'a>` correctly preserves the callback-local lifetime invariant and that its safe methods are only safe because they tag-check before every concrete PostgreSQL expression cast.
