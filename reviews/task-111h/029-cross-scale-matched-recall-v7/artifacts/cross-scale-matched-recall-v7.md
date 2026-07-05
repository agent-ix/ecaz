# Task 111h Cross-Scale Matched-Recall V7 Decision Slice

This derived analysis re-slices the committed post-v7 warm-cache local suites at
matched recall targets `0.95`, `0.97`, and `0.99`.

Selection rule: for each scale, target, and format, select the lowest-p50 row
that reaches the target recall. If no row reaches the target, report the
maximum-recall row as `NO_REACH`.

This analysis uses formal latency p50/p95/p99 rows, not recall command
`mean q-time`, and uses the packet-local `ec_ivf` index size from storage rows.

## Sources

| Scale | Source packet | Result source |
| --- | --- | --- |
| 10k | `reviews/task-111h/026-rerank-suite-10k-v7` | `artifacts/results.jsonl` |
| 50k | `reviews/task-111h/027-rerank-suite-50k-v7` | `artifacts/results.jsonl` |
| 100k | `reviews/task-111h/024-rerank-suite-100k-v7` | main `results.jsonl` plus RaBitQ4/RaBitQ8/TurboQuant continuation report JSONL files |
| 1M | `reviews/task-111h/028-rerank-suite-1m-v7-shared` | `artifacts/results.jsonl` |

The 10k/50k/100k suites use isolated one-index-per-table surfaces. The 1M v7
suite uses a shared-table surface with one active IVF index at a time because
the isolated 1M shape did not fit the local disk budget.

## Target Recall >= 0.95

| Scale | Variant | Status | Row | Recall@10 | p50 | p95 | p99 | IVF index |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| 10k | source/f32 | hit | w32 np8 | 0.9890 | 2.74 ms | 2.99 ms | 3.14 ms | 5.1 MiB |
| 10k | index/f16 | hit | w32 np8 | 0.9885 | 1.79 ms | 2.27 ms | 2.57 ms | 37.0 MiB |
| 10k | index/rabitq4 | hit | w32 np8 | 0.9700 | 1.55 ms | 1.76 ms | 1.94 ms | 14.7 MiB |
| 10k | index/rabitq8 | hit | w32 np8 | 0.9775 | 1.61 ms | 1.92 ms | 2.14 ms | 22.3 MiB |
| 10k | index/turboquant | hit | w32 np8 | 0.9735 | 1.74 ms | 2.18 ms | 2.72 ms | 14.7 MiB |
| 50k | source/f32 | hit | w32 np32 | 0.9520 | 3.49 ms | 3.81 ms | 4.34 ms | 13.8 MiB |
| 50k | index/f16 | hit | w32 np32 | 0.9520 | 3.63 ms | 4.43 ms | 5.32 ms | 172.5 MiB |
| 50k | index/rabitq4 | NO_REACH | w128 np200 | 0.9460 | 8.86 ms | 10.3 ms | 11.0 ms | 54.0 MiB |
| 50k | index/rabitq8 | hit | w64 np128 | 0.9520 | 6.21 ms | 7.44 ms | 7.96 ms | 93.4 MiB |
| 50k | index/turboquant | hit | w32 np128 | 0.9550 | 5.47 ms | 6.09 ms | 6.87 ms | 62.3 MiB |
| 100k | source/f32 | hit | w32 np64 | 0.9625 | 6.23 ms | 6.98 ms | 7.51 ms | 24.6 MiB |
| 100k | index/f16 | hit | w32 np64 | 0.9620 | 6.51 ms | 8.06 ms | 8.60 ms | 342.0 MiB |
| 100k | index/rabitq4 | NO_REACH | w64 np200 | 0.9380 | 15.3 ms | 18.8 ms | 21.0 ms | 110.2 MiB |
| 100k | index/rabitq8 | hit | w64 np200 | 0.9525 | 14.4 ms | 15.8 ms | 16.8 ms | 183.6 MiB |
| 100k | index/turboquant | hit | w128 np128 | 0.9530 | 11.6 ms | 14.5 ms | 18.1 ms | 104.4 MiB |
| 1M | source/f32 | hit | w64 np32 | 0.9570 | 12.2 ms | 14.0 ms | 14.9 ms | 226.8 MiB |
| 1M | index/f16 | hit | w64 np32 | 0.9570 | 13.6 ms | 16.1 ms | 16.8 ms | 3.2 GiB |
| 1M | index/rabitq4 | NO_REACH | w128 np200 | 0.9370 | 42.0 ms | 48.3 ms | 49.6 ms | 1014.4 MiB |
| 1M | index/rabitq8 | hit | w128 np200 | 0.9520 | 47.1 ms | 60.9 ms | 69.4 ms | 1.7 GiB |
| 1M | index/turboquant | hit | w128 np200 | 0.9510 | 42.4 ms | 48.9 ms | 50.4 ms | 1013.9 MiB |

