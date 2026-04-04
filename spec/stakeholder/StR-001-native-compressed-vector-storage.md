---
id: StR-001
title: Native Compressed Vector Storage in PostgreSQL
type: stakeholder-requirement
status: APPROVED
derived_usecases:
  - US-001
  - US-002
  - US-003
  - US-005
---
# StR-001: Native Compressed Vector Storage in PostgreSQL

## Need

The agent memory system requires storing and querying millions of high-dimensional embedding vectors (1536-dim fp32) across 100K+ agents. Raw fp32 storage consumes 6KB per vector — at scale this requires 8.8TB of HNSW indexes alone, making it impractical on standard hardware.

## Expectation

A PostgreSQL extension SHALL provide a native data type that stores TurboQuant-compressed vectors at approximately 7–8x compression (fp32 6KB → ~783 bytes including metadata at 4-bit), with approximate nearest neighbor search running directly over compressed codes inside the database engine.

## Rationale

- Existing open-source extensions (pgvecto.rs, VectorChord) have licensing issues (AGPL/ELv2) incompatible with product distribution
- pgvector stores fp32 — no compression, 8x larger storage
- Keeping ANN search inside PostgreSQL eliminates external vector database infrastructure and simplifies operations
- TurboQuant is data-oblivious (no training) — new agents produce valid compressed codes on their first INSERT with zero warm-up

## Success Criteria

- 1M vectors at 1536-dim, 4-bit stored in < 1GB of index space
- ANN queries return results via standard SQL (ORDER BY ... <#> ... LIMIT k)
- Extension installable via CREATE EXTENSION on PostgreSQL 14–17
