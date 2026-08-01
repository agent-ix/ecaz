---
id: FR-087
title: DistANN Catalog Relations
type: FR
status: PROPOSED
object: data_schema
relationships:
  - target: "ix://agent-ix/ecaz/FR-082"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-078"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-087: DistANN Catalog Relations

## Description

The `ec_distann` extension SHALL persist its distributed control plane as
exactly twenty catalog relations created by the extension bootstrap SQL. These
relations are the durable substrate for the roster and identity registry
([FR-078](../build/FR-078-distann-hash-placement.md)), the T1–T4a build and
epoch lifecycle ([FR-082](../lifecycle/FR-082-distann-epoch-lifecycle.md)),
the head persistence and replica attestation
([FR-080](../read/FR-080-distann-coordinator-head-index.md)), and the opt-in
traversal replica
([FR-084](../read/FR-084-distann-coordinator-traversal-replica.md)).

Every catalog row SHALL be scoped by `(index_oid, logical_index_uuid)` (the
head-replica pair, which is epoch-keyed, scopes by `index_oid` plus the
34-byte epoch fingerprint) so that PostgreSQL OID reuse can never select
state belonging to a different logical index. Catalog access SHALL flow only
through the extension's SECURITY DEFINER control-plane endpoints; direct
table privileges SHALL be revoked from PUBLIC for every relation in this
schema.

Under [NFR-021](../../../non-functional/NFR-021-distann-distribution-invariant.md)
each relation carries a storage class:

- **control** — metadata whose row count is bounded by roster size, build
  count, and epoch count, never by corpus size N. These relations MUST audit
  to zero derived vector bytes.
- **bounded** — copies of the bounded head structure (capacity C divided
  across the roster, times the replica count) or the explicitly
  non-conforming traversal replica bookkeeping. Bounded relations may hold
  landmark vectors but never O(N) graph or row-tier content.

## Schema

```json
{
  "schema": "ec_distann_catalog",
  "relation_count": 20,
  "groups": {
    "identity_registry": [
      { "relation": "ec_distann_participant_identity", "storage_class": "control", "scope": "logical_index", "primary_key": ["index_oid", "logical_index_uuid"] },
      { "relation": "ec_distann_registry_state", "storage_class": "control", "scope": "logical_index", "primary_key": ["index_oid", "logical_index_uuid"] },
      { "relation": "ec_distann_node_descriptor", "storage_class": "control", "scope": "logical_index", "primary_key": ["index_oid", "logical_index_uuid", "roster_ordinal"] }
    ],
    "build_coordination": [
      { "relation": "ec_distann_build_registration", "storage_class": "control", "scope": "build", "primary_key": ["index_oid", "logical_index_uuid", "build_id"] },
      { "relation": "ec_distann_build_participant_binding", "storage_class": "control", "scope": "build", "primary_key": ["index_oid", "logical_index_uuid", "build_id", "roster_ordinal"] },
      { "relation": "ec_distann_build_candidate", "storage_class": "control", "scope": "build", "primary_key": ["index_oid", "logical_index_uuid", "build_id"] }
    ],
    "generations_batches": [
      { "relation": "ec_distann_generation", "storage_class": "control", "scope": "build", "primary_key": ["index_oid", "logical_index_uuid", "build_id"] },
      { "relation": "ec_distann_generation_batch", "storage_class": "control", "scope": "build", "primary_key": ["index_oid", "logical_index_uuid", "build_id", "batch_seq"] }
    ],
    "head_state_replicas": [
      { "relation": "ec_distann_generation_head_state", "storage_class": "bounded", "scope": "build", "primary_key": ["index_oid", "logical_index_uuid", "build_id"], "gaps": ["revoke_missing"] },
      { "relation": "ec_distann_generation_head_sample", "storage_class": "bounded", "scope": "build", "primary_key": ["index_oid", "logical_index_uuid", "build_id", "sample_ordinal"], "gaps": ["revoke_missing"] },
      { "relation": "ec_distann_head_shard_replica", "storage_class": "bounded", "scope": "epoch_fingerprint", "primary_key": ["index_oid", "epoch_fingerprint", "vec_id"], "gaps": ["revoke_missing", "no_reclaim_path"] },
      { "relation": "ec_distann_head_replica_state", "storage_class": "control", "scope": "epoch_fingerprint", "primary_key": ["index_oid", "epoch_fingerprint"], "gaps": ["revoke_missing", "no_reclaim_path"] }
    ],
    "publish_retire_reclaim_ledgers": [
      { "relation": "ec_distann_publish_decision", "storage_class": "control", "scope": "build", "primary_key": ["index_oid", "logical_index_uuid", "build_id"] },
      { "relation": "ec_distann_predecessor_disposition", "storage_class": "control", "scope": "build", "primary_key": ["index_oid", "logical_index_uuid", "successor_build_id", "predecessor_roster_ordinal"] },
      { "relation": "ec_distann_retire_decision", "storage_class": "control", "scope": "epoch_fingerprint", "primary_key": ["index_oid", "logical_index_uuid", "epoch_fingerprint"] },
      { "relation": "ec_distann_generation_reclaim", "storage_class": "control", "scope": "build", "primary_key": ["index_oid", "logical_index_uuid", "build_id"] },
      { "relation": "ec_distann_cancelled_generation_reclaim", "storage_class": "control", "scope": "build", "primary_key": ["index_oid", "logical_index_uuid", "build_id"] }
    ],
    "active_epoch": [
      { "relation": "ec_distann_active_epoch", "storage_class": "control", "scope": "logical_index", "primary_key": ["index_oid", "logical_index_uuid"] }
    ],
    "traversal_replica": [
      { "relation": "ec_distann_traversal_replica", "storage_class": "bounded", "scope": "build", "primary_key": ["index_oid", "logical_index_uuid", "build_id"] },
      { "relation": "ec_distann_traversal_replica_owner", "storage_class": "control", "scope": "build", "primary_key": ["index_oid", "logical_index_uuid", "build_id", "owner_ordinal"] }
    ]
  }
}
```

