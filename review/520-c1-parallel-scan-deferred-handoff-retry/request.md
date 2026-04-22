# Review Request: Parallel Scan Deferred Handoff Retry

Current head: `6eb4c49`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Deferred blocked rows still drained locally by default once the scan phase
  exhausted.
- That meant the deferred stash preserved ordering and diagnostics better than
  eager `KeepLocalEmit`, but it still did not give the shared ownership seam a
  final chance to resolve before local fallback.

What changed:
- Deferred blocked rows now remember which scan phase produced them.
- Added phase-aware helpers so scan code can temporarily restore a deferred row
  into its original graph or linear active state.
- Deferred drain now retries the shared handoff seam once more before local
  emit:
  - if the foreign blocker has cleared, the shared handoff output drains first
  - the deferred local row is preserved and re-stashed if it still has pending
    heap tids afterward
  - only unresolved blockers fall back to local emit
- Added focused coverage for restoring a deferred linear-fallback row, draining
  a foreign selected handoff, and preserving the deferred row afterward.
- Updated Task 18 notes to describe the deferred-row shared retry contract.

Why this matters:
- It narrows the remaining ownership gap by probing the shared seam at the last
  possible point instead of draining every deferred row locally.
- The deferred stash now behaves more like a staging area for future ownership
  transfer instead of a terminal local-only queue.

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
- Whether giving deferred rows one last shared-handoff retry is the right
  interim contract before full ownership transfer lands
- Whether restoring deferred rows into their original graph/linear phase is the
  right staging shape for that retry
