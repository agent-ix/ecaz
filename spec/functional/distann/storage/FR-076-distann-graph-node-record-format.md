---
id: FR-076
title: Distann Graph-Node Record Format and Global Identity
type: FR
status: PROPOSED
object: binary_format
relationships:
  - target: "ix://agent-ix/ecaz/FR-075"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-076: Distann Graph-Node Record Format and Global Identity

## Description

Each indexed vector SHALL be stored as exactly one graph-node record, keyed
by a global `vec_id`, containing everything a single read needs to score and
expand the node during beam search: a coarse search code, the adjacency list,
and a compressed code for each neighbor. The record SHALL NOT store the
full-precision vector; the exact distance of an expanded node is computed
from its co-placed epoch row ([FR-078](../build/FR-078-distann-hash-placement.md),
[FR-079](../read/FR-079-distann-remote-expansion-protocol.md)), so the corpus
vectors live once in the row tier and are never duplicated into the index
(ADR-085 decision D11). Records SHALL be self-describing and
format-versioned; epoch binding lives at the generation and manifest level
([FR-082](../lifecycle/FR-082-distann-epoch-lifecycle.md)), not in the
record — a record carries no epoch field, and a published record is
immutable for its generation's lifetime (ADR-085 decision D10).

This is the `ec_diskann` record shape (coarse code + adjacency; full vector
in the heap), sharded: coarse code drives beam ordering, the co-placed heap
row supplies exact rerank.

## Layout

```yaml
record: distann_graph_node
version: 1
fields:
  - { name: record_version, type: u16, rule: exactly 1; little-endian at byte offset 0 }
  - { name: flags, type: u16, rule: bit0 = tombstone }
  - { name: vec_id, type: u64, rule: identity hash per the two derivation modes in Behavior (global source-identity or default local heap-TID); unique per logical row }
  - { name: heap_tid, type: item_pointer, rule: owner-local epoch-row-tier tuple; resolves the full-precision vector for exact rerank and the frozen source payload for final materialization }
  - { name: neighbor_count, type: u16, rule: "<= graph_degree (R)" }
  - { name: search_code, type: bytes[code_stride], rule: one neighbor_code_format code for this node's own vector; used to score the node when it enters the beam }
  - { name: neighbor_vec_ids, type: u64[R], rule: first neighbor_count slots are adjacency; remaining slots are canonical zero padding }
  - { name: neighbor_codes, type: bytes[R * code_stride], rule: one fixed-stride code per live neighbor slot; remaining slots are canonical zero padding }
```

## Record Fields

| Field | Type | Rule |
|-------|------|------|
| record_version | u16 | Exactly 1, little-endian at byte offset 0; unknown and byte-swapped versions reject before any other field is interpreted |
| flags | u16 | Bit 0 = tombstone (deleted, retained until vacuum) |
| vec_id | u64 | Identity hash per the two derivation modes in Behavior; unique per logical row within the index, and across all nodes and epochs in global mode |
| heap_tid | ItemPointer | Owner-local epoch-row-tier tuple; the co-placed row it resolves ([FR-078](../build/FR-078-distann-hash-placement.md)) is the source of the node's full-precision vector for exact rerank and the frozen source payload for final materialization ([FR-079](../read/FR-079-distann-remote-expansion-protocol.md)) |
| neighbor_count | u16 | ≤ `graph_degree` (R) |
| search_code | byte[code_stride] | One `neighbor_code_format` code for this node's own vector; scores the node when it enters the beam without a heap read |
| neighbor_vec_ids | u64[R] | First `neighbor_count` slots are the adjacency list; unused slots are zero |
| neighbor_codes | byte[R × code_stride] | One scoreable code per live neighbor slot; unused slots are zero |

The fixed header is 20 bytes with offsets: `record_version=0`, `flags=2`,
`vec_id=4`, `heap_tid=12`, `neighbor_count=18`, and `search_code=20`.
The 6-byte `ItemPointer` encoding is exactly `block_number u32_le` followed by
`offset_number u16_le`, with no alignment padding. Thus the header arithmetic
is `2 + 2 + 8 + 6 + 2 = 20` bytes.
Consequently the complete record remains exactly
`20 + code_stride + (R × 8) + (R × code_stride)` bytes. The legacy local-v4
tuple's `(tag=0x09, reserved=0)` prefix is not a physical-generation version
and SHALL NOT be accepted by the physical-v1 decoder.
Physical graph-record version `9` SHALL never be assigned: its little-endian
prefix `(0x09, 0x00)` byte-collides with that legacy local tuple prefix.

The `record_version = 1` layout above is written by physical generations
only. The legacy lane (`distributed_control = false`,
[FR-075](../FR-075-ec-distann-access-method-surface.md)) SHALL continue to
encode the identical payload behind the legacy two-byte
`(tag=0x09, reserved=0)` prefix in place of `record_version`; its decoder
SHALL reject any other tag/reserved pair. The two record shapes are
lane-disjoint: legacy records decode only through the legacy decoder,
physical records only through the physical-v1 decoder.

## Handoff Entry Layout

