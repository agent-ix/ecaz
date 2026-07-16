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

`ec_distann_configure_participant_identity(index_regclass regclass,
endpoint_identity text) RETURNS void`

`ec_distann_control_identity(index_regclass regclass)
RETURNS TABLE (logical_index_uuid uuid, index_format_version integer,
distributed_control boolean, compatibility_digest bytea,
endpoint_identity text, canonical_index_regclass text)`

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
- A version-1 endpoint identity SHALL match the exact ASCII grammar
  `[A-Za-z0-9][A-Za-z0-9._/-]{0,254}`. A conninfo secret reference SHALL match
  `[A-Z][A-Z0-9_]{0,127}`. The latter is an injective input to the environment
  provider key `EC_SPIRE_REMOTE_CONNINFO_<reference>`; case-folding or
  punctuation aliases are not accepted. Both grammars exclude whitespace,
  `=`, URI schemes, quoting, and every libpq keyword/value form by construction
  rather than by a keyword blocklist. The endpoint identity is not transport
  secret material.
- Before a control can be registered as a participant, its owner SHALL call
  `ec_distann_configure_participant_identity`. The operation is insert-only for
  one logical-index UUID: an exact replay is idempotent and a different value
  raises `EC_NODE_DESCRIPTOR`. The identity is stored in a participant-local
  durable catalog, survives restart, and is removed by DROP or destructive
  REINDEX together with the old logical UUID. A session GUC or relation name
  SHALL NOT supply this identity.
- `ec_distann_control_identity` SHALL return that configured identity and the
  server-canonical schema-qualified index locator. An unconfigured control MAY
  return NULL identity for inspection but SHALL be rejected by registration.
  The version-1 canonical locator SHALL contain exactly two unquoted lower-case
  PostgreSQL identifiers matching
  `[a-z_][a-z0-9_]{0,62}\.[a-z_][a-z0-9_]{0,62}`. Registration SHALL persist
  the returned canonical locator, never the caller's locator spelling. Names
  that PostgreSQL must quote because they are reserved words are outside this
  v1 locator grammar even when their characters match the regular expression.
- Registration SHALL store the conninfo secret reference only in the
  coordinator-local descriptor catalog governed by
  [NFR-014](../../../non-functional/NFR-014-spire-transport-security-and-operations.md).
- Registration SHALL resolve the secret, call the participant's secured
  `ec_distann_control_identity`, and persist the returned logical-index UUID
  and returned canonical index locator only after verifying v5
  `distributed_control` metadata and exact equality between the requested and
  returned configured endpoint identity. A caller-supplied or OID-derived UUID,
  an unqualified/quoted locator, or a caller-only endpoint label SHALL NOT be
  accepted as provenance.
- `compatibility_digest` SHALL be
  `SHA-256("ec_distann_control_compatibility_v1\0" || canonical_body)`. The
  canonical body SHALL contain, in order, `compatibility_version u16 = 1`,
  graph degree `u16`, build-list size `u16`, alpha IEEE-754 `f32_le`, codec seed
  `u64`, neighbor-codec kind `u8`, head-index cap `u32`, closure epsilon
  IEEE-754 `f32_le`, source-identity provider `u8 = 1` (`include`), indexed
  vector attribute number `u16`, indexed-key kind `u8`
  (`1 = extension-owned ecvector inner-product opclass`, `2 =
  extension-owned tqvector inner-product opclass`), identity attribute number
  `u16`, identity base kind `u8` (`1 = uuid`, `2 = bytea16`), identity
  `attnotnull u8 = 1`, and the 32-byte row-schema fingerprint. The key shall be
  one base-table attribute using the named extension-owned opclass; expression,
  custom, or shadow-schema opclasses are incompatible. Registration SHALL
  compare this digest with the coordinator control before inserting a
  descriptor. The digest excludes the logical UUID and endpoint identity so
  compatible controls on distinct participants compare equal.
- A `distributed_control=true` indexed vector base column SHALL have a concrete
  dimensional typmod. Untyped `ecvector`/`tqvector` keys are rejected as
  `EC_SCHEMA_UNSUPPORTED`; version 1 never defers dimensional compatibility to
  a later handoff batch.
