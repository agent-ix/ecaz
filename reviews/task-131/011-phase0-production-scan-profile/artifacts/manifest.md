# Task 131 Packet 011 Artifact Manifest

- head SHA: `7a76413f50b49dcfd356cc8d7e7129f86b4778ca`
- task bucket: `reviews/task-131/`
- packet path: `reviews/task-131/011-phase0-production-scan-profile/`
- timestamp: `2026-07-01T08:38:33-07:00`
- isolated one-index-per-table or shared-table surface: not applicable; focused build/unit validation only
- lane / fixture / storage format / rerank mode: not applicable

## Artifacts

### `cargo-check-lib.log`

- command: `cargo check --lib`
- result: passed
- key lines:
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 0.14s`

### `cargo-test-ecaz-cli-production-scan-profile.log`

- command: `cargo test -p ecaz-cli spire_pipeline_renders_production_scan_profile`
- result: passed
- key lines:
  - `test commands::bench::spire_pipeline::tests::spire_pipeline_renders_production_scan_profile ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 419 filtered out`

### `cargo-test-ecaz-cli-sql-contracts.log`

- command: `cargo test -p ecaz-cli spire_pipeline_sql_uses_public_snapshot_contracts`
- result: passed
- key lines:
  - `test commands::bench::spire_pipeline::tests::spire_pipeline_sql_uses_public_snapshot_contracts ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 419 filtered out`

### `cargo-test-lib-selected-leaf-scan-profile.log`

- command: `cargo test --lib collect_quantized_selected_leaf_scan_profile_reports_scan_counters`
- result: passed
- key lines:
  - `test am::ec_spire::scan::tests::collect_quantized_selected_leaf_scan_profile_reports_scan_counters ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2240 filtered out`
