# Task 28 IVF PQ-FastScan g8 Rerank Narrowing

This packet narrows the 25k `pq_group_size=8` high-recall rerank frontier
from packet 30088. Packet 30088 showed that `rerank_width=1000` reached
recall@10 `0.9960` at nprobe 48. This packet checks whether smaller
frontiers keep most of that quality.

## Result

| rerank_width | nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|---:|
| 500 | 32 | 0.9700 | 0.9985 | 102.55 ms |
| 500 | 48 | 0.9840 | 0.9998 | 130.12 ms |
| 750 | 32 | 0.9790 | 0.9987 | 115.97 ms |
| 750 | 48 | 0.9940 | 1.0000 | 140.85 ms |

Latency for the better narrowed profile:

| rerank_width | nprobe | count | mean | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|---:|
| 750 | 32 | 100 | 115.6 ms | 114.1 ms | 127.9 ms | 133.1 ms |
| 750 | 48 | 100 | 141.2 ms | 140.0 ms | 146.3 ms | 151.3 ms |

For comparison:

- Packet 30088 width 1000, nprobe 48: recall@10 `0.9960`, p50 `153.2 ms`,
  p95 `164.1 ms`, p99 `175.8 ms`.
- Packet 30052 TurboQuant nprobe 48: recall@10 `0.9990`, p50 `351.4 ms`,
  p95 `383.2 ms`, p99 `584.5 ms`.

## Interpretation

`pq_group_size=8`, `rerank_width=750`, `nprobe=48` is the better current
25k operating point than width 1000. It gives up only 0.2 recall points
versus width 1000 (`0.9940` vs `0.9960`) and improves p50/p95/p99.

Width 500 is not enough for the high-recall target at nprobe 48; it lands
at recall@10 `0.9840`.

## Recommendation

Carry `pq_group_size=8`, `rerank_width=750`, `nprobe=48` as the current
25k high-recall PQ-FastScan point. The next slice should either:

1. Try `rerank_width=875` if we need to close the final recall gap to the
   width-1000 point, or
2. Move this profile to a 100k slice and see whether the speedup over
   TurboQuant grows with corpus size.

The `task28_ivf_pqg25k_g8_idx` surface was restored to `rerank_width=25`
after the measurements.

## Artifacts

See `artifacts/manifest.md`.
