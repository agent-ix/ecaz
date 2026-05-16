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
  - target: "ix://agent-ix/ecaz/US-022"
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

Ecaz SHALL provide a canonical row type and multiple opt-in ANN access methods under one extension identity. HNSW SHALL remain the default general-purpose path; IVF and DiskANN SHALL be available as explicit access-method choices with their own tuning, observability, and measurement boundaries. SPIRE SHALL provide the partition-object path for local multi-store operation and distributed PostgreSQL-node scale.

## Success Criteria

1. `ecvector(dim)` works as the canonical indexed column type for HNSW, IVF, and DiskANN.
2. `ec_hnsw`, `ec_ivf`, `ec_diskann`, and `ec_spire` are registered by `CREATE EXTENSION ecaz`.
3. Documentation and benchmarks distinguish default product guidance from local research/measurement lanes.
4. SPIRE specs define PID-addressed partition objects, epoch publication, local multi-store placement, CustomScan distributed reads, typed tuple transport, coordinator-routed DML, 2PC recovery, and operator diagnostics.
