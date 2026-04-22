# Review Request: Parallel Scan Blocker Snapshots

Current head: `353b1db`

Scope:
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged ownership blocker (`ForeignSelectedPending`,
  `ForeignAdmittedHead`, `AdmissionWindow`) existed only in local scan control
  flow.
- Shared worker-runtime snapshots still exposed frontier/visited/emitted state
  but not the reason a worker was blocked, which left the remaining ownership
  handoff seam invisible to shared diagnostics.
- That made multi-worker follow-up work harder to reason about because blocked
  state had to be inferred indirectly from result-slot side effects.

What changed:
- Extended the shared worker-slot runtime snapshot with:
  - blocker kind code
  - blocker owner slot index
- Bumped the AM-private parallel descriptor version for the worker-slot layout
  change.
- `TqScanOpaque` now retains the current blocked-owner state long enough to
  publish it into the shared worker snapshot.
- `try_take_parallel_scan_next_output(...)` now:
  - clears blocker state before the optimistic publish/read cycle
  - republishes the shared worker snapshot when a blocked-owner state is
    returned
  - clears blocker state again on successful consume or explicit discard
- Added focused coverage that blocked materialized/prefetched staging now
  leaves the blocker reason visible in the shared worker snapshot.

Why this matters:
- It turns the current ownership blocker into explicit shared runtime state
  instead of a purely local branch condition.
- That gives the next multi-worker handoff slice a real, inspectable seam
  without pretending the final ownership contract is already complete.

Still intentionally deferred:
- the real multi-worker output handoff / ownership contract
- planner-visible parallel execution and `amcanparallel = true`
- `n = 2/4/8` correctness coverage once the real multi-worker path lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether publishing blocker kind/slot in the worker snapshot is the right
  shared-state seam for the remaining ownership handoff work
- Whether blocker state now clears at the right boundaries when local emit or
  discard paths move back out of the blocked state
