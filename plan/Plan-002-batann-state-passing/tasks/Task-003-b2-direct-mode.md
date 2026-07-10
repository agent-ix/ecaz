---
id: Task-003
title: "B2 — direct return: flush spike, shmem mailbox, deliver endpoint"
type: Task
status: not_started
track: A
priority: P1
relationships:
  - target: ix://agent-ix/ecaz/Task-002
    type: depends_on
  - target: ix://agent-ix/ecaz/FR-088
    type: references
  - target: ix://agent-ix/ecaz/NFR-021
    type: references
  - target: ix://agent-ix/ecaz/TC-047
    type: verifies
---
# Task-003: B2 — direct return: flush spike, shmem mailbox, deliver endpoint

## Scope

Repo task `plan/tasks/176-batann-b2-direct-mode.md` (normative). Gate G1
(pre-implementation flush spike; direct-lite fallback), fixed-slot shmem
mailbox + monotonic query_id allocator, `ec_distann_deliver_result` with
at-most-once delivery-rights semantics, timeout = classified error,
slot-exhaustion coordinator-mode fallback, mailbox lifecycle drills,
`ec_distann_relay_mailbox_status()`.

## Subtasks

- [ ] **G1 flush spike** (timeboxed, before mailbox work); verdict recorded
      against ADR-086 D4.
- [ ] Shmem: fixed slot array, `_PG_init` wiring, PGPROC latch wakeup,
      xact-abort slot cleanup; first-of-kind — budget API discovery.
- [ ] Deliver endpoint: fingerprint + structural validation,
      first-delivery-wins, WARNING drops, oversize error, D11 grants,
      primary-only.
- [ ] Forwarding per verdict (send-and-abandon busy-until-drained /
      cap-degrade / evict-on-error, or direct-lite).
- [ ] TC-047 happy paths + slot lifecycle drills; stack≡direct check.

## Deliverables

- Working batann_direct mode (variant recorded); packet
  `reviews/task-176/00N-*`.

## Notes

- Branch `task-176-batann-b2`. Entry gated on Task-002's G0 verdict.
- Unblocks: Task-004 completion, Task-005.
