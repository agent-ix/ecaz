---
id: FR-082
title: Distann Epoch Lifecycle and Consistency
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-078"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-079"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-082: Distann Epoch Lifecycle and Consistency

## Description

The ec_distann cluster SHALL transition every participant generation only
through the Building → Ready → Published → Retired lifecycle table below, with
Aborted as the terminal state for an unpublished generation. "Generation" and
"epoch" have the participant-shard and cluster-wide meanings defined by FR-078.

The coordinator SHALL make a new generation query-visible only after every
roster participant has durably published the matching physical shard and epoch
row tier.

Every distributed read SHALL carry one coordinator-registered epoch
fingerprint so one scan attempt never mixes generations.

For each logical-index UUID, the **authoritative coordinator** SHALL be the one
PostgreSQL coordinator instance that owns that logical index's durable active
pointer, publish/retire decisions, and coordinator-local in-flight registry.
Every scan capable of addressing a generation for that logical index SHALL
originate on that instance and register there before invoking a participant
endpoint. A second coordinator instance SHALL NOT invoke expansion,
materialization, publication, or reclaim endpoints for that logical index.

## Inputs

- The stitched global-graph identity and head sample from
  [FR-077](../build/FR-077-distann-sharded-build-and-stitch.md) and
  [FR-080](../read/FR-080-distann-coordinator-head-index.md).
- The ordered roster, placement version, participant Ready receipts, row-schema
  fingerprint, format/codec identity, source-snapshot identity, and canonical
  content/build-specification digests from
  [FR-078](../build/FR-078-distann-hash-placement.md).
- The currently active epoch manifest, if any.

## Outputs

- A durable versioned epoch manifest and SHA-256 manifest digest.
- Participant generations addressable by epoch fingerprint while retained.
- One coordinator active-epoch pointer whose swap is the query-visible publish
  linearization point.
- Durable publish-decision and recovery state sufficient to finish or abort an
  interrupted build without guessing.

## Lifecycle States

| Current state | Event | Next state | Query visibility |
|---------------|-------|------------|------------------|
| absent | accepted `begin_epoch_handoff` | Building | hidden |
| Building | all owner streams seal with matching receipts | Ready | hidden |
| Building | abort before publish decision | Aborted | hidden |
| Ready | abort before publish decision | Aborted | hidden |
| Ready | participant applies the durable cluster publish decision | Published | addressable by explicit fingerprint |
| Published and active | coordinator activates a successor | Published, non-active, retirement mark pending | retained scans may continue by fingerprint |
| Published and non-active | authenticated successor-activation marker exactly matches the local predecessor | Retired | retained scans may continue by fingerprint |
| Published and non-active | coordinator binding is explicitly Abandoned after successor activation while the participant is unreachable | Published orphan locally; coordinator binding Abandoned | unavailable through authoritative coordinator routing; out-of-band operator cleanup only |
| Retired | coordinator fence observes zero in-flight scans and durable retire decision is applied | Reclaimed tombstone | unavailable |

Any transition absent from this table SHALL fail with `EC_EPOCH_STATE` and
leave the generation unchanged.

## Behavior

### Build Snapshot and Manifest

The canonical source-snapshot descriptor SHALL contain, in order:
`snapshot_version u16 = 1`, PostgreSQL cluster system identifier `u64`,
length-prefixed UTF-8 database name, snapshot `xmin_full u64`, `xmax_full u64`,
`curcid u32`, `xip_count u32` followed by the ascending `u64` in-progress full
XIDs, `subxip_count u32` followed by the ascending `u64` subtransaction full XIDs,
`suboverflowed u8`, and `taken_during_recovery u8`. Fixed-width values use
little-endian encoding. The source-snapshot digest SHALL be
`SHA-256("ec_distann_source_snapshot_v1\0" || canonical_snapshot_descriptor)`.
The coordinator SHALL expand every 32-bit `TransactionId` in PostgreSQL's
`SnapshotData` to its wrap-aware `FullTransactionId` relative to the captured
next full XID before sorting or serialization; bare 32-bit xmin/xmax/xip values
SHALL NOT enter the descriptor.
`xmin_full` SHALL be less than or equal to `xmax_full`; every `xip` and
`subxip` value SHALL lie in `[xmin_full, xmax_full)`. Values in each array
remain strictly ascending.

The version-2 canonical epoch manifest SHALL contain these fields in this order:

| Field | Wire type | Rule |
|-------|-----------|------|
| manifest_version | u16 | exactly 2 |
| epoch | u64 | non-zero; when a parent exists, greater than the epoch obtained by resolving `parent_fingerprint` to its retained manifest |
| build_id | byte[16] | RFC 4122 version-4 UUID bytes from FR-078 |
| parent_fingerprint | length-prefixed bytes | empty for the first epoch; otherwise one valid retained fingerprint |
| source_snapshot_digest | byte[32] | SHA-256 of the canonical PostgreSQL snapshot identity recorded by the build |
| build_spec_digest | byte[32] | immutable pre-handoff specification from FR-078 |
| generation_descriptor_digest | byte[32] | FR-078 descriptor identity binding roster, physical formats, trained codec artifact, and row schema |
| placement_hash_version | u16 | version accepted by every participant |
| roster | length-prefixed array | ordered `(node_id u32, logical_index_uuid byte[16], length-prefixed endpoint_identity)` entries from the FR-078 generation descriptor; no secret material |
| index_format_version | u16 | distributed-control/generation storage format version |
| graph_record_version | u16 | FR-076 persisted graph-record format |
| handoff_wire_version | u16 | FR-076 handoff version |
| codec_parameters | length-prefixed bytes | canonical version-1 codec kind and shape subrecord defined below |
| build_options | length-prefixed bytes | canonical version-1 legacy or version-2 trained-head graph/build options subrecord defined below |
| row_schema_fingerprint | byte[32] | FR-078 schema identity |
| head_sample_digest | byte[32] | canonical coordinator head-sample identity |
| global_record_count | u64 | exact source vec_id cardinality |
| global_graph_digest | byte[32] | canonical stitched graph content |
| global_row_tier_digest | byte[32] | canonical frozen row payload content |
| participant_receipts | length-prefixed array | exactly one receipt per roster entry, in roster order |

