# Review Request: Parallel Scan Stale-Slot Reap

Current head: `01fb487`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`

Problem:
- The prior Task 18 slice made coordinator selection/read/take claim-aware, so
  stale staged result slots were no longer exposed once their worker claim had
  dropped.
- But those dead staged slots still remained counted in
  `published_result_slots`, which meant the shared coordinator header could
  overstate staged availability until some later explicit clear/reset path ran.
- We needed coordinator refresh to reap those dead staged result slots as part
  of the staged contract before building the real shared top-K merge heap on
  top of it.

What changed:
- Added `reap_dead_parallel_scan_result_slots_with_attachment(...)` in
  `src/am/common/parallel.rs`.
- `refresh_coordinator_selection_snapshot(...)` now reaps staged result slots
  whose owning worker slot is no longer claimed for the active rescan epoch
  before recomputing the coordinator fast path.
- Reaping:
  - resets the dead staged result slot back to idle
  - decrements `published_result_slots`
  - advances `result_publish_generation`
- Tightened the claim-drop regression coverage so read/take now assert:
  - the stale staged slot is reset to idle
  - coordinator counters reflect the reap, not just claim-aware skipping
- Updated Task 18 notes so the staged status now explicitly includes dead-slot
  reaping from the shared published-result counts.

Still intentionally deferred:
- `amcanparallel` remains `false`
- no shared top-K heap mutation path yet
- no worker-local traversal scratch in DSM yet
- no planner-visible parallel execution yet

Validation:
- Passed:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `bash scripts/run_pg18_preload_pgstat_test.sh`

Review focus:
- Whether refresh-time reaping is the right place to keep staged-slot counts
  truthful before the shared top-K heap exists
- Whether reaping by reset-and-recount is the right lifetime contract for dead
  worker-owned staged results
- Whether this keeps the staged coordinator bookkeeping narrow and coherent for
  the next merge/drain slices
