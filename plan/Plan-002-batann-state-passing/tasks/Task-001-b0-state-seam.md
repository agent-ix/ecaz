---
id: Task-001
title: "B0 — beam-state seam, relay-state serde, local relay identity"
type: Task
status: not_started
track: A
priority: P0
relationships:
  - target: ix://agent-ix/ecaz/FR-084
    type: references
  - target: ix://agent-ix/ecaz/FR-085
    type: references
  - target: ix://agent-ix/ecaz/FR-086
    type: references
  - target: ix://agent-ix/ecaz/TC-045
    type: verifies
---
# Task-001: B0 — beam-state seam, relay-state serde, local relay identity

## Scope

Repo task `plan/tasks/174-batann-b0-state-seam.md` (normative). Pure
refactor of `scan.rs:distann_orchestrated_search` into
`DistannBeamState` + `distann_local_drain` (both read paths via
`collect_distann_hits`), DISTANN_RELAY_STATE_V1 serde with structural
validation, FR-084 GUC surface + counter taxonomy, local-only
`ec_distann_relay_search`.

## Subtasks

- [ ] Extract state + drain; preserve kth `select_nth_unstable` reordering,
      early-exit position, `debug_fail_hop_round` injection order.
- [ ] Expansion budget authoritative; rounds derived (FR-085).
- [ ] `relay_state.rs` encode/decode + FR-085-AC-6 structural rejection.
- [ ] GUCs: coordination_mode, relay_max_depth (min(H,16)),
      relay_wait_timeout_ms (10000), debug_fail/hold_relay_depth,
      debug_relay_trace_notice; `application_name` tag; counter stubs.
- [ ] Local-only relay endpoint; single-node relay identity test.
- [ ] TC-045 suite incl. append-only-beam invariant guard.

## Deliverables

- Refactor + serde + GUC surface; packet `reviews/task-174/00N-*` with
  refactor-parity evidence (existing distann tests green).

## Notes

- Branch `task-174-batann-b0`. Freezes the state field set and counter
  taxonomy for B1+.
- Unblocks: Task-002.