### Identity and Registry

#### ec_distann_participant_identity

Storage class: control. One row per participant-side v5 control index,
binding `(index_oid, logical_index_uuid)` to the node's `endpoint_identity`.

- Key columns: `endpoint_identity` SHALL match
  `^[A-Za-z0-9][A-Za-z0-9._/-]{0,254}$`.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid)`. No FKs.
- Writers: `ec_distann_configure_participant_identity`. Readers: the
  participant handoff and lifecycle endpoints that validate caller identity.
- Lifecycle: created at participant configuration; deleted by
  `catalog_index_cleanup` when the index is dropped. Not epoch-scoped.

#### ec_distann_registry_state

Storage class: control. Roster-change revision fence: a single monotonic
`revision` counter per logical index.

- Key columns: `revision bigint` SHALL be `>= 0`; every roster mutation SHALL
  bump it so an in-flight build detects roster drift against its snapshotted
  `registry_revision`.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid)`. No FKs.
- Writers: `ec_distann_initialize_control_registry` (init),
  `ec_distann_register_node_descriptor` /
  `ec_distann_unregister_node_descriptor` (bump). Readers: T1 registration
  and the drop-index event trigger.
- Lifecycle: one row per logical index for its whole life. Not epoch-scoped.

#### ec_distann_node_descriptor

Storage class: control. The operator-managed roster: one row per registered
participant of a logical index.

- Key columns: `roster_ordinal >= 0`, `node_id > 0`, validated
  `endpoint_identity`, `conninfo_secret_name` (SHALL match
  `^[A-Z][A-Z0-9_]{0,127}$` — a secret *name*, never a connection string),
  `remote_index_regclass` (schema-qualified lowercase identifier pattern),
  `participant_logical_index_uuid`, and a 32-byte `compatibility_digest`.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid, roster_ordinal)`;
  UNIQUE on `node_id`, on `endpoint_identity`, and on
  `participant_logical_index_uuid` within the logical index; partial unique
  index enforcing at most one `is_local` row per logical index.
- Writers: `ec_distann_register_node_descriptor`,
  `ec_distann_unregister_node_descriptor` (which SHALL refuse removal while
  the descriptor is referenced by an active build). Readers: T1 roster
  snapshot, read-path connection resolution.
- Lifecycle: operator-managed; deleted by index cleanup. Not epoch-scoped —
  builds consume an immutable snapshot via the participant binding table.

### Build Coordination

#### ec_distann_build_registration

Storage class: control. The coordinator's per-build gate row and build state
machine.

- Key columns: RFC 4122 v4 `build_id` (byte-level version/variant CHECK),
  `epoch > 0`, `state` SHALL be one of `Registered`, `Building`, `Ready`,
  `Aborted`, `Decided`, `Published`, `Cancelled`; `registry_revision`,
  `roster_snapshot` (non-empty), 32-byte `roster_digest`,
  `row_schema_fingerprint`, and `registration_digest`.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid, build_id)`; UNIQUE
  `(…, epoch)`; UNIQUE `(…, build_id, epoch, registration_digest)` (FK
  target for the candidate). Partial unique index
  `ec_distann_build_registration_one_gate_active` SHALL enforce at most one
  build in `{Registered, Building, Ready, Decided}` per logical index.
