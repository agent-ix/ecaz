# Task 85 Packet 033 Artifact Manifest

- head SHA: `d5bfafb587f08579778a61cd5f79edf4f79d0314`
- task bucket: `reviews/task-85/`
- packet path: `reviews/task-85/033-comparator-product-policy-gate/`
- lane: comparator/product policy gate
- fixture: 1M DBpedia/OpenAI3 where comparator evidence exists
- storage format: SPIRE/IVF RaBitQ, DiskANN pq_fastscan, HNSW unavailable at 1M
- rerank mode: SPIRE retained `rerank_width=25`; IVF `rerank_width=50`;
  DiskANN default `rerank_budget=64`
- surface isolation: cited immutable benchmark and review packets
- timestamp: `2026-06-07T15:42:17-07:00`

## Sources

### SPIRE

Source: `reviews/task-85/023-aws-summary-scoring-single-payload-fast-path/`

Key packet 023 repeat row:

- `recall@10=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- `latency_p50=222.692 ms`
- `latency_p95=275.769 ms`
- `latency_p99=286.980 ms`
- V5 index size `872.1 MiB` / `923.7 B/row`

### IVF/RaBitQ

Source: `benchmarks/task51-aws-ivf-rabitq-final-gate/manifest.md`

Profile: AWS Graviton `m8g.2xlarge`, 990,000 rows, `ec_ivf`,
`storage_format=rabitq`, `rerank_width=50`.

Key rows:

- `nprobe=128`: `recall@10=0.9864`, p50/p95/p99 `34.6/41.5/48.0 ms`
- `nprobe=256`: `recall@10=0.9936`, p50/p95/p99 `66.2/72.5/75.7 ms`
- index size `298.0 MiB` / `315.6 B/row`

### DiskANN

Source: `benchmarks/task59-aws-diskann-final-graviton-suite/manifest.md`

Profile: AWS Graviton `m8g.2xlarge`, 990,000 rows, `pq_fastscan`,
`rerank_budget=64`.

Key 1M rows:

- `L800`: `recall@10=0.9825`, p50/p95/p99 `19.7/30.9/35.6 ms`
- index size `455.1 MiB` / `482.0 B/row`

### HNSW

Sources:

- `benchmarks/task61-aws-hnsw-graviton-baseline/manifest.md`
- `benchmarks/profile-hnsw-1m/manifest.md`
- `benchmarks/profile-hnsw-1m/artifacts/suite-manifest.json`
- `benchmarks/profile-hnsw-1m/artifacts/load-hnsw-real1m-m16.log`

Key disposition:

- AWS Graviton Task 61 completed 10k/50k/100k only and explicitly deferred 1M
  because the 100 GiB data volume had insufficient free space.
- The local focused 1M packet has only the load/build log populated. Its
  `suite-manifest.json` still marks recall, latency, and storage steps as
  `pending`, so it is not a completed 1M comparator row.

## Decision

SPIRE remains research/opt-in for 1M. Packet 023 is the strongest SPIRE point,
but IVF/RaBitQ at `nprobe=256` has higher recall, roughly one third the p99
latency, and roughly one third the index size.
