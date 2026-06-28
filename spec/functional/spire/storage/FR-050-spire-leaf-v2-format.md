---
id: FR-050
title: SPIRE Leaf V2 Format
type: FR
status: APPROVED
object: binary_format
relationships:
  - target: "ix://agent-ix/ecaz/FR-048"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-049"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-050: SPIRE Leaf V2 Format

## Description

SPIRE leaf V2 objects SHALL store assignment rows in a segmented, column-major
layout so scans can borrow row references, batch score encoded payloads, and
avoid copying entire leaf objects into per-query state. Leaf V2 MAY also store
query-time block-summary metadata used to score or prune groups of rows before
row-segment payload decode.

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

## Leaf Block Summaries

Leaf V2 may include leaf-local block summaries for latency-oriented scans. A
block summary groups a bounded row range inside a leaf and stores the
format-specific summary payload needed to estimate whether that block should be
decoded and scored for a query.

Rules:

1. Block summaries SHALL be scoped to one leaf object and SHALL NOT change the
   canonical assignment row encoding.
2. Summary rows SHALL identify the covered row range deterministically.
3. Summary scoring and pruning SHALL be optional. A scan must be able to fall
   back to full leaf segment decode when summaries are absent, stale, or
   disabled.
4. Summary pruning policies SHALL be benchmark-gated because product-scale Task
   82-84 evidence showed blanket candidate-cap expansion recovers recall only
   by increasing the scored candidate surface.
5. Any summary format that becomes externally durable SHALL be covered by the
   on-disk format evolution discipline in `NFR-016`.

## Canonical Segment Encoding

Leaf V2 segment payload bytes SHALL be decoded as a packed logical stream after
the `FR-049` common header. The logical stream uses no implicit padding. All
multi-byte integers and `float4` values SHALL be little-endian IEEE-compatible
encodings.

| Offset expression | Field | Encoding |
| --- | --- | --- |
| `0` | `segment_no` | `u32le` |
| `4` | `row_base` | `u32le` |
| `8` | `row_count` | `u32le` |
| `12` | `next_segment_locator` | `item_pointer_v1` |
| `18` | `flags` | `row_count` `u16le` values |
| `18 + 2*row_count` | `vec_ids` | `row_count * vec_id_stride` bytes |
| previous end | `heap_tids` | `row_count` `item_pointer_v1` values |
| previous end | `gammas` | `row_count` `float4le` values |
| previous end | `payloads` | `row_count * payload_stride` bytes |

`item_pointer_v1` is the canonical six-byte PostgreSQL heap locator encoding
`block_number: u32le` followed by `offset_number: u16le`. A zero block with zero
offset is invalid except where a locator field is explicitly marked invalid for
an empty object.

The `payloads` byte region SHALL be row-major: row `i` occupies bytes
`i * payload_stride .. (i + 1) * payload_stride`. Format-specific payload
decoders SHALL reject trailing bytes, short rows, and payload-format tags that
do not match `payload_stride`.

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

## Layout

