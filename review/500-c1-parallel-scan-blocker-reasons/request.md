# Review Request: Parallel Scan Blocker Reasons

Current head: `68a60dc`

Scope:
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The prior owner-output state seam made blocked vs empty explicit, but the
  blocked state still carried no reason.
- That left the next multi-worker handoff work guessing whether a worker was
  blocked by:
  - a foreign selected pending output
  - a stronger foreign admitted head
  - losing to the current admitted window entirely
- Without that taxonomy, the scan layer still had to reverse-engineer ownership
  outcomes from coordinator side effects.

What changed:
- Added `EcParallelOwnedOutputBlockerKind` with:
  - `ForeignSelectedPending`
  - `ForeignAdmittedHead`
  - `AdmissionWindow`
- Added `EcParallelOwnedOutputBlocker` and made
  `EcParallelOwnedOutputState::Blocked(...)` carry blocker metadata.
- Updated `read_parallel_scan_owned_output_state(...)` to report the specific
  blocker kind and relevant foreign slot index when one exists.
- Updated scan-side `ParallelScanOutputState::Blocked(...)` to carry the same
  blocker metadata through the staging seam.
- Added focused coverage for:
  - foreign selected pending blocker
  - foreign admitted-head blocker
  - admission-window loser blocker
  - scan-side blocked materialized/prefetched emit paths
- Updated Task 18 notes to record the explicit blocker taxonomy.

Why this matters:
- The remaining worker/consumer handoff work now has a concrete blocker model
  instead of a generic blocked bit.
- This keeps the staged shared-merge seam honest about what actually prevents a
  worker from draining output at a given moment.

Still intentionally deferred:
- the actual multi-worker output handoff / ownership contract
- planner-visible parallel execution and `amcanparallel = true`
- `n = 2/4/8` correctness coverage once the real multi-worker path lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the blocker taxonomy is the right shared boundary for the remaining
  ownership handoff work
- Whether any blocked cases still collapse distinct coordination outcomes into a
  single blocker kind
