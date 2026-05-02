---
id: StR-005
title: Multi-Access-Method Vector Search Portfolio
type: stakeholder-requirement
artifact_type: StR
status: APPROVED
relationships:
  - target: "ix://agent-ix/tqvector/US-012"
    type: "derives"
    cardinality: "1:N"
  - target: "ix://agent-ix/tqvector/US-013"
    type: "derives"
    cardinality: "1:N"
  - target: "ix://agent-ix/tqvector/US-014"
    type: "derives"
    cardinality: "1:N"
  - target: "ix://agent-ix/tqvector/US-017"
    type: "derives"
    cardinality: "1:N"
  - target: "ix://agent-ix/tqvector/US-018"
    type: "derives"
    cardinality: "1:N"
  - target: "ix://agent-ix/tqvector/US-019"
    type: "derives"
    cardinality: "1:N"
  - target: "ix://agent-ix/tqvector/US-020"
    type: "derives"
    cardinality: "1:N"
---
# StR-005: Multi-Access-Method Vector Search Portfolio

## Need

The extension now serves more than a single HNSW/TurboQuant experiment. Users need one PostgreSQL extension that can store vectors once and compare access-method tradeoffs without changing application tables.

## Expectation

Ecaz SHALL provide a canonical row type and multiple opt-in ANN access methods under one extension identity. HNSW SHALL remain the default general-purpose path; IVF and DiskANN SHALL be available as explicit access-method choices with their own tuning, observability, and measurement boundaries. SPIRE SHALL be planned as the recursive, partition-object path for local multi-NVMe and future multi-machine scale.

## Success Criteria

1. `ecvector(dim)` works as the canonical indexed column type for HNSW, IVF, and DiskANN.
2. `ec_hnsw`, `ec_ivf`, and `ec_diskann` are registered by `CREATE EXTENSION ecaz`.
3. Documentation and benchmarks distinguish default product guidance from local research/measurement lanes.
4. SPIRE planning defines PID-addressed partition objects, configurable epoch consistency, local multi-store placement, and the future multi-machine routing surface before persistence implementation starts.
