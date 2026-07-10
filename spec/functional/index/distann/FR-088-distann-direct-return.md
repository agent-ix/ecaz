---
id: FR-088
title: Distann Direct-Mode Relay Return
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-086"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-084"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-085"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-088: Distann Direct-Mode Relay Return

## Description

In `batann_direct` mode, the terminal node delivers the final state directly
to the coordinator via `ec_distann_deliver_result(query_id,
epoch_fingerprint, status, state)`, which lands in a fresh backend on the
coordinator host and hands the payload to the waiting coordinator backend
through a fixed-size shared-memory result mailbox (ADR-086 D4). Under the
send-and-abandon forwarding variant, intermediate nodes do not hold the
relay chain; under the recorded direct-lite fallback (synchronous forward
acks), intermediate occupancy matches stack mode and only the result's
return trip through the chain is removed.

## Behavior

- When a direct-mode scan issues its initial relay, the coordinator SHALL
  first allocate a query_id from a 64-bit monotonic per-host shared-memory
  counter — query_ids are never reused within a postmaster lifetime, so a
  late delivery can never alias a newer scan's slot — register a mailbox
  slot recording its backend latch, and embed
  `(coordinator_node_id, query_id)` in the state (FR-085); it then waits on
  the latch with `CHECK_FOR_INTERRUPTS` and the
  `ec_distann.relay_wait_timeout_ms` bound (default 10000 ms).
- When the send-and-abandon flush guarantee holds (pre-B2 spike, ADR-086
  D4), intermediate forwarding SHALL free the forwarding backend promptly:
  the relay statement to the next node is issued without awaiting its
  response (the pooled connection is marked busy-until-drained and
  abandoned in-flight sends are capped, ADR-086 D4/D5). When the in-flight
  cap is hit, that forward SHALL degrade to a synchronous ack; a
  busy-until-drained connection whose drain errors or whose peer dies SHALL
  be evicted from the pool, never reused. Under the direct-lite fallback,
  forwarding is a synchronous ack and NFR-021's stack-mode occupancy bound
  applies; the NFR-022 gate packet SHALL record which variant ran.
- `ec_distann_deliver_result` SHALL validate the epoch fingerprint and the
  payload's structural bounds (FR-085), locate the mailbox slot by
  query_id, copy the payload (inline up to the configured cap; oversize →
  error status), set the waiting backend's latch, and return. EXECUTE on
  the function SHALL be revoked from PUBLIC and granted to the roster
  operator role (ADR-086 D11).
- **Delivery is at-most-once and delivery rights travel with the state**
  (ADR-086 D4): a node that has confirmed a downstream handoff SHALL NOT
  deliver anything for that query_id (it must not race its own downstream
  chain); a node whose forward outcome is indeterminate SHALL deliver
  nothing (the coordinator timeout backstops); a node that fails before a
  confirmed handoff SHALL deliver the failure — with its FR-079
  classification — to the mailbox. The first delivery to a slot wins;
  subsequent deliveries for the same query_id SHALL be dropped with a
  WARNING.
- **Wait timeout is a classified error, never a silent rerun** (NFR-020
  correct-or-error posture): when `relay_wait_timeout_ms` expires, the
  coordinator SHALL free the slot and raise a non-retriable relay-timeout
  error; it SHALL NOT automatically re-execute the search (a black-holed
  chain may still be running — a rerun would race its late delivery and
  double the expansion work outside the FR-082 two-attempt accounting).
- **Slot exhaustion**: when the mailbox has no free slot at registration,
  the scan SHALL fall back transparently to coordinator mode for that query
  (recorded in `fallback_resumed`/mode counters); registration SHALL never
  wait.
- Mailbox slot lifecycle: registered before the first send; freed by the
  coordinator on success or timeout; freed by a transaction-abort callback
  on cancel/abort. A delivery to an unknown or already-freed query_id SHALL
  be dropped with a WARNING (late deliveries are harmless by query_id
  non-reuse). Direct-mode cancel is coordinator-local (ADR-086 D10):
  in-flight remote drains run to completion bounded by the expansion and
  depth budgets, then their delivery is dropped.
- A debug introspection surface (`ec_distann_relay_mailbox_status()`,
  superuser/operator-gated) SHALL report slot states (free / waiting /
  filled, query_id, payload size) so drills can assert zero leaked slots.
- Direct mode is primary-only: the mailbox registration and delivery
  endpoint SHALL NOT be used on a hot standby (standby scans use
  coordinator or stack mode).
- Result equivalence: for identical query/params/budgets, direct mode SHALL
  return the same results as stack mode (the traversal is identical; only
  the return path differs).
- Materialization follows FR-087's rule (locally-owned INVALID-ctid hits
  re-resolve via the local directory).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-088-AC-1 | Terminal delivery wakes the waiting coordinator backend via the mailbox latch and the scan returns the delivered results | Test |
| FR-088-AC-2 | Direct-mode results equal stack-mode results for the same query/params on the multinode fixture | Test |
| FR-088-AC-3 | A failing relay node's error (each drill class) arrives at the coordinator via the mailbox with its original classification | Test (drill) |
| FR-088-AC-4 | Wait timeout fires when the terminal node is killed mid-drain (`debug_hold_relay_depth` pins the kill window); the slot is freed and the scan raises the non-retriable relay-timeout error — no automatic rerun | Test (drill) |
| FR-088-AC-5 | Slot lifecycle drills: freed on success, timeout, and coordinator cancel/abort; duplicate and post-free deliveries are dropped with a WARNING and corrupt nothing (`ec_distann_relay_mailbox_status()` asserts zero leaked slots) | Test (drill) |
| FR-088-AC-6 | Oversize payload is rejected per the configured cap with a delivered error status | Test |
| FR-088-AC-7 | Mailbox slot exhaustion at registration falls back transparently to coordinator mode with correct results and counter attribution | Test (drill) |

## Dependencies

- **Upstream**: [FR-086](./FR-086-distann-relay-endpoint-local-drain.md),
  [FR-085](./FR-085-distann-relay-state-wire-format.md),
  [FR-084](./FR-084-distann-coordination-mode-selection.md); ADR-086 D4/D10
- **Downstream**: [FR-089](./FR-089-distann-relay-depth-hybrid-resume.md),
  [NFR-021](../../../non-functional/NFR-021-distann-relay-resource-bounds.md),
  [NFR-022](../../../non-functional/NFR-022-distann-batann-mode-bench-gate.md)
