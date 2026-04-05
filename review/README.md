# Review Packet

Current head: `9444d4b`

Purpose:
- Leave focused review requests for another agent to process independently.
- Keep each request narrow and tied to the current validated state.

Validation status at this checkpoint:
- `cargo test`
- `cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Current tqhnsw state summary:
- Build path is implemented and tested.
- Planner still avoids using `tqhnsw` scans.
- `aminsert` supports a narrow live path:
  - validates `(dimensions, bits, seed)` against metadata
  - serializes empty-index metadata initialization under an exclusive metadata-page lock
  - initializes empty-index metadata on first insert
  - appends disconnected level-0 nodes
  - reuses tail page when possible
  - allocates a new page when the tail page cannot fit another neighbor+element pair
  - coalesces duplicate encoded vectors into existing element tuples
  - rejects duplicate heap-TID overflow
  - rejects `build_source_column` indexes
- Vacuum callbacks are benign no-ops that return current page/tuple stats.
- `ambeginscan` allocates a real scan descriptor plus opaque state.
- `amrescan` validates a single `real[]` ORDER BY query and records minimal query-shape state.
- `amgettuple` now requires `amrescan`-initialized scan state before execution.
- `amgettuple` still rejects actual tuple production, so planner-visible scan execution remains disabled in practice.
- `amrescan` defensive error paths now have explicit regression coverage for NULL queries, empty queries, index quals, and multiple ORDER BY keys.
- Vacuum no-op coverage now includes empty-index and repeated-vacuum regression tests.
- Scan lifecycle coverage now includes repeated-`amendscan` idempotency.
- Tail-page coverage now includes rollover-followed-by-reuse on the new tail page.

Review triage at `9444d4b`:
- Addressed `01-aminsert-groundwork.md` comment 1 by locking the metadata page across the current narrow `aminsert` path.
- Addressed `01-aminsert-groundwork.md` comment 4 with a sequential empty-index second-insert regression test.
- Marked `01-aminsert-groundwork.md` comments 2, 3, and 5 as not needed for this stage because they are optimization or future-invariant notes rather than current defects.
- Addressed `02-tail-page-reuse-and-rollover.md` comment 5 with rollover-followed-by-reuse regression coverage.
- Marked `02-tail-page-reuse-and-rollover.md` comments 1-4 and 6 as not needed for this stage because they validate accepted current behavior.
- Marked `03-duplicate-coalescing-and-capacity.md` comments 1-6 as not needed for this stage because the review found no current correctness gap or missing test that justifies more change.
- Marked `04-build-source-live-insert-rejection.md` comments 1-6 as not needed for this stage because the review found the current restriction correct and sufficiently covered.
- Addressed `07-rescan-query-validation.md` comment 7 with explicit regression tests for the reviewed `amrescan` defensive cases.
- Marked `07-rescan-query-validation.md` comments 1-6 and 8 as not needed for this stage because they are validation of current behavior or future-slice notes rather than actionable defects.
- Addressed `05-vacuum-noop-callbacks.md` comments 6 and 7 with empty-index and repeated-vacuum regression coverage.
- Marked `05-vacuum-noop-callbacks.md` comments 1-5 and 8 as not needed for this stage because they document accepted current behavior rather than requiring code changes.
- Addressed `06-scan-descriptor-scaffolding.md` comment 6 with repeated-`amendscan` idempotency coverage.
- Marked `06-scan-descriptor-scaffolding.md` comments 1-5 and 7 as not needed for this stage because they validate accepted lifecycle behavior.
- Marked `08-amgettuple-state-gating.md` comments 1-7 as not needed for this stage; the repeated-rescan note remains blocked on the current fatal scan-execution boundary and does not justify more helper surface yet.

Review instructions:
- Prefer correctness findings over style comments.
- Focus on behavior, invariants, page/WAL safety, SQL-surface coherence, and missing tests.
- Treat the current on-disk layout as intentional unless a small, concrete defect requires change.

Open requests:
- None right now. The next work item should be chosen as a new implementation slice rather than a pending review follow-up.

Closed requests:
- `01-aminsert-groundwork.md`
- `02-tail-page-reuse-and-rollover.md`
- `03-duplicate-coalescing-and-capacity.md`
- `04-build-source-live-insert-rejection.md`
- `05-vacuum-noop-callbacks.md`
- `06-scan-descriptor-scaffolding.md`
- `07-rescan-query-validation.md`
- `08-amgettuple-state-gating.md`
- `09-rescan-defensive-cases.md`
- `10-vacuum-noop-coverage.md`
- `11-scan-lifecycle-idempotency.md`
- `12-tail-page-rollover-followup.md`
