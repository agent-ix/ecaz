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

- A build SHALL assemble the full record set, placement metadata, and head
  sample under a Building epoch; queries SHALL never observe a Building
  epoch.
- Publishing SHALL be atomic per node roster: after publication every
  participant resolves the same (manifest, placement, head sample) for the
  epoch fingerprint.
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
  epoch fingerprint attests to roster, placement, format version, and the
  build-time record set — not to the mutable delta/tombstone state.
  Physical record reclaim and edge repair happen only at the next epoch
  build, which re-establishes the FR-077 structural invariants.
- Concurrent-mutation visibility: an in-flight scan MAY observe pre- or
  post-amendment adjacency for any record (both are valid graphs); results
  SHALL be drawn only from records the scan actually expanded, tombstones
  SHALL be honored at expansion time, and a scan SHALL never observe a
  half-applied back-edge amendment (per-record write atomicity).
- A Retired epoch's storage SHALL be reclaimed only after its in-flight
  query count reaches zero (the existing retention gate). Operators SHALL
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

## Dependencies

- **Upstream**: [FR-078](./FR-078-distann-hash-placement.md),
  [FR-079](./FR-079-distann-remote-expansion-protocol.md)
- **Downstream**: [FR-083](./FR-083-distann-dml-path.md)
