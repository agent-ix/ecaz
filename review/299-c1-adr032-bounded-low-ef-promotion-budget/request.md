# Review Request: C1 ADR-032 Bounded Low-Ef Promotion Budget

## Context

Retrospective split from the original packet `293`.

Packet `298` showed that exact-promoting every layer-0 source was too expensive. This follow-up
tested a much narrower hybrid: allow only a tiny amount of early low-`ef` source promotion.

## Attempt

- keep the kept `297` exact-on-head path as the base
- only at low `ef_search` (`<= 64`), allow a tiny layer-0 early-promotion budget
- budget tried here: exact-promote only the first `8` layer-0 expansion candidates

This was a local follow-up experiment off the kept `297` path. No separate green checkpoint was
committed from it because it was discarded after measurement.

## Measurements

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`.

All known warm runs for this attempt:

- `p50=1.051ms`
- `p95=1.491ms`
- `p99=1.741ms`
- `mean=1.080ms`

Full real-`50k`, `1000` queries.

All known recall rows for this attempt:

- `graph_recall_at_10 = 0.7612`
- `exact_quantized_recall_at_10 = 0.7612`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

As with later ADR-032 follow-ups on this branch, the exact-quantized comparator is not a reliable
exact reference; the meaningful quality read is `graph_recall_at_10`.

## Outcome

Discarded.

Latency stayed excellent, but recall fell below the standing kept `297` `ef=40` read of `0.8080`.
A small low-`ef` source-promotion budget was not a free middle ground.
