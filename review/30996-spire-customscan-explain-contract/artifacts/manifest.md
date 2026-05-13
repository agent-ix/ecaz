# Artifacts: 30996 SPIRE CustomScan Explain Contract

- Head SHA before code commit: `245f69f0c264eaf36aeaafa43435d0d97603aefd`
- Packet/topic: `30996-spire-customscan-explain-contract`
- Lane / fixture / storage format / rerank mode: Phase 12b.3 CustomScan EXPLAIN callback; PG18 loopback remote tuple-payload fixture; existing fixture storage; relation rerank width `0`
- Isolated one-index-per-table or shared-table surfaces: existing fixture one coordinator table/index plus loopback remote table/index

## Passing validation

### `cargo-test-customscan-explain-contract-pass.log`

- Command: `script -q -e -c "cargo test --no-default-features --features pg18 test_ec_spire_customscan_returns_loopback_remote_tuple_payload -- --nocapture" review/30996-spire-customscan-explain-contract/artifacts/cargo-test-customscan-explain-contract-pass.log`
- Timestamp: 2026-05-13 12:31 PDT
- Key result lines:
  - `test tests::pg_test_ec_spire_customscan_returns_loopback_remote_tuple_payload ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out`
- Notes: compile emitted the pre-existing unused-import warning in `src/am/mod.rs`.

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30996-spire-customscan-explain-contract/artifacts/cargo-fmt-check.log`
- Timestamp: 2026-05-13 12:32 PDT
- Key result lines:
  - command exited `0`
  - rustfmt emitted the repository's existing stable-channel warnings for unstable `imports_granularity` / `group_imports` options

## Diagnostic failed iterations

The following logs are kept to make the callback/fixture adjustment trail explicit:

- `cargo-test-customscan-explain-contract.log`: first implementation called the broad index options snapshot and failed with `ec_spire local heap tuple delivery requires custom_scan_tuple_delivery`.
- `cargo-test-customscan-explain-contract-rerun.log`: callback narrowed, but fixture used plain `EXPLAIN`; no exec-method JSON shape was emitted.
- `cargo-test-customscan-explain-contract-analyze.log`: fixture switched to `ANALYZE`, but still used the wrong SPI JSON iteration shape.
- `cargo-test-customscan-explain-contract-debug-json.log`: confirmed the SPI JSON string was empty under the wrong iteration shape.
- `cargo-test-customscan-explain-contract-json-iteration.log`: confirmed the JSON shape was emitted; expected `nprobe` was corrected from `1` to the fixture's actual `2`.
