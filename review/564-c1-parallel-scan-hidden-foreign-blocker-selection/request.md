# Review Request: Parallel Scan Hidden Foreign Blocker Selection

Current head: `2753d76`

Scope:
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Hidden local-only rows were already preserved in DSM and handoff-capable once
  a caller explicitly targeted the right slot and element.
- But `read_parallel_scan_owned_output_state(...)` still only blocked on the
  selected fast path or the admitted head.
- That meant a better hidden foreign row could still be ignored by the owned
  read path, leaving the worker to advance or fall back locally even though the
  foreign row was already transferable through the hidden-slot handoff helper.

What changed:
- Added a coordinator-locked scan for the best live hidden foreign row that
  outranks the owner's pending output.
- `read_parallel_scan_owned_output_state(...)` now returns that better hidden
  row as a blocker instead of treating the owner as ready.
- Reused the existing hidden-slot handoff path by surfacing the blocker as a
  `ForeignSelectedPending` blocker with the hidden slot and element identity.
- Hardened the legacy foreign-selected handoff regression test so it reads the
  actual blocker from shared state instead of hardcoding a stale generation.
- Added focused regressions proving:
  - the owned read path reports a better hidden foreign row as blocked
  - `try_take_parallel_scan_next_output(...)` drains that hidden foreign row
    before advancing the owner
- Updated Task 18 notes to record that better hidden foreign rows now surface
  as blockers.

Why this matters:
- This is a concrete ownership-transfer step, not just more local fallback
  cleanup.
- Hidden foreign rows are now visible to the owned shared read path when they
  really should block the owner, so the existing hidden-slot handoff helper can
  consume them through the shared seam.
- It narrows the remaining blocker to the harder case: genuinely blocked unique
  rows that still need a true cross-worker takeover contract.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for blocked unique outputs
  whose owner row itself must be taken over by another worker
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the ownership contract lands

Validation:
- Passed:
  - `cargo test --lib read_parallel_scan_owned_output_state_reports_blocked_for_better_hidden_foreign_row -- --nocapture`
  - `cargo test --lib try_take_parallel_scan_next_output_drains_better_foreign_hidden_row -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the hidden-blocker scan is correctly scoped to better live hidden
  foreign rows and does not make hidden rows coordinator-visible
- Whether reusing `ForeignSelectedPending` for this blocker stays coherent with
  the hidden-slot handoff and retained-blocker refresh paths
- Whether the scan-side next-output regression proves a real shared handoff
  instead of another local-only wakeup
