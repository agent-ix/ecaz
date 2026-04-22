# Review Request: Parallel Scan Graph Admission Window

Current head: `2fbd5dd`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The earlier blocker-taxonomy slice only changed linear fallback behavior.
- Graph traversal still treated `AdmissionWindow` the same way as a foreign
  owner blocker, so a prefetched row that had already lost shared admission
  could still fall through the local-direct emit fallback.
- That left graph and linear scan phases inconsistent at the exact point where
  the blocker taxonomy is meant to guide staged ownership behavior.

What changed:
- Reworked graph traversal tuple production to loop over
  `emit_prefetched_parallel_scan_output(...)` instead of handling only a single
  blocked-or-emitted decision.
- When the blocker is `AdmissionWindow`, graph traversal now:
  - discards the active staged local/shared output
  - refreshes prefetched output
  - continues local search for the next candidate
- Foreign-owner blockers still keep the explicit staging fallback until the
  real multi-worker handoff contract lands.
- Updated Task 18 notes so the blocker-taxonomy behavior now explicitly covers
  both linear and graph scan paths.

Why this matters:
- It keeps staged graph traversal aligned with the same shared-admission rule
  already enforced for linear fallback.
- That removes another bypass around the shared merge seam before real
  multi-worker ownership is enabled.

Still intentionally deferred:
- the real multi-worker output handoff / ownership contract
- planner-visible parallel execution and `amcanparallel = true`
- `n = 2/4/8` correctness coverage once the real multi-worker path lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether graph traversal should drop admission-window losers now, or whether
  there is any staged ownership reason to keep the old local-direct emit
  fallback there
- Whether the graph-side discard-and-refresh loop leaves any prefetched-output
  state behind when a loser is dropped
