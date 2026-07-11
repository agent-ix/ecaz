---
id: FR-078
title: Distann Physical Hash Placement and Epoch Handoff
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-076"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-077"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-078: Distann Physical Hash Placement and Epoch Handoff

## Description

The ec_distann epoch build pipeline SHALL place each stitched graph-node record
on exactly one deterministic hash owner.

The epoch build pipeline SHALL place the record's frozen logical source row on
the same owner in an AM-owned epoch row tier.

The placement protocol SHALL preserve one coherent global graph while
physically distributing its records and row tier across the published roster.

In this specification, a **generation** is one participant's physical shard
for one `(logical_index_uuid, build_id, epoch)` identity. An **epoch** is the
cluster-wide unit comprising exactly one such generation for every ordered
roster participant plus its canonical manifest.

## Inputs

- The globally sorted stitched stream from
  [FR-077](./FR-077-distann-sharded-build-and-stitch.md), encoded as
  `distann_epoch_handoff_entry` values from
  [FR-076](./FR-076-distann-graph-node-record-format.md).
- One build identifier, epoch identifier, ordered roster, placement-hash
  version, row-schema fingerprint, format/codec identity, expected global
  record count, expected canonical content digest, and immutable pre-handoff
  build-specification digest.
- Every build identifier SHALL be a non-zero RFC 4122 version-4 UUID; the same
  predicate applies in the build specification, handoff batches, Ready
  receipts, and epoch manifest.
- A single PostgreSQL MVCC build snapshot of the indexed source relation.
- A coordinator-local node-descriptor registry whose ordered entries identify
  each participant, its remote logical index, and a conninfo secret reference
  without persisting raw conninfo in an epoch artifact.

## Outputs

- One hidden Building generation per roster participant containing only that
  node's owned graph records, sorted local directory, and frozen epoch row tier.
- One versioned Ready receipt per participant containing counts, digests,
  physical byte totals, and the participant/build/epoch identity.
- One topology-only placement manifest containing the ordered roster, placement
  hash version, per-node counts/digests, schema fingerprint, and receipt set.

## Coordinator Build and Node Registry

The coordinator SHALL expose these operator operations:

`ec_distann_register_node_descriptor(index_regclass regclass,
roster_ordinal integer, node_id integer, endpoint_identity text,
conninfo_secret_name text, remote_index_regclass text, is_local boolean)
RETURNS void`

`ec_distann_unregister_node_descriptor(index_regclass regclass,
roster_ordinal integer) RETURNS void`

`ec_distann_begin_epoch_build(index_regclass regclass, epoch bigint,
build_id uuid) RETURNS bytea`

`ec_distann_build_epoch(index_regclass regclass, epoch bigint, build_id uuid)
RETURNS bytea`

`ec_distann_abort_epoch_build(index_regclass regclass, build_id uuid)
RETURNS void`

`ec_distann_epoch_build_status(index_regclass regclass, build_id uuid)
RETURNS TABLE (epoch bigint, build_state text, publish_decision_state text,
node_id integer, participant_state text, next_batch_seq bigint,
record_count bigint, receipt_digest bytea, last_error_category text)`

Each participant SHALL expose these identity/recovery inspection operations:

`ec_distann_control_identity(index_regclass regclass)
RETURNS TABLE (logical_index_uuid uuid, index_format_version integer,
distributed_control boolean)`

`ec_distann_list_unpublished_generations(index_regclass regclass)
RETURNS TABLE (build_id uuid, epoch bigint, state text,
build_spec_digest bytea, generation_descriptor_digest bytea,
created_at timestamptz)`

- Registration SHALL reject duplicate ordinals, duplicate node ids, duplicate
  endpoint identities, more than one local participant, raw conninfo in any
  argument, or a remote index that is not a schema/reloption-compatible
  `distributed_control` ec_distann index.
- Version 1 node ids SHALL be in `1..=2,147,483,647`. They remain encoded as
  `u32` on canonical wires, but the restricted domain is shared by descriptors
  and PostgreSQL `integer` catalogs.
- An endpoint identity SHALL be non-empty canonical UTF-8 with no NUL or
  leading/trailing whitespace, at most 65,535 bytes, and SHALL NOT be a
  PostgreSQL URI or keyword/value conninfo string. The exact canonical value
  is the identity persisted in the generation descriptor and matched by an
  owner; it is not transport secret material.
