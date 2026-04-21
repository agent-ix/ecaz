# Review Request: Parallel Scan Serialized Heap Mutation

Current head: `65a21b6`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`

Problem:
- The staged parallel heap now supports incremental publish, clear, and staged
  take without full rebuilds, but the mutation helpers still assumed a single
  writer.
- That was safe only because `amcanparallel` is still `false`. The review on
  the earlier selection slice explicitly called out heap mutation as needing
  serialization before planner-visible parallel scans can land.
- The next Task 18 slices need the staged heap to keep the same behavior under
  a real shared serializer, not just under the current single-writer staging
  assumption.

What changed:
- Added a coordinator-owned shared lock word to the AM-private staged-heap
  state and bumped the parallel descriptor version to match the DSM layout
  change.
- Added a small lock guard in `src/am/common/parallel.rs` so staged-heap
  mutation paths acquire and release that shared serializer consistently.
- Serialized the staged-heap mutation/read paths that touch heap entries or
  rebuild the cached selection:
  - publish staged result slot
  - clear staged result slot
  - staged coordinator take
  - heap snapshot read
  - direct best-slot selection helper
  - stale-fast-path refresh in direct selected-result read
- Kept the normal fast-path selected-result read lock-free until it detects a
  stale selection, then retries under the shared heap serializer.
- Updated Task 18 notes to state that staged-heap mutation is now serialized
  behind the shared lock word, while `amcanparallel` and the true shared top-K
  admission path remain deferred.

Still intentionally deferred:
- `amcanparallel` remains `false`
- no lock-guarded shared top-K admission path yet
- no worker-owned traversal scratch in DSM yet
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
- Whether the shared lock-word boundary is the right serializer for staged heap
  mutation before the eventual shared top-K admission API lands
- Whether the lock-free first read plus locked stale-refresh retry is the right
  contract for selected-result reads
- Whether the serialized public entry points now cover every heap-entry mutation
  or heap-root read that would have raced under real parallel execution
