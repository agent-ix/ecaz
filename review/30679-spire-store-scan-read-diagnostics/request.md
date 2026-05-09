# Review Request: SPIRE Store Scan Read Diagnostics

Code checkpoint: `f30f6eaf` (`Expose SPIRE per-store scan read diagnostics`)

## Scope

- Advances Phase 10.4 by expanding per-store scan diagnostics from
  candidate/read counts to explicit route and prefetch counts.
- Adds `route_count`, `leaf_route_count`, `delta_route_count`, and
  `prefetched_object_count` to `SpireStoreScanDiagnostics`.
- Wires those counters through the SQL-visible
  `ec_spire_index_scan_placement_snapshot` table function.
- Keeps existing candidate winner/dedupe/truncation and scanned leaf/delta
  counters unchanged.
- Marks the Phase 10.4 per-store route/candidate/read diagnostics checklist
  item complete.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 collect_scan_placement_diagnostics_counts_routed_store_rows --lib`
- `cargo test --no-default-features --features pg18 group_leaf_and_delta_reads_by_local_store --lib`
- `cargo test --no-default-features --features pg18 prefetch_store_object_read_groups --lib`
- `cargo test --no-default-features --features pg18 collect_scan_placement_diagnostics --lib`
- `cargo test --no-default-features --features pg18 test_ec_spire_scan_placement_snapshot_sql --lib`

## Review Focus

- Confirm route, prefetch, and scanned/read counters are recorded at distinct
  points in the pipeline.
- Confirm dropped unselected delta routes remain visible without being counted
  as selected routes.
- Confirm adding SQL-visible columns is acceptable for this diagnostic surface.
