# Review Request: Statistics Scaffolding Snapshot

Scope:
- `src/am/stats.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `spec/functional/FR-025-custom-statistics.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added a planner-owned `src/am/stats.rs` module that holds pure PG18 statistics scaffolding state:
  the intended SQL function name `tqvector_stats` plus explicit readiness flags for pgstat-kind
  registration and SQL-surface wiring.
- Added a read-only SQL/admin surface, `tqhnsw_stats_snapshot()`, that reports that intended
  function name and the current readiness flags without defining `tqvector_stats()` itself.
- Added unit and pg coverage that keep the current boundary honest: the snapshot reports
  `tqvector_stats`, both PG18 readiness flags are `false`, and `tqvector_stats()` is still absent
  on PG17.
- Updated FR-025, the test matrix, and Task 11 tracking so this is recorded as descriptive D1
  scaffolding rather than active custom pgstat support.

Review focus:
- Whether a dedicated `tqhnsw_stats_snapshot()` SQL surface is the right seam for operational
  statistics planning before PG18 pgstat APIs exist in this repository
- Whether `src/am/stats.rs` is the right long-lived home for pure stats scaffolding, similar to the
  recent `am/explain.rs` and `am/cost.rs` seams
- Whether the current contract is explicit enough that users and later code will not confuse it
  with a real `tqvector_stats()` implementation

Questions to answer:
- Is `tqhnsw_stats_snapshot()` the right near-term productization surface, or should the intended
  `tqvector_stats` identity remain purely internal until PG18 support lands?
- Are the two readiness flags enough, or is there another explicit boundary we should surface now
  for later pgstat-kind versus SQL-function activation work?
- Does this make the FR-025 current-vs-target story clearer without adding unnecessary SQL surface
  area?
