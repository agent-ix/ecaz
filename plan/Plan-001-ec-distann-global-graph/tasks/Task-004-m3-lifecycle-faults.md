---
id: Task-004
title: "M3 — epoch lifecycle + fault drills"
type: Task
status: not_started
track: A
priority: P0
relationships:
  - target: ix://agent-ix/ecaz/Task-003
    type: depends_on
  - target: ix://agent-ix/ecaz/FR-082
    type: references
  - target: ix://agent-ix/ecaz/FR-083
    type: references
  - target: ix://agent-ix/ecaz/NFR-020
    type: references
  - target: ix://agent-ix/ecaz/TC-042
    type: verifies
---
# Task-004: M3 — epoch lifecycle + fault drills

## Scope

Repo task `plan/tasks/165-ec-distann-m3-lifecycle-faults.md` (normative).
FR-082 full lifecycle on a 3-worker fixture (Building → atomic publication →
immutable published epoch → retirement reclaim), FR-083 early DML slices
live, TC-042 drill matrix, NFR-020 correct-or-error posture.

## Subtasks

- [ ] **FR-082 full.** Heap tier under the epoch lifecycle: assembled in
      Building, in the publication tuple, immutable within a published epoch
      (D10 — tombstone flags only; reclaim + edge repair at next epoch
      build), fingerprint-attested, reclaimed at retirement gated on
      in-flight count (AC-6 wedged-count operator override).
- [ ] **FR-083 early.** Tombstone delete + delta-buffer insert wired to the
      read path.
- [ ] **Multinode fixture + TC-042 drills.** hop_round_failure_mid_beam,
      missing_node_record, placement_drift, missing_heap_row,
      coplacement_drift, mid-delete; epoch swap under load; FR-082-AC-5
      concurrent delete+VACUUM+TID-reuse never reranks a different tuple.
- [ ] **Recall floor.** 50k multinode recall ≥ single-node − 0.001.

## Deliverables

- Lifecycle + drills; packet `reviews/task-165/00N-*` with the drill matrix
  results and recall evidence.

## Notes

- Branch `task-165-ec-distann-m3`. Epoch mismatch = discard partial state,
  one full restart under refreshed epoch, then error.
- Unblocks: Task-005, Task-006.
