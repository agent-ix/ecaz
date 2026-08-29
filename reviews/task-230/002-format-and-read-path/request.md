---
task: 230
packet: 002-format-and-read-path
agent: Codex
role: coder
model: gpt-5
date: 2026-08-28
seq: 08
---

# Task 230 packet 002 — receipt V3 and manifest V4 sealing

Review code checkpoint `5214b6d98` against the review-closed packet-001
contract and reviewer seq-07, which accepted paired handoff and authorized
receipt V3 / manifest V4 sealing. This remains a narrow packet-002 checkpoint:
it seals and binds the paired heaps, but production read admission remains the
next slice.

## Seq-08 receipt/manifest scope

- Adds append-only Ready receipt V3 while preserving V1 and Task 229 V2 bytes.
  V3 retains the logical row digest used for unchanged handoff completeness and
  appends separate initial hot/cold content digests and heap bytes. The legacy
  physical row-byte field becomes the sum of the two authoritative heaps; the
  appended fields preserve exact per-tier attribution.
- Sealing decodes Graph V2, validates both real locators and both internal
  `vec_id` echoes under the active snapshot, reconstructs source attributes in
  original attnum order, and recomputes the unchanged owner-stream/logical-row
  digests. It separately hashes each tier's NULL shape and canonical values in
  ascending vec_id order and requires equal graph/hot/cold counts.
- Adds epoch manifest V4 and fingerprint version 4. V4 binds the row-tier
  layout-descriptor digest plus roster-ordered global hot and cold initial
  content digests; the existing global logical row digest remains the build
  specification's source/handoff completeness identity.
- Build-candidate validation now binds the descriptor layout to the manifest.
  The coordinator uses the descriptor's graph-record version rather than
  hard-coding V1. Legacy V2 and covered V3 manifest/fingerprint bytes remain
  admitted unchanged.
- Extends the bootstrap Ready-receipt constraint with the exact 383-byte V3
  shape and retains exact 303-byte V1 / 351-byte V2 dispatch.
- Adds deterministic V3/V4 golden fixtures with independent field walkers,
  byte-swap rejection, production decode/re-encode equality, and V4 fingerprint
  admission. All 26 DistANN persisted-format fixtures pass.
- Closes reviewer seq-07's inherited write-layer gap: both legacy and hot/cold
  handoff now reject a NULL indexed vector before any physical write.
- Extends the PG18 handoff callback through seal and proves V3 length/version,
  nonzero distinct tier digests, positive exact per-tier bytes, and summed
  logical row storage.

## Seq-07 handoff scope

- Preserves the existing handoff wire and its frozen source-schema ordering;
  the receiver decodes each non-dropped source value once, then routes it by
  the descriptor's compact physical ordinal into the hot or cold tuple.
- Writes the cold tuple first, captures its CTID, writes the hot tuple second,
  captures its CTID, and persists both locators in the version-dispatched Graph
  V2 record. The graph row and directory remain the visibility authority.
- Gives both compact heaps their internal `vec_id` and validates the actual
  compact relation schemas against every frozen placement's type, typmod,
  collation, and binary I/O identity before admitting a batch.
- Retains the legacy full-row and Task 229 sidecar handoff paths unchanged and
  dispatches graph encoding from the generation descriptor rather than
  hard-coding V1.
- Adds a focused PG18 callback test proving the hot vector/identity and cold
  payload/generated value are materialized, the graph hot TID joins the hot
  tuple, and the decoded V2 trailer exactly equals the cold tuple CTID.
- Re-runs the legacy batch atomic-replay/directory callback as a regression
  guard; both focused PG18 tests pass at the committed SHA.

## Seq-06 relation-creation scope

- Adds nullable, globally unique `cold_tier_relid` to bootstrap and the
  0.1.1→0.1.2 upgrade. The catalog and decoder require the cold relation to be
  mutually exclusive with Task 229's sidecar pair.
- Creates compact hot and cold heaps from descriptor physical ordinals. Both
  begin with internal `vec_id bigint`; source columns use generated
  `a_{attnum}` names and retain their resolved SQL type, typmod, and collation.
- Gives the hot heap `fillfactor=100`, sets both the exact vector and source
  identity to `STORAGE PLAIN`, and verifies `pg_attribute.attstorage='p'`
  before admitting the generation.
