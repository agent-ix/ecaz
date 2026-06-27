# Artifact Manifest

- Head SHA: `fe9ff76b7c5c1379aa7b9ffb213e0865708d3917`
- Task bucket: `reviews/task-122`
- Packet path: `reviews/task-122/003-spire-batched-materialization-prune`
- Timestamp: 2026-06-27
- Lane / fixture / storage format / rerank mode: SPIRE local unit validation; TurboQuant V2 leaf batched bounded candidate retention path.
- Isolation: unit tests only; no index/table benchmark run.
- Commands:
  - `cargo test -p ecaz --lib --no-default-features --features pg18 collect_scan_placement_diagnostics_reports_candidate_truncation`
  - `cargo test -p ecaz --lib --no-default-features --features pg18 collect_scan_placement_diagnostics`
- Key result lines:
  - `test am::ec_spire::scan::tests::collect_scan_placement_diagnostics_reports_candidate_truncation ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2219 filtered out; finished in 0.05s`
  - `test am::ec_spire::scan::tests::collect_scan_placement_diagnostics_reports_candidate_truncation ... ok`
  - `test am::ec_spire::scan::tests::collect_scan_placement_diagnostics_reports_boundary_dedupe_and_winners ... ok`
  - `test am::ec_spire::scan::tests::collect_scan_placement_diagnostics_skips_degraded_unavailable_leaf ... ok`
  - `test am::ec_spire::scan::tests::collect_scan_placement_diagnostics_counts_routed_store_rows ... ok`
  - `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2216 filtered out; finished in 0.04s`