- Registration SHALL store the conninfo secret reference only in the
  coordinator-local descriptor catalog governed by
  [NFR-014](../../../non-functional/NFR-014-spire-transport-security-and-operations.md).
- Registration SHALL resolve the secret, call the participant's secured
  `ec_distann_control_identity`, and persist the returned logical-index UUID
  only after verifying v5 `distributed_control` metadata and the configured
  endpoint identity. A caller-supplied or OID-derived UUID SHALL NOT be
  accepted as provenance.
- Registration is insert-only for one ordinal. Updating an entry requires
  `ec_distann_unregister_node_descriptor` followed by registration, and
  unregister SHALL reject while a build gate, publish decision, or active
  manifest references the registry. A build always consumes an immutable
  copied roster snapshot, so later registry edits cannot mutate an epoch.
- A distributed-control build SHALL require the ADR-063 global
  `source_identity = 'include'` provider with exactly one non-NULL UUID or
  16-byte bytea identity attribute. Heap-TID-derived local identity SHALL be
  rejected as `EC_SOURCE_IDENTITY` before snapshot capture.
- `ec_distann_begin_epoch_build` SHALL acquire a session-level source-relation
  lock that permits reads but blocks DML and schema changes, copy the ordered
  registry/reloptions/schema identity into a coordinator build registration,
  persist the durable build gate, and return its digest without contacting a
  participant.
- The transaction containing `ec_distann_begin_epoch_build` SHALL commit before
  the first remote `ec_distann_begin_epoch_handoff` call. A caller SHALL NOT
  invoke `ec_distann_build_epoch` until that commit succeeds.
- `ec_distann_build_epoch` SHALL require the matching durable registration and
  held session lock, capture one source MVCC snapshot in its new transaction,
  and consume the immutable registered roster, reloptions, and schema.
- The gate SHALL cause source DML and schema-changing DDL to fail closed if the
  coordinating session exits and releases its session lock before publish or
  abort.
- The durable build gate SHALL reject `INSERT`, `UPDATE`, `DELETE`, `MERGE`,
  `COPY FROM`, `TRUNCATE`, source-relation `ALTER`/`DROP`, `CLUSTER`,
  `VACUUM FULL`, and any other source tuple/schema rewrite. It SHALL continue to permit
  `SELECT` and non-rewriting inspection of the prior Published epoch.
- The build-to-Ready operation SHALL use one coordinator transaction and MAY
  use bounded PostgreSQL temporary files for FR-077 stitch streams.
- A successful `ec_distann_build_epoch` call SHALL return the 32-byte candidate
  manifest digest after all owners are Ready. It SHALL NOT return a Published
  fingerprint or make the generation query-visible.
- The coordinator SHALL keep the session-level relation lock across the
  build-to-Ready, publish-decision, and publish-recovery transactions.
- `ec_distann_abort_epoch_build` SHALL idempotently abort every remote
  unpublished generation, remove the coordinator build gate, and release the
  session-level lock when held by the caller.
- If the coordinator exits before a durable publish decision, recovery SHALL
  leave the prior epoch active and require explicit resume with the original
  frozen build workspace or abort. A build SHALL NOT publish from a newly
  observed source snapshot under the old build id.
- A participant SHALL list every local Building or Ready generation through
  `ec_distann_list_unpublished_generations`, including one whose coordinator
  disappeared before a remote receipt returned. An operator can therefore
  reconcile it to the durable coordinator registration or abort it explicitly;
  relation-name discovery and logs are not recovery state.
- Reinvoking the build operation with an already Published build id and exact
  immutable inputs SHALL return the existing 32-byte manifest digest. Reusing the build id
  with different inputs SHALL raise `EC_BUILD_ID_CONFLICT`.
- The build-status operation SHALL aggregate local coordinator state and
  participant status by build id, sanitize the last error to one stable
  category, and expose no row payload, source identity, raw conninfo, or secret
  reference.

## Generation Descriptor

Every owner SHALL receive and persist one canonical
`distann_generation_descriptor` before accepting batches. It SHALL use this
layout:

