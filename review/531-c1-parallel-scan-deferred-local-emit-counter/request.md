# Review Request: Parallel Scan Deferred Local-Emit Counter

Current head: `11a6ce5`

Scope:
- `src/am/common/explain.rs`
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The branch still has one intentional staged fallback: if every surviving
  deferred row is genuinely still blocked after shared retry, the scan locally
  emits the best deferred row to keep making progress.
- That path was real but invisible. Operators could see blocked-owner counters,
  but not how often the scan actually had to fall back to a local deferred emit
  because the final ownership-transfer contract is still missing.

What changed:
- Added a new EXPLAIN counter:
  - `stats_parallel_deferred_local_emits`
  - rendered as `Parallel Deferred Local Emits`
- The counter increments only at the last-resort deferred local-emission path
  in `take_next_deferred_parallel_blocked_output(...)`.
- Added focused scan-side coverage proving that draining the only remaining
  blocked deferred row increments the new counter.
- Updated Task 18 notes so this fallback is now explicitly visible in the
  staged PG18 diagnostics contract.

Why this matters:
- The remaining ownership gap is now measurable instead of silent.
- That gives the next ownership-transfer slices a concrete signal to drive
  down, rather than burying the fallback inside ordinary tuple-return counts.

Still intentionally deferred:
- final cross-worker ownership transfer instead of deferred local retention
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement after the remaining ownership seam
  lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether this counter is the right visibility seam for the current staged
  fallback
- Whether the counter name and EXPLAIN label make it clear that the event is
  specifically the deferred blocked-row local-emission fallback, not ordinary
  tuple production
