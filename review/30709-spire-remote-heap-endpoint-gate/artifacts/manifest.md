# 30709 Artifact Manifest

Head SHA: `d4e84e5ccecc17bf5a84cfe39288c951d4994930`

Packet: `30709-spire-remote-heap-endpoint-gate`

Generated: `2026-05-09T22:09:38-07:00`

## Artifacts

### `cargo-pgrx-pg18-endpoint-contract.log`

- Lane: PG18 pgrx focused contract test
- Fixture: SQL-visible remote search receive and endpoint contract rows
- Storage format: contract-only, no index fixture storage format
- Rerank mode: fixed endpoint contract row, no rerank execution
- Command: `script -q -e -c "cargo pgrx test pg18 test_ec_spire_remote_search_receive_contract" review/30709-spire-remote-heap-endpoint-gate/artifacts/cargo-pgrx-pg18-endpoint-contract.log`
- Isolation: contract-only, no shared table/index fixture
- Key result lines:
  - `test tests::pg_test_ec_spire_remote_search_receive_contract ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1518 filtered out`

### `cargo-pgrx-pg18-libpq-loopback.log`

- Lane: PG18 pgrx focused loopback executor test
- Fixture: one coordinator table/index plus one loopback remote table/index
- Storage format: coordinator default local storage, remote-serving index `storage_format = 'rabitq'`
- Rerank mode: fixed inner-product scoring profile
- Command: `script -q -e -c "cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty" review/30709-spire-remote-heap-endpoint-gate/artifacts/cargo-pgrx-pg18-libpq-loopback.log`
- Isolation: isolated one-index-per-table surfaces for coordinator and remote loopback tables
- Key result lines:
  - `test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1518 filtered out`

### `cargo-pgrx-pg18-heap-endpoint-rejection.log`

- Lane: PG18 pgrx focused negative coordinator-result test
- Fixture: one coordinator table/index plus one loopback remote table/index
- Storage format: remote-serving index uses default non-RaBitQ storage and is expected to be non-ready
- Rerank mode: fixed inner-product scoring profile
- Command: `script -q -e -c "cargo pgrx test pg18 test_ec_spire_heap_endpoint_rejects_non_ready" review/30709-spire-remote-heap-endpoint-gate/artifacts/cargo-pgrx-pg18-heap-endpoint-rejection.log`
- Isolation: isolated one-index-per-table surfaces for coordinator and remote loopback tables
- Key result lines:
  - `test tests::pg_test_ec_spire_heap_endpoint_rejects_non_ready - should panic ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1518 filtered out`

### `cargo-pgrx-pg18-libpq-strict-degraded.log`

- Lane: PG18 pgrx focused strict/degraded receive diagnostics
- Fixture: loopback remote descriptor tests with strict non-ready rejection and degraded non-ready skip reporting
- Storage format: remote-serving indexes use default non-RaBitQ storage and are expected to be non-ready
- Rerank mode: fixed inner-product scoring profile
- Command: `script -q -e -c "cargo pgrx test pg18 test_ec_spire_libpq" review/30709-spire-remote-heap-endpoint-gate/artifacts/cargo-pgrx-pg18-libpq-strict-degraded.log`
- Isolation: isolated one-index-per-table surfaces for each focused fixture
- Key result lines:
  - `test tests::pg_test_ec_spire_libpq_receive_attempts_degraded_skip ... ok`
  - `test tests::pg_test_ec_spire_libpq_executor_rejects_non_ready_endpoint - should panic ... ok`
  - `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1517 filtered out`