- `ec_distann_node_descriptor` is the desired roster for the next build, not
  historical routing state. Registration inserts one immutable desired entry
  and rejects conflicting ordinal, node id, endpoint, participant UUID, or
  local-participant values. Unregister removes the desired entry. Replacing an
  ordinal is one operator transaction containing unregister then register, so
  another build observes either complete roster and never the intermediate
  gap.
- Every control SHALL have one durable registry-state row with a monotonically
  increasing revision. Configure, register, unregister, and begin-build SHALL
  serialize on the same coordinator-control relation lock and lock this row
  `FOR UPDATE`; register/unregister increment its revision in their transaction.
  The coordinator-control relation lock SHALL remain held until transaction
  end, including across an operator transaction that calls unregister and then
  register to replace one ordinal. Releasing it when either SQL function
  returns is forbidden because a waiter could otherwise acquire the relation
  lock, wait on the registry row, and deadlock the replacement call through
  inverted lock ownership.
  Under READ COMMITTED a waiter sees the committed roster. Under Repeatable
  Read or Serializable, a registry-state change after the caller's snapshot
  SHALL cause PostgreSQL serialization failure rather than a stale edit.
- `ec_distann_begin_epoch_build` SHALL copy every desired descriptor, including
  its secret reference, canonical remote index locator, compatibility digest,
  and local flag, into a private build-participant binding keyed by build id
  and ordinal. It SHALL also bind the registry revision. Manifests continue to
  contain only the public roster `(node_id, UUID, endpoint_identity)`.
  Publication, reads, recovery, and retirement SHALL use the build-specific
  private bindings, never the mutable desired roster.
- Unregister SHALL reject a desired entry already captured by a Registered,
  Building, Ready, or decided-but-unapplied build. It MAY remove an entry used
  only by Published/retained epochs because those epochs have immutable private
  bindings. Build-participant bindings remain until their last epoch is
  reclaimed; active/retained fingerprint lookup SHALL resolve unambiguously to
  its build id before transport.
- A distributed-control build SHALL require the ADR-063 global
  `source_identity = 'include'` provider with exactly one non-NULL UUID or
  16-byte bytea identity attribute. Heap-TID-derived local identity SHALL be
  rejected as `EC_SOURCE_IDENTITY` before snapshot capture.
- `ec_distann_begin_epoch_build` SHALL acquire a session-level source-relation
  lock that permits reads but blocks DML and schema changes, copy the ordered
  registry/reloptions/schema identity into a coordinator build registration,
  persist the durable build gate, and return its digest without contacting a
  participant.
- The build registration SHALL persist the local source relation OID alongside
  the coordinator control-index OID, logical-index UUID, and build id so DML
  and utility hooks can locate the
  durable gate without consulting mutable names. `Registered`, `Building`,
  `Ready`, and `Decided` registrations are gate-active; `Aborted` and
  `Published` registrations are not. A stale OID without the matching live
  control-index UUID SHALL never gate a reused relation.
- Begin-build SHALL acquire the source session `ShareLock` before retaining the
  coordinator-control session `ShareRowExclusiveLock`, then lock the
  registry-state row before any registration/replay row. It MAY resolve the source OID under a
  short-lived control-index `AccessShareLock`, but it SHALL reopen and revalidate
  the v5 control metadata and logical UUID after the source lock is held. The
  short-lived `AccessShareLock` SHALL be released before acquisition of the
  retained source lock begins. This
  source → control → registry → registration order is shared by abort and
  publish recovery and prevents inversion with source DDL.
- Successful begin-build retains both session locks through build, decision,
  and active-pointer-swap commit. The control lock prevents concurrent control
  mutation/DROP/REINDEX and gives one live coordinator backend ownership. A
  second backend attempting the same or another build while that ownership is
  live SHALL fail non-blockingly with `EC_BUILD_BUSY`; after owner exit, durable
  recovery may reacquire both locks in the same order. Desired-roster edits are
  therefore serialized behind the live build; immutable private bindings remain
  required for retained epochs and post-publish roster changes.
