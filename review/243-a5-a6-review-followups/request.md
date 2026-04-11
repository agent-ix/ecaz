# Review Request: A5/A6 Reviewer Follow-Ups

## Context

Branch:
- `main`

Reviewer inputs addressed here:
- `review/235-a5-concurrency-retry-hardening/feedback/2026-04-11-01-reviewer.md`
- `review/242-a6-vacuum-concurrency-validation/feedback/2026-04-11-01-reviewer.md`

Primary scope:
- `src/am/insert.rs`
- `spec/adr/ADR-026-live-insert-backlink-lock-ordering.md`
- `spec/functional/FR-016-hnsw-insert.md`
- `src/am/graph.rs`
- `src/am/mod.rs`
- `src/am/scan_debug.rs`
- `src/am/vacuum.rs`
- `src/lib.rs`
- `scripts/vacuum_concurrency_scratch.sh`
- `spec/adr/ADR-027-vacuum-graph-repair-lock-ordering.md`
- `plan/tasks/07-vacuum.md`
- `spec/functional/FR-010-hnsw-vacuum.md`
- `spec/functional/FR-022-vacuum-implementation.md`
- `spec/tests.md`

This is a narrow review-followup checkpoint after the outside review on A5 and
the closing A6 packet. It does not open a new runtime lane. It tightens the
durable docs around A5 retry semantics and corrects the A6 concurrency harness
proof so it asserts a real invariant instead of a metric that turned out not to
mean what the review initially assumed.

## What Landed

### 1. A5 retry semantics are now called out directly

The A5 follow-ups from packet 235 are now recorded in the durable surfaces:

- `MAX_BACKLINK_REPLAN_PASSES = 3` now has an explicit code comment in
  `src/am/insert.rs`
- ADR-026 now says the retry payload may carry only
  `(target_element_tid, layer)` out of the write phase
- `FR-016-AC-3` now explicitly says the landed A5 proof is about lock ordering
  and stale-plan retry safety, not multi-writer throughput benchmarking

These are comment/doc clarifications only. Insert behavior is unchanged.

### 2. Vacuum repair now carries source level directly and reuses graph slot math

The A6 review also called out two cleanup nits in `src/am/vacuum.rs`:

- pass-2 repair was duplicating `layer_slot_bounds(...)`
- `apply_repair_plan(...)` was reverse-inferring source level from tuple width

This checkpoint fixes both:

- `src/am/graph.rs` exports `layer_slot_bounds(...)` as `pub(crate)`
- `LayerRepairPlan` now carries `source_level`
- `src/am/vacuum.rs` reuses the shared layer-slot helper and deletes the local
  reverse-inference helpers

That keeps pass-2 repair aligned with the same tuple-layout authority as scan
and insert.

### 3. The A6 concurrency harness now asserts a real post-quiesce invariant

The original review asked for:

- final `VACUUM (ANALYZE)` after workers stop
- a hard failure if final live rows and final scan rows diverge
- iteration counts printed for each worker

I implemented the final `VACUUM (ANALYZE)` and worker-iteration reporting, but
while wiring the hard equality I found that the assumption behind it was wrong
for the current runtime stage:

- `tests.tqhnsw_debug_scan_result_count(...)` exercises the live
  `ambeginscan/amrescan/amgettuple` path, but that path is intentionally bounded
  by current graph traversal behavior and `ef_search`
- on a clean 2000-row built fixture, the helper returns `40`, not `2000`
- even with `ef_search = 1000`, a 500-row built fixture returned `412`, not
  `500`

So `final_scan_result_count` is useful as a live-scan liveness probe, but it is
not a full-row-count metric and cannot support the equality the review proposed.

To address the review’s *intent* without pretending otherwise, this checkpoint
lands:

- `tests.tqhnsw_debug_reachable_live_element_count(index_oid oid)`, a
  `pg_test`-only SQL helper backed by a full layer-0 BFS from the current entry
  point
- `test_tqhnsw_debug_reachable_live_count_matches_admin_snapshot`, proving the
  SQL wrapper matches the Rust helper and the helper matches the admin snapshot
  on a connected small fixture
- a corrected post-quiesce harness proof:
  1. run concurrent INSERT + tqhnsw scan + VACUUM workers
  2. issue final `VACUUM (ANALYZE)` after all workers stop
  3. compute live-index reachable live elements
  4. build a fresh reference tqhnsw index on the same final table data
  5. fail if the live index falls below 90% of the rebuilt reference’s
     reachable live-element count

This keeps the real tqhnsw scan path in the worker mix, but moves the final
binary assertion onto a metric that actually corresponds to graph repair health.

### 4. ADR-027 now names same-page multi-layer repair explicitly

The vacuum lock-order ADR now also says one page-local `EXCLUSIVE` window may
rewrite multiple logical layer slices of the same persisted neighbor tuple. The
ordering rule is per physical page, not per logical layer.

## Validation

Standard checkpoint validation:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Harness validation:

- `scripts/vacuum_concurrency_scratch.sh --duration 5`
- `scripts/vacuum_concurrency_scratch.sh --duration 60`

Observed harness results:

- 5s:
  - `insert_worker_iterations=27`
  - `vacuum_worker_iterations=27`
  - `scan_a_worker_iterations=38`
  - `scan_b_worker_iterations=38`
  - `final_live_elements=2054`
  - `final_reachable_live_elements=1284`
  - `reference_reachable_live_elements=1262`
  - `reachable_vs_reference_percent=101`
  - `final_scan_result_count=40`
- 60s:
  - `insert_worker_iterations=447`
  - `vacuum_worker_iterations=259`
  - `scan_a_worker_iterations=699`
  - `scan_b_worker_iterations=696`
  - `final_live_elements=3270`
  - `final_reachable_live_elements=2207`
  - `reference_reachable_live_elements=1819`
  - `reachable_vs_reference_percent=121`
  - `final_scan_result_count=40`

## Review Focus

- Is the A6 proof correction sound: keeping the live scan helper in the worker
  mix, but comparing the final mutated index against a rebuilt reference index
  instead of forcing a bogus scan-count equality?
- Is `90%` of rebuilt-reference reachable live elements the right narrow
  threshold for `TC-215`, or should this threshold move up/down before we treat
  the harness as the primary concurrency proof?
- Are the A5 follow-up clarifications in `ADR-026`, `FR-016`, and the
  `MAX_BACKLINK_REPLAN_PASSES` comment sufficient, or is one more durable note
  still missing?
