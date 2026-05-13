# Artifact Manifest: SPIRE Diagnostics Sanity Fixture Split

- Head SHA: `2325892db352ccc91d6f90e787f551b4a0f47779`
- Packet/topic: `31010-spire-diagnostics-sanity-fixture-split`
- Lane: Phase 12b.2 fixture-sink cleanup
- Fixture: scan-sanity, health, and relation-storage snapshot fixtures
- Storage format: mixed fixture diagnostic coverage; no benchmark storage format
- Rerank mode: not applicable
- Surface: existing textual `include!("diagnostics.rs")`; no isolated one-index-per-table or shared-table measurement surface
- Measurement claim: none

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Timestamp: 2026-05-13 16:45:54 PDT
- Result: command exit code `0`
- Key line: `Script done on 2026-05-13 16:45:54-07:00 [COMMAND_EXIT_CODE="0"]`
- Note: emitted the repo's stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`.

### `cargo-test-scan-sanity-snapshot.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_scan_sanity_snapshot_sql -- --nocapture`
- Timestamp: 2026-05-13 16:48:16 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_scan_sanity_snapshot_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 41.65s`

### `cargo-test-relation-storage-snapshot.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_relation_storage_snapshot_sql -- --nocapture`
- Timestamp: 2026-05-13 16:50:21 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_relation_storage_snapshot_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 34.67s`

### `location-check.log`

- Command: `rg -n 'fn test_ec_spire_scan_sanity_snapshot_sql|fn test_ec_spire_health_snapshot_sql|fn test_ec_spire_relation_storage_snapshot_sql' src/tests/diagnostics.rs src/tests/mod.rs`
- Timestamp: 2026-05-13 16:50:32 PDT
- Result: command exit code `0`
- Key lines:
  - `src/tests/diagnostics.rs`
  - `642:    fn test_ec_spire_scan_sanity_snapshot_sql() {`
  - `737:    fn test_ec_spire_health_snapshot_sql() {`
  - `816:    fn test_ec_spire_relation_storage_snapshot_sql() {`

### `line-counts.log`

- Command: `wc -l src/tests/mod.rs src/tests/diagnostics.rs src/lib.rs`
- Timestamp: 2026-05-13 16:50:32 PDT
- Result: command exit code `0`
- Key lines:
  - `37607 src/tests/mod.rs`
  - `980 src/tests/diagnostics.rs`
  - `17812 src/lib.rs`
