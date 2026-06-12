---
id: FR-049
title: SPIRE Partition Object Header
type: functional-requirement
artifact_type: FR
status: APPROVED
object: binary_format
relationships:
  - target: "ix://agent-ix/ecaz/FR-048"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-049: SPIRE Partition Object Header

## Description

Every persisted SPIRE partition object SHALL begin with a validated binary
header that identifies object kind, format version, PID, object version,
published epoch back-reference, hierarchy position, row counts, and flags.

## Binary Layout

All integer fields SHALL be little-endian.

| Offset | Size | Field | Rule |
| ---: | ---: | --- | --- |
| 0 | 4 | magic | `0x4f50_5345` (`ESPO` in little-endian bytes) |
| 4 | 2 | format_version | `1` or `2` |
| 6 | 1 | kind | `1=root`, `2=internal`, `3=leaf`, `4=delta`, `5=top_graph` |
| 7 | 1 | reserved | SHALL be zero |
| 8 | 8 | pid | Nonzero `u64` |
| 16 | 8 | object_version | Nonzero `u64` |
| 24 | 8 | published_epoch_backref | `0` for draft routing/top-graph objects; nonzero for published leaf V2 objects |
| 32 | 2 | level | `0` for leaves/deltas; positive for routing/top graph objects |
| 34 | 8 | parent_pid | `0` only for root; otherwise parent PID |
| 42 | 4 | child_count | Routing child count or top-graph node count |
| 46 | 4 | assignment_count | Leaf/delta row count |
| 50 | 4 | flags | Format-specific flags |

Header size SHALL be exactly 54 bytes.

## Object Kinds

| Kind | Name | Format | Payload owner |
| ---: | --- | --- | --- |
| 1 | Root | V1 | `FR-051` routing object |
| 2 | Internal | V1 | `FR-051` routing object |
| 3 | Leaf | V1 or V2 | `FR-050` leaf object |
| 4 | Delta | V1 | `FR-051` delta object |
| 5 | TopGraph | V1 | `FR-051` top-graph object |

## Validation

1. Decode SHALL reject unsupported format versions.
2. Decode SHALL reject nonzero reserved bytes.
3. Decode SHALL reject `pid = 0` and `object_version = 0`.
4. Decode SHALL reject unknown kind tags.
5. Payload decoders SHALL verify header kind, level, row counts, and flags
   against their own format-specific invariants before returning structured
   objects.

## Layout

```yaml
format: spire-partition-object-header
title: SPIRE partition object header
endianness: little
encoding: "binary, exactly 54 bytes"
record_types:
  - name: partition_object_header
    magic: 0x4f505345
    size: 54
    fields:
      - { name: magic, offset: 0, size: 4, type: u32, const: 0x4f505345, description: "'ESPO' in little-endian bytes" }
      - { name: format_version, offset: 4, size: 2, type: u16, enum: [1, 2] }
      - { name: kind, offset: 6, size: 1, type: u8, enum: { 1: root, 2: internal, 3: leaf, 4: delta, 5: top_graph } }
      - { name: reserved, offset: 7, size: 1, type: u8, const: 0 }
      - { name: pid, offset: 8, size: 8, type: u64, minimum: 1 }
      - { name: object_version, offset: 16, size: 8, type: u64, minimum: 1 }
      - { name: published_epoch_backref, offset: 24, size: 8, type: u64, description: "0 for draft routing/top-graph objects; nonzero for published leaf V2 objects" }
      - { name: level, offset: 32, size: 2, type: u16, description: "0 for leaves/deltas; positive for routing/top-graph objects" }
      - { name: parent_pid, offset: 34, size: 8, type: u64, description: "0 only for root; otherwise parent PID" }
      - { name: child_count, offset: 42, size: 4, type: u32, description: routing child count or top-graph node count }
      - { name: assignment_count, offset: 46, size: 4, type: u32, description: leaf/delta row count }
      - { name: flags, offset: 50, size: 4, type: u32, description: format-specific flags }
```

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-049-AC-1 | A binary object with invalid magic, unsupported version, unknown kind, nonzero reserved byte, zero PID, or zero object version is rejected | Test |
| FR-049-AC-2 | Every object-specific decoder validates that the common header kind and flags match the payload type being decoded | Test |
| FR-049-AC-3 | The object kind table is stable enough for an independent implementation to route encoded bytes to the correct decoder | Inspection |

### FR-049-AC-1: Header rejection

A binary partition object with an invalid magic, unsupported version, unknown
kind, nonzero reserved byte, zero PID, or zero object version is rejected.

### FR-049-AC-2: Decoder kind and flag checks

Every object-specific decoder validates that the common header kind and flags
match the payload type being decoded.

### FR-049-AC-3: Kind-table routing stability

The object kind table is stable enough for an independent implementation to
route encoded bytes to the correct decoder.

## Dependencies

- **Upstream**: FR-048 (SPIRE domain model: PIDs, object versions, epochs, and
  object kinds this header encodes).
- **Downstream**: FR-050 (leaf object payloads) and FR-051 (routing, delta, and
  top-graph payloads), which own the format-specific payloads behind this
  common header; FR-052 writes header-bearing objects during build.
