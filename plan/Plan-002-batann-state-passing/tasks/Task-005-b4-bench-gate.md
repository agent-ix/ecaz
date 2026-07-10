---
id: Task-005
title: "B4 — three-way coordination-mode bench gate"
type: Task
status: not_started
track: A
priority: P0
relationships:
  - target: ix://agent-ix/ecaz/Task-004
    type: depends_on
  - target: ix://agent-ix/ecaz/Task-006
    type: depends_on
  - target: ix://agent-ix/ecaz/NFR-021
    type: references
  - target: ix://agent-ix/ecaz/NFR-022
    type: references
  - target: ix://agent-ix/ecaz/TC-048
    type: verifies
---
# Task-005: B4 — three-way coordination-mode bench gate

## Scope

Repo task `plan/tasks/178-batann-b4-bench-gate.md` (normative). Gate G2:
pre-registered coordinator / batann_stack / batann_direct matrix at
10k/50k/100k per NFR-022 on the real multi-instance topology; D9b one-sided
recall bar; relay counters + pre-registered relay-rate formula; pinned
reduced-depth row; D7 hash-placement finding; promote/iterate/shelve
verdict into ADR-086.

## Subtasks

- [ ] Packet suite config (mode × scale × recall/latency; storage once per
      scale); prerequisites recorded (task-165 posture, Task-006 SHA,
      task-172 protocol packet).
- [ ] Run matrix on the Intel bench host, release-verified.
- [ ] Gate packet: D9b rows, p50/p95 per mode, counter rows, D7 finding,
      direct-mode variant, verdict written into ADR-086 status.

## Deliverables

- NFR-022 gate packet `reviews/task-178/00N-*`; ADR-086 status verdict.

## Notes

- Branch `task-178-batann-b4`. Program gate; 1m encouraged if 100k shows
  promise.