The `codec_parameters` subrecord SHALL contain, in order:
`parameters_version u16 = 1`, `codec_kind u8`, `dimensions u16`,
`code_stride u32`, `seed u64`, `transform_dim u32`, `group_count u32`,
`group_size u32`, and `centroids_per_group u16`. RaBitQ and TurboQuant SHALL
encode zero for the four GroupedPQ-only shape fields. GroupedPQ4 SHALL encode
`group_count × group_size = transform_dim`,
`code_stride = ceil(group_count / 2)`, and `centroids_per_group = 16`.
The subrecord is exactly 31 bytes.

The `build_options` subrecord SHALL contain, in order:
`options_version u16 = 1`, `graph_degree u16`, `build_list_size u16`,
IEEE-754 `alpha f32_le`, `seed u64`, IEEE-754 `closure_epsilon f32_le`,
`head_index_cap u32`, and `build_shards u32`. `build_shards = 0` retains the
FR-077 auto-shard meaning. The subrecord is exactly 30 bytes.
After its version and graph-degree prefix, its remaining 26 bytes are exactly
the FR-078 canonical build-options bytes used inside the build specification;
the manifest does not define a second interpretation.

For a trained generation the subrecord SHALL instead contain
`options_version u16 = 2`, `graph_degree u16`, a length-prefixed FR-078
version-2 build-options record, including policy, training count, and training
digest. Legacy version-1 subrecords remain exactly 30 bytes and preserve
current-sample/Vamana semantics. The manifest fingerprint therefore binds both
head selection and query-time scoring semantics without reinterpreting an old
epoch.

Every array in the manifest SHALL begin with an unsigned little-endian `u32`
element count. Each participant receipt SHALL then be encoded as one
`u32` byte length followed by its complete canonical receipt bytes.

Each participant Ready receipt SHALL contain, in order: `receipt_version u16 =
1`, `node_id u32`, `epoch u64`, `build_id byte[16]`, build-specification digest,
generation-descriptor digest, `last_acknowledged_batch_sequence u64`, owned
record count `u64`, row count `u64`, owner-stream digest, persisted graph
digest, persisted row-tier digest, local-directory digest, graph bytes `u64`,
row-tier bytes `u64`, directory bytes `u64`, and `state u8 = 1` (`Ready`). Receipt
fixed-width integers use little-endian encoding; build UUID is version 4 and uses RFC 4122 byte order;
digests are 32 bytes.

Receipt v1 deliberately carries both `owned_record_count` and `row_count` for
independent graph-coverage and row-tier topology accounting, but requires them
to be equal because v1 has exactly one frozen row per owned graph record. Any
future representation that permits divergence requires a new receipt version.

The receipt's canonical body is 271 bytes and it SHALL append
`SHA-256("ec_distann_ready_receipt_v1\0" || canonical_receipt_without_digest)`.
The complete receipt is therefore exactly 303 bytes.

- The coordinator SHALL capture one PostgreSQL MVCC snapshot for the complete
  build and handoff.
- While the distributed build is active, the coordinator SHALL hold a relation
  lock that permits reads and blocks source-row DML and schema changes until
  publish or abort finishes.
- The Building generation SHALL contain the full stitched record set, owner
  placement metadata, head sample, and frozen epoch row tier.
- A query SHALL NOT resolve a Building or Ready generation.
- The manifest SHALL identify the epoch, UUID build id, parent fingerprint, source
  snapshot, immutable build-specification and generation-descriptor digests, ordered roster,
  placement-hash version, graph/row wire versions,
  index format version, codec parameters, build options, row-schema
  fingerprint, head-sample digest, global record count, global graph digest,
  global row-tier digest, and every participant receipt.
- The manifest SHALL contain endpoint identities and SHALL contain neither raw
  conninfo nor conninfo secret references.
- The canonical manifest encoding SHALL serialize fields in the order listed by
  this requirement, fixed-width integers in little-endian order, strings and
  byte arrays with little-endian `u32` lengths, and participant receipts in
  roster order.
- The manifest digest SHALL be
  `SHA-256("ec_distann_epoch_manifest_v2\0" || canonical_manifest_bytes)`.
- The epoch fingerprint SHALL be the 34-byte value
  `u16_le(2) || manifest_digest`.
- A participant SHALL reject an unknown fingerprint version as
  `EC_EPOCH_FINGERPRINT_VERSION`.
- The epoch fingerprint SHALL NOT bind mutable tombstone, delta-buffer, or
  incremental-insert/replacement state permitted by
  [FR-083](./FR-083-distann-dml-path.md).

### Cluster Publication

Each participant SHALL expose these internal lifecycle operations:

`ec_distann_publish_epoch(index_regclass regclass, build_id uuid,
epoch_manifest bytea, manifest_digest bytea) RETURNS bytea`

`ec_distann_epoch_generation_status(index_regclass regclass, build_id uuid)
RETURNS TABLE (epoch bigint, state text, build_spec_digest bytea,
generation_descriptor_digest bytea, epoch_fingerprint bytea,
manifest_digest bytea, record_count bigint, row_count bigint,
successor_activation_digest bytea, retire_decision_digest bytea)`

`ec_distann_mark_epoch_retired(index_regclass regclass,
successor_activation bytea, successor_activation_digest bytea) RETURNS void`

`ec_distann_apply_epoch_retire(index_regclass regclass,
retire_decision bytea, retire_decision_digest bytea)
RETURNS void`

The coordinator SHALL expose these decision/recovery operations:

`ec_distann_decide_epoch_publish(index_regclass regclass, build_id uuid)
RETURNS bytea`

`ec_distann_recover_epoch_publish(index_regclass regclass) RETURNS bytea`

`ec_distann_retire_epoch(index_regclass regclass, epoch_fingerprint bytea)
RETURNS void`

`ec_distann_recover_epoch_retire(index_regclass regclass,
epoch_fingerprint bytea) RETURNS void`

`ec_distann_force_retire_epoch(index_regclass regclass,
epoch_fingerprint bytea, reason text) RETURNS void`