- A session source lock acquired by a subtransaction SHALL be promoted to its
  parent only when that subtransaction commits. Subtransaction or top-level
  abort SHALL release every newly acquired nontransactional session lock.
  Committed lock ownership SHALL cover both relations and be keyed by source
  relation, coordinator control identity, and build id; another build in the
  same session SHALL not borrow it.
- PostgreSQL releases all default-lock-manager session relation locks when a
  backend aborts a top-level transaction, including source/control locks that
  backend committed in an earlier transaction. Such an abort loses only
  ephemeral ownership: the backend SHALL clear its local ownership mirror, the
  durable build gate SHALL remain authoritative, and any backend resuming the
  build SHALL reacquire source then control locks and revalidate the exact
  registration before a side effect. Durable DML and utility-hook enforcement
  SHALL cover the interval between abort and reacquisition. A subtransaction
  abort releases only session locks acquired by that subtransaction; a
  subcommit promotes its ownership record to the parent.
- The returned registration digest SHALL bind the complete immutable private
  binding list in ordinal order, including secret reference, canonical remote
  locator, participant UUID, compatibility digest, endpoint identity, node id,
  ordinal, and local flag. Exact replay SHALL lock and reconstruct the complete
  registration and binding list; a count-only replay check is forbidden.
- Registration-digest version 1 SHALL encode, in order: `version u16 = 1`,
  coordinator control-index OID `u32`, coordinator logical-index UUID
  `byte[16]`, source relation OID `u32`, epoch `u64`, build id `byte[16]`,
  registry revision `u64`, length-prefixed public roster snapshot, roster digest
  `byte[32]`, row-schema fingerprint `byte[32]`, compatibility digest `byte[32]`,
  private-binding count `u32`, then each ordinal-ordered binding as ordinal
  `u32`, node id `u32`, length-prefixed endpoint identity, secret reference,
  canonical remote locator, participant UUID `byte[16]`, compatibility digest
  `byte[32]`, and local flag `u8`. Its result SHALL be
  `SHA-256("ec_distann_build_registration_v1\0" || canonical_bytes)` and is a
  coordinator-local identity: OIDs SHALL NOT enter a participant descriptor,
  manifest, or remote request.
- At most one registration in `Registered`, `Building`, `Ready`, or `Decided`
  state SHALL exist per `(index_oid, logical_index_uuid)`. A second build id or
  epoch raises `EC_BUILD_STATE`; exact replay is only the same build id. Epoch
  identity SHALL additionally be unique per logical index for the retained
  lifetime through final reclaim.
- Version 1 also rejects begin-build while any publish decision for the logical
  index is `Pending` or `Activated`. Recovery is therefore unambiguous for the
  build-id-free `ec_distann_recover_epoch_publish(index)` operation; a later
  build may begin only after the prior decision is `Applied`. An audited
  predecessor-binding abandonment described by FR-082 is one terminal binding
  outcome that permits the covering decision to become `Applied`; it does not
  bypass or delete the publish decision.
- The transaction containing `ec_distann_begin_epoch_build` SHALL commit before
  the first remote `ec_distann_begin_epoch_handoff` call. A caller SHALL NOT
  invoke `ec_distann_build_epoch` until that commit succeeds.
- `ec_distann_build_epoch` SHALL require the matching durable registration and
  held or safely reacquired build-specific session lock, capture one source
  MVCC snapshot in its new transaction,
  and consume the immutable registered roster, reloptions, and schema.
- In the physical-generation lane, the coordinator SHALL supply its one
  registered MVCC snapshot to `table_index_build_scan` using concurrent-build
  visibility semantics. PostgreSQL therefore filters recently-dead/invisible
  rows before the callback and reports `tuple_is_alive = true` for rows it does
  deliver. The callback SHALL nevertheless exclude a defensive
  `tuple_is_alive = false` invocation before vector, identity, or row-payload
  access. Old snapshots continue to use the prior Published epoch; the new
  frozen epoch contains only rows visible in its single captured snapshot. The
  legacy non-control AM lane retains PostgreSQL's normal SnapshotAny
  recently-dead indexing behavior.
