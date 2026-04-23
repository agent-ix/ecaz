# Review Request: Parallel Scan Deferred Local Emit Breakdown

Current head: `e3948ed`

Scope:
- `src/am/common/explain.rs`
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged Task 18 branch already exposed `Parallel Deferred Local Emits` as
  the aggregate count for last-resort deferred local fallback.
- But that aggregate did not say whether the remaining fallback pressure came
  from foreign selected-pending blockers or from foreign admitted-head
  blockers.
- That made the final ownership gap visible only as one total, even though the
  branch already distinguishes those blocker kinds everywhere else in the
  staged shared-state path.

What changed:
- Split the EXPLAIN counter surface with two new integer counters:
  - `stats_parallel_deferred_local_emits_foreign_selected_pending`
  - `stats_parallel_deferred_local_emits_foreign_admitted_head`
- Added matching human-readable EXPLAIN properties:
  - `Parallel Deferred Local Emits: Foreign Selected`
  - `Parallel Deferred Local Emits: Foreign Head`
- When deferred local fallback actually happens,
  `take_next_deferred_parallel_blocked_output(...)` now increments the blocker-
  specific counter that matches the retained blocker kind before it increments
  the existing aggregate counter.
- Updated the staged explain and scan-side tests to prove:
  - the new counters are part of the FR-024 explain surface
  - the counters record and reset correctly
  - admitted-head fallback increments the admitted-head-specific counter
  - the selected-pending-specific counter stays untouched for that path
- Updated Task 18 notes to record that the last-resort deferred local emit is
  now measurable by blocker kind, not only as one aggregate fallback count.

Why this matters:
- This slice does not change runtime behavior; it makes the remaining ownership
  gap measurable with the same blocker vocabulary the branch already uses for
  shared snapshots and explain counters elsewhere.
- Reviewers can now tell whether last-resort local fallback is still coming
  from foreign selected rows, foreign admitted heads, or both.
- That keeps the final cross-worker ownership-transfer work diagnosable instead
  of leaving the hardest remaining seam hidden behind one total fallback count.

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
- Whether splitting deferred local emits by blocker kind is the right EXPLAIN
  surface, versus keeping that breakdown only in internal counters
- Whether the blocker-specific scan-side assertion is enough proof that the new
  counters stay aligned with the retained-blocker contract
