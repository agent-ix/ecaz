# Artifact Manifest: SPIRE Diagnostics Fixture Split

- Head SHA: `64756fa05a1983955229f35106c83bc0ea020eea`
- Packet/topic: `31007-spire-diagnostics-fixture-split`
- Lane: Phase 12b.2 fixture-sink cleanup
- Fixture: hierarchy, object, delta, and options snapshot diagnostics
- Storage format: not applicable; structural fixture move
- Rerank mode: not applicable
- Surface: existing textual `include!("diagnostics.rs")`; no isolated one-index-per-table or shared-table measurement surface
- Measurement claim: none

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Timestamp: 2026-05-13 16:23:51 PDT
- Result: command exit code `0`
- Key line: `Script done on 2026-05-13 16:23:51-07:00 [COMMAND_EXIT_CODE="0"]`
- Note: emitted the repo's stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`.

### `cargo-test-hierarchy-snapshot.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_hierarchy_snapshot_sql -- --nocapture`
- Timestamp: 2026-05-13 16:26:26 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_hierarchy_snapshot_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 48.00s`

### `cargo-test-options-snapshot.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_options_snapshot_sql -- --nocapture`
- Timestamp: 2026-05-13 16:28:32 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_options_snapshot_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 31.52s`

### `location-check.log`

- Command: `rg -n 'fn test_ec_spire_hierarchy_snapshot_sql|fn test_ec_spire_object_snapshot_sql|fn test_ec_spire_delta_snapshot_sql|fn test_ec_spire_delta_snapshot_sql_delete_delta|fn test_ec_spire_options_snapshot_sql' src/tests/diagnostics.rs src/tests/mod.rs`
- Timestamp: 2026-05-13 16:28:41 PDT
- Result: command exit code `0`
- Key lines:
  - `src/tests/diagnostics.rs`
  - `2:    fn test_ec_spire_hierarchy_snapshot_sql() {`
  - `120:    fn test_ec_spire_object_snapshot_sql() {`
  - `213:    fn test_ec_spire_delta_snapshot_sql() {`
  - `295:    fn test_ec_spire_delta_snapshot_sql_delete_delta() {`
  - `360:    fn test_ec_spire_options_snapshot_sql() {`

### `line-counts.log`

- Command: `wc -l src/tests/mod.rs src/tests/diagnostics.rs src/lib.rs`
- Timestamp: 2026-05-13 16:28:41 PDT
- Result: command exit code `0`
- Key lines:
  - `39092 src/tests/mod.rs`
  - `640 src/tests/diagnostics.rs`
  - `17812 src/lib.rs`