- Writers: `ec_distann_begin_epoch_build` (T1),
  `ec_distann_build_epoch` / `_with_training` (T2),
  `ec_distann_decide_epoch_publish` (T3), `ec_distann_recover_epoch_publish`
  (T4a), `ec_distann_abort_epoch_build`, `ec_distann_cancel_epoch_publish`.
  Readers: `ec_distann_epoch_build_status`, the DML build-gate statement
  trigger, `ec_distann_build_gate_relation_mask`.
- Lifecycle: coordinator-owned; rows persist as build history until index
  cleanup. Epoch-scoped by `epoch`.

#### ec_distann_build_participant_binding

Storage class: control. Immutable per-build roster snapshot: one row per
participant of one build, fanned out at T1.

- Key columns: mirrors the node-descriptor validation CHECKs
  (`roster_ordinal`, `node_id`, `endpoint_identity`,
  `conninfo_secret_name`, `remote_index_regclass`,
  `participant_logical_index_uuid`, 32-byte `compatibility_digest`,
  `is_local`). Rows SHALL NOT be updated after T1.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid, build_id,
  roster_ordinal)`; per-build UNIQUEs on `node_id`, `endpoint_identity`,
  `participant_logical_index_uuid`, and the wide identity tuple (FK target
  for predecessor dispositions); partial unique one-local per build. FK to
  `ec_distann_build_registration` ON DELETE CASCADE.
- Writers: `ec_distann_begin_epoch_build` (T1 insert only). Readers: T2
  fan-out, T4a disposition fan-out, retire endpoints.
- Lifecycle: cascades with its registration. Epoch-scoped via `build_id`.

#### ec_distann_build_candidate

Storage class: control. The immutable T2 output: the sealed candidate that a
publish decision can later reference.

- Key columns: v4 `build_id`, `epoch > 0`, non-empty `build_spec`,
  `generation_descriptor`, `source_snapshot`, `ready_receipt_set`, and
  `epoch_manifest`, each paired with a 32-byte digest column; the 34-byte
  `epoch_fingerprint` SHALL equal `u16_le(version) || manifest_digest` per
  [FR-082](../lifecycle/FR-082-distann-epoch-lifecycle.md); 32-byte
  `candidate_digest` covers the whole candidate.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid, build_id)`; wide UNIQUE
  over `(build_id, epoch, registration_digest, candidate_digest,
  manifest_digest, epoch_fingerprint)` (FK target for the publish decision).
  FK to the registration's `(build_id, epoch, registration_digest)` UNIQUE
  ON DELETE CASCADE; held ON DELETE RESTRICT by `ec_distann_publish_decision`
  once decided.
- Writers: `ec_distann_build_epoch` / `_with_training` (T2 insert only).
  Readers: T3 decide, T4a recover, status endpoints.
- Lifecycle: cascades with its registration until a decision RESTRICT-pins
  it. Epoch-scoped.

### Generations and Batches

#### ec_distann_generation

Storage class: control (the generation's graph/row/directory relations hold
the O(N) data; this row is their bounded catalog record). One row per
physical shard generation on its owning participant.

- Key columns: v4 `build_id`, `epoch > 0`, `owner_ordinal >= 0`,
  `node_id > 0`; `state` SHALL be one of `Building`, `Ready`, `Published`,
  `Retired`; 32-byte digests for build spec, roster, generation descriptor,
  expected-owner set, and cumulative owner stream; `row_tier_relid`,
  `graph_store_relid`, `directory_relid` (all non-zero); streaming resume
  state (`next_batch_seq`, `cumulative_record_count`, optional 8-byte
  `last_vec_id_le`, 107-byte tagged `owner_stream_sha256_state`); optional
  303-byte `ready_receipt`; publish/retire columns (`epoch_fingerprint` 34
  bytes, `manifest_digest`, `epoch_manifest`, `published_at`,
  `successor_activation`, `successor_activation_digest`, `retired_at`).
