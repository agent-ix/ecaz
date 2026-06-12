---
id: FR-042
title: "SPIRE Distributed libpq Coordinator (Superseded)"
type: functional-requirement
artifact_type: FR
status: SUPERSEDED
relationships:
  - target: "ix://agent-ix/ecaz/FR-055"
    type: "superseded_by"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-056"
    type: "superseded_by"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-057"
    type: "superseded_by"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-058"
    type: "superseded_by"
    cardinality: "1:N"
---
# FR-042: SPIRE Distributed libpq Coordinator (Superseded)

## Description

This identifier was assigned during the earlier SPIRE partition-object design
checkpoint for libpq coordinator behavior. The active requirements are now:

- `FR-055` for topology and placement-directory behavior.
- `FR-056` for endpoint typed transport.
- `FR-057` for the production remote executor.
- `FR-058` for distributed CustomScan reads.

This tombstone has no active acceptance criteria. It exists to preserve the
immutable requirement ID history required by the master specification lifecycle
policy.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-042-AC-1 | The tombstone preserves the superseded requirement ID and routes readers to the active superseding requirements | Inspection |

## Dependencies

- **Upstream**: none (superseded; see Description).
- **Downstream**: FR-055, FR-056, FR-057, FR-058 (superseding active
  requirements).
