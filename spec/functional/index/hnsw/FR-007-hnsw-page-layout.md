---
id: FR-007
title: HNSW Index Access Method — Page Layout
type: FR
status: APPROVED
object: entity
traces:
  - US-003
  - StR-001
---
# FR-007: HNSW Index Access Method — Page Layout

## Description

The extension SHALL implement a custom PostgreSQL index access method named `ec_hnsw` with a page layout modeled on pgvector's HNSW implementation.

### Page 0 — Metadata

| Field | Type | Description |
|---|---|---|
| M | u16 | Max neighbors per layer (from WITH clause) |
| ef_construction | u16 | Build-time beam width |
| entry_point_blkno | u32 | Block number of the graph entry point |
| entry_point_offno | u16 | Offset within the block |
| dimensions | u16 | Vector dimensionality |
| bits | u8 | Quantization bits |
| max_level | u8 | Highest layer in the graph |

### Page 1+ — Interleaved Element and Neighbor Tuples

#### TqElementTuple (tag = 0x01)

| Field | Type | Description |
|---|---|---|
| type | u8 | `0x01` (ELEMENT) |
| level | u8 | HNSW layer this node was assigned to |
| deleted | bool | Soft-delete flag |
| heaptids | [ItemPointerData; 10] | Inline pointers back to heap rows that share this element's duplicate key |
| heaptid_count | u8 | Number of valid heaptids, capped at 10 inline entries |
| neighbortid | ItemPointerData | Pointer to this node's TqNeighborTuple |
| code | [u8; code_len] | The tqvector stored code bytes (`mse_packed + qjl_packed`) |

Duplicate semantics:
- Multiple heap rows MAY coalesce into one element tuple when they share the same persisted duplicate key.
- In the current implementation, that duplicate key is `(gamma, code_bytes)`.
- The element tuple stores only the shared `code_bytes`; `gamma` remains recoverable from representative heap rows until a future page-layout revision persists it in-page.
- `HEAPTID_INLINE_CAPACITY = 10` is a page-layout invariant in v0.1.

#### TqNeighborTuple (tag = 0x02)

| Field | Type | Description |
|---|---|---|
| type | u8 | `0x02` (NEIGHBOR) |
| count | u16 | Total number of active neighbors across all layers |
| tids | [ItemPointerData] | Per-layer neighbor pointers — M at layers > 0, 2M at layer 0 |

Neighbor TID array layout for a node at level `L` with max neighbors M:
```
Layer 0:  tids[0..2M]       (2M neighbors at the base layer)
Layer 1:  tids[2M..3M]      (M neighbors)
Layer 2:  tids[3M..4M]      (M neighbors)
...
Layer L:  tids[(L+1)*M..(L+2)*M]
```

Total TID slots: `2M + L*M` where L = node's assigned level.

### Storage Density

At 4-bit 1536-dim, each TqElementTuple is approximately:
- 1 (tag) + 1 (level) + 1 (deleted) + 60 (heaptids) + 1 (count) + 6 (neighbortid) + 772 (payload) = ~842 bytes
- Postgres page = 8,192 bytes (minus page header, line pointers, alignment, and neighbor tuple storage) → element-tuple density MUST be measured empirically; element-only arithmetic is not sufficient for size claims
- Compared to pgvector fp32: ~6,144 bytes per vector → ~1 per page

Compressed codes significantly reduce per-node payload versus fp32 storage, but full index density SHALL be accounted at the node level (element tuple + neighbor tuple + overhead), not from element tuples alone.

### Page Allocation Strategy

During **ambuild** (bulk build):
1. Start writing tuples to page 1 (page 0 is metadata)
2. For each tuple, check `PageGetFreeSpace(page)` against the tuple size
3. If insufficient space: call `RelationGetBufferForTuple` or extend the relation with `ReadBufferExtended(P_NEW)`
4. Track the current "fill page" block number for sequential allocation
5. Element and neighbor tuples for the same node SHOULD be placed on the same page when space permits (locality for scan)

During **aminsert** (single row insert):
1. Scan pages starting from the last known page with free space
2. If no page has enough space: extend the relation with a new page
3. Write element and neighbor tuples, preferring the same page

During **vacuum**:
- Deleted tuples leave dead space on their pages
- Dead space is reclaimed when new tuples are inserted into the same page
- Full page compaction is NOT performed (matches pgvector behavior)

### Tuple Fit and Level Boundaries

- The implementation SHALL define and enforce a maximum representable HNSW level such that every valid `TqNeighborTuple` fits on an 8KB page together with tuple headers and alignment.
- The maximum representable level SHALL be computed as the largest integer `max_level_cap` for which a neighbor tuple with `2M + max_level_cap * M` TID slots still fits on one page under the actual tuple-header and alignment rules used by the implementation.
- `aminsert` and `ambuild` SHALL reject or clamp any generated level that would violate this invariant, and the chosen policy SHALL be documented in the implementation.

### Page Locking Protocol

