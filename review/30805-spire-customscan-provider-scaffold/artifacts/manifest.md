# Artifact Manifest: 30805 SPIRE CustomScan Provider Scaffold

- head SHA: `e6a6a806`
- packet/topic: `30805-spire-customscan-provider-scaffold`
- lane / fixture / storage format / rerank mode: Rust unit + PG18 pgrx status
  fixture; storage format and rerank mode not applicable
- command used: `cargo test custom_scan --lib`
- timestamp: 2026-05-10T22:05:18-0700
- isolated one-index-per-table or shared-table surfaces: not applicable
- key result lines cited by `request.md`:
  - `test am::ec_spire::custom_scan::tests::custom_scan_status_reports_provider_name_and_disabled_execution ... ok`
  - `test tests::pg_test_ec_spire_custom_scan_status_registered_fail_closed ... ok`
  - `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1601 filtered out`
- hygiene command: `git diff --check`
- hygiene result: passed with no output
