# SPIRE Manifest Persist Epoch Guard

## Scope

Task 30 SPIRE Phase 7 manifest persistence now rechecks the current active
epoch immediately before durable catalog writes.

Code checkpoint: `9bb3d383` (`Guard SPIRE manifest persist epoch`)

## Changes

- Added a narrow root-control active-epoch helper for `ec_spire`.
- `ec_spire_persist_remote_epoch_manifest(...)` now keeps the index relation
  open through the SPI write block and compares the current active epoch with
  the manifest summary epoch before inserting/updating catalog rows.
- If the epoch advanced, persistence fails closed with a retryable error instead
  of returning success for a stale manifest.
- Updated the Phase 7 task note.

## Validation

- `cargo fmt`
- `cargo test --no-default-features --features "pg18 pg_test" remote_epoch_manifest_persist`
- `git diff --check`

The first validation attempt used
`ec_spire_index_active_snapshot_diagnostics(...)` for the recheck and failed
because that diagnostic path validates remote node placement state. The final
patch uses a root-control-only helper; the persistence filter then passed.

## Review Focus

- Whether the root-control recheck is the right mitigation for the stale-success
  race called out in the manifest persistence review feedback.
- Whether keeping the relation open through the SPI write block is acceptable
  for this mutating SQL entrypoint.
