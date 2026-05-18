# Artifact Manifest: SPIRE Diagnostics Leaf Fixture Split

- Head SHA: `56657029b7f0784c5e89f0eaf8660a6c1db5382f`
- Packet/topic: `31013-spire-diagnostics-leaf-fixture-split`
- Lane: Phase 12b.2 fixture-sink cleanup
- Fixture: leaf snapshot fixture
- Storage format: mixed fixture diagnostic coverage; no benchmark storage format
- Rerank mode: not applicable
- Surface: existing textual `include!("diagnostics.rs")`; no isolated one-index-per-table or shared-table measurement surface
- Measurement claim: none

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Timestamp: 2026-05-13 17:06:50 PDT
- Result: command exit code `0`
- Key line: `Script done on 2026-05-13 17:06:50-07:00 [COMMAND_EXIT_CODE="0"]`
- Note: emitted the repo's stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`.

### `cargo-test-leaf-snapshot.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_leaf_snapshot_sql -- --nocapture`
- Timestamp: 2026-05-13 17:09:18 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_leaf_snapshot_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 43.97s`

### `location-check.log`

- Command: `rg -n 'fn test_ec_spire_leaf_snapshot_sql' src/tests/diagnostics.rs src/tests/mod.rs`
- Timestamp: 2026-05-13 17:09:24 PDT
- Result: command exit code `0`
- Key lines:
  - `src/tests/diagnostics.rs`
  - `1472:    fn test_ec_spire_leaf_snapshot_sql() {`

### `line-counts.log`

- Command: `wc -l src/tests/mod.rs src/tests/diagnostics.rs src/lib.rs`
- Timestamp: 2026-05-13 17:09:24 PDT
- Result: command exit code `0`
- Key lines:
  - `37014 src/tests/mod.rs`
  - `1576 src/tests/diagnostics.rs`
  - `17812 src/lib.rs`
