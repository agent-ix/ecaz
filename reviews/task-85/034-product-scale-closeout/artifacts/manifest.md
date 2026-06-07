# Task 85 Packet 034 Artifact Manifest

- head SHA: `d5bfafb587f08579778a61cd5f79edf4f79d0314`
- task bucket: `reviews/task-85/`
- packet path: `reviews/task-85/034-product-scale-closeout/`
- lane: Task 85 final closeout
- fixture: AWS 1M/q500 for accepted and rejected SPIRE claims
- storage format: RaBitQ for SPIRE and IVF comparator; pq_fastscan for DiskANN
- rerank mode: SPIRE retained `rerank_width=25`; IVF `rerank_width=50`;
  DiskANN default `rerank_budget=64`
- surface isolation: shared corpus table, one-index-per-profile where packet
  manifests state that surface
- timestamp: `2026-06-07T15:42:17-07:00`

## Closeout Evidence

| Packet | Role |
| --- | --- |
| `reviews/task-85/023-aws-summary-scoring-single-payload-fast-path/` | strongest accepted SPIRE same-recall latency point |
| `reviews/task-85/027-aws-local-heap-fetch-order/` | rejected local heap TID fetch ordering |
| `reviews/task-85/030-aws-local-heap-prefetch/` | rejected local heap block prefetch |
| `reviews/task-85/032-candidate-surface-stop-condition/` | candidate-surface stop condition |
| `reviews/task-85/033-comparator-product-policy-gate/` | comparator and product policy gate |
| `benchmarks/task51-aws-ivf-rabitq-final-gate/` | IVF/RaBitQ 1M comparator |
| `benchmarks/task59-aws-diskann-final-graviton-suite/` | DiskANN 1M comparator |
| `benchmarks/task61-aws-hnsw-graviton-baseline/` | HNSW AWS comparator limitation |
| `benchmarks/profile-hnsw-1m/` | local HNSW 1M incomplete-run limitation |

## Key Result Lines

Accepted SPIRE packet 023 repeat:

- `recall@10=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- p50/p95/p99 `222.692/275.769/286.980 ms`
- index `872.1 MiB`

Rejected rerank-locality packet 027 repeat:

- `recall@10=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- p50/p95/p99 `228.595/284.140/295.823 ms`

Rejected rerank-locality packet 030 repeat:

- `recall@10=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- p50/p95/p99 `227.414/282.375/297.652 ms`

IVF/RaBitQ comparator:

- `nprobe=256`: `recall@10=0.9936`, p50/p95/p99
  `66.2/72.5/75.7 ms`, index `298.0 MiB`

AWS final status:

- `artifacts/cloud-status-final-closeout.log` records the final `1m` status.
