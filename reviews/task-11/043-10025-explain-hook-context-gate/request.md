# Review Request: Explain Hook Context Gate

Scope:
- `src/am/explain.rs`
- `spec/functional/FR-024-custom-explain.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added pure `ExplainNodeKind` and `ExplainHookContext` types in `src/am/explain.rs`.
- Tightened `should_emit_explain_properties(...)` so the staged EXPLAIN hook contract now requires
  all three real conditions from FR-024: the `tqvector` option is enabled, the current node is an
  `IndexScan`, and the access method is `tqhnsw`.
- Expanded unit coverage so the future PG18 hook has an explicit tested gate for all three cases
  instead of only option-plus-access-method checking.
- Updated FR-024, the test matrix, and Task 11 notes so the spec matches the tighter pure gating
  contract already expected by the eventual `explain_per_node_hook`.

Review focus:
- Whether `ExplainHookContext` is the right pure seam for the future PG18 `explain_per_node_hook`
- Whether `ExplainNodeKind` should stay minimal (`IndexScan` / `Other`) at D1, or model more plan
  node categories now
- Whether the stricter gate captures the right boundary without overfitting to current staging code

Questions to answer:
- Should the pure hook context eventually also carry `ANALYZE` state, or is node kind plus access
  method the right stopping point for D1?
- Is `ExplainNodeKind::Other` sufficient for now, or would a broader enum help the later PG18 hook
  binding without adding premature surface area?
- Does this make the FR-024 gating seam complete enough that the next EXPLAIN work should wait for
  real PG18 hook registration and scan-owned counter wiring?