The coordinator-to-owner handoff in
[FR-078](../build/FR-078-distann-hash-placement.md) SHALL use a versioned canonical
entry that contains no node-local physical locator. The owning node allocates
the epoch-row-tier tuple first and writes the resulting local `ItemPointer`
into the persisted graph-node record.

```yaml
record: distann_epoch_handoff_entry
version: 1
fields:
  - { name: wire_version, type: u16, rule: exactly 1 for this contract }
  - { name: vec_id, type: u64, rule: global identity of the logical source row }
  - { name: source_identity, type: length_prefixed_bytes, rule: exactly 16 canonical ADR-063 identity bytes used for collision detection; any other length rejects }
  - { name: graph_flags, type: u16, rule: build handoff accepts zero only; tombstones are a published-epoch mutation }
  - { name: search_code, type: length_prefixed_bytes, rule: exactly one code at the epoch codec stride }
  - { name: neighbor_vec_ids, type: length_prefixed_u64_array, rule: global adjacency with length <= graph_degree }
  - { name: neighbor_codes, type: length_prefixed_bytes, rule: one fixed-stride code per neighbor_vec_id }
  - { name: row_null_bitmap, type: length_prefixed_bytes, rule: one bit per non-dropped source attribute in ascending attnum order }
  - { name: row_values, type: length_prefixed_byte_array, rule: PostgreSQL binary typsend value per non-NULL, non-dropped source attribute in ascending attnum order }
```

The batch envelope SHALL use the following versioned layout:

```yaml
record: distann_epoch_handoff_batch
version: 1
fields:
  - { name: wire_version, type: u16, rule: exactly 1 }
  - { name: epoch, type: u64, rule: non-zero target epoch }
  - { name: build_id, type: uuid_bytes, rule: 16 RFC 4122 version-4 network-order bytes }
  - { name: batch_seq, type: u64, rule: starts at zero and increments by one per owner stream }
  - { name: build_spec_digest, type: byte[32], rule: SHA-256 of the immutable pre-handoff build specification }
  - { name: row_schema_fingerprint, type: byte[32], rule: FR-078 source/destination schema identity }
  - { name: index_format_version, type: u16, rule: destination graph format }
  - { name: neighbor_codec_kind, type: u8, rule: FR-076 codec discriminator }
  - { name: entry_count, type: u32, rule: number of entries in this envelope }
  - { name: encoded_entries_bytes, type: u32, rule: total of entry length prefixes plus entry bytes }
  - { name: entries, type: length_prefixed_entry[entry_count], rule: strictly increasing vec_id order }
  - { name: batch_digest, type: byte[32], rule: SHA-256 over the domain separator and every preceding field/entry byte }
```

Fixed-width integer fields SHALL use little-endian encoding except the UUID
bytes defined above. Every length prefix SHALL be an unsigned little-endian
`u32`. The row NULL bitmap SHALL contain `ceil(non_dropped_attribute_count / 8)`
bytes, with the first non-dropped attnum in the least-significant bit of byte zero and
`1 = NULL`, `0 = non-NULL`. A NULL
attribute SHALL consume no `row_values` element. Each non-NULL attribute SHALL
consume exactly one length-prefixed `row_values` element in ascending attnum
order.

The entry digest SHALL be
`SHA-256("ec_distann_handoff_entry_v1\0" || canonical_entry)`.
The batch digest SHALL be
`SHA-256("ec_distann_handoff_batch_v1\0" || canonical_batch_without_digest)`.
Raw conninfo, PostgreSQL OIDs, source heap TIDs, destination heap TIDs, and
caller-supplied send-function names SHALL NOT appear in either the entry or
batch envelope.

## Behavior

- The physical record SHALL carry `record_version` at byte offset zero and the
  generation descriptor/control metadata SHALL declare the same graph format;
  version bumps follow
  [NFR-016](../../../non-functional/NFR-016-on-disk-format-evolution-discipline.md)
  (research posture: rebuild, no migration).
- `vec_id` SHALL be stable across index rebuilds for the same logical row,
  so cross-epoch and cross-node references never alias distinct rows. The
  derivation is `vec_id = hash64(identity)` with collision handling (ADR-085
  decision D6): a pinned deterministic 64-bit hash (murmur3 fmix64
  avalanche) whose value is persisted on disk, so any hash change is an
  NFR-016 format-version bump. Build-time collision is a build error;
  insert-time collision is an insert error
  ([FR-083](../lifecycle/FR-083-distann-dml-path.md)). Placement
  ([FR-078](../build/FR-078-distann-hash-placement.md)) consumes this identity.
- The derivation SHALL support two modes (ADR-063 lineage), hashed under
  distinct domain tags so the two identity namespaces never alias:
  - **Global** (`source_identity = 'include'`): the 16-byte canonical
    ADR-063 identity payload (UUID or bytea(16) INCLUDE column) is hashed.
    Stable across index rebuilds, table rewrites, nodes, and epochs for the
    same logical row; the only mode valid for multinode placement.
  - **Local** (the default, `source_identity` absent): the row's heap TID is
    hashed. Stable across index rebuilds of an unchanged table
    (FR-076-AC-2) but NOT across table rewrites (`VACUUM FULL`, `CLUSTER`,
    rewriting `ALTER TABLE`), and unusable for multinode placement; it is a
    single-node legacy-lane convenience only.
