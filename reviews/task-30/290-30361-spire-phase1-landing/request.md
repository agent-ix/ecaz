# SPIRE Phase 1 Landing

## Checkpoint

- Code commit: `7fc2eb9c`
  (`Cover SPIRE scan sanity status labels`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: final Phase 1 landing packet for local single-store, single-level
  `ec_spire`

## Summary

This packet closes the Phase 1 review-packet checklist item for the SPIRE
single-level foundation. The implemented Phase 1 boundary is:

- `ec_spire` exposes local single-store, single-level partition-object storage.
- TurboQuant and RaBitQ assignment payloads are scannable.
- Populated PQ-FastScan SPIRE indexes remain deferred until the grouped-PQ
  metadata/scorer slice lands; empty PQ-FastScan scans remain safe because
  there are no assignments to score.
- Strict consistency remains the local single-store default.
- Replacement epochs, insert/delete deltas, retired-manifest residue, root
  routing diagnostics, and scan root-control refresh have focused coverage.
- Remote placement, replicas, boundary-replica promotion, physical page
  reclamation, and populated PQ-FastScan scans remain future work.

The small recall/latency sanity row remains a configuration diagnostic rather
than a measured benchmark claim. Packet `30305-spire-scan-sanity-diagnostics`
already validated the SQL surface: approximate scans report
`recall_sanity_status = 'approximate_leaf_coverage'`, and exact leaf plus
full-frontier sanity checks report
`recall_sanity_status = 'exact_leaf_and_frontier_coverage'` with
`latency_risk_status = 'full_scan'`. This packet adds packet-local unit logs
that lock those labels at the helper boundary.

## Artifacts

- `artifacts/scan_sanity_status_unit.log`
  - `test am::ec_spire::tests::scan_sanity_status_reports_empty_approximate_and_full_scan ... ok`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1129 filtered out`
- `artifacts/epoch_bundle_residue_unit.log`
  - `test am::ec_spire::tests::epoch_snapshot_bundle_residue_keeps_previous_root_manifest_authoritative ... ok`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1129 filtered out`
- `artifacts/scan_root_cache_unit.log`
  - `test am::ec_spire::scan::tests::scan_opaque_refreshes_root_control_on_every_rescan_observation ... ok`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1129 filtered out`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test scan_sanity_status_reports_empty_approximate_and_full_scan --no-default-features --features pg18`
  - Passed; see `artifacts/scan_sanity_status_unit.log`.
- `cargo test epoch_snapshot_bundle_residue_keeps_previous_root_manifest_authoritative --no-default-features --features pg18`
  - Passed; see `artifacts/epoch_bundle_residue_unit.log`.
- `cargo test scan_opaque_refreshes_root_control_on_every_rescan_observation --no-default-features --features pg18`
  - Passed; see `artifacts/scan_root_cache_unit.log`.
- `git diff --check`
  - Clean.

## Notes

- This packet does not claim measured recall, throughput, or latency.
- A current-sandbox rerun of the pgrx SQL test path would attempt to write the
  pgrx install tree outside the repo writable root, so this packet keeps the
  new packet-local evidence to pure Rust tests and points reviewers to the
  earlier `30305` SQL validation for the scan-sanity row shape.
- Review requests remain open for outside reviewer response.