- For every callback-live row, the coordinator SHALL resolve the callback's
  index-entry TID under that same snapshot with the table AM index-fetch API
  before accepting any callback datum. The callback TID MAY be a HOT root while
  its datums describe the visible HOT member; exact physical-row fetch that
  does not follow HOT chains is forbidden. If the resolved vector or
  source-identity bytes differ from the callback datums, or no tuple is visible
  through that index TID under the snapshot, the build SHALL raise
  `EC_SOURCE_SNAPSHOT` before graph construction, participant begin, or remote
  mutation.
- The gate SHALL cause source DML and schema-changing DDL to fail closed if the
  coordinating session exits and releases its session lock before publish or
  abort.
- The durable build gate SHALL reject `INSERT`, `UPDATE`, `DELETE`, `MERGE`,
  `COPY FROM`, `TRUNCATE`, source-relation `ALTER`/`DROP`, `CLUSTER`,
  `VACUUM FULL`, and any other source tuple/schema rewrite. It SHALL continue to permit
  `SELECT` and non-rewriting inspection of the prior Published epoch.
- The same gate SHALL reject DROP, REINDEX, or ALTER of the coordinator control
  index, because removing or changing the UUID-bearing control while a remote
  build exists would destroy recovery identity. Gate lookup SHALL revalidate the
  live `(source_relation_oid, index_oid, logical_index_uuid)` triple before
  rejecting; OID coincidence alone is never sufficient.
- The build-to-Ready operation SHALL use one coordinator transaction and MAY
  use bounded PostgreSQL temporary files for FR-077 stitch streams.
- A successful `ec_distann_build_epoch` call SHALL return the 32-byte candidate
  manifest digest after all owners are Ready. It SHALL NOT return a Published
  fingerprint or make the generation query-visible.
- Before changing the coordinator registration to `Ready`, build-epoch SHALL
  atomically persist one immutable build-candidate row containing the canonical
  build specification and digest, generation descriptor and digest, source
  snapshot descriptor and digest, epoch manifest and digest/fingerprint, and
  the complete canonical Ready-receipt set. The manifest supplies the exact
  global record count and graph/row/head digests. The later decision transaction
  SHALL consume this durable candidate rather than client memory or a newly
  observed source snapshot. Exact candidate replay is idempotent; any byte or
  digest mismatch is `EC_BUILD_ID_CONFLICT` and changes no state.
- `ec_distann_build_candidate` SHALL have primary/foreign-key identity
  `(index_oid, logical_index_uuid, build_id)` and store epoch, registration
  digest, canonical build-spec bytes/digest, generation-descriptor bytes/digest,
  source-snapshot bytes/digest, roster-ordered Ready-receipt-set bytes/digest,
  epoch-manifest bytes/digest/fingerprint, candidate digest, and creation time.
  The receipt set uses the manifest's exact `u32 count` plus repeated
  `u32 length || receipt` encoding and SHALL byte-equal the receipts embedded in
  the manifest. Its digest is
  `SHA-256("ec_distann_ready_receipt_set_v1\0" || exact_receipt_set_bytes)`.
  Candidate-digest v1 encodes, in exact order: `version u16 = 1`, registration
  digest, `u32 build_spec_length || build_spec || build_spec_digest`,
  `u32 descriptor_length || descriptor || descriptor_digest`,
  `u32 snapshot_length || snapshot || snapshot_digest`,
  `u32 receipt_set_length || receipt_set || receipt_set_digest`,
  `u32 manifest_length || manifest || manifest_digest`, and the 34-byte
  fingerprint; it is
  `SHA-256("ec_distann_build_candidate_v1\0" || canonical_body)`. The row is
  immutable. Registration becomes Ready iff candidate insertion succeeds in
  the same transaction, and publish decision has an exact FK/identity link to
  it. T3 may re-run read-only topology but SHALL compare it with this candidate.
- The coordinator SHALL keep both session-level relation locks across the
  build-to-Ready, publish-decision, and publish-recovery transactions.
