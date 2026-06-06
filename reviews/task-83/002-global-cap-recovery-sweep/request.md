# Task 83 Review Request: Global-Cap Recovery Sweep and Closeout

## Summary

This packet runs the bounded AWS 1M/q500 recovery sweep implied by packet 001's
target-block-rank diagnostic. It tests whether raising the selected-block global
cap from the retained `1152` baseline can recover the `81` selected-leaf misses
without undoing the Task 79/81 candidate-surface reduction.

Result: larger caps recover recall, but only by raising the q500 candidate
surface from `9.21M` to `10.24M-13.31M`. That is useful attribution, but not a
clean recovery policy for the latency task line. Task 83 closes with the
recommendation to pursue better block scoring or selective rescue, not a blanket
global-cap increase.

## AWS 1M/q500 Results

Baseline from packet 001 / Task 82 retained surface:

| cap | recall@10 | candidate_sum | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1152 | 0.9832 | 9,213,846 | 288.769 ms | 363.138 ms | 375.732 ms |

Recovery sweep:

| cap | recall@10 | candidate_sum | delta candidates vs 1152 | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1280 | 0.9846 | 10,237,554 | +1,023,708 | 292.896 ms | 363.363 ms | 380.597 ms |
| 1536 | 0.9876 | 12,284,852 | +3,071,006 | 287.312 ms | 344.188 ms | 354.646 ms |
| 1664 | 0.9892 | 13,308,518 | +4,094,672 | 295.989 ms | 352.377 ms | 364.170 ms |

## Decision

No Task 83 recovery policy lands. The evidence confirms the remaining selected
leaf recall misses are block-containment misses, and a higher cap can recover
some of them. But the narrow cap sweep still grows candidate scoring by
`11%-44%` over the retained surface, with `1664` already approaching the older
large candidate surface that Task 79/80 moved away from.

The next concrete SPIRE task should target selective recovery instead of a
blanket cap increase:

- improve selected-block scoring so the true target blocks rank inside the
  existing `1152` cap;
- or add a bounded rescue path that spends extra block budget only on ambiguous
  near-cap leaves/queries, then validates against the same Task 79/81 baseline.

## Validation

- `ecaz bench suite audit`: passed for the 3-step recovery suite.
- AWS `1m` q500 suite completed and synced artifacts for run
  `20260606T001806Z`.
- AWS `1m` was paused after the run; final packet-local status shows
  `state: paused`.

See `artifacts/manifest.md` for commands, artifact paths, and provenance.

## Requested Review

Please review whether the closeout decision is consistent with Task 83's gate:
recovering recall via a global-cap increase is measured, but rejected as the
policy because it materially increases the retained candidate surface.
