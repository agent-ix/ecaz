---
task: 231
packet: 003-format-and-reader
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 04
---

# Task 231 fixed-stride format and persisted selector checkpoint

Review code checkpoints `1e0d5906abfa5a586091ca51b4ccf1a48690f37f`
and `c644b3fb0cc7bad7027bd51d277a6578e69b81c1`, plus block-zero metadata
checkpoint `95c974ec61c1918d84136daafdfbe040f8f6ed6d`. Raw relation I/O and the
production read-path switch remain open and will be requested as later
sequences in this packet.

The new `fixed_stride` module implements the Packet 001 arithmetic and byte
contract without PostgreSQL backend dependencies:

- checked dense-ordinal addressing for packed, exactly-one-page, and aligned
  multi-block records;
- re-derivation validation for every persisted layout value;
- SHA-256-bound 80-byte page envelopes and 80-byte node headers;
- exact vector, search code, adjacency ids/codes, row locator, tombstone, and
  canonical padding in one fixed-stride node;
- version-first admission, generation/ordinal/vec-id binding, per-page and
  per-node corruption rejection, and pooled decode buffer reuse.

Focused validation is 5/5 green. It exercises all three page-fit classes,
address overflow, persisted-arithmetic drift, byte-exact node round-trip,
wrong version/identity, digest and padding corruption, every multi-block
segment, and build-identity binding. See
`artifacts/fixed-stride-format-tests.log` and its packet manifest.

Please review the checked arithmetic, version-first decode ordering, digest
coverage, padding rules, and whether the pure format API is suitable for the
next relation/WAL slice.

## Sequence 02: persisted admission selector

The second checkpoint adds:

- generation descriptor V5 with a digested
  `DistannFixedStrideLayoutDescriptorV1` and graph-record discriminator V3;
- decode admission that reads the descriptor version before its version-sized
  tail, re-derives every arithmetic field from the admitted codec artifact,
  dimensions, degree, and code stride, and rejects drift;
- opt-in `node_storage_layout='fixed_stride'`, defaulting to `graph_heap`, legal
  only for distributed control indexes and mutually exclusive with Tasks
  229/230; and
- build registration resolution that freezes the descriptor from the actual
  workspace codec stride rather than trusting user-supplied byte counts.

The combined format/descriptor gate is 6/6 green with no compiler warnings;
see `artifacts/fixed-stride-descriptor-tests.log`. Durable crash/replay binding
of the selector belongs to Packet 004 and is not claimed here.

## Sequence 03: block-zero admission record

The third checkpoint freezes and implements the 160-byte `EFM1` block-zero
metadata record called out in Packet 001 sequence 04. It binds the exact
42-byte layout descriptor, its canonical SHA-256 digest, and the 16-byte
generation tag under a second metadata digest. Decode remains version-first
and rejects length, reserved-byte, digest, layout-digest, and descriptor
arithmetic drift before relation data can be admitted.

The focused gate remains 6/6 green; the generation-binding test now includes
metadata round-trip and corruption rejection. See
`artifacts/fixed-stride-metadata-tests.log`. PostgreSQL raw-page I/O remains the
next slice in this packet.

## Sequence 04: PostgreSQL raw relation and WAL-backed reader

Code checkpoint `ee25eb7d92112697badc87944dd92ea1ee4a38e3f` adds the optional
catalogued `node_store_relid`, deterministic auxiliary relation creation with
autovacuum disabled, internal control-index ownership, and lifecycle drop /
rebuild propagation. Candidate graph stores now have the frozen directory
shape with `node_ordinal` in place of `graph_record`; control generations keep
their existing heap shape.

The new relation layer:

- initializes and admits EFM1 on block zero;
- appends packed nodes or aligned multi-block extents through GenericXLog
  full-page images;
- advances `pd_lower` through initialized raw bytes so PostgreSQL does not
  treat them as the WAL page hole;
- permits only the next dense ordinal or an idempotent rewrite of the
  unreferenced tail;
- validates metadata, every EFS1 envelope/segment, node digest and directory
  identity before returning a node; and
- reports physical blocks and bytes read for Packet 005 telemetry.

The focused PG18 fixture passes for three packed nodes and two multi-block
nodes, including retry, ordered identity, wrong-vec-id rejection, and the
autovacuum reloption. The PG18 clippy surface is also clean after explicitly
allowing two unrelated pre-existing lints in `ambuild.rs` and the Task 230
head-sizing path. See `artifacts/fixed-stride-store-pg18.log` and
`artifacts/fixed-stride-store-clippy.log`.

Handoff publication and production lookup are still open in this packet; this
sequence requests review only of raw-page durability/admission and relation
lifecycle plumbing.