- If the owning backend exits, the session locks disappear but the durable gate
  remains. Explicit abort or publish recovery in another backend SHALL first
  acquire a new build-specific source `ShareLock`, then revalidate control UUID,
  registry/registration identity, and lifecycle state in the normative lock
  order. It does not and cannot release the dead backend's lock. Build-epoch may
  resume only with the original still-live frozen workspace; a replacement
  backend without that workspace may abort pre-decision or recover a durable
  post-decision build, but may not recapture the source under the old build id.
- `ec_distann_abort_epoch_build` SHALL idempotently abort every remote
  unpublished generation, remove the coordinator build gate, and release the
  session-level lock when held by the caller.
- Abort and activation recovery SHALL schedule both session-lock releases from a
  transaction callback only after the gate-clearing transaction commits. An
  error, subtransaction rollback, or top-level rollback preserves a previously
  committed build lock and gate; no endpoint releases it precommit.
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
version: 2
fields:
  - { name: descriptor_version, type: u16, rule: exactly 2 }
  - { name: coordinator_logical_index_uuid, type: byte[16], rule: authoritative coordinator control UUID captured by begin-build }
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
`SHA-256("ec_distann_generation_descriptor_v2\0" || canonical_descriptor)`.
The pre-publication descriptor-v1 draft is superseded and SHALL be rejected by
the physical handoff lane rather than reinterpreted; existing draft fixtures
are rebuild-only. Descriptor v2's coordinator UUID lets every participant bind
later authenticated activation/retire decisions even when the authoritative
coordinator is outside the owner roster.
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
- The coordinator logical control itself MAY be the local participant entry
  when its participant identity is configured and authenticated like every
  other owner. Its root remains metadata-only; its owned graph and frozen rows
  live only in the build-scoped generation relations.
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
- The row tier is captured storage, not a writable copy of the source table.
  Every non-dropped row-tier column SHALL therefore be physically nullable and
  SHALL carry no source column default, table/column CHECK, NOT-NULL, identity,
  or generated expression. Domain identity and validation remain part of the
  bound type semantics. Type, typmod, collation, attnum layout, dropped slots,
  and captured values remain exact. Source `attnotnull` outside the
  required identity attribute is deliberately not part of row-schema version
  1; differing source constraints cannot make stage fail after schema
  compatibility has passed.
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

The coordinator SHALL invoke begin/stage/seal/abort handoff operations in
`READ COMMITTED` transactions. Their replay contract depends on each statement
seeing a concurrently committed generation or batch journal. Every handoff and
FR-082 lifecycle endpoint SHALL inspect the current transaction isolation level
before lock, catalog, or RPC side effects and reject Repeatable Read or
Serializable with `EC_TRANSACTION_ISOLATION`; the caller SHALL retry in a new
READ COMMITTED transaction. Merely documenting stronger isolation as outside
the transport contract is insufficient.

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
- Every owner SHALL receive and acknowledge sequence zero even when its owner
  stream is empty. An empty owner uses one canonical zero-entry batch and a
  Ready receipt with `last_acknowledged_batch_sequence = 0`; there is no
  unsigned sentinel for "no batch" in receipt version 1.
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
- The version-1 journaled content identity is the verified SHA-256 batch digest
  plus exact encoded byte length. An acknowledged replay with a different
  digest or byte length SHALL raise `EC_BATCH_CONFLICT` without mutation; an
  exact identity replay returns the journaled receipt. Version 1 relies on
  SHA-256 collision resistance and SHALL NOT retain complete encoded batches,
  which would duplicate the frozen row tier.
- Each generation row SHALL persist three restart identities: nullable
  `last_vec_id_le` (exactly eight little-endian bytes when at least one record
  has been accepted), a non-NULL 107-byte owner-stream SHA-256 state, and a
  nullable exact 303-byte Ready receipt. `last_vec_id_le` SHALL be NULL exactly
  when cumulative record count is zero. The Ready receipt SHALL be NULL exactly
  in Building state and present in Ready, Published, or Retired state. A
  non-Building generation SHALL have acknowledged at least sequence zero.
