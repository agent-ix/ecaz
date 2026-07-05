# Artifact Manifest: SPIRE Diagnostics Active Fixture Split

- Head SHA: `a828f2b210e8d43f6cccfbf77b7990cbf74e3751`
- Packet/topic: `31011-spire-diagnostics-active-fixture-split`
- Lane: Phase 12b.2 fixture-sink cleanup
- Fixture: active snapshot, large-routing diagnostics, and allocator snapshot fixtures
- Storage format: mixed fixture diagnostic coverage; no benchmark storage format
- Rerank mode: not applicable
- Surface: existing textual `include!("diagnostics.rs")`; no isolated one-index-per-table or shared-table measurement surface
- Measurement claim: none

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Timestamp: 2026-05-13 16:52:56 PDT
- Result: command exit code `0`
- Key line: `Script done on 2026-05-13 16:52:56-07:00 [COMMAND_EXIT_CODE="0"]`
- Note: emitted the repo's stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`.

### `cargo-test-active-snapshot-diagnostics.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_active_snapshot_diagnostics_sql -- --nocapture`
- Timestamp: 2026-05-13 16:55:18 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_active_snapshot_diagnostics_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 40.39s`

### `cargo-test-allocator-snapshot.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_allocator_snapshot_sql -- --nocapture`
- Timestamp: 2026-05-13 16:57:15 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_allocator_snapshot_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 31.74s`

### `location-check.log`

- Command: `rg -n 'fn test_ec_spire_active_snapshot_diagnostics_sql|fn test_ec_spire_large_routing_object_builds_and_scans|fn test_ec_spire_allocator_snapshot_sql' src/tests/diagnostics.rs src/tests/mod.rs`
- Timestamp: 2026-05-13 16:57:22 PDT
- Result: command exit code `0`
- Key lines:
  - `src/tests/diagnostics.rs`
  - `983:    fn test_ec_spire_active_snapshot_diagnostics_sql() {`
  - `1047:    fn test_ec_spire_large_routing_object_builds_and_scans() {`
  - `1099:    fn test_ec_spire_allocator_snapshot_sql() {`

### `line-counts.log`

- Command: `wc -l src/tests/mod.rs src/tests/diagnostics.rs src/lib.rs`
- Timestamp: 2026-05-13 16:57:22 PDT
- Result: command exit code `0`
- Key lines:
  - `37401 src/tests/mod.rs`
  - `1187 src/tests/diagnostics.rs`
  - `17812 src/lib.rs`
