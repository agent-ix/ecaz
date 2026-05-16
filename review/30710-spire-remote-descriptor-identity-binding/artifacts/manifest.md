# 30710 Artifact Manifest

Head SHA: `e5bdf0803ef834849b2a997fd8d914dc76b4a565`

Packet: `30710-spire-remote-descriptor-identity-binding`

Generated: `2026-05-09T22:23:41-07:00`

## Artifacts

### `cargo-pgrx-pg18-libpq-identity-binding.log`

- Lane: PG18 pgrx focused strict/degraded libpq receive tests
- Fixture: loopback remote descriptor tests for descriptor/endpoint identity mismatch, non-ready endpoint rejection, and degraded non-ready reporting
- Storage format: identity-mismatch remote uses `storage_format = 'rabitq'`; non-ready remotes use default non-RaBitQ storage
- Rerank mode: fixed inner-product scoring profile
- Command: `script -q -e -c "cargo pgrx test pg18 test_ec_spire_libpq" review/30710-spire-remote-descriptor-identity-binding/artifacts/cargo-pgrx-pg18-libpq-identity-binding.log`
- Isolation: isolated one-index-per-table surfaces for each loopback fixture
- Key result lines:
  - `test tests::pg_test_ec_spire_libpq_receive_attempts_degraded_skip ... ok`
  - `test tests::pg_test_ec_spire_libpq_rejects_identity_mismatch - should panic ... ok`
  - `test tests::pg_test_ec_spire_libpq_executor_rejects_non_ready_endpoint - should panic ... ok`
  - `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1517 filtered out`

### `cargo-pgrx-pg18-libpq-loopback-descriptor-identity.log`

- Lane: PG18 pgrx focused ready loopback executor test
- Fixture: one coordinator table/index plus one loopback remote table/index
- Storage format: coordinator default local storage; remote-serving index `storage_format = 'rabitq'`; descriptor identity registered from the live endpoint fingerprint
- Rerank mode: fixed inner-product scoring profile
- Command: `script -q -e -c "cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty" review/30710-spire-remote-descriptor-identity-binding/artifacts/cargo-pgrx-pg18-libpq-loopback-descriptor-identity.log`
- Isolation: isolated one-index-per-table surfaces for coordinator and remote loopback tables
- Key result lines:
  - `test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1519 filtered out`
