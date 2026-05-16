# Artifacts: 31003 SPIRE Scan Fixture Split

- Code commit: `475fcb0d237db629994f6c2545498b14a49cb0d6`
- Packet/topic: `31003-spire-scan-fixture-split`
- Lane / fixture / storage format / rerank mode: Phase 12b.2 PG18 fixture layout split; focused scan-placement fixture uses existing ec_spire defaults
- Isolated one-index-per-table or shared-table surfaces: not applicable to the layout change; focused pg_test creates its existing local table/index fixture

## Validation

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/31003-spire-scan-fixture-split/artifacts/cargo-fmt-check.log`
- Timestamp: 2026-05-13 15:54 PDT
- Key result lines:
  - command exited `0`
  - rustfmt emitted the repository's existing stable-channel warnings for unstable `imports_granularity` / `group_imports` options

### `cargo-test-scan-fixture.log`

- Command: `script -q -e -c "cargo test --no-default-features --features pg18 test_ec_spire_scan_placement_snapshot_sql -- --nocapture" review/31003-spire-scan-fixture-split/artifacts/cargo-test-scan-fixture.log`
- Timestamp: 2026-05-13 15:55 PDT
- Key result lines:
  - `test tests::pg_test_ec_spire_scan_placement_snapshot_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out`
  - pre-existing unused-import warning in `src/am/mod.rs`

### `fixture-location-check.log`

- Command: `script -q -e -c "rg -n 'include!\\(\\\"scan.rs\\\"\\)|fn test_ec_spire_scan_placement_snapshot_sql|fn test_ec_spire_classify_centroid_recursive_sql|fn test_ec_spire_schema_drift_fails_before_dispatch_sql|fn test_ec_spire_boundary_replica_build_writes_and_dedupes_scan' src/tests && wc -l src/tests/mod.rs src/tests/scan.rs src/tests/placement.rs src/tests/dml_frontdoor.rs src/tests/insert.rs src/tests/remote_search.rs src/tests/custom_scan.rs src/lib.rs" review/31003-spire-scan-fixture-split/artifacts/fixture-location-check.log`
- Timestamp: 2026-05-13 15:57 PDT
- Key result lines:
  - `src/tests/mod.rs:2766:    include!("scan.rs");`
  - `src/tests/scan.rs:2:    fn test_ec_spire_scan_placement_snapshot_sql() {`
  - `src/tests/scan.rs:763:    fn test_ec_spire_classify_centroid_recursive_sql() {`
  - `src/tests/mod.rs:2771:    fn test_ec_spire_schema_drift_fails_before_dispatch_sql() {`
  - `src/tests/mod.rs:4775:    fn test_ec_spire_boundary_replica_build_writes_and_dedupes_scan() {`
  - `41296 src/tests/mod.rs`
  - `840 src/tests/scan.rs`
