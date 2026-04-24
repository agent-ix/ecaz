# Review Request: Parallel Scan N=2 Hidden-Slot Diagnostics

Current head: `6f6633e`

Scope:
- `src/am/ec_hnsw/scan_debug.rs`
- `src/lib.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged PG18 `n=2` round-robin gates already reported per-worker streams,
  worker snapshots, and local visited/emitted sets.
- That still left one visibility gap in the current ownership seam: when a row
  moved into a hidden local-only coordinator slot, the staged `n=2` failures
  had to infer that from the worker snapshot and local state instead of showing
  the shared hidden DSM slot directly.
- For the remaining multi-worker ownership work, that made it too easy to
  misread whether a row was still staged in shared hidden state or had already
  fallen back to purely local state.

What changed:
- Extended `debug_gettuple_scan_heap_tids_with_scores_parallel_round_robin_details(...)`
  so it now returns both workers' hidden local-only coordinator slot snapshots
  in addition to the existing worker snapshots and local visited/emitted sets.
- Threaded those new hidden-slot snapshots through both staged PG18 `n=2`
  round-robin regressions.
- Expanded the failure messages for the unique-row and duplicate-drain fixtures
  so any future staged `n=2` mismatch now prints the hidden DSM slot state
  alongside the worker snapshots and local ownership traces.
- Updated the Task 18 notes to record that the staged `n=2` diagnostics now
  include hidden-slot state.

Why this matters:
- The branch's remaining blocker is a shared ownership-transfer seam, not basic
- worker contribution or duplicate drain anymore.
- Hidden local-only DSM slots are part of that seam, so the staged `n=2`
  diagnostics need to show them explicitly when a failure happens.
- This does not change runtime behavior, but it should make the next real
  multi-worker ownership regression much cheaper to localize and reason about.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely live-blocked
  unique outputs
- planner-visible enablement and `amcanparallel = true`
- broader `n=4/8` correctness and measurement once the final ownership seam
  lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether hidden local-only coordinator slot snapshots are the right additional
  shared-state surface to include in the staged PG18 `n=2` failure output
- Whether the widened round-robin debug tuple keeps the staged diagnostics
  focused enough without turning the helper into an unstable dumping ground
