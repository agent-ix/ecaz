---
id: US-015
title: Compare Access Method Benchmarks
type: user-story
artifact_type: US
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-006"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/NFR-007"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/NFR-008"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/NFR-015"
    type: "derives_into"
    cardinality: "1:N"
---
# US-015: Compare Access Method Benchmarks

**As** a reviewer or platform engineer,
**I want** benchmark rows to cite the exact AM profile, quantizer or storage format, commands, settings, metrics, and raw artifacts,
**So that** I can distinguish landed local evidence from future product claims.

## Acceptance Criteria

### US-015-AC-1

Benchmark documentation cites the review packet or artifact source for each landed measurement row.

### US-015-AC-2

Local evidence is labeled as local development evidence and not as a product benchmark claim.

### US-015-AC-3

Future product claims identify hardware, storage, cache state, PostgreSQL settings, corpus, query set, and command provenance.

### US-015-AC-4

Quantizer, storage-format, access-method, and option-set comparisons use the
shared benchmark reporting schema so future rows remain comparable with older
packet-backed rows.
