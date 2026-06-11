---
id: FR-062
title: "DiskANN Persisted Vamana Graph Format"
artifact_type: FR
status: IMPLEMENTED
object: data_schema
relationships:
  - target: "ix://agent-ix/ecaz/US-014"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-034"
    type: "constrains"
    cardinality: "1:1"
  - target: "ix://agent-ix/ecaz/FR-035"
    type: "constrains"
    cardinality: "1:1"
---
# [FR-062] DiskANN Persisted Vamana Graph Format

## Description

`ec_diskann` SHALL persist its Vamana graph in the byte-level layout defined
here. All multi-byte integers and `float4` values are little-endian. Tuples
live on standard 8KB PostgreSQL pages with line pointers; the metadata block
uses the page-0 special area. Node tuples are fixed-length per index, which is
what makes block-oriented traversal and prefilter batching possible.

Implementation anchor: `src/am/ec_diskann/page.rs` (metadata) and
`src/am/ec_diskann/tuple.rs` (node and codebook tuples).

## Schema

```json
{
  "$id": "ix://agent-ix/ecaz/diskann-persisted-format",
  "title": "ec_diskann persisted Vamana graph format",
  "encoding": "little-endian; 8KB PostgreSQL pages; metadata in page-0 special area",
  "item_pointer": "block_number u32le followed by offset_number u16le; 6 bytes",
  "metadata_block": {
    "location": "page 0 special area",
    "size_bytes": 48,
    "fields": [
      { "name": "format_version", "offset": 0, "size": 2, "type": "u16le", "const": 3, "description": "INDEX_FORMAT_V3_DISKANN; distinct from HNSW v1/v2" },
      { "name": "entry_point", "offset": 2, "size": 6, "type": "item_pointer", "description": "medoid node TID" },
      { "name": "graph_degree_r", "offset": 8, "size": 2, "type": "u16le", "description": "max neighbors per node (R)" },
      { "name": "build_list_size_l", "offset": 10, "size": 2, "type": "u16le", "description": "Vamana build beam (L)" },
      { "name": "alpha", "offset": 12, "size": 4, "type": "float4le", "description": "Vamana pruning factor" },
      { "name": "dimensions", "offset": 16, "size": 2, "type": "u16le" },
      { "name": "seed", "offset": 18, "size": 8, "type": "u64le" },
      { "name": "inserted_since_rebuild", "offset": 26, "size": 8, "type": "u64le" },
      { "name": "needs_medoid_refresh", "offset": 34, "size": 1, "type": "u8", "enum": [0, 1] },
      { "name": "transform_kind", "offset": 35, "size": 1, "type": "u8", "const": 1, "description": "SRHT rotation" },
      { "name": "search_codec_kind", "offset": 36, "size": 1, "type": "u8", "enum": { "2": "grouped_pq", "3": "rabitq", "4": "turboquant" } },
      { "name": "payload_flags", "offset": 37, "size": 1, "type": "u8", "flag_bits": { "0": "binary_sidecar", "1": "grouped_search_code", "2": "cold_rerank_payload (reserved, 0 in v0)" } },
      { "name": "search_subvector_count", "offset": 38, "size": 2, "type": "u16le", "description": "PQ group count" },
      { "name": "search_subvector_dim", "offset": 40, "size": 2, "type": "u16le", "description": "dimensions per PQ group" },
      { "name": "grouped_codebook_head", "offset": 42, "size": 6, "type": "item_pointer", "description": "PQ codebook chain head; INVALID when absent" }
    ]
  },
  "node_tuple": {
    "tag": "0x06",
    "fixed_length_per_index": "16 + 8*W + C + 6*R bytes, where R = graph_degree_r, W = binary word count (0 without sidecar), C = search code length from codec kind and dimensions",
    "fields": [
      { "name": "tag", "offset": 0, "size": 1, "type": "u8", "const": "0x06" },
      { "name": "flags", "offset": 1, "size": 1, "type": "u8", "flag_bits": { "0": "deleted", "1": "has_overflow_heaptids" } },
      { "name": "neighbor_count", "offset": 2, "size": 2, "type": "u16le", "constraint": "<= R" },
      { "name": "primary_heaptid", "offset": 4, "size": 6, "type": "item_pointer" },
      { "name": "rerank_tid", "offset": 10, "size": 6, "type": "item_pointer", "description": "reserved for cold-rerank payload; INVALID in v0" },
      { "name": "binary_words", "offset": 16, "type": "u64le[W]", "description": "binary sidecar bit vector; present only when payload_flags.binary_sidecar" },
      { "name": "search_code", "offset": "16 + 8*W", "type": "bytea[C]", "description": "quantized search code in the metadata codec kind" },
      { "name": "neighbors", "offset": "16 + 8*W + C", "type": "item_pointer[R]", "description": "first neighbor_count slots valid; remaining slots INVALID" }
    ]
  },
  "pq_codebook_tuple": {
    "tag": "0x07",
    "fields": [
      { "name": "tag", "offset": 0, "size": 1, "type": "u8", "const": "0x07" },
      { "name": "group_index", "offset": 1, "size": 2, "type": "u16le" },
      { "name": "next_tid", "offset": 3, "size": 6, "type": "item_pointer", "description": "INVALID at chain tail" },
      { "name": "centroids", "offset": 9, "type": "float4le[centroid_count]", "description": "centroid_count = search_subvector_dim * 256" }
    ],
    "chain": "singly linked via next_tid from metadata.grouped_codebook_head; one shard per PQ group (search_subvector_count shards)"
  },
  "not_persisted_in_v1": ["duplicate overflow heaptid chains (has_overflow_heaptids flag is reserved; one node = one primary heap row in v1)"]
}
```

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-062-CON-1 | Decode rejects metadata blocks whose `format_version` is not 3 | Technical | pg_test decode rejection |
| FR-062-CON-2 | Node tuple length is constant per index and fully derived from metadata (`graph_degree_r`, codec kind, dimensions, sidecar flag); length mismatches reject | Technical | pg_test |
| FR-062-CON-3 | `neighbor_count <= R` and every neighbor slot beyond `neighbor_count` is INVALID | Technical | pg_test |
| FR-062-CON-4 | Vacuum tombstones use the node `deleted` flag; readers skip deleted nodes without dereferencing their neighbors as live candidates | Technical | pg_test |
| FR-062-CON-5 | Externally durable changes to this layout follow `NFR-016` format-evolution discipline and bump `format_version` | Architecture | Spec review |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-062-AC-1 | An independent implementation can decode the metadata block, walk the graph from `entry_point`, and decode node payloads from this schema alone | Spec audit against `src/am/ec_diskann/tuple.rs` |
| FR-062-AC-2 | The binary sidecar, grouped-PQ search code, and codec kind are all resolvable from metadata before reading any node tuple | Code review |
| FR-062-AC-3 | Malformed version, tuple-length, and neighbor-slot violations are rejected | pg_test |

## Dependencies

- **Upstream**: `NFR-016` on-disk format evolution.
- **Downstream**: `FR-034` (build writes this format), `FR-035` (scan reads it), `FR-036` (insert/vacuum mutate it), `FR-067` scan-pipeline process FR.
