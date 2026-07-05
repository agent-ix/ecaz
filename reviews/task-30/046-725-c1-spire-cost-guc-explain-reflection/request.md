# Review Request: SPIRE Cost GUC EXPLAIN Reflection

- agent: coder1
- date: 2026-05-14
- code commit: `48e824894fb12d5d82f8551616b6384e766b109a`
- task rows: closes `12c.10.c`

## Summary

Added focused PG18 pg_test coverage for the remaining EXPLAIN-level half of
`12c.10.c`.

Packet `703` already pinned the pure cost model under all six SPIRE cost GUCs
but explicitly left EXPLAIN reflection open. This slice adds
`test_ec_spire_cost_gucs_reflect_in_explain_sql` in
`src/tests/spire_cost_tuning.rs`.

## What Changed

- Builds a 64-row SPIRE index fixture with `rerank_width = 0`, so
  `ec_spire.cost_rerank_multiplier` is active.
- Rewrites one leaf PID to a remote node and forces the planner to choose
  `Custom Scan (EcSpireDistributedScan)` for the ordered vector query.
- Captures the baseline CustomScan EXPLAIN total cost and the baseline
  `ec_spire_index_cost_snapshot(...).modeled_total_cost`.
- For each GUC, resets all SPIRE cost GUCs to default, sets that GUC to 2x its
  default, and asserts:
  - the CustomScan EXPLAIN total cost increases
  - the EXPLAIN cost delta tracks the modeled-cost delta within the expected
    text-EXPLAIN rounding tolerance

Covered GUCs:

- `ec_spire.cost_routing_dimension_scale`
- `ec_spire.cost_leaf_dimension_scale`
- `ec_spire.cost_index_page_scale`
- `ec_spire.cost_local_store_page_fanout_scale`
- `ec_spire.cost_storage_scoring_multiplier`
- `ec_spire.cost_rerank_multiplier`

## Test File Size Discipline

The touched test file remains well under the 2500-line target:

```text
231 src/tests/spire_cost_tuning.rs
```

This keeps cost-GUC coverage in its dedicated split file rather than growing
`custom_scan.rs` again.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/spire_cost_tuning.rs plan/tasks/task30-phase12c-spire-test-coverage.md
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_cost_gucs_reflect_in_explain_sql --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`, but exited
successfully.

Blocked before test execution:

```text
cargo pgrx test pg18 test_ec_spire_cost_gucs_reflect_in_explain_sql
```

Result:

```text
undefined symbol: BufferBlocks
```

The pg_test binary failed at local loader startup before the focused test body
could run.

## Review Focus

- Confirm comparing EXPLAIN total-cost delta to
  `ec_spire_index_cost_snapshot` is the right proportionality assertion for
  this row.
- Confirm the `0.02` tolerance is appropriate for PostgreSQL text EXPLAIN cost
  rounding.
