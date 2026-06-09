# Task 94 HNSW Grouped-PQ Disposition

Task 94 closeout should treat HNSW grouped-PQ as codec-surface registered but
production traversal scalar-only.

## What Is Implemented

- `HnswGroupedPqScanCodec::score_ip_batch` supports grouped-PQ batch scoring
  through `score_grouped_pq_batch_for(surface=Hnsw, quant=GroupedPq)`.
- Packet 007 validates that codec-level batch override with a 39-candidate
  unit test and exact counter attribution.

## What Production HNSW Traversal Does

- The HNSW greedy-search path still scores grouped-PQ search codes one at a
  time through `score_grouped_search_code_result`.
- That function calls `QuantCodec::score_ip_candidate`, not
  `QuantCodec::score_ip_batch`.
- Therefore real HNSW grouped-PQ scans are expected to remain per-candidate
  scalar for Task 94.

## Counter Expectations

- Real Task 94 closeout benchmark evidence should not expect
  `surface=hnsw, quant=grouped_pq` kernel rows from production HNSW scans.
- `surface=hnsw, quant=grouped_pq` rows can appear in codec-surface unit tests,
  but those are not traversal-level production evidence.
- If HNSW traversal batching is wanted later, it should be a follow-up task
  that introduces a natural batch boundary in greedy search, analogous to the
  DiskANN traversal batching work in packet 009.

## Source References

- Reviewer finding: `reviews/task-94/007-grouped-pq-diskann-hnsw-codec-registration/feedback/2026-06-09-01-reviewer.md`
- Acknowledged readiness correction: `reviews/task-94/011-local-readiness-matrix/feedback/2026-06-09-01-reviewer.md`
- Task-file clarification: `plan/tasks/94-grouped-pq-block-kernel-family.md`
- HNSW scalar call site: `src/am/ec_hnsw/scan.rs`, `score_grouped_search_code_result`
