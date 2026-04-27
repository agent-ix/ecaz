# Review Request: Task 28 IVF 100-Query Anchor Check

## Summary

This packet broadens the query sample from 20 to 100 ordered DBPedia anchor
queries after commit `a9d70f3`. It compares three candidate IVF points from the
previous grids against a packet-local exact top-10 table.

No DiskANN implementation or measurement is included.

## Fixture

- corpus table: `task28_ivf_anchor10k1536_heap_corpus`
- query table: `task28_ivf_anchor10k1536_queries100`, copied from
  `ec_hnsw_real_ann_benchmarks_anchor_queries`
- rows: 10,000
- dimensions: 1536
- queries: first 100 ordered anchor queries
- exact truth: seq-scan top-10, `task28_ivf_anchor10k1536_exact100_top10`
- storage: `turboquant`
- rerank: `heap_f32`
- cache state: normal local scratch state; not cold-cache controlled

## Results

Exact truth materialization:

| metric | result |
|---|---:|
| exact rows | 1,000 |
| exact queries | 100 |
| exact materialization time | `07:21.785` |

Candidate points:

| nlists | nprobe | width | build | materialize | recall@10 | hits |
|---:|---:|---:|---:|---:|---:|---:|
| 32 | 16 | 50 | `24.418 s` | `10.083 s` | `0.9800` | 980/1000 |
| 32 | 32 | 25 | `24.329 s` | `18.877 s` | `1.0000` | 1000/1000 |
| 64 | 16 | 25 | `42.214 s` | `10.880 s` | `0.9800` | 980/1000 |

Latency loop:

| nlists | nprobe | width | p50 | p95 | p99 | avg |
|---:|---:|---:|---:|---:|---:|---:|
| 32 | 16 | 50 | `100.013 ms` | `120.946 ms` | `123.989 ms` | `101.617 ms` |
| 32 | 32 | 25 | `177.806 ms` | `202.947 ms` | `215.933 ms` | `171.247 ms` |
| 64 | 16 | 25 | `108.563 ms` | `131.297 ms` | `135.519 ms` | `110.231 ms` |

## Interpretation

The 100-query sample keeps the same direction as the 20-query packet but
exposes boundary misses for partial routing. Both `32/16,width=50` and
`64/16,width=25` drop from `0.9400` on 20 queries to `0.9800` on 100 queries
only because the denominator is larger and the misses are 20/1000; they are
fast but not exact-oracle quality.

The best local full-recall point remains `nlists=32,nprobe=32,width=25`, with
`1.0000` recall@10 on 100 queries and p50/p95 around `178/203 ms`.

## Next Slice Recommendation

Before trying a larger 50k corpus, run a small middle-ground routing check:

- `nlists=32,nprobe=24,width=25`
- `nlists=32,nprobe=24,width=50`
- `nlists=64,nprobe=24,width=25`
- `nlists=64,nprobe=24,width=50`

Use the already materialized `task28_ivf_anchor10k1536_exact100_top10` truth
table to avoid recomputing the 7-minute exact baseline.

## Artifacts

- `artifacts/pg18-ivf-anchor10k1536-anchor100-query-check.sql`
- `artifacts/pg18-ivf-anchor10k1536-anchor100-query-check.log` (failed fixture attempt; only 20 local queries, no claims)
- `artifacts/pg18-ivf-anchor10k1536-anchor100-query-check-rerun.log`
- `artifacts/manifest.md`

## Validation

Packet-only change.

- `git diff --check`
