---
id: FR-087
title: Distann Stack-Mode Relay Return
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-086"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-084"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-082"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-087: Distann Stack-Mode Relay Return

## Description

In `batann_stack` mode, a handoff SHALL be a synchronous nested SQL call:
node A calls `ec_distann_relay_search` on node B and blocks until B returns
the terminal state (B may itself relay to C, and so on). The final state
unwinds the call chain to the coordinator's original call (ADR-086 D3).

## Behavior

- The coordinator SHALL seed the state (head-index descent per FR-080), run
  its own local drain first, and issue the initial relay only when the drain
  hands off; the coordinator's call returns the terminal state.
- Each intermediate node SHALL return to its caller exactly the bytes
  returned by its downstream call (no re-drain on unwind).
- Error propagation SHALL preserve the FR-079 SQLSTATE classification
  through the chain: a retriable epoch-mismatch raised at any depth unwinds
  to the coordinator and triggers the single FR-082 restart; placement and
  structural faults unwind as non-retriable. Relay-specific faults (unknown
  state version, depth misuse) are non-retriable.
- Cancellation SHALL propagate down the chain: transport awaits are
  interrupt-sliced (detect inside `block_on`, return, then raise — the
  SPIRE dispatch pattern, ADR-086 D10) and a cancelled or timed-out
  statement at any hop cancels its downstream call (libpq cancel token),
  unwinding the chain. This transport change is shared: it also makes
  coordinator-mode `ec_distann_expand_nodes` calls cancellable (a
  pre-existing gap) and SHALL land as its own slice at B1.
- When the connection to a downstream hop is lost while that hop is
  draining (link failure, distinct from cancel/timeout), the sender SHALL
  attempt downstream cancellation via the retained cancel token, classify
  the fault in the FR-079 transport-error (non-retriable) class, and
  unwind; the orphaned sub-chain's residual work is bounded by the
  expansion and depth budgets and SHALL quiesce (NFR-021 zero-orphan
  drill).
- Stack mode has no dedicated wait GUC: the coordinator's
  `statement_timeout` (and any intermediate node's) is the operator wait
  bound for a hung chain, and the relay hop inherits the transport's
  interrupt-sliced timeout handling; this operator control SHALL be stated
  in the roster/transport operations documentation (NFR-021 sizing
  guidance).
- When a chain re-enters a node (A→B→A), the second visit SHALL land in a
  distinct backend over an ordinary pooled connection; no relay call ever
  waits on its own backend. Occupancy bounds are stated in
  [NFR-021](../../../non-functional/NFR-021-distann-relay-resource-bounds.md).
- Coordinator-side materialization SHALL accept relay-produced hits: when
  (and only when) the scan ran in a batann mode, a locally-owned hit
  arriving without a heap ctid (expanded on another node's drain) SHALL be
  re-resolved through the local directory. Coordinator-mode scans keep the
  existing FR-079 structural-fault classification for this case — the
  relaxation is scoped to relay-produced states, not the shared path.
- Fixture bar (ADR-086 D9a): stack mode SHALL return the same top-k as
  coordinator mode on the deterministic multinode fixture for identical
  query, BW, H, and k, under convergence-dominant termination (generous H,
  `early_exit` counter-asserted, seeded corpus with a deterministic
  distance tie-break at the k boundary); when the expansion budget binds,
  the identity assertion does not apply and the NFR-022 bench bar (D9b)
  governs instead.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-087-AC-1 | Multinode fixture under convergence-dominant termination (seeded corpus, fixed BW/H/k, `early_exit` asserted): stack-mode top-k identical to coordinator-mode top-k | Test |
| FR-087-AC-2 | Mid-chain epoch mismatch unwinds as retriable and the scan succeeds after the single restart | Test (drill) |
| FR-087-AC-3 | Coordinator cancel terminates all in-flight chain hops (no orphaned relay backends after the drill) | Test (drill) |
| FR-087-AC-4 | Mid-chain non-retriable fault (structural / injected via debug_fail_relay_depth) surfaces at the coordinator with its original classification | Test (drill) |
| FR-087-AC-5 | Locally-owned hits produced by remote drains materialize via local-directory re-resolution (no structural-fault error) | Test |

## Dependencies

- **Upstream**: [FR-086](./FR-086-distann-relay-endpoint-local-drain.md),
  [FR-084](./FR-084-distann-coordination-mode-selection.md),
  [FR-082](./FR-082-distann-epoch-lifecycle.md); ADR-086 D3/D9/D10
- **Downstream**: [FR-089](./FR-089-distann-relay-depth-hybrid-resume.md),
  [NFR-021](../../../non-functional/NFR-021-distann-relay-resource-bounds.md),
  [NFR-022](../../../non-functional/NFR-022-distann-batann-mode-bench-gate.md)
