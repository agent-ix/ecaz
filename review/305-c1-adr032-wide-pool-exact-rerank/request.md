# Review Request: C1 ADR-032 Wide-Pool Exact Rerank

## Context

Packet `297` is still the kept ADR-032 runtime base:

- binary-scored candidates drive the low-cost layer-0 search
- exact scoring is deferred instead of being paid eagerly on cache miss

That cut shifted the warm latency frontier decisively, but low-`ef_search=40` recall stayed at
`graph_recall_at_10 = 0.8080`.

Follow-up packets `303` and `304` ruled out two simpler explanations:

- `303`: exact-scoring more of the existing visible frontier did not recover recall
- `304`: relaxing low-`ef` source-local binary pruning did not recover recall either

That points at a different failure mode: the approximate search may be discovering the right
candidates, but the final exact-rerank pool is too narrow because it is effectively capped at the
same `ef_search` window.

## Problem

The current ADR-032 path still stages only the approximate search result window of size `ef_search`
for later exact adjudication.

If the approximate search is finding good nodes just outside that final `ef_search` frontier, then:

- exact-scoring the current frontier more carefully cannot help
- relaxing source-local pruning cannot help
- the next lever is a bounded *wider* cheap candidate pool followed by exact rerank

## Planned Slice

Prototype a two-stage low-`ef` ADR-032 search:

1. run the existing cheap approximate layer-0 search
2. keep a bounded candidate pool larger than `ef_search`
3. exact-score that widened pool after search
4. stage the best exact-scored candidates for output

Likely first cut:

- only arm for binary low-`ef_search <= 64`
- widen the rerank pool to a small multiple of `ef_search`
- keep the rest of the runtime path unchanged

## Success Criteria

- the attempt records all known warm and recall results for the widened rerank pool
- low-`ef` recall improves meaningfully over the kept `297` read (`0.8080`)
- the warm latency stays materially below the older ADR-031 path
