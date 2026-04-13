# Review Request: C1 ADR-032 Score-Pressure Diagnostic

## Context

Retrospective split from the original packet `293`.

Before attempting another ADR-032 runtime cut after `295`, I sampled the existing debug hot-path
counters on the kept ADR-031 path to determine whether the next redesign should target exact-score
volume or exact-score timing.

## Method

- real `50k` fixture
- `m=8`
- `ef_search=40`
- first `20` queries from `tqhnsw_real_50k_queries`
- `tests.tqhnsw_debug_scan_hot_path_profile(...)`
- compared with `tqhnsw.disable_binary_prefilter` reset vs `on`

This was a measurement-only diagnostic. No runtime code from this packet was kept or discarded.

## Measurements

All known reads for this diagnostic:

ADR-031 enabled:

- `avg candidate_score_calls = 521.45`
- `avg candidate_score_elapsed_us = 546.50`
- `avg graph_element_cache_misses = 527.50`
- `avg graph_neighbor_cache_misses = 48.50`
- `avg rescan_layer0_seed_elapsed_us = 1495.45`

ADR-031 disabled:

- `avg candidate_score_calls = 527.50`
- `avg candidate_score_elapsed_us = 546.50`
- `avg graph_element_cache_misses = 527.50`
- `avg graph_neighbor_cache_misses = 48.50`
- `avg rescan_layer0_seed_elapsed_us = 1215.70`

## Outcome

The next credible ADR-032 lever was **when** nodes graduate to exact scoring, not **how many**
candidates are scored.

On this seam, ADR-031 on/off barely changed exact-score call count. That ruled out another minor
survivor-pruning tweak as the likely next win and directly motivated the exact-on-head cut in
packet `297`.
