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
  fingerprint; mismatch SHALL raise a retriable error. The coordinator SHALL
  retry once after refreshing its epoch view, then fail the query
  ([NFR-020](../../../non-functional/NFR-020-distann-fault-behavior.md)).
- A Retired epoch's storage SHALL be reclaimed only after its in-flight
  query count reaches zero (the existing retention gate).
- In-scan consistency: one scan SHALL execute entirely within one epoch;
  epoch swap during a scan never mixes records from two epochs.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-082-AC-1 | Queries during an epoch swap return results wholly from one epoch | Test (lifecycle drill) |
| FR-082-AC-2 | Fingerprint mismatch is retried once then errors; never silent data | Test (fault drill) |
| FR-082-AC-3 | Retired-epoch storage reclaim waits for in-flight queries | Test |

## Dependencies

- **Upstream**: [FR-078](./FR-078-distann-hash-placement.md),
  [FR-079](./FR-079-distann-remote-expansion-protocol.md)
- **Downstream**: [FR-083](./FR-083-distann-dml-path.md)