- Owner-stream hash-state version 1 SHALL have this exact 107-byte layout:

  | Offset | Bytes | Field |
  |---:|---:|---|
  | 0 | 2 | `state_format_version u16_le = 1` |
  | 2 | 1 | `implementation_id u8 = 1` |
  | 3 | 32 | eight SHA-256 chaining words as `u32_le` |
  | 35 | 8 | compressed-block count as `u64_le` |
  | 43 | 1 | buffered-byte count in `0..=63` |
  | 44 | 63 | eager buffer; first buffered-count bytes are input and the unused suffix is zero |

  Implementation id 1 is the exact `Sha256` `SerializableState` representation
  from the direct dependency `sha2 = "=0.11.0"`; the project SHALL pin that
  version and SHALL reject any other implementation id. The specified offsets,
  canonical zero suffix, and golden fixtures are normative even if a transitive
  dependency changes. Compressed-block count times 64 plus buffered-byte count
  is the total hashed input length and SHALL not exceed `u64::MAX / 8`.
- A new owner hasher SHALL first consume the 27-byte domain
  `"ec_distann_owner_stream_v1\0"`. Each accepted entry then consumes exactly
  `u32_le(encoded_entry_length) || encoded_entry`. The state is snapshotted only
  after a complete batch transaction. Restoring state SHALL validate the full
  envelope and canonical buffer, finalize a clone, and require equality with
  the separately stored cumulative owner digest before any physical write.
  Exact replay SHALL return the journaled receipt without advancing state.
  Empty sequence zero leaves the initialized hash state and NULL last vec-id
  unchanged.
- If a sequence skips the participant's next expected value, then the
  participant SHALL raise `EC_BATCH_SEQUENCE` without mutation.
- The seal operation SHALL reject missing sequences, count disagreement,
  digest disagreement, duplicate vec_ids, non-owned vec_ids, and row/record
  count disagreement.
- The seal operation SHALL make the generation Ready but query-invisible.
- Every durable publish-decision insertion SHALL first take the same
  `ShareRowExclusiveLock` on the logical control relation that begin, stage,
  seal, and abort take. The relation-lock-then-generation-row order is the
  serialization boundary that makes abort's final decision guard race-free.
- The expected owner digest SHALL be
  `SHA-256("ec_distann_owner_stream_v1\0" ||
  repeated(u32_le(encoded_entry_length) || encoded_entry))` over the owner's
  canonical handoff entries in vec_id order.
- The expected owner digest SHALL exclude participant-local row-tier and graph
  `ItemPointer` values.
- Seal SHALL independently reconstruct the locator-free canonical handoff entry
  for every physical graph/row pair in unsigned vec_id order and require its
  owner-stream digest to equal the begin-time expectation. It SHALL also derive
  these Ready-receipt digests from physical storage:
  - `persisted_graph_digest = SHA-256("ec_distann_persisted_graph_v1\0" ||
    repeated(u64_le(vec_id) || u32_le(record_len) || physical_graph_record))`;
  - `persisted_row_tier_digest =
    SHA-256("ec_distann_persisted_row_tier_v1\0" ||
    repeated(u64_le(vec_id) || u32_le(null_bitmap_len) || null_bitmap ||
    u32_le(value_count) || repeated(u32_le(value_len) || typsend_value)))`;
  - `local_directory_digest =
    SHA-256("ec_distann_local_directory_v1\0" ||
    repeated(u64_le(vec_id) || graph_heap_ctid_6_bytes))`.
  Repetitions are in unsigned vec_id order. The physical graph digest includes
  its destination-local row locator; the owner-stream digest does not.
- Ready-receipt byte totals SHALL use `pg_table_size(graph_store_relid)`,
  `pg_table_size(row_tier_relid)`, and
  `pg_total_relation_size(directory_relid)` respectively. The two table-size
  calls include their attributed TOAST storage and exclude the directory index;
  the directory total includes its own relation storage.

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
| build_options | length-prefixed bytes | Legacy 26-byte options retain `build_list_size u16`, IEEE-754 `alpha f32_le`, `seed u64`, IEEE-754 `closure_epsilon f32_le`, `head_index_cap u32`, and `build_shards u32`. Task 182 version 2 prefixes `options_version u16 = 2` and appends `head_policy u8`, `training_query_count u32`, and `training_query_digest byte[32]`; zero build_shards means FR-077 auto selection; negative-zero closure epsilon is non-canonical and rejected |
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

