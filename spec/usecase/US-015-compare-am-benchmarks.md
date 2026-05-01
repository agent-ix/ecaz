---
id: US-015
title: Compare Access Method Benchmarks
type: user-story
artifact_type: US
status: APPROVED
relationships:
  - target: "ix://agent-ix/tqvector/StR-006"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/tqvector/NFR-007"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/tqvector/NFR-008"
    type: "derives_into"
    cardinality: "1:N"
---
# US-015: Compare Access Method Benchmarks

**As** a reviewer or platform engineer,
**I want** benchmark rows to cite the exact AM profile, commands, settings, and raw artifacts,
**So that** I can distinguish landed local evidence from future product claims.

## Acceptance Criteria

### US-015-AC-1

Benchmark documentation cites the review packet or artifact source for each landed measurement row.

### US-015-AC-2

Local evidence is labeled as local development evidence and not as a product benchmark claim.

### US-015-AC-3

Future product claims identify hardware, storage, cache state, PostgreSQL settings, corpus, query set, and command provenance.
