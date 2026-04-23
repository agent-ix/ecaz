# Review Request: Parallel Scan Local-only Emit Counter Fix

Current head: `2478249`

Scope:
- `src/am/ec_hnsw/scan.rs`

Problem:
- The previous local-only emit counter slice landed the EXPLAIN surface and
  wired the counters into the later `resolve_local_only_parallel_scan_duplicate`
  wakeup branch.
- But the earliest wakeup emit branches could still locally drain the hidden
  row before that later helper ran:
  - graph prefetched-output emit
  - linear fallback pending-output emit
- That meant the new `Parallel Local-only Emits` counters undercounted the real
  local-only fallback path.

What changed:
- Tightened `record_parallel_local_only_emit_counters(...)` so it only records
  when a retained local-only blocker is actually present.
- Called that helper in the first graph and linear local wakeup emit branches,
  not just the later duplicate-resolution branch.

Why this matters:
- The new EXPLAIN counters now cover the real local-only wakeup emit path
  instead of only one sub-branch of it.
- This keeps the observability surface honest before any planner-visible
  enablement work starts depending on these counters.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether this is the right boundary for the local-only emit counters, or if
  there is any other hidden wakeup path that should also feed the same surface