```yaml
record: distann_generation_descriptor
version: 1
fields:
  - { name: descriptor_version, type: u16, rule: exactly 1 }
  - { name: index_format_version, type: u16, rule: destination generation format }
  - { name: graph_record_version, type: u16, rule: FR-076 graph-node record version }
  - { name: handoff_wire_version, type: u16, rule: FR-076 handoff version }
  - { name: dimensions, type: u16, rule: indexed vector dimension }
  - { name: graph_degree, type: u16, rule: maximum record out-degree }
  - { name: placement_hash_version, type: u16, rule: FR-078 owner function version }
  - { name: roster, type: length_prefixed_array, rule: ordered entries of node_id u32, logical_index_uuid byte[16], and length-prefixed endpoint_identity; no secret references or conninfo }
  - { name: neighbor_codec_kind, type: u8, rule: FR-076 codec discriminator }
  - { name: codec_artifact, type: length_prefixed_bytes, rule: canonical codec shape plus every trained codebook/model byte required to prepare and score queries }
  - { name: row_schema_descriptor, type: length_prefixed_bytes, rule: canonical descriptor defined by Epoch Row Tier below }
  - { name: row_schema_fingerprint, type: byte[32], rule: SHA-256 identity of row_schema_descriptor }
```

Fixed-width integers SHALL use little-endian encoding and variable fields SHALL
use unsigned little-endian `u32` lengths. Every canonical array SHALL begin
with an unsigned little-endian `u32` element count; the roster then encodes
each entry as `node_id u32`, `logical_index_uuid byte[16]`, and one
length-prefixed UTF-8 endpoint identity. The descriptor digest SHALL be
`SHA-256("ec_distann_generation_descriptor_v1\0" || canonical_descriptor)`.
The handoff endpoint's `roster_digest` SHALL be
`SHA-256("ec_distann_roster_v1\0" || canonical_roster_array)`, where the
canonical roster array is the descriptor's exact `u32` count plus ordered entry
encoding above. It is a redundant early-check identity; the descriptor digest
remains the transitive owner of the roster bytes.
The immutable build-specification digest SHALL include that descriptor digest.

The `codec_artifact` SHALL begin with `artifact_version u16 = 1`,
`codec_kind u8`, `dimensions u16`, and `seed u64`. Its remaining canonical
variant SHALL be:

| Codec | Variant fields |
|-------|----------------|
| RaBitQ | `bits u8`; all other transform state is deterministically derived from the header dimensions/seed |
| TurboQuant | `bits u8`; all other transform state is deterministically derived from the header dimensions/seed |
| GroupedPQ4 | `transform_dim u32`, `sign_count u32`, IEEE-754 `f32_le[sign_count]`, `group_count u32`, `group_size u32`, `centroids_per_group u16 = 16`, then for every group in ascending group index: `centroid_value_count u32` and IEEE-754 `f32_le[centroid_value_count]` |

Every count SHALL be validated against dimensions, codec rules, and the
generation-descriptor byte length before allocation. A GroupedPQ4 artifact
SHALL have `sign_count = transform_dim`, `group_count × group_size =
transform_dim`, and `centroid_value_count = group_size × 16` for every group.

The seeded RaBitQ/TurboQuant derivation (transform-dimension rounding, ChaCha8
sign stream, SRHT/tiled-FWHT selection, RaBitQ `quant_clip` bit pattern,
TurboQuant MSE-bit selection and Lloyd-Max codebook construction) is part of
codec-artifact version 1 even though those derived values are not repeated in
the bytes. Changing any derivation requires a codec-artifact version bump and
new fixed-input code-byte/score golden vectors; a live constant change SHALL
NOT reinterpret artifact-v1 bytes.

- The codec artifact SHALL be sufficient for an owner with no source corpus to
  construct the same `DistannPreparedQuery` and score the same codes as the
  coordinator. A trained codec SHALL NOT be independently retrained on an
  owner.
- The participant SHALL validate supported versions, dimensions, degree,
  descriptor digest, codec shape, schema descriptor, and schema fingerprint
  before creating a Building generation. The descriptor digest transitively
  binds the complete codec artifact; no separate codec-artifact digest exists.
