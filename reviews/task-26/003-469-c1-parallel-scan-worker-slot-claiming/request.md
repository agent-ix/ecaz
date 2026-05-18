# Review Request: Parallel Scan Worker-Slot Claiming

Current head: `54a4dd1`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`

Problem:
- The prior Task 18 slice established the shared DSM descriptor layout, but
  `TqScanOpaque` still did not claim a stable worker slot from that descriptor.
- Without explicit claim/release semantics, later coordinator and worker-local
  traversal state would either race across scans or need another seam rewrite
  when `amcanparallel` eventually flips on.
- Rescan and teardown paths also needed an epoch-aware release contract so
  stale scan state could drop out harmlessly once the shared descriptor resets.

What changed:
- Converted coordinator and worker-slot claim state in
  `src/am/common/parallel.rs` to atomics.
- Added shared helpers to:
  - claim the first free worker slot for the current rescan epoch
  - release a claimed slot exactly once
  - ignore stale-epoch releases after a parallel rescan
- `TqScanOpaque` now tracks its claimed worker-slot index alongside the shared
  descriptor pointer, epoch, and slot capacity.
- Scan attachment now claims one worker slot on bind, and scan teardown or
  rebinding releases that slot before clearing local state.
- Added unit coverage for:
  - ordered slot claiming
  - double-release safety
  - scan-side release on `clear_parallel_scan_state`
- Updated Task 18 notes so the staged status matches the live worker-slot seam.

Still intentionally deferred:
- `amcanparallel` remains `false`
- no worker-slot ownership of beam/frontier scratch yet
- no coordinator top-K heap or push/pop contract yet
- no planner-visible parallel execution yet

Validation:
- Passed:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `bash scripts/run_pg18_preload_pgstat_test.sh`

Review focus:
- Whether the claim/release atomic protocol is the right stable boundary for
  the later coordinator/worker state handoff
- Whether scan-side release on clear/teardown is the right place to drop worker
  slot ownership before real parallel traversal is enabled
- Whether the rescan-epoch guard is sufficient for stale worker-slot cleanup
  without a more complex generation protocol