`ec_distann_abandon_predecessor_binding(index_regclass regclass,
successor_build_id uuid, predecessor_roster_ordinal integer, reason text)
RETURNS void`

Every lifecycle operation above, together with FR-078 begin/build/abort/status,
SHALL be `SECURITY DEFINER` with a fixed trusted search path and no `PUBLIC`
EXECUTE privilege. Before catalog, secret, lock, or RPC side effects it SHALL
authorize the extension owner or explicitly granted internal operator/cluster
role. Temporary-schema name resolution or an ordinary reader SHALL confer no
lifecycle side effect.

PostgreSQL releases every default-lock-manager session relation lock on a
top-level transaction abort, including a source/control lock acquired and
committed by an earlier transaction in the same backend. The coordinator SHALL
therefore clear its backend-local ownership mirror on top-level abort while
leaving the durable build registration/gate unchanged. Before that backend or
another backend resumes build, decision, or pre-activation recovery work it
SHALL reacquire source `ShareLock` then control `ShareRowExclusiveLock` and
revalidate the exact registration. Durable DML/utility gate enforcement SHALL
remain active during this recovery window; code SHALL NOT infer that the
durable gate vanished because PostgreSQL released the session locks. A
subtransaction abort releases only session locks newly acquired by that
subtransaction and does not discard an ownership record committed by its
parent.

- The coordinator SHALL verify that every roster participant supplied one Ready
  receipt for the same epoch, build id, build-specification digest, and
  generation-descriptor digest, then verify that the descriptor transitively
  bound by that digest contains the manifest's row-schema fingerprint. The
  Ready receipt has no separate schema-digest field.
- The coordinator SHALL verify global count, owner-count sum, graph digest,
  row-tier digest, exact vec_id coverage, empty owner intersections, and
  record/row co-placement before deciding publication.
- If any required receipt or topology invariant is absent, then the coordinator
  SHALL keep the generation query-invisible.
- After validation, the coordinator SHALL persist one durable commit-only
  publish decision containing the build id, epoch fingerprint, manifest digest,
  complete receipt set, and the FR-078 private build-participant binding
  identity. Fingerprint-to-build lookup SHALL be unique and retained until
  final reclaim.
- T3 and every T4 recovery transaction SHALL recompute and verify the canonical
  candidate digest chain over the stored registration, build-specification,
  generation-descriptor, source-snapshot, Ready-receipt-set, and epoch-manifest
  bytes before consuming them. A stored-byte or digest mismatch SHALL raise
  `EC_PUBLISH_DIGEST` before participant, decision, or active-pointer mutation.
- The decision SHALL also persist an all-or-none predecessor tuple `(build id,
  epoch, fingerprint, manifest digest)`, the canonical successor-activation
  bytes/digest defined below, `activated_at`, and application progress. T3 SHALL
  require the candidate parent fingerprint to equal the active pointer selected
  under lock (empty iff no active pointer), and the pointer swap SHALL compare
  and swap against that predecessor. Overwriting an unrecorded predecessor is
  forbidden.
- `ec_distann_decide_epoch_publish` SHALL validate all receipts and topology,
  persist that decision, return its 32-byte manifest digest, and return without
  calling a participant publish endpoint or swapping the active pointer.
- The transaction that calls `ec_distann_decide_epoch_publish` SHALL commit
  before any caller invokes `ec_distann_recover_epoch_publish` for that
  decision.
- After the publish decision is durable, the coordinator SHALL drive every
  participant from Ready to Published with an idempotent
  `ec_distann_publish_epoch` operation.
- The participant SHALL validate the canonical manifest digest before changing
  state.
- The participant SHALL cross-check the manifest codec-parameter plaintext
  against the generation descriptor's complete codec artifact: kind,
  dimensions, seed, code stride, and any GroupedPQ transform/group shape SHALL
  agree before changing state. Digest binding alone is insufficient.
- The participant SHALL verify that the manifest contains its exact Ready
  receipt and build-specification digest before changing state.
- A participant SHALL acknowledge publication only after its Published state,
  manifest, graph shard, row tier, directory, and receipt survive PostgreSQL
  restart.
- The coordinator SHALL wait for a matching Published acknowledgement from
  every participant.
- The coordinator SHALL atomically swap its active-epoch pointer only after all
  matching acknowledgements are durable.
- The active pointer SHALL persist the matching build id in addition to epoch,
  fingerprint, and manifest digest. Active and retained read/recovery transport
  SHALL resolve the immutable private participant bindings by that build id and
  SHALL never consult the mutable desired-node registry.
- The active-pointer swap SHALL be the cluster's query-visible publication
  linearization point.
- The pointer-swap transaction SHALL retain the predecessor fingerprint and
  immutable predecessor build binding until successor recovery has applied
  `ec_distann_mark_epoch_retired` to every participant in the predecessor
  roster. This includes participants absent from the successor roster.
- The canonical successor-activation record SHALL contain, in order:
  `activation_version u16 = 1`, coordinator logical-index UUID `byte[16]`,
  `predecessor_present u8`, then—only when present—predecessor build id
  `byte[16]`, predecessor epoch `u64`, length-prefixed predecessor fingerprint,
  and predecessor manifest digest `byte[32]`; followed by successor build id
  `byte[16]`, successor epoch `u64`,
  length-prefixed successor fingerprint, and successor manifest digest
  `byte[32]`. Its digest SHALL be
  `SHA-256("ec_distann_successor_activation_v1\0" || canonical_bytes)`.
  `predecessor_present = 0` is valid only for an empty candidate parent and no
  active pointer, omits every predecessor field, and requires no retirement
  RPC; otherwise it is exactly 1.
- `ec_distann_mark_epoch_retired` SHALL receive those complete canonical bytes
  and digest over authenticated coordinator transport, validate its exact local
  Published predecessor build/fingerprint/manifest identity, persist the bytes
  and digest, bind the successor identity,
  transition `Published → Retired` idempotently, and change no physical row,
  graph, or directory bytes. A conflicting successor identity SHALL raise
  `EC_EPOCH_STATE` with zero mutation.
- Successor retirement marking SHALL occur only after the coordinator active
  pointer swap commits. A crash after the swap leaves the successor active and
  publish recovery SHALL finish every missing old-roster retirement mark before
  declaring recovery complete. Therefore an old participant removed from the
  successor roster is still retired through the predecessor's private binding.
