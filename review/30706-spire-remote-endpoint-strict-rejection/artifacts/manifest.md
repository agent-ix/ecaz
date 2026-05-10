# Artifact Manifest

Packet: `30706-spire-remote-endpoint-strict-rejection`
Head SHA: `9107021600e15fd657c0275838007516d481deb9`
Timestamp: `2026-05-10T04:11:42Z`

## cargo-pgrx-pg18-non-ready-endpoint-rejection.log

- Command: `script -q -c "cargo pgrx test pg18 test_ec_spire_libpq_executor_rejects_non_ready_endpoint" /home/peter/dev/ecaz/review/30706-spire-remote-endpoint-strict-rejection/artifacts/cargo-pgrx-pg18-non-ready-endpoint-rejection.log`
- Lane: Phase 11 Stage B remote endpoint strict rejection.
- Fixture: PG18 loopback coordinator plus one loopback remote PostgreSQL SPIRE index.
- Storage format: remote-serving index uses default non-RaBitQ storage.
- Rerank mode: `ec_spire_remote_search` candidate scoring via current SQL endpoint.
- Surface shape: isolated one-index-per-table coordinator and remote loopback surfaces.
- Key result line: `test tests::pg_test_ec_spire_libpq_executor_rejects_non_ready_endpoint - should panic ... ok`
- Key result line: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1516 filtered out`

## cargo-pgrx-pg18-ready-loopback-accepted.log

- Command: `script -q -c "cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty" /home/peter/dev/ecaz/review/30706-spire-remote-endpoint-strict-rejection/artifacts/cargo-pgrx-pg18-ready-loopback-accepted.log`
- Lane: Phase 11 Stage B ready endpoint acceptance guardrail.
- Fixture: PG18 loopback coordinator plus one loopback remote PostgreSQL SPIRE index.
- Storage format: remote-serving index uses `storage_format = 'rabitq'`.
- Rerank mode: `ec_spire_remote_search` candidate scoring via current SQL endpoint.
- Surface shape: isolated one-index-per-table coordinator and remote loopback surfaces.
- Key result line: `test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok`
- Key result line: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1516 filtered out`
