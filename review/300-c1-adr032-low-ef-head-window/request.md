# Review Request: C1 ADR-032 Low-Ef Head Window

## Context

Retrospective split from the original packet `293`.

After `299` showed that a small source-promotion budget still hurt recall, this follow-up tried a
more conservative idea: exact-score only a tiny frontier head window before selecting the next
output/expansion candidate.

## Attempt

- keep the kept `297` exact-on-head base intact
- only at low `ef_search` (`<= 64`), exact-score a tiny frontier head window before choosing the
  next output/expansion candidate
- window tried here: `4` candidates
- requeue the non-winning window members with their exact scores

This was another local follow-up experiment off the kept `297` path. No separate green checkpoint
was committed from it because it was discarded after measurement.

## Measurements

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`.

All known warm runs for this attempt:

- `p50=0.794ms`
- `p95=1.101ms`
- `p99=1.334ms`
- `mean=0.814ms`

Full real-`50k`, `1000` queries.

All known recall rows for this attempt:

- `graph_recall_at_10 = 0.4507`
- `exact_quantized_recall_at_10 = 0.4507`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

As with later ADR-032 follow-ups on this branch, the exact-quantized comparator is not a reliable
exact reference; the meaningful quality read is `graph_recall_at_10`.

## Outcome

Discarded.

Latency was outstanding, but recall collapsed badly. The head-window policy was overfitting to the
approximate frontier ordering instead of repairing it.