- Commitment ordering is enforced by the authoritative coordinator and its
  authenticated transport: the marker bytes are not independently a proof that
  a remote transaction committed. A participant validates the marker digest,
  its exact predecessor generation identity, and the coordinator UUID captured
  for that build before mutation.
- A successor decision normally transitions `Pending → Activated → Applied`.
  An operator may instead make the terminal transition `Pending → Cancelled`
  through `ec_distann_cancel_epoch_publish` only while the exact recorded
  predecessor tuple is still the active pointer (or both are absent for a
  first epoch). `Pending`
  means the durable decision exists while the predecessor pointer remains
  active. `Activated` means the successor pointer swap committed while one or
  more predecessor retirement marks remain. A successor decision reaches
  `Applied` only after the active-pointer swap has committed and every
  predecessor private binding has one immutable terminal disposition: either
  `Retired`, carrying the exact acknowledged activation digest, or `Abandoned`,
  carrying the exact operator audit described below. Before that, it remains
  recoverable with `activated_at` distinguishing pre-swap from post-swap
  progress. An unavailable predecessor reports
  `EC_PREDECESSOR_RETIRE_PENDING`; the successor remains active and the pointer
  never rolls back.
- Cancellation SHALL require a nonempty UTF-8 reason of at most 1,024 bytes,
  record `session_user`, reason, and timestamp atomically with the decision CAS,
  and move the matching build registration `Decided → Cancelled`. It clears the
  build gate but never deletes the decision, candidate, fingerprint, or private
  participant bindings. Exact replay with the same reason succeeds from stored
  audit fields; another reason conflicts. `Activated` and `Applied` decisions
  cannot be cancelled, and publish recovery SHALL issue no participant call for
  a `Cancelled` decision.
- Canonical cancel-publish audit version 1 SHALL contain, in order: `version
  u16 = 1`, coordinator logical-index UUID `byte[16]`, cancelled build id
  `byte[16]`, epoch `u64`, length-prefixed 34-byte epoch fingerprint, manifest
  digest `byte[32]`, decision timestamp as signed Unix microseconds `i64`,
  length-prefixed caller name, and length-prefixed reason. Its digest SHALL be
  `SHA-256("ec_distann_cancel_epoch_publish_v1\0" || canonical_bytes)`. TC-050
  SHALL freeze its bytes, independent decode, endian handling, and
  unknown-version rejection.
- A cancelled decision remains the authoritative registration for its
  never-active fingerprint. A participant that acknowledged publication before
  cancellation may retain a Published-but-never-active orphan; it is never
  routable and may be reclaimed only through an explicit audited force-retire
  path tied to that cancelled decision. `ec_distann_recover_cancelled_publish`
  SHALL reject recovery when the cancelled decision's `xmin` belongs to the
  current transaction; cancellation must commit before cleanup recovery begins.
  Recovery SHALL replay the immutable private participant bindings and send
  the exact canonical audit/digest to each participant. Each participant SHALL
  accept only a matching non-active Ready or Published generation, atomically insert
  an immutable `CancelledReclaimed` tombstone before relation deletion, and
  replay exactly from that tombstone. A crash after a subset of remote commits
  is completed by re-drive; only after every binding acknowledges may the
  coordinator record `cancellation_reclaimed_at`. Ordinary abort and unaudited
  cleanup continue to refuse every generation named by a durable publish
  decision.
- T4a SHALL create one pending predecessor-disposition row for every ordinal in
  the immutable predecessor private-binding roster. T4b changes a row from
  `Pending` to `Retired` only after exact remote acknowledgement. Exact replay
  is idempotent; a different activation or participant identity is
  `EC_EPOCH_STATE` with zero mutation. The decision row SHALL NOT become
  `Applied` based on a count detached from these binding identities.
- `ec_distann_abandon_predecessor_binding` is the only availability override
  for a predecessor that cannot ever acknowledge retirement. It SHALL require
  an `Activated` successor decision, name one still-`Pending` predecessor
  ordinal, require a nonempty UTF-8 reason of at most 1,024 bytes, and authorize
  the extension owner or explicitly granted operator role. Under a row lock,
  one transaction SHALL construct and insert the immutable audit and compare-
  and-swap the disposition `Pending → Abandoned`; neither may commit without
  the other. A crash before commit leaves Pending with no audit, and a crash
  after commit leaves Abandoned with the complete audit. It SHALL never run
  automatically and SHALL NOT contact or reclaim the missing participant.
- Canonical abandon-binding audit version 1 SHALL contain, in order: `version
  u16 = 1`, coordinator logical-index UUID `byte[16]`, successor build id
  `byte[16]`, successor epoch `u64`, length-prefixed successor fingerprint,
  predecessor build id `byte[16]`, predecessor epoch `u64`, length-prefixed
  predecessor fingerprint, predecessor manifest digest `byte[32]`, predecessor
  roster ordinal `u32`, node id `u32`, participant logical-index UUID
  `byte[16]`, length-prefixed endpoint identity and canonical remote locator,
  successor-activation digest `byte[32]`, decision timestamp as signed Unix
  microseconds `i64`, length-prefixed caller name, and length-prefixed reason.
  Its digest SHALL be
  `SHA-256("ec_distann_abandon_predecessor_binding_v1\0" || canonical_bytes)`.
  TC-050 SHALL freeze the bytes, digest, independent decode, endian handling,
  and unknown-version rejection.
- Exact abandon replay for the same decision, ordinal, authenticated caller,
  and reason SHALL return success from the stored audit bytes and timestamp;
  it SHALL not regenerate time-dependent bytes. A different caller, reason,
  binding, activation, or participant identity is `EC_PREDECESSOR_ABANDON` with
  zero mutation. Concurrent exact retirement acknowledgement and abandonment
  serialize on the same disposition row: exactly one terminal state commits,
  and the loser replays only if it exactly matches that committed state.
