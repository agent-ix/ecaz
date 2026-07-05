# Review Request: Task 28 IVF nlists=128 Forced-Index Sweep

## Summary

This packet follows packet 30053 by rerunning the n128 sweep with the new
`ecaz bench --force-index` mode from commit `210e4e8`. The flag disables
sequential scans in benchmark sessions so the run isolates IVF access-method
behavior even when the current cost model would choose a sequential scan.

## Fixture

- Local PG18 scratch cluster, database `postgres`.
- Existing isolated surfaces from packet 30053:
  - `task28_ivf_postopt10k_n128w25`
  - `task28_ivf_postopt25k_n128w25`
- Corpus rows / dimensions:
  - 10k x 1536
  - 25k x 1536
- Query set: 100 DBPedia-derived queries.
- IVF reloptions:
  - `nlists = 128`
  - `nprobe = 128` persisted, swept at scan time with `ec_ivf.nprobe`
  - `training_sample_rows = 2000`
  - `storage_format = turboquant`
  - `rerank = heap_f32`
  - `rerank_width = 25`
- Benchmark mode: `--force-index`.
- Cache state: warm local development run.
- Memory high-water mark: not captured.

## Recall / Latency

10k x 1536, `nlists=128`, `rerank_width=25`, forced index:

| nprobe | recall@10 | mean q-time | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|
| 8 | 0.7070 | 35.88 ms | 33.3 ms | 37.2 ms | 43.9 ms |
| 16 | 0.8540 | 49.25 ms | 42.5 ms | 48.9 ms | 50.9 ms |
| 24 | 0.8970 | 53.18 ms | 53.2 ms | 61.5 ms | 67.0 ms |
| 32 | 0.9250 | 64.38 ms | 63.8 ms | 72.4 ms | 87.4 ms |
| 48 | 0.9660 | 86.31 ms | 84.2 ms | 90.6 ms | 97.4 ms |
| 64 | 0.9860 | 113.56 ms | 104.7 ms | 112.7 ms | 116.1 ms |

25k x 1536, `nlists=128`, `rerank_width=25`, forced index:

| nprobe | recall@10 | mean q-time | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|
| 8 | 0.6740 | 56.48 ms | 55.3 ms | 66.1 ms | 76.6 ms |
| 16 | 0.8090 | 89.01 ms | 88.3 ms | 102.3 ms | 118.2 ms |
| 24 | 0.8990 | 118.46 ms | 116.5 ms | 136.5 ms | 144.7 ms |
| 32 | 0.9370 | 144.76 ms | 145.7 ms | 163.0 ms | 165.9 ms |
| 48 | 0.9770 | 202.66 ms | 202.6 ms | 233.6 ms | 247.5 ms |
| 64 | 0.9860 | 264.68 ms | 256.2 ms | 275.1 ms | 287.0 ms |

## Interpretation

`nlists=128` is not the better frontier for this fixture. It can produce
sub-50 ms p50 on 10k only at low recall (`nprobe=16`, recall@10 0.8540).
At high recall, it remains above target and is worse than hoped:

- 10k `nprobe=64` reaches recall@10 0.9860 at p50 104.7 ms.
- 25k `nprobe=64` reaches recall@10 0.9860 at p50 256.2 ms.

Compared to packet 30052, n64 had better recall at equivalent probes. The next
productive direction is not simply more lists. The branch should pivot to:

- `rerank_width` reduction at the best n64 and n128 routing points.
- cost-model repair so normal planning chooses IVF where appropriate.
- posting-list scoring/layout work if high-recall p50 remains above target.

DiskANN remains task 29 and is not included.
