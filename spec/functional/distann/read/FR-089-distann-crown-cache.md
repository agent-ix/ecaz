---
id: FR-089
title: DistANN Crown Cache
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-080"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-086"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-089: DistANN Crown Cache

## Description

The coordinator MAY hold a crown: a fixed-capacity navigation cache over a
subset of head landmarks, at per-landmark granularity, holding
`(vec_id, quantized code)` entries — routing payload of the same
[FR-086](./FR-086-distann-gateway-copies.md) bounded codes-only class,
never a full-precision vector. The crown lets the coordinator rank
promising head shards (and, with
[FR-090](./FR-090-distann-fused-head-hop.md), select approximate seeds)
without a dedicated fan-out, recovering part of the sharded head's
round-trip cost (+8.6% @10k / +5.1% @100k vs the non-conforming local-head
referent, `reviews/task-210/006-zero-byte-head/`). The crown narrows the
distributed protocol; it never substitutes for it (the FR-084 bright line).

## Behavior

- **Capacity.** Crown capacity SHALL be the `ec_distann.crown_capacity`
  session GUC (default 0 = off), a stated constant in entries independent
  of N **and of head size C**. When C exceeds capacity, the crown holds a
  coarser subset and hit quality degrades; coordinator memory never grows.
  Admission past capacity SHALL be refused, not evicted. The crown SHALL
  never attempt to mirror the aggregate head.
- **Selection.** Crown membership SHALL be a static, deterministic,
  structural selection from the head membership (a coarser sample of the
  head or its upper navigation layers), sized to capacity; the selection
  digest SHALL be attested. Frequency-aware admission is out of scope until
  a measured skew case justifies its nondeterminism.
- **Content.** Entries SHALL hold exactly `(vec_id, quantized code)`.
  Nothing vector-shaped SHALL be resident at the coordinator.
- **Lifecycle.** The crown SHALL be epoch-fingerprint-keyed and per-backend,
  populated lazily by bounded batch RPCs from the owners (owners remain the
  source of truth), rebuilt on epoch flip, and discarded and repopulated on
  a capacity GUC change (the FR-086 staleness rule). There SHALL be no
  serve-time remote calls. The crown is rebuild-only by design: head
  membership is frozen within an epoch (D10), so crown and head cannot
  diverge; inserts reach new rows through owner graphs, not through the
  head.
- **Fallback.** A crown miss (or crown off/unpopulated) SHALL fall back to
  the full sharded head fan-out
  ([FR-080](./FR-080-distann-coordinator-head-index.md)): identical
  results, one round trip slower, never a wrong answer
  ([NFR-021](../../../non-functional/NFR-021-distann-distribution-invariant.md)
  clause 4).
- **Width pruning.** When the crown is populated, the coordinator MAY use
  crown scores to fan the head search only to promising shard holders.
  Without [FR-090](./FR-090-distann-fused-head-hop.md), the crown's win is
  owner CPU and tail width, not the round trip itself; A/B evidence SHALL
  report that honestly rather than promising latency.
- **Conformance.** The crown SHALL pre-register under NFR-021 as a bounded
  conforming structure; its resident bytes SHALL be itemised on the
  coordinator storage row and stay within the stated capacity.
- **Observability.** The extension SHALL expose activation counters
  (`crown_seeds_served`, `crown_fallbacks`) from day one; the candidate arm
  of any A/B SHALL assert non-zero activation (four Task 210 mechanisms ran
  inert inside green suite runs; a fifth is not acceptable).

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-089-CON-1 | Crown resident bytes SHALL be bounded by capacity × (8 + code bytes + entry header), independent of both N and C | Memory | Analysis + storage audit |
| FR-089-CON-2 | Crown selection SHALL be deterministic for a fixed epoch and capacity; the selection digest is attested | Determinism | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-089-AC-1 | With the crown on, per-query results are identical to the crown-off path (fallback correctness); a forced miss produces identical results one RTT slower | Test |
| FR-089-AC-2 | Admission past capacity is refused; resident bytes never exceed the stated bound as C or N grow | Test |
| FR-089-AC-3 | Epoch flip and capacity change discard the crown; no cross-epoch or cross-capacity reuse | Test |
| FR-089-AC-4 | Nothing vector-shaped is resident at the coordinator; entries decode to (vec_id, code) only | Inspection + test |
| FR-089-AC-5 | Candidate A/B arms at 10k/50k/100k show non-zero `crown_seeds_served`, `coordinator_resident_unsharded_bytes=0`, `outstanding_distribution_gap=none`, and crown bytes within capacity | Analysis (bench) |

## Dependencies

- **Upstream**: [FR-080](./FR-080-distann-coordinator-head-index.md) (head
  membership as the selection universe),
  [FR-086](./FR-086-distann-gateway-copies.md) (bounded codes-only class,
  staleness rule), [FR-082](../lifecycle/FR-082-distann-epoch-lifecycle.md)
  (epoch scoping), [FR-088](./FR-088-distann-head-scaling-law.md) (capacity
  independence from the head law).
- **Downstream**: [FR-090](./FR-090-distann-fused-head-hop.md) (crown codes
  answer the candidate half at the coordinator);
  [NFR-021](../../../non-functional/NFR-021-distann-distribution-invariant.md)
  pre-registration.

## Verification scope

Sizing sweep: crown capacity × the FR-088 head law at 10k/50k/100k;
1M+ deferred with FR-088's scale bound (user ruling, 2026-08-01).