- An abandoned binding remains durably forfeited for the lifetime of the
  coordinator control identity. If the participant later returns, authoritative
  coordinator routing SHALL never select the forfeited predecessor binding or
  count a late acknowledgement as `Retired`; a direct data request carrying the
  successor fingerprint fails that participant's ordinary fingerprint and
  coordinator-activation validation. The participant is not claimed to know a
  coordinator-local abandonment it never received. Only dependency cleanup for
  DROP or destructive REINDEX of the exact UUID-bearing control may remove the
  audit. Abandonment permits publication progress but does not assert remote
  reclamation; operator cleanup of the abandoned node is out of band and
  auditable.
- Publish-decision `Applied` denotes complete authoritative-coordinator
  disposition of the immutable predecessor binding set. It is not evidence
  that an unreachable abandoned participant transitioned its local generation
  to Retired or Reclaimed; that generation may remain a deliberately unroutable
  Published orphan until out-of-band cleanup.
- Before that swap, new scans SHALL continue to register the prior active epoch
  in the coordinator-local in-flight registry. After that swap, new scans SHALL
  register the new epoch.
- A participant SHALL retain Published and Retired generation storage until it
  receives the coordinator's durable retire decision through
  `ec_distann_apply_epoch_retire`; it SHALL NOT reclaim autonomously.
- A roster change SHALL require a new build id, new epoch, and new manifest.
- A roster change SHALL NOT mutate placement inside an existing Published or
  Retired epoch.
- Applying a retire decision SHALL atomically create an immutable participant
  reclaim tombstone before deleting the generation catalog row and physical
  relations. The tombstone SHALL retain logical-index UUID, build id, epoch,
  epoch fingerprint, manifest digest, build-specification and generation-
  descriptor digests, exact record/row counts, canonical retire-decision bytes
  and digest, the prior successor-activation digest when present, and reclaim
  timestamp. Exact apply replay SHALL compare both
  canonical bytes and digest and succeed from the tombstone; an unknown or
  conflicting identity SHALL fail closed. Generation status SHALL report
  `Reclaimed` and the retained retire-decision digest after physical deletion,
  so recovery never infers success from missing relations.
- Reclaim tombstones are immutable and SHALL survive routine generation,
  build-binding, publish-decision, and retire-decision cleanup for the lifetime
  of the participant control identity. Only dependency cleanup for DROP or
  destructive REINDEX of that exact UUID-bearing control may remove them; they
  SHALL NOT cascade from generation-row deletion.
- Canonical retire-decision version 1 SHALL contain, in order: `version u16 =
  1`, coordinator logical-index UUID `byte[16]`, target build id `byte[16]`,
  epoch `u64`, length-prefixed target fingerprint, target manifest digest
  `byte[32]`, length-prefixed target canonical `ec_distann_roster_v1` snapshot,
  roster digest `byte[32]`, abandoned-binding count `u32`, then each abandoned
  binding in ascending roster ordinal as ordinal `u32` and abandon-audit digest
  `byte[32]`; followed by forced flag `u8`, overridden in-flight count `u64`,
  decision timestamp as signed Unix microseconds `i64`, length-prefixed caller
  name, and length-prefixed nonempty reason (at most 1,024 UTF-8 bytes). Its digest SHALL be
  `SHA-256("ec_distann_retire_decision_v1\0" || canonical_bytes)`. The
  abandoned-binding segment has its own frozen identity:
  `SHA-256("ec_distann_abandoned_binding_set_v1\0" || u32_le(count) ||
  ordinal_and_audit_digest_entries)`. The separately stored set bytes SHALL be
  byte-identical to the retire-decision segment, and the coordinator SHALL
  recompute both digests whenever the decision is created or consumed.
  Participant validation covers canonical decoding, both digests, exact local
  target identity, immutable-roster membership, and proof that its own ordinal
  is not in the abandoned set before inserting the tombstone and reclaiming.
  A participant is not required or permitted to query coordinator disposition
  rows to validate other ordinals. Under the retirement fence, the coordinator
  locks the Applied covering decision and its exact dispositions and proves the
  set equals all and only terminal Abandoned rows before decision commit.
  Conflict is `EC_EPOCH_STATE` with zero mutation.

### Coordinator Scan Retention and Retire Fence

- Each logical-index UUID SHALL have exactly one coordinator-local retirement
  fence, keyed only by that logical-index UUID. There is no separate
  per-fingerprint fence. Both active-pointer selection/registration and every
  retirement attempt SHALL use this same fence, so pointer selection,
  registration, zero-count observation, and retire-decision creation have one
  linear order.
- Before issuing the first expansion for one attempt, the coordinator SHALL
  generate a never-reused UUID scan token and atomically read/register the
  selected fingerprint in a coordinator-local in-flight registry under that
  logical index's retirement fence.
- Registration SHALL re-check under the fence that the selected fingerprint has
  no committed retire decision. If one exists, registration SHALL reject with
  `EC_EPOCH_STATE`, add no registry entry, and issue no participant request.
- The registry SHALL be visible to every backend on the authoritative
  coordinator, but it SHALL require no participant RPC, participant catalog
  write, WAL flush, or synchronous commit on the query path.
- The version-1 registry and its one-fence-per-logical-index state SHALL live in
  PostgreSQL add-in shared memory initialized at postmaster startup. Distributed
  control serving SHALL require `ecaz` in `shared_preload_libraries`; if the
  shared registry is unavailable, scan registration and retirement SHALL fail
  closed before participant access. The registry SHALL store exact
  `(database_oid, logical_index_uuid, fingerprint, scan_token)` entries rather than a
  coalesced counter, allowing exact idempotency, backend-exit cleanup, and the
  force-retire override count. Advisory-lock hashes alone are not an acceptable
  token registry.