- The participant SHALL match exactly one roster entry to its local logical
  index UUID and endpoint identity, derive its owner ordinal from that entry,
  and reject a missing or ambiguous match as `EC_NODE_DESCRIPTOR`.
- The descriptor, including its codec artifact and schema descriptor, SHALL be
  immutable for the generation lifetime and SHALL contain no PostgreSQL OID,
  raw conninfo, source heap TID, or destination-local relation locator.
- Endpoint-identity text screening is defense-in-depth only. The security
  boundary is authenticated node registration: it resolves a secret reference,
  verifies the remote control identity, and persists the separately validated
  endpoint identity. A string blocklist SHALL NOT be treated as proof that
  arbitrary text contains no disguised conninfo.

## Physical Placement

- The placement function SHALL compute the owner as
  `placement_hash(vec_id, hash_version) mod roster_count`.
- For `hash_version = 1`, `placement_hash` SHALL be the unsigned 64-bit
  MurmurHash3 finalizer `fmix64` applied to
  `u64(vec_id) XOR 0x64697374616e6e70`. Every multiplication wraps modulo
  `2^64`, and the steps are exactly: `h ^= h >> 33`; `h *=
  0xff51afd7ed558ccd`; `h ^= h >> 33`; `h *= 0xc4ceb9fe1a85ec53`;
  `h ^= h >> 33`. TC-050 SHALL pin at least the vec_id/hash vectors
  `0→0x2046ffe66003c942`, `1→0x19ec555c128bedc0`,
  `42→0x3ccbc4b5c8aa40ea`, `u64::MAX→0x00498282650ac8a8`, and
  `0xdeadbeefcafef00d→0xdaa2d74d76f4450a`. With `roster_count = 3`, those
  vectors SHALL resolve to owner ordinals `2, 1, 0, 2, 0`, respectively.
- The epoch manifest SHALL preserve the exact roster order used by the
  placement function.
- The placement manifest SHALL contain no per-record routing entries.
- Each published vec_id SHALL have exactly one physical graph-node record across
  the roster.
- Each published vec_id SHALL have exactly one physical epoch-row-tier tuple
  across the roster.
- Each graph-node record SHALL reference its co-located row-tier tuple through a
  destination-local epoch-scoped `ItemPointer`.
- Each record's adjacency SHALL preserve global neighbor vec_ids without
  owner-based filtering or edge rewriting.
- Each serving index directory SHALL contain only vec_ids owned by that roster
  participant.
- A coordinator outside the serving roster SHALL store no graph-node shard.
- A coordinator inside the serving roster SHALL store only its own graph-node
  shard.
- A replicated full index hidden behind serving-ownership filtering SHALL NOT
  satisfy this requirement.
- A full index whose non-owned records are merely tombstoned SHALL NOT satisfy
  this requirement.

## Epoch Row Tier

- A multi-node build SHALL store the source tuple in an AM-owned heap relation
  scoped to one build identifier and epoch.
- The epoch row tier SHALL preserve every non-dropped source attribute needed
  for final tuple materialization and coordinator-side qual evaluation.
- The epoch row tier SHALL preserve NULLs and PostgreSQL binary values in
  ascending source-attnum order.
- The indexed full-precision vector in the row tier SHALL be byte-identical to
  the vector observed by the build snapshot.
- The row-tier schema fingerprint SHALL cover attribute order, names, qualified
  type identities, typmods, qualified collation identities, dropped/generated
  flags, and the resolved binary send/receive identities.
- The row-tier schema fingerprint SHALL be
  `SHA-256("ec_distann_row_schema_v1\0" || canonical_schema_descriptor)`.
- The canonical row-schema descriptor SHALL begin with
  `descriptor_version u16 = 1` and `physical_attribute_count u16`.
- For every physical attribute in ascending positive attnum, including dropped
  attributes, the descriptor SHALL encode: `attnum u16`, length-prefixed UTF-8
  attribute name, length-prefixed UTF-8 type namespace, length-prefixed UTF-8
  type name, `typmod i32`, length-prefixed UTF-8 collation namespace,
  length-prefixed UTF-8 collation name, `dropped u8`, `generated_kind u8`,
  length-prefixed UTF-8 send-function namespace/name, and length-prefixed UTF-8
  receive-function namespace/name.
