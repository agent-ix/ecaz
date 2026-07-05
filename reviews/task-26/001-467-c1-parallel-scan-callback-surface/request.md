# Review Request: Parallel Scan Callback Surface

Current head: `a8f8e3a`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/mod.rs`
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/mod.rs`
- `src/am/ec_hnsw/routine.rs`
- `src/am/ec_hnsw/scan.rs`

Problem:
- Task 18 did not have any shared AM-private parallel-scan descriptor or
  callback surface in the split `common` / `ec_hnsw` layout.
- `ec_hnsw` still exposed no `amestimateparallelscan`,
  `aminitparallelscan`, or `amparallelrescan` wiring, so later
  coordinator/worker work had no stable ABI boundary to build on.
- PG17 and PG18 differ slightly at the callback-signature and
  `ParallelIndexScanDesc` offset-field seams, so the repo needed one
  shared implementation with thin version gates rather than two
  diverging parallel-scan paths.

What changed:
- Added `src/am/common/parallel.rs` as the shared AM-private descriptor and
  callback helper module.
- The shared header tracks a magic/versioned descriptor size and a shared
  rescan epoch for later coordinator/worker reuse.
- Wired `ec_hnsw` to expose:
  - `amestimateparallelscan`
  - `aminitparallelscan`
  - `amparallelrescan`
- Kept `amcanparallel = false` intentionally. This slice only lands the
  callback surface and descriptor contract; planner-visible parallel scan
  remains gated until the real coordinator and worker-local traversal
  semantics exist.
- Added scan-side attachment plumbing so `TqScanOpaque` captures the
  shared descriptor pointer and current rescan epoch during `amrescan`.
- Updated Task 18 text so it points at `src/am/common/parallel.rs` and
  documents the staged callback-first landing.

Validation:
- Passed:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether `src/am/common/parallel.rs` is the right shared boundary for
  later multi-AM coordinator/worker reuse
- Whether the PG17/PG18 compatibility gates are isolated cleanly to the
  callback signature and offset-field seams
- Whether scan-side attachment during `amrescan` is the right minimal
  staging point before planner-visible parallel scans exist
- Whether keeping `amcanparallel = false` in this checkpoint is the right
  scope discipline
