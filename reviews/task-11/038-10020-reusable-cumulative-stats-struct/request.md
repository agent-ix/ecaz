# Review Request: Reusable Cumulative Stats Struct

Scope:
- `src/am/stats.rs`
- `spec/functional/FR-025-custom-statistics.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added a reusable `TqStatsCounters` struct in `src/am/stats.rs` with one field per staged FR-025
  cumulative metric.
- Added pure record/reset helpers for each metric so the runtime lane can increment the intended
  counters later without this branch touching `scan.rs` or wiring PostgreSQL pgstat support early.
- Kept the existing PG17 snapshot boundary intact, but made `am/stats.rs` more than a metadata
  stub by giving it real pure behavior with unit coverage.
- Updated FR-025, the test matrix, and Task 11 notes to record this as the current D1 seam for
  custom cumulative statistics.

Review focus:
- Whether `TqStatsCounters` is the right ownership boundary for future runtime increments and PG18
  pgstat flush/serialization work
- Whether the chosen field set matches the intended FR-025 metric contract closely enough to avoid
  churn when real pgstat wiring lands
- Whether pure per-counter helper methods are the right API at this stage, or whether a more
  generic mutation surface would be better before runtime integration begins
- Whether this addresses the reviewer concern that `am/stats.rs` should provide real scaffolding
  rather than only a static snapshot wrapper

Questions to answer:
- Should `TqStatsCounters` stay in `am/stats.rs`, or move to more shared territory before runtime
  and pgstat integration begin?
- Are any FR-025 metrics missing or prematurely included in the struct?
- Does this leave the remaining FR-025 D1 work in the right state: reusable counters exist, but no
  pgstat registration or SQL-visible cumulative stats exist on PG17?
