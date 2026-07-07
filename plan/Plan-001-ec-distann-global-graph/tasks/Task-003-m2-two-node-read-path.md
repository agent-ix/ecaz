---
id: Task-003
title: "M2 — two-node read path (placement, expand protocol, remote hop-rounds)"
type: Task
status: not_started
track: A
priority: P0
relationships:
  - target: ix://agent-ix/ecaz/Task-002
    type: depends_on
  - target: ix://agent-ix/ecaz/FR-078
    type: references
  - target: ix://agent-ix/ecaz/FR-079
    type: references
  - target: ix://agent-ix/ecaz/FR-081
    type: references
  - target: ix://agent-ix/ecaz/FR-082
    type: references
  - target: ix://agent-ix/ecaz/NFR-019
    type: references
  - target: ix://agent-ix/ecaz/TC-040
    type: verifies
  - target: ix://agent-ix/ecaz/TC-041
    type: verifies
---
# Task-003: M2 — two-node read path

## Scope

Repo task `plan/tasks/164-ec-distann-m2-two-node-read-path.md` (normative).
Hash placement with co-placed heap rows (FR-078), `ec_distann_expand_nodes`
(FR-079, exact_dist from the co-located local heap read, four-outcome
table), remote FR-081 hop-round loop against the M0-frozen seam, epoch
fingerprint validation subset of FR-082, build→publish hand-off. Gate G1.

## Subtasks

- [ ] **FR-078 placement.** hash(vec_id) ownership; record + full-precision
      heap row co-placed on the owning node; placement directory (adapt
      `ec_spire/meta/{epoch,placement_directory}.rs`, don't share); every
      placement disagreement = error, never a silent miss.
- [ ] **FR-079 expand protocol.** Wire contract independent of record layout
      (D1-fallback-safe); four outcomes: present / not-owned /
      owned-but-absent (fault c) / vector-missing (fault d); tombstones MAY
      return NULL exact_dist. Transport: pooled async dispatch pattern
      (`ec_spire/coordinator/remote_candidates/dispatch.rs:587-1305`).
- [ ] **Remote FR-081 loop.** Coordinator-local head index seeds; H batched
      hop-rounds; results ONLY from expanded records; sub-k on beam
      exhaustion is a complete result; BW×H per attempt, max 2 attempts
      (epoch-mismatch restart).
- [ ] **Epoch fingerprint subset.** Publication tuple + fingerprint
      attestation on the read path (FR-082 subset).
- [ ] **TC-040 / TC-041** pg_tests + 2-node fixture; per-query expansion
      ≤ BW×H asserted (NFR-019); EXPLAIN counters.
- [ ] **Gate G1 measurement.** 2-node vs 1-node latency delta; hop-RTT share
      (D4 baton-passing reopen trigger: RTT ≥ 50% of multinode p50).

## Deliverables

- Two-node read path; packet `reviews/task-164/00N-*` with identity
  evidence, cap assertions, and the D4 trigger measurement.

## Notes

- Branch `task-164-ec-distann-m2`. Do not start before Gate G0 passes.
- Unblocks: Task-004.
