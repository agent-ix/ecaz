# Artifact Manifest: SPIRE Build Fixture Split

- Head SHA: `82bb7fffab004af04ab1ce55eb232d949d36e9a5`
- Packet/topic: `31008-spire-build-fixture-split`
- Lane: Phase 12b.2 fixture-sink cleanup
- Fixture: boundary-replica, recursive boundary-replica, and PQ-FastScan populated build-deferral fixtures
- Storage format: mixed fixture build coverage; no benchmark storage format
- Rerank mode: not applicable
- Surface: existing textual `include!("build.rs")`; no isolated one-index-per-table or shared-table measurement surface
- Measurement claim: none

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Timestamp: 2026-05-13 16:30:51 PDT
- Result: command exit code `0`
- Key line: `Script done on 2026-05-13 16:30:51-07:00 [COMMAND_EXIT_CODE="0"]`
- Note: emitted the repo's stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`.

### `cargo-test-boundary-replica-build.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_boundary_replica_build_writes_and_dedupes_scan -- --nocapture`
- Timestamp: 2026-05-13 16:33:17 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_boundary_replica_build_writes_and_dedupes_scan ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 40.52s`

### `cargo-test-pq-fastscan-build-deferral.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_pq_fastscan_populated_build_reports_deferral -- --nocapture`
- Timestamp: 2026-05-13 16:35:17 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_pq_fastscan_populated_build_reports_deferral - should panic ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 33.66s`

### `location-check.log`

- Command: `rg -n 'fn test_ec_spire_boundary_replica_build_writes_and_dedupes_scan|fn test_ec_spire_recursive_boundary_replica_build_dedupes|fn test_ec_spire_pq_fastscan_populated_build_reports_deferral' src/tests/build.rs src/tests/mod.rs`
- Timestamp: 2026-05-13 16:35:26 PDT
- Result: command exit code `0`
- Key lines:
  - `src/tests/build.rs`
  - `2:    fn test_ec_spire_boundary_replica_build_writes_and_dedupes_scan() {`
  - `136:    fn test_ec_spire_recursive_boundary_replica_build_dedupes() {`
  - `209:    fn test_ec_spire_pq_fastscan_populated_build_reports_deferral() {`

### `line-counts.log`

- Command: `wc -l src/tests/mod.rs src/tests/build.rs src/lib.rs`
- Timestamp: 2026-05-13 16:35:26 PDT
- Result: command exit code `0`
- Key lines:
  - `38865 src/tests/mod.rs`
  - `229 src/tests/build.rs`
  - `17812 src/lib.rs`
