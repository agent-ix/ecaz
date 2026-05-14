# Artifact Manifest: 31026 SPIRE remote-search final fixture split

Head SHA: `141ab3e64fe1f02a6487e7febaea41fe77fcbb20`

Packet/topic: `31026-spire-remote-search-final-fixture-split`

Timestamp: `2026-05-14T02:23:30Z`

Surface note: this packet moves Rust test fixtures only. No benchmark lane,
fixture corpus, storage format, rerank mode, or isolated/shared index surface
applies.

## Artifacts

### `cargo-fmt-check.log`

- Command: `script -q -e -c 'cargo fmt --check' review/31026-spire-remote-search-final-fixture-split/artifacts/cargo-fmt-check.log`
- Key result: command exited `0`.

### `git-diff-check.log`

- Command: `script -q -e -c 'git diff --check -- plan/tasks/task30-phase12b-spire-cleanup.md src/tests/mod.rs src/tests/remote_search.rs' review/31026-spire-remote-search-final-fixture-split/artifacts/git-diff-check.log`
- Key result: command exited `0`.

### `line-counts.log`

- Command: `script -q -e -c 'wc -l src/tests/mod.rs src/tests/remote_search.rs src/tests/cost_and_planner.rs' review/31026-spire-remote-search-final-fixture-split/artifacts/line-counts.log`
- Key result lines:
  - `24599 src/tests/mod.rs`
  - `12245 src/tests/remote_search.rs`
  - `54 src/tests/cost_and_planner.rs`

### `location-check.log`

- Command: `script -q -e -c 'rg -n "test_ec_spire_remote_search_local_heap_resolution_plan|test_ec_spire_remote_search_degraded_stale_leaf|test_ec_spire_reaper_resolves_lost_prepare_ack_fixture|test_ec_spire_remote_pk_select_isolation_contract_sql|analyzed_query|include!\\(\"custom_scan.rs\"\\)|test_ec_spire_relation_object_tuple_roundtrip" src/tests/mod.rs src/tests/remote_search.rs' review/31026-spire-remote-search-final-fixture-split/artifacts/location-check.log`
- Key result lines:
  - `src/tests/remote_search.rs:2637: fn test_ec_spire_remote_search_local_heap_resolution_plan()`
  - `src/tests/remote_search.rs:11881: fn test_ec_spire_remote_search_degraded_stale_leaf()`
  - `src/tests/remote_search.rs:11933: fn test_ec_spire_reaper_resolves_lost_prepare_ack_fixture()`
  - `src/tests/remote_search.rs:12071: fn test_ec_spire_remote_pk_select_isolation_contract_sql()`
  - `src/tests/mod.rs:2520: unsafe fn analyzed_query(sql: &str) -> *mut pg_sys::Query`
  - `src/tests/mod.rs:2541: include!("custom_scan.rs");`
  - `src/tests/mod.rs:2546: fn test_ec_spire_relation_object_tuple_roundtrip()`

### `pg18-test-remote-local-heap-resolution-plan.log`

- Command: `script -q -e -c 'cargo pgrx test pg18 test_ec_spire_remote_search_local_heap_resolution_plan' review/31026-spire-remote-search-final-fixture-split/artifacts/pg18-test-remote-local-heap-resolution-plan.log`
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 42.85s`

### `pg18-test-remote-degraded-stale-leaf.log`

- Command: `script -q -e -c 'cargo pgrx test pg18 test_ec_spire_remote_search_degraded_stale_leaf' review/31026-spire-remote-search-final-fixture-split/artifacts/pg18-test-remote-degraded-stale-leaf.log`
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 34.09s`

### `pg18-test-reaper-lost-prepare-ack.log`

- Command: `script -q -e -c 'cargo pgrx test pg18 test_ec_spire_reaper_resolves_lost_prepare_ack_fixture' review/31026-spire-remote-search-final-fixture-split/artifacts/pg18-test-reaper-lost-prepare-ack.log`
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 32.41s`

### `pg18-test-remote-pk-select-isolation.log`

- Command: `script -q -e -c 'cargo pgrx test pg18 test_ec_spire_remote_pk_select_isolation_contract_sql' review/31026-spire-remote-search-final-fixture-split/artifacts/pg18-test-remote-pk-select-isolation.log`
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 34.25s`
