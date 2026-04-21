# Review Request: Parallel Scan Capacity-Based ef_search Split

Current head: `f41409c`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/ec_hnsw/options.rs`
- `src/am/ec_hnsw/scan.rs`

Problem:
- Parallel bootstrap diversification is now live, but bound scans still sized
  the bootstrap frontier directly from serial `effective_ef_search`.
- That left the staged parallel runtime ignoring the Task 18 overlap/budget
  split contract entirely.

What changed:
- Added `ec_hnsw.parallel_ef_overlap` as a float GUC with default `0.1`
  and range `[0.0, 0.5]`.
- Added `resolve_parallel_scan_ef_search(...)` in `options.rs`.
- The helper currently uses the shared descriptor's worker-slot capacity as the
  staging stand-in for actual worker count:
  - keep serial `effective_ef_search` when capacity is `0` or `1`
  - otherwise compute `ceil(effective_ef_search / worker_slots)` and apply the
    configured overlap multiplier
- `ec_hnsw_amrescan(...)` now routes bootstrap frontier sizing through
  `resolve_bootstrap_frontier_limit(...)` instead of copying serial
  `effective_ef_search` directly.
- Added focused unit coverage for:
  - serial/no-parallel fallback
  - split-plus-overlap math
  - scan-side frontier-limit resolution
- Updated Task 18 notes to mark this as a staged capacity-based budget split,
  not the final executor-visible worker-count contract.

Important staging note:
- This is intentionally capacity-based, not actual-worker-count-based.
- The shared descriptor only exposes reserved worker-slot capacity at this seam.
- Planner-visible LIMIT/cost integration and true executor worker-count
  budgeting remain deferred.

Still intentionally deferred:
- actual executor-visible worker count in budget math
- planner-visible cost/LIMIT exposure for the split budget
- `amcanparallel = true`
- final coordinator/worker execution ownership and lock refinement

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether capacity-based runtime budget split is the right staging seam before
  executor-visible worker count is wired in
- Whether the overlap GUC surface is scoped correctly for staged runtime use
- Whether routing scan bootstrap sizing through the helper preserves serial
  behavior while making the parallel path meaningfully closer to the ADR/task
  contract
