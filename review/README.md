# Review Packet

Current head: `d57a2b5`

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
- Linear scan coverage now verifies that repeated `amgettuple` calls stay `false` after exhaustion.
- Linear scan coverage now verifies that `amgettuple` still rejects backward scan direction after a valid `amrescan`.
- Linear scan coverage now verifies that `amrescan` after full exhaustion restarts tuple production from the beginning.
- Build-time detoast handling now uses pointer comparison against the original datum to decide whether `pg_detoast_datum_packed` returned an allocated copy.
- `encode_to_tqvector` now rejects embeddings whose dimension exceeds the persisted `u16` limit instead of truncating on pack.
- The in-memory `DataPage` tuple insertion path now uses a checked `u16` conversion for returned offset numbers.
- The `hnsw_rs` Cargo dependency is now pinned to the currently locked `0.3.4` release instead of `*`.
- `amrescan` now caches scan dimensions, bits, and derived code length in scan-owned opaque state so `amgettuple` no longer rereads the metadata page on every call.
- `relation_options` now reads parsed reloptions directly from the relation descriptor cache instead of issuing an SPI catalog query on every `aminsert`.
- Code-to-code inner-product scoring now has a zero-allocation raw-code fast path, and `score_code_inner_product` no longer builds temporary fake payload buffers on each call.
- Empty-index scan coverage now explicitly verifies repeated `amgettuple` calls stay `false` and that `amrescan` on an empty index still produces no tuples.
- `amrescan` now rejects oversized `real[]` queries with an explicit dimension error instead of relying on an internal `u16` conversion panic.
- The linear scan cursor now uses a direct `offset + 1` advance with a `debug_assert!` on the page-local `u16` invariant instead of carrying an unreachable saturation branch.
- Query-scoring coverage now explicitly exercises candidate/query dimension mismatch and the negative-query wrapper contract.
- Linear scan coverage now explicitly verifies that a duplicate-heavy scan continues correctly across multiple data pages and mixed element/neighbor tuple pages.
- `amrescan` now caches the current relation block count in scan-owned state so the bootstrap linear scan does not re-fetch it on every tuple-producing call.
- `amrescan` now also caches a prepared quantizer query object in scan-owned state for non-empty indexes as groundwork for ordered traversal.
- The bootstrap linear scan now also tracks an explicit current-result tuple pointer in scan-owned state, clearing it on rescan and exhaustion so later ordered execution can hang score/result bookkeeping off a stable slot.
- Current-result lifecycle coverage now verifies that duplicate draining stays attached to the same element tuple and that the current-result slot clears on exhaustion and on `amrescan`.
- The bootstrap linear scan now also computes and stores an operator-facing `<#>` score for the current result element by combining the cached prepared query with the representative heap row's persisted `gamma`.
- Current-result score coverage now verifies that score validity flips on with first tuple production, remains attached while draining duplicate heap TIDs from one element, and clears back to zero on exhaustion.
- Duplicate matching now uses persisted `gamma` plus code bytes instead of code bytes alone, so same-code tqvectors with distinct gamma terms no longer collapse into one element during build or live insert.
- Live duplicate coalescing now recovers representative `gamma` from the heap row when a same-code element candidate is found, preserving the current page layout while keeping duplicate semantics query-score-correct.
- ADR for the duplicate-drain decision: `spec/adr/ADR-009-linear-scan-duplicate-heaptids.md`
- ADR for gamma-aware duplicate semantics: `spec/adr/ADR-010-gamma-aware-duplicate-coalescing.md`
- Plan/task tracking now reflects the implemented phases instead of leaving completed work marked as not started.
- FR-007, FR-009, and FR-016 now backport the current staged implementation boundaries into the functional spec.
- ADR for the planner cost gate: `spec/adr/ADR-011-planner-cost-override-until-ordered-scan.md`
- `src/am/mod.rs` splitting has started by extracting planner-cost and vacuum callbacks into dedicated submodules with no behavior change.
- `src/am/mod.rs` now also extracts relation-option parsing and `amoptions` registration into a dedicated module with no behavior change.
- `src/am/mod.rs` now also extracts AM routine assembly plus the SQL handler entrypoints into a dedicated module with no behavior change.
- `src/am/mod.rs` now also extracts the build entry callbacks into a dedicated module while leaving deeper build helpers in place.
- `src/am/mod.rs` now also extracts build tuple decoding and `build_source_column` heap scan plumbing into the build module.
- `src/am/mod.rs` now also extracts graph construction, entry-point selection, and staged data-page writes into the build module.
- The build module now owns `BuildState` and `BuildTuple`, leaving `src/am/mod.rs` focused much more narrowly on live insert and scan behavior.
- Scan descriptor lifecycle, scan-local state, bootstrap linear scan execution, and scan debug helpers now live in `src/am/scan.rs`.
- ADR for long-horizon AM growth boundaries: `spec/adr/ADR-012-am-module-boundaries-for-growth.md`