## Target Recall >= 0.97

| Scale | Variant | Status | Row | Recall@10 | p50 | p95 | p99 | IVF index |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| 10k | source/f32 | hit | w32 np8 | 0.9890 | 2.74 ms | 2.99 ms | 3.14 ms | 5.1 MiB |
| 10k | index/f16 | hit | w32 np8 | 0.9885 | 1.79 ms | 2.27 ms | 2.57 ms | 37.0 MiB |
| 10k | index/rabitq4 | hit | w32 np8 | 0.9700 | 1.55 ms | 1.76 ms | 1.94 ms | 14.7 MiB |
| 10k | index/rabitq8 | hit | w32 np8 | 0.9775 | 1.61 ms | 1.92 ms | 2.14 ms | 22.3 MiB |
| 10k | index/turboquant | hit | w32 np8 | 0.9735 | 1.74 ms | 2.18 ms | 2.72 ms | 14.7 MiB |
| 50k | source/f32 | hit | w32 np64 | 0.9730 | 4.42 ms | 4.70 ms | 4.85 ms | 13.8 MiB |
| 50k | index/f16 | hit | w32 np64 | 0.9730 | 4.58 ms | 5.62 ms | 6.30 ms | 172.5 MiB |
| 50k | index/rabitq4 | NO_REACH | w128 np200 | 0.9460 | 8.86 ms | 10.3 ms | 11.0 ms | 54.0 MiB |
| 50k | index/rabitq8 | NO_REACH | w128 np200 | 0.9545 | 9.69 ms | 13.4 ms | 17.3 ms | 90.8 MiB |
| 50k | index/turboquant | NO_REACH | w128 np200 | 0.9600 | 8.53 ms | 9.57 ms | 11.0 ms | 53.9 MiB |
| 100k | source/f32 | hit | w64 np64 | 0.9720 | 7.65 ms | 8.61 ms | 10.0 ms | 24.6 MiB |
| 100k | index/f16 | hit | w64 np64 | 0.9710 | 8.75 ms | 11.0 ms | 13.4 ms | 330.1 MiB |
| 100k | index/rabitq4 | NO_REACH | w64 np200 | 0.9380 | 15.3 ms | 18.8 ms | 21.0 ms | 110.2 MiB |
| 100k | index/rabitq8 | NO_REACH | w64 np200 | 0.9525 | 14.4 ms | 15.8 ms | 16.8 ms | 183.6 MiB |
| 100k | index/turboquant | NO_REACH | w128 np200 | 0.9565 | 18.2 ms | 28.2 ms | 36.2 ms | 104.4 MiB |
| 1M | source/f32 | hit | w64 np64 | 0.9770 | 17.8 ms | 21.1 ms | 21.8 ms | 226.8 MiB |
| 1M | index/f16 | hit | w64 np64 | 0.9770 | 19.6 ms | 22.4 ms | 24.7 ms | 3.2 GiB |
| 1M | index/rabitq4 | NO_REACH | w128 np200 | 0.9370 | 42.0 ms | 48.3 ms | 49.6 ms | 1014.4 MiB |
| 1M | index/rabitq8 | NO_REACH | w128 np200 | 0.9520 | 47.1 ms | 60.9 ms | 69.4 ms | 1.7 GiB |
| 1M | index/turboquant | NO_REACH | w128 np200 | 0.9510 | 42.4 ms | 48.9 ms | 50.4 ms | 1013.9 MiB |

## Target Recall >= 0.99

