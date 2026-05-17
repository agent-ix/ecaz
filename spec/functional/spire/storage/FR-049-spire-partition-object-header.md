---
id: FR-049
title: SPIRE Partition Object Header
type: functional-requirement
artifact_type: FR
status: APPROVED
object: data_schema
relationships:
  - target: "ix://agent-ix/ecaz/FR-048"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-049: SPIRE Partition Object Header

## Requirement

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

## Acceptance Criteria

### FR-049-AC-1

A binary partition object with an invalid magic, unsupported version, unknown
kind, nonzero reserved byte, zero PID, or zero object version is rejected.

### FR-049-AC-2

Every object-specific decoder validates that the common header kind and flags
match the payload type being decoded.

### FR-049-AC-3

The object kind table is stable enough for an independent implementation to
route encoded bytes to the correct decoder.
