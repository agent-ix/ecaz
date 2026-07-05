# Review Request: Task 28 IVF Vacuum Dead Posting Cleanup

Scope: Phase 6 vacuum checkpoint. IVF `ambulkdelete` now honors the
PostgreSQL bulkdelete callback and removes dead heap TIDs from posting tuples.

Task: `plan/tasks/28-ivf-access-method.md` Phase 6

Branch: `task28-ivf`

Head SHA: `77286b7a06ff425dc280554f9748450d9328b24c`

Owner: coder2

Files:

- `src/am/ec_ivf/page.rs`
- `src/am/ec_ivf/vacuum.rs`
- `src/am/ec_ivf/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_bulkdelete_removes_dead_heap_tid`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_callbacks_keep_live_count_noop`
- `git diff --check`

Validation notes:

- Validation was PG18-only per the current user direction to focus on PG18.
- The PG tests were run against PostgreSQL 18.3 through pgrx.
- No measurement claim is made in this packet.

## Summary

This slice turns the Phase 6 vacuum baseline into real dead-TID cleanup:

- Passes the PostgreSQL bulkdelete callback through live IVF postings.
- Removes callback-dead heap TIDs from posting tuples and marks fully emptied
  postings as deleted.
- Updates list live/dead counts and metadata live/dead totals.
- Preserves callback-less `ambulkdelete`/`amvacuumcleanup` as no-op stats
  reporters.
- Adds a PG debug helper and a PG test proving the deleted heap TID no longer
  appears in IVF scan output after vacuum cleanup.

## Review Focus

Please review for:

- Whether directory live/dead counters should count heap TIDs, as implemented,
  or posting tuples.
- Whether per-posting WAL rewrites are acceptable for this first correctness
  slice before page compaction and directory repair.
- Whether deleted postings should retain their original payload for now.
- Whether the debug bulkdelete helper should open the index with a stronger or
  weaker lock.

## Non-Goals

This packet does not compact posting pages, repair list head/tail refs, reclaim
empty pages, expose drift/admin snapshots, run SQL `VACUUM`, implement planner
costing, or make measurement claims.