| Scale | Variant | Status | Row | Recall@10 | p50 | p95 | p99 | IVF index |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| 10k | source/f32 | hit | w32 np16 | 0.9960 | 2.77 ms | 3.13 ms | 3.29 ms | 5.1 MiB |
| 10k | index/f16 | hit | w32 np16 | 0.9950 | 1.90 ms | 2.17 ms | 2.63 ms | 37.0 MiB |
| 10k | index/rabitq4 | NO_REACH | w64 np64 | 0.9790 | 2.35 ms | 2.77 ms | 3.17 ms | 13.9 MiB |
| 10k | index/rabitq8 | NO_REACH | w64 np128 | 0.9865 | 2.95 ms | 4.40 ms | 5.26 ms | 21.3 MiB |
| 10k | index/turboquant | NO_REACH | w64 np64 | 0.9815 | 2.37 ms | 2.80 ms | 3.24 ms | 13.9 MiB |
| 50k | source/f32 | hit | w64 np128 | 0.9965 | 7.32 ms | 8.28 ms | 9.44 ms | 13.8 MiB |
| 50k | index/f16 | hit | w64 np128 | 0.9965 | 7.17 ms | 8.47 ms | 9.63 ms | 166.7 MiB |
| 50k | index/rabitq4 | NO_REACH | w128 np200 | 0.9460 | 8.86 ms | 10.3 ms | 11.0 ms | 54.0 MiB |
| 50k | index/rabitq8 | NO_REACH | w128 np200 | 0.9545 | 9.69 ms | 13.4 ms | 17.3 ms | 90.8 MiB |
| 50k | index/turboquant | NO_REACH | w128 np200 | 0.9600 | 8.53 ms | 9.57 ms | 11.0 ms | 53.9 MiB |
| 100k | source/f32 | hit | w64 np128 | 0.9945 | 11.1 ms | 12.0 ms | 13.1 ms | 24.6 MiB |
| 100k | index/f16 | hit | w64 np128 | 0.9935 | 11.7 ms | 13.4 ms | 16.1 ms | 330.1 MiB |
| 100k | index/rabitq4 | NO_REACH | w64 np200 | 0.9380 | 15.3 ms | 18.8 ms | 21.0 ms | 110.2 MiB |
| 100k | index/rabitq8 | NO_REACH | w64 np200 | 0.9525 | 14.4 ms | 15.8 ms | 16.8 ms | 183.6 MiB |
| 100k | index/turboquant | NO_REACH | w128 np200 | 0.9565 | 18.2 ms | 28.2 ms | 36.2 ms | 104.4 MiB |
| 1M | source/f32 | hit | w128 np200 | 0.9910 | 44.1 ms | 51.2 ms | 52.3 ms | 226.8 MiB |
| 1M | index/f16 | hit | w256 np200 | 0.9910 | 61.9 ms | 104 ms | 120 ms | 3.1 GiB |
| 1M | index/rabitq4 | NO_REACH | w128 np200 | 0.9370 | 42.0 ms | 48.3 ms | 49.6 ms | 1014.4 MiB |
| 1M | index/rabitq8 | NO_REACH | w128 np200 | 0.9520 | 47.1 ms | 60.9 ms | 69.4 ms | 1.7 GiB |
| 1M | index/turboquant | NO_REACH | w128 np200 | 0.9510 | 42.4 ms | 48.9 ms | 50.4 ms | 1013.9 MiB |

## Readout

- At 10k, compact index-side formats can be faster than source/f32 at the lower
  targets, but all compact formats use a larger IVF index than source/f32. This
  small fixture is not representative of the larger tradeoff.
- At 50k, 100k, and 1M, source/f32 is the best warm-cache local reference at
  `0.95` and `0.97`: it is at least as fast as f16, materially faster than
  RaBitQ8/TurboQuant where they reach the target, and has the smallest IVF
  index.
- At `0.99`, only source/f32 and f16 reach the target beyond 10k. f16 is
  recall-neutral but storage-heavy: 37.0 MiB vs 5.1 MiB at 10k, 166.7 MiB vs
  13.8 MiB at 50k, 330.1 MiB vs 24.6 MiB at 100k, and 3.1 GiB vs 226.8 MiB at
  1M.
- RaBitQ4 does not reach `0.95` at 50k or larger in this grid.
- RaBitQ8 and TurboQuant can reach `0.95` at 50k/100k/1M, but not `0.97` or
  `0.99` at those scales. TurboQuant is generally the smaller compact option;
  RaBitQ8 is generally larger and does not produce a higher matched-recall
  frontier in this data.

## Warm-Cache Local Decisions

| Format / placement | Decision for this evidence slice | Rationale |
| --- | --- | --- |
| `source/f32` | Promote as the warm-cache local reference/default | Smallest IVF index and best matched recall/latency frontier at 50k, 100k, and 1M. |
| `index/f16` | Iterate, do not promote current layout | Recall-neutral, but current index layout is storage-heavy and loses latency at 100k/1M matched recall. |
| `index/rabitq4` | Iterate only if a new fidelity/storage hypothesis exists | Does not reach `0.95` at 50k or larger in the post-v7 grid. |
| `index/rabitq8` | Iterate, do not promote current layout | Reaches `0.95` only at high nprobe for larger scales and does not reach `0.97`; larger than TurboQuant. |
| `index/turboquant` | Iterate as the best compact index-side candidate, not promote | Smaller than RaBitQ8 and strongest compact option, but still misses `0.97`/`0.99` at 50k+ and loses to source/f32 at matched recall. |

## Non-Claims

This is not a final Task 111h closeout. It does not supply:

- cold-cache or remote-storage evidence,
- table-owned persisted compact storage or blocker evidence,
- a legacy `0x2A` direct-TID sidecar baseline,
- new read-amplification/page-read counters beyond the existing packet
  counters,
- new PG18 lifecycle tests beyond the already reviewed fixtures.

It closes only the cross-scale warm-cache matched-recall decision-table gap.
