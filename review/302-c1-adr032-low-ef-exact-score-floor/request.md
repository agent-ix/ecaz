# Review Request: C1 ADR-032 Low-Ef Exact-Score Floor

## Context

Retrospective split from the original packet `293`.

After `301` showed that score-shape calibration still pushed the frontier too far toward the
approximate scorer, this follow-up tried the first score-budget accounting cut.

## Attempt

- keep the kept `297` exact-on-head base intact
- only at low `ef_search` (`<= 64`), arm a bounded exact-score budget derived from `ef_search`
- budget used here: `min(ef_search / 2, 24)` total exact scores, spent at most `1` per source
  expansion
- spend that budget on the best binary survivor from each expansion before it enters the frontier

This was a local follow-up experiment off the kept `297` path. It had focused unit-test coverage
and a release install for measurement, but it was discarded before a separate full green
checkpoint was committed.

## Measurements

Diagnostic sample, real `50k`, `m=8`, `ef_search=40`, first `10` queries.

All known diagnostic measurements for this attempt:

- baseline kept `297` path:
  - `avg candidate_score_calls = 2.00`
  - `avg graph_element_cache_misses = 572.80`
  - `avg score_cache_hits = 0.80`
  - `avg score_cache_misses = 2.00`
- exact-score-floor variant:
  - `avg candidate_score_calls = 60.50`
  - `avg graph_element_cache_misses = 655.20`
  - `avg score_cache_hits = 18.80`
  - `avg score_cache_misses = 60.50`
  - `min candidate_score_calls = 59`
  - `max candidate_score_calls = 61`

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`.

All known warm runs for this attempt:

- `p50=0.839ms`
- `p95=1.148ms`
- `p99=1.346ms`
- `mean=0.863ms`

Full real-`50k`, `1000` queries.

All known recall rows for this attempt:

- `graph_recall_at_10 = 0.6774`
- `exact_quantized_recall_at_10 = 0.6774`
- `graph_below_exact_queries = 0`
- `worst_exact_gap = 0`

As with the other ADR-032 follow-ups on this branch, the exact-quantized comparator is not a
reliable exact reference; the meaningful quality read is `graph_recall_at_10`.

## Outcome

Discarded.

This cut spent far more exact work than the standing `297` path while recovering much less recall
than expected. The missing quality was not solved by exact-scoring the best candidate from each
local source expansion; the added work was being spent in the wrong place.
