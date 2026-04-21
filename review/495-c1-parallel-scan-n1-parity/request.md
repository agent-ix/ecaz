# Review Request: Parallel Scan N=1 Parity

Current head: `bd6408d`

Scope:
- `src/am/ec_hnsw/scan_debug.rs`
- `src/am/ec_hnsw/scan.rs`
- `src/am/ec_hnsw/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`

Problem:
- Task 18 had staged shared-merge plumbing, but no PG18 executor-path parity
  coverage for a parallel-bound scan with exactly one worker slot.
- That left a real gap between "shared merge seams compile" and "the bound
  scan path preserves the serial emitted stream when no actual parallel split
  should change semantics."
- While adding that coverage, the graph-side emit path exposed a real bug:
  draining a shared admitted output before the graph emitter ran could skip the
  graph prefetch refresh that the next graph emit expects.

What changed:
- Added a test-only debug parallel-scan harness that allocates a real
  `ParallelIndexScanDescData` plus ecaz AM-private descriptor and binds it to
  the existing heap-backed debug scan path.
- Added parallel-bound debug helpers for:
  - heap-tid plus score streams
  - heap-tid plus approximate/comparison-score streams
- Added PG18 parity coverage for `n=1`:
  - scalar ordered scan parity against the serial emitted `(heap_tid, score)`
    stream
  - PqFastScan ordered scan parity against the serial emitted
    `(heap_tid, approx_score, comparison_score, approx_rank)` stream
- Fixed the graph traversal path so that when a bound scan consumes a shared
  admitted output through the coordinator merge seam, it repopulates prefetched
  graph output before the next graph-side emit if graph traversal is still
  active.

Why this matters:
- `n=1` is the first hard correctness bar for Task 18.
- If a parallel-bound scan with one worker slot does not stay byte-identical to
  the serial stream, later `amcanparallel` enablement is not defensible.
- The graph prefetch refill bug was a real runtime defect on the staged merge
  path, not just a missing test.

Still intentionally deferred:
- real multi-worker traversal ownership
- planner-visible costing and `amcanparallel = true`
- EXPLAIN rollups and measurement passes

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the debug parallel descriptor harness is the right staging seam for
  `n=1` executor-path parity coverage
- Whether the graph-side refill fix is in the correct place in scan control
  flow
- Whether the new parity assertions are strong enough to protect future
  `amcanparallel` bring-up
