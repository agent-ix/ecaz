# Review Request: Parallel Scan Blocked-Owner Staging

Current head: `f2b9d68`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The prior owner-aware drain slice stopped a worker from mutating a foreign
  slot, but the graph and linear emit helpers still assumed that staging a new
  local row must always immediately produce an owned coordinator output.
- In the blocked state where a foreign admitted head stays ahead, those helpers
  could still panic even though the branch intentionally remains in staged
  shared-infra mode with `amcanparallel = false`.
- The direct local emit fallback also needed to republish the advanced local
  duplicate cursor so the shared snapshot would not drift after the local row
  drained outside the shared take helper.

What changed:
- `emit_materialized_parallel_scan_output(...)` now returns `None` instead of
  panicking when the owner-aware shared take does not currently yield an owned
  output.
- The graph and linear local-direct emit fallbacks now republish the worker
  slot snapshot after advancing local duplicate drain state.
- Added focused regressions for:
  - blocked local materialization under a foreign admitted head
  - blocked prefetched graph output under a foreign admitted head
- Updated Task 18 notes to record this blocked-owner staging fallback behavior.

Why this matters:
- This keeps the staging branch usable and honest while the final multi-worker
- output-handoff contract is still deferred.
- It also makes the blocked state explicit in tests instead of hiding it behind
  an impossible-output panic path.

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
- Whether the blocked-owner fallback is the right staged behavior while
  `amcanparallel` remains off
- Whether the republish-after-local-drain seam keeps the shared snapshot aligned
  enough for the remaining Task 18 work
