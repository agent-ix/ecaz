---
head_sha: cca32b586ddecc71102cf6ed072e18783aeec437
packet: 30711-spire-remote-capability-search-gates
date: 2026-05-09
---

# Artifact Manifest

## cargo-pgrx-pg18-libpq-capability-blocks.log

- Head SHA: `cca32b586ddecc71102cf6ed072e18783aeec437`
- Lane: PG18 pgrx focused libpq capability gates
- Fixture: local coordinator indexes with remote node descriptors for stale
  epoch and extension-version skew; strict and degraded modes
- Storage format: default coordinator indexes; descriptor-only remote fixtures
- Rerank mode: not applicable
- Surface: shared-table SQL test surfaces
- Command:
  `cargo pgrx test pg18 test_ec_spire_libpq`
- Timestamp: 2026-05-09 22:46:52-07:00
- Key result lines:
  - `test tests::pg_test_ec_spire_libpq_capability_blocks ... ok`
  - `test tests::pg_test_ec_spire_libpq_receive_attempts_degraded_skip ... ok`
  - `test tests::pg_test_ec_spire_libpq_rejects_identity_mismatch - should panic ... ok`
  - `test tests::pg_test_ec_spire_libpq_executor_rejects_non_ready_endpoint - should panic ... ok`
  - `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1517 filtered out`

## cargo-pgrx-pg18-receive-contract.log

- Head SHA: `cca32b586ddecc71102cf6ed072e18783aeec437`
- Lane: PG18 pgrx focused receive/executor contract
- Fixture: SQL-visible contract rows
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: shared-table SQL test surfaces
- Command:
  `cargo pgrx test pg18 test_ec_spire_remote_search_receive_contract`
- Timestamp: 2026-05-09 22:49:00-07:00
- Key result lines:
  - `test tests::pg_test_ec_spire_remote_search_receive_contract ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1520 filtered out`

## cargo-pgrx-pg18-libpq-loopback.log

- Head SHA: `cca32b586ddecc71102cf6ed072e18783aeec437`
- Lane: PG18 pgrx focused ready-path loopback
- Fixture: one coordinator index and one loopback remote-serving RaBitQ index
- Storage format: coordinator default, remote `storage_format = 'rabitq'`
- Rerank mode: remote heap candidate path
- Surface: shared-table SQL test surfaces
- Command:
  `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
- Timestamp: 2026-05-09 22:51:06-07:00
- Key result lines:
  - `test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1520 filtered out`

## git-diff-check.log

- Head SHA: `cca32b586ddecc71102cf6ed072e18783aeec437`
- Lane: whitespace/static patch check
- Fixture: working tree diff after code slice
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: not applicable
- Command: `git diff --check`
- Timestamp: 2026-05-09 22:51:11-07:00
- Key result lines:
  - `COMMAND_EXIT_CODE="0"`
