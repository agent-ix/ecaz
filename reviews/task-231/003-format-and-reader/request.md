---
task: 231
packet: 003-format-and-reader
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 06
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
- seals raw heap pages with `pd_lower == pd_upper == PageHeaderData`, so heapam
  sees zero line pointers while GenericXLog has a zero-length WAL page hole;
- permits only the next dense ordinal or an idempotent rewrite of the
  unreferenced tail;
- validates metadata, every EFS1 envelope/segment, node digest and directory
  identity before returning a node; and
- reports logical extent blocks and bytes touched for Packet 005 telemetry.

The focused PG18 fixture passes for three packed nodes and two multi-block
nodes, including retry, ordered identity, wrong-vec-id rejection, and the
autovacuum reloption. The PG18 clippy surface is also clean after explicitly
allowing two unrelated pre-existing lints in `ambuild.rs` and the Task 230
head-sizing path. See `artifacts/fixed-stride-store-pg18.log` and
`artifacts/fixed-stride-store-clippy.log`.

Handoff publication and production lookup are still open in this packet; this
sequence requests review only of raw-page durability/admission and relation
lifecycle plumbing.

## Sequence 05: reviewer findings, handoff, and production reader

Source checkpoint `65f166bf64664127ed7dfe52db9999145576c081` resolves the five blocking findings in
`feedback/2026-08-29-01-reviewer.md` and completes Packet 003's handoff/read
slice:

1. Every metadata/data page now presents a valid empty heap header with zero
   line pointers. The PG18 fixture audits every block, executes heapam
   `SELECT count(*)` and `ANALYZE`, and documents why database-wide VACUUM,
   anti-wraparound vacuum, pg_dump, and explicit owner/superuser reads cannot
   interpret EFM1/EFS1 bytes as `ItemId`s.
2. EFM1 and its derived layout are admitted once when a generation is opened.
   The default read path performs structural/identity checks plus direct
   arithmetic and decode; complete page/node SHA-256 and canonical-padding
   verification is retained behind the off-by-default
   `ec_distann.debug_fixed_stride_full_verification` drill GUC. Batched reads
   sort requested ordinals and decode each packed page once. The packet now
   includes a release per-node decode microbenchmark with verification off/on.
3. The V1 adjacency contract remains `neighbor_vec_ids`, deliberately. Task
   231's frozen Goal/P1 explicitly requires `vec_id -> owner-local ordinal`
   through a generation-local directory, while graph/community reordering and
   changed distributed search semantics are non-goals. DISJOINT-SHARD placement
   hashes canonical identity independently of graph edges, so its baseline
   owner-local fraction is approximately `1 / roster_size` (one third in the
   three-owner acceptance fixture), not “most.” A local-ordinal-plus-remote
   escape would add a mixed edge encoding, require globally communicated
   remote ordinals, change FR-079 request identity, and confound this layout
   A/B. The implementation instead fulfills P2: resolve the batch directory,
   sort/coalesce its admitted raw extents, and restore requested response order.
   Packet 005 will report the measured owner-local edge fraction with the suite
   evidence rather than treating the placement expectation as a measurement.
4. Telemetry is renamed `logical_blocks_touched` /
   `logical_bytes_touched`; it no longer claims physical I/O or buffer misses.
5. The relation-level PG18 matrix now injects metadata/generation mismatch,
   unpublished slots, block gaps, envelope/page digest and generation-tag
   corruption, node version/ordinal/digest/padding corruption, and truncated
   multiblock extents. The end-to-end fixture covers create, stage, seal,
   receipt/manifest binding, topology admission, abort, and node-store drop.

Packed build appends now document their page-bounded O(n^2) rehash cost;
extension documents the generation-row single-writer lock; output state is
explicitly undefined after decode error. Handoff writes dense ordinals and
whole nodes into the WAL-backed relation, the Ready receipt/manifest binds the
layout, relation OID, committed count, and committed-page digest, and retained
search consumes exact vectors directly from the admitted node record.

Please review the seq-01 finding dispositions, relation corruption/lifecycle
coverage, Ready admission, batched production reader, and the decision to
preserve the task's directory attribution boundary. Packet 004 remains the
owner of append/overlay DML and crash/restart lifecycle work.

## Sequence 06: seq-02 read-path and retry findings

Source checkpoint `a75a0bb039969c77eb8756aae05a70f29206d77a` resolves all six
items in `feedback/2026-08-30-02-reviewer.md`:

1. `logical_blocks_touched` and `logical_bytes_touched` now feed dedicated
   `distann-head-attribution-benchmark` work counters. The CLI's strict work-row
   cardinality moved with the server enum, so a missing counter fails the suite
   rather than disappearing from Packet 005.
2. Retained-generation exact vectors are cleared at each resolved batch and
   moved—not cloned—out of lookup records. Memory is bounded by the active
   batch instead of every distinct node touched during the scan.
3. Fixed-stride request dedup now uses a capacity-sized `HashSet<u64>` and is
   expected O(n), matching the graph-heap arm's hash lookup behavior.
4. Both PG18 fixed-stride fixtures assert `SHOW data_checksums = on`. The
   packet records checksum version 1 from `pg_controldata`; Packet 005 suite
   manifests must record and assert the same prerequisite because the default
   fast decoder deliberately relies on PostgreSQL's whole-page checksum for
   payload-bit corruption detection.
5. Packed retries now receive the unpublished floor inside the page writer and
   reject any truncation whose first discarded ordinal precedes that floor.
   The store fixture bypasses the outer guard and proves the page-local defense.
6. Ready evidence for packed pages hashes exactly the committed slot bytes,
   not the stored whole-page digest. The PG18 fixture changes an unreachable
   later slot and proves the committed-prefix digest remains byte-identical;
   multiblock pages retain their fully verified per-page digest binding.

Fresh PG18 results are 2/2 green for the raw store and 1/1 green for
stage/seal/Ready/topology. See the two `fixed-stride-review-seq02-*.log`
artifacts and manifest entries. Please review these dispositions as Packet
003 seq-06; Packet 004 work remains a separate checkpoint.
