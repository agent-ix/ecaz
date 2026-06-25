# Task 119: M5 Counter-Bearing Sidecar Matrix

## Summary

This packet reruns the Task 119 required sidecar matrix on the M5 host using
the counter columns added in `4614d4c0ef8dbf4b8072aaa60773325f4a74b7f5`.

It measures:

```text
HNSW RaBitQ 1-bit candidate frontier + second-stage rerank representation
```

Across 10k, 50k, and 100k with the required variants:

- `f32`
- `rabitq2`, `rabitq4`, `rabitq8`
- `turboquant_2bit`, `turboquant_3bit`, `turboquant_4bit`,
  `turboquant_5bit`, `turboquant_6bit`, `turboquant_7bit`,
  `turboquant_8bit`

Each JSONL file has 33 rows: 11 variants x `ef_search={320,500,1000}`.

## Artifacts

- Manifest: `reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/manifest.md`
- 10k results: `reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/suite-results.10k.jsonl`
- 50k results: `reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/suite-results.50k.jsonl`
- 100k results: `reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/suite-results.100k.jsonl`
- Full logs:
  - `reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/sidecar-10k-hnsw-rabitq-required-rerank-matrix.log`
  - `reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/sidecar-50k-hnsw-rabitq-required-rerank-matrix.log`
  - `reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/sidecar-100k-hnsw-rabitq-required-rerank-matrix.log`

## Outcome

This supersedes packet `003` for the free-I/O sidecar matrix because it includes
explicit counters:

- `frontier_p50/p95`
- `reranked_p50/p95`
- `heap_source_reads_p50/p95`
- `emitted_p50/p95`

At `ef_search=1000`, every required variant reports:

- `frontier_p50=1000`
- `reranked_p50=1000`
- `emitted_p50=10`
- `heap_source_reads_p50=0`

The zero heap/source read count is expected because these are `read_mode=free`
upper-bound runs.

The result still points to the same product conclusion:

- `f32` is the recall ceiling but too large for a storage win.
- `rabitq8` is the fastest compact scorer but loses too much recall at 50k/100k.
- `turboquant_4bit` is the best compact CPU/scoring Pareto point in this M5
  harness.
- `turboquant_8bit` recovers much more recall but is too slow in the current
  scoring implementation.

## Closeout Status

This packet satisfies the required free-I/O 10k/50k/100k representation matrix
with explicit counters. It still does not prove production heap/source read
behavior because `read_mode=free` intentionally excludes sidecar I/O. A
production-style M5 sidecar read packet should follow for the viable lanes
before final Task 119 closeout.
