# Review Request: C1 ADR-032 Hybrid Approx-Exact Ordering

## Context

Packet `297` remains the kept ADR-032 runtime base:

- low-cost approximate layer-0 search
- lazy exact scoring instead of eager exact-on-miss scoring

It is fast, but its low-`ef_search=40` full real-`50k` recall stays at `0.8080`.

Recent follow-ups established an important pattern:

- `303`: exact-scoring more of the current visible frontier did not improve recall
- `304`: relaxing low-`ef` source-local pruning did not improve recall
- `305`: exact-reranking a wider discovered pool made both latency and recall worse

## Problem

The low-`ef` ADR-032 quality point now looks less like “not enough exact scoring” and more like
“too much collapse toward quantized-exact ordering once exact scores enter the loop.”

The evidence for that is packet `305`: wider exact rerank pushed recall from `0.8080` down to
`0.7790`.

So the next credible seam is not another larger exact-rerank pool. It is a hybrid ordering policy
that preserves some approximate signal even after exact scores are available.

## Planned Slice

Prototype a hybrid low-`ef` ADR-032 ordering policy that blends or gates exact-score influence
instead of replacing approximate ordering outright.

Likely first cut:

1. keep the current cheap approximate layer-0 search
2. when exact scores become available, do not let them fully override approximate ordering
3. try a simple bounded hybrid policy first, not a learned model
4. remeasure the canonical warm real-`50k`, `m=8`, `ef=40` seam and the full recall summary

## Success Criteria

- the attempt records all known warm and recall results for the hybrid ordering policy
- low-`ef` recall improves over the kept `297` `0.8080` read
- warm latency stays materially below the older ADR-031 path
