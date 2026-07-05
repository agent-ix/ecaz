# Review Request: RaBitQ Leaf Block Pruning Benchmark

Code commit measured: `b27202e08d02dda7fee8f81dd9f81d83e5c86a8f`

This packet benchmarks the first direct Task 79 candidate-surface reduction
implementation for the primary/default RaBitQ lane. It is a negative benchmark
packet: the implementation reduces candidates and latency, but the current
mean-summary block selector does not preserve recall.

## Result

The V3 no-prune row reproduces the high-recall baseline:

- candidates: `15,506,227`
- p50 latency: `62.907 ms`
- p95 latency: `71.058 ms`
- recall@10: `0.9975`

The best-looking candidate/latency tradeoff by raw throughput is not acceptable:

- block64 prune4 candidates: `4,547,347`
- block64 prune4 p50 latency: `37.292 ms`
- block64 prune4 recall@10: `0.7790`

The highest-recall pruned row also fails the recall gate:

- block128 prune4 candidates: `8,464,459`
- block128 prune4 p50 latency: `45.882 ms`
- block128 prune4 recall@10: `0.8835`

## Key Table

| Step | Candidate sum | p50 latency | p95 latency | recall@10 |
| --- | ---: | ---: | ---: | ---: |
| block64 prune0 | 15,506,227 | 62.907 ms | 71.058 ms | 0.9975 |
| block64 prune2 | 2,236,837 | 31.361 ms | 37.082 ms | 0.6075 |
| block64 prune3 | 3,408,906 | 33.926 ms | 36.969 ms | 0.7095 |
| block64 prune4 | 4,547,347 | 37.292 ms | 43.306 ms | 0.7790 |
| block64 prune6 | 6,760,069 | 42.130 ms | 46.094 ms | 0.8685 |
| block32 prune6 | 3,523,523 | 35.352 ms | 40.180 ms | 0.7850 |
| block32 prune8 | 4,679,339 | 38.159 ms | 43.018 ms | 0.8405 |
| block32 prune10 | 5,821,849 | 40.774 ms | 44.325 ms | 0.8820 |
| block128 prune2 | 4,248,302 | 35.613 ms | 41.697 ms | 0.6740 |
| block128 prune3 | 6,436,616 | 40.860 ms | 45.935 ms | 0.8110 |
| block128 prune4 | 8,464,459 | 45.882 ms | 52.635 ms | 0.8835 |

## Interpretation

This directly addresses the Task 79 candidate-surface problem, but the first
selector is too lossy. A single encoded block centroid does not rank the blocks
that contain high-scoring outlier vectors well enough for high-recall search.

I am not running TurboQuant comparison for this packet because the primary RaBitQ
lane failed the recall gate first. TurboQuant remains a comparison/control target
after RaBitQ has a viable operating point.

## Validation And Artifacts

See `artifacts/manifest.md`.

- `ecaz bench suite audit`: pass
- `ecaz bench suite run --dry-run`: pass
- `cargo build -p ecaz-cli`: pass
- `ecaz dev install ecaz-pg-test --pg 18`: pass
- final `ecaz bench suite run`: 15 steps succeeded, 0 failed
- `ecaz bench suite report`: produced `artifacts/report-results.jsonl`

## Next Work

The next implementation should replace mean-only top-N block selection with a
recall-preserving selector, most likely an upper-bound score or a
multi-representative summary per block. The performance target remains the Task
79 gate: recall in the target band, candidates at or below 5.2M with a strong
goal at or below 4.0M, and p50 latency at or below 45 ms or at least 25% better
than baseline.
