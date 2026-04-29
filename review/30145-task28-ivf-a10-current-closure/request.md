# Review Request: Task 28 IVF A10 Current Closure

## Scope

This packet consolidates the current A10 quantizer recommendation after the
post-A7 smaller-corpus refreshes and the follow-up memory/RaBitQ fill packets.

It introduces no new measurements. Source packets:

- 30097: 10k/25k TurboQuant vs PQ-FastScan g8 matched-width recall/latency/size
- 30126: current-head 100k PQ-FastScan g8 selected point
- 30137: post-A7 10k/25k PQ-FastScan g8 recall/latency/HWM
- 30143: 10k/25k TurboQuant matched-width latency/HWM
- 30144: 10k/25k RaBitQ bounded recall/latency/HWM and 25k build

## Current A10 Matrix

10k and 25k matched shape: `nlists=64`, `nprobe=48`, `rerank=heap_f32`,
`rerank_width=750`.

| corpus | profile | recall@10 | recall@100 | p50 | p95 | p99 | HWM | index size |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| 10k | TurboQuant | 1.0000 | 0.9966 | 130.6 ms | 231.6 ms | 267.9 ms | 109600 kB | 9,641,984 B |
| 10k | PQ-FastScan g8 | 0.9910 | 0.9360 | 77.3 ms | 80.4 ms | 82.2 ms | 137244 kB | 2,506,752 B |
| 10k | RaBitQ | 1.0000 | 0.9930 | 1947.8 ms | 2096.9 ms | 2128.3 ms | 69980 kB | 9,641,984 B |
| 25k | TurboQuant | 0.9990 | 0.9929 | 284.5 ms | 402.4 ms | 441.5 ms | 155540 kB | 23,289,856 B |
| 25k | PQ-FastScan g8 | 0.9940 | 0.9256 | 116.8 ms | 123.7 ms | 125.7 ms | 156112 kB | 5,300,224 B |
| 25k | RaBitQ | 1.0000 | 0.9915 | 4973.0 ms | 5257.7 ms | 5327.9 ms | 145012 kB | 23,519,232 B |

RaBitQ recall and latency rows use bounded samples (`queries-limit=20`,
`iterations=10`) because the measured per-query latency is already
multi-second. They are sufficient to classify the current RaBitQ IVF scan path
as latency-uncompetitive.

100k selected point:

| corpus | profile | recall@10 | recall@100 | p50 | p95 | p99 | HWM | index size |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| 100k | PQ-FastScan g8 n128/w500 | 0.9920 | 0.9552 | 169.3 ms | 191.2 ms | 194.4 ms | 153816 kB | 19,791,872 B |

Packet 30091 remains the best direct 100k TurboQuant vs PQ-FastScan comparison:
at `nlists=64`, PQ-FastScan g8 tied TurboQuant recall at the measured nprobe
points while being much faster and much smaller.

## Recommendation

Do not change `quantizer = 'auto'` in Task 28.

Measured recommendation:

- For 100k high-dimensional local IVF, recommend explicit
  `quantizer = 'pq_fastscan', pq_group_size = 8`.
- For smaller 10k/25k workloads where recall@100 matters more than index size
  and latency, TurboQuant remains the safer default profile.
- RaBitQ should remain selectable, but it is not a current IVF default
  candidate until its scan scoring path is optimized.

The reason to keep `auto` unchanged is not historical inertia. It is the
measured smaller-corpus recall@100 gap: TurboQuant keeps about `0.993-0.997`
recall@100 at the matched width, while PQ-FastScan g8 is about `0.926-0.936`.
PQ-FastScan g8 wins strongly on speed and size, but the default should not
trade away that recall@100 behavior globally in this task.

## Remaining Caveats

- Cache state is warm local development for these rows; no explicit OS or
  PostgreSQL buffer-cache drop was performed.
- RaBitQ rows are intentionally bounded because full 100-query runs would spend
  several minutes per row on a profile that is already outside the latency
  band.
- The 100k TurboQuant and RaBitQ rows are not freshly rebuilt at current head;
  the current practical 100k recommendation is based on the selected
  PQ-FastScan point plus packet 30091's direct comparison.

## Validation

- Synthesis only; packet-local measurements are cited above.

## Artifacts

- `artifacts/manifest.md`
