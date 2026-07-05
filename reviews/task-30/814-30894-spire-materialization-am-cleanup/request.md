# SPIRE Materialization AM Cleanup

## Scope

This packet removes the remaining superseded row-materialization AM cleanup
paths after packet `30892` removed the catalog/register surface.

Code/docs commit under review:

- `74f511d4` Remove SPIRE row materialization AM cleanup paths

Changes:

- Removed the row-materialization and mirror-sync SQL contract functions.
- Removed their operator-entrypoint contract rows.
- Removed the catalog-backed AM materialization provider and its tests.
- Removed the materialized-heap owner from AM tuple delivery.
- Changed remote-origin AM cursor diagnostics from the superseded
  `requires_remote_row_materialization` / `remote_row_materialization` blocker
  to the CustomScan-oriented `requires_custom_scan_tuple_delivery` /
  `custom_scan_tuple_delivery`.
- Updated the Phase 11 cleanup checklist to mark this cleanup complete.

## Validation

Packet-local logs are in `artifacts/`:

- `cargo test custom_scan --lib`
- `cargo test remote_search_final_contract --lib`
- `cargo test production_fault_matrix --lib`
- `cargo test phase7_policy_contracts --lib`
- `cargo test production_scan_am --lib`
- `cargo test production_scan_result_stream_am_outputs --lib`
- `cargo test local_heap_delivery_gate --lib`
- `cargo fmt --check`
- `git diff --check`

## Review Focus

- Confirm no runtime or SQL-visible code under `src/` still exposes the
  superseded row-materialization contract/status symbols.
- Confirm remote-origin rows are now directed to CustomScan tuple delivery
  rather than the AM cursor/materialization model.
- Confirm the remaining `row_materialization_*` cleanup columns in catalog
  cleanup diagnostics are acceptable compatibility shims returning zero.
