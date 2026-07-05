# Review Request: Admin Snapshot For Planner And Insert Stats

Scope:
- `src/am/mod.rs`
- `src/am/shared.rs`
- `src/lib.rs`
- `spec/functional/FR-009-hnsw-scan.md`
- `spec/functional/FR-016-hnsw-insert.md`
- `spec/tests.md`
- `plan/tasks/archive/05-build-and-scan.md`

What changed:
- Added a read-only SQL/admin surface, `tqhnsw_index_admin_snapshot(regclass)`, that reports
  block count, live-node count, effective `ef_search`, tuning source, and current planner-gate
  state for a `tqhnsw` index.
- Kept insert-drift accounting honest by exposing `inserted_since_rebuild` as explicitly
  unavailable (`NULL`) until that metric is actually tracked.
- Added pg coverage for both the happy path and non-`tqhnsw` index rejection.
- Updated FR-009 / FR-016 / test-matrix / task tracking so this new admin surface is recorded as
  planner-and-statistics scaffolding rather than mistaken for full planner enablement or completed
  insert-drift accounting.

Review focus:
- Whether the new SQL/admin snapshot surface is the right boundary for planner-facing and
  productization-facing stats without bleeding into runtime scan work
- Whether returning `NULL` for `inserted_since_rebuild` is the right temporary contract versus
  inventing a placeholder value
- Whether the non-`tqhnsw` index validation/error behavior is clear and safe
- Whether the spec updates accurately separate current admin scaffolding from future FR-016 drift
  accounting and planner-visible scan behavior

Questions to answer:
- Is `tqhnsw_index_admin_snapshot(regclass)` the right long-lived SQL surface for current
  planner/admin introspection, or should this information eventually move behind a view or separate
  explain/stats function family?
- Is exposing `total_live_nodes` now, while keeping `inserted_since_rebuild` nullable, the right
  incremental contract for FR-016 staging?
- Does the current result shape include the essential fields needed for upcoming costing/explain
  work without overcommitting the later statistics design?