- State-machine invariants (load-bearing CHECKs, all SHALL hold):
  - `(cumulative_record_count = 0) = (last_vec_id_le IS NULL)`;
  - a non-`Building` row SHALL have `next_batch_seq > 0`;
  - `ready_receipt` SHALL be NULL exactly while `Building`;
  - `Published`/`Retired` SHALL each carry fingerprint + manifest digest +
    manifest + `published_at`, and only those states may;
  - `Retired` SHALL additionally carry `successor_activation`, its digest,
    and `retired_at`, and only `Retired` may.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid, build_id)`; global
  UNIQUEs on each of `row_tier_relid`, `graph_store_relid`,
  `directory_relid`; partial unique fingerprint index per logical index
  where `epoch_fingerprint IS NOT NULL`.
- Writers: `ec_distann_begin_epoch_handoff`, `ec_distann_stage_epoch_batch`,
  `ec_distann_seal_epoch_handoff`, `ec_distann_abort_epoch_handoff`,
  `ec_distann_publish_epoch`, `ec_distann_mark_epoch_retired`,
  `ec_distann_apply_epoch_retire`, `ec_distann_reclaim_cancelled_generation`.
  Readers: `ec_distann_epoch_generation_status`,
  `ec_distann_list_unpublished_generations`, the read path's generation
  resolution.
- Lifecycle: participant-owned; row is deleted (with its physical relations)
  at retire/cancel reclaim, leaving a tombstone in the reclaim ledgers.
  Epoch-scoped.

#### ec_distann_generation_batch

Storage class: control (bounded by batch count of a build, not by N —
`encoded_bytes` SHALL be between 141 and 8388608). Append-only batch
acknowledgement ledger for the handoff stream.

- Key columns: `batch_seq >= 0`, 32-byte `batch_digest` and
  `cumulative_owner_digest`, `accepted_record_count >= 0`,
  `cumulative_record_count >= accepted_record_count`; batch 0 SHALL be the
  only batch permitted to accept zero records, and its cumulative count
  SHALL equal its accepted count.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid, build_id, batch_seq)`;
  FK to `ec_distann_generation` ON DELETE CASCADE.
- Writers: `ec_distann_stage_epoch_batch` (insert only). Readers: staging
  replay validation on retried batches.
- Lifecycle: cascades with the generation. Epoch-scoped.

### Head State and Replicas

#### ec_distann_generation_head_state

Storage class: bounded (capacity C metadata; membership-only rows hold zero
vector bytes). Per-build head attestation row: makes an empty head
distinguishable from a missing or corrupt one.

- Key columns: `dimensions` in `(0, 65535]`, `sample_count >= 0`, 32-byte
  `head_sample_digest`, `head_graph_digest`, `training_query_digest`;
  `head_policy` SHALL be 0 (uniform) or 1 (trained), and the policy CHECK
  SHALL hold: policy 0 requires `training_query_count = 0` with an all-zero
  training digest, policy 1 requires `training_query_count = 200` with a
  non-zero digest.
- Membership invariant: `membership` SHALL be NULL (legacy full-vector
  shape) or exactly `4 + 8 * sample_count` bytes (u32 count + u64 vec_ids,
  little-endian). The membership-only shape is NFR-021 clause 3: on a
  multi-owner roster the coordinator persists head membership only, and the
  sample table SHALL then hold zero rows for the build.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid, build_id)`; FK to
  `ec_distann_build_candidate` ON DELETE CASCADE.
- Writers: T2 head construction (`ec_distann_build_epoch` /
  `_with_training`). Readers: `ec_distann_active_head_policy`, head search
  and shard export.
- Lifecycle: cascades with the candidate. Epoch-scoped.
- Gap (normative obligation, current code non-conforming): this relation
  SHALL be in the bootstrap REVOKE-from-PUBLIC block; as audited it is not.

#### ec_distann_generation_head_sample

Storage class: bounded (at most C rows per build; zero rows in
membership-only mode). Per-landmark rows of the coordinator head.

- Key columns: `sample_ordinal >= 0`, `vec_id`, `neighbors integer[]` NOT
  NULL; `vector real[]` SHALL be NULL under sharded head storage (the
  landmark's full-precision vector lives on its
  [FR-078](../build/FR-078-distann-hash-placement.md) hash owner) — a
  non-NULL vector is the unsharded legacy shape only.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid, build_id,
  sample_ordinal)`; UNIQUE `(…, build_id, vec_id)`; FK to
  `ec_distann_generation_head_state` ON DELETE CASCADE.
- Writers: T2 head construction. Readers: legacy coordinator-resident head
  search; single-owner rosters.
- Lifecycle: cascades with head state. Epoch-scoped. In the shipped
  multi-owner default this table SHALL audit to zero derived vector bytes.
- Gap: SHALL be in the REVOKE block; as audited it is not.

#### ec_distann_head_shard_replica

