# Review Request: Parallel Scan Graph Handoff Emitted Bookkeeping

Current head: `223e030`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The earlier foreign-handoff bookkeeping correction tightened the linear
  shared paths, but the graph shared-emission branches still unconditionally
  marked the active local prefetched element as emitted.
- That was wrong for foreign handoff: `consume_parallel_scan_admitted_result(...)`
  already records the actually emitted foreign/admitted element, while the
  local prefetched row remains staged in the worker slot for later progress.

What changed:
- Removed the redundant active-local emitted mark from the two graph shared
  branches:
  - prefetched shared merge emit
  - republished local-only shared wakeup emit
- Strengthened the existing graph foreign-handoff regression to assert both:
  - the emitted set includes the foreign emitted element
  - the still-staged local prefetched element is not marked emitted
- Updated Task 18 notes so the bookkeeping rule is stated once across graph
  and linear paths:
  - direct local-only emit uses the active-result helper
  - shared handoff paths rely on admitted-result bookkeeping

Why this matters:
- Shared graph handoff now follows the same source-accurate emitted bookkeeping
  rule as the corrected linear path.
- The emitted-element set stays aligned with the row actually returned to the
  executor, not the local prefetched owner row that remains staged for later
  retries.
- This prevents ownership bookkeeping drift while the final cross-worker
  ownership-transfer contract is still deferred.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo fmt`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether any graph shared-handoff path still risks tagging the local
  prefetched owner row instead of the actually emitted foreign/admitted row
- Whether the strengthened graph regression now covers the same foreign-vs-
  local emitted bookkeeping distinction the linear follow-up already pins
