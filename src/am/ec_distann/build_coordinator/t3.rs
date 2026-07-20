//! T3 durable publish-decision SQL endpoint.

use pgrx::datum::Uuid;
use pgrx::{pg_extern, PgRelation};

use super::*;

#[pg_extern(volatile, strict)]
fn ec_distann_decide_epoch_publish(index_regclass: PgRelation, build_id: Uuid) -> Vec<u8> {
    decide_epoch_publish(index_regclass, build_id)
}

pub(super) fn decide_epoch_publish(index_regclass: PgRelation, build_id: Uuid) -> Vec<u8> {
    (|| -> Result<Vec<u8>, String> {
        super::super::lifecycle_guard::require_read_committed("ec_distann_decide_epoch_publish")?;
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        let index_oid = index_regclass.oid();
        drop(index_regclass);
        let (preflight_source_oid, preflight_uuid) = preflight_build_lock_identity(
            index_oid,
            "ec_distann_decide_epoch_publish preflight",
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
            "ec_distann_decide_epoch_publish",
        )?;
        control.retain_lock_until_transaction_end();
        if logical_index_uuid != preflight_uuid
            || index_heap_relation_oid_handle(handle) != preflight_source_oid
        {
            return Err(
                "EC_BUILD_ID_CONFLICT: control identity changed while acquiring source lock"
                    .to_owned(),
            );
        }
        let _registry_revision = lock_registry_revision(index_oid, logical_index_uuid)?;
        source_lock.retain();

        let candidate = load_build_candidate(index_oid, logical_index_uuid, build_id)?
            .ok_or_else(|| "EC_PUBLISH_DIGEST: no build candidate for this build id".to_owned())?;
        let candidate_digest = candidate.digest()?;
        let build_spec = super::super::generation_descriptor::DistannBuildSpec::decode(&candidate.build_spec)?;
        let epoch = i64::try_from(build_spec.epoch)
            .map_err(|_| "EC_PUBLISH_DIGEST: candidate epoch exceeds bigint".to_owned())?;

        let decision_table = extension_relation_name("ec_distann_publish_decision")?;
        // Exact replay: an existing decision returns the same manifest digest.
        let existing_decision = Spi::connect(
            |client| -> Result<Option<(Vec<u8>, PublishDecisionState)>, String> {
            client
                .select(
                    &format!(
                        "SELECT manifest_digest, decision_state FROM {decision_table}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| {
                    format!("EC_PUBLISH_DIGEST: decision replay lookup failed: {error}")
                })?
                .map(|row| Ok((
                    row["manifest_digest"]
                        .value::<Vec<u8>>()
                        .map_err(|_| "EC_PUBLISH_DIGEST: decision digest decode failed".to_owned())?
                        .ok_or_else(|| "EC_PUBLISH_DIGEST: decision digest is NULL".to_owned())?,
                    PublishDecisionState::parse(
                        &row["decision_state"]
                            .value::<String>()
                            .map_err(|error| {
                                format!("EC_EPOCH_STATE: decision state decode failed: {error}")
                            })?
                            .ok_or_else(|| {
                                "EC_EPOCH_STATE: decision state is NULL".to_owned()
                            })?,
                    )?,
                )))
                .next()
                .transpose()
            },
        )?;
        if let Some((existing_decision, decision_state)) = existing_decision {
            if existing_decision != candidate.manifest_digest.to_vec() {
                return Err(
                    "EC_PUBLISH_DIGEST: an existing decision for this build id has a different manifest"
                        .to_owned(),
                );
            }
            if decision_state == PublishDecisionState::Cancelled {
                return Err(
                    "EC_PUBLISH_CANCEL: cancelled publish decision cannot be decided again"
                        .to_owned(),
                );
            }
            if decision_state == PublishDecisionState::Applied {
                source_lock.release_after_commit();
            }
            return Ok(existing_decision);
        }

        // T3 requires the registration to be Ready and CASes it to 'Decided'
        // atomically with the decision insert, so a concurrent abort cannot
        // destroy the build under a durable Pending decision. Lock the row now.
        let registration = extension_relation_name("ec_distann_build_registration")?;
        let registration_state = Spi::connect_mut(
            |client| -> Result<Option<RegistrationState>, String> {
            client
                .update(
                    &format!(
                        "SELECT state FROM {registration}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid
                          FOR UPDATE"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| {
                    format!("EC_EPOCH_STATE: registration lookup failed: {error}")
                })?
                .map(|row| {
                    let state = row["state"]
                        .value::<String>()
                        .map_err(|error| {
                            format!("EC_EPOCH_STATE: registration state decode failed: {error}")
                        })?
                        .ok_or_else(|| "EC_EPOCH_STATE: registration state is NULL".to_owned())?;
                    RegistrationState::parse(&state)
                })
                .next()
                .transpose()
            },
        )?;
        match registration_state {
            Some(RegistrationState::Ready) => {}
            Some(other) => {
                return Err(format!(
                    "EC_EPOCH_STATE: cannot decide a build whose registration is {other}"
                ));
            }
            None => return Err("EC_EPOCH_STATE: build registration is absent".to_owned()),
        }

        // Active pointer, taken under the control lock. Require parent==active.
        let active_table = extension_relation_name("ec_distann_active_epoch")?;
        let active = Spi::connect_mut(
            |client| -> Result<Option<([u8; 16], i64, Vec<u8>, [u8; 32])>, String> {
                client
                    .update(
                        &format!(
                            "SELECT build_id, epoch, epoch_fingerprint, manifest_digest
                               FROM {active_table}
                              WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                              FOR UPDATE"
                        ),
                        None,
                        &[index_oid.into(), logical_index_uuid.into()],
                    )
                    .map_err(|error| {
                        format!("EC_PUBLISH_DIGEST: active pointer lookup failed: {error}")
                    })?
                    .map(|row| {
                        let build_id = row["build_id"]
                            .value::<Uuid>()
                            .map_err(|_| "EC_PUBLISH_DIGEST: active build id decode failed".to_owned())?
                            .ok_or_else(|| "EC_PUBLISH_DIGEST: active build id is NULL".to_owned())?;
                        let epoch = row["epoch"]
                            .value::<i64>()
                            .map_err(|_| "EC_PUBLISH_DIGEST: active epoch decode failed".to_owned())?
                            .ok_or_else(|| "EC_PUBLISH_DIGEST: active epoch is NULL".to_owned())?;
                        let fingerprint = row["epoch_fingerprint"]
                            .value::<Vec<u8>>()
                            .map_err(|_| "EC_PUBLISH_DIGEST: active fingerprint decode failed".to_owned())?
                            .ok_or_else(|| "EC_PUBLISH_DIGEST: active fingerprint is NULL".to_owned())?;
                        let manifest_digest: [u8; 32] = row["manifest_digest"]
                            .value::<Vec<u8>>()
                            .map_err(|_| "EC_PUBLISH_DIGEST: active manifest decode failed".to_owned())?
                            .ok_or_else(|| "EC_PUBLISH_DIGEST: active manifest is NULL".to_owned())?
                            .try_into()
                            .map_err(|_| "EC_PUBLISH_DIGEST: active manifest is not 32 bytes".to_owned())?;
                        Ok((*build_id.as_bytes(), epoch, fingerprint, manifest_digest))
                    })
                    .next()
                    .transpose()
            },
        )?;

        let parent_matches = match (&active, build_spec.parent_fingerprint.is_empty()) {
            (None, true) => true,
            (Some((_, _, fingerprint, _)), false) => *fingerprint == build_spec.parent_fingerprint,
            _ => false,
        };
        if !parent_matches {
            return Err(
                "EC_PUBLISH_DIGEST: candidate parent fingerprint differs from the active pointer"
                    .to_owned(),
            );
        }

        // Canonical successor activation.
        let successor = DistannPublishedEpochIdentity {
            build_id: *build_id.as_bytes(),
            epoch: build_spec.epoch,
            fingerprint: candidate.epoch_fingerprint,
            manifest_digest: candidate.manifest_digest,
        };
        let predecessor = active.as_ref().map(|(pred_build_id, pred_epoch, pred_fp, pred_md)| {
            let fingerprint: [u8; 34] = pred_fp.as_slice().try_into().map_err(|_| {
                "EC_PUBLISH_DIGEST: active fingerprint is not 34 bytes".to_owned()
            })?;
            Ok::<DistannPublishedEpochIdentity, String>(DistannPublishedEpochIdentity {
                build_id: *pred_build_id,
                epoch: u64::try_from(*pred_epoch)
                    .map_err(|_| "EC_PUBLISH_DIGEST: active epoch is negative".to_owned())?,
                fingerprint,
                manifest_digest: *pred_md,
            })
        });
        let predecessor = predecessor.transpose()?;
        let activation = DistannSuccessorActivationV1 {
            coordinator_logical_index_uuid: metadata.logical_index_uuid,
            predecessor,
            successor,
        };
        let activation_bytes = activation.encode()?;
        let activation_digest = activation.digest()?;

        // Persist the commit-only Pending decision. No participant call, no swap.
        let (pred_build_id, pred_epoch, pred_fp, pred_md) = match &active {
            Some((build_id, epoch, fingerprint, manifest_digest)) => (
                Some(Uuid::from_bytes(*build_id)),
                Some(*epoch),
                Some(fingerprint.clone()),
                Some(manifest_digest.to_vec()),
            ),
            None => (None, None, None, None),
        };
        Spi::connect_mut(|client| -> Result<(), String> {
            client
                .update(
                    &format!(
                        "INSERT INTO {decision_table} (
                             index_oid, logical_index_uuid, build_id, epoch,
                             epoch_fingerprint, manifest_digest, epoch_manifest,
                             registration_digest, candidate_digest,
                             predecessor_build_id, predecessor_epoch,
                             predecessor_epoch_fingerprint, predecessor_manifest_digest,
                             successor_activation, successor_activation_digest,
                             decision_state, committed_at
                         ) VALUES (
                             $1::oid, $2::uuid, $3::uuid, $4::bigint,
                             $5::bytea, $6::bytea, $7::bytea,
                             $8::bytea, $9::bytea,
                             $10::uuid, $11::bigint,
                             $12::bytea, $13::bytea,
                             $14::bytea, $15::bytea,
                             'Pending', clock_timestamp()
                         )"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        build_id.into(),
                        epoch.into(),
                        candidate.epoch_fingerprint.to_vec().into(),
                        candidate.manifest_digest.to_vec().into(),
                        candidate.epoch_manifest.clone().into(),
                        candidate.registration_digest.to_vec().into(),
                        candidate_digest.to_vec().into(),
                        pred_build_id.into(),
                        pred_epoch.into(),
                        pred_fp.into(),
                        pred_md.into(),
                        activation_bytes.into(),
                        activation_digest.to_vec().into(),
                    ],
                )
                .map_err(|error| format!("EC_PUBLISH_DIGEST: publish decision insert failed: {error}"))?;
            // Atomically transition the registration Ready -> Decided so the
            // durable decision and the registration state cannot diverge.
            let transitioned = client
                .update(
                    &format!(
                        "UPDATE {registration} SET state = 'Decided'
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid AND state = 'Ready'
                          RETURNING 1"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| {
                    format!("EC_EPOCH_STATE: registration Decided transition failed: {error}")
                })?
                .len();
            require_exact_transition(
                RegistrationState::Ready,
                RegistrationState::Decided,
                transitioned,
                "registration",
            )?;
            Ok(())
        })?;

        Ok(candidate.manifest_digest.to_vec())
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"))
}
