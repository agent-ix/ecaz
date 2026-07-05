---
id: FR-041
title: "SPIRE Epoch Consistency (Superseded)"
type: FR
status: SUPERSEDED
relationships:
  - target: "ix://agent-ix/ecaz/FR-048"
    type: "superseded_by"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-052"
    type: "superseded_by"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-054"
    type: "superseded_by"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-057"
    type: "superseded_by"
    cardinality: "1:N"
---
# FR-041: SPIRE Epoch Consistency (Superseded)

## Description

This identifier was assigned during the earlier SPIRE partition-object design
checkpoint for epoch consistency. The active requirements are now:

- `FR-048` for epoch identity and visibility boundaries.
- `FR-052` for build and publish behavior.
- `FR-054` for replacement, split, merge, and maintenance behavior.
- `FR-057` for production remote executor epoch readiness.

This tombstone has no active acceptance criteria. It exists to preserve the
immutable requirement ID history required by the master specification lifecycle
policy.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-041-AC-1 | The tombstone preserves the superseded requirement ID and routes readers to the active superseding requirements | Inspection |

## Dependencies

- **Upstream**: none (superseded; see Description).
- **Downstream**: FR-048, FR-052, FR-054, FR-057 (superseding active
  requirements).
