# Task 05: Graph Scan

Status: in progress

Progress notes:
- Build path is complete.
- Scan lifecycle, query validation, metadata/prepared-query caching, and bootstrap linear scan are implemented.
- AM module split in progress: cost, vacuum, options, routine, build extracted; insert and scan extraction pending.
- Graph traversal, ef_search, distance ordering, and planner enablement remain pending.

## Scope

Complete the HNSW scan path from module split through validated recall measurement.

## Subtasks

- [ ] **A1: Finish am split.** Extract insert and scan into `am/insert.rs` and `am/scan.rs`. Mechanical refactor, no logic changes.
- [ ] **A2: Graph traversal helpers.** Implement shared greedy descent + beam search over Postgres buffer pages. Parameterize by scoring function so scan (LUT) and insert (code-to-code) can both use it. Key concerns: buffer pin discipline, visited set, BinaryHeap ordering, layer traversal.
- [ ] **A3: Wire scan.** Replace `next_linear_scan_heap_tid` with graph-based search. `amrescan` calls traversal helper and stores scored results. `amgettuple` pops from BinaryHeap in distance order. Add `tqhnsw.ef_search` GUC. ADR-011 cost gate remains active — planner cost activation is in Task 11 (Track D).
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
