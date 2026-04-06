# Task 07: Vacuum Three-Pass

Status: blocked on Task 05 (A2 traversal), Task 06 (neighbor pruning)

Progress notes:
- Vacuum callbacks are currently benign no-ops returning page/tuple stats.

## Scope

Implement ambulkdelete with three-pass delete algorithm and amvacuumcleanup with statistics update.

## Subtasks

- [ ] **Pass 1 — Mark.** Scan element tuples, compare heap TIDs against dead-tuple bitmap, remove dead TIDs from element tuples, build hash table of fully-dead elements.
- [ ] **Pass 2 — Graph repair.** For each broken connection (deleted neighbor), search for replacement neighbors using A2 traversal with code-to-code scoring. Reuse neighbor selection/pruning logic from Task 06.
- [ ] **Pass 3 — Finalize.** Set `deleted = true` on fully-dead element tuples.
- [ ] **amvacuumcleanup.** Update pg_class.reltuples and relpages.
- [ ] **Concurrency validation.** Vacuum must not block concurrent INSERT or SELECT.

## Owns

- `FR-010`

## Dependencies

- Task 05 subtask A2 (graph traversal for repair search)
- Task 06 (neighbor selection and pruning logic)

## Unblocks

- Task 10 (post-vacuum recall and quality benchmarks)

## Deliverables

- Three-pass `ambulkdelete`
- `amvacuumcleanup` with statistics
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
