# Review Request: Parallel Scan Worker Bootstrap Diversification

Current head: `ee9b405`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/ec_hnsw/scan.rs`

Problem:
- The shared merge seam now stages local graph and linear outputs through the
  coordinator, but every worker would still seed the bootstrap frontier from
  the exact same ordered layer-0 candidate list.
- That keeps the staged execution contract too close to single-worker behavior
  and leaves the existing `scan_seed` / claimed worker-slot seam unused.

What changed:
- Added `parallel_scan_worker_bootstrap_candidates(...)`.
- When a parallel descriptor is bound with more than one worker slot, the
  helper now:
  - keeps the shared best bootstrap candidate as the first seed for every worker
  - derives a worker-local splitmix64 seed from `scan_seed`, worker-slot index,
    and worker-slot count
  - rotates the remaining layer-0 bootstrap tail by that derived seed
  - strides the rotated tail by worker slot so each worker starts from a
    different staged bootstrap subset
- Unbound scans and single-worker scans keep the original ordered candidate
  list unchanged, preserving serial / `n=1` behavior.
- Added focused unit coverage for:
  - serial-order preservation when no parallel descriptor is bound
  - worker-tail diversification while retaining the shared best seed candidate
- Updated Task 18 notes to record the staged worker-bootstrap diversification
  seam.

Important staging note:
- `amcanparallel` remains `false`.
- This is still bootstrap staging, not final planner-visible parallel query
  execution.
- The later `ef_search` overlap / ownership / planner-cost work remains open.

Still intentionally deferred:
- no final coordinator/worker execution loop yet
- no planner-visible LIMIT or cost plumbing yet
- no LWLock-based serializer yet
- no planner-visible parallel execution yet

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the shared-best-plus-diversified-tail seeding shape is the right
  staging contract before full worker ownership and `ef_search` budgeting land
- Whether the helper preserves the intended serial / `n=1` invariants
- Whether the new seam is narrow enough to support later planner-visible
  enablement without forcing a redesign
