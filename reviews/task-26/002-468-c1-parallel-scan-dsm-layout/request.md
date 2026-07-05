# Review Request: Parallel Scan DSM Layout

Current head: `fe87999`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`

Problem:
- The prior Task 18 slice only established the callback surface and a minimal
  AM-private header.
- There was still no real shared descriptor contract for a coordinator block
  plus worker slots, so later worker claiming and coordinator state would have
  needed an ABI-breaking rewrite.
- PostgreSQL's `amestimateparallelscan` callback does not receive the chosen
  executor worker count, so the repo needed an explicit staged policy for how
  to size shared worker-slot headers without pretending parallel traversal was
  already live.

What changed:
- Expanded `src/am/common/parallel.rs` from a header-only stub into a shared
  descriptor layout with:
  - `EcParallelScanState`
  - `EcParallelCoordinatorState`
  - `EcParallelWorkerSlot`
- `amestimateparallelscan` now returns the full shared descriptor size rather
  than only the common header.
- The staged descriptor reserves worker-slot headers for up to
  `max_parallel_workers_per_gather + 1` participants.
- Initialization and rescan now reset coordinator state and stamp worker-slot
  headers with the active rescan epoch.
- Added validated worker-slot addressing helpers and unit coverage for:
  - descriptor sizing
  - slot header initialization
  - rescan layout reset
  - out-of-bounds slot rejection
- `TqScanOpaque` now carries the shared worker-slot capacity from scan
  attachment, so the next slice can layer worker claiming on top without
  reopening the attachment seam.
- Updated Task 18 notes to document the bounded-capacity sizing choice at the
  callback seam.

Still intentionally deferred:
- `amcanparallel` remains `false`
- no worker-slot claiming yet
- no shared top-K heap or coordinator push/pop path yet
- no planner-visible parallel scan behavior yet

Validation:
- Passed:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the shared descriptor layout is the right stable ABI boundary for
  later coordinator and worker-local state
- Whether the bounded `max_parallel_workers_per_gather + 1` slot-capacity
  policy is the right staged answer to the `amestimateparallelscan` signature
  constraint
- Whether scan-side attachment should carry worker-slot capacity already, or
  stay narrower until slot claiming lands
- Whether the rescan reset semantics are appropriate for the next worker-claim
  slice
