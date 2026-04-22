# Review Request: Parallel Scan Deferred Blocked Output

Current head: `44292e4`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- A stable foreign-owner blocker still collapsed into `KeepLocalEmit`, which
  meant a worker could immediately emit its own blocked local row out of the
  shared merge order as soon as the retry budget closed.
- That preserved forward progress, but it also kept the ownership seam
  artificially weak: the row stopped participating in shared staging while it
  was being emitted locally.

What changed:
- Added a scan-local deferred blocked-output stash for blocked local
  `ScanResultState` rows that still have pending heap tids.
- Stable `KeepLocalEmit` blockers now stash the active local row instead of
  immediately emitting it.
- Deferred blocked rows still count as staged for duplicate suppression and
  keep retained blocker metadata visible in worker diagnostics.
- Once the shared seam is exhausted, scan-side drain emits the best deferred
  blocked row and continues draining any remaining duplicates from that stashed
  state.
- Reset paths now clear the deferred stash, and Task 18 notes now describe the
  deferred blocked-output contract.

Why this matters:
- It removes the eager out-of-order local emit fallback from the stable
  blocker path.
- The remaining ownership gap is narrower and more explicit: the deferred row
  stays scan-local, but it no longer bypasses the shared seam immediately.

Still intentionally deferred:
- full cross-worker ownership transfer instead of scan-local deferred fallback
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement after the remaining ownership seam
  lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether scan-local deferred blocked rows are the right interim replacement
  for eager `KeepLocalEmit`
- Whether the stash still preserves the right duplicate-suppression and
  blocker-visibility invariants while the final ownership-transfer seam is
  still deferred