- A distributed-control build SHALL reject the local mode: `ambuild` fails
  unless `source_identity = 'include'` names exactly one UUID or bytea(16)
  INCLUDE column, confining the local mode to the legacy lane.
- Expanding a record SHALL require exactly one index-record read to score all
  its neighbors: neighbor scoring uses the embedded codes, never a secondary
  lookup. The expanded node's exact distance SHALL come from a single read of
  its co-placed epoch row (via `heap_tid`), not from any vector stored in the
  record ([FR-079](../read/FR-079-distann-remote-expansion-protocol.md)); both reads
  are node-local (the epoch row is co-placed by [FR-078](../build/FR-078-distann-hash-placement.md)),
  so expansion adds no network round-trip.
- The record SHALL NOT carry the full-precision vector. The single
  authoritative copy of each vector lives in the co-placed epoch row tier; the
  index stores only codes and adjacency. This removes exactly 1.0× the raw
  vector bytes from per-record amplification versus an inline-vector layout
  (ADR-085 decisions D1, D11; [NFR-018](../../../non-functional/NFR-018-distann-space-amplification.md)).
- Tombstoned records SHALL remain readable (for graph traversal continuity)
  but SHALL be excluded from result sets.
- The handoff decoder SHALL reject any entry whose `source_identity` payload
  is not exactly 16 bytes: the ADR-063 canonical identity payload is pinned
  to 16 bytes on this wire, and no other length is valid.
- The handoff encoder SHALL serialize source attributes with their catalog
  `typsend` functions resolved locally from the build snapshot.
- The handoff decoder SHALL resolve matching `typreceive` functions from the
  validated destination row-tier schema.
- The handoff decoder SHALL reject the batch before writing any row when its
  row-schema fingerprint differs from the destination generation.
- The handoff digest SHALL cover the canonical source identity, graph payload,
  NULL bitmap, and row-value bytes.
- The handoff digest SHALL exclude the destination-local `ItemPointer` assigned
  while ingesting the epoch row tier.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-076-CON-1 | Total on-disk bytes SHALL stay within the NFR-018 space-amplification budget | Storage | Benchmark storage step |
| FR-076-CON-2 | neighbor_count SHALL NOT exceed the `graph_degree` reloption | Integrity | Unit test + build assertion |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-076-AC-1 | Record round-trip (encode → write → read → decode) preserves search code, adjacency, and neighbor codes byte-exactly | Test |
| FR-076-AC-2 | Two rebuilds of the same corpus assign identical vec_ids to identical source rows | Test |
| FR-076-AC-3 | Scoring a node's neighbors after one record read matches direct codec scoring of the neighbors' vectors | Test |
| FR-076-AC-4 | Tombstoned records are traversable but never returned | Test |
| FR-076-AC-5 | The encoded record layout contains no full-precision vector field (structural inspection of the decoded record) | Test |
| FR-076-AC-6 | At fixed R and fixed codec stride `S`, encoded record bytes equal `20 + S + (R × 8) + (R × S)`; dimension may affect `S` but contributes no additional full-precision `4 × dimension` field | Test |
| FR-076-AC-7 | A handoff entry round-trip preserves source identity, graph payload, NULLs, and every source-column binary value byte-exactly | Test (TC-040) |
| FR-076-AC-8 | Structural inspection proves that handoff entries contain no heap TID, PostgreSQL OID, raw conninfo, or caller-selected send function | Test (TC-040) |
| FR-076-AC-9 | A destination with a different row-schema fingerprint rejects the batch before allocating a row-tier tuple or graph record | Test (TC-040) |
| FR-076-AC-10 | Re-encoding an identical entry produces the same SHA-256 entry digest on coordinator and owner | Test (TC-040) |
| FR-076-AC-11 | With `source_identity` absent, two rebuilds of the same unchanged table assign identical local-mode vec_ids to identical rows, and local-mode and global-mode hashes of colliding inputs never alias (distinct domain tags) | Test |
| FR-076-AC-12 | A distributed-control build without `source_identity = 'include'` is rejected at build time | Test |
| FR-076-AC-13 | A handoff entry whose source-identity payload is not exactly 16 bytes is rejected before any row-tier or graph write | Test (TC-040) |
| FR-076-AC-14 | A legacy-lane record carries the `(0x09, 0x00)` prefix and decodes only through the legacy decoder, while a physical-generation record carries `record_version = 1` and decodes only through the physical-v1 decoder | Test |

## Dependencies

- **Upstream**: [FR-075](../FR-075-ec-distann-access-method-surface.md); the
  ADR-068 source-identity contract; ADR-085 decisions D1 (code duplication),
  D6 (vec_id derivation), D7 (codec choice), D11 (co-placed heap rerank)
- **Downstream**: [FR-077](../build/FR-077-distann-sharded-build-and-stitch.md),
  [FR-078](../build/FR-078-distann-hash-placement.md) (co-places the heap row),
  [FR-079](../read/FR-079-distann-remote-expansion-protocol.md)
