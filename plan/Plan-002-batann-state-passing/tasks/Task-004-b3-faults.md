---
id: Task-004
title: "B3 — cross-cutting fault matrix + resource-bound drills"
type: Task
status: not_started
track: A
priority: P0
relationships:
  - target: ix://agent-ix/ecaz/Task-002
    type: depends_on
  - target: ix://agent-ix/ecaz/Task-003
    type: depends_on
  - target: ix://agent-ix/ecaz/FR-087
    type: references
  - target: ix://agent-ix/ecaz/FR-088
    type: references
  - target: ix://agent-ix/ecaz/NFR-021
    type: references
  - target: ix://agent-ix/ecaz/TC-046
    type: verifies
  - target: ix://agent-ix/ecaz/TC-047
    type: verifies
---
# Task-004: B3 — cross-cutting fault matrix + resource-bound drills

## Scope

Repo task `plan/tasks/177-batann-b3-faults-lifecycle.md` (normative).
Fixture fault orchestration in `distann_multicluster.rs` and the NFR-021
evidence rows, both modes, real 3-instance topology. Deliberately scoped:
cancel landed at B1, mailbox lifecycle at B2.

## Subtasks

- [ ] Drills: mid-chain republish (restart-once), killed terminal node
      (held window), link failure, forward-connect failure,
      `debug_fail_relay_depth` depth matrix, drain hygiene.
- [ ] NFR-021 rows: occupancy ≤ depth+1 at held peak (relay-tagged
      backends per instance), bounded settle-poll zero-orphan, state-bytes
      envelope, expansion cap per mode.
- [ ] Force-retire mid-flight → retriable restart (FR-086 retention-gate
      interaction).

## Deliverables

- Green fault matrices (TC-046/047 fault rows); NFR-021 measurement table
  produced packet-locally; packet `reviews/task-177/00N-*`.

## Notes

- Branch `task-177-batann-b3`.
- Unblocks: Task-005.
