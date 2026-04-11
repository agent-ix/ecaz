# Task 07: Vacuum Three-Pass

Status: in progress on `main`

Progress notes:
- A5 is complete on `main`, so the shared traversal and neighbor-pruning dependencies are no
  longer blockers.
- Pass 1 is now landed: `ambulkdelete` strips dead heap TIDs from element tuples, and
  `amvacuumcleanup` reports live-element counts instead of raw tuple tags.
- Pass 2 dead-edge unlink is now landed: vacuum scans persisted neighbor tuples and clears
  references to fully-dead element TIDs one page at a time, so stale dead-node edges no longer
  remain on disk after vacuum.
- Pass 3 is also now landed for the no-repair checkpoint: fully dead elements are finalized to
  `deleted = true` once pass 1 removes their last heap TID.
- Duplicate discovery now skips deleted / empty-heaptid elements, so a post-vacuum duplicate
  insert cannot reattach to a finalized dead node.
- Replacement search / reconnect is still pending inside pass 2. The current repair slice unlinks
  dead edges but does not yet search for substitute neighbors.

## Scope

Implement ambulkdelete with three-pass delete algorithm and amvacuumcleanup with statistics update.

## Subtasks

- [x] **Pass 1 — Mark.** Scan element tuples, compare heap TIDs against the vacuum callback,
  remove dead TIDs from element tuples, and persist the compacted heaptid payload in place.
- [x] **Pass 2a — Dead-edge unlink.** Scan persisted neighbor tuples, remove references to
  fully-dead element TIDs, and rewrite changed pages one page at a time.
- [ ] **Pass 2b — Replacement search.** For each broken connection (deleted neighbor), search for
  replacement neighbors using A2 traversal with code-to-code scoring. Reuse neighbor
  selection/pruning logic from Task 06.
- [x] **Pass 3 — Finalize.** Set `deleted = true` on fully-dead element tuples once pass 1 strips
  their last heap TID.
- [x] **amvacuumcleanup.** Report live-element tuple counts and page counts through `IndexBulkDeleteResult`.
- [ ] **Concurrency validation.** Vacuum must not block concurrent INSERT or SELECT.

## Owns

- `FR-010`

## Dependencies

- Task 05 subtask A2 (graph traversal for repair search) — complete
- Task 06 (neighbor selection and pruning logic) — complete

## Unblocks

- Task 10 (post-vacuum recall and quality benchmarks)

## Deliverables

- Three-pass `ambulkdelete`
- `amvacuumcleanup` with live-element statistics
- Concurrent safety under INSERT + SELECT + VACUUM

## Primary Tests

- `TC-115`, `TC-118`, `TC-132`: vacuum behavior
- `BC-016`: post-vacuum recall quality
- Deleted rows absent from scan results
- Recall >= 80% of pre-vacuum after 10% deletion
- No corruption under concurrent INSERT + SELECT + VACUUM for 60 seconds

## Notes

- Graph repair is the hardest correctness problem. A deleted node's former neighbors need new connections to maintain graph connectivity. This is essentially "insert-like" neighbor finding for existing nodes.
- The landed pass-2 unlink slice intentionally scans all neighbor tuples instead of only the
  deleted node's outgoing adjacency. That catches asymmetric stale edges too.
- Multi-page repair writes now follow `spec/adr/ADR-027-vacuum-graph-repair-lock-ordering.md`.
- pgvector reference: `hnswvacuum.c` three-pass algorithm.
- Runtime scan and graph traversal still skip both `deleted` elements and empty-heaptid elements,
  but finalized dead tuples now carry `deleted = true` instead of relying only on the empty-heaptid
  intermediate state.
