---
id: FR-082
title: Distann Epoch Lifecycle and Consistency
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-078"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-079"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-082: Distann Epoch Lifecycle and Consistency

## Description

The global graph SHALL be published and consumed in epochs with a
Building → Published → Retired lifecycle (reusing the SPIRE epoch-manifest
machinery), and every remote call SHALL carry the coordinator's epoch
fingerprint so cross-node reads are always mutually consistent.

## Behavior

- A build SHALL assemble the full record set, placement metadata, head
  sample, and the co-placed full-precision vector (heap) tier
  ([FR-078](./FR-078-distann-hash-placement.md), ADR-085 D11) under a
  Building epoch; queries SHALL never observe a Building epoch.
- Publishing SHALL be atomic per node roster: after publication every
  participant resolves the same (manifest, placement, head sample, vector
  tier) for the epoch fingerprint.
- Every `ec_distann_expand_nodes` call SHALL validate the caller's epoch
  fingerprint; mismatch SHALL raise a retriable error. When any hop round
  fails with epoch mismatch, the coordinator SHALL discard all partial scan
  state, refresh its epoch view, and restart the entire scan from the head
  index under the new epoch, at most once; a second mismatch fails the query
  ([NFR-020](../../../non-functional/NFR-020-distann-fault-behavior.md)).
  A restart resets the [NFR-019](../../../non-functional/NFR-019-distann-per-query-touch-bound.md)
  expansion accounting (the bound applies per attempt; attempts are capped
  at two).
- **Published-epoch mutation model** (ADR-085 D10): within a Published
  epoch, graph-node records and adjacency are immutable except for
  (a) monotonic tombstone-flag sets ([FR-083](./FR-083-distann-dml-path.md)),
  (b) delta-buffer appends (ADR-085 D5), and (c) incremental-insert record
  appends with their back-edge amendments (FR-083, final milestone). The
  co-placed vector (heap) tier is likewise immutable within a Published
  epoch: the exact-rerank vector bound to a vec_id is frozen at build and is
  never mutated or physically reclaimed under the epoch. A `heap_tid` SHALL
  resolve the epoch's frozen vector for its vec_id and SHALL NOT be exposed
  to a base-table TID-reuse race — multi-node deployments serve an
  epoch-owned frozen snapshot; the single-node degenerate case serves the
  base table under the AM's existing tombstone/vacuum-consistency handling,
  which SHALL guarantee the same vec_id→vector correspondence. The epoch
  fingerprint attests to roster, placement, format version, the build-time
  record set, and the vector tier — not to the mutable delta/tombstone
  state. Physical record and vector reclaim and edge repair happen only at
  the next epoch build, which re-establishes the FR-077 structural
  invariants.
- Concurrent-mutation visibility: an in-flight scan MAY observe pre- or
  post-amendment adjacency for any record (both are valid graphs); results
  SHALL be drawn only from records the scan actually expanded, tombstones
  SHALL be honored at expansion time, and a scan SHALL never observe a
  half-applied back-edge amendment (per-record write atomicity).
- A Retired epoch's storage (records, head sample, and the co-placed vector
  tier) SHALL be reclaimed only after its in-flight query count reaches zero
  (the existing retention gate). Operators SHALL
  have a documented override for a wedged in-flight count (e.g. a crashed
  coordinator that never decremented), with the epoch's storage retained
  until the override is invoked.
- In-scan consistency: one scan attempt SHALL execute entirely within one
  epoch; the restart rule above is the only path by which a query touches
  two epochs, and the two attempts never mix results.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-082-AC-1 | Queries during an epoch swap return results wholly from one epoch | Test (lifecycle drill) |
| FR-082-AC-2 | Fingerprint mismatch triggers one full scan restart under the refreshed epoch, then errors; partial state is never carried across the restart | Test (fault drill) |
| FR-082-AC-4 | Under concurrent tombstone/insert amendments, scans return only expanded records and never observe a half-applied back-edge amendment | Test (concurrency drill) |
| FR-082-AC-3 | Retired-epoch storage reclaim waits for in-flight queries | Test |
| FR-082-AC-5 | Within a Published epoch, a vec_id's exact-rerank vector resolved via `heap_tid` is byte-identical to its build-time vector — a concurrent base-table delete+VACUUM+TID-reuse never causes rerank against a different tuple | Test (concurrency drill) |
| FR-082-AC-6 | A wedged in-flight query count (e.g. crashed coordinator) never auto-reclaims Retired-epoch storage; storage is retained until the operator override is invoked, and the override is logged | Test (fault drill) |

## Dependencies

- **Upstream**: [FR-078](./FR-078-distann-hash-placement.md),
  [FR-079](./FR-079-distann-remote-expansion-protocol.md)
- **Downstream**: [FR-083](./FR-083-distann-dml-path.md)
