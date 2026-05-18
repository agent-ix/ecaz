# Task 28 IVF PQ-FastScan g8 100k nprobe Middle Sweep

Packet 30093 narrowed the 100k `pq_group_size=8`, `nlists=128` profile to
`rerank_width=500` and recommended probing between nprobe 48 and 64. This
packet measures that middle region on the same isolated 100k surface.

Profile:

- `storage_format = 'pq_fastscan'`
- `pq_group_size = 8`
- `training_sample_rows = 2000`
- `nlists = 128`
- `rerank = 'heap_f32'`
- `rerank_width = 500`
- `nprobe in {40, 48, 56, 64}`

## Result

Recall:

| nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|
| 40 | 0.9810 | 0.9993 | 204.82 ms |
| 48 | 0.9920 | 0.9997 | 234.61 ms |
| 56 | 0.9930 | 0.9997 | 265.49 ms |
| 64 | 0.9940 | 0.9997 | 295.03 ms |

Latency:

| nprobe | count | mean | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|
| 40 | 100 | 207.2 ms | 205.8 ms | 233.8 ms | 255.8 ms |
| 48 | 100 | 237.0 ms | 238.5 ms | 262.1 ms | 274.7 ms |
| 56 | 100 | 269.9 ms | 266.3 ms | 309.5 ms | 356.0 ms |
| 64 | 100 | 296.1 ms | 295.8 ms | 322.5 ms | 332.8 ms |

## Interpretation

`nprobe=48` remains the better low-latency high-recall point. It reaches
recall@10 `0.9920` with p50 `238.5 ms`.

`nprobe=56` is a useful optional point when the extra recall matters:
recall improves to `0.9930`, but p50 rises to `266.3 ms` and this run had a
noisier p99 (`356.0 ms`) than nprobe 64. That p99 should be rechecked before
using nprobe 56 for a latency-sensitive default.

`nprobe=40` is faster, but recall drops to `0.9810`, so it is below the
current high-recall target. `nprobe=64` remains the measured quality step up
to recall@10 `0.9940`, but it costs p50 `295.8 ms`.

## Recommendation

Keep `pq_group_size=8`, `nlists=128`, `rerank_width=500`, `nprobe=48` as the
current 100k local-lane candidate. Treat nprobe 64 as the quality-biased
toggle, and keep nprobe 56 as a possible middle point only after another
latency repeat validates the tail behavior.

The next implementation-facing slice should turn this into planner/default
guidance for IVF `auto` selection, while keeping the measured profile explicit
in review packets.

## Artifacts

See `artifacts/manifest.md`.
