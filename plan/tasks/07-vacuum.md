# Task 07: Vacuum Three-Pass

Status: in progress on `main`

Progress notes:
- A5 is complete on `main`, so the shared traversal and neighbor-pruning dependencies are no
  longer blockers.
- Pass 1 is now landed: `ambulkdelete` strips dead heap TIDs from element tuples, and
  `amvacuumcleanup` reports live-element counts instead of raw tuple tags.
- This first slice intentionally stops short of graph repair and finalization; fully dead elements
  currently end pass 1 as `heaptids = []` with `deleted = false`, which the scan/runtime path
  already treats as unreachable.

## Scope

Implement ambulkdelete with three-pass delete algorithm and amvacuumcleanup with statistics update.

## Subtasks

- [x] **Pass 1 — Mark.** Scan element tuples, compare heap TIDs against the vacuum callback,
  remove dead TIDs from element tuples, and persist the compacted heaptid payload in place.
- [ ] **Pass 2 — Graph repair.** For each broken connection (deleted neighbor), search for replacement neighbors using A2 traversal with code-to-code scoring. Reuse neighbor selection/pruning logic from Task 06.
- [ ] **Pass 3 — Finalize.** Set `deleted = true` on fully-dead element tuples.
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
- pgvector reference: `hnswvacuum.c` three-pass algorithm.
- Pass 1 already gives correct scan invisibility for fully-dead elements because runtime scan and
  graph traversal skip `deleted` elements and also skip any element whose `heaptids` array is empty.
