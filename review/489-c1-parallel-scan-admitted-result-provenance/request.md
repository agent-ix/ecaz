# Review Request: Parallel Scan Admitted Result Provenance

Current head: `7bf79b8`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`

Problem:
- The coordinator could already order and drain admitted rows.
- But the admitted window flattened away source worker-slot and element
  provenance, so scan-side code had no way to map an admitted row back onto the
  local duplicate-drain state that produced it.

What changed:
- Extended the admitted-result DSM snapshot to retain:
  - source worker-slot index
  - source element TID
- Updated admitted-result load/store/reset paths to preserve that provenance
  through admission, shifting, and drain.
- Added `consume_parallel_scan_admitted_result(...)` on the scan side.
- The scan-side helper now:
  - projects an admitted coordinator row back into `PendingScanOutput`
  - advances the local duplicate-drain cursor when the admitted row came from
    this worker slot and still matches the current local result state
  - leaves unrelated worker-local state untouched
- Added focused regression coverage in both `parallel.rs` and `scan.rs`.
- Updated Task 18 notes to record the admitted-result provenance seam.

Important staging note:
- This lands the provenance bridge only.
- It does not yet wire the helper into `produce_next_scan_heap_tid(...)` or
  enable planner-visible parallel execution.

Still intentionally deferred:
- `amcanparallel` remains `false`
- no final coordinator/worker execution loop yet
- no planner-visible parallel execution yet
- the serializer is still the staged raw lock word rather than the later
  LWLock follow-up

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether admitted-result provenance is the right minimal seam before wiring the
  scan-side consume path into the real execution loop
- Whether the scan-side helper advances only the owned duplicate-drain cursor
  and stays benign for non-owned admitted rows
- Whether the staging note is clear that this is still pre-`amcanparallel`