- Preserves the control index's owner, schema, WAL persistence, effective
  tablespace, and internal dependency for both heaps. Begin replay checks the
  cataloged cold shape and every relation's existence.
- Carries the cold OID through abort, retire, cancelled-generation reclaim,
  control-index rebuild reset, and retained-generation cache invalidation.
- Extends the V4 frozen layout/descriptor fixture with a real Cold placement,
  pinning the `Cold = 2` discriminant and the new domain digest.
- Replaces the overloaded varlena-header alignment value with explicit PG int
  and internal-vec-id alignment constants and removes the redundant leading
  alignment operation identified by seq-05.
- Adds a focused PG18 test at the 1,536-dimension boundary. It verifies catalog
  exclusivity, compact schemas, ownership/persistence/schema/tablespace,
  `fillfactor=100`, both PLAIN attributes, four internal dependencies, exact
  maximal formed-tuple size, replay identity, and atomic abort cleanup.
- Updates FR-078, FR-085, and FR-087 to describe the paired relation identity,
  storage settings, and lifecycle contract.

## Prior descriptor V4 and layout-identity scope

- Preserves legacy no-cover descriptor V2 and Task 229 cover descriptor V3
  bytes, and adds descriptor V4 with a length-prefixed row-tier layout plus its
  domain-separated digest. The independent V4 fixture pins Graph V2 and the
  complete layout bytes.
- Enforces the admission matrix directly from the tuple-format constant:
  row-heap/no-cover and Task 229 cover use Graph V1; hot/cold uses Graph V2;
  cover and hot/cold remain mutually exclusive.
- Adds build-registration V3, binding the row-tier layout digest during both
  begin-build and build replay. The catalog resolver derives dimensions from
  the indexed `ecvector` typmod, because an unbuilt distributed-control index
  correctly still has zero dimensions in page metadata.
- Computes `maximum_hot_tuple_bytes` internally from the exact PG18 formed
  tuple shape, including header/bitmap MAXALIGN, per-attribute alignment,
  internal `vec_id`, four-byte-varlena exact vector, source identity, and fixed
  hot scalars. UUID identity contributes 16 bytes; `bytea(16)` contributes 20
  because the hot relation pins `attstorage='p'` and therefore cannot use a
  short-varlena header.
- Pins the indexed vector's type namespace and canonical send/receive identity,
  and adds generated-identity and persisted-inline-width drift coverage.
- Adds a focused PG18 callback test that creates hot/cold distributed-control
  indexes, registers a participant, begins the epoch build, and proves exact
  replay returns the frozen registration digest.

## Graph V2 scope

- Adds physical graph-record version 2 with `cold_tid` appended after the V1
  search code, neighbor IDs, and neighbor codes. Every existing V1 field offset
  and the complete V1 length remain unchanged.
- Adds explicit version-sized length calculation plus versioned encode,
  decode, and pooled-decode dispatch. The versioned path reads and admits the
  first two bytes before applying the selected version's length check.
- Leaves legacy tag/reserved `decode`/`decode_into` V1-sized and unchanged.
  Existing legacy and physical-V1 writers must carry `cold_tid=None`; V1 cannot
  silently discard a cold locator.
- Requires valid hot and cold owner-local TIDs for V2 and preserves canonical
  adjacency-padding validation.
- Adds `distann_graph_record_v2.hex` plus an independent fixture decoder that
  walks every field, compares the bytes after the version through the V1 end
  with the frozen V1 fixture, and reads the six-byte trailer.
- Exports the V2 format constant, trailer size, and offset helper through the
  existing benchmark/test API.
- Renames the offset helper to `distann_node_v2_cold_tid_offset` and factors the
  canonical adjacency-padding validator shared by V1 and V2.

The follow-up `9b13d2aca` replaces `Option::is_none_or` with an equivalent
Rust-1.75-compatible expression after the exact clippy gate caught the MSRV
violation. No history was rewritten.

## Seq-03 disposition

- A valid `cold_tid` is now explicitly rejected by both the physical-V1 writer
  and the legacy writer. Deleting either guard makes the new test fail.
- The public pooled physical-version decoder is now exercised by decoding V2
  into a reused tuple, observing its cold locator, then decoding V1 into the
  same tuple and proving the locator is cleared rather than retained.
- Both tests share the `distann_physical_node` prefix, so the packet's focused
  command executes all five Graph V2 tests.

