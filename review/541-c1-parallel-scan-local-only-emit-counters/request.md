# Review Request: Parallel Scan Local-only Emit Counters

Current head: `ad933e7`

Scope:
- `src/am/common/explain.rs`
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The branch already exposed deferred local fallback in `Ecaz Stats` as
  `Parallel Deferred Local Emits`, split by foreign-selected versus
  foreign-admitted blockers.
- But the other remaining fallback surface, a hidden
  `parallel_local_only_output_active` row that still has to emit locally after
  shared retry, was still invisible in EXPLAIN.
- That made the remaining ownership gap harder to diagnose because local-only
  wakeup emits only showed up indirectly through ordinary heap-tid counts.

What changed:
- Extended `TqExplainCounters` with:
  - `stats_parallel_local_only_emits`
  - `stats_parallel_local_only_emits_foreign_selected_pending`
  - `stats_parallel_local_only_emits_foreign_admitted_head`
- Added the matching `Ecaz Stats` properties:
  - `Parallel Local-only Emits`
  - `Parallel Local-only Emits: Foreign Selected`
  - `Parallel Local-only Emits: Foreign Head`
- Added `record_parallel_local_only_emit_counters(...)` in the scan path so
  the real local-only emit branches record those counters before locally
  emitting the hidden row.
- Wired that counter helper into both local-only wakeup emit paths:
  - graph traversal wakeup
  - linear fallback wakeup
- Updated Task 18 notes so the local-only wakeup fallback is now recorded as a
  first-class observable seam, not just an implicit behavior.

Why this matters:
- It makes the remaining ownership gap measurable instead of inferred.
- We can now distinguish:
  - deferred local fallback
  - local-only wakeup local fallback
- The blocker-kind split keeps the remaining foreign-selected versus
  foreign-admitted pressure visible while the final ownership-transfer contract
  is still deferred.

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
- Whether local-only wakeup fallback belongs in the same EXPLAIN visibility
  family as deferred local fallback
- Whether the blocker-kind split is the right one for this surface, or whether
  additional local-only detail would be useful before planner-visible
  enablement
