# Review Request: Task 28 IVF Post-Optimization Smoke

## Summary

This packet records the first post-optimization smoke after the reviewer
follow-up items for heap rerank prefetch, internal rerank scoring, quantizer
dispatch, and candidate-dedup pooling.

It uses isolated one-index-per-table surfaces so planner choice is not
confounded by the older shared-table IVF indexes.

## Fixture

- Local PG18 scratch cluster, database `postgres`.
- One-index-per-table surfaces:
  - `task28_ivf_postopt10k_n32w25`
  - `task28_ivf_postopt25k_n32w25`
- Corpus rows / dimensions:
  - 10k x 1536
  - 25k x 1536
- Query set: 100 DBPedia-derived queries from
  `task28_ivf_anchor10k1536_queries100`.
- IVF reloptions:
  - `nlists = 32`
  - `nprobe = 32` persisted, swept at scan time with `ec_ivf.nprobe`
  - `training_sample_rows = 2000`
  - `storage_format = turboquant`
  - `rerank = heap_f32`
  - `rerank_width = 25`
- Cache state: warm local development run. The latency run followed index
  build and recall on the same scratch cluster.
- Memory high-water mark: not captured in this smoke.

## Build / Size

From `artifacts/create_postopt_n32w25_surfaces.log`:

| surface | build time | index size |
|---|---:|---:|
| 10k n32 width25 | 25.876 s | 9160 kB |
| 25k n32 width25 | 47.589 s | 22 MB |

## Recall / Latency

10k x 1536, `nlists=32`, `rerank_width=25`:

| nprobe | recall@10 | mean q-time | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|
| 16 | 0.9800 | 108.17 ms | 103.5 ms | 114.4 ms | 118.8 ms |
| 24 | 0.9980 | 142.24 ms | 141.3 ms | 181.4 ms | 197.9 ms |
| 32 | 1.0000 | 187.02 ms | 186.4 ms | 223.8 ms | 248.3 ms |

25k x 1536, `nlists=32`, `rerank_width=25`:

| nprobe | recall@10 | mean q-time | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|
| 16 | 0.9480 | 242.21 ms | 231.7 ms | 266.7 ms | 278.8 ms |
| 24 | 0.9760 | 341.40 ms | 336.8 ms | 385.9 ms | 473.9 ms |
| 32 | 1.0000 | 457.31 ms | 445.7 ms | 457.3 ms | 465.4 ms |

## Interpretation

The optimized scan path is correct, but this narrow smoke does not show the
sub-50 ms 10k target at high recall. The best 10k point here is `nprobe=24`
for `recall@10 = 0.9980`, but p50 is still 141.3 ms. The 25k point needs
`nprobe=32` for perfect recall and remains roughly 446 ms p50.

That says the next IVF slice should not be more small rerank-loop tuning. The
cost is still dominated by scanning too much posting-list work for the selected
recall target.

## Next Recommendation

Build the next isolated surfaces around the structural knobs:

- `nlists=64` and `nlists=128` with the same 100-query fixture.
- Sweep lower `nprobe` values against those higher-list indexes.
- Include a smaller `rerank_width` sweep once recall is near 0.99, because
  width 25 may be doing more exact heap work than the useful frontier needs.

If those points still stay above 100 ms p50 on 10k, the substrate needs a
posting-list scoring/layout pass before this becomes a product benchmark
candidate.

DiskANN remains task 29 and is not included.
