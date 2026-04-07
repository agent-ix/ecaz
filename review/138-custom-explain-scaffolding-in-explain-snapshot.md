# Review Request: Custom EXPLAIN Scaffolding In Explain Snapshot

Scope:
- `src/am/explain.rs`
- `src/am/mod.rs`
- `src/am/shared.rs`
- `src/lib.rs`
- `spec/functional/FR-006-sql-operators.md`
- `spec/functional/FR-024-custom-explain.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added a planner-owned `src/am/explain.rs` module that holds pure custom-EXPLAIN scaffolding
  state, currently just the intended option name (`tqvector`) and explicit readiness flags for
  PG18 option registration and `explain_per_node_hook` wiring.
- Extended `tqhnsw_index_explain_snapshot(regclass)` to report `explain_option_name`,
  `pg18_custom_explain_option_ready`, and `pg18_explain_per_node_hook_ready` alongside the
  existing planner gate and ordering-semantics scaffolding.
- Kept both readiness flags hard-false because the repository still has no PG18 feature flag,
  toolchain support, or live hook registration path.
- Added pure unit coverage for the explain scaffolding snapshot, extended pg coverage for the SQL
  snapshot contract, and updated FR-006 / FR-024 / test matrix / Task 11 notes to record this as
  descriptive D1 groundwork rather than active EXPLAIN integration.

Review focus:
- Whether introducing `src/am/explain.rs` now is the right boundary for planner-owned EXPLAIN
  groundwork before any PG18-specific hook wiring exists
- Whether exposing custom-EXPLAIN readiness through the existing explain snapshot is a coherent
  staging seam or too much planner-facing detail too early
- Whether the readiness flags stay explicit enough that no one can mistake this for real
  `EXPLAIN (tqvector)` support on PG17

Questions to answer:
- Is `src/am/explain.rs` the right long-lived home for pure EXPLAIN scaffolding, or should this
  remain folded into `shared.rs` until there is more real PG18 implementation?
- Is reporting `explain_option_name = 'tqvector'` plus both readiness flags through
  `tqhnsw_index_explain_snapshot(...)` the right productization-facing contract?
- Does this make the FR-024 current-vs-target state clearer without implying that parser/hook
  integration is closer than it really is?
