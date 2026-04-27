# Review Request: Task 28 IVF nlists=128 Planner Blocker

## Summary

This packet continues the post-optimization nlists sweep from packets 30051
and 30052, but `nlists=128` exposed a planner/cost blocker before a clean
recall/latency grid could be recorded.

The isolated n128 indexes built successfully. The normal `ecaz bench recall`
run on the 10k surface did not complete a result table after several minutes,
so it was stopped. A follow-up single-query EXPLAIN shows the normal planner
choosing a sequential scan for the n128 surface. Forcing the index path with
`enable_seqscan=off` returns the same nprobe=8 single-query shape in about
68 ms.

## Fixture

- Local PG18 scratch cluster, database `postgres`.
- One-index-per-table surfaces:
  - `task28_ivf_postopt10k_n128w25`
  - `task28_ivf_postopt25k_n128w25`
- Corpus rows / dimensions:
  - 10k x 1536
  - 25k x 1536
- Query set: 100 DBPedia-derived queries from
  `task28_ivf_anchor10k1536_queries100`.
- IVF reloptions:
  - `nlists = 128`
  - `nprobe = 128` persisted; attempted recall sweep at scan time
  - `training_sample_rows = 2000`
  - `storage_format = turboquant`
  - `rerank = heap_f32`
  - `rerank_width = 25`

## Build / Size

From `artifacts/create_postopt_n128w25_surfaces.log`:

| surface | build time | index size |
|---|---:|---:|
| 10k n128 width25 | 80.349 s | 9928 kB |
| 25k n128 width25 | 134.008 s | 23 MB |

## Planner Finding

Normal planner path for one 10k n128 nprobe=8 query:

- `EXPLAIN (ANALYZE, BUFFERS)` selected a sequential scan.
- Execution time: 4426.320 ms.
- Buffers: shared hit=50145 read=6.

Forced index path for the same query shape:

- `SET enable_seqscan = off; SET ec_ivf.nprobe = 8`.
- `EXPLAIN (ANALYZE, BUFFERS)` selected
  `task28_ivf_postopt10k_n128w25_idx`.
- Execution time: 67.987 ms.
- Buffers: shared hit=370 read=192.

## Interpretation

This is not a usable n128 latency/recall point yet. It is a planner-cost
blocker for the n128 tuning lane:

- The IVF index path can run, and the forced-index single query is much faster
  than the sequential scan.
- The normal benchmark path can fall off the IVF path for n128, making a plain
  CLI recall/latency table misleading.

## Next Recommendation

Before continuing n128 or product-facing measurements:

1. Add a benchmark-only force-index mode to `ecaz bench recall` and
   `ecaz bench latency`, or make those benchmark commands set
   `enable_seqscan=off` explicitly and document that they isolate AM
   performance.
2. Revisit `ec_ivf` cost estimates for high `nlists`, because the planner
   thinks the n128 IVF path is more expensive than a sequential scan even for
   KNN.
3. After that, rerun the n128 grid and then sweep `rerank_width` at the best
   routing point.

DiskANN remains task 29 and is not included.
