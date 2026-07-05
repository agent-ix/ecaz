# Review Request: Consolidate Planner Snapshot SQL Surface

Commit: `610134a`

Scope:
- `src/lib.rs`
- `src/am/mod.rs`
- `src/am/shared.rs`
- `spec/tests.md`

Summary:
- remove the redundant SQL snapshot entry points so the public planner/admin surface now keeps only
  `tqhnsw_index_cost_snapshot(...)` and `tqhnsw_planner_integration_snapshot(...)`
- drop the corresponding `lib.rs` SQL tests for the removed admin/explain/stats/explain-counter/
  read-stream/pg18-upgrade/pg18-diagnostics functions
- trim the now-unused AM wrapper/helpers that only existed to feed those deleted SQL functions
- update the test matrix to stop advertising the removed SQL entry points as supported contracts

Please review:
- whether the remaining public SQL surface is now small enough for D1 scaffolding
- whether any deleted SQL output should have been preserved inside the surviving cost or planner
  integration snapshots instead of being removed entirely
- whether the test matrix cleanup matches the intended post-merge public contract
