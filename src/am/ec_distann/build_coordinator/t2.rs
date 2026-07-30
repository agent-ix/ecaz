//! T2 physical generation construction SQL endpoint.

use pgrx::datum::Uuid;
use pgrx::{pg_extern, PgRelation};

use super::*;

#[pg_extern(volatile, strict)]
fn ec_distann_build_epoch(index_regclass: PgRelation, epoch: i64, build_id: Uuid) -> Vec<u8> {
    build_epoch(index_regclass, epoch, build_id, None)
}

#[pg_extern(volatile, strict)]
fn ec_distann_build_epoch_with_training(
    index_regclass: PgRelation,
    epoch: i64,
    build_id: Uuid,
    training_relation: PgRelation,
) -> Vec<u8> {
    let training_relation_oid = training_relation.oid();
    let result = build_epoch(index_regclass, epoch, build_id, Some(training_relation_oid));
    drop(training_relation);
    result
}

fn load_training_query_set(
    relation_oid: pg_sys::Oid,
    dimensions: usize,
) -> Result<super::super::head_sample::DistannTrainingQuerySet, String> {
    let relation = super::super::handoff::qualified_relation_name(relation_oid)?;
    let rows = Spi::connect(|client| {
        let valid_shape = client
            .select(
                "SELECT count(*) = 2
                        AND count(*) FILTER (
                            WHERE attname = 'training_ordinal'
                              AND atttypid = 'bigint'::regtype) = 1
                        AND count(*) FILTER (
                            WHERE attname = 'vector'
                              AND atttypid = 'real[]'::regtype) = 1 AS valid
                   FROM pg_catalog.pg_attribute
                  WHERE attrelid = $1::oid AND attnum > 0 AND NOT attisdropped",
                None,
                &[relation_oid.into()],
            )
            .map_err(|error| {
                format!("EC_HEAD_TRAINING: training relation shape lookup failed: {error}")
            })?
            .map(|row| {
                row["valid"]
                    .value::<bool>()
                    .map_err(|error| {
                        format!("EC_HEAD_TRAINING: training relation shape decode failed: {error}")
                    })
                    .map(|value| value.unwrap_or(false))
            })
            .next()
            .transpose()?
            .unwrap_or(false);
        if !valid_shape {
            return Err(
                "EC_HEAD_TRAINING: relation must contain exactly training_ordinal bigint and vector real[]"
                    .to_owned(),
            );
        }
        client
            .select(
                &format!(
                    "SELECT training_ordinal, vector FROM {relation} ORDER BY training_ordinal"
                ),
                None,
                &[],
            )
            .map_err(|error| format!("EC_HEAD_TRAINING: training relation read failed: {error}"))?
            .map(|row| {
                let ordinal = row["training_ordinal"]
                    .value::<i64>()
                    .map_err(|error| {
                        format!("EC_HEAD_TRAINING: training ordinal decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_HEAD_TRAINING: training ordinal is NULL".to_owned())?;
                let vector = row["vector"]
                    .value::<Vec<f32>>()
                    .map_err(|error| {
                        format!("EC_HEAD_TRAINING: training vector decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_HEAD_TRAINING: training vector is NULL".to_owned())?;
                Ok((ordinal, vector))
            })
            .collect::<Result<Vec<_>, String>>()
    })?;
    super::super::head_sample::DistannTrainingQuerySet::from_ordered_rows(rows, dimensions)
}

fn capture_source_snapshot() -> Result<DistannSourceSnapshot, String> {
    let snapshot_ptr = unsafe { pg_sys::GetActiveSnapshot() };
    if snapshot_ptr.is_null() {
        return Err("EC_SOURCE_SNAPSHOT: build_epoch has no active PostgreSQL snapshot".to_owned());
    }
    let snapshot = unsafe { &*snapshot_ptr };
    let next_full = unsafe { pg_sys::ReadNextFullTransactionId() }.value;
    let next_lo = next_full as u32;
    let next_epoch = (next_full >> 32) as u32;
    // A read snapshot's ids all precede nextXid, so place each in the epoch that
    // keeps its full id at or below nextFullXid.
    let to_full = |xid: pg_sys::TransactionId| -> u64 {
        let xid = xid.into_inner();
        let epoch = if xid > next_lo {
            next_epoch.wrapping_sub(1)
        } else {
            next_epoch
        };
        (u64::from(epoch) << 32) | u64::from(xid)
    };
    let read_full_array = |base: *mut pg_sys::TransactionId, count: usize| -> Vec<u64> {
        let mut ids = Vec::with_capacity(count);
        if !base.is_null() {
            for offset in 0..count {
                ids.push(to_full(unsafe { *base.add(offset) }));
            }
        }
        // The snapshot arrays are unordered sets; canonical encoding requires a
        // strictly-ascending order and the members are distinct in-progress ids.
        ids.sort_unstable();
        ids
    };

    let xcnt = usize::try_from(snapshot.xcnt)
        .map_err(|_| "EC_SOURCE_SNAPSHOT: in-progress xid count is invalid".to_owned())?;
    let xip = read_full_array(snapshot.xip, xcnt);
    let subxip = if snapshot.suboverflowed {
        Vec::new()
    } else {
        let subxcnt = usize::try_from(snapshot.subxcnt)
            .map_err(|_| "EC_SOURCE_SNAPSHOT: subtransaction xid count is invalid".to_owned())?;
        read_full_array(snapshot.subxip, subxcnt)
    };

    let database_name = {
        let name_ptr = unsafe { pg_sys::get_database_name(pg_sys::MyDatabaseId) };
        if name_ptr.is_null() {
            return Err("EC_SOURCE_SNAPSHOT: current database name is unavailable".to_owned());
        }
        let name = unsafe { std::ffi::CStr::from_ptr(name_ptr) }
            .to_str()
            .map_err(|_| "EC_SOURCE_SNAPSHOT: database name is not UTF-8".to_owned())?
            .to_owned();
        unsafe { pg_sys::pfree(name_ptr.cast()) };
        name
    };

    let captured = DistannSourceSnapshot {
        system_identifier: unsafe { pg_sys::GetSystemIdentifier() },
        database_name,
        xmin_full: to_full(snapshot.xmin),
        xmax_full: to_full(snapshot.xmax),
        curcid: snapshot.curcid,
        xip,
        subxip,
        suboverflowed: snapshot.suboverflowed,
        taken_during_recovery: snapshot.takenDuringRecovery,
    };
    captured.validate()?;
    Ok(captured)
}

/// Build a distributed epoch to Ready in one coordinator transaction and return
/// the 32-byte candidate manifest digest (FR-078:275-323). Requires the durable
/// registration from `ec_distann_begin_epoch_build`. Captures one frozen source
/// snapshot, builds the physical graph workspace, makes the counting/digest pass
/// binding per-owner and global identities, drives the single local participant
/// begin/stage/seal, then atomically persists the immutable build candidate and
/// transitions the registration to Ready. This slice supports a single local
/// participant; multi-owner remote transport is a later slice.
pub(super) fn build_epoch(
    index_regclass: PgRelation,
    epoch: i64,
    build_id: Uuid,
    training_relation_oid: Option<pg_sys::Oid>,
) -> Vec<u8> {
    (|| -> Result<Vec<u8>, String> {
        super::super::lifecycle_guard::require_read_committed("ec_distann_build_epoch")?;
        super::super::build_gate::require_shared_preload()?;
        let epoch_u64 = u64::try_from(epoch)
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| "EC_BUILD_STATE: epoch must be positive".to_owned())?;
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        let index_oid = index_regclass.oid();
        drop(index_regclass);

        // Match begin-build's source -> control order. A replacement backend
        // may reacquire these locks to inspect or abort durable state, but it
        // must not capture a new source snapshot under the old build id.
        let (preflight_source_oid, preflight_uuid) = preflight_build_lock_identity(
            index_oid,
            "ec_distann_build_epoch preflight",
        )?;
        let mut source_lock = SourceSessionLockGuard::acquire(
            preflight_source_oid,
            index_oid,
            preflight_uuid,
            build_id,
        )?;

        let (mut control, handle, metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_build_epoch",
        )?;
        control.retain_lock_until_transaction_end();
        let source_relation_oid = index_heap_relation_oid_handle(handle);
        if logical_index_uuid != preflight_uuid || source_relation_oid != preflight_source_oid {
            return Err(
                "EC_BUILD_ID_CONFLICT: control identity changed while acquiring source lock"
                    .to_owned(),
            );
        }
        let _registry_revision = lock_registry_revision(index_oid, logical_index_uuid)?;
        let row_schema = resolve_relation_schema(source_relation_oid)?.descriptor;
        let row_schema_fingerprint = row_schema.fingerprint()?;
        let compatibility_digest = control_compatibility_digest(handle, &metadata)?;

        // build_epoch consumes the durable registration written by begin.
        let (registration_digest, requires_source_lock) = replay_registration(
            index_oid,
            logical_index_uuid,
            build_id,
            epoch_u64,
            row_schema_fingerprint,
            compatibility_digest,
            source_relation_oid,
        )?
        .ok_or_else(|| {
            "EC_BUILD_STATE: build_epoch requires a durable registration from begin_epoch_build"
                .to_owned()
        })?;

        // Exact replay is idempotent: a build id whose immutable candidate
        // already exists returns the stored candidate digest without rebuilding
        // (FR-078:312-314, 372-373). Divergent inputs under the same build id are
        // already rejected by replay_registration above.
        let candidate_table = extension_relation_name("ec_distann_build_candidate")?;
        if let Some(existing_candidate) =
            load_build_candidate(index_oid, logical_index_uuid, build_id)?
        {
            let manifest = DistannEpochManifestV2::decode(&existing_candidate.epoch_manifest)?;
            match training_relation_oid {
                Some(training_relation_oid) => {
                    let descriptor = DistannGenerationDescriptor::decode(
                        &existing_candidate.generation_descriptor,
                    )?;
                    let training = load_training_query_set(
                        training_relation_oid,
                        usize::from(descriptor.dimensions),
                    )?;
                    if manifest.build_options.options.head_policy
                        != DistannHeadPolicy::TrainingLandmarksExact
                        || manifest.build_options.options.training_query_count
                            != training.queries.len() as u32
                        || manifest.build_options.options.training_query_digest != training.digest
                    {
                        return Err(
                            "EC_BUILD_ID_CONFLICT: training head policy/input differs from immutable candidate"
                                .to_owned(),
                        );
                    }
                }
                None => {
                    if manifest.build_options.options.head_policy
                        != DistannHeadPolicy::CurrentSampleGraph
                    {
                        return Err(
                            "EC_BUILD_ID_CONFLICT: current-head replay targets a trained-head candidate"
                                .to_owned(),
                        );
                    }
                }
            }
            if requires_source_lock {
                source_lock.retain();
            } else {
                source_lock.release_after_commit();
            }
            return Ok(existing_candidate.digest()?.to_vec());
        }
        if source_lock.reacquired_after_owner_loss() {
            return Err(
                "EC_BUILD_STATE: replacement backend cannot recapture the source snapshot under an existing build id; abort and begin a new build"
                    .to_owned(),
            );
        }

        // One frozen source MVCC snapshot for the whole build, captured only
        // after the original owner proves its retained build-lock ownership.
        let source_snapshot = capture_source_snapshot()?;
        let source_snapshot_digest = source_snapshot.digest()?;

        // Reacquire the frozen roster (revision-locked so it matches registration).
        let participants = desired_participants(index_oid, logical_index_uuid)?;
        let roster = public_roster(&participants)?;
        let roster_digest_bytes = roster_digest(&roster)?;
        if participants.iter().filter(|participant| participant.is_local).count() > 1 {
            return Err(
                "EC_BUILD_STATE: build_epoch permits at most one local participant".to_owned(),
            );
        }

        // Capture source rows and build the physical graph workspace.
        let index_relation = crate::storage::relation_guard::IndexRelationGuard::open(
            index_oid,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            "ec_distann_build_epoch",
        );
        let heap = crate::storage::relation_guard::HeapRelationGuard::try_access_share(
            index_relation.heap_relation_oid(),
        )
        .ok_or_else(|| "EC_BUILD_INCOMPLETE: source heap could not open".to_owned())?;
        let index_info = crate::am::common::index_info::IndexInfoGuard::build(
            index_relation.as_ptr(),
            "ec_distann_build_epoch",
        );
        let capture = unsafe {
            super::super::capture_physical_source_rows(
                heap.as_ptr(),
                index_relation.as_ptr(),
                index_info.as_ptr(),
            )
        }?;
        let mut workspace =
            super::super::build_physical_graph_workspace(index_relation.as_ptr(), capture)?;
        let training = training_relation_oid
            .map(|relation_oid| {
                load_training_query_set(
                    relation_oid,
                    usize::from(workspace.codec_artifact().dimensions()),
                )
            })
            .transpose()?;

        // Counting/digest pass before the first participant begin.
        let expectations =
            workspace.owner_expectations(&roster, super::super::DISTANN_PLACEMENT_HASH_VERSION)?;
        let (global_count, global_graph_digest, global_row_tier_digest) =
            workspace.global_digests()?;
        let options = super::super::options::relation_options(index_relation.as_ptr());
        let to_u16 = |value: i32, field: &str| -> Result<u16, String> {
            u16::try_from(value).map_err(|_| format!("EC_BUILD_STATE: {field} out of range"))
        };
        let to_u32 = |value: i32, field: &str| -> Result<u32, String> {
            u32::try_from(value).map_err(|_| format!("EC_BUILD_STATE: {field} out of range"))
        };
        let build_options = DistannBuildOptions {
            build_list_size: to_u16(options.build_list_size, "build_list_size")?,
            alpha: options.alpha,
            seed: metadata.seed,
            closure_epsilon: options.closure_epsilon,
            head_index_cap: to_u32(options.head_index_cap, "head_index_cap")?,
            build_shards: to_u32(options.build_shards, "build_shards")?,
            head_policy: if training.is_some() {
                DistannHeadPolicy::TrainingLandmarksExact
            } else {
                DistannHeadPolicy::CurrentSampleGraph
            },
            training_query_count: training
                .as_ref()
                .map(|set| set.queries.len() as u32)
                .unwrap_or(0),
            training_query_digest: training.as_ref().map(|set| set.digest).unwrap_or([0; 32]),
        };
        build_options.encode()?;
        let head_sample = workspace.head_sample(
            build_options.head_index_cap as usize,
            training.as_ref(),
        )?;
        let head_sample_digest = head_sample.digest()?;
        let head_graph = super::super::head_sample::DistannPersistedHeadGraph::build(
            &head_sample,
            usize::from(metadata.graph_degree_r),
            usize::from(build_options.build_list_size),
            build_options.alpha,
            build_options.seed,
        )?;

        let codec_artifact = workspace.codec_artifact().clone();
        let dimensions = to_u16(i32::from(codec_artifact.dimensions()), "dimensions")?;
        let shape = workspace.shape();
        let code_stride = to_u32(
            i32::try_from(shape.code_stride)
                .map_err(|_| "EC_BUILD_STATE: code stride out of range".to_owned())?,
            "code_stride",
        )?;

        let descriptor = DistannGenerationDescriptor {
            coordinator_logical_index_uuid: metadata.logical_index_uuid,
            index_format_version: super::super::DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
            graph_record_version: super::super::DISTANN_GRAPH_RECORD_VERSION,
            handoff_wire_version: super::super::DISTANN_HANDOFF_WIRE_VERSION,
            dimensions,
            graph_degree: metadata.graph_degree_r,
            placement_hash_version: super::super::DISTANN_PLACEMENT_HASH_VERSION,
            roster: roster.clone(),
            neighbor_codec_kind: metadata.neighbor_codec_kind,
            codec_artifact,
            row_schema,
        };
        let descriptor_bytes = descriptor.encode()?;
        let descriptor_digest = descriptor.digest()?;

        // A successor build binds the current active-epoch fingerprint as its
        // parent (empty for the first epoch); the decision re-checks it against
        // the pointer taken under lock.
        let active_table = extension_relation_name("ec_distann_active_epoch")?;
        let parent_fingerprint = Spi::connect(|client| -> Result<Vec<u8>, String> {
            Ok(client
                .select(
                    &format!(
                        "SELECT epoch_fingerprint FROM {active_table}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into()],
                )
                .map_err(|error| {
                    format!("EC_BUILD_INCOMPLETE: active pointer lookup failed: {error}")
                })?
                .map(|row| {
                    row["epoch_fingerprint"]
                        .value::<Vec<u8>>()
                        .map_err(|_| "EC_BUILD_INCOMPLETE: active fingerprint decode failed".to_owned())?
                        .ok_or_else(|| "EC_BUILD_INCOMPLETE: active fingerprint is NULL".to_owned())
                })
                .next()
                .transpose()?
                .unwrap_or_default())
        })?;

        let build_spec = DistannBuildSpec {
            epoch: epoch_u64,
            build_id: *build_id.as_bytes(),
            parent_fingerprint: parent_fingerprint.clone(),
            source_snapshot_digest,
            generation_descriptor_digest: descriptor_digest,
            build_options: build_options.clone(),
            expected_global_count: global_count,
            expected_global_graph_digest: global_graph_digest,
            expected_global_row_tier_digest: global_row_tier_digest,
            head_sample_digest,
            owner_expectations: expectations.clone(),
        };
        let build_spec_digest = build_spec.digest()?;

        // Resolve private transport only after all public build identities are
        // frozen. Secret values never enter the descriptor, candidate, or log.
        let conninfos = participants
            .iter()
            .map(|participant| {
                if participant.is_local {
                    Ok(None)
                } else {
                    super::super::node_registry::resolve_conninfo_secret(
                        &participant.conninfo_secret_name,
                    )
                    .map(Some)
                }
            })
            .collect::<Result<Vec<_>, String>>()?;
        let build_id_text = build_id.to_string();
        let begin_fn = extension_relation_name("ec_distann_begin_epoch_handoff")?;
        for (owner, participant) in participants.iter().enumerate() {
            let expectation = expectations.get(owner).ok_or_else(|| {
                "EC_BUILD_INCOMPLETE: owner expectation count disagrees with roster".to_owned()
            })?;
            if expectation.node_id != participant.node_id {
                return Err("EC_BUILD_INCOMPLETE: owner expectation order mismatch".to_owned());
            }
            let owner_count = i64::try_from(expectation.expected_count)
                .map_err(|_| "EC_BUILD_STATE: owner count exceeds bigint".to_owned())?;
            if participant.is_local {
                Spi::connect_mut(|client| -> Result<(), String> {
                    client
                        .update(
                            &format!(
                                "SELECT state FROM {begin_fn}(
                                     $1::oid::regclass, $2::bigint, $3::uuid, $4::bytea, $5::bytea,
                                     $6::bytea, $7::bytea, $8::bigint, $9::bytea
                                 )"
                            ),
                            None,
                            &[
                                index_oid.into(), epoch.into(), build_id.into(),
                                build_spec_digest.to_vec().into(), roster_digest_bytes.to_vec().into(),
                                descriptor_bytes.clone().into(), descriptor_digest.to_vec().into(),
                                owner_count.into(), expectation.expected_owner_digest.to_vec().into(),
                            ],
                        )
                        .map_err(|error| format!("EC_BUILD_INCOMPLETE: local begin failed: {error}"))?;
                    Ok(())
                })?;
            } else {
                super::super::remote_transport::remote_begin_epoch_handoff(
                    super::super::remote_transport::RemoteHandoffBegin {
                        conninfo: conninfos[owner].as_deref().expect("remote conninfo resolved"),
                        index_regclass: &participant.remote_index_regclass,
                        epoch,
                        build_id: &build_id_text,
                        build_spec_digest: &build_spec_digest,
                        roster_digest: &roster_digest_bytes,
                        descriptor: &descriptor_bytes,
                        descriptor_digest: &descriptor_digest,
                        expected_count: owner_count,
                        expected_owner_digest: &expectation.expected_owner_digest,
                    },
                )?;
            }
        }

        let route_identity = DistannHandoffRouteIdentity {
            epoch: epoch_u64,
            build_id: *build_id.as_bytes(),
            build_spec_digest,
            row_schema_fingerprint,
            index_format_version: super::super::DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
            neighbor_codec_kind: metadata.neighbor_codec_kind,
            placement_hash_version: super::super::DISTANN_PLACEMENT_HASH_VERSION,
        };
        let stage_fn = extension_relation_name("ec_distann_stage_epoch_batch")?;
        workspace.route(route_identity, participants.len(), &mut |owner, sequence, digest, encoded| {
            let sequence = i64::try_from(sequence)
                .map_err(|_| "EC_BUILD_STATE: batch sequence exceeds bigint".to_owned())?;
            let participant = &participants[owner];
            if !participant.is_local {
                return super::super::remote_transport::remote_stage_epoch_batch(
                    conninfos[owner].as_deref().expect("remote conninfo resolved"),
                    &participant.remote_index_regclass,
                    &build_id_text,
                    sequence,
                    digest,
                    encoded,
                );
            }
            Spi::connect_mut(|client| -> Result<DistannStageAck, String> {
                let row = client
                    .update(
                        &format!(
                            "SELECT accepted_record_count, cumulative_record_count,
                                    cumulative_owner_digest
                               FROM {stage_fn}(
                                   $1::oid::regclass, $2::uuid, $3::bigint, $4::bytea, $5::bytea
                               )"
                        ),
                        None,
                        &[
                            index_oid.into(),
                            build_id.into(),
                            sequence.into(),
                            digest.to_vec().into(),
                            encoded.to_vec().into(),
                        ],
                    )
                    .map_err(|error| {
                        format!("EC_BUILD_INCOMPLETE: stage batch dispatch failed: {error}")
                    })?
                    .next()
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: stage batch returned no row".to_owned())?;
                let accepted = row["accepted_record_count"]
                    .value::<i64>()
                    .map_err(|_| "EC_BUILD_INCOMPLETE: accepted count decode failed".to_owned())?
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: accepted count is NULL".to_owned())?;
                let cumulative = row["cumulative_record_count"]
                    .value::<i64>()
                    .map_err(|_| "EC_BUILD_INCOMPLETE: cumulative count decode failed".to_owned())?
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: cumulative count is NULL".to_owned())?;
                let cumulative_owner_digest = row["cumulative_owner_digest"]
                    .value::<Vec<u8>>()
                    .map_err(|_| "EC_BUILD_INCOMPLETE: cumulative digest decode failed".to_owned())?
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: cumulative digest is NULL".to_owned())?;
                Ok(DistannStageAck {
                    accepted_record_count: u64::try_from(accepted).map_err(|_| {
                        "EC_BUILD_INCOMPLETE: accepted count is negative".to_owned()
                    })?,
                    cumulative_record_count: u64::try_from(cumulative).map_err(|_| {
                        "EC_BUILD_INCOMPLETE: cumulative count is negative".to_owned()
                    })?,
                    cumulative_owner_digest: cumulative_owner_digest.try_into().map_err(|_| {
                        "EC_BUILD_INCOMPLETE: cumulative digest is not 32 bytes".to_owned()
                    })?,
                })
            })
        })?;

        let seal_fn = extension_relation_name("ec_distann_seal_epoch_handoff")?;
        let mut receipts = Vec::with_capacity(participants.len());
        for (owner, participant) in participants.iter().enumerate() {
            let expectation = &expectations[owner];
            let owner_count = i64::try_from(expectation.expected_count)
                .map_err(|_| "EC_BUILD_STATE: owner count exceeds bigint".to_owned())?;
            let receipt_bytes = if participant.is_local {
                Spi::connect_mut(|client| -> Result<Vec<u8>, String> {
                    client
                        .update(
                            &format!(
                                "SELECT {seal_fn}(
                                     $1::oid::regclass, $2::uuid, $3::bigint, $4::bytea
                                 ) AS ready_receipt"
                            ),
                            None,
                            &[
                                index_oid.into(), build_id.into(), owner_count.into(),
                                expectation.expected_owner_digest.to_vec().into(),
                            ],
                        )
                        .map_err(|error| format!("EC_BUILD_INCOMPLETE: local seal failed: {error}"))?
                        .map(|row| {
                            row["ready_receipt"]
                                .value::<Vec<u8>>()
                                .map_err(|_| "EC_BUILD_INCOMPLETE: ready receipt decode failed".to_owned())?
                                .ok_or_else(|| "EC_BUILD_INCOMPLETE: ready receipt is NULL".to_owned())
                        })
                        .next()
                        .transpose()?
                        .ok_or_else(|| "EC_BUILD_INCOMPLETE: seal returned no receipt".to_owned())
                })?
            } else {
                super::super::remote_transport::remote_seal_epoch_handoff(
                    conninfos[owner].as_deref().expect("remote conninfo resolved"),
                    &participant.remote_index_regclass,
                    &build_id_text,
                    owner_count,
                    &expectation.expected_owner_digest,
                )?
            };
            receipts.push(super::super::manifest_v2::DistannReadyReceipt::decode(&receipt_bytes)?);
        }

        let codec_parameters = DistannManifestCodecParameters {
            codec_kind: metadata.neighbor_codec_kind,
            dimensions,
            code_stride,
            seed: metadata.seed,
            transform_dim: 0,
            group_count: 0,
            group_size: 0,
            centroids_per_group: 0,
        };
        let manifest = DistannEpochManifestV2 {
            epoch: epoch_u64,
            build_id: *build_id.as_bytes(),
            parent_fingerprint: parent_fingerprint.clone(),
            source_snapshot_digest,
            build_spec_digest,
            generation_descriptor_digest: descriptor_digest,
            placement_hash_version: super::super::DISTANN_PLACEMENT_HASH_VERSION,
            roster: roster.clone(),
            index_format_version: super::super::DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
            graph_record_version: super::super::DISTANN_GRAPH_RECORD_VERSION,
            handoff_wire_version: super::super::DISTANN_HANDOFF_WIRE_VERSION,
            codec_parameters,
            build_options: DistannManifestBuildOptions {
                graph_degree: metadata.graph_degree_r,
                options: build_options,
            },
            row_schema_fingerprint,
            head_sample_digest,
            global_record_count: global_count,
            global_graph_digest,
            global_row_tier_digest,
            participant_receipts: receipts,
        };

        let candidate = DistannBuildCandidateV1::from_components(
            registration_digest,
            &build_spec,
            &descriptor,
            &source_snapshot,
            &manifest,
        )?;
        let candidate_digest = candidate.digest()?;

        // Atomically persist the immutable candidate and mark the registration
        // Ready in this coordinator transaction.
        let registration = extension_relation_name("ec_distann_build_registration")?;
        Spi::connect_mut(|client| -> Result<(), String> {
            client
                .update(
                    &format!(
                        "INSERT INTO {candidate_table} (
                             index_oid, logical_index_uuid, build_id, epoch,
                             registration_digest, build_spec, build_spec_digest,
                             generation_descriptor, generation_descriptor_digest,
                             source_snapshot, source_snapshot_digest,
                             ready_receipt_set, ready_receipt_set_digest,
                             epoch_manifest, manifest_digest, epoch_fingerprint,
                             candidate_digest
                         ) VALUES (
                             $1::oid, $2::uuid, $3::uuid, $4::bigint,
                             $5::bytea, $6::bytea, $7::bytea,
                             $8::bytea, $9::bytea,
                             $10::bytea, $11::bytea,
                             $12::bytea, $13::bytea,
                             $14::bytea, $15::bytea, $16::bytea,
                             $17::bytea
                         )"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        build_id.into(),
                        epoch.into(),
                        candidate.registration_digest.to_vec().into(),
                        candidate.build_spec.clone().into(),
                        candidate.build_spec_digest.to_vec().into(),
                        candidate.generation_descriptor.clone().into(),
                        candidate.generation_descriptor_digest.to_vec().into(),
                        candidate.source_snapshot.clone().into(),
                        candidate.source_snapshot_digest.to_vec().into(),
                        candidate.ready_receipt_set.clone().into(),
                        candidate.ready_receipt_set_digest.to_vec().into(),
                        candidate.epoch_manifest.clone().into(),
                        candidate.manifest_digest.to_vec().into(),
                        candidate.epoch_fingerprint.to_vec().into(),
                        candidate_digest.to_vec().into(),
                    ],
                )
                .map_err(|error| {
                    format!("EC_BUILD_INCOMPLETE: build candidate insert failed: {error}")
                })?;
            super::super::head_sample::persist_head_sample(
                client,
                index_oid,
                logical_index_uuid,
                build_id,
                &head_sample,
                &head_graph,
                build_options,
                // NFR-021 clause 3 (Task 210 P2a): with sharded head storage
                // the coordinator persists landmark ids only; the vectors stay
                // on the owners that already hold them.
                super::super::options::shard_head_storage(),
            )?;
            let transitioned = client
                .update(
                    &format!(
                        "UPDATE {registration} SET state = 'Ready'
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid AND state = 'Registered'
                          RETURNING 1"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| {
                    format!("EC_BUILD_INCOMPLETE: registration Ready transition failed: {error}")
                })?
                .len();
            require_exact_transition(
                RegistrationState::Registered,
                RegistrationState::Ready,
                transitioned,
                "registration",
            )?;
            Ok(())
        })?;

        Ok(candidate_digest.to_vec())
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"))
}

/// Load the immutable build-candidate row for a build id and reconstruct the
/// canonical `DistannBuildCandidateV1`, recomputing and verifying its full
/// digest chain (validate() re-checks every component digest and the
/// cross-component consistency) — FR-082:266-270.
pub(crate) fn load_build_candidate(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
) -> Result<Option<DistannBuildCandidateV1>, String> {
    let candidate_table = extension_relation_name("ec_distann_build_candidate")?;
    Spi::connect(
        |client| -> Result<Option<DistannBuildCandidateV1>, String> {
            client
                .select(
                    &format!(
                        "SELECT registration_digest, build_spec, build_spec_digest,
                            generation_descriptor, generation_descriptor_digest,
                            source_snapshot, source_snapshot_digest,
                            ready_receipt_set, ready_receipt_set_digest,
                            epoch_manifest, manifest_digest, epoch_fingerprint
                       FROM {candidate_table}
                      WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                        AND build_id = $3::uuid"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| {
                    format!("EC_PUBLISH_DIGEST: build candidate lookup failed: {error}")
                })?
                .map(|row| {
                    let bytea = |name: &str| -> Result<Vec<u8>, String> {
                        row[name]
                            .value::<Vec<u8>>()
                            .map_err(|_| {
                                format!("EC_PUBLISH_DIGEST: candidate {name} decode failed")
                            })?
                            .ok_or_else(|| format!("EC_PUBLISH_DIGEST: candidate {name} is NULL"))
                    };
                    let fixed32 = |name: &str| -> Result<[u8; 32], String> {
                        bytea(name)?.try_into().map_err(|_| {
                            format!("EC_PUBLISH_DIGEST: candidate {name} is not 32 bytes")
                        })
                    };
                    let candidate = DistannBuildCandidateV1 {
                        registration_digest: fixed32("registration_digest")?,
                        build_spec: bytea("build_spec")?,
                        build_spec_digest: fixed32("build_spec_digest")?,
                        generation_descriptor: bytea("generation_descriptor")?,
                        generation_descriptor_digest: fixed32("generation_descriptor_digest")?,
                        source_snapshot: bytea("source_snapshot")?,
                        source_snapshot_digest: fixed32("source_snapshot_digest")?,
                        ready_receipt_set: bytea("ready_receipt_set")?,
                        ready_receipt_set_digest: fixed32("ready_receipt_set_digest")?,
                        epoch_manifest: bytea("epoch_manifest")?,
                        manifest_digest: fixed32("manifest_digest")?,
                        epoch_fingerprint: bytea("epoch_fingerprint")?.try_into().map_err(
                            |_| {
                                "EC_PUBLISH_DIGEST: candidate fingerprint is not 34 bytes"
                                    .to_owned()
                            },
                        )?,
                    };
                    candidate.validate()?;
                    Ok(candidate)
                })
                .next()
                .transpose()
        },
    )
}
