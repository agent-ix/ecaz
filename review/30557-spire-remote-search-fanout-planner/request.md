# Review Request: SPIRE Remote Search Fanout Planner

- Code commit: `531036f2` (`Add SPIRE remote search fanout planner`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 coordinator transport groundwork
- Agent: coder1

## Summary

This checkpoint adds the coordinator-side fanout planning contract that sits
between leaf selection and future libpq execution:

- adds `plan_remote_search_fanout`;
- validates selected PID input, rejecting PID 0 and duplicate selected PIDs;
- validates the published epoch snapshot before planning fanout;
- separates selected leaf PIDs into local execution and per-remote-node target
  groups by placement `node_id`;
- sorts remote target groups by `node_id` while preserving selected PID order
  within each node;
- applies strict/degraded placement state rules before fanout;
- records degraded skipped placements with `(node_id, pid, state)` diagnostics;
- adds unit coverage for local/remote grouping, degraded unavailable/skipped
  diagnostics, and duplicate selected PID rejection;
- updates the Phase 7 task note to mark fanout planning as landed while keeping
  libpq execution open.

This does not open libpq connections or execute remote SQL yet. It gives the
next slice a deterministic request shape to execute.

## Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/tests.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Check that local and remote selected PIDs are split on `placement.node_id`
   using `node_id = 0` as the local coordinator node.
2. Check the degraded skip behavior: unavailable/skipped placements are recorded
   in diagnostics and excluded from target requests, while stale remains
   fail-closed.
3. Check whether remote target ordering by `node_id` is the right deterministic
   shape for the first libpq fanout executor.
4. Check that this remains a planner-only slice and does not prematurely expose
   a public coordinator API.

## Validation

- `cargo test --lib remote_search_fanout --no-default-features --features pg18`
  - Result: passed; 3 tests passed.
- `cargo check --lib --no-default-features --features pg18`
- `git diff --check`
