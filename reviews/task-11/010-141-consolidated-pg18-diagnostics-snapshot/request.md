# Review Request: Consolidated PG18 Diagnostics Snapshot

Scope:
- `src/am/mod.rs`
- `src/am/shared.rs`
- `src/lib.rs`
- `spec/functional/FR-024-custom-explain.md`
- `spec/functional/FR-025-custom-statistics.md`
- `spec/functional/FR-027-pgrx-pg18-upgrade.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added a read-only SQL/admin surface, `tqhnsw_pg18_diagnostics_snapshot()`, that reports the
  intended custom EXPLAIN option name (`tqvector`) and statistics function name
  (`tqvector_stats`) together, plus explicit readiness flags for EXPLAIN option registration,
  EXPLAIN per-node hook wiring, pgstat-kind registration, and stats SQL-function availability.
- Built that surface by consolidating the existing pure EXPLAIN and stats scaffolding seams into
  one shared snapshot in `src/am/shared.rs`, without turning on any live PG18 behavior.
- Added pg coverage that verifies the intended names and keeps every readiness flag false on the
  current PG17/toolchain state.
- Updated FR-024, FR-025, FR-027, the test matrix, and Task 11 notes so this is recorded as a
  consolidated diagnostics boundary for productization work, not as active PG18 diagnostics support.

Review focus:
- Whether a single diagnostics snapshot is the right productization seam on top of the separate
  EXPLAIN and stats scaffolding helpers
- Whether this surface clarifies the broader PG18 diagnostics boundary or just duplicates the
  narrower snapshot helpers without enough added value
- Whether the current result shape is explicit enough that no one will mistake it for real
  `EXPLAIN (tqvector)` or `tqvector_stats()` support

Questions to answer:
- Is `tqhnsw_pg18_diagnostics_snapshot()` the right consolidated near-term boundary, or should
  productization keep consuming the separate explain/stats snapshot surfaces directly?
- Does reporting the intended EXPLAIN option and stats SQL function together make the PG18
  diagnostics story easier to reason about?
- Is there any missing readiness signal that should be surfaced now for the future diagnostics lane?