- A dropped attribute SHALL use one canonical empty form: empty name, type,
  collation, send-function, and receive-function strings; `typmod = -1`; and
  `generated_kind = 0`. Historical catalog residue SHALL NOT enter its schema
  fingerprint.
- A field with no collation, send function, or receive function SHALL encode an
  empty length-prefixed string in that position; absence SHALL NOT remove a
  field from the descriptor.
- The canonical schema descriptor SHALL encode each field as a little-endian
  `u32` byte length followed by UTF-8 bytes, except numeric attnum/typmod/flag
  fields which use fixed-width little-endian integers.
- The canonical schema descriptor SHALL identify types, collations, and binary
  functions by qualified namespace/name rather than PostgreSQL OID.
- The destination node SHALL resolve binary receive functions from its own
  catalogs after validating the schema fingerprint.
- If a non-dropped source attribute lacks binary send or receive support on any
  participant, then the coordinator SHALL fail the build as
  `EC_SCHEMA_UNSUPPORTED` before beginning handoff.
- The destination node SHALL NOT execute a function name supplied by the
  coordinator.
- The epoch row tier SHALL store generated-column values captured by the build
  snapshot without re-evaluating generation expressions on a participant.
- In a multi-node scan, system-column projections or quals SHALL be rejected as
  `EC_UNSUPPORTED_PROJECTION` unless a later FR defines their distributed
  identity.
- A Published row-tier tuple SHALL remain immutable until its epoch is retired
  under [FR-082](./FR-082-distann-epoch-lifecycle.md).
- `distributed_control=true` SHALL always create an AM-owned frozen row tier,
  including a one-owner degenerate roster. Only the legacy
  `distributed_control=false` single-node lane MAY reference an indexed
  base-table tuple directly.

## Handoff Protocol

Every participant SHALL expose the following internal SQL operations over the
secured pooled libpq transport governed by
[NFR-014](../../../non-functional/NFR-014-spire-transport-security-and-operations.md):

`ec_distann_begin_epoch_handoff(index_regclass regclass, epoch bigint,
build_id uuid, build_spec_digest bytea, roster_digest bytea,
generation_descriptor bytea, generation_descriptor_digest bytea,
expected_owner_count bigint, expected_owner_digest bytea) RETURNS TABLE (state text, next_batch_seq bigint,
cumulative_record_count bigint, cumulative_owner_digest bytea)`

`ec_distann_stage_epoch_batch(index_regclass regclass, build_id uuid,
batch_seq bigint, batch_digest bytea, encoded_batch bytea) RETURNS TABLE
(accepted_record_count bigint, cumulative_record_count bigint,
cumulative_owner_digest bytea)`

`ec_distann_seal_epoch_handoff(index_regclass regclass, build_id uuid,
expected_owner_count bigint, expected_owner_digest bytea) RETURNS bytea`

`ec_distann_abort_epoch_handoff(index_regclass regclass, build_id uuid)
RETURNS void`

| Operation | Required inputs | Observable result |
|-----------|-----------------|-------------------|
| `ec_distann_begin_epoch_handoff` | index relation, epoch, UUID build id, build-spec/roster/generation-descriptor digests and descriptor bytes, expected owner count and owner digest | Building-state receipt with the next batch sequence and cumulative count |
| `ec_distann_stage_epoch_batch` | index relation, build id, monotonically increasing batch sequence, SHA-256 batch digest, versioned handoff entries | committed batch receipt with accepted count, cumulative count, and cumulative digest |
| `ec_distann_seal_epoch_handoff` | index relation, build id, expected final count and owner digest | Ready receipt with graph, row-tier, directory, and physical-byte digests/counts; schema is bound transitively by the generation-descriptor digest |
| `ec_distann_abort_epoch_handoff` | index relation and build id | idempotent removal of a non-Published Building or Ready generation |

- The begin operation SHALL return the existing progress receipt when the same
  build identity and immutable parameters are replayed.
- If a build identifier is replayed with different immutable parameters, then
  the begin operation SHALL raise `EC_BUILD_ID_CONFLICT` without mutation.
- The coordinator SHALL retain at most one unacknowledged batch per owner.
- The coordinator SHALL preserve strictly increasing vec_id order within each
  owner stream.
