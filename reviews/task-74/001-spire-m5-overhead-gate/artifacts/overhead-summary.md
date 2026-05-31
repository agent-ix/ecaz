# Task 74 M5 Overhead Summary

## Decision

Proceed to AWS testing, but profile only Task 73-selected points plus the IVF control. The M5 laptop shows SPIRE can reach the recall ceiling locally, so Task 74's speed question is now about the cost of the high-recall path rather than whether that path exists.

## Local Candidate Points

| surface | setting | recall@10 | p50 | p95 | p99 | note |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| SPIRE default shape | tg16 b0 nprobe=16 | 0.8525 | 13.505 ms | 15.410 ms | 15.868 ms | Fast but too low recall. |
| SPIRE b0 candidate | tg128 b0 nprobe=64 | 0.9825 | 51.227 ms | 54.958 ms | 59.428 ms | Good diagnostic point below 0.99. |
| SPIRE b0 candidate | tg128 b0 nprobe=96 | 0.9975 | 75.790 ms | 79.387 ms | 82.456 ms | Best local high-recall AWS candidate. |
| SPIRE b0 ceiling | tg128 b0 nprobe=128 | 1.0000 | 95.960 ms | 96.476 ms | 99.049 ms | Recall ceiling candidate. |
| SPIRE b1 | tg128 b1 nprobe=64 | 0.9940 | 108.444 ms | 116.407 ms | 119.364 ms | Slower than b0 high-recall candidate. |
| SPIRE b2 | tg128 b2 nprobe=64 | 0.9970 | 167.272 ms | 180.893 ms | 184.764 ms | Slower than b0 high-recall candidate. |
| IVF control | nprobe=96, heap rerank 500 | 0.9980 | 10.6 ms | 11.9 ms | 14.0 ms | Same-host leaf-scan control. |
| IVF control | nprobe=128, heap rerank 500 | 1.0000 | 12.7 ms | 13.8 ms | 14.3 ms | Same-host ceiling control. |

## Overhead Read

The local high-recall SPIRE b0 path is roughly 7x to 8x slower than the IVF control at comparable recall on this M5 host:

- SPIRE tg128 b0 nprobe=96: recall@10 `0.9975`, p50 `75.790 ms`
- IVF nprobe=96: recall@10 `0.9980`, p50 `10.6 ms`
- SPIRE tg128 b0 nprobe=128: recall@10 `1.0000`, p50 `95.960 ms`
- IVF nprobe=128: recall@10 `1.0000`, p50 `12.7 ms`

SPIRE local pipeline counters show the high-recall b0 path reads the same saturated local route/candidate envelope at nprobe 64/96/128 on this fixture:

- nprobe 64: `leaf_route_sum=3556`, `candidate_sum=2784952`, `object_bytes_sum=2237295832`
- nprobe 96: `leaf_route_sum=3556`, `candidate_sum=2784952`, `object_bytes_sum=2237295832`
- nprobe 128: `leaf_route_sum=3556`, `candidate_sum=2784952`, `object_bytes_sum=2237295832`

That makes the AWS profile useful: it can separate whether the high-recall cost is mostly SPIRE local leaf/candidate processing, remote fanout/tuple transport, or distributed placement behavior. The local b1/b2 runs are not worth carrying forward as default AWS candidates because their recall gains come with much higher local latency.

## Recommended AWS Matrix

- SPIRE current default: tg16 b0 nprobe=16
- SPIRE high-recall candidate: tg128 b0 nprobe=96
- SPIRE ceiling candidate: tg128 b0 nprobe=128
- IVF control: nprobe=96 and nprobe=128 with heap rerank 500

Do not use older IVF recall@100 numbers as a direct comparator for this decision; this packet uses recall@10 for both SPIRE and IVF.
