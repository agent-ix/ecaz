---
id: FR-050
title: SPIRE Leaf V2 Format
type: functional-requirement
artifact_type: FR
status: APPROVED
object: data_schema
relationships:
  - target: "ix://agent-ix/ecaz/FR-048"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-049"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-050: SPIRE Leaf V2 Format

## Requirement

SPIRE leaf V2 objects SHALL store assignment rows in a segmented, column-major
layout so scans can borrow row references, batch score encoded payloads, and
avoid copying entire leaf objects into per-query state.

## Leaf V2 Meta Tuple

Leaf V2 meta is a partition object with `kind = leaf`, `format_version = 2`,
and `flags = 0x0000_0001`.

| Field | Type | Rule |
| --- | --- | --- |
| common header | `FR-049` header | `level = 0`, `child_count = 0`, `assignment_count = total rows` |
| payload_format | `u8` | `0=none`, `1=turboquant`, `2=pq_fastscan`, `3=rabitq` |
| vec_id_kind | `u8` | `1=local_u64`, `2=global_bytes` |
| reserved | `u16` | zero |
| payload_stride | `u32` | bytes per encoded payload row; nonzero for non-empty leaves |
| vec_id_stride | `u16` | `16` for local IDs; `2..=32` for global IDs |
| reserved2 | `u16` | zero |
| segment_count | `u32` | number of segment tuples |
| first_segment_locator | item pointer | invalid for empty leaf; valid for non-empty leaf |
| object_bytes_total | `u64` | nonzero byte total for meta plus segment chain |

## Leaf V2 Segment Tuple

Leaf V2 segment tuples use `kind = leaf`, `format_version = 2`, and
`flags = 0x0000_0002`. Each segment stores rows in this order:

1. `segment_no: u32`
2. `row_base: u32`
3. `row_count: u32`
4. `next_segment_locator: item pointer`
5. `flags[row_count]: u16[]`
6. `vec_ids[row_count * vec_id_stride]: bytea`
7. `heap_tids[row_count]: item pointer[]`
8. `gammas[row_count]: float4[]`
9. `payloads[row_count * payload_stride]: bytea`

## Vector Identity

| Form | Bytes | Dedupe scope |
| --- | --- | --- |
| local | `0x01 || little_endian_u64` | origin node only |
| global | `0x02 || stable_global_payload_bytes` | all nodes |

`SpireVecId` SHALL be at most 32 bytes including the discriminator. The
production global source identity payload is 16 bytes, producing a 17-byte
stored global `SpireVecId`.

## Assignment Flags

| Flag | Meaning |
| --- | --- |
| `primary` | Primary assignment for the vector. |
| `boundary_replica` | Replica assignment for border recall. |
| `tombstone` | Row suppresses or marks deleted state. |
| `stale_locator` | Stored locator is no longer trusted. |
| `delta_insert` | Delta object insert row. |
| `delta_delete` | Delta object delete row. |

Leaf V2 base segments SHALL NOT set `delta_insert` or `delta_delete`; those
flags are reserved for delta objects.

## Validation

1. A non-empty meta tuple SHALL have nonzero `segment_count`, valid first
   segment locator, nonzero payload stride, and payload format other than
   `none`.
2. Segment tuple headers SHALL match the meta PID, object version, parent PID,
   and published epoch back-reference.
3. Segment tuple `row_count` SHALL equal the header `assignment_count`.
4. Segment tuple arrays SHALL have lengths exactly implied by `row_count`,
   `vec_id_stride`, and `payload_stride`.
5. Segment tuple heap TIDs and gammas SHALL be valid and finite.
6. Assignment payload format SHALL be one of the defined tags.

## Acceptance Criteria

### FR-050-AC-1

An independent implementation can decode a Leaf V2 meta tuple and follow its
segment chain without consulting Rust-specific structures.

### FR-050-AC-2

Malformed stride, row-count, non-finite gamma, invalid heap TID, and invalid
vector-ID encodings are rejected.

### FR-050-AC-3

The spec defines enough vector identity and assignment flag semantics to
reproduce scan dedupe, boundary-replica handling, and delta overlay behavior.
