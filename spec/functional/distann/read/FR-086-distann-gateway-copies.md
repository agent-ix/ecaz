---
id: FR-086
title: DistANN Bounded Gateway Copies
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-079"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-080"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-086: DistANN Bounded Gateway Copies

## Description

The coordinator MAY cache the routing payload — neighbor vec_ids and
quantized neighbor codes, never full-precision vectors and never the row
tier — of a bounded set of gateway nodes (the TRAV-30 mechanism, Task 210
P3). With gateway copies populated, owners omit the neighbor payload for
gateway-cached ids in expansion responses and the coordinator reconstructs
that candidate half locally, removing per-hop response bytes without moving
corpus-proportional state to the coordinator. The gateway set is the
[FR-080](./FR-080-distann-coordinator-head-index.md) head membership:
already bounded by C and expanded first by every scan.

## Behavior

- **Bound.** Capacity SHALL be the `ec_distann.gateway_copy_capacity`
  session GUC (0..65536, default 0 = disabled), a stated constant
  independent of corpus size. Inserts past capacity SHALL be refused, not
  evicted, so the structure cannot exceed its bound under any traversal
  pattern. When the head membership exceeds capacity, the set holds a
  refusal-bounded subset.
- **Content.** Each entry SHALL hold exactly `(vec_id, tombstone flag,
  neighbor vec_ids, quantized neighbor codes)` — the
  [FR-079](./FR-079-distann-remote-expansion-protocol.md) candidate half.
  Full-precision vectors, row payloads, and exact distances SHALL NOT be
  cached.
- **Population.** The set SHALL be populated per epoch from the head
  membership via bounded owner batch RPCs
  (`ec_distann_gateway_routing_export`); owners remain the source of truth.
  The set is epoch-fingerprint-scoped and rebuildable; an epoch flip
  discards it. A capacity GUC change SHALL discard and repopulate the set
  (staleness rule: a cached subset chosen under a different bound is not
  reusable).
- **Serving.** When the coordinator holds gateway copies for requested ids,
  the expansion request SHALL name them (`skip_neighbor_vec_ids`) and the
  owner SHALL omit those rows' neighbor payloads; exact distances and
  tombstone authority still come from the owner. The coordinator SHALL
  reconstruct the candidate half from its copies and re-apply the
  [FR-081](./FR-081-distann-query-orchestration.md) batch candidate limit
  once across the merged batch, preserving owner-only result equivalence.
- **Fallback.** A missing, unpopulated, or capacity-refused entry SHALL
  fall back to the full owner response for that id — identical results,
  larger response. Gateway copies narrow the distributed protocol; they
  SHALL NOT substitute for it (NFR-021 clause 4 boundary: this is a bounded
  routing cache, not a graph replica).
- **Observability.** The extension SHALL expose activation counters for
  gateway-copy serving (`ec_distann_gateway_copy_stats`); an A/B arm
  claiming gateway-copy effect SHALL show non-zero activation.

## Flows

Population and skip-mask serving:

```mermaid
sequenceDiagram
    participant S as scan backend (coordinator)
    participant G as gateway-copy set (bounded)
    participant O as owner

    Note over S,G: per epoch, capacity > 0
    S->>O: ec_distann_gateway_routing_export(head member ids)
    O-->>S: routing payloads (neighbor ids + codes, no vectors)
    S->>G: insert up to capacity (refusal past bound)

    Note over S,O: each hop round
    S->>O: expand(vec_ids, skip_neighbor_vec_ids = cached ids)
    O-->>S: exact dists + tombstones; neighbor payload omitted for skipped ids
    S->>G: reconstruct candidate half for skipped ids
    S->>S: re-apply batch candidate limit across merged batch
    Note over S: miss / refused id ⇒ full owner response (identical results)
```

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-086-CON-1 | Resident gateway-copy bytes SHALL be bounded by capacity × (degree × (8 + code bytes) + entry header), independent of N | Memory | Analysis + unit test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-086-AC-1 | With gateway copies on, per-query results (ordered hits, distances, tombstone handling) are identical to the owner-only path | Test |
| FR-086-AC-2 | Inserts past capacity are refused; the set never exceeds its stated bound | Test |
| FR-086-AC-3 | An epoch flip or capacity GUC change discards the set; no cross-epoch or cross-capacity reuse | Test |
| FR-086-AC-4 | No full-precision vector or row payload is ever resident in the gateway set | Inspection + test |
| FR-086-AC-5 | Response-byte reduction and activation counters are measured per arm at 10k/50k/100k | Analysis (bench) |

## Dependencies

- **Upstream**: [FR-079](./FR-079-distann-remote-expansion-protocol.md)
  (expansion wire), [FR-080](./FR-080-distann-coordinator-head-index.md)
  (head membership as the gateway set),
  [FR-082](../lifecycle/FR-082-distann-epoch-lifecycle.md) (epoch scoping).
- **Downstream**:
  [NFR-021](../../../non-functional/NFR-021-distann-distribution-invariant.md)
  (bounded-structure conformance); ADR-087 (selection of TRAV-30 over the
  FR-084 replica); Task 212 crown cache (builds on the same bounded
  codes-only class).

## Measured outcome (Task 210 P3)

`reviews/task-210/004-gateway-copies/`: response bytes −36% @10k, −9% @50k,
−7% @100k with identical recall; semantics-preservation proof in
`gateway_copy.rs` unit tests (owner-only equivalence including batch-limit
re-application).
