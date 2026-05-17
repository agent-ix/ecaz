# Review Request: SPIRE CustomScan begin state

## Summary

This checkpoint closes the Phase 12b `BeginCustomScan` state-struct coverage
row at the decoded plan-part level.

Code checkpoint: `9c670b44837c01d6ae09ea1c55bfe4f822b8db0c`

The change extracts vector-order begin-state assignment into
`custom_scan_init_vector_order_limit_exec_state`, which `BeginCustomScan` now
calls after decoding the PostgreSQL plan. The unit test drives that helper with
minimal in-memory decoded plan parts and asserts:

- index OID and `top_k` planned output count are stored;
- query vector and tuple payload column descriptors are stored;
- output cursor and loaded/emitted progress state are reset to zero.

The test intentionally does not fake PostgreSQL tuple descriptors or expression
initialization outside a backend. The task tracker now states this limitation
explicitly.

## Scope Guard

This slice does not add to shrink-list files:

- `src/tests/remote_search.rs` remains deleted.
- `src/tests/mod.rs` is unchanged by this checkpoint.

The added test lives in `src/am/ec_spire/custom_scan/tests.rs`, now 474 lines.

## Validation

- `cargo fmt --check`
- `cargo test -p ecaz custom_scan_`

The focused test run passed 17 selected tests, including the new begin-state
unit test and the selected PG18 pgrx pg_test items. Raw logs and line counts are
in `artifacts/`.

## Reviewer Focus

- Confirm the helper-level boundary is acceptable for `BeginCustomScan` unit
  coverage.
- Confirm the refactor preserves DML begin-state initialization behavior.
- Confirm the tracker closure is honest about not constructing a raw in-memory
  PostgreSQL plan.
