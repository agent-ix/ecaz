# Task 231 Packet 004 artifact manifest

- Head SHA: `08075a341274f9f76df018f503af912d6d95b0e5`.
- Task bucket and packet: `reviews/task-231/004-lifecycle-and-dml/`.
- Lane: local Intel development host, PostgreSQL 18 / pgrx 0.17.
- Fixture/storage format: fixed-stride EFM1 node relation with an ordinary
  row-tier payload and a `node_ordinal` graph directory; no Task 229 covering
  sidecar and no Task 230 hot/cold row tier.
- Rerank mode: exact vector embedded in each fixed-stride node; focused
  correctness fixtures do not execute the full benchmark rerank matrix.
- PostgreSQL integrity prerequisite: the PG18 pgrx cluster has data page
  checksums enabled. The Packet 003 fixed-stride fixtures assert
  `SHOW data_checksums = on`, and `pg_controldata /home/peter/.pgrx/data-18`
  reports `Data page checksum version: 1` at this source checkpoint.
- Isolation: each pgrx test creates its own one-index, one-generation fixture.
  No shared-table benchmark surface or external corpus is used.

## `fixed-stride-dml-pg18.log`

- Timestamp: `2026-08-29T22:46:28-07:00`.
- Head SHA: `08075a341274f9f76df018f503af912d6d95b0e5` (the source tree used by the
  run was committed immediately after the green result with no source edit).
- Command: `cargo pgrx test pg18 test_distann_fixed_stride_dml_append_overlay_and_rollback`.
- SHA-256: `3f354364e2ffafd1db75c250e17f1b02599b5cc908cb0a84736625d77d958094`.
- Result: `1 passed; 0 failed`; command exit code 0.
- Key covered result: ordinals 0 through 6 remain dense across insert,
  replacement, tombstone, aborted-tail reuse, and two same-transaction
  backlinks; the current node retains exact-vector, row-locator, tombstone,
  and adjacency identity.

## `fixed-stride-lifecycle-pg18.log`

- Timestamp: `2026-08-29T22:53:48-07:00`.
- Head SHA: `08075a341274f9f76df018f503af912d6d95b0e5` (same no-edit source tree).
- Command: `cargo pgrx test pg18 test_distann_fixed_stride_retire_reclaim_rollback`.
- SHA-256: `4c0df47cdd841805be2a2b4b2d229d85dca4d091497cb60a69aa36450cff747d`.
- Result: `1 passed; 0 failed`; command exit code 0.
- Key covered result: retirement retains all four generation relations;
  failed reclaim rolls all relation drops back; successful repeated reclaim
  drops the raw store with the other relations and reports `Reclaimed`.

## `clippy-pg18.log`

- Timestamp: `2026-08-29T22:59:53-07:00`.
- Head SHA: `08075a341274f9f76df018f503af912d6d95b0e5`.
- Command: `cargo clippy --lib --no-default-features --features pg18 -- -D warnings -A clippy::collapsible-if -A clippy::unnecessary-unwrap`.
- SHA-256: `5cd6067aae709085a06495504c5c5ee5a3805ffc4555cfdd26372c31cd665112`.
- Result: PASS; command exit code 0.
- Exceptions: the two allowed lints are pre-existing repository-wide
  exceptions used by Packet 003; no Task 231 warning is suppressed.
