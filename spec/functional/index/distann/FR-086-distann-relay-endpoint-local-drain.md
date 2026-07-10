---
id: FR-086
title: Distann Relay Endpoint and Local Drain
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-085"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-079"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-078"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-080"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/NFR-014"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-086: Distann Relay Endpoint and Local Drain

## Description

Every node SHALL expose a relay endpoint `ec_distann_relay_search(index,
state) RETURNS bytea` that resumes a received query state with a **local
drain** implementing BatANN Algorithm 2: expand all locally-owned candidates
among the top-BW frontier before handing off, and hand off only when the
entire top-BW frontier is remote-owned — to the node owning the best
unexpanded candidate.

## Behavior

- When a relay state is received, the endpoint SHALL validate the epoch
  fingerprint, the state version, and the state's structural bounds
  (FR-085) before any index read. A state arriving with
  `relay_depth_remaining = 0` is valid — the receiver drains locally and
  flags `incomplete` only at the next pending handoff (FR-089); depth is
  never a receive-time rejection.
- EXECUTE on `ec_distann_relay_search` SHALL be revoked from PUBLIC and
  granted to the roster operator role (ADR-086 D11).
- The local drain loop SHALL, per round: sort the frontier by code distance;
  take the best BW unexpanded candidates; apply the FR-081 convergence
  early-exit test against the carried hits; partition the top-BW frontier by
  owning node (FR-078, computed locally from the shared roster);
  - if any top-BW candidates are locally owned: expand **all** locally-owned
    ones in one local batch (same expansion semantics as
    `ec_distann_expand_nodes` — code-score neighbors from embedded codes,
    exact distance from the co-placed heap row, tombstone handling per
    FR-079), merge into the state, decrement the global budget, continue;
  - if the entire top-BW frontier is remote: hand off — serialize the state
    and relay it to the owner of the best unexpanded candidate, decrementing
    `relay_depth_remaining` (FR-089).
- Drain termination SHALL be exactly FR-081's rules evaluated on the carried
  state: convergence early-exit, beam exhaustion, or budget exhaustion — in
  each case the state is complete and returns (per the active return mode)
  rather than handing off.
- Every expansion in every drain SHALL decrement the single expansion
  budget carried in the state (ADR-086 D8; the expansion count is the
  authoritative bound, FR-085); the visited-set invariant (no vec_id
  expanded twice per scan attempt) SHALL hold across all nodes that drain
  the query.
- Progress guarantee: every handoff target owns the best unexpanded
  frontier candidate, so each relay hop expands at least one record before
  any further handoff — A↔B ping-pong is legal but always makes progress
  and is bounded by the depth budget.
- Relay states are invisible to the FR-082 node-local retention gate
  (`in_flight_count` tracks only local scans): epoch safety for in-flight
  relayed queries rests entirely on the per-hop fingerprint check failing
  closed after a retire/reclaim. A force-retire on a remote node mid-flight
  therefore surfaces as the retriable epoch-mismatch class and restarts
  once from the coordinator.
- Relay nodes SHALL NOT perform head-index descent (FR-080 seeding already
  happened on the coordinator; the state arrives seeded). Relay nodes MAY
  load the index cache entry for directory/codebook access, as the expand
  endpoint already does.
- Relay calls SHALL use the same pooled node-to-node transport and session
  identity discipline as FR-079 (ADR-086 D5); every node can reach every
  other node in the roster (full mesh via shared roster).
- Ownership discipline: a relay endpoint SHALL NOT expand vec_ids it does
  not own; structural faults (owned-but-absent record, missing co-placed
  vector) keep their FR-079 classifications and SQLSTATEs.
- An **attempt** is one execution of the search between FR-082
  epoch-mismatch restarts (max two per scan, per NFR-019); the dedupe and
  budget invariants in this FR are per-attempt, across every node that
  drains the attempt.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-086-AC-1 | Epoch fingerprint is validated on every relay hop before any index read (drill: republish mid-chain → retriable mismatch) | Test |
| FR-086-AC-2 | While any top-BW frontier candidate is locally owned, no handoff occurs (drain-all-local-first, counter-asserted on the fixture) | Test |
| FR-086-AC-3 | Handoff target is the owner of the best unexpanded frontier candidate | Test |
| FR-086-AC-4 | Total records expanded across all drains of one attempt ≤ BW×H; no vec_id expanded twice per attempt | Test (counter assertion) |
| FR-086-AC-5 | Single-node relay identity: `ec_distann_relay_search` on one node reproduces the local scan routine's results exactly for the same query/params | Test |
| FR-086-AC-6 | Relay nodes perform no head-index descent (counter/inspection) | Test |

## Dependencies

- **Upstream**: [FR-085](./FR-085-distann-relay-state-wire-format.md),
  [FR-079](./FR-079-distann-remote-expansion-protocol.md),
  [FR-078](./FR-078-distann-hash-placement.md),
  [FR-080](./FR-080-distann-coordinator-head-index.md); ADR-086 D5/D8;
  transport posture [NFR-014](../../../non-functional/NFR-014-spire-transport-security-and-operations.md)
- **Downstream**: [FR-087](./FR-087-distann-stack-return.md),
  [FR-088](./FR-088-distann-direct-return.md),
  [FR-089](./FR-089-distann-relay-depth-hybrid-resume.md)
