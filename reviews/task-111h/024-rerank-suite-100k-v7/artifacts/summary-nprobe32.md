# Task 111h 100k v7 Rerank Summary

Head SHA: `bc95e5f761c96b64f4a9bf594e074888981af8fe`
Packet: `reviews/task-111h/024-rerank-suite-100k-v7/`
Fixture: staged real 100k corpus, 200 queries, `k=10`, `nlists=256`, `coarse_bits=1`, `nprobe` sweep `8,16,32,64,128,200`.
Surface: isolated one-index-per-table prefixes in scratch database `task111h_rerank_100k_v7`.

## nprobe=32

| Placement / format | Width | recall@10 | p50 | p95 | p99 | Index size |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| source f32 | 32 | 0.9285 | 4.50 ms | 5.02 ms | 5.36 ms | 24.6 MiB |
| source f32 | 64 | 0.9350 | 5.57 ms | 6.49 ms | 7.14 ms | 24.6 MiB |
| source f32 | 128 | 0.9350 | 7.48 ms | 8.03 ms | 8.45 ms | 24.6 MiB |
| source f32 | 256 | 0.9350 | 13.7 ms | 15.7 ms | 16.1 ms | 24.6 MiB |
| index f16 | 32 | 0.9280 | 4.36 ms | 5.55 ms | 7.90 ms | 342.0 MiB |
| index f16 | 64 | 0.9345 | 7.60 ms | 10.4 ms | 14.9 ms | 330.1 MiB |
| index f16 | 128 | 0.9345 | 9.23 ms | 13.2 ms | 18.0 ms | 324.3 MiB |
| index f16 | 256 | 0.9345 | 14.3 ms | 27.2 ms | 49.3 ms | 323.7 MiB |
| index RaBitQ4 | 32 | 0.8910 | 3.81 ms | 4.33 ms | 4.67 ms | 121.8 MiB |
| index RaBitQ4 | 64 | 0.8945 | 5.12 ms | 6.84 ms | 8.34 ms | 110.2 MiB |
| index RaBitQ4 | 128 | 0.8940 | 5.05 ms | 6.35 ms | 7.74 ms | 104.5 MiB |
| index RaBitQ4 | 256 | 0.8940 | 6.67 ms | 8.86 ms | 12.4 ms | 104.0 MiB |
| index RaBitQ8 | 32 | 0.9010 | 3.80 ms | 4.56 ms | 5.15 ms | 195.4 MiB |
| index RaBitQ8 | 64 | 0.9060 | 4.81 ms | 6.28 ms | 7.18 ms | 183.6 MiB |
| index RaBitQ8 | 128 | 0.9055 | 5.87 ms | 7.59 ms | 9.52 ms | 177.9 MiB |
| index RaBitQ8 | 256 | 0.9055 | 8.62 ms | 14.1 ms | 16.9 ms | 177.4 MiB |
| index TurboQuant | 32 | 0.9040 | 3.91 ms | 4.52 ms | 4.88 ms | 121.8 MiB |
| index TurboQuant | 64 | 0.9075 | 5.03 ms | 6.46 ms | 7.67 ms | 110.1 MiB |
| index TurboQuant | 128 | 0.9070 | 5.17 ms | 6.08 ms | 6.94 ms | 104.4 MiB |
| index TurboQuant | 256 | 0.9070 | 6.74 ms | 8.28 ms | 9.64 ms | 101.8 MiB |

## nprobe=200 recall ceiling

| Placement / format | Width | recall@10 |
| --- | ---: | ---: |
| source f32 | 32 | 0.9875 |
| source f32 | 64 | 0.9985 |
| source f32 | 128 | 0.9990 |
| source f32 | 256 | 0.9990 |
| index f16 | 32 | 0.9870 |
| index f16 | 64 | 0.9975 |
| index f16 | 128 | 0.9980 |
| index f16 | 256 | 0.9980 |
| index RaBitQ4 | 32 | 0.9330 |
| index RaBitQ4 | 64 | 0.9380 |
| index RaBitQ4 | 128 | 0.9380 |
| index RaBitQ4 | 256 | 0.9380 |
| index RaBitQ8 | 32 | 0.9455 |
| index RaBitQ8 | 64 | 0.9525 |
| index RaBitQ8 | 128 | 0.9525 |
| index RaBitQ8 | 256 | 0.9525 |
| index TurboQuant | 32 | 0.9525 |
| index TurboQuant | 64 | 0.9565 |
| index TurboQuant | 128 | 0.9565 |
| index TurboQuant | 256 | 0.9565 |

## Notes

- The 100k v7 run does not reproduce a 150 ms f16 latency. The worst f16 nprobe=32 result in this packet is width 256, p99 49.3 ms, max 53.2 ms.
- Source f32 is still the exact baseline here. It uses the existing heap source vector and adds no compact rerank payload index storage; the measured ec_ivf index stays 24.6 MiB across widths.
- Persisted f16 keeps source-like recall but bloats the ec_ivf index to roughly 324-342 MiB in this layout and has wider tails at larger widths.
- RaBitQ8 and TurboQuant improve over RaBitQ4 recall, but all three compact quantized rerank formats remain materially below source f32/f16 recall in this 100k sweep.
- TurboQuant is closest among compact quantized formats in this packet: RaBitQ4-like index size with slightly better recall than RaBitQ8 at the reported high-nprobe ceiling.
