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

## Inputs

- The stitched global-graph identity and head sample from
  [FR-077](./FR-077-distann-sharded-build-and-stitch.md) and
  [FR-080](./FR-080-distann-coordinator-head-index.md).
- The ordered roster, placement version, participant Ready receipts, row-schema
  fingerprint, format/codec identity, source-snapshot identity, and canonical
  content/build-specification digests from
  [FR-078](./FR-078-distann-hash-placement.md).
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
| Published and active | coordinator activates a successor | Retired | retained scans may continue by fingerprint |
| Retired | coordinator fence observes zero in-flight scans and durable retire decision is applied | reclaimed | unavailable |

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

The version-2 canonical epoch manifest SHALL contain these fields in this order:

| Field | Wire type | Rule |
|-------|-----------|------|
| manifest_version | u16 | exactly 2 |
| epoch | u64 | non-zero; when a parent exists, greater than the epoch obtained by resolving `parent_fingerprint` to its retained manifest |
| build_id | byte[16] | UUID bytes from FR-078 |
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
| build_options | length-prefixed bytes | canonical version-1 graph/build options subrecord defined below |
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

Every array in the manifest SHALL begin with an unsigned little-endian `u32`
element count. Each participant receipt SHALL then be encoded as one
`u32` byte length followed by its complete canonical receipt bytes.

