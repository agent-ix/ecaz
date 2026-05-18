# Review Request: Task 28 IVF Concurrent Inserts

Status: open
Owner: coder2
Code checkpoint: bbc26c04d1d7d71e2573793494ec631e67847aac
Branch: task28-ivf
Date: 2026-04-25

## Scope

This packet covers the Phase 5 concurrent-insert checkpoint for the first
`ec_ivf` access method baseline.

Changes in scope:

- Add an exclusive-lock metadata page update helper that reads the latest
  metadata under the metadata-page buffer lock before incrementing live insert
  counters.
- Split live insert stats updates so list-directory counters are updated on
  the assigned list tuple and global metadata counters are updated through the
  new read-modify-write path.
- Add PG18 coverage that starts two separate `psql` sessions behind a shared
  advisory-lock barrier, inserts rows assigned to different IVF lists, and
  verifies heap count, directory count, metadata count, per-list inserted
  totals, and full-probe scan reachability.
- Mark Phase 5 concurrency coverage complete in
  `plan/tasks/28-ivf-access-method.md`.

## Files

- `src/am/ec_ivf/page.rs`
- `src/am/ec_ivf/insert.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

## Validation

PG18-only validation:

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_ec_ivf_concurrent_inserts`
- `cargo pgrx test pg18 test_ec_ivf_insert_appends_posting_and_updates_stats`
- `git diff --check`

PostgreSQL version: 18.3 via pgrx `pg18`.

No measurement claims are made in this packet.

## Review Focus

- Whether the metadata page read-modify-write helper is the right narrow fix
  for different-list live insert races.
- Whether the test's advisory-lock barrier plus two external `psql` sessions is
  acceptable PG18 coverage for concurrent AM callbacks.
- Whether same-list concurrent inserts should remain a separate future slice,
  since this checkpoint covers the planned different-list case only.

## Non-Goals

- Same-list concurrent insert serialization.
- Concurrent insert plus vacuum stress.
- Page compaction or posting-list page reclamation.
- Planner/admin integration.