Version-2 trained-head options require policy `training_landmarks_exact`, query
count 200, and a nonzero canonical digest. Legacy 26-byte options decode as
`current_sample_graph`, count zero, and a zero no-training digest; they re-encode
byte-identically. Policy/input mismatches and unknown versions fail closed.

The build-specification digest SHALL be
`SHA-256("ec_distann_build_spec_v1\0" || canonical_build_specification)`.
Fixed-width fields and length prefixes SHALL follow the same integer/UUID rules
as the generation descriptor. `owner_expectations` SHALL begin with its `u32`
element count and then encode the three fixed-width fields for each roster
entry without per-entry byte lengths. The build specification SHALL contain no raw
conninfo, secret reference, PostgreSQL OID, or local physical locator.
- `expected_global_graph_digest` SHALL be
  `SHA-256("ec_distann_global_graph_v1\0" || repeated(locator_free_graph))`,
  where `locator_free_graph` is `u64_le(vec_id)`, `graph_flags u16_le`, the
  length-prefixed search code, neighbor count `u32_le`, neighbor vec ids in
  stored order, and the length-prefixed concatenated neighbor-code bytes.
- `expected_global_row_tier_digest` SHALL be
  `SHA-256("ec_distann_global_row_tier_v1\0" || repeated(canonical_row))`,
  where `canonical_row` is `u64_le(vec_id)`, the length-prefixed 16-byte source
  identity, length-prefixed NULL bitmap, row-value count `u32_le`, and each
  length-prefixed `typsend` value. Both global repetitions use unsigned vec_id
  order and no participant-local locator.
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
- `graph_digest` and `row_tier_digest` SHALL be the independently recomputed
  persisted graph/row-tier digests defined for seal, and SHALL equal the Ready
  receipt. `graph_bytes`, `row_tier_bytes`, and `directory_bytes` SHALL use the
  exact Ready-receipt size functions defined above and exclude the logical
  control index.
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
| `EC_BUILD_BUSY` | Another live backend owns the source/control session locks for this logical build surface | Fail non-blockingly without registration or remote mutation |
| `EC_NODE_DESCRIPTOR` | Roster ordinal/id/endpoint is duplicate, malformed, secret-bearing, or incompatible with the remote control index | Reject before catalog or remote mutation |
| `EC_SOURCE_IDENTITY` | Physical build lacks one valid global UUID/bytea16 source identity per row | Reject before snapshot capture or handoff |
| `EC_BATCH_SEQUENCE` | Gap, regression, or out-of-order batch sequence | Reject before mutation |
| `EC_BATCH_CONFLICT` | Replayed sequence has different verified digest or encoded length | Reject before mutation |
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
| FR-078-AC-14 | Participant identity is durably configured; node registration resolves a secret reference, obtains UUID/endpoint/canonical locator only from the secured identity endpoint, rejects duplicate/raw/incompatible inputs before insertion, and desired-roster replacement cannot alter active/retained build bindings | Test (TC-040) |
| FR-078-AC-15 | The 107-byte owner-stream hash state is golden-frozen, resumes to the one-shot digest across every entry/batch split, rejects malformed or mismatched state before mutation, and preserves empty sequence-zero semantics | Test (TC-040, TC-050) |
| FR-078-AC-16 | Begin-build commits a source/control-identity gate plus complete golden-frozen private-binding digest under source→control→registry→registration locking; exact replay validates every bound byte, subcommit promotes ownership, top/subtransaction rollback releases new locks, and same-session/competing-backend builds cannot borrow ownership | Test (TC-042, TC-050) |

## Dependencies

- **Upstream**: [FR-076](./FR-076-distann-graph-node-record-format.md),
  [FR-077](./FR-077-distann-sharded-build-and-stitch.md), and
  [NFR-014](../../../non-functional/NFR-014-spire-transport-security-and-operations.md)
- **Downstream**: [FR-079](./FR-079-distann-remote-expansion-protocol.md),
  [FR-082](./FR-082-distann-epoch-lifecycle.md), and
  [NFR-020](../../../non-functional/NFR-020-distann-fault-behavior.md)
