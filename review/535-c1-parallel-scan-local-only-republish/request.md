# Review Request: Parallel Scan Local-Only Republish

Current head: `3d2a606`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- A row hidden in `parallel_local_only_output_active` stayed visible in the
  worker snapshot but intentionally cleared its coordinator result slot.
- On the next shared retry, `try_take_parallel_scan_next_output(...)` cleared
  the local-only flag and immediately ran owner-slot reconciliation before
  republishing the row.
- That reconciliation saw an empty coordinator slot plus a still-live worker
  snapshot and misclassified the hidden row as a stale fully drained owner,
  clearing the local cursor and returning `Empty`.

What changed:
- `try_take_parallel_scan_next_output(...)` now detects the
  "waking local-only output" transition.
- In that one transition, it publishes the worker/coordinator snapshots first
  instead of calling `sync_and_publish_parallel_scan_worker_slot_snapshot(...)`.
- That avoids stale-owner reconciliation against an intentionally blank
  coordinator slot while the row is still hidden.
- Added focused coverage proving:
  - a local-only row is hidden from the coordinator published set before retry
  - once the foreign blocker clears, the row republishes into the shared slot
  - the shared path emits the row's next heap tid and republishes the remaining
    duplicate afterward
- Updated Task 18 notes to record that local-only wakeup now republished into
  shared state.

Why this matters:
- It closes a real correctness gap in the remaining ownership-transfer staging.
- Local-only fallback is now a temporary concealment, not a one-way drop out of
  the shared path after the blocker clears.
- That narrows the remaining gap to the genuinely still-blocked unique rows,
  which is the seam still blocking planner-visible enablement.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique deferred outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining ownership seam lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether skipping reconciliation on the exact local-only wakeup transition is
  the right fix boundary, versus teaching reconciliation itself about hidden
  local-only rows
- Whether the new republish test adequately proves that the row re-enters the
  shared coordinator path instead of only staying alive locally
