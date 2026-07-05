# SPIRE Materialization Catalog Cleanup

## Scope

This packet removes the superseded SPIRE coordinator-side remote row
materialization catalog surface after the CustomScan read path and ADR-069
write path landed.

Code/docs commit under review:

- `e716c07a` Drop SPIRE remote row materialization catalog

Changes:

- Removed `ec_spire_remote_row_materialization` from `sql/bootstrap.sql`.
- Removed the public pgrx externs
  `ec_spire_register_remote_row_materialization(...)` and
  `ec_spire_remote_row_materialization_catalog(...)`.
- Added `ecaz--0.1.1--0.1.2.sql` with
  `DROP TABLE IF EXISTS ec_spire_remote_row_materialization`.
- Removed the catalog-backed AM materialization provider and the PG18 fixtures
  that depended on explicit materialization registration.
- Kept remote catalog cleanup function signatures stable, but the retired
  row-materialization cleanup counters now report `0` and no longer read or
  delete the dropped table.
- Updated the Phase 11 task file cleanup checklist to track this packet.

## Validation

Packet-local logs are in `artifacts/`:

- `cargo test custom_scan --lib`
- `cargo test remote_catalog --lib`
- `cargo fmt --check`
- `git diff --check`

## Review Focus

- Confirm the cleanup removes the vestigial catalog/register surface without
  breaking the surviving remote catalog diagnostics.
- Confirm preserving the cleanup result columns with zero values is acceptable
  for operator-facing compatibility.
- Confirm leaving the historical `ecaz--0.1.0--0.1.1.sql` table creation in
  place, paired with the new `ecaz--0.1.1--0.1.2.sql` drop migration, is the
  right migration shape for this branch.
