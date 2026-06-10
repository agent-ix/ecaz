# Task 94 Review Request: HNSW Scalar Traversal Disposition

## Scope

This no-code packet addresses Packet 007 feedback F1 and the readiness-series
correction: HNSW grouped-PQ has codec-level batch registration, but production
HNSW traversal remains per-candidate scalar in Task 94.

## Code

- `088910672` - `Document Task 94 HNSW scalar traversal disposition`

## Changes

- Added an explicit HNSW out-of-scope note to `plan/tasks/94-grouped-pq-block-kernel-family.md`.
- Added `artifacts/hnsw-disposition.md` as the closeout-ready wording:
  - HNSW codec-level batch override exists and is tested.
  - Production greedy search still calls `score_grouped_search_code_result` one search code at a time.
  - Real closeout benchmarks should not expect `surface=hnsw, quant=grouped_pq` kernel rows.
  - HNSW traversal batching is follow-up scope if wanted later.

## Validation

```text
git diff --check -- plan/tasks/94-grouped-pq-block-kernel-family.md reviews/task-94/017-hnsw-scalar-disposition
```

Result: passed.

No CI, AWS, tests, or benchmark runs were used for this packet.
