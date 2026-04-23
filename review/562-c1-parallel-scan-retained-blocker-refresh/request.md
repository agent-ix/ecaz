# Review Request: Parallel Scan Retained Blocker Refresh

Current head: `deb4910`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Retained blocker metadata could go stale while the same foreign owner row
  stayed live but moved across shared states.
- Hidden local-only rows and deferred blocked rows could keep an out-of-date
  blocker kind or generation after that same foreign row republished on the
  same slot or moved from selected-pending into the admitted head.
- That stale metadata widened the false-ready window in the staged ownership
  model and made worker snapshots less trustworthy.

What changed:
- Added `refresh_retained_parallel_owned_output_blocker(...)` to normalize a
  retained blocker against the current shared owner state for the same foreign
  row.
- `resolve_local_only_parallel_scan_duplicate(...)` now:
  - preserves the existing same-element obsolete-owner drop before refresh
  - refreshes retained selected/admitted blockers in place
  - republishes the worker snapshot when blocker kind or generation changes
- `take_next_deferred_parallel_blocked_output(...)` now refreshes any retained
  blocker before deciding whether the deferred row is still blocked or ready.
- Added focused regressions proving:
  - a retained selected blocker refreshes to the newer selected generation when
    the same foreign row republishes
  - a retained selected blocker converts into an admitted-head blocker when the
    same foreign row moves from selected-pending into the admitted head
- Updated Task 18 notes to record in-place retained-blocker refresh.

Why this matters:
- Hidden and deferred rows now track the foreign owner row's current shared
  state instead of carrying stale blocker metadata forward.
- Worker snapshots now stay aligned with the blocker state actually driving the
  ownership decision.
- This narrows another false-ready case without claiming the final
  cross-worker ownership-transfer contract is complete.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test --lib resolve_local_only_parallel_scan_duplicate_refreshes_selected_blocker_generation -- --nocapture`
  - `cargo test --lib resolve_local_only_parallel_scan_duplicate_refreshes_selected_blocker_into_admitted_head -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether retained blockers now refresh only for the same foreign owner row and
  do not accidentally broaden across unrelated same-slot publications
- Whether hidden local-only and deferred paths now use current blocker kind and
  generation before falling back
- Whether the obsolete same-element drop still wins before any blocker refresh
