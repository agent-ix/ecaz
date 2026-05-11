# Artifact Manifest: 30806 SPIRE CustomScan Index Eligibility

- head SHA: `9b41cfd9`
- packet/topic: `30806-spire-customscan-index-eligibility`
- lane / fixture / storage format / rerank mode: Rust unit + PG18 pgrx
  CustomScan status/eligibility fixture; storage format and rerank mode not
  applicable
- command used: `cargo test custom_scan --lib`
- timestamp: 2026-05-10T22:13:40-0700
- isolated one-index-per-table or shared-table surfaces: one-index-per-table
  PG18 fixture
- key result lines cited by `request.md`:
  - `test am::ec_spire::custom_scan::tests::custom_scan_status_reports_provider_name_and_disabled_execution ... ok`
  - `test am::ec_spire::custom_scan::tests::custom_scan_eligibility_counts_remote_available_placements ... ok`
  - `test tests::pg_test_ec_spire_custom_scan_status_registered_fail_closed ... ok`
  - `test tests::pg_test_ec_spire_custom_scan_index_eligibility_remote ... ok`
  - `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1601 filtered out`
- failed intermediate validation that shaped the implementation:
  - `ec_spire local heap tuple delivery requires remote_row_materialization before consuming 1 remote placement(s); first remote pid 2 is on node_id 2`
  - This came from the first attempt to base eligibility on
    `ec_spire_index_placement_snapshot(...)`; the landed implementation uses
    coordinator-fanout manifest loading instead.
- hygiene command: `git diff --check`
- hygiene result: passed with no output