## Prior descriptor slice

Reviewer seq-02 accepted code `8faac4bad` as DONE for the descriptor slice. Its
seq-01 disposition below remains for history.

## Seq-01 disposition

- `maximum_hot_tuple_bytes` is now constrained by a checked descriptor-derived
  minimum: MAXALIGN'd PG18 heap header and NULL bitmap, internal `vec_id`,
  four-byte-varlena exact vector, persisted identity inline width, and every
  declared hot scalar width. A 1,536-dimensional descriptor with a one-byte
  bound fails `validate()`.
- The descriptor now persists `source_identity_maximum_inline_bytes`, so pure
  decode validation is self-contained. Seq-05 supersedes the provisional
  short-varlena assumption here: frozen-schema validation pins that value to 16
  for UUID or 20 for PLAIN `bytea(16)`; the behavioral test rejects a UUID-sized
  bound for bytea and accepts the catalog-exact bytea bound.
- The new `options.rs` clippy error is fixed. The exact all-target PG18 clippy
  gate now reports only the five pre-existing failures named by reviewer
  seq-01 (`ambuild.rs`, `generation_descriptor.rs`, `head_sample.rs`,
  `remote_endpoint.rs`, and `ec_distann_physical_lifecycle.rs`).
- Corrupt descriptor failures now distinguish missing hot vector, missing hot
  identity, missing declared hot scalar, and relation attribute overflow.
- Frozen-schema validation rejects a generated or non-`ecvector` indexed
  vector and a generated identity. The duplicated 1,664 physical-attribute
  constant now comes from `row_schema.rs`, and the shared attnum parser no
  longer incorrectly claims every legal value must contain at least one
  attnum.

The common fixed-width scalar helper moved from `payload_sidecar.rs` to
`row_schema.rs`; Task 229 behavior and its persisted bytes are unchanged. The
stale P1 task bullet was reconciled with the accepted graph-only visibility
contract as requested by packet-001 reviewer seq-03.

## Validation

Packet-local seq-08 output and provenance are recorded in
`artifacts/manifest.md`:

- eight active receipt/manifest tests pass (one deterministic emitter ignored);
- all 26 DistANN independent on-disk fixture tests pass;
- the two-test extension upgrade matrix passes;
- the focused PG18 hot/cold handoff-and-seal callback passes;
- formatting passes, and all-target PG18 clippy records only the same five
  pre-existing repository failures, with none in a seq-08 touched line.

Prior seq-07 evidence remains recorded in the same manifest:

- the focused PG18 hot/cold handoff and Graph V2 locator test passes;
- the focused legacy PG18 stage/replay/directory regression test passes;
- formatting passes, and the all-target PG18 clippy command records only the
  same five pre-existing repository failures, with none in a seq-07 touched
  line. These two artifacts were added immediately after the seq-07 verdict to
  close the reviewer's required evidence debt.

Prior seq-06 evidence remains recorded in the same manifest:

- the focused PG18 1,536-D relation creation/replay/abort test passes;
- the independent row-layout and descriptor V4 frozen fixtures pass;
- both extension upgrade-matrix tests pass;
- formatting passes (`cargo fmt --all -- --check`).

Prior seq-05 evidence remains recorded in the same manifest:

- five row-layout tests pass;
- three descriptor unit tests and three independent descriptor fixtures pass;
- the standalone row-layout fixture, registration digest golden, and all five
  Graph V2 tests pass;
- the focused PG18 hot/cold begin-build/replay callback test passes;
- formatting passes (`cargo fmt --all -- --check`);
- the all-target PG18 clippy command records only the same five pre-existing
  repository failures already identified in seq-02; none is in a seq-05
  touched line.

No benchmark is claimed for this sealing slice. Production reads, DML, full
lifecycle, and benchmark coverage remain required before Task 230 closes.

## Review request

Please review append-only receipt/manifest dispatch, logical versus physical
tier digest separation, paired-locator sealing and reconstruction, global
roster-ordered tier digests, descriptor/manifest binding, SQL length admission,
golden fixtures, and NULL-vector rejection at `5214b6d98`. If DONE, the next
packet-002 slice will implement production exact-vector and typed payload read
admission. Legacy tag-guarded `expand.rs`, `reader.rs`, and `insert.rs` paths
remain unversioned V1.
