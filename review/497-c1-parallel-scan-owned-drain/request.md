# Review Request: Parallel Scan Owner-Aware Drain

Current head: `61fb1b5`

Scope:
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged parallel merge path still let any worker mutate the globally
  selected pending output, even when that output belonged to another worker
  slot.
- The same global take path could also drain an admitted head sourced from a
  foreign worker slot.
- That made the current Task 18 seam unsafe for real multi-worker execution:
  a worker could advance a peer's duplicate-drain cursor just by probing the
  shared merge helper.

What changed:
- Extracted the selected-pending mutation logic into a locked helper so the
  same slot-advance path is reused consistently.
- Added `take_parallel_scan_owned_next_output_snapshot(...)` in the shared
  parallel layer. It only mutates staged pending or admitted outputs when the
  owning worker slot matches the caller.
- Switched scan-side output take to that owner-aware helper instead of the
  old global take helper.
- Added focused regressions for:
  - foreign selected slots staying unchanged under an owned probe
  - an owned better pending output overtaking a foreign admitted head without
    draining the foreign slot
- Updated Task 18 notes to record that the staged drain is now owner-aware.

Why this matters:
- This closes the concrete correctness hole behind the current runtime blocker:
  probing the shared merge seam no longer mutates a peer worker's local
  duplicate state.
- It also narrows the remaining Task 18 work to the bigger execution contract
  questions instead of a known cross-slot mutation bug in the staged drain
  helper.

Still intentionally deferred:
- true multi-worker planner-visible execution
- final output ownership / handoff semantics across workers
- `amcanparallel = true`
- planner costing, LIMIT budgeting, EXPLAIN rollups, and `n = 2/4/8`
  correctness coverage

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the owner-aware take helper is the right staged boundary for scan-side
  consumption before the full multi-worker contract lands
- Whether the new regressions pin the actual cross-slot mutation hazard clearly
