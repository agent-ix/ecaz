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

## Attempt

Implement a low-`ef` global-frontier exact policy on top of the kept `297` path.

Shape tried here:

1. identify the best visible-frontier candidates globally, not per source
2. exact-score a small bounded number of them and cache those exact scores
3. select the next candidate against the whole frontier using exact scores where available and
   approximate scores otherwise
4. keep the existing exact-on-head fallback for candidates that still reach selection without an
   exact score

Concrete policy used here:

- arm only for low `ef_search <= 64`
- target `ceil(ef_search / 8)` exact-scored visible candidates, clamped to `2..6`
- total extra exact-score budget `min(ceil(ef_search / 4), 12)` per scan

Non-goals:

- no persisted-format change
- no new quantizer or scoring algorithm
- no resurrection of the local-per-source budget policy from `302`

## Validation

This attempt was measured and then discarded. No green code checkpoint was committed from it.

All known validation reads:

- `cargo test`: green
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`:
  - first full run reported a failure on `tests::pg_test_tqhnsw_successor_candidate_from_entry_adjacency`
  - isolated rerun of that exact test passed cleanly
  - full rerun then completed green
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`:
  - failed on a test-only `clippy::field_reassign_with_default` lint in the new ADR-032 helper test

Because the slice was discarded, I did not spend more time polishing the clippy-only test lint.

## Measurements

Diagnostic sample, real `50k`, `m=8`, `ef_search=40`, first `10` queries.

All known diagnostic measurements for this attempt:

- representative single-query read (`id=50000`):
  - `candidate_score_calls = 41`
  - `score_cache_hits = 2`
  - `score_cache_misses = 41`
  - `graph_element_cache_misses = 766`
  - `rescan_layer0_seed_elapsed_us = 2053`
- `10`-query sample averages:
  - `avg candidate_score_calls = 41.00`
  - `avg graph_element_cache_misses = 588.40`
  - `avg score_cache_hits = 1.90`
  - `avg score_cache_misses = 41.00`
  - `avg rescan_layer0_seed_elapsed_us = 725.50`
  - `min candidate_score_calls = 41`
  - `max candidate_score_calls = 41`

For comparison, packet `302` showed the kept `297` path at roughly `2` exact-score calls per query
on the same low-`ef` seam.

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`.

All known warm runs for this attempt:

- run 1:
  - `p50=0.813ms`
  - `p95=1.107ms`
  - `p99=1.334ms`
  - `mean=0.832ms`
  - `min=0.515ms`
  - `max=1.769ms`
  - `server_qps=1202.64`
  - `wall=10.90s`
- run 2:
  - `p50=0.826ms`
  - `p95=1.134ms`
  - `p99=1.396ms`
  - `mean=0.847ms`
  - `min=0.531ms`
  - `max=1.598ms`
  - `server_qps=1180.82`
  - `wall=11.40s`

Reference kept `297` warm reads on the same seam:

- run 1: `p50=0.869ms`, `p99=1.559ms`, `mean=0.889ms`
- run 2: `p50=0.875ms`, `p99=1.558ms`, `mean=0.904ms`

Full real-`50k`, `1000` queries.

All known recall rows for this attempt:

- `graph_recall_at_10 = 0.8080`
- `exact_quantized_recall_at_10 = 0.8080`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

As with the other ADR-032 follow-ups on this branch, the exact-quantized comparator is not a
reliable exact reference; the meaningful quality read is `graph_recall_at_10` versus fp32 truth.

## Outcome

Discarded.

This cut spent far more exact work than the kept `297` path while leaving low-`ef` recall flat at
`0.8080`. The latency read was modestly better, but not enough to justify the extra exact-score
pressure or the added control-flow complexity.

The main read is that globally exact-scoring the current visible-frontier leaders still spends
budget in the wrong place. The next ADR-032 follow-up should not be another variant of “exact-score
more of the same low-`ef` visible frontier.” It needs a different lever for recovering quality.
