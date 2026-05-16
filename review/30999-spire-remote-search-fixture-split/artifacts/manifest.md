# Artifacts: 30999 SPIRE Remote Search Fixture Split

- Code commit: `a88c92d339d37bdbd6743cd6397b0a20ddbc9435`
- Packet/topic: `30999-spire-remote-search-fixture-split`
- Lane / fixture / storage format / rerank mode: Phase 12b.2 PG18 fixture layout split; focused remote-search SQL fixture uses existing ec_spire defaults
- Isolated one-index-per-table or shared-table surfaces: not applicable to the layout change; focused pg_test creates its existing local table/index fixture

## Validation

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30999-spire-remote-search-fixture-split/artifacts/cargo-fmt-check.log`
- Timestamp: 2026-05-13 15:33 PDT
- Key result lines:
  - command exited `0`
  - rustfmt emitted the repository's existing stable-channel warnings for unstable `imports_granularity` / `group_imports` options

### `cargo-test-remote-search-fixture.log`

- Command: `script -q -e -c "cargo test --no-default-features --features pg18 test_ec_spire_remote_search_sql_scores_selected_leaf_pids -- --nocapture" review/30999-spire-remote-search-fixture-split/artifacts/cargo-test-remote-search-fixture.log`
- Timestamp: 2026-05-13 15:33 PDT
- Key result lines:
  - `test tests::pg_test_ec_spire_remote_search_sql_scores_selected_leaf_pids ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out`
  - pre-existing unused-import warning in `src/am/mod.rs`

### `fixture-location-check.log`

- Command: `script -q -e -c "rg -n 'include!\\(\\\"remote_search.rs\\\"\\)|fn test_ec_spire_remote_search_sql_scores_selected_leaf_pids|fn test_ec_spire_remote_search_final_contract|fn test_ec_spire_custom_scan_status_registered_fail_closed' src/tests && wc -l src/tests/mod.rs src/tests/remote_search.rs src/tests/custom_scan.rs src/lib.rs" review/30999-spire-remote-search-fixture-split/artifacts/fixture-location-check.log`
- Timestamp: 2026-05-13 15:35 PDT
- Key result lines:
  - `src/tests/mod.rs:10354:    include!("remote_search.rs");`
  - `src/tests/remote_search.rs:2:    fn test_ec_spire_remote_search_sql_scores_selected_leaf_pids() {`
  - `src/tests/remote_search.rs:2553:    fn test_ec_spire_remote_search_final_contract() {`
  - `src/tests/mod.rs:10357:    fn test_ec_spire_custom_scan_status_registered_fail_closed() {`
  - `45298 src/tests/mod.rs`
  - `2636 src/tests/remote_search.rs`
