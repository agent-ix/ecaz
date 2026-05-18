# Review Request: Task 43 Pure Graph Miri Prefixes

## Summary

This checkpoint promotes four existing pure graph helper tests into the
`miri_` lane:

- DiskANN Vamana `robust_prune` alpha-dominance behavior.
- DiskANN Vamana greedy graph search convergence.
- HNSW beam search deduplication of self-loops and parallel edges.
- HNSW visible-frontier live-node candidate preference.

No algorithm behavior changed. The code change is limited to renaming existing
bounded unit tests so `make miri`, `make miri-tree`, and `make miri-full`
include them through the existing `miri_` prefix.

## Review Focus

- Confirm these are appropriate first pure graph coverage targets for Task 43.
- Confirm promoting existing bounded tests is acceptable for this slice rather
  than adding larger graph fixtures under Miri.
- Confirm the Miri logs are enough evidence for the renamed tests.

## Validation

Validation artifacts are in `artifacts/` and summarized by
`artifacts/manifest.md`.

- `cargo +nightly miri test --lib miri_robust_prune_excludes_alpha_dominated`
  passed.
- `cargo +nightly miri test --lib miri_greedy_search_finds_nearest` passed.
- `cargo +nightly miri test --lib miri_beam_search_deduplicates_self_loops_and_parallel_edges`
  passed.
- `cargo +nightly miri test --lib miri_visible_frontier_best_candidate_prefers_live_scheduler_node`
  passed.

No full Miri run is claimed in this packet; the prior infrastructure packet
added the aggregate lanes, and this packet validates only the newly promoted
pure graph tests.

