# Task 28 IVF 990k Balanced Recall@100

## Scope

This packet follows packet 30132's lower-`nprobe` latency/recall@10 frontier with a narrow recall@100 check for the two best 990k candidates.

Fixture:

- prefix: `task28_ivf_pqg990k_g8_n128`
- quantizer: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- `rerank=heap_f32`
- `rerank_width=500`
- 100-query exact recall cap

## Result

| nprobe | recall@100 | NDCG@100 | mean q-time |
|---:|---:|---:|---:|
| 32 | 0.9360 | 0.9968 | 781.56 ms |
| 40 | 0.9466 | 0.9975 | 928.39 ms |

For comparison, packet 30130 measured the original selected point:

| nprobe | recall@100 | NDCG@100 | mean q-time | latency p50 | latency p95 |
|---:|---:|---:|---:|---:|---:|
| 48 | 0.9509 | 0.9979 | 1088.61 ms | 1029.2 ms | 1169.1 ms |

Packet 30132 measured latency for the lower probes:

| nprobe | latency p50 | latency p95 | latency p99 |
|---:|---:|---:|---:|
| 32 | 740.1 ms | 833.8 ms | 876.6 ms |
| 40 | 884.2 ms | 994.8 ms | 1036.2 ms |

## Interpretation

`nprobe=40` is the better 990k balanced point from this slice. It keeps recall@100 within 0.0043 absolute of the original nprobe 48 point while cutting p50 latency by about 145 ms and p95 latency by about 174 ms.

`nprobe=32` is the lower-latency point, but the recall@100 drop is more visible: 0.9360 versus 0.9509 at nprobe 48.

Carry both points forward:

- balanced: `nprobe=40`, `rerank_width=500`
- lower-latency: `nprobe=32`, `rerank_width=500`

The next implementation slice should add scan score-volume counters so future packets can explain whether latency is dominated by postings scored, dedup pressure, or heap rerank width.

## Validation

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 32,40 --rerank-width 500 --force-index --log-output review/30133-task28-ivf-990k-balanced-recall100/artifacts/recall100_pqg8_990k_n128_w500_nprobe32_40.log`

## Artifacts

- `artifacts/recall100_pqg8_990k_n128_w500_nprobe32_40.log`
- `artifacts/manifest.md`
