# Review Request: EXPLAIN Index Node Boundary

Task: `plan/tasks/50-unsafe-burndown.md`

Code commit: `13a3b4b4493a4be3fbaae09702057dfe5aa12cc1`

## Summary

This slice reduces the PG18 EXPLAIN hook's direct unsafe surface in `src/am/common/explain.rs`.

- Deleted the standalone `explain_extension_id()` unsafe helper and resolved the extension id inside the existing EXPLAIN option state boundaries.
- Replaced the separate `explain_node_kind()` raw PlanState read and `explain_access_method_name()` raw IndexScanState relation read with one private `explain_index_scan_node()` boundary.
- The new boundary checks `PlanState.type_` before casting to `IndexScanState`, then returns private `NonNull` handles for the already-validated hook path.
- `explain_access_method_name()` now takes a `RelationHandle` instead of a raw `IndexScanState` pointer.

Unsafe count movement:

- `src/am/common/explain.rs`: 7 -> 5 direct `unsafe {` blocks.
- `src`: 1176 -> 1174 direct `unsafe {` blocks.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- `git diff --check` passed.
- `rustfmt --check src/am/common/explain.rs` passed, with stable rustfmt's known warnings for ignored nightly-only import grouping options.
- Raw-boundary guard found no public safe raw PG boundary helper signatures.
- Unsafe ledger generated and checked: `ledger covers 1174 current unsafe rows`.

Artifacts are in `reviews/task-50/376-explain-index-node-boundary/artifacts/`.