```yaml
format: spire-leaf-v2
title: SPIRE Leaf V2 object
endianness: little
encoding: "binary, packed logical stream after the FR-049 common header"
item_pointer_v1:
  encoding: "block_number u32 followed by offset_number u16; 6 bytes"
  invalid: "zero block with zero offset, except where explicitly marked invalid for an empty object"
record_types:
  - name: meta_tuple
    header: { $ref: "ix://agent-ix/ecaz/spire-partition-object-header", kind: leaf, format_version: 2, flags: "0x00000001", level: 0, child_count: 0 }
    fields:
      - { name: payload_format, type: u8, enum: { 0: none, 1: turboquant, 2: pq_fastscan, 3: rabitq } }
      - { name: vec_id_kind, type: u8, enum: { 1: local_u64, 2: global_bytes } }
      - { name: reserved, type: u16, const: 0 }
      - { name: payload_stride, type: u32, description: "bytes per encoded payload row; nonzero for non-empty leaves" }
      - { name: vec_id_stride, type: u16, description: "16 for local IDs; 2..=32 for global IDs" }
      - { name: reserved2, type: u16, const: 0 }
      - { name: segment_count, type: u32 }
      - { name: first_segment_locator, type: item_pointer_v1, description: "invalid for empty leaf; valid for non-empty leaf" }
      - { name: object_bytes_total, type: u64, minimum: 1 }
  - name: segment_tuple
    header: { $ref: "ix://agent-ix/ecaz/spire-partition-object-header", kind: leaf, format_version: 2, flags: "0x00000002" }
    fields:
      - { name: segment_no, offset: 0, type: u32 }
      - { name: row_base, offset: 4, type: u32 }
      - { name: row_count, offset: 8, type: u32 }
      - { name: next_segment_locator, offset: 12, type: item_pointer_v1 }
      - { name: flags, offset: 18, type: "u16[row_count]", flag_bits: [primary, boundary_replica, tombstone, stale_locator, delta_insert, delta_delete] }
      - { name: vec_ids, offset: "18 + 2*row_count", type: "bytes[row_count * vec_id_stride]" }
      - { name: heap_tids, offset: previous end, type: "item_pointer_v1[row_count]" }
      - { name: gammas, offset: previous end, type: "f32[row_count]", constraint: finite }
      - { name: payloads, offset: previous end, type: "bytes[row_count * payload_stride]", layout: "row-major; row i occupies bytes i*payload_stride..(i+1)*payload_stride" }
vec_id:
  local: "0x01 || little_endian_u64 (dedupe scope: origin node only)"
  global: "0x02 || stable_global_payload_bytes (dedupe scope: all nodes; production payload is 16 bytes, 17 bytes stored)"
  max_bytes_including_discriminator: 32
block_summary:
  status: "optional, evidence-gated (FR-050 rules 1-5)"
  description: "Leaf-local block summaries group a deterministic row range and store a format-specific summary payload used to estimate whether the block should be decoded; the canonical assignment row encoding above is unchanged, scans fall back to full segment decode when summaries are absent/stale/disabled, and any externally durable summary format must pass NFR-016 format-evolution discipline before this schema pins its byte layout."
```

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-050-AC-1 | An independent implementation can decode a Leaf V2 meta tuple and follow its segment chain without Rust-specific or in-memory layout assumptions | Analysis |
| FR-050-AC-2 | Malformed stride, row-count, non-finite gamma, invalid heap TID, and invalid vector-ID encodings are rejected | Test |
| FR-050-AC-3 | The spec defines enough vector identity and assignment flag semantics to reproduce scan dedupe, boundary-replica handling, and delta overlay behavior | Inspection |
| FR-050-AC-4 | Leaf block summaries decode independently from row segments and scans fall back to full row-segment decode when summary metadata is unavailable or disabled | Test |

### FR-050-AC-1: Independent decodability

An independent implementation can decode a Leaf V2 meta tuple and follow its
segment chain without consulting Rust-specific structures, host pointer layout,
or PostgreSQL in-memory struct alignment.

### FR-050-AC-2: Malformed-encoding rejection

Malformed stride, row-count, non-finite gamma, invalid heap TID, and invalid
vector-ID encodings are rejected.

### FR-050-AC-3: Identity and flag semantics

The spec defines enough vector identity and assignment flag semantics to
reproduce scan dedupe, boundary-replica handling, and delta overlay behavior.

### FR-050-AC-4: Block-summary independence and fallback

Leaf block summaries can be decoded independently from row segments and scans
can fall back to full row-segment decode when summary metadata is unavailable
or disabled.

## Dependencies

- **Upstream**: FR-048 (domain model: vector identity, boundary replicas,
  delta overlay semantics), FR-049 (common partition object header preceding
  every leaf tuple), NFR-016 (on-disk format evolution discipline for any
  externally durable summary format).
- **Downstream**: FR-051 (delta objects reuse the Leaf V2 segment payload
  encoding defined here).
