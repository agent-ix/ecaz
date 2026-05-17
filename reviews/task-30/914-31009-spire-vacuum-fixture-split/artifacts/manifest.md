# Artifact Manifest: SPIRE Vacuum Fixture Split

- Head SHA: `2674571c2c5fb3e9b7300944a6ed1ea5c70ab1b2`
- Packet/topic: `31009-spire-vacuum-fixture-split`
- Lane: Phase 12b.2 fixture-sink cleanup
- Fixture: epoch cleanup, epoch snapshot, and maintenance-run fixtures
- Storage format: mixed fixture vacuum/maintenance coverage; no benchmark storage format
- Rerank mode: not applicable
- Surface: existing textual `include!("vacuum.rs")`; no isolated one-index-per-table or shared-table measurement surface
- Measurement claim: none

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Timestamp: 2026-05-13 16:38:13 PDT
- Result: command exit code `0`
- Key line: `Script done on 2026-05-13 16:38:13-07:00 [COMMAND_EXIT_CODE="0"]`
- Note: emitted the repo's stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`.

### `cargo-test-epoch-cleanup-run.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_epoch_cleanup_run_reclaims_old_tuples_sql -- --nocapture`
- Timestamp: 2026-05-13 16:40:37 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_epoch_cleanup_run_reclaims_old_tuples_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 40.38s`

### `cargo-test-maintenance-split-publish.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_maintenance_run_split_publish_sql -- --nocapture`
- Timestamp: 2026-05-13 16:43:06 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_maintenance_run_split_publish_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 34.18s`

### `location-check.log`

- Command: `rg -n 'fn test_ec_spire_epoch_cleanup_run_reclaims_old_tuples_sql|fn test_ec_spire_epoch_snapshot_sql|fn test_ec_spire_maintenance_run_empty_sql|fn test_ec_spire_locked_maintenance_run_plan_no_write_sql|fn test_ec_spire_maintenance_run_no_candidate_sql|fn test_ec_spire_recursive_maintenance_run_rejected|fn test_ec_spire_maintenance_run_merge_publish_sql|fn test_ec_spire_maintenance_run_split_publish_sql' src/tests/vacuum.rs src/tests/mod.rs`
- Timestamp: 2026-05-13 16:43:16 PDT
- Result: command exit code `0`
- Key lines:
  - `src/tests/vacuum.rs`
  - `2:    fn test_ec_spire_epoch_cleanup_run_reclaims_old_tuples_sql() {`
  - `100:    fn test_ec_spire_epoch_snapshot_sql() {`
  - `278:    fn test_ec_spire_maintenance_run_empty_sql() {`
  - `354:    fn test_ec_spire_locked_maintenance_run_plan_no_write_sql() {`
  - `570:    fn test_ec_spire_maintenance_run_no_candidate_sql() {`
  - `681:    fn test_ec_spire_recursive_maintenance_run_rejected() {`
  - `711:    fn test_ec_spire_maintenance_run_merge_publish_sql() {`
  - `841:    fn test_ec_spire_maintenance_run_split_publish_sql() {`

### `line-counts.log`

- Command: `wc -l src/tests/mod.rs src/tests/vacuum.rs src/lib.rs`
- Timestamp: 2026-05-13 16:43:16 PDT
- Result: command exit code `0`
- Key lines:
  - `37947 src/tests/mod.rs`
  - `920 src/tests/vacuum.rs`
  - `17812 src/lib.rs`
