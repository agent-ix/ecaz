# Task 28 IVF PQ-FastScan g8 100k nlists=128

This packet tests whether increasing IVF list count improves the current
100k PQ-FastScan g8 profile. Packet 30090 used `nlists=64`; this packet
uses the same 100k fixture with `nlists=128`.

Profile:

- `storage_format = 'pq_fastscan'`
- `pq_group_size = 8`
- `training_sample_rows = 2000`
- `rerank = 'heap_f32'`
- `rerank_width = 750`

## Result

Recall:

| nlists | nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|---:|
| 128 | 32 | 0.9710 | 0.9984 | 186.65 ms |
| 128 | 48 | 0.9920 | 0.9997 | 248.20 ms |
| 128 | 64 | 0.9940 | 0.9997 | 310.05 ms |
| 128 | 96 | 0.9980 | 0.9999 | 440.95 ms |

Latency:

| nlists | nprobe | count | mean | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|---:|
| 128 | 48 | 100 | 253.3 ms | 251.5 ms | 279.8 ms | 290.1 ms |
| 128 | 64 | 100 | 314.3 ms | 311.9 ms | 352.8 ms | 372.4 ms |

Build/index:

- Index build: `216414.112 ms`
- Index size: `19 MB`

## Comparison To nlists=64

Packet 30090 measured the same PQ-FastScan g8 profile at `nlists=64`:

| nlists | nprobe | recall@10 | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|
| 64 | 32 | 0.9930 | 279.5 ms | 312.5 ms | 323.1 ms |
| 64 | 48 | 1.0000 | 407.6 ms | 439.6 ms | 496.1 ms |
| 128 | 48 | 0.9920 | 251.5 ms | 279.8 ms | 290.1 ms |
| 128 | 64 | 0.9940 | 311.9 ms | 352.8 ms | 372.4 ms |

`nlists=128` is a better latency/recall region for this 100k surface:

- `n128/p48` nearly matches `n64/p32` recall (`0.9920` vs `0.9930`) and
  improves p50 (`251.5 ms` vs `279.5 ms`).
- `n128/p64` slightly improves recall over `n64/p32` (`0.9940` vs
  `0.9930`) but is slower (`311.9 ms` p50).
- `n128/p96` approaches the `n64/p48` full-recall point but is slower than
  the current low-latency target and does not need a latency follow-up yet.

## Recommendation

Carry `pq_group_size=8`, `rerank_width=750`, `nlists=128`, `nprobe=48` as
the current 100k low-latency high-recall PQ-FastScan point.

The next tuning slice should test whether rerank width can be narrowed at
`nlists=128`:

- `rerank_width in {500, 625, 750}`
- `nprobe in {48, 64}`

If `n128/p48` holds recall with a narrower rerank frontier, it should become
the candidate IVF default profile for this local-lane stage.

## Artifacts

See `artifacts/manifest.md`.
