# Review Request: Parallel Scan Linear Shared Emit Bookkeeping

Current head: `003a871`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The graph-side shared emit branches already mark their active element as
  emitted before returning a heap TID.
- The linear-fallback path did that for its direct local emit and materialized
  emit branches too.
- But the two linear shared wakeup branches were still emitting without
  recording the current element in emitted-element bookkeeping.
- That left a narrow path where a linear local-only/shared retry could drain
  through the staged coordinator seam without the usual emitted-element guard.

What changed:
- Added `mark_active_result_element_emitted(...)` as the shared helper for
  "mark whichever result state is active for the current execution phase".
- Wired the linear-fallback shared wakeup branches through that helper:
  - `resolve_local_only_parallel_scan_duplicate(...) -> EmitShared`
  - `try_take_republished_local_only_parallel_output(...) -> Emitted`
- Switched the existing linear direct local-only emit branch over to the same
  helper for consistency.
- Added focused unit coverage proving that marking the active result in
  `LinearFallback` records the fallback current element in staged/emitted
  bookkeeping.
- Updated Task 18 notes to record that linear shared wakeup emits now keep the
  same emitted-element bookkeeping as the graph path and the later linear emit
  branches.

Why this matters:
- The staged shared seam no longer has a linear-only bookkeeping gap.
- A linear shared retry cannot bypass emitted-element tracking while draining
  under the staged coordinator path.
- This is small, but it removes another edge-case asymmetry while the final
  ownership-transfer contract is still deferred.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test mark_active_result_element_emitted_marks_linear_fallback_current_element -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether any other linear shared emit branch still bypasses
  emitted-element bookkeeping
- Whether the active-result helper keeps the emitted-element mark aligned with
  the current execution phase instead of hard-coding graph-only state
