# Review Request: Parallel Scan Post-Handoff Republish Reconciliation

Current head: `bb4a1e2`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Foreign selected-pending handoff is live, but after another worker drained a
  selected pending row through the shared seam, the owning worker could still
  republish from stale local `ScanResultState`.
- That let the owner restage the already-drained row on its next worker-slot
  publish instead of reconciling to the advanced shared pending index first.

What changed:
- Added `sync_and_publish_parallel_scan_worker_slot_snapshot(...)` on the scan
  side.
- Mutable worker-slot publish call sites now reconcile against the worker's own
  shared result slot before publishing the next snapshot.
- Tightened `reconcile_parallel_owner_progress_from_shared_slot(...)` so an
  empty shared slot only clears local state when it represents genuine stale
  foreign drain rather than the normal transient empty slot after an owned
  local take.
- Added focused regression coverage for:
  - a foreign worker handing off the first selected pending row
  - the owner republishing after that handoff
  - a second handoff draining the next pending row instead of re-emitting the
    already-drained first row
- Task 18 notes now record the post-handoff republish reconciliation seam.

Why this matters:
- It closes the immediate post-handoff duplicate-restaging gap in the staged
  ownership model.
- Without this, the foreign selected-pending handoff seam was correct for the
  first drain but could regress on the next owner publish by putting the stale
  row back into the shared slot.

Still intentionally deferred:
- full multi-worker ownership transfer beyond staged handoff plus republish
  reconciliation
- planner-visible parallel execution and `amcanparallel = true`
- `n = 2/4/8` correctness coverage once the full multi-worker path lands
- the LWLock release-path follow-up noted in packet 511

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether reconciling shared owner progress in the mutable publish path is the
  right staged boundary for post-handoff state repair
- Whether the tightened empty-shared-slot guard correctly distinguishes stale
  foreign drain from normal owned local-take progress
