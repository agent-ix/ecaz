# Review Request: C1 ADR-032 Global Frontier Exact Policy

## Context

Packet `297` is the current kept ADR-032 cut:

- binary-filtered successors enter the frontier with approximate scores
- exact scoring is deferred until a candidate reaches the frontier head

That cut materially improved the warm real-`50k` frontier, but low-`ef_search=40` recall stayed at
`graph_recall_at_10 = 0.8080`.

Follow-up packets `298` through `302` all failed to recover low-`ef` quality:

- `298`: exact-promote every layer-0 source before expansion -> too expensive
- `299`: tiny low-`ef` source-promotion budget -> fast, recall worse
- `300`: low-`ef` head-window -> very fast, recall collapsed
- `301`: binary-score calibration -> very fast, recall worse
- `302`: low-`ef` exact-score floor per source expansion -> far more exact work, recall still bad

The common failure mode is that all of those attempts spend exact work locally:

- per source expansion
- per temporary head window
- per score-shape tweak

They do not treat the visible frontier as one global competition set.

## Problem

The next plausible ADR-032 recovery seam is global rather than local:

- spend a bounded amount of extra exact work on the most globally competitive visible-frontier
  candidates
- let those exact-scored candidates compete against the rest of the visible frontier as a whole
- avoid tying the extra exact work to each individual source expansion

If this helps, it would mean the missing low-`ef` quality is about where exact work is spent, not
just how much of it exists.

## Planned Slice

Implement a low-`ef` global-frontier exact policy on top of the kept `297` path.

Likely shape:

1. identify the best visible-frontier candidates globally, not per source
2. exact-score a small bounded number of them and cache those exact scores
3. select the next candidate against the whole frontier using exact scores where available and
   approximate scores otherwise
4. keep the existing exact-on-head fallback for candidates that still reach selection without an
   exact score

Non-goals:

- no persisted-format change
- no new quantizer or scoring algorithm
- no resurrection of the local-per-source budget policy from `302`

## Success Criteria

- the attempt records all known warm and recall results for the new global-frontier policy
- low-`ef` recall improves meaningfully over the kept `297` `ef=40` read (`0.8080`) without losing
  the ADR-032 warm-latency advantage
- if the attempt fails, the packet explains whether it failed on latency, recall, or both
