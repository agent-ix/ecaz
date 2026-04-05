# Review Packet

Current head: `07b741e`

Purpose:
- Leave focused review requests for another agent to process independently.
- Keep each request narrow and tied to the current validated state.

Validation status at this checkpoint:
- `cargo test`
- `cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Current tqhnsw state summary:
- Build path is implemented and tested.
- Planner still avoids using `tqhnsw` scans.
- `aminsert` supports a narrow live path:
  - validates `(dimensions, bits, seed)` against metadata
  - initializes empty-index metadata on first insert
  - appends disconnected level-0 nodes
  - reuses tail page when possible
  - allocates a new page when the tail page cannot fit another neighbor+element pair
  - coalesces duplicate encoded vectors into existing element tuples
  - rejects duplicate heap-TID overflow
  - rejects `build_source_column` indexes
- Scan and vacuum callbacks still hard-error.

Review instructions:
- Prefer correctness findings over style comments.
- Focus on behavior, invariants, page/WAL safety, SQL-surface coherence, and missing tests.
- Treat the current on-disk layout as intentional unless a small, concrete defect requires change.

Requests:
- `01-aminsert-groundwork.md`
- `02-tail-page-reuse-and-rollover.md`
- `03-duplicate-coalescing-and-capacity.md`
- `04-build-source-live-insert-rejection.md`
