# Review Request: Parallel Scan Coordinator Selection

Current head: `929cb0d`

Scope:
- `benches/iai/bitpack.rs`
- `benches/iai/hadamard.rs`
- `benches/iai/quant_score.rs`
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/explain.rs`
- `src/am/common/parallel.rs`
- `tests/proptest_page.rs`

Problem:
- The prior Task 18 slice staged coordinator-owned result slots in shared DSM,
  but there was still no shared helper that could choose the coordinator's
  current best staged result across workers.
- Without that selection seam, the next shared top-K merge work would have to
  fold result ordering and descriptor traversal together, which would make the
  coordinator path harder to validate incrementally.
- This slice also carries the formatter output encountered while touching the
  Rust files in scope.

What changed:
- Added `EcParallelCoordinatorResultSelection` in
  `src/am/common/parallel.rs`.
- Added shared selection logic that:
  - scans the staged coordinator result slots
  - ignores unpublished, stale-epoch, score-invalid, and invalid-TID entries
  - chooses the lowest live score
  - breaks ties by lower slot index for deterministic coordinator behavior
- Added coverage for:
  - no-live-result selection returning `None`
  - lowest-score selection
  - tie-breaking by slot index
- Updated Task 18 notes to record that staged coordinator selection is now
  live while shared top-K mutation remains deferred.
- Committed the formatter output encountered in the touched Rust surfaces.

Still intentionally deferred:
- `amcanparallel` remains `false`
- no shared top-K heap mutation path yet
- no worker-local traversal scratch in DSM yet
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
- Whether the staged selection contract is the right seam ahead of shared top-K
  heap mutation
- Whether lowest-score plus slot-index tie-breaking is the right deterministic
  coordinator policy for this intermediate slice
- Whether the helper rejects the right invalid/stale slot states without
  overconstraining later coordinator merge work
