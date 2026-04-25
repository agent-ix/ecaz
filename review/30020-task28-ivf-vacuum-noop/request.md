# Review Request: Task 28 IVF Vacuum No-Op Baseline

Scope: Phase 6 vacuum checkpoint. IVF `ambulkdelete` and `amvacuumcleanup`
now return stable no-op stats instead of failing with not-implemented errors.

Task: `plan/tasks/28-ivf-access-method.md` Phase 6

Branch: `task28-ivf`

Head SHA: `d51f24c696a1c4cee3e6120871dfae209bfdc485`

Owner: coder2

Files:

- `src/am/ec_ivf/vacuum.rs`
- `src/am/ec_ivf/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_ec_ivf_empty_vacuum_callbacks_report_noop_stats`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_callbacks_keep_live_count_noop`
- `git diff --check`

Validation notes:

- Validation was PG18-only per the current user direction to focus on PG18.
- The PG tests were run against PostgreSQL 18.3 through pgrx.
- No measurement claim is made in this packet.

## Summary

This slice starts Phase 6 with a safe vacuum callback baseline:

- Replaces `ec_ivf` vacuum not-implemented errors with metadata-backed no-op
  stats.
- Allocates `IndexBulkDeleteResult` when PostgreSQL calls `ambulkdelete` with
  null stats.
- Reports relation block count and persisted metadata live tuple count without
  mutating postings, directory entries, or drift counters.
- Adds a PG debug hook for direct callback exercise from pgrx tests.
- Covers empty and populated IVF indexes, including a deleted heap row where the
  no-op baseline deliberately keeps the existing live-count state.

## Review Focus

Please review for:

- Whether metadata-backed live tuple count is the right no-op baseline before
  dead-tuple cleanup lands.
- Whether ignoring the bulkdelete callback in this checkpoint is explicit enough
  in code and tests.
- Whether the debug hook should live in `vacuum.rs` or be moved beside the scan
  debug helpers.
- Whether Phase 6 should next prioritize dead-posting marking or directory
  repair.

## Non-Goals

This packet does not implement dead tuple cleanup, directory repair, drift
snapshots, SQL `VACUUM` coverage, planner costing, heap/source rerank, or
measurement gates.