- During initial snapshot capture, before Vamana construction, stitching, or
  the first participant handoff, the coordinator SHALL serialize and preflight
  every frozen source row against the complete handoff-entry size formula
  (row bytes plus fixed graph payload for the selected degree/codec). If one
  canonical entry can exceed 8 MiB, the build SHALL fail immediately as
  `EC_HANDOFF_TOO_LARGE`. Entry chunking is not supported in v1; this is an
  explicit corpus constraint rather than a mid-handoff discovery.
- The participant SHALL verify the batch digest before decoding entries.
- The participant SHALL validate wire version, schema, codec shape, vector
  dimension, owner, vec_id order, and duplicate absence before writing a batch.
- The participant SHALL write each row-tier tuple, graph record, and local
  directory entry in one PostgreSQL transaction.
- If any entry in a batch is invalid, then the participant SHALL roll back the
  entire batch.
- The participant SHALL return the prior receipt when an acknowledged batch is
  replayed with the same sequence and digest.
- If an acknowledged sequence is replayed with different bytes or digest, then
  the participant SHALL raise `EC_BATCH_CONFLICT` without mutation.
- If a sequence skips the participant's next expected value, then the
  participant SHALL raise `EC_BATCH_SEQUENCE` without mutation.
- The seal operation SHALL reject missing sequences, count disagreement,
  digest disagreement, duplicate vec_ids, non-owned vec_ids, and row/record
  count disagreement.
- The seal operation SHALL make the generation Ready but query-invisible.
- The expected owner digest SHALL be
  `SHA-256("ec_distann_owner_stream_v1\0" || length_prefixed_entries)` over the
  owner's canonical handoff entries in vec_id order.
- The expected owner digest SHALL exclude participant-local row-tier and graph
  `ItemPointer` values.

The canonical immutable build specification SHALL contain these fields in
order:

| Field | Wire type | Rule |
|-------|-----------|------|
| build_spec_version | u16 | exactly 1 |
| epoch | u64 | target epoch |
| build_id | byte[16] | UUID RFC 4122 version-4 bytes |
| parent_fingerprint | length-prefixed bytes | empty or one retained FR-082 fingerprint |
| source_snapshot_digest | byte[32] | FR-082 canonical snapshot identity |
| generation_descriptor_digest | byte[32] | descriptor above, binding roster/formats/codec/schema |
| build_options | length-prefixed bytes | `build_list_size u16`, IEEE-754 `alpha f32_le`, `seed u64`, IEEE-754 `closure_epsilon f32_le`, `head_index_cap u32`, and `build_shards u32`; zero build_shards means FR-077 auto selection; negative-zero closure epsilon is non-canonical and rejected |
| expected_global_count | u64 | exact source vec_id count |
| expected_global_graph_digest | byte[32] | canonical stitched graph content |
| expected_global_row_tier_digest | byte[32] | canonical source row payload content |
| head_sample_digest | byte[32] | canonical coordinator head sample |
| owner_expectations | length-prefixed array | one roster-ordered `(node_id u32, expected_count u64, expected_owner_digest byte[32])` entry per owner |

The frozen version-1 validity domain is: graph degree `4..=256`, build-list size
`10..=1000`, alpha `1.0..=2.0`, closure epsilon `+0.0..=1.0`, head-index cap
`16..=1,048,576`, and build-shard count `0..=4096`. These are format-decoder
bounds, not references to mutable reloption constants. Later reloption tuning
SHALL NOT make existing version-1 bytes invalid or broaden what an older
version-1 decoder accepts.

The build-specification digest SHALL be
`SHA-256("ec_distann_build_spec_v1\0" || canonical_build_specification)`.
Fixed-width fields and length prefixes SHALL follow the same integer/UUID rules
as the generation descriptor. `owner_expectations` SHALL begin with its `u32`
element count and then encode the three fixed-width fields for each roster
entry without per-entry byte lengths. The build specification SHALL contain no raw
conninfo, secret reference, PostgreSQL OID, or local physical locator.
- Because `owner_expectations` bind final per-owner counts and stream digests,
  v1 SHALL finish the bounded source/stitched spools and make a counting/digest
  pass before the first participant `begin`. This deliberate second pass keeps
  immutable begin/replay identities complete; overlapping handoff with stitch
  output is deferred to a later version.