Storage class: bounded — an attested copy of head shards on non-owner nodes
per the head-shard replica clauses of
[FR-080](../read/FR-080-distann-coordinator-head-index.md) (the
DISTRIBUTEDANN-paper §4.1 mechanism): at most C landmarks divided across
the roster, times the replica count; never O(N) content.

- Key columns: 34-byte `epoch_fingerprint` scope, `shard_ordinal >= 0`,
  `vec_id`, `vector real[]` NOT NULL (a replica must hold the shard's
  landmark vectors to serve it).
- Keys: PRIMARY KEY `(index_oid, epoch_fingerprint, vec_id)`; secondary
  index by `(index_oid, epoch_fingerprint, shard_ordinal)`. No FKs — scoped
  by fingerprint, not by catalog row.
- Writers: `ec_distann_head_shard_import` (fed by
  `ec_distann_head_shard_export` / `ec_distann_populate_head_replicas`).
  Readers: replica-local head search (`ec_distann_head_search_physical`
  routing).
- Lifecycle: epoch-scoped and rebuildable.
- Gaps (normative obligations, current code non-conforming):
  1. Rows for a retired epoch or a dropped index SHALL be deleted; as
     audited there is no deletion path anywhere (not in
     `catalog_index_cleanup`, not in any retire/reclaim endpoint), so stale
     epochs and dropped indexes leak rows. Candidate code fix: add this
     relation to index cleanup and to epoch retirement.
  2. SHALL be in the REVOKE block; as audited it is not.

#### ec_distann_head_replica_state

Storage class: control (one row per (index, epoch)). Population attestation:
written only after every (shard, replica) pair has imported, so routing may
trust that a replica holds its shard.

- Key columns: 34-byte `epoch_fingerprint`, `replica_count >= 0`.
- Keys: PRIMARY KEY `(index_oid, epoch_fingerprint)`. No FKs.
- Writers: `ec_distann_populate_head_replicas` (final attestation insert).
  Readers: the head-search routing gate — absent attestation, routing SHALL
  fall back to owner-only fan-out.
- Lifecycle: epoch-scoped.
- Gaps: same two as `ec_distann_head_shard_replica` — no deletion/reclaim
  path (SHALL be reclaimed with its epoch; candidate code fix alongside the
  replica table) and absent from the REVOKE block (SHALL be revoked).

### Publish, Retire, and Reclaim Ledgers

#### ec_distann_publish_decision

Storage class: control. The coordinator's durable T3 decision ledger and
T4a recovery state machine.

- Key columns: v4 `build_id`, `epoch > 0`, 34-byte `epoch_fingerprint`,
  32-byte `manifest_digest` / `registration_digest` / `candidate_digest`,
  non-empty `epoch_manifest` and `successor_activation` (+ 32-byte digest);
  `decision_state` SHALL be one of `Pending`, `Activated`, `Applied`,
  `Cancelled`.
- Predecessor invariant: the predecessor quadruple (`predecessor_build_id`,
  `predecessor_epoch`, `predecessor_epoch_fingerprint`,
  `predecessor_manifest_digest`) SHALL be all-NULL (first epoch) or
  all-present with valid lengths, and SHALL resolve through the
  self-referential predecessor FK to an existing decision row
  (ON DELETE RESTRICT) — the decision chain cannot dangle.
- State invariants (SHALL hold): `activated_at` is NULL exactly in
  `{Pending, Cancelled}`; `applied_at` is non-NULL exactly in `Applied`;
  `Cancelled` requires the full cancellation record (`cancelled_by`,
  `cancellation_reason` 1–1024 bytes, `cancellation_time_unix_micros`,
  non-empty `cancellation_audit` + 32-byte digest) and non-`Cancelled` rows
  SHALL carry none of it.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid, build_id)`; UNIQUE on
  `epoch_fingerprint` per logical index; two wide UNIQUEs (FK targets for
  dispositions and retire decisions); partial unique
  `one_recovery_active` SHALL enforce at most one decision in
  `{Pending, Activated}` per logical index. FKs to the registration and the
  candidate, both ON DELETE RESTRICT.
- Writers: `ec_distann_decide_epoch_publish` (T3 insert Pending),
  `ec_distann_recover_epoch_publish` (T4a Pending → Activated → Applied),
  `ec_distann_cancel_epoch_publish` /
  `ec_distann_recover_cancelled_publish`. Readers: active-epoch FK
  validation, retire endpoints, abandonment.
- Lifecycle: permanent coordinator ledger until index cleanup. Epoch-scoped.

#### ec_distann_predecessor_disposition

Storage class: control. Per-(successor build × predecessor roster ordinal)
settlement rows: every predecessor binding SHALL settle (`Retired` or
`Abandoned`) before the successor decision may reach `Applied`.

- Key columns: v4 successor/predecessor build ids, predecessor epoch
  identity triple (epoch, 34-byte fingerprint, 32-byte manifest digest),
  binding identity (ordinal, node id, endpoint, regclass, participant
  UUID), `successor_activation_digest`; `disposition` SHALL be `Pending`,
  `Retired`, or `Abandoned`.
- Disposition invariant (SHALL hold): `Pending` carries no settlement
  fields; `Retired` requires `retired_activation_digest =
  successor_activation_digest` and no abandon fields; `Abandoned` requires
  the full abandon audit (non-empty `abandon_audit` + 32-byte digest,
  `abandon_caller_name`, `abandon_reason` 1–1024 bytes,
  `abandon_time_unix_micros`) and no retire digest.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid, successor_build_id,
  predecessor_roster_ordinal)`; UNIQUE per successor on
  `participant_logical_index_uuid`; RESTRICT FKs to the publish decision's
  predecessor tuple and to the predecessor's participant binding.
