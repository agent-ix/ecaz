# Review Request: Task 28 IVF Vacuum Directory Repair

Scope: Phase 6 vacuum checkpoint. IVF vacuum now repairs per-list live counts
and head/tail block refs from the postings that remain live after cleanup.

Task: `plan/tasks/28-ivf-access-method.md` Phase 6

Branch: `task28-ivf`

Head SHA: `e65b4148421f67481865a39b92ad225425501e8f`

Owner: coder2

Files:

- `src/am/ec_ivf/vacuum.rs`
- `src/am/ec_ivf/scan.rs`
- `src/am/ec_ivf/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_repairs_empty_list_directory_refs`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_bulkdelete_removes_dead_heap_tid`
- `git diff --check`

Validation notes:

- Validation was PG18-only per the current user direction to focus on PG18.
- The PG tests were run against PostgreSQL 18.3 through pgrx.
- No measurement claim is made in this packet.

## Summary

This slice repairs IVF list directory state during vacuum:

- Recomputes each processed list's live heap-TID count from remaining live
  postings.
- Recomputes live head/tail block refs from the first and last live postings.
- Sets empty lists to invalid head/tail refs after vacuum removes all postings.
- Updates metadata total live count from the repaired list totals.
- Adds a PG debug reader for directory entries and a PG test for the empty-list
  repair case.

## Review Focus

Please review for:

- Whether recomputing metadata total live count from directory totals is the
  right repair source of truth.
- Whether empty lists should immediately invalidate head/tail refs, leaving old
  deleted-only pages unreachable until a future reclaim slice.
- Whether the debug directory-entry hook belongs in `scan.rs` or another IVF
  module.
- Whether additional coverage is needed for trimming only the head or tail of a
  multi-page list.

## Non-Goals

This packet does not compact posting pages, reclaim relation blocks, run SQL
`VACUUM`, expose drift/admin snapshots, implement planner costing, or make
measurement claims.
