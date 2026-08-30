---
task: 231
packet: 001-plan
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 06
---

# Task 231 fixed-stride graph/vector block design freeze

Status: review-closed by Packet 005's final task-level verdict at
`../005-full-scale-decision/feedback/2026-08-30-06-reviewer.md`. The
fixed-stride layout remains opt-in only for Task 233's factorial and is not
promoted. The format and ownership contract below remains the implemented
prototype contract; its performance hypothesis was answered negatively on the
128 MiB shared-buffer lane.

This packet requests review of the implementation design at checkpoint
`4b7256f61`. Task 230 is review-closed STOP and neither Task 229 nor Task 230
will be enabled in this arm. No runtime code or measurements are under review
yet.

## Attribution boundary

The control remains the current physical-generation graph heap plus ordinary
row heap. The candidate keeps the ordinary row heap byte-for-byte and replaces
only graph/vector retrieval with:

1. a generation-local heap/B-tree directory mapping current `vec_id` to an
   owner-local dense `node_ordinal`; and
2. a PostgreSQL-managed auxiliary relation whose main fork contains
   GenericXLog-protected raw fixed-stride node pages.

The candidate node deliberately duplicates the exact vector already present in
the ordinary row heap. Expansion reads it from the node extent; finalist/source
payload materialization continues to use the ordinary row heap. Graph ordering,
placement, search budgets, codecs, payload projection, and Tasks 229/230/232
remain unchanged.

## Frozen page arithmetic

All integers are little-endian. The build descriptor persists `BLCKSZ`, every
derived byte count below, and the inputs to a 16-byte binding computed by
truncating the repository's canonical domain-separated SHA-256 digest over
`descriptor_digest || logical_index_uuid || build_id`.
Admission rejects a runtime `BLCKSZ` or derived value mismatch before opening
the candidate path.

```text
PG_PAGE_HEADER_BYTES = MAXALIGN(SizeOfPageHeaderData) = 24 at BLCKSZ=8192
NODE_PAGE_HEADER_BYTES = 80
PAGE_PAYLOAD_BYTES = BLCKSZ - PG_PAGE_HEADER_BYTES - NODE_PAGE_HEADER_BYTES
                   = 8088 at BLCKSZ=8192
NODE_HEADER_BYTES = 80
NODE_BODY_BYTES = 4*D + C + 8*R + R*C
NODE_RECORD_BYTES = NODE_HEADER_BYTES + NODE_BODY_BYTES
NODE_STRIDE_BYTES = MAXALIGN(NODE_RECORD_BYTES)
```

`D` is the persisted exact-vector dimension, `R` the graph degree, and `C` the
persisted codec stride. Every checked multiply/add converts through `u64` and
must fit `u32` and PostgreSQL block-number bounds. Zero dimensions, unsupported
degree/codec shapes, zero page capacity, or overflow fail registration.

For `NODE_STRIDE_BYTES <= PAGE_PAYLOAD_BYTES`:

```text
nodes_per_page = floor(PAGE_PAYLOAD_BYTES / NODE_STRIDE_BYTES)
block = 1 + floor(node_ordinal / nodes_per_page)
slot = node_ordinal % nodes_per_page
byte_offset = PG_PAGE_HEADER_BYTES + NODE_PAGE_HEADER_BYTES
            + slot * NODE_STRIDE_BYTES
extent_blocks = 1
```

For `NODE_STRIDE_BYTES > PAGE_PAYLOAD_BYTES`:

```text
nodes_per_page = 0
extent_blocks = ceil(NODE_STRIDE_BYTES / PAGE_PAYLOAD_BYTES)
first_block = 1 + node_ordinal * extent_blocks
segment i block = first_block + i
segment byte_offset = PG_PAGE_HEADER_BYTES + NODE_PAGE_HEADER_BYTES
```

Block zero is a relation metadata page. A packed node never crosses a page; a
multi-block node starts at its own fixed aligned extent. There is no graph-heap
CTID in the addressing contract.

## Frozen block-zero metadata

The PostgreSQL page header on block zero is followed by this 160-byte admission
record. The encoded layout descriptor is exactly 42 bytes; bytes 98..127 are
canonical zero. Its digest is calculated with bytes 128..159 zeroed.

```text
0    magic[4]             "EFM1"
4    format_version u16   1
6    metadata_bytes u16   160
8    generation_tag[16]
24   layout_digest[32]    canonical layout SHA-256 domain digest
56   layout_bytes[42]     admitted fixed-stride layout descriptor
98   reserved[30]         zero
128  metadata_digest[32]  SHA-256 domain digest of the full canonical record
```

