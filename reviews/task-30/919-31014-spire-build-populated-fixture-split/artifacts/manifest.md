# Artifact Manifest: SPIRE Populated Build Fixture Split

- Head SHA: `cd72767fbf1a136ffd3556159b155016ee3bcd0b`
- Packet/topic: `31014-spire-build-populated-fixture-split`
- Lane: Phase 12b.2 fixture-sink cleanup
- Fixture: populated build root-control and logical-store hash-routing fixtures
- Storage format: mixed fixture build coverage; no benchmark storage format
- Rerank mode: not applicable
- Surface: existing textual `include!("build.rs")`; no isolated one-index-per-table or shared-table measurement surface
- Measurement claim: none

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Timestamp: 2026-05-13 17:11:08 PDT
- Result: command exit code `0`
- Key line: `Script done on 2026-05-13 17:11:08-07:00 [COMMAND_EXIT_CODE="0"]`
- Note: emitted the repo's stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`.

### `cargo-test-populated-build-root-control.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_populated_build_publishes_root_control -- --nocapture`
- Timestamp: 2026-05-13 17:13:30 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_populated_build_publishes_root_control ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 40.05s`

### `cargo-test-populated-build-logical-store.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_populated_build_hash_routes_logical_store_set -- --nocapture`
- Timestamp: 2026-05-13 17:15:30 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_populated_build_hash_routes_logical_store_set ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 31.83s`

### `location-check.log`

- Command: `rg -n 'fn test_ec_spire_populated_build_publishes_root_control|fn test_ec_spire_populated_build_hash_routes_logical_store_set' src/tests/build.rs src/tests/mod.rs`
- Timestamp: 2026-05-13 17:15:39 PDT
- Result: command exit code `0`
- Key lines:
  - `src/tests/build.rs`
  - `231:    fn test_ec_spire_populated_build_publishes_root_control() {`
  - `280:    fn test_ec_spire_populated_build_hash_routes_logical_store_set() {`

### `line-counts.log`

- Command: `wc -l src/tests/mod.rs src/tests/build.rs src/lib.rs`
- Timestamp: 2026-05-13 17:15:39 PDT
- Result: command exit code `0`
- Key lines:
  - `36787 src/tests/mod.rs`
  - `456 src/tests/build.rs`
  - `17812 src/lib.rs`
