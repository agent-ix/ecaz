---
id: Task-002
title: "M1 — sharded clustered build with closure overlap + stitch"
type: Task
status: not_started
track: A
priority: P0
relationships:
  - target: ix://agent-ix/ecaz/Task-001
    type: depends_on
  - target: ix://agent-ix/ecaz/FR-077
    type: references
  - target: ix://agent-ix/ecaz/TC-038
    type: verifies
  - target: ix://agent-ix/ecaz/TC-039
    type: verifies
---
# Task-002: M1 — sharded clustered build with closure overlap + stitch

## Scope

Repo task `plan/tasks/163-ec-distann-m1-stitch.md` (normative). FR-077:
sharded closure-overlap build + stitch pass, seed-deterministic end-to-end;
build emits the co-placed vector tier alongside records.

## Subtasks

- [ ] **Closure-overlap sharding.** Repurpose the SPIRE closure generator
      (`src/am/ec_spire/build/routing_plan.rs:132-250`, `closure_epsilon`
      option) to build-time shard assignment.
- [ ] **Per-shard Vamana + stitch pass.** Degree ≤ R, vec_id uniqueness,
      medoid reachability, idempotent stitch, α-prune invariant preserved
      (FR-077-CON-1..3).
- [ ] **Determinism.** Same seed ⇒ identical graph (the M2 result-identity
      test depends on this).
- [ ] **TC-038 proptest suite** in `src/am/ec_distann/`.
- [ ] **TC-039 bench A/B.** Stitched vs monolithic distinct_recall at 100k
      within 0.001 (`ecaz bench suite`).
- [ ] **Epoch manifest rows.** Duplication factor, stitch stats, peak memory.

## Deliverables

- Stitch build path + property suite; packet `reviews/task-163/00N-*` with
  the 100k A/B evidence and manifest rows.

## Notes

- Branch `task-163-ec-distann-m1`. Single-node still — no placement/remote.
- Unblocks: Task-003 (M2 consumes the deterministic stitched build).
