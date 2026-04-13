# Review Request: C1 ADR-032 Low-Ef Binary-Prune Relaxation

## Context

Packet `297` is still the kept ADR-032 runtime base:

- binary-filtered layer-0 successors enter the frontier with approximate scores
- exact scoring is deferred until a candidate reaches the frontier head

That cut materially improved warm latency on the real `50k` seam, but low-`ef_search=40` recall
stayed at `graph_recall_at_10 = 0.8080`.

Packet `303` then tried a global-frontier exact policy. It spent much more exact work and improved
low-`ef` latency only modestly, but recall stayed flat at `0.8080`. That makes it unlikely that the
missing quality is recoverable by exact-scoring more of the same visible frontier.

## Problem

The surviving ADR-032 path still uses ADR-031-style source-local binary pruning before candidates
ever reach the visible frontier.

That is now suspicious for two reasons:

- the big ADR-032 win came from lazy exact scoring, not from binary rejection itself
- if low-`ef` recall loss is partly caused by pruning too aggressively before frontier competition,
  then exact-scoring policies later in the pipeline cannot recover those lost candidates

## Planned Slice

Relax or disable source-local binary pruning at low `ef_search` on top of the kept `297` exact-on-
head path.

Likely shape:

1. keep the exact-on-head ADR-032 runtime base intact
2. only at low `ef_search` (`<= 64`), widen or disable the source-local binary rejection budget
3. remeasure the canonical warm real-`50k`, `m=8`, `ef=40` seam
4. rerun full real-`50k` recall to see whether low-`ef` quality recovers meaningfully

## Success Criteria

- the attempt records all known warm and recall results for the relaxed-pruning variant
- low-`ef` recall improves meaningfully over the kept `297` read (`0.8080`)
- the ADR-032 warm-latency advantage remains materially better than the kept ADR-031 path
