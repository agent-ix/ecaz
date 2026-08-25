# Task 236 packet 002 artifact manifest

- Head SHA: `8c535390dea709d5000a2ce376c3cae8d812ebd6`
- Task / packet: `reviews/task-236/002-shared-secure-connector/`
- Timestamp: 2026-08-24 13:47–13:48 America/Los_Angeles
- Lane: local PG18 compile and unit validation
- Fixture / corpus / storage / rerank: not applicable; this packet reviews the
  connector implementation, while the multinode TLS/mTLS and benchmark matrix
  remains packet 003 work.
- Isolation: no index or table fixture was used.

## Artifacts

### `cargo-check-pg18.log`

- Command: `cargo check --lib --no-default-features --features pg18,pg_test`
- Result: exit 0; dev profile completed.

### `remote-postgres-tls-tests-pg18.log`

- Command: `cargo test remote_postgres_tls --lib --no-default-features --features pg18,pg_test`
- Result: 8 passed, 0 failed.
- Covers strict/default TLS policy, downgrade rejection, explicit-loopback
  plaintext, unsupported/duplicate options, redaction, credential rotation,
  SPIRE compatibility, and the side-transaction loopback gate.

### `distann-remote-transport-tests-pg18.log`

- Command: `cargo test am::ec_distann::remote_transport::tests --lib --no-default-features --features pg18,pg_test`
- Result: 15 passed, 0 failed.
- Covers deadlines, cancellation support, typed pool disposition, credential
  rotation/redaction, config redaction, bounded wire-detail extraction, batch
  ordering, and existing transport invariants.

### `distann-route-tests-pg18.log`

- Command: `cargo test am::ec_distann::generation_read::cache_tests --lib --no-default-features --features pg18,pg_test`
- Result: 3 passed, 0 failed.
- Includes proof that catalog-backed session roster serialization contains the
  canonical secret reference and not resolved conninfo.

### `distann-resolved-secret-test-pg18.log`

- Command: `cargo test resolved_secret_separates_identity_rotation_and_redacts_debug --lib --no-default-features --features pg18,pg_test`
- Result: 1 passed, 0 failed.
- Proves stable secret identity across rotation, changed security fingerprint,
  and redacted debug output.
