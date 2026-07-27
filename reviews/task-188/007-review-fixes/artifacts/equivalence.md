# Task 188 review-fix equivalence check

The shared latency worker was compared at the pre-refactor parent
`1426c838b` and at the review-fix code checkpoint `2d9f6099b`, with the same
PG18 backend, fixture, command, and `worker_batch_size` left at its default
zero. The extension backend was release build `c1c43a9bf66c25b390535ba47e52e0e251a5d6e7`
in both runs.

Fixture and command:

- 10k DBpedia fixture, HNSW `m=16`, `ef_construction=128`
- prefix `task188_review_fixes_hnsw`, `k=10`, `ef_search=64`
- one worker, 30 timed queries, `--force-index`, cache label
  `post_recall_warm`
- suite config: `task188-review-fixes-equivalence-suite.json`
- backend: PG18 on `/home/peter/.pgrx:28818`, database `tqvector_bench`
- one index on one table; no rerank or alternate storage format

| checkpoint | mean | p50 | p95 | p99 | max |
| --- | ---: | ---: | ---: | ---: | ---: |
| pre-refactor `1426c838b` | 2.56 ms | 2.38 ms | 2.81 ms | 6.34 ms | 7.74 ms |
| review fix `2d9f6099b` | 2.57 ms | 2.38 ms | 2.82 ms | 6.47 ms | 7.89 ms |
| review fix minus baseline | +0.01 ms (+0.4%) | 0.00 ms | +0.01 ms (+0.4%) | +0.13 ms (+2.1%) | +0.15 ms (+1.9%) |

The default path agrees within run-to-run noise: mean, p50, and p95 are
effectively unchanged; the small p99/max differences are expected from a
30-sample local run. The current row explicitly emits `worker_batch_size=0`.
The old row has no provenance column because that field was added by the fix.

Raw rows:

- `run/pre-refactor-1426c838b/latency-10k-hnsw.log`
- `run/current-r2/latency-10k-hnsw.log`
