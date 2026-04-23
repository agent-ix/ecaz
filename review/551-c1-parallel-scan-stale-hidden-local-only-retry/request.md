# Review Request: Parallel Scan Stale Hidden Local-Only Retry

Current head: `63d80e9`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The previous stale-blocker slices cleared dead retained blocker metadata and
  let stale-blocked deferred rows regain ordering priority.
- But a hidden `parallel_local_only_output_active` row whose retained blocker
  had already gone stale still woke by falling straight back into the local
  wakeup path.
- That bypassed the staged shared handoff seam at the exact point where the
  foreign blocker had already cleared.

What changed:
- Added `try_take_republished_local_only_parallel_output(...)` so a hidden
  local-only row with no live retained blocker retries the shared
  next-output/merge seam before any direct local emit.
- Wired both graph-traversal and linear-fallback wakeup paths through that
  helper.
- If the shared retry emits, the row drains under the staged coordinator path.
- If the shared retry is still blocked, the wakeup path now routes through the
  existing blocked-owner disposition instead of silently dropping back into a
  direct local wakeup emit.
- Added focused coverage proving a stale hidden local-only row:
  - stays unpublished while hidden
  - retries the shared seam once the blocker is gone
  - clears the local-only flag and advances the local duplicate cursor on that
    shared retry
- Updated Task 18 notes to record that stale hidden local-only rows now retry
  shared handoff before any direct local wakeup emit.

Why this matters:
- A cleared blocker now re-enters the staged coordinator path instead of
  immediately degrading into another local-only wakeup emit.
- This narrows one more stale-blocker edge case before the final
  cross-worker ownership-transfer contract lands.
- The remaining local-only fallback cases are now more clearly limited to
  genuinely still-live foreign blockers.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test try_take_republished_local_only_parallel_output_retries_stale_hidden_row -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether stale hidden local-only rows now rejoin the staged shared path at the
  right wakeup seam
- Whether the new blocked wakeup handling preserves the existing
  blocked-owner/disposition behavior without creating a direct local bypass
