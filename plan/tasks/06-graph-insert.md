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
- Backlink mutation, upper-layer insert search, neighbor shrinking, drift statistics, and
  `build_source_column` insert support remain pending.

## Milestone Tracker

- [x] `20%` Level assignment, neighbor tuple sizing, metadata entry-point / max-level promotion
- [x] `35%` Greedy descent, layer-0 candidate search, and new-node forward links
- [ ] `50%` Upper-layer candidate search and/or broader forward-link coverage
- [ ] `75%` Backlink mutation with explicit lock-ordering ADR
- [ ] `90%` Neighbor overflow handling and shrinking
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
- [ ] **Upper-layer insert search.** Extend candidate discovery above layer 0 when insert-side
  forward links stop being layer-0 only.
- [ ] **Upper-layer forward links.** Decide whether upper-layer forward links are written with the
  upper-layer search step above or deferred until backlink work lands.
- [ ] **Back-link updates.** For each selected neighbor, read their TqNeighborTuple, add new node's TID, prune weakest if at capacity M. Each update in its own GenericXLog transaction.
- [ ] **Neighbor list shrinking.** Prune weakest existing links when backlink mutation overflows a
  target layer budget.
- [ ] **Drift statistics.** Track `inserted_since_rebuild` in metadata. Expose via page-inspection or SQL.
- [ ] **Lock ordering protocol.** Define and document consistent page lock ordering to prevent deadlock under concurrent insert.

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
- Pruning weakest neighbor requires scoring all existing neighbors to decide which to evict.
- The neighbor selection logic is shared with vacuum graph repair (Task 07).
