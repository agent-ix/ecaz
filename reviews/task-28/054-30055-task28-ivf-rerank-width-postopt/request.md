# Review Request: Task 28 IVF Rerank-Width Post-Optimization Sweep

## Summary

This packet follows packets 30052 and 30054 by reducing `rerank_width` on the
best current n64 surface. It tests whether exact heap rerank work is still the
dominant latency lever after heap prefetch and index-internal rerank scoring.

The result is clear: narrowing rerank width is not the next useful latency
lever. Width 10 and width 5 preserve almost all of the posting-list/probe
latency while dropping recall sharply. The n64 indexes were restored to
`rerank_width=25` after the measurements.

## Fixture

- Local PG18 scratch cluster, database `postgres`.
- Existing isolated surfaces from packet 30052:
  - `task28_ivf_postopt10k_n64w25`
  - `task28_ivf_postopt25k_n64w25`
- Corpus rows / dimensions:
  - 10k x 1536
  - 25k x 1536
- Query set: 100 DBPedia-derived queries.
- IVF reloptions before and after packet:
  - `nlists = 64`
  - `nprobe = 64` persisted, swept at scan time with `ec_ivf.nprobe`
  - `training_sample_rows = 2000`
  - `storage_format = turboquant`
  - `rerank = heap_f32`
  - `rerank_width = 25`
- Benchmark mode: `--force-index`.
- Cache state: warm local development run.
- Memory high-water mark: not captured.

## Width 10 Results

10k x 1536, `nlists=64`, `rerank_width=10`, forced index:

| nprobe | recall@10 | mean q-time | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|
| 32 | 0.9260 | 99.72 ms | 96.1 ms | 105.0 ms | 107.0 ms |
| 48 | 0.9370 | 139.37 ms | 138.9 ms | 149.3 ms | 152.0 ms |

25k x 1536, `nlists=64`, `rerank_width=10`, forced index:

| nprobe | recall@10 | mean q-time | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|
| 32 | 0.9280 | 246.19 ms | 244.6 ms | 268.2 ms | 296.1 ms |
| 48 | 0.9390 | 344.30 ms | 348.5 ms | 362.5 ms | 369.0 ms |

## Width 5 Spot Check

10k x 1536, `nlists=64`, `rerank_width=5`, forced index:

| nprobe | recall@10 | mean q-time | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|
| 32 | 0.5000 | 95.34 ms | 93.7 ms | 102.4 ms | 108.4 ms |
| 48 | 0.5000 | 140.55 ms | 135.5 ms | 143.0 ms | 148.0 ms |

## Baseline Comparison

Packet 30052 measured the same n64 surfaces at `rerank_width=25`:

| corpus | nprobe | width 25 recall | width 25 p50 | width 10 recall | width 10 p50 |
|---|---:|---:|---:|---:|---:|
| 10k | 32 | 0.9800 | 98.1 ms | 0.9260 | 96.1 ms |
| 10k | 48 | 1.0000 | 140.2 ms | 0.9370 | 138.9 ms |
| 25k | 32 | 0.9840 | 246.2 ms | 0.9280 | 244.6 ms |
| 25k | 48 | 0.9990 | 351.4 ms | 0.9390 | 348.5 ms |

## Interpretation

Width reduction is preserving the same latency band while discarding the exact
rerank candidates needed for high recall. The dominant work at these routing
points is still probe/posting-list scoring and tuple/candidate handling, not
the last 15 to 20 exact heap-f32 rerank evaluations.

The next useful slice should not keep shaving `rerank_width`. Continue with:

- cost-model repair so normal planning chooses IVF without `--force-index`;
- posting-list scoring/layout work to reduce probe cost;
- the build/training/vacuum deeper pass requested by reviewer seq 02;
- the concurrent insert measurement before broad concurrency claims.

DiskANN remains task 29 and is not included.
