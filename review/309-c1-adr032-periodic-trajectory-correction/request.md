# Review Request: C1 ADR-032 Periodic In-Search Trajectory Correction

## Context

Packet `307` established the current practical ADR-032 operating points on the kept runtime:

- `ef=56`: `graph_recall_at_10 = 0.8417`, `mean = 0.990ms`
- `ef=64`: `graph_recall_at_10 = 0.8519`, `mean = 1.043ms`

That is useful, but it leaves the lower-`ef` structural gap unresolved:

- kept `ef=40`: `graph_recall_at_10 = 0.8080`

Packet `308` tested exact-scored multi-seed anchoring as the first trajectory-oriented follow-up.
That moved in the wrong direction:

- latency improved to `mean = 0.835ms`
- recall dropped to `0.7827`

So the simple “better initial seeds” version is not enough.

## Problem

The remaining reviewer-suggested structural seam is to correct the search trajectory *during*
low-`ef` layer-0 exploration, not only before it starts and not only after candidates have already
been discovered.

The aim is:

- keep the cheap approximate search as the default driver
- periodically exact-score the would-be next expansion source
- if the exact score says that source is worse than the next approximate frontier option, requeue it
  with its exact score and expand a different source instead

That changes which graph neighborhoods get explored under low `ef_search`, which is the remaining
hypothesis not yet tested directly.

## Planned Slice

Prototype periodic in-search trajectory correction for binary low-`ef` ADR-032 scans.

Likely first cut:

1. only arm for binary scans with `ef_search <= 64`
2. during layer-0 search, exact-score every `Nth` would-be expansion source
3. compare that exact score against the current approximate frontier head
4. if exact scoring makes it worse, requeue it and expand a different source instead
5. leave the rest of the ADR-032 runtime path unchanged

## Success Criteria

- the attempt records all known warm and recall results
- low-`ef` recall improves over the kept `ef=40` ADR-032 read (`0.8080`)
- latency remains in the low-millisecond ADR-032 band
