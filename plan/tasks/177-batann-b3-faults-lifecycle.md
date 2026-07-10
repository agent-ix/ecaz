# Task 177: BatANN B3 — Cross-Cutting Fault Matrix and Resource-Bound Drills

Status: proposed (2026-07-09). Depends on: Tasks 175, 176.
Owner: coder (to be assigned). One coder, one branch.
Priority: P0 — NFR-020/NFR-021 evidence for both relay modes.

## Why

NFR-021's zero-leak and occupancy bounds and the FR-086/087/088 fault
classifications are only claims until drilled on the real multi-instance
fixture. B3 is deliberately scoped to the cross-cutting matrix (cancel
landed at B1, mailbox lifecycle at B2) so it stays tractable.

## Goal

Every relay fault class drilled, classified, and leak-free on the
3-instance fixture, in both modes.

## Scope

- Fixture fault orchestration in `distann_multicluster.rs`: mid-chain
  republish (epoch mismatch → restart-once), killed terminal node
  (`debug_hold_relay_depth` pins the window), link-failure sub-chain
  teardown, forward-connect failure, `debug_fail_relay_depth` matrix at
  0-based depths, busy-until-drained hygiene / evict-on-error.
- NFR-021 evidence rows: occupancy ≤ depth+1 at held peak
  (relay-tagged backends per instance), bounded settle-poll zero-orphan
  assertions, state-bytes envelope counter assertion, expansion cap in
  every mode.
- Retention-gate interaction: force-retire on a remote node mid-flight →
  retriable restart (FR-086).

## Required Evidence

TC-046/TC-047 fault-matrix rows green on the real 3-instance fixture;
NFR-021 measurement table rows produced packet-locally.

## Non-Goals

Bench gate (178); any new relay features.

## Acceptance Criteria

1. Full drill matrix green in both batann modes; FR-088-AC-4 completed.
2. NFR-021 rows: zero orphans/undrained/leaked-slots after every drill.
3. Fault classifications match FR-079 classes end-to-end.

## References

- NFR-020, NFR-021; FR-086/087/088 fault clauses; ADR-086 D10
- `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs`
