---
id: FR-043
title: "SPIRE Update, Split, and Merge Lifecycle (Superseded)"
type: functional-requirement
artifact_type: FR
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

## Tombstone

This identifier was assigned during the earlier SPIRE partition-object design
checkpoint for update, split, and merge lifecycle behavior. The active
requirements are now:

- `FR-054` for local update and maintenance lifecycle behavior.
- `FR-059` for coordinator-routed DML and two-phase commit behavior.
- `FR-060` for diagnostics and operational reporting.

This tombstone has no active acceptance criteria. It exists to preserve the
immutable requirement ID history required by the master specification lifecycle
policy.
