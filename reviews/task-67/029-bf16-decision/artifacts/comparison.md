# Task 67 bf16 Decision Comparison

- fixture: `ec_real_10k`, `ec_ivf`, `storage_format=rabitq`, `quant_bits=4`, `rerank=heap_f32`, `rerank_width=100`
- host lane: AWS Intel `10k-intel`, `ECAZ_SIMD=auto`, PG18
- queries / iterations: 200 / 200
- off run: `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-bf16-off/20260530T145755Z/`
- on run: `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-bf16-on/20260530T153009Z/`

## Latency

| nprobe | bf16 off p50 | bf16 on p50 | on/off p50 | bf16 off mean | bf16 on mean | on/off mean |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 16 | 2.02 ms | 2.25 ms | 1.11x slower | 2.08 ms | 2.31 ms | 1.11x slower |
| 32 | 3.32 ms | 3.58 ms | 1.08x slower | 3.34 ms | 3.62 ms | 1.08x slower |
| 64 | 5.52 ms | 6.45 ms | 1.17x slower | 5.56 ms | 6.45 ms | 1.16x slower |

## Recall

| nprobe | bf16 off recall@k | bf16 on recall@k | bf16 off mean q-time | bf16 on mean q-time |
| ---: | ---: | ---: | ---: | ---: |
| 16 | 0.9985 | 0.9985 | 2.61 ms | 2.76 ms |
| 32 | 1.0000 | 1.0000 | 3.48 ms | 3.98 ms |
| 64 | 1.0000 | 1.0000 | 5.96 ms | 6.78 ms |

## Decision

Do not enable `rabitq-bf16` by default for this Task 67 SQL lane. It preserves recall but regresses p50 and mean latency at every tested nprobe value.
