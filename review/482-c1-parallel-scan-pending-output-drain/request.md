# Review Request: Parallel Scan Pending-Output Drain

Current head: `fef815b`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`

Problem:
- The staged coordinator result slots previously exposed only one selected
  result snapshot at a time.
- That was enough for staged slot selection, but not for the next coordinator
  seam: workers already own duplicate-drain state and can have multiple pending
  heap TIDs behind one current result.
- Clearing the whole staged slot after the first emitted heap TID would lose
  the rest of that worker result's pending output, while leaving the slot
  untouched would keep the coordinator fast path pointed at stale heap-TID
  state.

What changed:
- Extended `EcParallelCoordinatorResultSlot` so each staged slot now carries
  the full inline pending-output heap-TID buffer, not just the current heap
  TID.
- Extended the runtime snapshot mirror so scan-side publishing can stage the
  worker's pending duplicate-drain state into the shared slot.
- Added a coordinator helper that takes exactly one pending heap TID from the
  currently selected staged slot:
  - if more pending heap TIDs remain, it advances the slot in place to the next
    pending output and refreshes the cached selection metadata
  - if no pending heap TIDs remain, it clears the slot and refreshes selection
- Updated the scan-side publish path so staged worker snapshots publish the
  first pending heap TID as the live heap-TID fast path while retaining the
  rest of the inline pending-output buffer in shared state.
- Added focused regressions for:
  - draining within one slot without clearing the rest of its pending output
  - clearing the slot only after the last pending heap TID is emitted
  - scan-side staging of pending duplicate-drain state into the shared slot
- Updated Task 18 notes to state that worker-frontier staged slots now carry
  full pending-output state and that coordinator drain can advance within one
  slot without clearing it early.

Still intentionally deferred:
- `amcanparallel` remains `false`
- no shared top-K admission heap yet
- no planner-visible parallel execution yet
- no worker-owned traversal scratch in DSM yet

Validation:
- Passed:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `bash scripts/run_pg18_preload_pgstat_test.sh`

Review focus:
- Whether staging the full inline pending-output heap-TID buffer in the shared
  worker-result slot is the right boundary before the later shared top-K heap
  work lands
- Whether the coordinator's one-at-a-time pending-output take contract is the
  right staged behavior for duplicate-drain semantics
- Whether the scan-side publish path now mirrors enough worker result state to
  keep coordinator drain coherent across slot republish and rescan
