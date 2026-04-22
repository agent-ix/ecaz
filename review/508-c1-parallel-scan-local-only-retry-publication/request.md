# Review Request: Parallel Scan Local-Only Retry Publication

Current head: `5f5211a`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- After packet 507, stable foreign-owner blockers only fell back to local emit
  after a repeated unchanged blocker, but the worker still left that same row
  published in the shared coordinator slot between retries.
- That kept an ineligible row visible to the shared coordinator even while the
  worker was treating it as a local-only fallback row.

What changed:
- Added a scan-local `parallel_local_only_output_active` flag.
- `publish_parallel_scan_worker_slot_snapshot(...)` now:
  - still publishes worker-runtime state
  - clears the coordinator result slot when the current row is in that
    local-only fallback state
- The flag is cleared when:
  - a new shared retry starts
  - a shared owned-output consume succeeds
  - a staged row is discarded
  - a new linear materialization begins
  - parallel bind/clear resets the scan
- Stable foreign-owner local emit paths now set the flag before republishing,
  so the row stays local-only between retries.
- Added focused coverage that:
  - local-only fallback keeps the worker snapshot active
  - the shared coordinator slot is cleared while local-only fallback is active

Why this matters:
- It makes the staged foreign-owner fallback contract more coherent:
  “local-only between retries” now means local-only in the shared publication
  surface too, not just local control flow.
- That reduces stale shared visibility for rows that are currently waiting on a
  foreign owner.

Still intentionally deferred:
- the real multi-worker output handoff / ownership transfer contract
- planner-visible parallel execution and `amcanparallel = true`
- the LWLock-based serializer replacement before real parallel enablement
- `n = 2/4/8` correctness coverage once the real multi-worker path lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether clearing the coordinator slot while retaining the worker snapshot is
  the right staged boundary for “local-only between retries”
- Whether the chosen clear/reset points for `parallel_local_only_output_active`
  align with the intended retry lifecycle
