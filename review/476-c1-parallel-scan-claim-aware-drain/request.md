# Review Request: Parallel Scan Claim-Aware Drain

Current head: `7e1b61c`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`

Problem:
- The prior Task 18 slices could stage coordinator-selected results, refresh
  the fast path, and drain one staged result at a time.
- But staged-result liveness still only meant "published + valid + current
  epoch". If a worker claim dropped without the coordinator snapshot getting
  refreshed first, the direct read/take path could still point at that stale
  slot until some later mutation refreshed selection.
- We needed the coordinator drain seam itself to treat "owning worker claim is
  still live" as part of staged-result liveness before building the real shared
  top-K merge path on top of it.

What changed:
- Added worker-claim-aware staged-result liveness helpers in
  `src/am/common/parallel.rs`.
- `select_best_parallel_scan_coordinator_result_slot_with_attachment(...)`
  now ignores staged result slots whose owning worker slot is no longer
  claimed for the active rescan epoch.
- `read_parallel_scan_selected_result_slot_snapshot(...)` now refreshes the
  coordinator fast path when the named staged result is no longer claim-live,
  and returns the next live staged result when one exists.
- `take_parallel_scan_selected_result_slot_snapshot(...)` now does the same
  refresh-before-consume behavior, rather than trusting a stale selected slot.
- Added focused coverage for:
  - reading past an unclaimed selected slot to the next live staged result
  - clearing the coordinator fast path when the only selected slot lost its
    worker claim
- Updated Task 18 notes to record that the staged coordinator drain is now
  claim-aware.

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
- Note:
  - an earlier overlapping `cargo pgrx test pg18` run tripped the `pgrx`
    global test mutex; I reran the pgrx lanes serially and the final results
    above are the clean ones.

Review focus:
- Whether worker-claim liveness belongs directly in the staged coordinator
  selection/drain contract
- Whether "refresh selected slot before exposing or consuming it" is the right
  seam for later coordinator merge work
- Whether this keeps the staged coordinator surface narrow enough before the
  real shared top-K heap lands
