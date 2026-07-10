---
id: FR-089
title: Distann Relay Depth Budget and Hybrid Resume
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-086"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-081"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-085"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-089: Distann Relay Depth Budget and Hybrid Resume

## Description

Relaying SHALL be bounded by `ec_distann.relay_max_depth`, decremented per
handoff. When a drain wants to hand off but the depth budget is exhausted,
the current node SHALL mark the state `incomplete` and return/deliver it; the
coordinator SHALL resume the same state in the coordinator-mode hop loop
(FR-081) with the remaining BW×H budget (ADR-086 D6). The relay depth budget
therefore bounds uncoordinated work without ever weakening completeness.

## Behavior

- `relay_depth_remaining` travels in the state (FR-085) and SHALL be
  decremented exactly once per handoff, on the sending side.
- When the depth budget is exhausted at a pending handoff, the state SHALL
  be flagged `incomplete` and returned via the active return mode (stack:
  unwound untouched through intermediates; direct: delivered to the mailbox
  with an `incomplete` status).
- The coordinator SHALL deserialize an incomplete state and continue it in
  the FR-081 grouped hop loop — same beam, same visited set, same hits, same
  remaining expansion budget (the expansion count is the authoritative
  bound, FR-085; remaining rounds are derived from it). The resume is
  **terminal coordinator mode**: no further handoffs occur within the
  attempt, so total handoffs per attempt are bounded by `relay_max_depth`.
  All FR-081 completeness and termination guarantees SHALL hold across the
  splice (they are properties of the state).
- Default `relay_max_depth` = min(effective hop-round budget H, 16)
  (ADR-086 D6; H alone is unsafe at the shipped H default of 100;
  revisited with B4 relay-rate evidence).
- `relay_max_depth = 0` SHALL suppress relaying entirely: the scan is
  coordinator-mode in all but name (FR-084-AC-4 equivalence).
- The `fallback_resumed` counter SHALL record whether a scan spliced back to
  coordinator mode, and the depth histogram SHALL record relay depths
  reached.
- CustomScan deepen-on-demand re-runs (iterative deepening) SHALL each be
  fresh relay journeys with fresh depth and expansion budgets — a journey
  is one execution of the search loop; each deepening re-run is a new
  FR-081/NFR-019 attempt with its own accounting, exactly as in coordinator
  mode — counted in the `relay_journeys` counter (FR-084).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-089-AC-1 | Depth decrements exactly once per handoff (counter vs relay trace on the fixture) | Test |
| FR-089-AC-2 | Depth-exhausted scans return an incomplete state and the coordinator resumes it to a complete result | Test (drill) |
| FR-089-AC-3 | Results after a depth-exhaustion resume equal coordinator-mode results for the same query/params | Test |
| FR-089-AC-4 | Total expansions across relay drains plus the resumed coordinator loop ≤ BW×H | Test (counter assertion) |
| FR-089-AC-5 | `fallback_resumed` and `relay_depth_histogram` are reported in the counter surface | Inspection |
| FR-089-AC-6 | A resumed attempt performs no further handoffs (relay counters flat after `fallback_resumed`), and deepen-on-demand re-runs increment `relay_journeys` with fresh budgets | Test (counter assertion) |

## Dependencies

- **Upstream**: [FR-086](./FR-086-distann-relay-endpoint-local-drain.md),
  [FR-085](./FR-085-distann-relay-state-wire-format.md),
  [FR-081](./FR-081-distann-query-orchestration.md); ADR-086 D6/D8
- **Downstream**: [NFR-021](../../../non-functional/NFR-021-distann-relay-resource-bounds.md),
  [NFR-022](../../../non-functional/NFR-022-distann-batann-mode-bench-gate.md)
