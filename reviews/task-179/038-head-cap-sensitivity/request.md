---
task: 179
packet: 038-head-cap-sensitivity
role: coder
status: review-requested
head: 5a66a47b67b6437fd3d80d840dff25f68e4dd139
benchmark_head: 6c25e55a22a7828ae5b3bb2c8309e15b3738d2d3
date: 2026-07-12
---

# Review request: 10k/50k/100k DistANN head-cap sensitivity

Please review the immutable suite evidence under `artifacts/`, the measured
FR-080/ADR-085 update in `5a66a47b6`, and the scoped decision in `verdict.md`.

This packet responds to the head-cap sensitivity half of packet 030 reviewer
P2-1. The requested decisions are:

1. Does the real three-owner PG18 3×3 matrix establish recall sensitivity for
   caps 64, 256, and 4096 at 10k, 50k, and 100k?
2. Do exact/disjoint topology and two remote-owner proofs in every arm isolate
   the observed recall differences to the cap rather than incomplete serving?
3. Do the 100k results justify retaining the frozen default of 4096?
4. Are the latency and storage results interpreted within their explicitly
   documented measurement boundaries?

All nine suite steps succeeded, all 27 thresholds passed, and no expected
artifact is missing. Physical recall for cap 64 / 256 / 4096 is:

| Scale | 64 | 256 | 4096 |
| --- | ---: | ---: | ---: |
| 10k | 0.9950 | 0.9950 | 1.0000 |
| 50k | 0.9750 | 0.9800 | 0.9800 |
| 100k | 0.9200 | 0.9450 | 0.9500 |

Cap 64 loses 0.030 recall at 100k. Cap 256 recovers most of that but still
trails 4096 by 0.005. Warm-majority physical p50 for 4096 is 70.7, 100.8, and
78.9 ms; one cold backend head fill remains in each 20-sample arm and is
visible in the default-cap mean/p95/p99.

This packet does not claim full packet 030 P2 closure: the historical removed
O(N) seed-scan A/B remains outstanding. It also does not claim Task 179 AC-13
latency closure, NFR-018 promotion, or closure of the new packet 033 P1
findings.
