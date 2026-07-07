---
id: Task-006
title: "M5 — incremental insert (committed scope)"
type: Task
status: not_started
track: A
priority: P1
relationships:
  - target: ix://agent-ix/ecaz/Task-004
    type: depends_on
  - target: ix://agent-ix/ecaz/Task-005
    type: depends_on
  - target: ix://agent-ix/ecaz/FR-083
    type: references
  - target: ix://agent-ix/ecaz/TC-043
    type: verifies
---
# Task-006: M5 — incremental insert

## Scope

Repo task `plan/tasks/167-ec-distann-m5-incremental-insert.md` (normative).
FR-083 full: write endpoint, distributed self-insertion (insert emits +
co-places the vector on the owning node), back-edge RMW + re-prune,
vec_id collision error, UPDATE semantics.

## Subtasks

- [ ] **Write endpoint + distributed self-insertion.** Placement per
      FR-078; co-placed vector at insert (FR-083-AC-7: inserted vec_id
      reads its vector node-locally).
- [ ] **Back-edge RMW + re-prune.** α-prune invariant maintained under
      concurrent inserts.
- [ ] **Collision + UPDATE semantics.** Insert-time vec_id collision errors
      (ADR-063 identity via ADR-068); UPDATE = delete + insert posture.
- [ ] **TC-043.** Mid-insert + concurrency drills; FR-083-AC-4 bench cell:
      insert-then-query distinct_recall parity with a fresh rebuild.

## Deliverables

- Insert path + drills; packet `reviews/task-167/00N-*` with the parity
  bench cell.

## Notes

- Branch `task-167-ec-distann-m5`. Sequenced after the G2 verdict; committed
  scope regardless of promote/iterate (only shelve stops it).
