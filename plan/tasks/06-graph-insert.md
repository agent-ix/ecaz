# Task 06: Graph-Aware Insert

Status: in progress

Progress notes:
- Live insert shape validation, metadata initialization, duplicate coalescing, and tail-page
  append/reuse are implemented.
- A4 recall gating and the shared traversal helpers are now complete on `main`, so A5 is no
  longer blocked.
- Initial A5 checkpoints landed random insert-level assignment, pre-sized neighbor tuple
  allocation, and metadata entry-point / max-level promotion.
- The next landed slice reuses `greedy_descend_from_entry` and the existing layer-0 search helper
  to populate simple top-`M` forward links on the new node only.
- Layer-0 backlink mutation now updates selected existing neighbors in physical page order and
  makes live inserts graph-reachable through the graph-first scan path when selected neighbors
  still have free layer-0 capacity.
- Insert now also runs upper-layer candidate discovery for live upper-level nodes, writes simple
  upper-layer forward links on the new node, and applies matching upper-layer backlinks when the
  selected targets still have free upper-layer capacity.
- Backlink mutation now also prunes full target slices with simple score-ordered top-`M` /
  top-`2M` selection, guarded by a same-snapshot check before the page rewrite so concurrent
  full-slice drift is conservatively skipped instead of overwritten.
- Drift statistics and `build_source_column` insert support remain pending.

## Milestone Tracker

- [x] `20%` Level assignment, neighbor tuple sizing, metadata entry-point / max-level promotion
- [x] `35%` Greedy descent, layer-0 candidate search, and new-node forward links
- [x] `50%` Layer-0 backlinks, graph reachability, and explicit lock-ordering ADR
- [x] `75%` Upper-layer insert search and upper-layer backlink coverage
- [x] `90%` Neighbor overflow handling and shrinking
- [ ] `100%` Drift accounting and concurrency hardening

## Scope

Replace disconnected-append insert with graph-connected insert using shared traversal helpers from Task 05.

## Subtasks

- [x] **Layer assignment.** `floor(-ln(random()) / ln(M))` geometric distribution.
- [x] **Pre-sized neighbor tuples.** Allocate `neighbor_slots(level, m)` at insert time instead of
  hard-coding empty level-0 storage.
- [x] **Entry point promotion.** Update metadata entry point and max_level when new node has higher layer.
- [x] **Greedy descent + layer-0 beam search.** Reuse the shared traversal helpers to seed insert
  candidate discovery and select a simple top-`M` forward set for the new node.
- [x] **Upper-layer insert search.** Live upper-level inserts now reuse `search_layer_result_candidates`
  above layer 0 to discover per-layer candidates.
- [x] **Upper-layer forward links.** Upper-level inserts now populate the matching upper-layer
  slices on the new node with simple top-`M` candidates.
- [x] **Back-link updates.** Selected layer-0 neighbors now receive the new node's TID when they
  still have free layer-0 capacity, with updates grouped one page at a time in ascending physical
  order. Overflow pruning remains deferred.
- [x] **Upper-layer back-link updates.** Selected upper-layer neighbors now receive the new node's
  TID in the matching upper-layer slice when free capacity exists; overflow pruning remains deferred.
- [x] **Neighbor list shrinking.** Full target slices now use simple score-ordered top-`M` /
  top-`2M` pruning for the selected layer, while guarded rewrites skip concurrent full-slice drift
  instead of overwriting it blindly.
- [ ] **Drift statistics.** Track `inserted_since_rebuild` in metadata. Expose via page-inspection or SQL.
- [x] **Lock ordering protocol.** Document and use ascending physical data-page order for backlink
  mutation, with metadata updates deferred until after data-page writes.

## Owns

- `FR-016`

## Dependencies

- Task 05 subtask A2 (graph traversal helpers) — complete
- Task 05 subtask A3 (working scan to test reachability of inserted nodes) — complete

## Unblocks

- Task 07 (vacuum reuses neighbor selection/pruning from insert)
- Task 10 (insert-drift benchmarks)

## Deliverables

- Graph-connected `aminsert`
- Lock ordering documentation (ADR)
- `inserted_since_rebuild` drift counter
- Concurrent insert safety

## Primary Tests

- `TC-128`, `TC-133`: insert behavior
- `BC-011`: insert-drift observability
- Inserted row reachable via HNSW scan
- No deadlock under concurrent insert

## Notes

- Back-link updates are the hardest part: each insert touches O(M) neighbor pages.
- The current backlink slices are still intentionally narrow: they only fill free layer-0 / upper-layer
  slots and defer concurrency hardening to the final A5 checkpoint.
- Overflow pruning now scores the current layer slice plus the new node and keeps the best
  `2M` layer-0 or `M` upper-layer links using the existing code scorer.
- Full-slice rewrite still refuses to overwrite a target layer that changed since the read-side
  planning snapshot; the concurrency-focused retry/hardening work remains part of the final
  `100%` milestone.
- The neighbor selection logic is shared with vacuum graph repair (Task 07).