- `ec_distann.max_scan_pins` and `ec_distann.max_retire_fences` SHALL be
  postmaster-start nonnegative integer GUCs with finite positive defaults (version-1
  defaults: 65,536 exact token entries and 4,096 logical-index fence records).
  Zero is an explicit disable/test value that makes the corresponding first
  allocation fail as `EC_EPOCH_PIN_CAPACITY`.
  Exhaustion raises `EC_EPOCH_PIN_CAPACITY` before participant access and does
  not evict or coalesce a live token. Dead-token reaping and reclaim of a
  provably dropped fence entry are permitted before the capacity decision; no
  live token or live logical-index fence may be evicted. Each token records
  fingerprint, UUID token, PostgreSQL `ProcNumber`, backend PID, and an
  extension-maintained per-`ProcNumber` backend generation. On a backend's
  first registry use it increments that slot's generation under the registry
  LWLock and caches the result. A token is live exactly when the current
  `PGPROC` at that `ProcNumber` has the stored nonzero PID and the shared slot
  still has the stored generation; PID mismatch, empty `PGPROC`, or generation
  mismatch proves it dead. Normal guard
  release and `before_shmem_exit` remove that backend's entries. Registration
  and retirement also reap entries whose `(ProcNumber, backend generation)` is
  provably no longer live while holding the per-index fence; callback execution
  is not assumed after abrupt backend death. A postmaster
  restart begins empty because no pre-restart scan survives.
- Shared memory SHALL map each exact `(database_oid, logical-index UUID)` to one
  collision-free allocated heavyweight-lock fence id. An LWLock protects only fence-map and
  token-row mutations and SHALL never be held across SPI, ERROR, transaction
  commit, or RPC. Registration takes that fence id's heavyweight session shared
  lock, reads the active pointer/retire decision and inserts its exact token,
  then releases the shared lock in `PG_TRY`/finally. Retirement takes the same
  fence id's heavyweight transaction exclusive lock, observes the exact token
  count, and holds exclusion automatically through decision commit or abort.
  A waiting registration then re-reads active/decision state. Fence ids are not
  hashes; UUID aliasing is forbidden, and the locktag carries the same database
  OID. Each map entry has an operation reference count acquired under the
  registry LWLock before any backend waits for or holds its heavyweight fence,
  and released under that LWLock only after the heavyweight lock is released.
  DROP or destructive REINDEX marks the exact `(database_oid, logical-index
  UUID)` entry dropped and rejects new references. Its fence id may be recycled
  only after durable dependency cleanup proves that UUID absent, its exact
  token set and operation reference count are both zero, and the entry is
  removed under the LWLock. Thus no holder or waiter can retain a recycled
  locktag. Ordinary churn through dropped UUIDs SHALL NOT consume monotonic
  postmaster-lifetime capacity. Startup reconstructs mappings lazily from exact
  UUIDs and durable catalog state.
- Replaying the same local `(fingerprint, scan_token)` registration SHALL be
  idempotent. Reusing one scan token for another fingerprint SHALL raise
  `EC_EPOCH_PIN_CONFLICT` without changing either local count.
- A guard SHALL release the local registration on normal completion, error,
  cancellation, rescan, `EndCustomScan`, and epoch-mismatch restart. Backend or
  coordinator process death releases its registrations because no scan in that
  process can remain live.
- Normal and forced retirement of a predecessor fingerprint covered by a
  successor publish decision SHALL require that decision to be `Applied`.
  `Activated` is not sufficient: recovery must first retire or explicitly
  abandon every predecessor binding. Rejection creates no retire decision and
  changes no registry or generation state.
- Normal retirement SHALL accept only a non-active Retired fingerprint. It
  SHALL acquire that logical index's sole retirement fence exclusively, re-read
  the active pointer, prevent new local registrations, and reject with
  `EC_RETENTION_ACTIVE` unless the target fingerprint's local in-flight count is
  zero.
- After observing zero under the fence, the coordinator SHALL commit one
  immutable retire decision containing the fingerprint, manifest digest,
  roster, exact abandoned-binding ordinal/audit-digest set, caller, and
  timestamp before instructing the first non-abandoned participant to reclaim.
  It SHALL hold the exclusive fence through that commit, release it
  before participant RPCs, and rely on the committed-decision registration
  check to prevent later use of the target. Participants SHALL validate that
  decision and apply it idempotently. Recovery SHALL issue no reclaim RPC to an
  abandoned binding and SHALL reach retire-decision `Applied` only after every
  non-abandoned binding reports exact Reclaimed state; the abandoned disposition
  and audit remain the truthful terminal record for that ordinal. A crash after
  a subset applies is completed by `ec_distann_recover_epoch_retire` from the
  durable decision.
- A retirement rejected as `EC_RETENTION_ACTIVE` SHALL create no retire decision
  and release the exclusive fence before returning the error. A scan arriving
  while the fence is held SHALL wait for that short local critical section,
  then re-read the active pointer and retire-decision state; fence contention
  alone SHALL NOT produce a query error.
- Participant restart is safe without a durable per-scan token: a participant
  never reclaims because of restart, age, or local state alone. It reclaims
  only after the authoritative coordinator has fenced new scans, observed zero
  live scans, and committed the retire decision.
- `ec_distann_force_retire_epoch` SHALL still reject the active fingerprint.
  For a non-active Retired fingerprint it MAY override a non-zero local count
  only by committing an audit record containing the epoch, build id, manifest
  digest, overridden count, caller, reason, and timestamp before participant
  reclaim. It SHALL NOT run automatically.
- Normal retirement encodes the canonical reason exactly as UTF-8 `normal`;
  force retirement encodes the caller-supplied nonempty bounded reason.

### Publication Recovery

- Before a durable publish decision exists, recovery SHALL leave the old epoch
  active.
- Before a durable publish decision exists, recovery MAY resume the Building or
  Ready generation from its receipts.
- Before a durable publish decision exists, an operator MAY abort the Building
  or Ready generation.
- After a durable publish decision exists, recovery SHALL complete participant
  publication and coordinator activation idempotently.
- After a durable publish decision exists, recovery SHALL NOT reinterpret the
  generation as aborted.
- If a successor participant is unavailable after the decision but before
  activation, then the coordinator
  SHALL keep the old active pointer until all participants acknowledge the new
  Published generation. An authorized operator may terminate that wait only by
  cancelling the still-`Pending` decision; timeout or transport failure never
  cancels it automatically.
- If a predecessor-only participant is unavailable after activation, the new
  active pointer remains authoritative, decision state remains `Activated`, and
  recovery raises `EC_PREDECESSOR_RETIRE_PENDING`. Only predecessor retirement
  and later reclaim are delayed; rollback is forbidden. An authorized operator
  may terminate that wait only through the audited binding-specific abandonment
  operation; recovery never infers abandonment from timeout or transport error.
