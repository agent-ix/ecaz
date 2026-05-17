# Review Request: Stats Summary Derived Rates

Scope:
- `src/am/stats.rs`
- `spec/functional/FR-025-custom-statistics.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added a pure `TqStatsSummary` struct in `src/am/stats.rs` plus `TqStatsCounters::summary()`.
- Implemented the derived FR-025 metrics that were previously only described in the spec example:
  `bootstrap_hit_rate` and `quantizer_cache_rate`.
- Added unit coverage for both the normal-rate case and zero-denominator handling so the future
  pgstat and SQL layers inherit stable semantics instead of inventing them later.
- Updated FR-025, the test matrix, and Task 11 notes to record that the derived statistics
  contract now exists as real planner-owned code.

Review focus:
- Whether `TqStatsSummary` is the right pure seam for the future `tqvector_stats()` SQL surface
- Whether computing the derived rates in planner-owned pure code is the right boundary before PG18
  pgstat registration and SQL-function wiring exist
- Whether zero-denominator behavior should remain `0.0` for both rates, or whether another staged
  representation would be better

Questions to answer:
- Should `TqStatsSummary` stay in `am/stats.rs`, or move closer to eventual SQL output wiring later?
- Are there any other FR-025 derived metrics that should be modeled now to keep the future SQL
  surface from improvising behavior?
- Does this make the FR-025 D1 seam complete enough that further stats work should wait for PG18
  pgstat and runtime integration?
