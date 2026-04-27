# Review Request: Task 28 IVF nlists=64 Post-Optimization Sweep

## Summary

This packet extends packet 30051 by moving from `nlists=32` to `nlists=64`
on the same isolated one-index-per-table 10k/25k DBPedia-derived surfaces.

## Fixture

- Local PG18 scratch cluster, database `postgres`.
- One-index-per-table surfaces:
  - `task28_ivf_postopt10k_n64w25`
  - `task28_ivf_postopt25k_n64w25`
- Corpus rows / dimensions:
  - 10k x 1536
  - 25k x 1536
- Query set: 100 DBPedia-derived queries from
  `task28_ivf_anchor10k1536_queries100`.
- IVF reloptions:
  - `nlists = 64`
  - `nprobe = 64` persisted, swept at scan time with `ec_ivf.nprobe`
  - `training_sample_rows = 2000`
  - `storage_format = turboquant`
  - `rerank = heap_f32`
  - `rerank_width = 25`
- Cache state: warm local development run. Latency followed index build and
  recall on the same scratch cluster.
- Memory high-water mark: not captured in this sweep.

## Build / Size

From `artifacts/create_postopt_n64w25_surfaces.log`:

| surface | build time | index size |
|---|---:|---:|
| 10k n64 width25 | 44.356 s | 9416 kB |
| 25k n64 width25 | 76.272 s | 22 MB |

## Recall / Latency

10k x 1536, `nlists=64`, `rerank_width=25`:

| nprobe | recall@10 | mean q-time | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|
| 8 | 0.7800 | 35.45 ms | 34.9 ms | 41.1 ms | 47.0 ms |
| 16 | 0.9010 | 55.74 ms | 55.9 ms | 62.6 ms | 65.9 ms |
| 24 | 0.9580 | 80.56 ms | 77.3 ms | 86.3 ms | 89.1 ms |
| 32 | 0.9800 | 99.52 ms | 98.1 ms | 105.9 ms | 111.7 ms |
| 48 | 1.0000 | 139.94 ms | 140.2 ms | 148.2 ms | 158.4 ms |

25k x 1536, `nlists=64`, `rerank_width=25`:

| nprobe | recall@10 | mean q-time | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|
| 8 | 0.8130 | 79.40 ms | 79.9 ms | 92.3 ms | 97.3 ms |
| 16 | 0.9110 | 137.84 ms | 141.3 ms | 179.8 ms | 216.9 ms |
| 24 | 0.9630 | 192.30 ms | 193.3 ms | 209.0 ms | 217.9 ms |
| 32 | 0.9840 | 248.14 ms | 246.2 ms | 261.0 ms | 264.5 ms |
| 48 | 0.9990 | 353.31 ms | 351.4 ms | 383.2 ms | 584.5 ms |

## Interpretation

Increasing to `nlists=64` gives the expected speed/recall tradeoff and is much
faster than the `nlists=32` high-recall point at the same corpus size, but it
still does not reach the target frontier:

- 10k can hit sub-50 ms p50 only at `nprobe=8`, where recall is 0.7800.
- 10k reaches perfect recall at `nprobe=48`, but p50 is still 140.2 ms.
- 25k reaches 0.9990 recall at `nprobe=48`, but p50 is 351.4 ms.

The next slice should test `nlists=128` and then vary `rerank_width`, because
the current width 25 may be too expensive once routing quality improves.

DiskANN remains task 29 and is not included.
