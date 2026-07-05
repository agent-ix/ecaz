# Review Request: EXPLAIN Property Emission Skeleton

Scope:
- `src/am/explain.rs`
- `spec/functional/FR-024-custom-explain.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added pure `ExplainPropertyValue` and `ExplainProperty` types in `src/am/explain.rs` so the
  future PG18 `explain_per_node_hook` has a concrete data contract for emitted properties.
- Added `TqExplainCounters::explain_properties()` to map the staged counter struct into the exact
  integer/bool property payloads the hook is expected to emit, without touching `scan.rs` or adding
  another SQL snapshot.
- Added `should_emit_explain_properties(...)` as a pure gate that requires both the `tqvector`
  EXPLAIN option and the `tqhnsw` access method before any property emission occurs.
- Added unit coverage plus FR-024 / test-matrix / Task 11 updates to record this as the current D1
  EXPLAIN-hook seam.

Review focus:
- Whether the property/value types are the right pure boundary before PG18 hook bindings exist
- Whether `TqExplainCounters::explain_properties()` captures the intended FR-024 output contract
  cleanly enough for the scan lane and eventual hook binding to share
- Whether `should_emit_explain_properties(...)` is the right minimal gate for the staged hook
  skeleton, or whether another pure condition should be represented now
- Whether this is the right response to the reviewer guidance to build real D1 seams in
  `am/explain.rs` instead of adding more SQL snapshot surfaces

Questions to answer:
- Should these property-emission types stay in `am/explain.rs`, or move to more shared territory
  before the scan lane starts embedding and reading counters?
- Is there any FR-024 output detail missing from the pure property contract that would make later
  PG18 hook binding awkward?
- Does this leave the remaining D1 EXPLAIN work in the right state: pure emission contract present,
  but no registered hook and no scan-lane wiring yet?
