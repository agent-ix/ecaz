# Review Request: Parallel Scan Deferred Blocker Metadata

Current head: `ee79aa1`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The deferred blocked-output stash introduced in packet 518 still flattened
  blocker visibility back down to one global retained blocker.
- If multiple blocked local rows accumulated, worker-runtime snapshots could
  only describe whichever blocker happened to remain in the global retained
  slot, not the blocker attached to the best deferred row.

What changed:
- Replaced `Vec<ScanResultState>` deferred stash entries with
  `DeferredParallelBlockedOutput`, which carries both the staged
  `ScanResultState` and the retained blocker metadata for that row.
- Added shared ordering helpers so deferred-row selection and snapshot
  publication both use the same score/element ordering.
- Worker-runtime snapshot publication now reports the blocker attached to the
  best deferred blocked row when no active local-only row is present.
- Updated stash tests to assert that blocker metadata moves into the deferred
  entry, and added focused coverage for the “best deferred blocker wins”
  snapshot case.
- Updated the Task 18 note to describe the per-row deferred blocker contract.

Why this matters:
- It keeps blocker diagnostics aligned with the actual deferred row that would
  be emitted next.
- It narrows one more mismatch between the scan-local deferred fallback and
  the eventual full ownership-transfer design.

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
- Whether per-row deferred blocker metadata is the right staged shape for the
  deferred stash
- Whether publishing the blocker from the best deferred row is the right
  diagnostic contract while the final ownership-transfer seam is still pending
