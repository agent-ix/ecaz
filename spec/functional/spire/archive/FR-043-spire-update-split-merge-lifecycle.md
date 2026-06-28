---
id: FR-043
title: "SPIRE Update, Split, and Merge Lifecycle (Superseded)"
type: FR
status: SUPERSEDED
relationships:
  - target: "ix://agent-ix/ecaz/FR-054"
    type: "superseded_by"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-059"
    type: "superseded_by"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-060"
    type: "superseded_by"
    cardinality: "1:N"
---
# FR-043: SPIRE Update, Split, and Merge Lifecycle (Superseded)

## Description

This identifier was assigned during the earlier SPIRE partition-object design
checkpoint for update, split, and merge lifecycle behavior. The active
requirements are now:

- `FR-054` for local update and maintenance lifecycle behavior.
- `FR-059` for coordinator-routed DML and two-phase commit behavior.
- `FR-060` for diagnostics and operational reporting.

This tombstone has no active acceptance criteria. It exists to preserve the
immutable requirement ID history required by the master specification lifecycle
policy.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-043-AC-1 | The tombstone preserves the superseded requirement ID and routes readers to the active superseding requirements | Inspection |

## Dependencies

- **Upstream**: none (superseded; see Description).
- **Downstream**: FR-054, FR-059, FR-060 (superseding active requirements).
