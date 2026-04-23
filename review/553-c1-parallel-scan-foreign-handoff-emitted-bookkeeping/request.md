# Review Request: Parallel Scan Foreign Handoff Emitted Bookkeeping

Current head: `d7048d0`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The previous linear bookkeeping slice tried to extend emitted-element marking
  into the linear shared wakeup branches.
- That was too broad: shared handoff already records the actually emitted
  foreign element inside `consume_parallel_scan_admitted_result(...)`.
- Adding an extra "active local result" mark there would incorrectly tag the
  still-staged local owner row as emitted when the shared output belongs to a
  foreign worker.

What changed:
- Kept the shared handoff branches on admitted-result bookkeeping instead of
  adding a second active-local emitted mark.
- Strengthened the existing foreign selected-pending handoff regression:
  - explicitly initializes emitted-element state
  - asserts the emitted set includes the foreign emitted element
  - asserts the emitted set does not include the still-staged local owner row
- Tightened the Task 18 note so it reflects the real rule:
  - linear direct local-only emit uses the shared active-result helper
  - shared handoff paths keep using admitted-result bookkeeping

Why this matters:
- Shared foreign handoff now has an explicit regression against the exact
  bookkeeping bug we want to avoid.
- The emitted-element set stays aligned with the row actually returned to the
  executor, not the local owner row that is still staged for later progress.
- This keeps the local/shared ownership seam narrower while the final
  cross-worker ownership-transfer contract remains deferred.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test resolve_local_only_parallel_scan_duplicate_handoffs_live_foreign_duplicate -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Notes:
- An overlapping first pass of `cargo test` failed only because it was run
  concurrently with the PG18 pgrx lane and the embedded PG18 harness collided
  with that session. The final serial rerun of `cargo test` is the green result
  above.

Review focus:
- Whether any shared handoff path still risks tagging the local owner row
  instead of the actual emitted foreign/admitted row
- Whether the strengthened regression covers the real foreign-vs-local emitted
  bookkeeping distinction
