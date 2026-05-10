---
head_sha: 5bf8632a4ce5db202ba8e45e7db2a4db3ca7e544
packet: 30713-spire-libpq-identity-cache-state
date: 2026-05-09
---

# Artifact Manifest

## cargo-check-pg18.log

- Head SHA: `5bf8632a4ce5db202ba8e45e7db2a4db3ca7e544`
- Lane: PG18 compile check
- Fixture: no SQL fixture
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command: `cargo check --no-default-features --features pg18`
- Timestamp: 2026-05-09 23:14:05-07:00
- Key result lines:
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 0.16s`
  - `COMMAND_EXIT_CODE="0"`

## cargo-pgrx-pg18-libpq-identity-cache-loopback.log

- Head SHA: `5bf8632a4ce5db202ba8e45e7db2a4db3ca7e544`
- Lane: PG18 pgrx loopback executor
- Fixture: `test_ec_spire_remote_search_libpq_executor_loopback_empty`
- Storage format: remote-serving SPIRE index uses `storage_format = 'rabitq'`;
  coordinator SPIRE index uses the default local format.
- Rerank mode: not applicable
- Surface: isolated one-index-per-table loopback surfaces for coordinator and
  remote-serving indexes.
- Command:
  `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
- Timestamp: 2026-05-09 23:07:47-07:00
- Key result lines:
  - `test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1520 filtered out; finished in 53.55s`

## cargo-pgrx-pg18-libpq-identity-mismatch.log

- Head SHA: `5bf8632a4ce5db202ba8e45e7db2a4db3ca7e544`
- Lane: PG18 pgrx endpoint-identity rejection
- Fixture: `test_ec_spire_libpq_rejects_identity_mismatch`
- Storage format: remote-serving SPIRE index uses `storage_format = 'rabitq'`;
  coordinator SPIRE index uses the default local format.
- Rerank mode: not applicable
- Surface: isolated one-index-per-table loopback surfaces for coordinator and
  remote-serving indexes.
- Command: `cargo pgrx test pg18 test_ec_spire_libpq_rejects_identity_mismatch`
- Timestamp: 2026-05-09 23:11:08-07:00
- Key result lines:
  - `test tests::pg_test_ec_spire_libpq_rejects_identity_mismatch - should panic ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1520 filtered out; finished in 26.10s`

## git-diff-check.log

- Head SHA: `5bf8632a4ce5db202ba8e45e7db2a4db3ca7e544`
- Lane: static whitespace check
- Fixture: code, Phase 11 task update, and review packet artifacts
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command: `git diff --check`
- Timestamp: 2026-05-09 23:13:56-07:00
- Key result lines:
  - command exited with code 0 and produced no output.
