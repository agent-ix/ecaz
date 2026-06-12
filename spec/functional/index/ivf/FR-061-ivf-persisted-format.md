---
id: FR-061
title: "IVF Persisted Index Format"
artifact_type: FR
status: IMPLEMENTED
object: binary_format
relationships:
  - target: "ix://agent-ix/ecaz/US-013"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-031"
    type: "constrains"
    cardinality: "1:1"
  - target: "ix://agent-ix/ecaz/FR-032"
    type: "constrains"
    cardinality: "1:1"
---
# [FR-061] IVF Persisted Index Format

## Description

`ec_ivf` SHALL persist its index state in the byte-level layout defined here.
All multi-byte integers and `float4` values are little-endian. Tuples live on
standard 8KB PostgreSQL pages with line pointers; the metadata block uses the
page special area. An independent implementation SHALL be able to decode the
metadata block, centroid chain, list directory, posting entries, and PQ
codebook chain from this schema without consulting Rust struct definitions.

Implementation anchor: `src/am/ec_ivf/page.rs` (metadata, centroid, posting
encode/decode) and `src/am/ec_ivf/quantizer.rs` (payload variants).

## Layout

```yaml
format: ivf-persisted-format
title: ec_ivf persisted index format
endianness: little
encoding: "8KB PostgreSQL pages with line pointers; metadata in page-0 special area"
item_pointer: "block_number u32 followed by offset_number u16; 6 bytes"
record_types:
  - name: metadata_block
    location: page 0 special area
    magic: 0x56494345
    size: 80
    fields:
      - { name: magic, offset: 0, size: 4, type: u32, const: "0x56494345 ('ECIV')" }
      - { name: format_version, offset: 4, size: 2, type: u16, enum: [1, 2] }
      - { name: dimensions, offset: 6, size: 2, type: u16 }
      - { name: nlists, offset: 8, size: 4, type: u32 }
      - { name: nprobe, offset: 12, size: 4, type: u32 }
      - { name: training_sample_rows, offset: 16, size: 4, type: u32 }
      - { name: training_version, offset: 20, size: 2, type: u16 }
      - { name: seed, offset: 24, size: 8, type: u64 }
      - { name: storage_format, offset: 32, size: 1, type: u8, enum: { 0: auto, 1: turboquant, 2: pq_fastscan, 3: rabitq } }
      - { name: rerank_mode, offset: 33, size: 1, type: u8, enum: { 0: auto, 1: "off", 2: heap_f32, 3: source_column } }
      - { name: quant_bits, offset: 34, size: 1, type: u8, enum: [1, 2, 4, 8], description: "RaBitQ per-dimension code width; v2 only, v1 reads 0 and coerces to 4" }
      - { name: centroid_head, offset: 36, size: 6, type: item_pointer }
      - { name: directory_head, offset: 42, size: 6, type: item_pointer }
      - { name: total_live_tuples, offset: 48, size: 8, type: u64 }
      - { name: total_dead_tuples, offset: 56, size: 8, type: u64 }
      - { name: inserted_since_build, offset: 64, size: 8, type: u64 }
      - { name: pq_codebook_head, offset: 72, size: 6, type: item_pointer, description: "pq_fastscan only; INVALID otherwise" }
      - { name: pq_group_size, offset: 78, size: 2, type: u16, description: dimensions per PQ group }
  - name: centroid_tuple
    magic: 0x21
    size: "7 + 4*dimensions"
    chain: tuples reachable by walking from metadata.centroid_head in physical tuple order
    fields:
      - { name: tag, offset: 0, size: 1, type: u8, const: 0x21 }
      - { name: list_id, offset: 1, size: 4, type: u32 }
      - { name: dimensions, offset: 5, size: 2, type: u16, constraint: must equal metadata.dimensions }
      - { name: centroid, offset: 7, type: "f32[dimensions]" }
  - name: directory_tuple
    magic: 0x22
    description: "per-list directory entry mapping list_id to its posting block range (head_block..tail_block) and live/dead counts; chain head is metadata.directory_head"
  - name: posting_tuple
    magic: 0x23
    fixed_header_bytes: 77
    fields:
      - { name: tag, offset: 0, size: 1, type: u8, const: 0x23 }
      - { name: list_id, offset: 1, size: 4, type: u32 }
      - { name: flags, offset: 5, size: 1, type: u8, flag_bits: { 0: deleted } }
      - { name: heaptid_count, offset: 6, size: 1, type: u8, maximum: 10 }
      - { name: heaptids, offset: 7, size: 60, type: "item_pointer[10]", description: "inline capacity 10; unused slots INVALID" }
      - { name: gamma, offset: 67, size: 4, type: f32, description: "TurboQuant residual norm; 0.0 for pq_fastscan and rabitq" }
      - { name: rerank_tid, offset: 71, size: 6, type: item_pointer, description: optional external rerank payload locator }
      - { name: payload, offset: 77, type: "bytes[payload_len]", description: "quantized code bytes; length fixed per index by storage_format, dimensions, quant_bits, pq_group_size" }
    payload_variants:
      turboquant: mse_packed || qjl_packed per FR-015 encode lengths
      rabitq: ceil(dimensions * quant_bits / 8) bytes
      pq_fastscan: "grouped-PQ4 code bytes; 256 centroids per group"
  - name: pq_codebook_tuple
    magic: 0x24
    chain: "singly linked via next_tid from metadata.pq_codebook_head; one shard per PQ group"
    fields:
      - { name: tag, offset: 0, size: 1, type: u8, const: 0x24 }
      - { name: group_index, offset: 1, size: 2, type: u16 }
      - { name: next_tid, offset: 3, size: 6, type: item_pointer, description: INVALID at chain tail }
      - { name: centroids, offset: 9, type: "f32[centroid_count]", description: centroid_count = group_size * 256 }
not_persisted:
  - admin/drift snapshots (recomputed from metadata and directory tuples at call time)
```

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-061-CON-1 | Decode rejects metadata blocks whose magic is not `0x56494345` or whose `format_version` is outside `1..=2` | Technical | pg_test decode rejection |
| FR-061-CON-2 | Centroid tuple `dimensions` must equal metadata `dimensions`; mismatches reject before scoring | Technical | pg_test |
| FR-061-CON-3 | Posting `heaptid_count` never exceeds the inline capacity of 10 | Technical | pg_test |
| FR-061-CON-4 | Posting payload length is constant per index and derived only from metadata (`storage_format`, `dimensions`, `quant_bits`, `pq_group_size`); trailing or short payload bytes reject | Technical | pg_test decode rejection |
| FR-061-CON-5 | Externally durable changes to this layout follow the on-disk format evolution discipline (`NFR-016`) and bump `format_version` | Architecture | Spec review |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-061-AC-1 | An independent implementation can decode metadata, centroid chain, posting entries, and PQ codebook chain from this schema alone | Spec audit against `src/am/ec_ivf/page.rs` |
| FR-061-AC-2 | Malformed magic, version, dimension mismatch, heaptid overflow, and payload-length mismatch are rejected | pg_test |
| FR-061-AC-3 | The three payload variants are distinguishable from metadata alone, without sniffing payload bytes | Code review |

## Dependencies

- **Upstream**: `NFR-016` on-disk format evolution discipline.
- **Downstream**: `FR-031` (build writes this format), `FR-032` (scan reads it), `FR-033` (insert/vacuum mutate it), `FR-068`/`FR-069` process FRs.