Readers admit magic and version before consuming the version-sized record,
then validate length, reserved bytes, metadata digest, layout decode and layout
digest. The admitted layout and generation tag must equal the generation
descriptor before any node data block is exposed.

## Frozen page envelope

Each data block begins with the PostgreSQL page header followed by this 80-byte
candidate envelope:

```text
0   magic[4]             "EFS1"
4   format_version u16   1
6   page_kind u8         1=packed, 2=multi-block
7   flags u8             must be zero
8   header_bytes u16     80
10  reserved u16         zero
12  record_bytes u32     NODE_STRIDE_BYTES
16  base_ordinal u64     first packed ordinal or the extent ordinal
24  slot_count u16       packed live slots; 1 for multi-block
26  segment_index u16    zero for packed; zero-based for multi-block
28  segment_count u16    1 for packed; extent_blocks otherwise
30  content_bytes u16    initialized bytes in this page payload
32  generation_tag[16]   descriptor-derived generation binding
48  page_digest[32]      SHA-256 domain digest of zeroed-digest envelope+content
```

Reads validate the PostgreSQL page bounds, magic and admitted version before
version-sized fields, all reserved bytes, kind-specific arithmetic, generation
binding, ordinal/segment identity, content bounds, and page digest before
exposing record bytes. Every segment of a multi-block extent is independently
validated.

## Frozen node record

The 80-byte node header is followed by fixed arrays and canonical zero padding:

```text
0   magic[4]             "EFN1"
4   format_version u16   1
6   flags u16            bit 0=tombstone; all others rejected
8   header_bytes u16     80
10  neighbor_count u16   <= R
12  reserved[4]          zero
16  node_ordinal u64
24  vec_id u64
32  row_tid[6]           ordinary payload-row locator, never invalid
38  reserved[10]         zero
48  node_digest[32]      SHA-256 domain digest of header with zero digest + body
80  exact_vector[D]      IEEE-754 f32 little-endian
..  search_code[C]
..  neighbor_vec_ids[R]  u64 little-endian
..  neighbor_codes[R*C]
..  alignment padding    zero through NODE_STRIDE_BYTES
```

Unused neighbor ids/codes after `neighbor_count`, all padding, and all reserved
bytes are canonical zero. Decoding checks the ordinal and expected vec_id from
the directory, rejects non-finite vector components, verifies both digests, and
reuses caller-owned vector/code/adjacency buffers.

## Directory and publication contract

The candidate graph relation becomes a directory heap with
`(vec_id, node_ordinal, row_tid, record_version, is_current)` and retains the
partial unique B-tree on current `vec_id`. A separate catalogued
`node_store_relid` owns the raw page relation. The auxiliary relation disables
autovacuum and is opened only through the raw buffer/WAL surface, never a heap
scan. Batch lookup resolves vec_ids to ordinals in request order;
sorted/coalesced block reads then restore that order.

Build and handoff allocate ordinals monotonically from the generation's
committed record count. The node page write, directory insert, batch receipt,
and count advance share one PostgreSQL transaction. Retry sees the committed
receipt or rewrites no visible directory entry.

Published base records are immutable. Task-167 insert, replacement, neighbor
rewrite, and tombstone operations append complete replacement records at new
ordinals, then atomically flip the directory's old `is_current` row and publish
the replacement. Abort leaves the old directory row authoritative. A failed or
partially written unreferenced tail is unreachable and reclaimable with the
generation. No reader follows an unpublished ordinal and no base block is
rewritten in place.

## Lifecycle and evidence contract

Descriptor/manifest/Ready admission binds the node-layout descriptor digest,
node relation OID identity, committed node count, page digest aggregate, and
node-store byte count. Create, cancel, restart, retain, retire, rollback,
reclaim, and control-index drop include the node relation. Missing, swapped,
truncated, wrong-generation, wrong-ordinal, bad-padding, or digest-corrupt data
fails closed as `EC_GENERATION_MISSING`/`EC_FORMAT` rather than falling back to
the graph heap.

Packet 003 will contain boundary/corruption and ordered-identity tests. Packet
004 will contain restart/lifecycle/DML evidence. Packet 005 will preregister and
run counterbalanced 10k/50k/100k warm plus suite-driven controlled-residency
A/B, reporting recall, latency/tails, storage, build/DML, block/byte/buffer-hit
telemetry, and conformance. Warm and controlled-residency decisions remain
separate.

Final reconciliation: later packets reviewed the arithmetic, raw relation
ownership, directory publication, digest/admission boundary, lifecycle, DML,
and reader implementation. Packet 005 confirmed the predicted fixed-stride
storage formula at every measured node but closed the isolated experiment STOP.