- If the coordinator crashes after all participant acknowledgements but before
  the active-pointer swap, then restart recovery SHALL perform the missing swap
  exactly once.
- If the coordinator crashes after the active-pointer swap, then restart
  recovery SHALL preserve the new active epoch.
- Cleanup SHALL identify generations by build id and manifest digest.
- Cleanup SHALL NOT delete an active generation, a generation named by a durable
  publish decision, or a Retired generation lacking a durable retire decision
  made under its coordinator retirement fence.
- A recovery process SHALL obtain generation state from
  `ec_distann_epoch_generation_status` rather than infer progress from relation
  existence or operator logs.
- `ec_distann_recover_epoch_publish` is repeatable across two post-decision
  transaction phases. T4a publishes every successor participant, waits for
  exact acknowledgements, conditionally swaps the active pointer from the
  recorded predecessor to the successor, records `Activated`, clears the build
  gate, and schedules both session-lock releases on commit. It performs no
  predecessor-retirement RPC after that commit in the same invocation. T4b is a
  later invocation/transaction that observes the committed successor pointer,
  marks each reachable Pending predecessor binding Retired, verifies exact
  activation digests,
  then records `Applied` once every binding is terminal `Retired` or
  `Abandoned`. With no predecessor, T4a may record `Applied`.
- Publish recovery SHALL be single-flight under a transaction-scoped advisory
  lock keyed by logical-index UUID. The pointer swap SHALL use the same lock and
  a conditional catalog update so concurrent explicit or scan-triggered
  recovery cannot apply it twice.
- Only the extension owner or an explicitly granted internal cluster role may
  execute `ec_distann_recover_epoch_publish` and its remote publish calls;
  `PUBLIC` and an ungranted reader SHALL never gain those side effects through
  a user query.
- An authorized backend that reads the coordinator active pointer while a
  `Pending` durable decision exists MAY attempt T4a only
  after acquiring the advisory lock non-blockingly. An unauthorized reader, or
  an authorized reader that cannot acquire the lock immediately, SHALL
  register and use the unchanged prior active fingerprint without waiting.
- If scan-triggered T4a cannot complete because a successor participant is
  unavailable, the scan SHALL register and use the unchanged predecessor active
  fingerprint. It SHALL NOT read a generation named only by a `Pending`
  decision. When state is `Activated`, scans SHALL register/use the committed
  successor pointer even while T4b predecessor marks are incomplete; they SHALL
  never fall back to the predecessor.
- If no durable decision exists, the recovery operation SHALL leave the prior
  active epoch unchanged and raise `EC_EPOCH_STATE`; the unpublished generation
  remains available through `ec_distann_epoch_build_status` for explicit resume
  or abort.
- T4a SHALL clear the coordinator build gate and release its source and control
  session locks only from the callback after the active-pointer swap commits.
  T4b uses immutable predecessor bindings and lifecycle rows and does not need
  the source lock, mutable registry, or live-build control lock.
- A successful publish-recovery call SHALL return the 34-byte active epoch
  fingerprint.

### Scan Consistency

- At scan start, the coordinator SHALL atomically select and locally register
  one active manifest and fingerprint for the entire attempt.
- Every expansion and materialization call SHALL carry the registered fingerprint.
- If any remote call raises epoch mismatch, then the coordinator SHALL discard
  all partial scan state.
- After the first epoch mismatch, the coordinator SHALL refresh the active
  manifest and restart once from the head index.
- After a second epoch mismatch, the coordinator SHALL fail the query.
- Each attempt SHALL maintain an independent
  [NFR-019](../../../non-functional/NFR-019-distann-per-query-touch-bound.md)
  expansion budget.
- A retained Retired generation SHALL remain addressable by its fingerprint for
  scans registered before its retirement fence closed.
- A reclaimed generation SHALL reject its fingerprint without returning data.

### Published Mutation and Retirement

- While an epoch is Published, its build-time graph records and adjacency SHALL
  remain immutable except for the mutations enumerated by FR-083.
- While an epoch is Published, its build-time row-tier tuples SHALL remain
  immutable.
- While an epoch is Published, FR-083 MAY append a new row-tier tuple and graph
  record for an insert or replacement.
- While an epoch is Published, FR-083 MAY atomically redirect one vec_id's
  owner-local directory entry from an old record to its complete replacement.
- A replaced record and row-tier tuple SHALL remain retained until the next
  epoch build.
- A graph-record `heap_tid` SHALL resolve the same frozen row-tier tuple for the
  complete retained lifetime of the epoch.
- Results SHALL be drawn only from records the scan actually expanded.
- Tombstones SHALL be honored at expansion and materialization time.
- A scan SHALL NOT observe a half-applied per-record back-edge amendment.
- A Retired epoch SHALL retain graph records, head sample, manifests,
  directories, and row-tier tuples until the coordinator commits and applies a
  retire decision made under the zero-in-flight retirement fence.
- Normal and forced retirement SHALL follow the coordinator decision and audit
  rules above; neither participant restart nor a participant-local timer may
  reclaim retained storage.

## Error Conditions

