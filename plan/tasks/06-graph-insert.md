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
  top-`2M` selection.
- Metadata now tracks `inserted_since_rebuild`, and the SQL/admin snapshot reports both that
  counter and the derived insert-drift fraction for live indexes.
- Full-slice backlink rewrites now use bounded read-only replanning when the live layer drifts
  before the page rewrite, so stale full-slice plans are retried instead of being silently skipped.
- `build_source_column` live insert remains intentionally unsupported in v0.1.

## Milestone Tracker

- [x] `20%` Level assignment, neighbor tuple sizing, metadata entry-point / max-level promotion
- [x] `35%` Greedy descent, layer-0 candidate search, and new-node forward links
- [x] `50%` Layer-0 backlinks, graph reachability, and explicit lock-ordering ADR
- [x] `75%` Upper-layer insert search and upper-layer backlink coverage
- [x] `90%` Neighbor overflow handling and shrinking
- [x] `95%` Drift accounting and SQL/admin observability
- [x] `100%` Concurrency hardening and concurrency-focused validation

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
  top-`2M` pruning for the selected layer.
- [x] **Drift statistics.** `inserted_since_rebuild` now persists in metadata and is exposed
  through `ec_hnsw_index_admin_snapshot(regclass)` alongside the derived drift fraction.
- [x] **Lock ordering protocol.** Document and use ascending physical data-page order for backlink
  mutation, with metadata updates deferred until after data-page writes.
- [x] **Concurrency hardening.** Stale full-slice backlink plans now re-enter a bounded read-only
  replan pass instead of being silently skipped, and deterministic regression coverage locks in
  that retry path.

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
- Overflow pruning now scores the current layer slice plus the new node and keeps the best
  `2M` layer-0 or `M` upper-layer links using the existing code scorer.
- Bulk build and live insert intentionally share the same construction metric:
  `score_code_inner_product` over compressed codes only. Ordered scan still uses the
  gamma-aware raw-query scorer from ADR-007.
- Full-slice rewrite now refuses to overwrite a target layer that changed since the read-side
  planning snapshot while that page lock is held, then retries those targets through a bounded
  read-only replan pass after the current write pass completes.
- Successful live inserts now always finish with a metadata-page write phase because the
  drift counter is metadata-resident; ADR-026 still applies because metadata remains last.
- The neighbor selection logic is shared with vacuum graph repair (Task 07).
- Pre-sized live-insert neighbor tuples intentionally leave unused slots as `INVALID`, and the
  runtime traversal contract already skips those placeholder TIDs (`collect_successor_candidates`
  regression coverage in `src/am/scan.rs`).
- Insert-level assignment is deterministic per `(seed, heap_tid)` today. If delete/rewrite flows
  start recycling heap TIDs, future insert work must add fresh per-row entropy so levels do not
  remain stable across unrelated rows that reuse the same physical TID.
