# Artifact Manifest: Task 43 Packet 003

- Head SHA: `a5d08abee5b9d4acd438f9281085d4a18a45f8c2`
- Code checkpoint SHA: `2d6adc05`
- Task bucket: `reviews/task-43/003-pure-graph-miri-prefixes`
- Timestamp: `2026-05-18T09:37:17-07:00`
- Lane: targeted Miri execution for promoted pure graph `miri_` tests
- Fixture / storage format / rerank mode: not applicable
- Index surface: not applicable; pure Rust graph helper unit tests only

## Artifacts

### `miri-robust-prune.log`

- Command: `cargo +nightly miri test --lib miri_robust_prune_excludes_alpha_dominated`
- Key result: `test am::ec_diskann::vamana::tests::miri_robust_prune_excludes_alpha_dominated ... ok`
- Exit: `0`

### `miri-greedy-search.log`

- Command: `cargo +nightly miri test --lib miri_greedy_search_finds_nearest`
- Key result: `test am::ec_diskann::vamana::tests::miri_greedy_search_finds_nearest ... ok`
- Exit: `0`

### `miri-beam-dedupe.log`

- Command: `cargo +nightly miri test --lib miri_beam_search_deduplicates_self_loops_and_parallel_edges`
- Key result: `test am::ec_hnsw::search::tests::miri_beam_search_deduplicates_self_loops_and_parallel_edges ... ok`
- Exit: `0`

### `miri-visible-frontier.log`

- Command: `cargo +nightly miri test --lib miri_visible_frontier_best_candidate_prefers_live_scheduler_node`
- Key result: `test am::ec_hnsw::search::tests::miri_visible_frontier_best_candidate_prefers_live_scheduler_node ... ok`
- Exit: `0`

