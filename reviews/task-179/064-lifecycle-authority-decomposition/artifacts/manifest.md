# Artifact manifest

- Task bucket: `reviews/task-179/`
- Packet: `064-lifecycle-authority-decomposition`
- Head SHA: `0043c3e746bef0baf6977dc8ae426006d7a0a887`
- Fixture: PG18, local pgrx installation
- Storage format / rerank mode: not applicable (lifecycle and shared-memory validation)
- Isolation: focused tests; no benchmark measurements

## Artifacts

### `scan-registry-pg18.log`

- Timestamp: `2026-07-13T22:32:46-07:00`
- SHA-256: `0bf8e6e498a5cb434911acb1ec093e15778ac28b58224706ae4512124109d18f`
- Command: `cargo pgrx test pg18 test_ec_distann_scan_registry_two_backend_retirement_contention --no-default-features --features pg18`
- Result: PASS (`1 passed; 0 failed; 2513 filtered out`)
- Key behavior: backend A held the transaction-exclusive retirement fence;
  backend B's production-equivalent shared registration fence timed out, then
  registered after A committed; backend A observed B's shared token count.

### `lifecycle-pg18.log`

- Timestamp: `2026-07-13T22:39:25-07:00`
- SHA-256: `64e9663836036c81477565ea9cc1ea788c4b3a296a65f87742e2ba79e5617381`
- Command: `cargo pgrx test pg18 test_distann_multi_epoch_publish --no-default-features --features pg18`
- Result: PASS (`1 passed; 0 failed; 2513 filtered out`)

### `lifecycle-unit.log`

- Timestamp: `2026-07-13T22:41:40-07:00`
- SHA-256: `beb1728ab37e7374c45c0b6092e31d3d100094dc12daa5f1ea36c9a2e12379be`
- Command: `cargo test --lib --no-default-features --features 'pg18 pg_test' lifecycle_transition_authority_rejects_terminal_and_skipped_edges`
- Result: PASS (`1 passed; 0 failed; 2513 filtered out`)

### `clippy-pg18.log`

- Timestamp: `2026-07-13T22:43:28-07:00`
- SHA-256: `25a1da33aaeb4f7a7f3dd1a73c4c726b10d8961561039741e447c708545db5fe`
- Command: `cargo clippy --lib --no-default-features --features 'pg18 pg_test' -- -D warnings`
- Result: PASS