- Writers: `ec_distann_recover_epoch_publish` (T4a fan-out insert),
  `ec_distann_retire_epoch` / `ec_distann_recover_epoch_retire` (→
  `Retired`), `ec_distann_abandon_predecessor_binding` (→ `Abandoned`).
  Readers: T4a apply gate, retire recovery.
- Lifecycle: settles then persists as ledger history. Epoch-scoped.

#### ec_distann_retire_decision

Storage class: control. Coordinator retire ledger: one durable decision per
retired epoch.

- Key columns: epoch identity (v4 `build_id`, `epoch`, 34-byte fingerprint,
  32-byte manifest digest), roster snapshot + digest, `abandoned_binding_set`
  (length ≥ 4) + digest, canonical `retire_decision` payload + digest,
  `forced` with its invariant (an unforced decision SHALL have
  `overridden_in_flight_count = 0` and `reason = 'normal'`), `caller_name`,
  `decision_time_unix_micros`; `decision_state` SHALL be `Pending` or
  `Applied`, with `applied_at` non-NULL exactly when `Applied`.
- Covering-successor invariant: the FK pair SHALL pin the retire decision to
  a publish decision whose successor row is in `decision_state = 'Applied'`
  (the `covering_successor_decision_state` column is CHECK-constrained to
  `'Applied'` and included in the FK) — an epoch cannot retire before its
  successor is fully applied.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid, epoch_fingerprint)`;
  RESTRICT FKs to the retired epoch's publish decision and to the covering
  successor tuple.
- Writers: `ec_distann_retire_epoch`, `ec_distann_force_retire_epoch`,
  `ec_distann_recover_epoch_retire`. Readers: participant
  `ec_distann_apply_epoch_retire` validation.
- Lifecycle: permanent ledger. Epoch-scoped.

#### ec_distann_generation_reclaim

Storage class: control. Participant-side retire tombstone: proof that a
generation's physical relations were reclaimed. Deliberately FK-free so the
tombstone survives deletion of the generation row it describes.

- Key columns: epoch identity, 32-byte spec/descriptor digests,
  `record_count` / `row_count` (≥ 0), optional
  `successor_activation_digest`, canonical `retire_decision` + digest,
  `reclaimed_at`.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid, build_id)`; UNIQUE per
  logical index on `epoch_fingerprint`. No FKs.
- Writers: `ec_distann_apply_epoch_retire` (insert at reclaim). Readers:
  retire recovery idempotence checks.
- Lifecycle: tombstone; persists until index cleanup. Epoch-scoped.

#### ec_distann_cancelled_generation_reclaim

Storage class: control. Cancel-path counterpart tombstone for generations
reclaimed from `prior_state IN ('Ready', 'Published')` after a build
cancellation.

- Key columns: epoch identity, `prior_state` CHECK, spec/descriptor digests,
  counts, non-empty `cancellation_audit` + 32-byte digest, `reclaimed_at`.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid, build_id)`; UNIQUE on
  `epoch_fingerprint`. No FKs.
- Writers: `ec_distann_reclaim_cancelled_generation`. Readers: cancel
  recovery idempotence checks.
- Lifecycle: tombstone; persists until index cleanup. Epoch-scoped.

### Active Epoch

#### ec_distann_active_epoch

Storage class: control. The single active-epoch pointer — the read path's
sole epoch authority per
[FR-082](../lifecycle/FR-082-distann-epoch-lifecycle.md).

- Key columns: v4 `build_id`, `epoch > 0`, 34-byte `epoch_fingerprint`,
  32-byte `manifest_digest`, `updated_at`.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid)` — there SHALL be at
  most one active epoch per logical index; UNIQUE
  `(…, epoch_fingerprint)`; FK (NO ACTION) to the publish decision's epoch
  tuple, so the pointer can only ever name a decided epoch.
