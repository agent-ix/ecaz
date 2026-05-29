# Task 65 Hot-Loop Static Allocation Audit

Head: `8e860324c1fa2a009bab209502962375f0207642`

Scope: `src/am/ec_diskann/ambuild.rs` and
`src/am/ec_diskann/vamana.rs`.

Command used:

```text
rg -n "vec!\[false; n\]|vec!\[false; node_count\]|greedy_search_view_with_scratch|SearchScratch|BinaryHeap|source_vectors_match_exactly|overflow_heap_tids|stage_overflow_heap_tids_in_chain|frontier\.sort|frontier\.truncate|min_by" src/am/ec_diskann/ambuild.rs src/am/ec_diskann/vamana.rs src/am/ec_diskann/routine.rs
```

Key output:

```text
src/am/ec_diskann/vamana.rs:22:use std::{cell::Cell, cmp::Reverse, collections::BinaryHeap, time::Instant};
src/am/ec_diskann/vamana.rs:155:pub struct SearchScratch {
src/am/ec_diskann/vamana.rs:159:    unexpanded: BinaryHeap<Reverse<Candidate>>,
src/am/ec_diskann/vamana.rs:160:    retained: BinaryHeap<Candidate>,
src/am/ec_diskann/vamana.rs:164:impl SearchScratch {
src/am/ec_diskann/vamana.rs:170:            unexpanded: BinaryHeap::with_capacity(list_size.saturating_add(1)),
src/am/ec_diskann/vamana.rs:171:            retained: BinaryHeap::with_capacity(list_size.saturating_add(1)),
src/am/ec_diskann/vamana.rs:235:/// `BinaryHeap<Reverse<Candidate>>` becomes a min-heap.
src/am/ec_diskann/vamana.rs:292:    let mut scratch = SearchScratch::new(graph.node_count(), list_size);
src/am/ec_diskann/vamana.rs:293:    greedy_search_view_with_scratch(graph, start, list_size, &mut scratch, query_dist)
src/am/ec_diskann/vamana.rs:296:pub fn greedy_search_view_with_scratch<G, D>(
src/am/ec_diskann/vamana.rs:300:    scratch: &mut SearchScratch,
src/am/ec_diskann/vamana.rs:351:    frontier.sort_unstable();
src/am/ec_diskann/vamana.rs:358:fn push_frontier_candidate(scratch: &mut SearchScratch, list_size: usize, candidate: Candidate) {
src/am/ec_diskann/vamana.rs:469:    scratch: &mut SearchScratch,
src/am/ec_diskann/vamana.rs:506:    scratch: &mut SearchScratch,
src/am/ec_diskann/vamana.rs:669:    let mut scratch = SearchScratch::new(node_count, list_size);
src/am/ec_diskann/vamana.rs:679:            greedy_search_view_with_scratch(&graph, medoid, list_size, &mut scratch, |n| {
src/am/ec_diskann/vamana.rs:814:    let mut seen = vec![false; n];
```

Interpretation:

- Build-time greedy search uses `SearchScratch` with `Vec<u64>` bitsets and
  two bounded heaps, not per-search `vec![false; n]` allocations.
- The build path uses `greedy_search_visited_with_scratch`, so it does not
  construct the sorted top-`L` frontier vector needed only by scan/test
  callers.
- The remaining `vec![false; n]` is in the persisted-graph BFS helper, not in
  the Vamana pivot build loop.
- The remaining `frontier.sort_unstable()` is final output ordering after
  heap-backed search, not repeated trim/truncate work inside expansion.
- The search found no build-time `source_vectors_match_exactly`,
  `overflow_heap_tids`, or `stage_overflow_heap_tids_in_chain` references in
  `ambuild.rs`; runtime insert overflow remains outside the build path.

Runtime smoke:

```text
cargo run --release --features bench,dhat-heap --bin dhat_vamana_build -- --input fixtures/m5_diskann_real10k/m5_diskann_real10k_corpus.tsv --rows 1000 --graph-degree 32 --list-size 200 --alpha 1.2 --seed 42 --output reviews/task-65/002-vamana-core-measurement/artifacts/dhat-vamana-build-real1k-r32-l200.json --summary-output reviews/task-65/002-vamana-core-measurement/artifacts/dhat-vamana-build-real1k-r32-l200-summary.md
```

The profiler starts after TSV parsing and medoid selection and stops
immediately after `build_vamana_graph_with_stats`. The JSON backtraces show the
expected one-time `SearchScratch::new` and graph/prune allocations, with no
`bfs_reachable` or per-search `vec![false; n]` path in the profiled build call.
