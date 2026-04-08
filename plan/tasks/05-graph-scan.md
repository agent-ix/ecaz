# Task 05: Graph Scan

Status: A3 complete, A4 ready

Progress notes:
- Build path is complete.
- Scan lifecycle, query validation, metadata/prepared-query caching, and bootstrap linear scan are implemented.
- AM module split is complete: scan, insert, build, options, cost, vacuum, routine, shared, and
  search all have dedicated modules.
- Planner-facing `ef_search` control-surface groundwork can land independently of ordered scan
  enablement.
- Graph now owns layer-0 neighbor loading and visible-seed expansion helpers.
- Search now owns the bootstrap visible-frontier protocol: trace seeding, discovered-candidate
  registration, scheduler-aware selection/consumption, post-consume progression, visible-seed
  top-up, and single-source refill.
- A3 is closed: graph/search traversal is now the primary ordered scan path, and the linear path
  is the explicit fallback shell only when graph traversal cannot produce an initial ordered result.
- Planner enablement remains gated by ADR-011 and is not part of this task until recall clears A4.

## Scope

Complete the HNSW scan path from module split through validated recall measurement.

## Subtasks

- [x] **A1: Finish am split.** Extract insert and scan into `am/insert.rs` and `am/scan.rs`. Mechanical refactor, no logic changes.
- [x] **A2: Graph traversal helpers.** The shared layer-0 traversal seam is now in place:
  graph owns neighbor loading and visible-seed expansion, and search owns the bootstrap
  visible-frontier protocol around that traversal.
- [x] **A3: Wire scan.** Graph-first ordered scan runtime is cursor-owned end-to-end on `main`.
  1. `amrescan` prefills the first ordered graph result through `GraphTraversalCursor`.
  2. `amgettuple` drains prefetched graph results and refreshes through the graph cursor.
  3. Linear scan remains the explicit fallback shell only when graph traversal cannot produce the
     initial ordered result.
  4. Retired bootstrap helpers are gated to test/debug surfaces.
- [ ] **A4: Recall gate.** Measure Recall@10 on synthetic data (10K+ vectors, 1536-dim, 4-bit). Brute-force fp32 ground truth. Test at (m=8,ef=40), (m=8,ef=128), (m=8,ef=200), (m=16,ef=200). Gate: Recall@10 >= 89% at m=8 ef=128. If not met, investigate before proceeding.

## Owns

- `FR-009`
- Initial `NFR-003` validation (recall gate only)

## Dependencies

- Tasks 01-04 (all complete)

## Unblocks

- Task 06 (graph-aware insert needs traversal helpers from A2)
- Task 07 (vacuum needs traversal helpers from A2)
- Task 10 (full benchmarks need working scan)
- Task 11 / D2 (planner wiring gated on A4 recall gate)
- End-to-end indexed ANN search

## Deliverables

- `am/scan.rs` and `am/insert.rs` extracted modules
- `hnsw_search` shared traversal helper
- `amgettuple` returning distance-ordered results
- `tqhnsw.ef_search` GUC
- Recall@10 measurement harness and initial results

## Primary Tests

- `TC-113`, `TC-120`, `TC-121`, `TC-131`: scan behavior
- Recall@10 benchmark at multiple (m, ef) configurations
- Buffer pin bound check during scan of 50K-row index

## Notes

- A2 is the hardest subtask. Reference pgvector `hnsw_search_layer` in `hnswscan.c` (~150 lines) but expect more complexity due to raw page tuple decoding.
- The traversal helper should be generic over scoring function signature to support both scan (LUT `score_ip_encoded`) and insert (code-to-code `score_ip_codes_lite`).
- A4 is a gate: if recall fails, all downstream work (insert, vacuum, benchmarks) is premature.
- Ordered result buffering / graph-first execution are now in place on `main`; A4 should both
  measure recall and leave behind an integration-level ordered-result regression test.
- Post-v0.1 follow-up notes from the A3 close review:
  1. The raw `self as *mut Self` aliasing pattern in `GraphTraversalPrefetchContext::run` is
     contained and approved, but a future search API that takes a visitor/trait object would be
     cleaner.
  2. Graph exhaustion currently transitions to `Exhausted`, not back to linear fallback. Keep that
     contract for v0.1 and revisit after A5 introduces live-write connectivity gaps.
  3. `with_visible_frontier_mut_and_bootstrap_expansion` is still a borrow-splitting helper that
     could disappear if graph-phase state is later separated from `TqScanOpaque`.
  4. `GraphTraversalCursor` is reconstructed several times per call site; likely inlined away, but
     could be held for readability in a future optimization pass.