- The final epoch-manifest digest SHALL be computed only after all Ready
  receipts exist under [FR-082](./FR-082-distann-epoch-lifecycle.md).
- PostgreSQL WAL SHALL preserve every acknowledged batch across backend or
  instance restart.
- An unacknowledged transaction SHALL disappear during PostgreSQL crash
  recovery.

## Topology Inspection

Every participant SHALL expose these read-only operations with the same result
shape:

`ec_distann_generation_topology(index_regclass regclass, build_id uuid)
RETURNS TABLE (node_id integer, state text, record_count bigint,
row_count bigint, owned_vec_id_digest bytea, graph_digest bytea,
row_tier_digest bytea, non_owned_live_count bigint,
non_owned_tombstone_count bigint, orphan_record_count bigint,
orphan_row_count bigint, graph_bytes bigint, row_tier_bytes bigint,
directory_bytes bigint, control_index_bytes bigint)`

`ec_distann_epoch_topology(index_regclass regclass, epoch_fingerprint bytea)
RETURNS TABLE (node_id integer, state text, record_count bigint,
row_count bigint, owned_vec_id_digest bytea, graph_digest bytea,
row_tier_digest bytea, non_owned_live_count bigint,
non_owned_tombstone_count bigint, orphan_record_count bigint,
orphan_row_count bigint, graph_bytes bigint, row_tier_bytes bigint,
directory_bytes bigint, control_index_bytes bigint)`

- The operation SHALL derive every count and digest from the selected physical
  generation, not from expected manifest fields.
- `owned_vec_id_digest` SHALL hash the strictly sorted local vec_id sequence as
  `SHA-256("ec_distann_owned_vec_ids_v1\0" || u64_le(vec_id)...)`.
- `graph_bytes`, `row_tier_bytes`, and `directory_bytes` SHALL include their
  attributed TOAST storage and exclude the logical control index.
  `control_index_bytes` SHALL separately report the local logical control
  relation so NFR-018 can sum it into the graph-side numerator without
  conflating it with generation storage.
- The coordinator SHALL use `ec_distann_generation_topology` to verify each
  build-id-selected Ready generation before persisting a publish decision. The
  function MAY report Building state for operator diagnostics, but a publish
  decision SHALL accept only Ready.
- The suite and scan diagnostics SHALL use `ec_distann_epoch_topology` for a
  Published or retained Retired fingerprint. Building and Ready state are
  selectable only by build id through `ec_distann_generation_topology`; bytes
  that do not resolve to a Published/retained manifest, and reclaimed
  fingerprints, SHALL fail closed on the epoch-selected operation.

## Error Conditions

