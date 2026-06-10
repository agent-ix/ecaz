# Task 97 Local Closeout Bench Summary

- Head SHA under test: `4804791fda0f6d62c1c520e349bd71798b436247`
- Host class: local Intel AVX2
- PostgreSQL: PG18 via `/home/peter/.pgrx`, port `28818`
- Suite: `task97-local-closeout-qjl32-suite`
- Status: `completed=34 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- Result rows: `results.jsonl` has 118 rows: 30 load, 28 recall, 60 latency/counter rows.
- Scope: QJL32 TurboQuant, dim 1024, corpus sizes 512 and 4096, 64 queries, 150 latency iterations.
- Surfaces: IVF, SPIRE, HNSW.
- AWS/CI: not run.

## Recall Parity

Every batch-on recall cell matched the corresponding batch-off cell exactly.

| Surface | Rows | Knob | Batch on recall@k | Batch off recall@k |
| --- | ---: | --- | ---: | ---: |
| IVF | 512 | nprobe=8 | 0.6109 | 0.6109 |
| IVF | 512 | nprobe=16 | 0.8016 | 0.8016 |
| SPIRE | 512 | nprobe=8 | 0.6500 | 0.6500 |
| SPIRE | 512 | nprobe=16 | 1.0000 | 1.0000 |
| HNSW | 512 | ef_search=32 | 0.6328 | 0.6328 |
| HNSW | 512 | ef_search=64 | 0.7406 | 0.7406 |
| IVF | 4096 | nprobe=8 | 0.2359 | 0.2359 |
| IVF | 4096 | nprobe=16 | 0.4016 | 0.4016 |
| IVF | 4096 | nprobe=32 | 0.6125 | 0.6125 |
| SPIRE | 4096 | nprobe=8 | 0.2359 | 0.2359 |
| SPIRE | 4096 | nprobe=16 | 0.4031 | 0.4031 |
| SPIRE | 4096 | nprobe=32 | 0.6797 | 0.6797 |
| HNSW | 4096 | ef_search=32 | 0.1531 | 0.1531 |
| HNSW | 4096 | ef_search=64 | 0.2672 | 0.2672 |

## End-to-End Latency

Speedup is batch-off p50 divided by batch-on p50.

| Surface | Rows | Knob | On p50 | Off p50 | p50 speedup | On p95 | Off p95 | On p99 | Off p99 |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| IVF | 512 | nprobe=8 | 1.01 ms | 1.15 ms | 1.14x | 1.17 ms | 1.27 ms | 1.25 ms | 1.38 ms |
| IVF | 512 | nprobe=16 | 1.20 ms | 1.46 ms | 1.22x | 1.41 ms | 1.56 ms | 1.59 ms | 1.68 ms |
| SPIRE | 512 | nprobe=8 | 9.03 ms | 9.33 ms | 1.03x | 9.86 ms | 10.3 ms | 10.2 ms | 11.3 ms |
| SPIRE | 512 | nprobe=16 | 17.2 ms | 17.8 ms | 1.03x | 18.1 ms | 19.5 ms | 20.2 ms | 20.9 ms |
| HNSW | 512 | ef_search=32 | 1.66 ms | 1.72 ms | 1.04x | 1.85 ms | 1.90 ms | 1.99 ms | 2.13 ms |
| HNSW | 512 | ef_search=64 | 2.07 ms | 1.99 ms | 0.96x | 2.46 ms | 2.21 ms | 2.93 ms | 2.51 ms |
| IVF | 4096 | nprobe=8 | 1.34 ms | 1.64 ms | 1.22x | 1.52 ms | 1.90 ms | 2.02 ms | 2.11 ms |
| IVF | 4096 | nprobe=16 | 1.60 ms | 2.30 ms | 1.44x | 1.74 ms | 2.59 ms | 1.82 ms | 3.12 ms |
| IVF | 4096 | nprobe=32 | 2.09 ms | 3.57 ms | 1.71x | 2.29 ms | 3.88 ms | 2.42 ms | 4.64 ms |
| SPIRE | 4096 | nprobe=8 | 18.5 ms | 19.2 ms | 1.04x | 21.6 ms | 21.5 ms | 25.6 ms | 23.5 ms |
| SPIRE | 4096 | nprobe=16 | 35.3 ms | 35.9 ms | 1.02x | 38.9 ms | 38.9 ms | 41.3 ms | 40.4 ms |
| SPIRE | 4096 | nprobe=32 | 69.0 ms | 68.8 ms | 1.00x | 77.2 ms | 74.2 ms | 84.0 ms | 76.9 ms |
| HNSW | 4096 | ef_search=32 | 2.09 ms | 2.29 ms | 1.10x | 2.46 ms | 2.72 ms | 3.00 ms | 3.21 ms |
| HNSW | 4096 | ef_search=64 | 2.98 ms | 3.11 ms | 1.04x | 3.41 ms | 3.60 ms | 3.74 ms | 4.06 ms |

## Direct Scoring Counter Notes

The suite emitted direct `[block-kernel-counters]` rows in the latency logs and parsed them into `results.jsonl`.

SPIRE has scalar-off direct counter baselines and shows the intended scoring-path win:

| Rows | Knob | Batch-on AVX2 + scalar elapsed | Batch-off scalar elapsed | Direct scoring speedup |
| ---: | --- | ---: | ---: | ---: |
| 512 | nprobe=8 | 13.645134 ms | 34.230285 ms | 2.51x |
| 512 | nprobe=16 | 27.742060 ms | 68.753122 ms | 2.48x |
| 4096 | nprobe=8 | 24.084181 ms | 71.541003 ms | 2.97x |
| 4096 | nprobe=16 | 49.079418 ms | 138.955281 ms | 2.83x |
| 4096 | nprobe=32 | 97.645540 ms | 274.268812 ms | 2.81x |

IVF and HNSW batch-on logs include AVX2 and scalar-tail attribution. Their batch-off comparison rows do not expose equivalent scalar direct counters through the same counter path, so the durable comparison for those surfaces in this packet is end-to-end latency plus recall parity.

Notable batch-on attribution:

| Surface | Rows | Knob | AVX2 candidates | Scalar-tail candidates |
| --- | ---: | --- | ---: | ---: |
| IVF | 512 | nprobe=8 | 37,928 | 472 |
| IVF | 512 | nprobe=16 | 76,800 | 0 |
| IVF | 4096 | nprobe=8 | 76,712 | 626 |
| IVF | 4096 | nprobe=16 | 153,432 | 556 |
| IVF | 4096 | nprobe=32 | 307,056 | 547 |
| HNSW | 512 | ef_search=32 | 5,608 | 948 |
| HNSW | 512 | ef_search=64 | 5,624 | 948 |
| HNSW | 4096 | ef_search=32 | 25,424 | 6,235 |
| HNSW | 4096 | ef_search=64 | 35,808 | 8,455 |

## Interpretation

The local closeout matrix is sufficient to support the non-AWS Task 97 state:

- Behavioral parity holds for all measured IVF, SPIRE, and HNSW cells.
- Direct SPIRE QJL32 scoring clears the 2x target on local AVX2.
- IVF end-to-end latency improves across every measured knob.
- HNSW end-to-end latency improves on 3 of 4 measured cells; the small 512/ef64 fixture regresses and should not be used as a win claim.
- SPIRE end-to-end latency is effectively flat at high 4096 nprobe despite direct scoring being faster, so the remaining optimization question is outside this local closeout packet and belongs with the later AWS/bench tasks.
