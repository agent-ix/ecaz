# Review Packet

Current head: `159bf20`

Purpose:
- Leave focused review requests for another agent to process independently.
- Keep each request narrow and tied to the current validated state.

Validation status at this checkpoint:
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
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
- Repeated `amrescan` coverage now verifies that a second rescan overwrites the recorded query dimensions on the same descriptor.
- `amgettuple` now returns `false` for valid rescans on empty indexes while keeping non-empty scan execution disabled.
- `amrescan` now persists the full query payload in scan-owned PostgreSQL memory and frees it during `amendscan`.
- `amgettuple` now supports a forward-only linear data-page scan for non-empty indexes.
- The current non-empty scan path now returns every heap TID from each live element tuple before advancing and keeps duplicate-drain progress in scan-local opaque state.
- Query-payload ownership, linear scan cursor state, and duplicate heap-TID progress all now live in scan-owned opaque memory.
- `tqvector_query_inner_product` now reconstructs the full persisted quantized payload from `(gamma, code_bytes)` before calling the prepared-query scorer.
- Query-inner-product coverage now verifies that the SQL-facing scorer uses the persisted `gamma` term instead of passing only code bytes into the quantizer API.
- ADR for the duplicate-drain decision: `spec/adr/ADR-009-linear-scan-duplicate-heaptids.md`

Review triage at `46d00bb`:
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
- `13-amgettuple-empty-index-noop.md`
- `14-rescan-query-payload-state.md`
- `15-amgettuple-linear-forward-scan.md`
- `16-query-inner-product-gamma-payload.md`

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
