---
head_sha: 2f03028be0129ff3bf0e505c4bc2a4cd36196323
packet: 30714-spire-libpq-identity-cache-test-matrix
date: 2026-05-09
---

# Artifact Manifest

## cargo-check-pg18.log

- Head SHA: `2f03028be0129ff3bf0e505c4bc2a4cd36196323`
- Lane: PG18 compile check
- Fixture: no SQL fixture
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command: `cargo check --no-default-features --features pg18`
- Timestamp: 2026-05-09 23:26:34-07:00
- Key result lines:
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 4.74s`
  - `COMMAND_EXIT_CODE="0"`

## cargo-pgrx-pg18-libpq-cache-key-probe.log

- Head SHA: `2f03028be0129ff3bf0e505c4bc2a4cd36196323`
- Lane: PG18 pgrx identity-cache key matrix
- Fixture: `test_ec_spire_remote_search_libpq_executor_loopback_empty`
- Storage format: remote-serving SPIRE index uses `storage_format = 'rabitq'`;
  coordinator SPIRE index uses the default local format.
- Rerank mode: not applicable
- Surface: isolated one-index-per-table loopback surfaces for coordinator and
  remote-serving indexes.
- Command:
  `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
- Timestamp: 2026-05-09 23:20:01-07:00
- Key result lines:
  - `test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1520 filtered out; finished in 35.58s`
  - `COMMAND_EXIT_CODE="0"`

## cargo-pgrx-pg18-libpq-cache-capability-blocks.log

- Head SHA: `2f03028be0129ff3bf0e505c4bc2a4cd36196323`
- Lane: PG18 pgrx capability blocker cache-matrix coverage
- Fixture: `test_ec_spire_libpq_capability_blocks`
- Storage format: default local SPIRE test indexes with blocked remote
  descriptors; no remote endpoint query should be attempted.
- Rerank mode: not applicable
- Surface: isolated one-index-per-table surfaces per strict/degraded stale,
  retention-gap, and extension-version case.
- Command: `cargo pgrx test pg18 test_ec_spire_libpq_capability_blocks`
- Timestamp: 2026-05-09 23:23:10-07:00
- Key result lines:
  - `test tests::pg_test_ec_spire_libpq_capability_blocks ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1520 filtered out; finished in 25.55s`
  - `COMMAND_EXIT_CODE="0"`

## git-diff-check.log

- Head SHA: `2f03028be0129ff3bf0e505c4bc2a4cd36196323`
- Lane: static whitespace check
- Fixture: code, ADR/task updates, and review packet artifacts
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command: `git diff --check`
- Timestamp: 2026-05-09 23:26:39-07:00
- Key result lines:
  - command exited with code 0 and produced no output.