Each participant Ready receipt SHALL contain, in order: `receipt_version u16 =
1`, `node_id u32`, `epoch u64`, `build_id byte[16]`, build-specification digest,
generation-descriptor digest, `last_acknowledged_batch_sequence u64`, owned
record count `u64`, row count `u64`, owner-stream digest, persisted graph
digest, persisted row-tier digest, local-directory digest, graph bytes `u64`,
row-tier bytes `u64`, directory bytes `u64`, and `state u8 = 1` (`Ready`). Receipt
fixed-width integers use little-endian encoding; UUID uses RFC 4122 byte order;
digests are 32 bytes.

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
manifest_digest bytea, record_count bigint, row_count bigint,
retire_decision_digest bytea)`

`ec_distann_apply_epoch_retire(index_regclass regclass,
epoch_fingerprint bytea, retire_decision_digest bytea)
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
epoch_fingerprint bytea) RETURNS void`

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
  publish decision containing the manifest digest and complete receipt set.
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
- The participant SHALL verify that the manifest contains its exact Ready
  receipt and build-specification digest before changing state.
- A participant SHALL acknowledge publication only after its Published state,
  manifest, graph shard, row tier, directory, and receipt survive PostgreSQL
  restart.
- The coordinator SHALL wait for a matching Published acknowledgement from
  every participant.
- The coordinator SHALL atomically swap its active-epoch pointer only after all
  matching acknowledgements are durable.
- The active-pointer swap SHALL be the cluster's query-visible publication
  linearization point.
- Before that swap, new scans SHALL continue to register the prior active epoch
  in the coordinator-local in-flight registry. After that swap, new scans SHALL
  register the new epoch.
- A participant SHALL retain Published and Retired generation storage until it
  receives the coordinator's durable retire decision through
  `ec_distann_apply_epoch_retire`; it SHALL NOT reclaim autonomously.
- A roster change SHALL require a new build id, new epoch, and new manifest.
- A roster change SHALL NOT mutate placement inside an existing Published or
  Retired epoch.

### Coordinator Scan Retention and Retire Fence

- Before issuing the first expansion for one attempt, the coordinator SHALL
  generate a never-reused UUID scan token and atomically read/register the
  selected fingerprint in a coordinator-local in-flight registry under that
  logical index's retirement fence.
- The registry SHALL be visible to every backend on the authoritative
  coordinator, but it SHALL require no participant RPC, participant catalog
  write, WAL flush, or synchronous commit on the query path.
- Replaying the same local `(fingerprint, scan_token)` registration SHALL be
  idempotent. Reusing one scan token for another fingerprint SHALL raise
  `EC_EPOCH_PIN_CONFLICT` without changing either local count.
- A guard SHALL release the local registration on normal completion, error,
  cancellation, rescan, `EndCustomScan`, and epoch-mismatch restart. Backend or
  coordinator process death releases its registrations because no scan in that
  process can remain live.
- Normal retirement SHALL accept only a non-active Retired fingerprint. It
  SHALL acquire the fingerprint's exclusive retirement fence, prevent new
  local registrations, and reject with `EC_RETENTION_ACTIVE` unless the local
  in-flight count is zero.
- After observing zero under the fence, the coordinator SHALL commit one
  immutable retire decision containing the fingerprint, manifest digest,
  roster, caller, and timestamp before instructing the first participant to
  reclaim. Participants SHALL validate that decision and apply it
  idempotently. A crash after a subset applies is completed by
  `ec_distann_recover_epoch_retire` from the durable decision.
- Participant restart is safe without a durable per-scan token: a participant
  never reclaims because of restart, age, or local state alone. It reclaims
  only after the authoritative coordinator has fenced new scans, observed zero
  live scans, and committed the retire decision.
- `ec_distann_force_retire_epoch` SHALL still reject the active fingerprint.
  For a non-active Retired fingerprint it MAY override a non-zero local count
  only by committing an audit record containing the epoch, build id, manifest
  digest, overridden count, caller, reason, and timestamp before participant
  reclaim. It SHALL NOT run automatically.

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
- If a participant is unavailable after the decision, then the coordinator
  SHALL keep the old active pointer until all participants acknowledge the new
  Published generation.
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
- `ec_distann_recover_epoch_publish` SHALL inspect the durable coordinator
  decision and participant status, then idempotently complete every missing
  participant publish and active-pointer swap.
- Publish recovery SHALL be single-flight under a transaction-scoped advisory
  lock keyed by logical-index UUID. The pointer swap SHALL use the same lock and
  a conditional catalog update so concurrent explicit or scan-triggered
  recovery cannot apply it twice.
- Only the extension owner or an explicitly granted internal cluster role may
  execute `ec_distann_recover_epoch_publish` and its remote publish calls;
  `PUBLIC` and an ungranted reader SHALL never gain those side effects through
  a user query.
- An authorized backend that reads the coordinator active pointer while a
  durable decision is pending MAY attempt the same recovery procedure only
  after acquiring the advisory lock non-blockingly. An unauthorized reader, or
  an authorized reader that cannot acquire the lock immediately, SHALL
  register and use the unchanged prior active fingerprint without waiting.
- If scan-triggered recovery cannot complete because a participant is
  unavailable, then the scan SHALL likewise register and use the unchanged
  prior active fingerprint. The scan SHALL NOT read any generation named only
  by the pending decision, and the pending recovery category SHALL remain
  operator-visible.
- If no durable decision exists, the recovery operation SHALL leave the prior
  active epoch unchanged and raise `EC_EPOCH_STATE`; the unpublished generation
  remains available through `ec_distann_epoch_build_status` for explicit resume
  or abort.
- Publish recovery SHALL clear the coordinator build gate and release its
  session-level source lock only after the active-pointer swap commits.
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
| `EC_PUBLISH_INCOMPLETE` | Receipt, schema, count, digest, coverage, co-placement, or topology proof is missing/mismatched before decision | Keep generation Ready and query-invisible; do not persist decision |
| `EC_PUBLISH_DIGEST` | Manifest/receipt bytes do not match their canonical digest | Reject before participant state or pointer mutation |
| `EC_PUBLISH_PENDING` | A participant is unavailable after the durable decision | Keep the prior pointer active; retain commit-only decision for retry |
| `EC_RETENTION_ACTIVE` | Normal retirement sees one or more coordinator-local in-flight references under the fence | Keep generation retained; require drain or explicit audited force-retire |
| `EC_GENERATION_MISSING` | Fingerprint/build id names an unknown, aborted, or reclaimed generation | Return no generation data and do not fall back to another epoch |

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
| FR-082-AC-13 | Coordinator-local scan registration is idempotent, cleans up every normal/error/cancel/restart path, adds zero participant RPC/WAL work per query, and retirement fencing prevents reclaim while a registered scan is live | Test (TC-042) |

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-082-CON-1 | A coordinator SHALL expose exactly one active-epoch pointer per logical ec_distann index | Integrity | State-machine test (TC-042) |
| FR-082-CON-2 | A scan SHALL execute at most two complete attempts | Resource | Fault drill (TC-042) |
| FR-082-CON-3 | A participant SHALL expose records only from Published or retained Retired generations selected by fingerprint | Integrity | Endpoint integration test (TC-040, TC-042) |
| FR-082-CON-4 | A durable publish decision SHALL be commit-only | Recovery | Crash-boundary drill (TC-042) |
| FR-082-CON-5 | An epoch fingerprint SHALL contain exactly one little-endian version field and one 32-byte SHA-256 manifest digest | Integrity | Wire-format test (TC-040) |

## Dependencies

- **Upstream**: [FR-078](./FR-078-distann-hash-placement.md),
  [FR-079](./FR-079-distann-remote-expansion-protocol.md), and
  [FR-080](./FR-080-distann-coordinator-head-index.md)
- **Downstream**: [FR-083](./FR-083-distann-dml-path.md)
