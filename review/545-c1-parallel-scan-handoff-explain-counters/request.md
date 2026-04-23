# Review Request: Parallel Scan Handoff Explain Counters

Current head: `a1d0ba5`

Scope:
- `src/am/common/explain.rs`
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged Task 18 branch already counted blocker stalls and local fallback
  paths, but it did not measure the successful shared handoff path itself.
- That left the branch able to say when workers stalled on foreign-owned
  outputs, but not when they actually drained those foreign-selected or
  foreign-head outputs through the shared coordinator seam.

What changed:
- Added two new `Ecaz Stats` counters:
  - `Parallel Handoffs: Foreign Selected`
  - `Parallel Handoffs: Foreign Head`
- Recorded those counters when scan-side handoff successfully drains:
  - a foreign selected pending output
  - a foreign admitted head
- Updated the explain-surface unit coverage so the staged FR-024 counter list,
  reset behavior, and rendered properties all include the new handoff counters.
- Updated scan-side tests so both direct handoff helpers and the graph/materialized
  emit paths assert the correct successful-handoff counters.
- Updated Task 18 notes to record that successful shared handoffs are now
  measurable alongside the existing blocker and local-fallback counters.

Why this matters:
- The branch can now distinguish "blocked on foreign ownership" from "successfully
  transferred through the shared seam".
- That makes the remaining ownership gap easier to quantify while the final
  cross-worker ownership-transfer contract is still deferred.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the successful-handoff counters are attached at the right scan-side
  ownership-transfer points
- Whether any other successful foreign handoff path should also increment these
  counters before planner-visible parallel execution is enabled
