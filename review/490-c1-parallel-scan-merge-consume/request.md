# Review Request: Parallel Scan Merge Consume Wiring

Current head: `0bfb5c0`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/ec_hnsw/scan.rs`

Problem:
- The staged coordinator merge helper could already choose the next admitted or
  selected pending row in score order.
- But the scan output loop still emitted only from the local graph and linear
  paths, so the shared merge seam was not yet part of scan-side tuple
  production.

What changed:
- Added `try_take_parallel_scan_next_output(...)` on the scan side.
- When a parallel-scan descriptor is bound, the helper now:
  - republishes the local worker snapshot
  - drains the staged coordinator merge seam through
    `take_parallel_scan_next_output_snapshot(...)`
  - projects the admitted row back into `PendingScanOutput`
  - republishes the local worker snapshot after the owned duplicate-drain
    cursor advances
- `produce_next_scan_heap_tid(...)` now checks that staged merge helper first
  before falling back to the local graph or linear production paths.
- Added focused regression coverage for:
  - advancing an owned worker slot through the shared merge seam
  - republishing the next pending heap tid back into the shared result-slot
    snapshot after consume
- Updated Task 18 notes to record the scan-side merge consume seam.

Important staging note:
- This is still staged shared-infra work.
- The scan-side merge path currently uses descriptor-capacity admission because
  planner-visible LIMIT budgeting is not wired into the scan descriptor yet.
- `amcanparallel` is still `false`.

Still intentionally deferred:
- no final coordinator/worker execution loop yet
- no planner-visible parallel execution yet
- no LWLock-based serializer yet
- no planner LIMIT / cost plumbing for the admitted window

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether checking the staged coordinator merge seam first in
  `produce_next_scan_heap_tid(...)` is the right next integration point before
  real parallel execution is enabled
- Whether republishing after consume keeps the shared result-slot view aligned
  with the local duplicate-drain cursor
- Whether the descriptor-capacity admission note is explicit enough about the
  still-missing planner LIMIT wiring
