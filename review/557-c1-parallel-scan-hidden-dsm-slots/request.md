# Review Request: Parallel Scan Hidden DSM Slots

Current head: `1e133de`

Scope:
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Hidden local-only rows were being represented by clearing the shared
  coordinator result slot outright.
- That kept them out of coordinator selection, but it also threw away the
  shared runtime snapshot for the still-staged row.
- The missing shared row state made the remaining ownership-transfer seam
  harder to reason about because DSM no longer reflected the hidden local-only
  row at all.

What changed:
- Added an explicit hidden-slot flag,
  `EC_PARALLEL_RESULT_SLOT_HIDDEN_LOCAL_ONLY`, for shared result-slot runtime
  that should stay visible in DSM but invisible to coordinator selection.
- Split result-slot publication so scan-side code can publish either:
  - a normal coordinator-visible result slot, or
  - a hidden local-only slot via
    `publish_hidden_parallel_scan_coordinator_result_slot_runtime_snapshot(...)`
- Hidden slot publication now:
  - preserves the runtime snapshot in DSM
  - keeps the slot out of `published_result_slots`
  - keeps it out of staged-heap selection
  - avoids bumping `result_publish_generation` for hidden-only updates
- Reap and clear paths now understand hidden slots so dead worker claims still
  clean up correctly without corrupting published-slot bookkeeping.
- `publish_parallel_scan_worker_slot_snapshot(...)` now keeps the result-slot
  runtime for hidden local-only rows instead of clearing the slot outright.
- Updated focused scan/common tests to assert that hidden rows remain readable
  in shared runtime while coordinator-visible publication still stays empty.
- Updated Task 18 notes to record the new hidden-slot DSM contract.

Why this matters:
- Hidden local-only rows now remain inspectable in shared memory without
  leaking into coordinator selection.
- That keeps worker/runtime state available for the next ownership-transfer
  slices while preserving the current coordinator invariants.
- The no-generation-bump rule for hidden-only updates also prevents stale
  blocker detection from misclassifying a hidden-row refresh as a visible
  publish event.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test publish_hidden_parallel_scan_coordinator_result_slot_runtime_snapshot_keeps_slot_unpublished -- --nocapture`
  - `cargo test publish_parallel_scan_worker_slot_snapshot_hides_local_only_output_from_coordinator -- --nocapture`
  - `cargo test resolve_local_only_parallel_scan_duplicate_handoffs_live_foreign_duplicate -- --nocapture`
  - `cargo test resolve_local_only_parallel_scan_duplicate_exhausts_last_duplicate -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether hidden local-only rows now stay staged in DSM without entering the
  coordinator published heap or selected fast path
- Whether hidden-only slot updates correctly avoid `result_publish_generation`
  churn and stale-blocker regressions
- Whether reap/clear bookkeeping still handles hidden slots safely when worker
  claims die or local-only state clears
