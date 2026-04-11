# Task 06: Graph-Aware Insert

Status: in progress

Progress notes:
- Live insert shape validation, metadata initialization, duplicate coalescing, and tail-page
  append/reuse are implemented.
- A4 recall gating and the shared traversal helpers are now complete on `main`, so A5 is no
  longer blocked.
- Current A5 checkpoint scopes random insert-level assignment, pre-sized neighbor tuple
  allocation, and metadata entry-point / max-level promotion before graph link mutation lands.
- Graph-aware insertion, drift statistics, and build_source_column insert support remain pending.

## Scope

Replace disconnected-append insert with graph-connected insert using shared traversal helpers from Task 05.

## Subtasks

- [ ] **Layer assignment.** `floor(-ln(random()) / ln(M))` geometric distribution.
- [ ] **Greedy descent + beam search.** Use A2 traversal helper with code-to-code scoring (`score_ip_codes_lite`). Find M best neighbors at each insertion layer.
- [ ] **Back-link updates.** For each selected neighbor, read their TqNeighborTuple, add new node's TID, prune weakest if at capacity M. Each update in its own GenericXLog transaction.
- [ ] **Entry point promotion.** Update metadata entry point and max_level when new node has higher layer.
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
