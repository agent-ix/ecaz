# Artifact Manifest: SPIRE Diagnostics Final Fixture Split

- Head SHA: `bb36316c8ef84cb1c1ab4933433be59a7edb77f9`
- Packet/topic: `31012-spire-diagnostics-final-fixture-split`
- Lane: Phase 12b.2 fixture-sink cleanup
- Fixture: top-graph snapshot and boundary-replica placement diagnostics fixtures
- Storage format: mixed fixture diagnostic coverage; no benchmark storage format
- Rerank mode: not applicable
- Surface: existing textual `include!("diagnostics.rs")`; no isolated one-index-per-table or shared-table measurement surface
- Measurement claim: none

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Timestamp: 2026-05-13 16:59:31 PDT
- Result: command exit code `0`
- Key line: `Script done on 2026-05-13 16:59:31-07:00 [COMMAND_EXIT_CODE="0"]`
- Note: emitted the repo's stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`.

### `cargo-test-top-graph-snapshot.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_top_graph_snapshot_sql -- --nocapture`
- Timestamp: 2026-05-13 17:01:56 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_top_graph_snapshot_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 40.42s`

### `cargo-test-boundary-replica-placement-diagnostics.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_boundary_replica_placement_diagnostics_sql -- --nocapture`
- Timestamp: 2026-05-13 17:04:04 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_boundary_replica_placement_diagnostics_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 32.21s`

### `location-check.log`

- Command: `rg -n 'fn test_ec_spire_top_graph_snapshot_sql|fn test_ec_spire_boundary_replica_placement_diagnostics_sql' src/tests/diagnostics.rs src/tests/mod.rs`
- Timestamp: 2026-05-13 17:04:12 PDT
- Result: command exit code `0`
- Key lines:
  - `src/tests/diagnostics.rs`
  - `1190:    fn test_ec_spire_top_graph_snapshot_sql() {`
  - `1348:    fn test_ec_spire_boundary_replica_placement_diagnostics_sql() {`

### `line-counts.log`

- Command: `wc -l src/tests/mod.rs src/tests/diagnostics.rs src/lib.rs`
- Timestamp: 2026-05-13 17:04:12 PDT
- Result: command exit code `0`
- Key lines:
  - `37120 src/tests/mod.rs`
  - `1469 src/tests/diagnostics.rs`
  - `17812 src/lib.rs`
