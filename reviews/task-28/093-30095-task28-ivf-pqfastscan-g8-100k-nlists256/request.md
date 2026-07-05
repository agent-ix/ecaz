# Task 28 IVF PQ-FastScan g8 100k nlists=256

This packet tests whether increasing `nlists` beyond the packet 30094
frontier improves the 100k PQ-FastScan g8 operating point.

Profile:

- `storage_format = 'pq_fastscan'`
- `pq_group_size = 8`
- `training_sample_rows = 2000`
- `nlists = 256`
- `rerank = 'heap_f32'`
- `rerank_width = 500`

## Result

Build/index:

| nlists | build | index size |
|---:|---:|---:|
| 256 | 338805.986 ms | 20 MB |

Recall:

| nlists | nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|---:|
| 256 | 64 | 0.9800 | 0.9988 | 205.89 ms |
| 256 | 96 | 0.9940 | 0.9998 | 263.65 ms |
| 256 | 128 | 0.9950 | 0.9998 | 332.51 ms |

Latency:

| nlists | nprobe | count | mean | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|---:|
| 256 | 96 | 100 | 272.3 ms | 270.1 ms | 305.7 ms | 342.2 ms |
| 256 | 128 | 100 | 333.5 ms | 330.0 ms | 370.6 ms | 416.6 ms |

## Comparison To nlists=128

Packet 30094 measured the current n128 width-500 profile:

| nlists | nprobe | recall@10 | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|
| 128 | 48 | 0.9920 | 238.5 ms | 262.1 ms | 274.7 ms |
| 128 | 56 | 0.9930 | 266.3 ms | 309.5 ms | 356.0 ms |
| 128 | 64 | 0.9940 | 295.8 ms | 322.5 ms | 332.8 ms |
| 256 | 96 | 0.9940 | 270.1 ms | 305.7 ms | 342.2 ms |

At the same measured recall as n128/p64 (`0.9940`), n256/p96 improves
p50 and p95, but p99 is slightly worse in this run:

- p50: `295.8 ms` -> `270.1 ms`
- p95: `322.5 ms` -> `305.7 ms`
- p99: `332.8 ms` -> `342.2 ms`

The tradeoff is build cost. n256 built in `338805.986 ms` versus packet
30092's n128 build at `216414.112 ms`, with index size only increasing from
19 MB to 20 MB.

## Recommendation

Keep n128/p48 as the low-latency high-recall default candidate at recall
`0.9920`. Add n256/p96 as the current quality-biased candidate when recall
`0.9940` is worth the longer build and roughly 32 ms higher p50 than n128/p48.

Do not carry n256/p128 forward unless a target needs recall above `0.9940`;
it adds latency and only improved recall by `0.0010` in this fixture.

## Artifacts

See `artifacts/manifest.md`.
