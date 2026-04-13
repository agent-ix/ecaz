# Review Request: C1 ADR-032 Binary-Score Calibration

## Context

Retrospective split from the original packet `293`.

After `300` showed that head-window lookahead was too approximate, this follow-up tried the next
reviewer-suggested seam: calibrate binary approximate frontier scores into the exact-score range
before using them for exact-on-head comparisons.

## Attempt

The calibration constants used here were fit from a real-corpus binary-sign study on the real
`50k` corpus:

- `exact_from_binary intercept = 0.013522`
- `exact_from_binary slope = 0.000857`

In scan-score space, that produced:

- `calibrated_scan_score = raw_binary_scan_score * 0.000857 - 0.013522`

This was a local follow-up experiment off the kept `297` path. No separate green checkpoint was
committed from it because it was discarded after measurement.

## Measurements

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`.

All known warm runs for this attempt:

- `p50=0.729ms`
- `p95=1.055ms`
- `p99=1.294ms`
- `mean=0.750ms`

Full real-`50k`, `1000` queries.

All known recall rows for this attempt:

- `graph_recall_at_10 = 0.6358`
- `exact_quantized_recall_at_10 = 0.6358`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

As with the other ADR-032 follow-ups on this branch, the exact-quantized comparator is not a
reliable exact reference; the meaningful quality read is `graph_recall_at_10`.

## Outcome

Discarded.

Calibration made the low-`ef` seam even faster, but only by making the frontier behave even more
like the approximate binary scorer. Score-shape tweaks alone were not enough to recover quality.
