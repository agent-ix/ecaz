# Review Request: C1 ADR-032 Low-Ef Frontier Sweep

## Context

Packet `297` is the current kept ADR-032 runtime base on this branch:

- approximate layer-0 search
- lazy exact scoring at frontier consumption time

That cut materially improved warm latency, but low-`ef_search=40` recall on the full real `50k`
corpus stayed at `graph_recall_at_10 = 0.8080`.

Follow-up packets `303`, `304`, and `305` ruled out several local recovery theories:

- more exact-scoring budget in the current frontier did not help
- disabling source-local pruning did not help
- exact-reranking a wider discovered pool made both latency and recall worse

Reviewer feedback now suggests the practical next read should be an `ef_search` sweep before more
runtime redesign: redraw the current ADR-032 low-`ef` frontier on a same-build basis and see where
it overtakes the older ADR-031 quality point.

## Planned Slice

Measure the current kept ADR-032 runtime on the full real `50k` corpus at:

- `m=8`
- `ef_search = 40, 48, 56, 64`

For each cell, capture:

1. canonical warm latency
2. full real-`50k`, `1000`-query recall summary

## Success Criteria

- the packet records all known warm and recall results for `ef=40/48/56/64`
- the read is same-build and apples-to-apples across the sweep
- if a practical low-`ef` operating point appears, the packet calls it out explicitly