| Operation | Lock Type | Scope |
|---|---|---|
| Read tuple (scan, neighbor traversal) | `BUFFER_LOCK_SHARE` | Single page, released immediately after read |
| Write tuple (insert, update neighbors) | `BUFFER_LOCK_EXCLUSIVE` | Single page, held during GenericXLog transaction |
| Extend relation | `BUFFER_LOCK_EXCLUSIVE` | New page only |

To prevent deadlocks: always acquire page locks in block number order. Never hold a lock on page A while acquiring a lock on page B where B < A.

## Properties

The on-disk layout is composed of one metadata page (`MetadataPage`) plus interleaved element and neighbor tuples (`TqElementTuple` / `TqNeighborTuple`) defined in `src/am/ec_hnsw/page.rs`. The table below reflects the persisted fields of each on-page structure as implemented.

| Property | Owning structure | Type | Description |
|---|---|---|---|
| m | MetadataPage | u16 | Max neighbors per layer (WITH `m`) |
| ef_construction | MetadataPage | u16 | Build-time beam width |
| entry_point | MetadataPage | ItemPointer | Graph entry-point TID (block + offset) |
| dimensions | MetadataPage | u16 | Vector dimensionality |
| bits | MetadataPage | u8 | Quantization bits |
| max_level | MetadataPage | u8 | Highest occupied layer |
| seed | MetadataPage | u64 | Quantizer/layer RNG seed |
| inserted_since_rebuild | MetadataPage | u64 | Live inserts since last bulk build/REINDEX (insert-drift accounting) |
| format_version | MetadataPage | u16 | Index format (`v1_scalar`, `v2_grouped`, `v3_turbo_hot_cold`, `v4_rabitq`) |
| transform_kind | MetadataPage | u8 enum | Rotation transform (`Unknown`/`Srht`/`Opq`) |
| search_codec_kind | MetadataPage | u8 enum | Search codec (`Unknown`/`ScalarQuantized`/`GroupedPq`/`RaBitQ`) |
| payload_flags | MetadataPage | u8 | Bitfield (binary sidecar, grouped search code, cold rerank payload) |
| search_bits / rerank_codec_kind | MetadataPage | u8 | Search-side bit width and rerank codec selection |
| search_subvector_count / search_subvector_dim | MetadataPage | u16 | Grouped-PQ subvector geometry |
| grouped_codebook_head | MetadataPage | ItemPointer | Head TID of the grouped-PQ codebook chain |
| tag | TqElementTuple | u8 | `TQ_ELEMENT_TAG` = `0x01` |
| level | TqElementTuple | u8 | HNSW layer assigned to this node |
| deleted | TqElementTuple | bool | Soft-delete flag |
| heaptids | TqElementTuple | [ItemPointer; 10] | Inline heap TIDs for coalesced duplicates (`HEAPTID_INLINE_CAPACITY` = 10) |
| heaptid_count | TqElementTuple | u8 | Number of valid inline heap TIDs |
| gamma | TqElementTuple | f32 | Per-node quantizer gamma scalar |
| neighbortid | TqElementTuple | ItemPointer | Pointer to this node's `TqNeighborTuple` |
| code | TqElementTuple | [u8; code_len] | Packed quantizer code bytes |
| binary_words | TqElementTuple | [u64] | Optional binary sidecar words (when `PAYLOAD_FLAG_BINARY_SIDECAR` set) |
| tag | TqNeighborTuple | u8 | `TQ_NEIGHBOR_TAG` = `0x02` |
| count | TqNeighborTuple | u16 | Total neighbor TID slots across layers |
| tids | TqNeighborTuple | [ItemPointer] | `2M` slots at layer 0, `M` per layer above (`2M + level*M`) |

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-007-AC-1 | After CREATE INDEX, page 0 contains valid metadata with the specified M and ef_construction | Test |
| FR-007-AC-2 | Writing and reading a TqElementTuple to/from a page preserves all fields | Test |
| FR-007-AC-3 | Each element's neighbortid points to a valid TqNeighborTuple on the same or adjacent page | Test |
| FR-007-AC-4 | Inserting more tuples than fit on a single page extends the relation without errors | Test |
| FR-007-AC-5 | Concurrent inserts do not deadlock under stress (10 concurrent inserters for 30 seconds) | Test |

### FR-007-AC-1: Metadata page readable
After CREATE INDEX, page 0 SHALL contain valid metadata with the specified M and ef_construction.

### FR-007-AC-2: Element tuple round-trip
Writing and reading a TqElementTuple to/from a page SHALL preserve all fields.

### FR-007-AC-3: Neighbor tuple integrity
Each element's neighbortid SHALL point to a valid TqNeighborTuple on the same or adjacent page.

### FR-007-AC-4: Page extension works
Inserting more tuples than fit on a single page SHALL extend the relation without errors.

### FR-007-AC-5: Lock ordering prevents deadlock
Concurrent inserts SHALL not deadlock (verified by stress test with 10 concurrent inserters for 30 seconds).

## Dependencies

- **Upstream**: US-003, StR-001 (traces)
- **Downstream**: FR-008, FR-009, FR-010, FR-016 (trace this page layout)
