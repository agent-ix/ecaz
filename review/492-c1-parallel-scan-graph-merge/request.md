# Review Request: Parallel Scan Graph Merge Staging

Current head: `d99d863`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/ec_hnsw/scan.rs`

Problem:
- Bound parallel scans could already drain staged coordinator output before
  local scan production.
- But prefetched graph-traversal rows still emitted directly from the local
  graph result state on first emit instead of flowing through the shared merge
  seam.

What changed:
- Added `emit_prefetched_parallel_scan_output(...)`.
- When a parallel-scan descriptor is bound and graph traversal already has a
  prefetched row, the helper now:
  - republishes that prefetched row into the shared result-slot snapshot
  - drains it through the staged coordinator merge helper
  - leaves the local graph duplicate-drain cursor advanced and republished for
    the next duplicate
- `produce_next_graph_traversal_heap_tid(...)` now checks that helper before
  falling back to direct local prefetched-output emit behavior.
- Added focused regression coverage for the prefetched graph seam and updated
  Task 18 notes to record that prefetched graph rows no longer bypass the
  shared merge path on first emit.

Important staging note:
- This closes the graph-side twin of the earlier linear-fallback seam.
- `amcanparallel` remains `false`.
- Planner-visible LIMIT budgeting is still not wired, so the admitted window
  still uses descriptor-capacity admission.

Still intentionally deferred:
- no final coordinator/worker execution loop yet
- no planner-visible parallel execution yet
- no LWLock-based serializer yet
- no planner LIMIT / cost plumbing for the admitted window
- real multi-worker traversal ownership and planner exposure still remain

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether routing prefetched graph rows through the shared merge helper is the
  right final first-emit constraint before real parallel execution is enabled
- Whether the helper keeps the serial graph path unchanged when no parallel
  descriptor is bound
- Whether the remaining open work is now clearly concentrated in execution
  ownership, planner exposure, and lock/runtime refinement
