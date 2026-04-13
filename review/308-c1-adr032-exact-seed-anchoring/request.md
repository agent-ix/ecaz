# Review Request: C1 ADR-032 Exact-Scored Multi-Seed Anchoring

## Context

Packet `307` established the practical low-`ef` ADR-032 frontier on the current kept runtime:

- `ef=56`: `graph_recall_at_10 = 0.8417`, `mean = 0.990ms`
- `ef=64`: `graph_recall_at_10 = 0.8519`, `mean = 1.043ms`

That is useful operationally, but it does not remove the underlying low-`ef` trajectory gap:

- kept ADR-032 at `ef=40` still sits at `graph_recall_at_10 = 0.8080`
- post-discovery fixes (`303`, `304`, `305`) did not recover that gap

Reviewer feedback now points at the next structural seam: change the *early expansion trajectory*
rather than spending more exact work later.

## Problem

The current ADR-032 layer-0 search starts from a very narrow seed situation:

- one upper-layer descent winner
- then approximate-scored layer-0 exploration from there

If that first anchor is slightly wrong, a low `ef_search` budget can spend most of its expansions in
the wrong neighborhood. Later exact scoring cannot recover candidates that were never discovered.

## Planned Slice

Prototype exact-scored multi-seed anchoring for low-`ef` ADR-032 scans.

Likely first cut:

1. derive a small upper-layer seed set rather than a single seed
2. exact-score only that small seed set
3. start the existing cheap approximate layer-0 search from those exact-scored seeds
4. leave the rest of the ADR-032 runtime path unchanged

## Success Criteria

- the attempt records all known warm and recall results
- low-`ef` recall improves over the kept `297` / packet `307` `ef=40` read (`0.8080`)
- latency stays in the low-millisecond ADR-032 band rather than regressing back toward ADR-031
