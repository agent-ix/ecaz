# Review Request: Task 28 IVF Duplicate Heap TID Guard

Scope: Phase 5 live-insert checkpoint. IVF `aminsert` now rejects a heap TID
that is already present in any live posting before appending a new posting.

Task: `plan/tasks/28-ivf-access-method.md` Phase 5

Branch: `task28-ivf`

Head SHA: `5e832e0cffdfa65f26c89e98443d3e057d5d2619`

Owner: coder2

Files:

- `src/am/ec_ivf/insert.rs`
- `src/am/ec_ivf/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_ec_ivf_insert_rejects_duplicate_heap_tid`
- `cargo pgrx test pg18 test_ec_ivf_insert_reuses_same_list_tail_page`
- `git diff --check`

Validation notes:

- Validation was PG18-only per the current AGENTS policy and the explicit user
  direction to test with PG18.
- The new PG test was run against PostgreSQL 18.3 through pgrx.
- No measurement claim is made in this packet.

## Summary

This slice closes duplicate heap-TID rejection for Phase 5:

- Adds a production guard before live append that scans current live postings
  and rejects an already-indexed heap TID.
- Keeps the empty-index bootstrap path unchanged because it cannot already have
  postings.
- Adds a PG debug hook that exercises the same guard against an existing indexed
  row.
- Verifies the normal same-list append path still succeeds with the guard active.
- Updates the task plan so true concurrent insert coverage is the remaining
  Phase 5 item.

## Review Focus

Please review for:

- Whether a full posting scan is acceptable as the first correctness guard, or
  whether duplicate detection should be narrowed to the assigned list.
- Whether deleted postings should be ignored, as implemented here.
- Whether the PG debug hook is the right way to cover an otherwise hard-to-reach
  duplicate heap-TID condition.
- Whether this guard needs stronger locking before concurrent insert coverage.

## Non-Goals

This packet does not implement concurrent insert stress coverage, vacuum cleanup,
planner costing, heap/source rerank, or measurement gates.
