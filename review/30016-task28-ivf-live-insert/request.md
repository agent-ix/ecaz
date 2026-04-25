# Review Request: Task 28 IVF Live Insert

Scope: Phase 5 live-insert checkpoint. Non-empty IVF indexes now append new
rows through `aminsert`, assign them to persisted centroids, and update
directory plus metadata counters.

Task: `plan/tasks/28-ivf-access-method.md` Phase 5

Branch: `task28-ivf`

Head SHA: `9057726e68c01ed7d7337388363b13d3a8600b7e`

Owner: coder2

Files:

- `src/am/ec_ivf/build.rs`
- `src/am/ec_ivf/insert.rs`
- `src/am/ec_ivf/page.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_ec_ivf_insert_appends_posting_and_updates_stats`
- `cargo pgrx test pg18 test_ec_ivf_insert_rejects_dimension_mismatch`
- `cargo pgrx test pg18 test_ec_ivf_gettuple_emits_probe_candidates_with_scores`
- `git diff --check`

Validation notes:

- Validation was PG18-only per the current AGENTS policy and the explicit user
  direction to test with PG18.
- The new PG tests were run against PostgreSQL 18.3 through pgrx.
- No measurement claim is made in this packet.

## Summary

This slice starts Phase 5 live insert:

- Reuses the build tuple decode path for `aminsert`, with context-specific
  error text for build vs insert callers.
- Rejects live insert into empty IVF indexes until the empty-index bootstrap
  contract is designed.
- Validates inserted tuple dimension, source vector shape, non-zero spherical
  assignment input, finite gamma, and posting tuple page fit.
- Loads persisted centroids, assigns the inserted row to the nearest centroid,
  and appends one posting tuple to the selected list's tail block or a new data
  block.
- Rewrites the selected list directory tuple with updated head/tail refs,
  live count, and inserted-since-build count.
- Rewrites metadata with updated total live count and inserted-since-build
  drift.
- Adds PG coverage proving that a live-inserted row is reachable through the
  IVF scan path and that dimension mismatch is rejected.

## Review Focus

Please review for:

- Whether the page append and directory rewrite helpers follow the existing
  GenericXLog/full-image pattern correctly.
- Whether rewriting metadata after the directory update is acceptable for this
  first live-insert slice, or whether metadata locking should be tightened
  before more Phase 5 work lands.
- Whether rejecting empty-index live insert is the right temporary v1 behavior.
- Whether the current shape validation is adequate for this checkpoint, given
  fuller quantizer bits/seed/source validation remains open.
- Whether duplicate heap-TID handling should be implemented before any broader
  live-insert concurrency coverage.

## Non-Goals

This packet does not implement empty-index first insert, duplicate heap-TID
coalescing/rejection, concurrent insert stress coverage, vacuum cleanup,
planner costing, heap/source rerank, or real measurement gates.