- Writers: `ec_distann_recover_epoch_publish` (T4a) is the ONLY writer; the
  pointer SHALL advance only by the T4a compare-and-swap. Readers: every
  scan open (`scan_epoch` resolution), `ec_distann_epoch_status`,
  `ec_distann_epoch_fingerprint`, retire preconditions.
- Lifecycle: one row per logical index once first published; deleted at
  index cleanup. Epoch-scoped by value, not by row multiplicity.

### Traversal Replica

#### ec_distann_traversal_replica

Storage class: bounded catalog metadata for the opt-in, non-conforming
coordinator graph copy of
[FR-084](../read/FR-084-distann-coordinator-traversal-replica.md) (the O(N)
payload lives in the separate `replica_relid` / `directory_relid`
relations, which NFR-021 accounts as non-conforming replica bytes, not as
this catalog row).

- Key columns: v4 `build_id`, 34-byte `epoch_fingerprint`, 32-byte
  `generation_descriptor_digest`, optional 32-byte `content_digest`;
  `state` SHALL be one of `Building`, `Ready`, `Stale`, `Retiring`;
  non-zero `replica_relid` / `directory_relid`; `format_version = 1`;
  shape columns (`dimensions`, `graph_degree`, `neighbor_codec_kind`,
  `owner_count`, all bounded CHECKs); copy progress counters with
  `copied_record_count <= expected_record_count`; `state_reason`
  (1–256 chars), optional `last_error` (1–1024 chars), state timestamps.
