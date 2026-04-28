# Task 28 IVF PQ-FastScan g8 100k nlists=128 Rerank Narrowing

This packet narrows the 100k `pq_group_size=8`, `nlists=128` profile from
packet 30092. Packet 30092 used `rerank_width=750`; this packet checks
whether smaller rerank frontiers preserve the same recall at the current
high-recall probes.

Profile:

- `storage_format = 'pq_fastscan'`
- `pq_group_size = 8`
- `training_sample_rows = 2000`
- `nlists = 128`
- `rerank = 'heap_f32'`
- `nprobe in {48, 64}`

## Result

Recall:

| rerank_width | nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|---:|
| 500 | 48 | 0.9920 | 0.9997 | 238.14 ms |
| 500 | 64 | 0.9940 | 0.9997 | 295.79 ms |
| 625 | 48 | 0.9920 | 0.9997 | 242.22 ms |
| 625 | 64 | 0.9940 | 0.9997 | 299.33 ms |

Latency for the narrower frontier:

| rerank_width | nprobe | count | mean | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|---:|
| 500 | 48 | 100 | 237.7 ms | 236.0 ms | 273.8 ms | 284.9 ms |
| 500 | 64 | 100 | 295.9 ms | 295.9 ms | 323.7 ms | 330.3 ms |

For comparison, packet 30092 measured `rerank_width=750` on the same
surface:

| rerank_width | nprobe | recall@10 | mean | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|---:|
| 750 | 48 | 0.9920 | 253.3 ms | 251.5 ms | 279.8 ms | 290.1 ms |
| 750 | 64 | 0.9940 | 314.3 ms | 311.9 ms | 352.8 ms | 372.4 ms |

## Interpretation

`rerank_width=500` holds the same measured recall as width 750 at both
tested probes and improves latency. The improvement is modest but consistent:

- nprobe 48: p50 improves from `251.5 ms` to `236.0 ms` at unchanged recall
  `0.9920`.
- nprobe 64: p50 improves from `311.9 ms` to `295.9 ms` at unchanged recall
  `0.9940`.

`rerank_width=625` does not improve recall over width 500 in this fixture, so
it is not a better operating point.

## Recommendation

Carry `pq_group_size=8`, `nlists=128`, `rerank_width=500`, `nprobe=48` as
the current 100k low-latency high-recall PQ-FastScan point. Use nprobe 64
when the local target needs the extra 0.2 recall points.

The next slice should probe between the two measured points:

- `nprobe in {40, 48, 56, 64}`
- `rerank_width = 500`

That should show whether there is a better middle point than nprobe 48 or 64
before trying a larger `nlists=256` surface.

The `task28_ivf_pqg100k_g8_n128_idx` surface was restored to
`rerank_width=750` after the measurements so it matches packet 30092's
baseline state.

## Artifacts

See `artifacts/manifest.md`.
