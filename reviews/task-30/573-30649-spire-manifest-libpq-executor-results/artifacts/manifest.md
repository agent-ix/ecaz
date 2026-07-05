# Artifact Manifest

Packet: `30649-spire-manifest-libpq-executor-results`

Head SHA: `38fae3e92f26d3c56b0db2707d7cd5ee6309ebbd`

Timestamp: `2026-05-08T22:14:55-07:00`

## Validation

- Command: `cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_libpq_executor_loopback`
- Fixture: PG18 pgrx loopback manifest executor test
- Lane: SPIRE remote epoch manifest libpq executor results
- Storage format: committed loopback remote SPIRE index plus transactional coordinator SPIRE index
- Rerank mode: not applicable
- Surface isolation: remote fixture created through a separate client connection; coordinator descriptor/index and persisted manifest created in the PG test transaction
- Result: passed
- Key lines:
  - `test tests::pg_test_ec_spire_remote_epoch_manifest_libpq_executor_loopback ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1464 filtered out`

## Measurement Claims

No benchmark or performance measurement claims are made in this packet.
