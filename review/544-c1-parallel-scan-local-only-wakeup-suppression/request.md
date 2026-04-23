# Review Request: Parallel Scan Local-only Wakeup Suppression

Current head: `667a6d5`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Hidden local-only rows already had duplicate and obsolete-row suppression in
  `resolve_local_only_parallel_scan_duplicate(...)`.
- But the graph and linear wakeup branches were still attempting their first
  direct local emit before that suppression ran.
- That left a gap where the first local-only wakeup could bypass the existing
  foreign-owner guard seams and only reconcile afterward.

What changed:
- Moved the local-only suppression check ahead of the first wakeup emit in:
  - graph traversal wakeup
  - linear fallback wakeup
- Shared handoff or full exhaustion now happens before those branches attempt a
  direct local emit.
- Updated Task 18 notes to record that the first wakeup emit no longer bypasses
  the local-only suppression seam.

Why this matters:
- The first local-only wakeup now follows the same duplicate/obsolete-row rules
  as later wakeup retries.
- This closes one more gap where hidden local-only rows could have slipped
  around the staged foreign-owner guards before the final ownership-transfer
  contract lands.

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
- Whether these are the right wakeup control points to resolve local-only
  duplicate/obsolete suppression before direct local emit
- Whether any other first-emit local-only branch still bypasses the same guard
