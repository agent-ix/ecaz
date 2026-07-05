# Task 28 IVF PQ-FastScan g8 100k Smoke

This packet moves the current PQ-FastScan high-recall profile from 25k to a
derived 100k slice of the local 990k DBPedia anchor.

Profile:

- `storage_format = 'pq_fastscan'`
- `pq_group_size = 8`
- `nlists = 64`
- `training_sample_rows = 2000`
- `rerank = 'heap_f32'`
- `rerank_width = 750`

## Result

The 100k smoke keeps high recall:

| nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|
| 32 | 0.9930 | 0.9998 | 286.73 ms |
| 48 | 1.0000 | 1.0000 | 410.19 ms |

Latency:

| nprobe | count | mean | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|
| 32 | 100 | 280.5 ms | 279.5 ms | 312.5 ms | 323.1 ms |
| 48 | 100 | 409.2 ms | 407.6 ms | 439.6 ms | 496.1 ms |

Build/index:

- Source: first 100000 rows from `ec_hnsw_real_ann_benchmarks_anchor_corpus`
- Queries: first 100 rows from `ec_hnsw_real_ann_benchmarks_anchor_queries`
- Corpus table copy: `36328.304 ms`
- Index build: `156088.030 ms`
- Index size: `18 MB`

## Interpretation

This closes the near-term "re-measure at 100k+" gap for the IVF
PQ-FastScan lane. The profile that emerged at 25k (`pq_group_size=8`,
`rerank_width=750`) still reaches the high-recall band at 100k.

This packet is not a full 100k head-to-head against TurboQuant because no
same-size TurboQuant 100k surface existed locally at the start of the run.
The next comparison slice should build the matching TurboQuant 100k
surface from the same derived 100k corpus and run the same nprobe sweep.

## Recommendation

Carry `pq_group_size=8`, `rerank_width=750` forward as the current
high-recall PQ-FastScan profile. For the next slice:

1. Build a same-fixture TurboQuant 100k reference surface.
2. Compare recall and p50/p95/p99 at nprobe 32/48.
3. If PQ-FastScan remains competitive, test whether `nlists=128` lowers
   scan volume enough to improve 100k latency at the same recall.

## Artifacts

See `artifacts/manifest.md`.
