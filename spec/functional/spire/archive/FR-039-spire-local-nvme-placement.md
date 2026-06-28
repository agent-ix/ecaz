---
id: FR-039
title: "SPIRE Local NVMe Placement (Superseded)"
type: FR
status: SUPERSEDED
relationships:
  - target: "ix://agent-ix/ecaz/FR-048"
    type: "superseded_by"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-053"
    type: "superseded_by"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-055"
    type: "superseded_by"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-060"
    type: "superseded_by"
    cardinality: "1:N"
---
# FR-039: SPIRE Local NVMe Placement (Superseded)

## Description

This identifier was assigned during the earlier SPIRE partition-object design
checkpoint for local NVMe/store placement. The active requirements are now:

- `FR-048` for the SPIRE domain model and placement concepts.
- `FR-053` for local search and local-store behavior.
- `FR-055` for topology and placement-directory boundaries.
- `FR-060` for diagnostics and configuration.

This tombstone has no active acceptance criteria. It exists to preserve the
immutable requirement ID history required by the master specification lifecycle
policy.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-039-AC-1 | The tombstone preserves the superseded requirement ID and routes readers to the active superseding requirements | Inspection |

## Dependencies

- **Upstream**: none (superseded; see Description).
- **Downstream**: FR-048, FR-053, FR-055, FR-060 (superseding active
  requirements).
