# Artifact Manifest

Packet: `30648-spire-libpq-executor-nonempty-receive`

Head SHA: `576a4c106f8fb7b6de2b53dbc460027b3256fb7c`

Timestamp: `2026-05-08T22:07:23-07:00`

## Validation

- Command: `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
- Fixture: PG18 pgrx loopback executor test
- Lane: SPIRE remote search libpq executor nonempty receive
- Storage format: committed loopback remote SPIRE index plus transactional coordinator SPIRE index
- Rerank mode: not applicable
- Surface isolation: remote fixture created through a separate client connection; coordinator descriptor/index created in the PG test transaction
- Result: passed
- Key lines:
  - `test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1463 filtered out`

## Measurement Claims

No benchmark or performance measurement claims are made in this packet.
