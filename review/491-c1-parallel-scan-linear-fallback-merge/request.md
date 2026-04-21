# Review Request: Parallel Scan Linear Fallback Merge Staging

Current head: `1711a05`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/ec_hnsw/scan.rs`

Problem:
- Bound parallel scans could already drain staged coordinator output before
  local scan production.
- But when linear fallback materialized a brand-new local row in the same call,
  that first emit still bypassed the shared coordinator merge path.

What changed:
- Added `emit_materialized_parallel_scan_output(...)`.
- When a parallel-scan descriptor is bound, the helper now:
  - materializes the newly selected local row into the active scan result state
  - republishes that row into the shared result-slot snapshot
  - drains it back through the staged coordinator merge helper
  - republishes the next local duplicate after the shared consume advances the
    owned cursor
- `produce_next_linear_fallback_heap_tid(...)` now stages newly selected local
  rows through that helper before falling back to direct local emit behavior.
- Added focused regression coverage for the new helper and updated Task 18 notes
  to record that newly materialized linear-fallback rows no longer bypass the
  shared merge seam on first emit.

Important staging note:
- This still only closes a shared-infra gap.
- `amcanparallel` remains `false`.
- Planner-visible LIMIT budgeting is still not wired, so the admitted window
  still uses descriptor-capacity admission.

Still intentionally deferred:
- no final coordinator/worker execution loop yet
- no planner-visible parallel execution yet
- no LWLock-based serializer yet
- no planner LIMIT / cost plumbing for the admitted window
- graph-side newly materialized staging is still separate follow-up work

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether routing freshly materialized linear-fallback rows through the shared
  merge helper is the right next constraint before enabling real parallel scan
- Whether the helper leaves the serial fallback path unchanged when no parallel
  descriptor is bound
- Whether the remaining graph-side follow-up is clearly separated from this
  fallback-only seam