| Code | Condition | Required outcome |
|------|-----------|------------------|
| `EC_EPOCH_STATE` | Requested lifecycle transition is invalid, or publish recovery has no durable decision | Leave generation and active pointer unchanged |
| `EC_EPOCH_FINGERPRINT_VERSION` | Fingerprint is not exactly one supported version plus a 32-byte digest | Reject before generation lookup |
| `EC_EPOCH_PIN_CONFLICT` | One coordinator-local scan token is reused for another fingerprint | Leave both local registrations/counts unchanged; issue no participant call |
| `EC_EPOCH_PIN_CAPACITY` | Exact token or fence capacity remains exhausted after dead-token and provably dropped-fence reclamation | Fail before participant access; evict no live token or live logical-index fence and issue no participant call |
| `EC_EPOCH_REGISTRY_UNAVAILABLE` | Shared registry is absent, uninitialized, or version-incompatible | Fail distributed scan/retirement before participant access |
| `EC_PUBLISH_INCOMPLETE` | Receipt, schema, count, digest, coverage, co-placement, or topology proof is missing/mismatched before decision | Keep generation Ready and query-invisible; do not persist decision |
| `EC_PUBLISH_DIGEST` | Manifest/receipt bytes do not match their canonical digest | Reject before participant state or pointer mutation |
| `EC_PUBLISH_PENDING` | A successor participant is unavailable while decision state is `Pending` | Keep the predecessor pointer active; retain commit-only decision for retry |
| `EC_PUBLISH_CANCEL` | Cancellation is malformed, conflicts with its audit replay, targets a non-Pending decision, or the exact predecessor is no longer active | Change no pointer, decision, registration, or participant state |
| `EC_PREDECESSOR_RETIRE_PENDING` | A predecessor owner is unavailable after successor activation | Keep the successor pointer active and decision `Activated`; retry only missing predecessor marks |
| `EC_PREDECESSOR_ABANDON` | Abandonment is unauthorized, malformed, targets a non-Activated decision or non-Pending binding, or conflicts with an existing audit | Change no binding/decision state; issue no participant call |
| `EC_RETENTION_ACTIVE` | Normal retirement sees one or more coordinator-local in-flight references under the fence | Keep generation retained; require drain or explicit audited force-retire |
| `EC_GENERATION_MISSING` | Fingerprint/build id names an unknown, aborted, or reclaimed generation | Data/topology/scan endpoints return no generation and do not fall back; generation-status alone may return the Reclaimed tombstone |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-082-AC-1 | Queries spanning an active-pointer swap return rows wholly from the old or new epoch and never mix fingerprints | Test (TC-042) |
| FR-082-AC-2 | First fingerprint mismatch discards partial state and restarts from the refreshed head index; second mismatch errors | Test (TC-042) |
| FR-082-AC-3 | Retired storage remains readable by locally registered scans and is reclaimed only after a zero-in-flight durable retire decision | Test (TC-042) |
| FR-082-AC-4 | Concurrent FR-083 mutations expose only complete per-record states, honor tombstones, and return only expanded records | Test (TC-042, TC-043) |
| FR-082-AC-5 | A retained epoch's `heap_tid` resolves the byte-identical build-snapshot row despite source-table delete, VACUUM, or TID reuse | Test (TC-042) |
| FR-082-AC-6 | Participants never reclaim autonomously; force-retire rejects the active epoch, requires explicit operator action, and emits the complete override audit record | Test (TC-042) |
| FR-082-AC-7 | Building and Ready generations remain query-invisible while the prior Published epoch continues serving | Test (TC-042) |
| FR-082-AC-8 | Missing/mismatched participant receipt, count, digest, schema, coverage, or co-placement prevents the active-pointer swap | Test (TC-042) |
| FR-082-AC-9 | A coordinator crash at each publication boundary recovers to either the unchanged old active epoch or the fully acknowledged new active epoch | Test (TC-042) |
| FR-082-AC-10 | Replaying publish and recovery operations produces one Published generation per owner with unchanged counts/digests and no leaked generation | Test (TC-042) |
| FR-082-AC-11 | Source-row DML and schema changes are blocked from snapshot capture through publish/abort, while reads of the prior epoch continue | Test (TC-042) |
| FR-082-AC-12 | Roster reorder, addition, or removal creates a new fingerprint and never changes owner resolution for the retained old epoch | Test (TC-042) |
| FR-082-AC-13 | Shared-memory exact-token scan registration is idempotent, fails closed without preload/at capacity, reaps normal and abrupt backend exits, cleans up every error/cancel/restart path, adds zero participant RPC/WAL work per query, and collision-free retirement fencing prevents reclaim while a registered scan is live | Test (TC-042) |
| FR-082-AC-14 | Ready commit durably stores the exact canonical build candidate, and both decision and later recovery after client/backend loss recompute its digest chain and consume those bytes without recapturing the source snapshot | Test (TC-042, TC-050) |
| FR-082-AC-15 | After successor activation, every predecessor-roster binding—including an owner removed from the successor roster—reaches exactly one immutable Retired acknowledgement or explicit audited Abandoned disposition; exact replay is stable, a returning abandoned binding fails closed, and conflicting successor identity changes no state or physical bytes | Test (TC-042, TC-050) |
| FR-082-AC-16 | Retire apply atomically removes physical storage and leaves an immutable Reclaimed tombstone; exact replay/status succeeds from it and conflicting identity fails closed | Test (TC-042, TC-050) |
| FR-082-AC-17 | An audited Pending-decision cancellation leaves the predecessor active, clears the build gate, permanently blocks activation of the cancelled fingerprint, and keeps any partially published successor storage non-routable until explicit audited cancellation recovery | Test (TC-042) |
| FR-082-AC-18 | Cancelled-generation recovery reclaims each reachable Ready or Published-but-never-active participant generation from the exact canonical cancellation audit, leaves an immutable participant tombstone, and replays safely after partial remote completion | Test (TC-042, TC-050) |

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-082-CON-1 | Each logical ec_distann index SHALL have exactly one authoritative coordinator instance, one active-epoch pointer, one shared in-flight registry, and one per-index retirement fence | Integrity | State-machine and two-coordinator rejection test (TC-042) |
| FR-082-CON-2 | A scan SHALL execute at most two complete attempts | Resource | Fault drill (TC-042) |
| FR-082-CON-3 | A participant SHALL expose records only from Published or retained Retired generations selected by fingerprint | Integrity | Endpoint integration test (TC-040, TC-042) |
| FR-082-CON-4 | A durable publish decision SHALL be commit-only | Recovery | Crash-boundary drill (TC-042) |
| FR-082-CON-5 | An epoch fingerprint SHALL contain exactly one little-endian version field and one 32-byte SHA-256 manifest digest | Integrity | Wire-format test (TC-040) |

## Dependencies

- **Upstream**: [FR-078](../build/FR-078-distann-hash-placement.md),
  [FR-079](../read/FR-079-distann-remote-expansion-protocol.md), and
  [FR-080](../read/FR-080-distann-coordinator-head-index.md)
- **Downstream**: [FR-083](./FR-083-distann-dml-path.md)