| Code | Condition | Required outcome |
|------|-----------|------------------|
| `EC_BUILD_ID_CONFLICT` | Reused build id with different immutable build parameters | Reject before mutation |
| `EC_NODE_DESCRIPTOR` | Roster ordinal/id/endpoint is duplicate, malformed, secret-bearing, or incompatible with the remote control index | Reject before catalog or remote mutation |
| `EC_SOURCE_IDENTITY` | Physical build lacks one valid global UUID/bytea16 source identity per row | Reject before snapshot capture or handoff |
| `EC_BATCH_SEQUENCE` | Gap, regression, or out-of-order batch sequence | Reject before mutation |
| `EC_BATCH_CONFLICT` | Replayed sequence has different digest or bytes | Reject before mutation |
| `EC_HANDOFF_DIGEST` | Supplied batch bytes do not match the batch digest | Roll back the current batch; generation remains resumable |
| `EC_WRONG_OWNER` | Entry hashes to another roster participant | Roll back the entire batch |
| `EC_DUPLICATE_VEC_ID` | Duplicate vec_id within or across acknowledged batches | Roll back the entire batch |
| `EC_SCHEMA_MISMATCH` | Source and destination row-tier schema fingerprints differ | Reject before tuple allocation |
| `EC_SCHEMA_UNSUPPORTED` | A required attribute lacks compatible binary send/receive support | Reject before handoff begins |
| `EC_GENERATION_DESCRIPTOR` | Descriptor/digest/version/codec/schema content is malformed, unsupported, or inconsistent | Reject before creating or mutating a generation |
| `EC_UNSUPPORTED_PROJECTION` | Multi-node query references an unspecified system-column identity | Reject during planning |
| `EC_HANDOFF_FORMAT` | Unknown wire/record/codec version or malformed entry | Roll back the entire batch |
| `EC_HANDOFF_TOO_LARGE` | Preflight shows one entry can exceed 8 MiB, or an encoded batch exceeds 8 MiB | Reject before graph construction/remote begin for an entry, or before declared-size allocation for a batch |
| `EC_BUILD_INCOMPLETE` | Seal observes missing sequence, count, row, directory, or final owner digest | Keep generation Building and query-invisible |
| `EC_BUILD_STATE` | Operation is invalid for the generation state | Reject without changing the state |

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-078-CON-1 | Each encoded batch SHALL be at most 8 MiB | Resource | Boundary test (TC-040) |
| FR-078-CON-2 | The coordinator SHALL keep at most one unacknowledged batch per owner | Resource | Instrumented integration test (TC-040) |
| FR-078-CON-3 | Owner streams SHALL be strictly increasing by vec_id | Integrity | Unit and integration test (TC-040) |
| FR-078-CON-4 | A participant SHALL persist zero non-owned graph records in a Ready or Published generation | Integrity | Topology audit (TC-040, TC-044) |
| FR-078-CON-5 | Handoff peak coordinator buffering SHALL be bounded by `8 MiB × roster_count` plus one FR-077 stitch group and fixed codec scratch | Resource | Peak-memory result row (TC-040) |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-078-AC-1 | Placement of every vec_id is identical on coordinator and participants for a fixed manifest | Test (TC-040) |
| FR-078-AC-2 | At 100k across three owners, each owned-record count is within 10% of `100000 / 3` | Analysis (TC-044) |
| FR-078-AC-3 | Changing roster order or membership creates a different epoch; placement within the old Published epoch remains unchanged | Test (TC-042) |
| FR-078-AC-4 | Every graph record resolves exactly one node-local row-tier tuple containing the build-snapshot vector and source payload | Test (TC-040) |
| FR-078-AC-5 | Across the roster, graph-record and row-tier vec_id unions equal the source corpus and pairwise intersections are empty | Test (TC-040, TC-044) |
| FR-078-AC-6 | Replaying begin or an acknowledged batch with identical parameters returns the prior receipt without adding records or bytes | Test (TC-040) |
| FR-078-AC-7 | Conflicting replay, wrong-owner, duplicate, malformed, oversize, schema-mismatch, unsupported-schema, and unsupported-projection inputs leave counts and physical bytes unchanged | Test (TC-040) |
| FR-078-AC-8 | After a participant PostgreSQL restart while the coordinator retains the source lock and build transaction, handoff resumes at the first unacknowledged sequence and seals to the same owner digest as an uninterrupted build | Test (TC-042) |
| FR-078-AC-9 | Final materialization from the epoch row tier reconstructs projected values and coordinator quals identically to the build-snapshot source rows | Test (TC-040) |
| FR-078-AC-10 | Topology audit reports zero non-owned live records, zero non-owned tombstoned records, exact corpus coverage, and one co-located row per record | Test (TC-040, TC-044) |
| FR-078-AC-11 | The handoff never buffers a complete epoch and never exceeds the batch or peak-memory constraints | Test (TC-040) |
| FR-078-AC-12 | A coordinator outside the roster holds no graph records; a coordinator inside the roster holds only its hash-owned records | Test (TC-040) |
| FR-078-AC-13 | A trained-codec generation descriptor round-trips byte-exactly and makes owner query preparation/scoring identical to the coordinator without owner-side retraining | Test (TC-040, TC-050) |

## Dependencies

- **Upstream**: [FR-076](./FR-076-distann-graph-node-record-format.md),
  [FR-077](./FR-077-distann-sharded-build-and-stitch.md), and
  [NFR-014](../../../non-functional/NFR-014-spire-transport-security-and-operations.md)
- **Downstream**: [FR-079](./FR-079-distann-remote-expansion-protocol.md),
  [FR-082](./FR-082-distann-epoch-lifecycle.md), and
  [NFR-020](../../../non-functional/NFR-020-distann-fault-behavior.md)
