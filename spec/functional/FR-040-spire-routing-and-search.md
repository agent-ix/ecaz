---
id: FR-040
title: "SPIRE Routing and Search (Superseded)"
type: functional-requirement
artifact_type: FR
status: SUPERSEDED
relationships:
  - target: "ix://agent-ix/ecaz/FR-048"
    type: "superseded_by"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-051"
    type: "superseded_by"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-053"
    type: "superseded_by"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-058"
    type: "superseded_by"
    cardinality: "1:N"
---
# FR-040: SPIRE Routing and Search (Superseded)

## Tombstone

This identifier was assigned during the earlier SPIRE partition-object design
checkpoint for routing and search behavior. The active requirements are now:

- `FR-048` for the SPIRE bounded context.
- `FR-051` for routing, delta, and top-graph object formats.
- `FR-053` for local eager bounded search.
- `FR-058` for distributed CustomScan reads.

This tombstone has no active acceptance criteria. It exists to preserve the
immutable requirement ID history required by the master specification lifecycle
policy.
