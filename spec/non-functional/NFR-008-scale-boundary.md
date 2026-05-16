---
id: NFR-008
title: Scale Boundary and Hardware Claim Policy
type: non-functional-requirement
artifact_type: NFR
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-006"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-008: Scale Boundary and Hardware Claim Policy

## Requirement

Ecaz SHALL separate local implementation-readiness evidence from larger scale claims that require AWS/RDS-class hardware.

## Policy

1. HNSW parallel build larger-scale validation SHALL be deferred to AWS/RDS-class hardware.
2. IVF 990K local results SHALL be treated as directional local evidence until exact controlled product runs are available.
3. DiskANN local Task 29 readiness SHALL establish implementation readiness, not billion-scale product claims.
4. Parallel index scan SHALL remain shelved until a new accepted ADR reopens it.

## Acceptance Criteria

### NFR-008-AC-1

Docs/specs identify local IVF and DiskANN results as local evidence.

### NFR-008-AC-2

Specs do not list parallel index scan as active work.

### NFR-008-AC-3

Future AWS/RDS-scale work is tracked as deferred measurement, not as an unfinished blocker for landed local implementation tasks.
