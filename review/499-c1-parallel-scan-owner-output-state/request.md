# Review Request: Parallel Scan Owner Output State

Current head: `a5e297a`

Scope:
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The prior blocked-owner staging slice used `Option<PendingScanOutput>` for the
  scan-side shared take seam.
- That collapsed two materially different states into `None`:
  - the owner has no staged output
  - the owner does have staged output, but a foreign admitted head or selected
    pending output is legitimately blocking delivery
- That ambiguity made the remaining multi-worker ownership work harder to reason
  about and left the scan layer inferring blocked-vs-empty from side effects.

What changed:
- Added `EcParallelOwnedOutputState` in `parallel.rs` with explicit
  `Empty` / `Ready` / `Blocked` outcomes.
- Added `read_parallel_scan_owned_output_state(...)` as the coordinator-side
  readiness probe for one worker slot.
- Added `ParallelScanOutputState` in `scan.rs` with explicit
  `Empty` / `Blocked` / `Emitted(PendingScanOutput)` outcomes.
- Updated the scan-side emit/take helpers to route through that explicit state
  machine instead of overloading `Option::None`.
- Renamed the blocked-owner scan tests to say `reports_blocked...` instead of
  the stale `returns_none...` wording.
- Updated Task 18 notes to record the explicit owner readiness/output state seam.

Why this matters:
- The remaining multi-worker handoff work now has a real scan-layer state
  machine instead of a hidden `None` contract.
- Blocked-owner waits are explicit and testable, which makes it easier to
  separate “nothing to emit” from “work exists but another worker currently owns
  the front of the merged output.”

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
- Whether the explicit owner readiness/read state machine is the right boundary
  between the shared coordinator seam and the scan layer
- Whether any remaining `Option`-shaped ambiguity still leaks into the blocked
  owner path
