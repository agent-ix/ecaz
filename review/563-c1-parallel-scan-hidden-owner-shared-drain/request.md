# Review Request: Parallel Scan Hidden Owner Shared Drain

Current head: `76ec1cf`

Scope:
- `src/am/common/parallel.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Once a worker hid its own local-only row in the shared DSM slot, the owned
  parallel read/take helpers still treated that slot as `Empty`.
- That meant the owner could preserve the row in DSM for later work, but it
  still could not re-enter the owned shared admission/take seam from that
  hidden state.
- The hidden wakeup path therefore stayed more dependent on direct local
  fallback than the shared state warranted.

What changed:
- `read_parallel_scan_owned_output_state(...)` now treats a live hidden
  local-only owner slot as a valid owned pending output instead of only looking
  at published selected slots.
- Added `take_hidden_local_only_pending_output_locked(...)` so
  `take_parallel_scan_owned_next_output_snapshot(...)` can consume the owner's
  hidden pending output under the coordinator lock.
- Fixed the read-path bug where "no selected slot at all" was still being
  reported as a `ForeignSelectedPending` blocker.
- Added focused regressions proving:
  - a hidden local-only owner slot reports `Ready`
  - the owning worker can drain that hidden slot through the shared owned-take
    path and the hidden runtime state clears afterward
- Updated Task 18 notes to record that hidden owner rows now re-enter the owned
  shared seam.

Why this matters:
- Hidden local-only rows are now more than passive DSM bookkeeping; the owning
  worker can actually rejoin the shared admission/take contract from that
  hidden state.
- This narrows the remaining ownership gap without claiming any new foreign
  worker ownership transfer.
- It gives the later cross-worker handoff seam a cleaner base because hidden
  owner rows no longer disappear from the owner's own shared state machine.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test --lib take_parallel_scan_owned_next_output_snapshot_drains_hidden_local_only_owner_slot -- --nocapture`
  - `cargo test --lib read_parallel_scan_owned_output_state_reports_ready_for_hidden_local_only_owner_slot -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether hidden local-only owner slots now participate in the owner's shared
  read/take path without accidentally becoming visible as foreign selected work
- Whether the new hidden-slot take logic advances or clears hidden runtime
  state correctly under the coordinator lock
- Whether the `ForeignSelectedPending` read-path fix is scoped narrowly to the
  "no selected slot" case
