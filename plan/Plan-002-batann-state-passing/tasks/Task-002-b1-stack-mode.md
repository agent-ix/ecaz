---
id: Task-002
title: "B1 — stack-mode relay, cancellation enabler, kill-check gate"
type: Task
status: not_started
track: A
priority: P0
relationships:
  - target: ix://agent-ix/ecaz/Task-001
    type: depends_on
  - target: ix://agent-ix/ecaz/FR-086
    type: references
  - target: ix://agent-ix/ecaz/FR-087
    type: references
  - target: ix://agent-ix/ecaz/FR-089
    type: references
  - target: ix://agent-ix/ecaz/NFR-021
    type: references
  - target: ix://agent-ix/ecaz/TC-046
    type: verifies
---
# Task-002: B1 — stack-mode relay, cancellation enabler, kill-check gate

## Scope

Repo task `plan/tasks/175-batann-b1-stack-mode.md` (normative). Transport
relay wiring for the B0 endpoint (node→node, pooled, full mesh, per-hop
fingerprint), interrupt-sliced cancellation enabler (own slice, SPIRE
dispatch port — shared-path fix), FR-089 depth budget + terminal resume,
batann-scoped materialization fix, link-failure teardown, D11 EXECUTE
grants, relay counters. Gate G0.

## Subtasks

- [ ] Relay call over `remote_transport.rs` pool; session identity +
      `ec_distann_relay` application_name.
- [ ] Cancellation enabler slice (detect-inside, return, raise;
      CancelToken downstream) — lands as its own commit.
- [ ] FR-089: per-handoff decrement, incomplete-state return, terminal
      coordinator-mode resume, depth-0 equivalence.
- [ ] FR-087: unwind semantics, SQLSTATE preservation, link-failure
      classification, `fetch_remote_payloads` batann-scoped fix.
- [ ] TC-046 core drills incl. delta-buffer seam + full-mesh check.
- [ ] **G0 kill-check**: stack-vs-coordinator latency + relay-rate rows;
      recorded proceed/de-scope verdict against ADR-086.

## Deliverables

- Working batann_stack mode; packet `reviews/task-175/00N-*` with identity
  evidence, occupancy counts, and the G0 verdict.

## Notes

- Branch `task-175-batann-b1`. Do not start Task-003 before G0's verdict.
- Unblocks: Task-003, Task-004.
