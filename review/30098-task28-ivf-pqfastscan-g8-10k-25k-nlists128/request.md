# Task 28 IVF PQ-FastScan g8 10k/25k nlists=128

Packet 30097 showed that PQ-FastScan g8 is faster and smaller than
TurboQuant at matched width750 on 10k/25k, but still trails TurboQuant on
recall, especially recall@100. This packet tests whether increasing PQ g8 from
`nlists=64` to `nlists=128` recovers that smaller-corpus recall gap.

## Fixture

Two new isolated surfaces were built from the existing PQ g8 10k and 25k
fixtures:

- `task28_ivf_pqg10k_g8_n128`
- `task28_ivf_pqg25k_g8_n128`

Both use:

- `storage_format = 'pq_fastscan'`
- `pq_group_size = 8`
- `nlists = 128`
- `training_sample_rows = 2000`
- `rerank = 'heap_f32'`
- `rerank_width = 750`

## Build/Size

| corpus | build time | index size |
|---|---:|---:|
| 10k | 40985.646 ms | 2960 kB |
| 25k | 69914.068 ms | 5688 kB |

## Recall

| corpus | nprobe | recall@10 | recall@100 | mean q-time @10 | mean q-time @100 |
|---|---:|---:|---:|---:|---:|
| 10k | 48 | 0.9600 | 0.9082 | 76.56 ms | 118.43 ms |
| 10k | 64 | 0.9770 | 0.9229 | 83.10 ms | 124.27 ms |
| 10k | 96 | 0.9900 | 0.9370 | 95.62 ms | 136.16 ms |
| 25k | 48 | 0.9720 | 0.8921 | 111.86 ms | 151.96 ms |
| 25k | 64 | 0.9810 | 0.9139 | 130.19 ms | 169.53 ms |
| 25k | 96 | 0.9910 | 0.9285 | 156.86 ms | 191.17 ms |

Latency at the best measured n128 quality point:

| corpus | nprobe | count | mean | p50 | p95 | p99 |
|---|---:|---:|---:|---:|---:|---:|
| 10k | 96 | 100 | 95.3 ms | 92.9 ms | 108.5 ms | 116.9 ms |
| 25k | 96 | 100 | 157.8 ms | 154.1 ms | 188.4 ms | 203.9 ms |

## Comparison To nlists=64

Packet 30097 measured PQ g8 `nlists=64`, width750:

| corpus | nlists | nprobe | recall@10 | recall@100 | p50 |
|---|---:|---:|---:|---:|---:|
| 10k | 64 | 48 | 0.9910 | 0.9360 | 85.4 ms |
| 10k | 128 | 96 | 0.9900 | 0.9370 | 92.9 ms |
| 25k | 64 | 48 | 0.9940 | 0.9256 | 145.7 ms |
| 25k | 128 | 96 | 0.9910 | 0.9285 | 154.1 ms |

`nlists=128` does not materially improve recall@100. It slightly improves
the 10k/25k recall@100 value only at nprobe 96, but it gives up recall@10 and
adds latency versus the n64/p48 profile.

It also remains well below packet 30097's TurboQuant width750 recall@100:

- 10k TurboQuant width750 p48 recall@100: `0.9966`
- 25k TurboQuant width750 p48 recall@100: `0.9929`

## Recommendation

Do not carry PQ-FastScan g8 `nlists=128` forward as the smaller-corpus
recall-recovery path. Keep the 10k/25k interpretation from packet 30097:

- PQ g8 is faster and smaller at matched width.
- TurboQuant remains the stronger smaller-corpus recall profile.
- The 100k PQ g8 win should be treated as scale/shape-specific until another
  PQ recall lever closes the 10k/25k recall@100 gap.

The next PQ recall-recovery attempt should not spend more time on `nlists`.
Better candidates are a wider rerank frontier, more training sample rows, or a
different PQ training/profile choice.

## Artifacts

See `artifacts/manifest.md`.