External review bundles:
- `review/external/2026-04-05-claude-opus/README.md`
- `review/external/2026-04-05-spec-plan-eval/evaluation.md`

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
- Addressed external review `18-varlena-detoast-check-inverted.md` by switching both build detoast paths to pointer-comparison copy detection.
- Addressed external review `14-encoding-dimension-u16-truncation.md` by adding explicit dimension validation before packing tqvector datums.
- Addressed external review `10-page-layout-offset-number-u16-overflow.md` by using a checked offset-number conversion in `DataPage::insert_raw_tuple`.
- Addressed external review `08-hnsw-rs-wildcard-dependency.md` by pinning `hnsw_rs` to the currently locked `0.3.4` release.
- Addressed external review `04-linear-scan-reads-metadata-every-gettuple.md` by caching scan metadata during `amrescan` and reusing it in `amgettuple`.
- Addressed external review `06-relation-options-spi-in-hot-path.md` by reading reloptions directly from `rd_options`.
- Addressed external review `16-score-code-inner-product-allocates-per-call.md` by adding a raw-code scorer that avoids temporary payload allocation.
- Addressed outside feedback on `13-amgettuple-empty-index-noop.md` by adding explicit repeated-empty-scan and empty-rescan coverage.
- Addressed outside feedback on `14-rescan-query-payload-state.md` by rejecting oversized scan queries before storing scan-owned payload state.
- Addressed outside feedback on `15-amgettuple-linear-forward-scan.md` by removing the unreachable saturated-offset overflow branch in the linear scan cursor.
- Addressed outside feedback on `16-query-inner-product-gamma-payload.md` by adding explicit coverage for dimension mismatch errors and the negative-query wrapper.
- Addressed outside feedback on `15-amgettuple-linear-forward-scan.md` with a regression that combines duplicate draining, neighbor-tuple skipping, and multi-page scan advancement.
- Addressed outside feedback on `15-amgettuple-linear-forward-scan.md` by caching the relation block count in scan state instead of re-reading it for each bootstrap scan step.
- Remaining open feedback notes around page-lock batching and larger architectural changes are deferred while ordered scan execution groundwork continues.
- Ordered-scan follow-on work now starts from explicit scan-local current-result state; planner enablement and score emission remain deferred.
- Addressed the next ordered-scan groundwork slice by teaching the bootstrap scan to compute a current-result `<#>` value from the cached prepared query plus persisted candidate `gamma`.
- Duplicate coalescing is now query-score-correct for persisted tqvectors, but future ordered-scan work still needs candidate-local access to `gamma` without representative heap fetches.
- Addressed the next structural boundary by extracting scan execution into `src/am/scan.rs` so traversal work grows in the scan module instead of re-expanding `src/am/mod.rs`.

Review instructions:
- Prefer correctness findings over style comments.
- Focus on behavior, invariants, page/WAL safety, SQL-surface coherence, and missing tests.
- Treat the current on-disk layout as intentional unless a small, concrete defect requires change.
- Keep request files in `review/` and put outside feedback under `review/feedback/<request-slug>/`.

Open requests:
- `13-amgettuple-empty-index-noop.md`
- `14-rescan-query-payload-state.md`
- `15-amgettuple-linear-forward-scan.md`
- `16-query-inner-product-gamma-payload.md`
- `17-linear-scan-exhaustion-and-direction-guards.md`
- `18-linear-scan-rescan-after-exhaustion.md`
- `19-build-detoast-copy-detection.md`
- `20-encode-dimension-boundary.md`
- `21-page-offset-checked-conversion.md`
- `22-pin-hnsw-rs-version.md`
- `23-scan-metadata-cache.md`
- `24-relation-options-cache.md`
- `25-zero-allocation-code-scoring.md`
- `26-scan-prepared-query-cache.md`
- `27-scan-current-result-state.md`
- `28-scan-current-result-lifecycle.md`
- `29-gamma-aware-duplicate-coalescing.md`
- `30-plan-and-spec-backfill.md`
- `31-am-mod-cost-vacuum-split.md`
- `32-am-options-module-split.md`
- `33-am-routine-module-split.md`
- `34-am-build-entrypoints-module-split.md`
- `35-am-build-tuple-and-source-scan-split.md`
- `36-am-build-graph-and-page-staging-split.md`
- `37-am-build-state-type-ownership.md`
- `38-scan-current-result-scoring.md`
- `39-am-scan-module-boundary.md`

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