- Four-way state-shape invariant (SHALL hold): `Building` has no
  `content_digest` and no ready/stale/retiring timestamps; `Ready`, `Stale`,
  and `Retiring` each require `content_digest`, a complete copy
  (`copied_record_count = expected_record_count`), `ready_at`, and their
  own marker timestamp (`stale_at` for Stale, `retiring_at` for Retiring).
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid, build_id)`; UNIQUE per
  logical index on `epoch_fingerprint`; global UNIQUEs on `replica_relid`
  and `directory_relid`; partial unique `one_active` (at most one
  Building/Ready row per logical index) and `one_local_authority` (at most
  one Building/Ready row per `logical_index_uuid` across all local control
  indexes). FK to `ec_distann_build_candidate` ON DELETE CASCADE.
- Writers: the traversal-replica endpoint family
  (`ec_distann_build_traversal_replica`, mark-stale, guard-mutation,
  control-preflight, recover-invalidation, retire, reclaim), plus a
  statement trigger guarding direct mutation. Readers: replica-eligible
  scan routing, `ec_distann_traversal_replica_status`.
- Lifecycle: owns its own retire/reclaim path (separate from generation
  reclaim); non-authoritative and rebuildable. Epoch-scoped.

#### ec_distann_traversal_replica_owner

Storage class: control. Per-owner copy completion receipts for a traversal
replica build.

- Key columns: `owner_ordinal >= 0`, expected/copied record counts with the
  invariant `copied_record_count = expected_record_count` (a receipt row
  SHALL exist only for a completed owner stream), 32-byte `content_digest`,
  `copied_bytes`, `completed_at`.
- Keys: PRIMARY KEY `(index_oid, logical_index_uuid, build_id,
  owner_ordinal)`; FK to `ec_distann_traversal_replica` ON DELETE CASCADE.
- Writers: `ec_distann_stream_traversal_replica_chunk` completion path.
  Readers: replica readiness validation.
- Lifecycle: cascades with the replica row. Epoch-scoped.

### Endpoint Naming Hazard

The current extension exposes legacy v1 lifecycle endpoints that share SQL
function names with the physical-generation endpoints and are distinguished
only by argument signature (overloads). Catalog writers listed above refer
to the physical-generation signatures. Tooling and operators SHALL resolve
these endpoints by full signature, never by bare name; a future revision
SHOULD remove or rename the v1 overloads to eliminate the trap.

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-087-CON-1 | Every catalog relation is control-plane metadata whose row count and size are bounded by roster size, build count, and epoch count — never by corpus N — except `ec_distann_generation_head_sample` and `ec_distann_head_shard_replica`, which are bounded by head capacity C times replica count | Invariant | NFR-021 storage audit over all twenty relations |
| FR-087-CON-2 | Control-class relations audit to zero derived vector bytes; in the multi-owner default, `ec_distann_generation_head_state.membership` is non-NULL and `ec_distann_generation_head_sample` holds zero rows | Invariant | NFR-021 clause 3 audit query |
| FR-087-CON-3 | `ec_distann_generation_head_state.membership` is NULL or exactly `4 + 8 * sample_count` bytes | Schema CHECK | Insert of a mis-sized blob is rejected by the CHECK |
| FR-087-CON-4 | At most one active-epoch pointer per logical index, advanced only by the T4a state machine and only to a decided epoch | Schema (PK + FK) | PK `(index_oid, logical_index_uuid)` and FK to `ec_distann_publish_decision` reject a second pointer or an undecided target |
| FR-087-CON-5 | At most one active build gate (`Registered`/`Building`/`Ready`/`Decided`) and at most one recovery-active publish decision (`Pending`/`Activated`) per logical index | Schema (partial unique) | Concurrent T1/T3 attempts fail with unique violations |
| FR-087-CON-6 | All twenty relations have PUBLIC privileges revoked; access is exclusively via SECURITY DEFINER endpoints | Security | `information_schema.table_privileges` shows no PUBLIC grants (currently fails for the four head relations — see gap notes) |
| FR-087-CON-7 | Every epoch-scoped row is deleted or tombstoned by index cleanup or its lifecycle reclaim path; no relation accumulates rows for dropped indexes or retired epochs | Lifecycle | Drop-index + retire-epoch test leaves zero orphan rows (currently fails for `ec_distann_head_shard_replica` and `ec_distann_head_replica_state` — see gap notes) |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-087-AC-1 | The extension bootstrap creates exactly the twenty catalog relations named in the Schema section, with the primary keys, unique constraints, foreign keys, and CHECK invariants stated per table | Inspection of `sql/bootstrap.sql` against this spec |
| FR-087-AC-2 | Every relation scoped by `(index_oid, logical_index_uuid)` (or `(index_oid, epoch_fingerprint)` for the head-replica pair) rejects rows violating its state-machine CHECKs: generation states, publish-decision states, disposition settlement shapes, retire coverage, and traversal-replica state shapes | Test: constraint-violation inserts fail for each state machine |
| FR-087-AC-3 | The membership blob CHECK holds: a `membership` value not equal to `4 + 8 * sample_count` bytes is rejected, and a membership-only head leaves zero rows in `ec_distann_generation_head_sample` | Test over head persistence in sharded mode |
| FR-087-AC-4 | The active-epoch pointer is unique per logical index, references an existing publish decision, and changes only via `ec_distann_recover_epoch_publish` | Test: direct second-row insert and dangling-FK insert fail; T4a CAS succeeds |
| FR-087-AC-5 | PUBLIC privileges are revoked on all twenty relations, including `ec_distann_generation_head_state`, `ec_distann_generation_head_sample`, `ec_distann_head_shard_replica`, and `ec_distann_head_replica_state` | Test: privilege query as an unprivileged role returns no access (known gap: the four head relations are currently missing from the REVOKE block) |
| FR-087-AC-6 | Epoch retirement and index drop reclaim `ec_distann_head_shard_replica` and `ec_distann_head_replica_state` rows along with the rest of the epoch's catalog state | Test: retire + drop leave zero rows (known gap: no deletion path exists today; candidate code fix) |
| FR-087-AC-7 | An NFR-021 storage audit over the catalog classifies every relation into its declared storage class and finds zero derived vector bytes in control-class relations | Audit query per [NFR-021](../../../non-functional/NFR-021-distann-distribution-invariant.md) |
| FR-087-AC-8 | Control-plane tooling resolves the physical-generation endpoints by full signature, unaffected by the same-named legacy v1 overloads | Inspection + call-path test |

## Dependencies

- **Upstream**: [FR-082](../lifecycle/FR-082-distann-epoch-lifecycle.md)
  (epoch lifecycle whose state the ledgers persist),
  [FR-078](../build/FR-078-distann-hash-placement.md) (roster, placement,
  and build protocol whose registry this schema stores).
- **Related**: [FR-080](../read/FR-080-distann-coordinator-head-index.md)
  (head object persisted by the head-state group),
  [FR-084](../read/FR-084-distann-coordinator-traversal-replica.md)
  (traversal replica whose catalog rows this schema defines),
  [NFR-021](../../../non-functional/NFR-021-distann-distribution-invariant.md)
  (storage-class conformance envelope).
