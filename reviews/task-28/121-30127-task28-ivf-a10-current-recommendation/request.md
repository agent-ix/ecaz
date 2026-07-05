# Task 28 IVF A10 Current Quantizer Recommendation

## Scope

This packet consolidates the current A10 quantizer evidence after:

- A7 score-bound pruning landed.
- PQ-FastScan group size 8 became the leading PQ shape.
- A9's current-head 100k IVF selected point was refreshed in packet 30126.

This is a synthesis packet over packet-local artifacts. It does not run a new measurement.

## Source Packets

- 30084: first 10k TurboQuant / PQ-FastScan / RaBitQ head-to-head smoke.
- 30091: 100k PQ-FastScan g8 versus TurboQuant comparison at nlists=64.
- 30094: 100k PQ-FastScan g8 nlists=128 nprobe middle sweep.
- 30097: 10k/25k TurboQuant versus PQ-FastScan g8 refresh.
- 30117: post-A7 PQ-FastScan g8 recall@100 confirmation.
- 30126: current-head 100k PQ-FastScan g8 selected-point refresh.

## Current Evidence

### 10k and 25k

At matched `nprobe=48`, `rerank_width=750`:

| corpus | profile | recall@10 | recall@100 | p50 | p95 | p99 | index size |
|---|---|---:|---:|---:|---:|---:|---:|
| 10k | TurboQuant | 1.0000 | 0.9966 | 118.8 ms | 147.2 ms | 160.8 ms | 9416 kB |
| 10k | PQ-FastScan g8 | 0.9910 | 0.9360 | 85.4 ms | 104.4 ms | 117.0 ms | 2448 kB |
| 25k | TurboQuant | 0.9990 | 0.9929 | 231.5 ms | 271.3 ms | 284.6 ms | 22 MB |
| 25k | PQ-FastScan g8 | 0.9940 | 0.9256 | 145.7 ms | 171.9 ms | 194.1 ms | 5176 kB |

Interpretation:

- PQ-FastScan g8 is clearly faster and smaller on 10k/25k.
- TurboQuant still wins recall@100 materially on 10k/25k.
- A global default switch to PQ-FastScan g8 would trade away smaller-corpus recall@100.

### 100k

The current selected 100k IVF point is PQ-FastScan g8 with `nlists=128`, `nprobe=48`, and `rerank_width=500`.

| metric | value |
|---|---:|
| build time | 216788.531 ms |
| index size | 19,791,872 bytes |
| recall@10 | 0.9920 |
| NDCG@10 | 0.9997 |
| recall@100 | 0.9552 |
| NDCG@100 | 0.9983 |
| latency p50 | 169.3 ms |
| latency p95 | 191.2 ms |
| latency p99 | 194.4 ms |
| memory HWM | 153816 kB |

Packet 30091's same-fixture 100k nlists=64 comparison showed PQ-FastScan g8 tied TurboQuant recall at the measured nprobe points while being much faster and smaller:

| profile | nprobe | recall@10 | p50 | p95 | p99 | index size |
|---|---:|---:|---:|---:|---:|---:|
| TurboQuant | 32 | 0.9930 | 464.8 ms | 538.0 ms | 556.8 ms | 87 MB |
| PQ-FastScan g8 | 32 | 0.9930 | 279.5 ms | 312.5 ms | 323.1 ms | 18 MB |
| TurboQuant | 48 | 1.0000 | 705.7 ms | 760.6 ms | 782.7 ms | 87 MB |
| PQ-FastScan g8 | 48 | 1.0000 | 407.6 ms | 439.6 ms | 496.1 ms | 18 MB |

Interpretation:

- PQ-FastScan g8 is the best measured 100k IVF lane.
- TurboQuant should not be treated as the presumed best high-dimensional IVF profile at 100k.

### RaBitQ

RaBitQ is wired and selectable, but packet 30084 showed the current IVF RaBitQ scan path is not latency-competitive:

- 10k, nprobe=32: recall@10 `0.9800`, p50 `1276.7 ms` in a narrowed 10-iteration latency smoke.
- 10k, nprobe=48 recall matched TurboQuant at `1.0000`, but mean query time was `1846.27 ms`.

Interpretation:

- RaBitQ correctness/selectability is useful substrate work.
- RaBitQ is not a current A10 default candidate until its IVF scan scoring path is optimized.

## Recommendation

Do not change `quantizer = 'auto'` in Task 28.

The measured recommendation is:

- For 100k high-dimensional local IVF, recommend explicit `quantizer = 'pq_fastscan', pq_group_size = 8`, with the current selected point `nlists=128`, `nprobe=48`, `rerank_width=500`.
- For smaller 10k/25k workloads where recall@100 matters more than index size and latency, TurboQuant remains the safer profile.
- RaBitQ should remain available but documented as not latency-competitive in the current IVF integration.

If `auto` changes later, it should be a separate task and likely dimension/corpus-size aware. A global switch to PQ-FastScan g8 is not justified by the 10k/25k recall@100 data.

## Remaining A10 Gaps

This packet consolidates the current recommendation, but the strict A10 wording still has gaps:

- The 10k/25k refresh does not include scan memory HWM.
- Cold-cache measurements are not present; current packet cache state is warm local development.
- RaBitQ has not been remeasured broadly after the PQ-FastScan g8 work because prior latency was far outside the practical tuning band.
- The 100k comparison has current selected-point PQ-FastScan evidence, but not a fresh current-head TurboQuant rebuild or RaBitQ 100k run.

These gaps should be treated as the remaining A10 closure checklist if the reviewer requires literal completion of every field.

## Artifacts

See `artifacts/manifest.md`.
