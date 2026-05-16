---
id: US-001
title: Store a Compressed Embedding
type: user-story
artifact_type: US
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-001"
    type: "derives_from"
    cardinality: "N:1"
---
# US-001: Store a Compressed Embedding

**As** an application developer inserting agent memories,
**I want** to compress an fp32 embedding into a `tqvector` column during INSERT,
**So that** the vector is stored at approximately 7–8x compression with no separate encoding step.

## Acceptance Criteria

### US-001-AC-1

`encode_to_tqvector(embedding float4[], bits int, seed bigint)` accepts a
1536-dim fp32 array and returns a `tqvector` value.

### US-001-AC-2

The returned value can be stored in a `tqvector` column.

### US-001-AC-3

No external process or training step is required before the first INSERT.

### US-001-AC-4

Bits parameter accepts values 2-8; invalid values produce a clear error.
