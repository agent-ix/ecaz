# Task 28 IVF PQ-FastScan g8 25k Smoke

This packet extends packet 30087 from the 10k slice to the existing 25k
DBPedia IVF slice, using the current best PQ-FastScan quality profile:

- `storage_format = 'pq_fastscan'`
- `pq_group_size = 8`
- `nlists = 64`
- `training_sample_rows = 2000`
- `rerank = 'heap_f32'`
- `nprobe in {32, 48}`

## Result

The 25k result confirms that `pq_group_size=8` plus a wider exact rerank
frontier is a real high-recall PQ-FastScan profile.

| rerank_width | nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|---:|
| 25 | 32 | 0.7180 | 0.9827 | 83.96 ms |
| 25 | 48 | 0.7220 | 0.9834 | 113.90 ms |
| 250 | 32 | 0.9530 | 0.9977 | 91.99 ms |
| 250 | 48 | 0.9650 | 0.9989 | 119.30 ms |
| 1000 | 32 | 0.9810 | 0.9988 | 126.46 ms |
| 1000 | 48 | 0.9960 | 1.0000 | 150.85 ms |

Latency packet for the high-recall `rerank_width=1000` setting:

| nprobe | count | mean | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|
| 32 | 100 | 124.9 ms | 124.4 ms | 130.2 ms | 133.6 ms |
| 48 | 100 | 154.6 ms | 153.2 ms | 164.1 ms | 175.8 ms |

The index size was `5176 kB`; build time was `47798.422 ms`.

## Comparison

Packet 30052 measured the TurboQuant `task28_ivf_postopt25k_n64w25`
surface on the same 25k fixture:

- nprobe 32: recall@10 `0.9840`, mean recall-harness q-time `248.14 ms`,
  latency p50 `246.2 ms`, p95 `261.0 ms`, p99 `264.5 ms`
- nprobe 48: recall@10 `0.9990`, mean recall-harness q-time `353.31 ms`,
  latency p50 `351.4 ms`, p95 `383.2 ms`, p99 `584.5 ms`

The PQ-FastScan g8 high-recall point is now meaningfully faster on 25k:

- `pq_group_size=8`, `rerank_width=1000`, `nprobe=32` gets similar recall
  to TurboQuant nprobe 32 (`0.9810` vs `0.9840`) at about half the p50.
- `pq_group_size=8`, `rerank_width=1000`, `nprobe=48` gets near-full recall
  (`0.9960`) with p50 `153.2 ms`, much lower than the TurboQuant nprobe 48
  p50 `351.4 ms`.

## Recommendation

Keep `pq_group_size=8` as the high-recall PQ-FastScan candidate for the
next slice. The next useful work is:

1. Re-run this profile on 100k if the local fixture is available.
2. Add a narrower sweep around `rerank_width` 500/750/1000 at 25k, because
   width 1000 is probably larger than needed for the 0.99 band.
3. Only after that, decide whether additional PQ quality mechanisms are
   needed before OPQ/LSQ-era work.

The `task28_ivf_pqg25k_g8_idx` surface was restored to `rerank_width=25`
after the measurements.

## Artifacts

See `artifacts/manifest.md`.
