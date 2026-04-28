# Task 28 IVF 100k PQ-FastScan vs TurboQuant Comparison

This packet adds a same-fixture 100k TurboQuant reference for the 100k
PQ-FastScan g8 surface from packet 30090.

Both surfaces use:

- first 100000 rows from `ec_hnsw_real_ann_benchmarks_anchor_corpus`
- first 100 rows from `ec_hnsw_real_ann_benchmarks_anchor_queries`
- `nlists = 64`
- `nprobe in {32, 48}`
- `training_sample_rows = 2000`
- `rerank = 'heap_f32'`

Profile differences:

- PQ-FastScan: `pq_group_size = 8`, `rerank_width = 750`
- TurboQuant: default TurboQuant payload, `rerank_width = 25`

## Result

Recall is tied at the measured points:

| profile | nprobe | recall@10 | NDCG@10 | recall mean q-time |
|---|---:|---:|---:|---:|
| PQ-FastScan g8 | 32 | 0.9930 | 0.9998 | 286.73 ms |
| TurboQuant | 32 | 0.9930 | 0.9998 | 466.29 ms |
| PQ-FastScan g8 | 48 | 1.0000 | 1.0000 | 410.19 ms |
| TurboQuant | 48 | 1.0000 | 1.0000 | 712.49 ms |

Latency:

| profile | nprobe | p50 | p95 | p99 |
|---|---:|---:|---:|---:|
| PQ-FastScan g8 | 32 | 279.5 ms | 312.5 ms | 323.1 ms |
| TurboQuant | 32 | 464.8 ms | 538.0 ms | 556.8 ms |
| PQ-FastScan g8 | 48 | 407.6 ms | 439.6 ms | 496.1 ms |
| TurboQuant | 48 | 705.7 ms | 760.6 ms | 782.7 ms |

Build/index:

| profile | build time | index size |
|---|---:|---:|
| PQ-FastScan g8 | 156088.030 ms | 18 MB |
| TurboQuant | 136602.323 ms | 87 MB |

## Interpretation

At 100k on this local DBPedia-derived fixture, PQ-FastScan g8 is the
better IVF storage profile at the measured high-recall points:

- same recall as TurboQuant at nprobe 32 and 48
- about 40% lower p50 at both nprobe points
- much smaller index size (`18 MB` vs `87 MB`)
- moderately slower build (`156.1s` vs `136.6s`)

This changes the Task 28 direction: TurboQuant remains a supported IVF
profile, but it should no longer be treated as the presumed best default
for IVF. The next tuning work should continue on PQ-FastScan g8 first.

## Recommendation

For the next slice:

1. Test `nlists=128` with PQ-FastScan g8 on 100k, because the current
   nprobe 48 point scans 75% of lists and may be leaving latency on the
   table.
2. Keep `rerank_width=750` as the first high-recall setting.
3. After nlists tuning, update the A10 recommendation on `storage_format =
   'auto'` for IVF.

## Artifacts

See `artifacts/manifest.md`.
