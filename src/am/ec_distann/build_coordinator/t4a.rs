//! T4a publish recovery and active-pointer activation SQL endpoint.

use pgrx::datum::Uuid;
use pgrx::{pg_extern, PgRelation};

use super::*;

#[pg_extern(volatile, strict)]
fn ec_distann_recover_epoch_publish(index_regclass: PgRelation, build_id: Uuid) -> Vec<u8> {
    recover_epoch_publish(index_regclass, build_id)
}

pub(super) fn recover_epoch_publish(index_regclass: PgRelation, build_id: Uuid) -> Vec<u8> {
    (|| -> Result<Vec<u8>, String> {
        super::super::lifecycle_guard::require_read_committed("ec_distann_recover_epoch_publish")?;
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        let index_oid = index_regclass.oid();
        drop(index_regclass);
        let (preflight_source_oid, preflight_uuid) = preflight_build_lock_identity(
            index_oid,
            "ec_distann_recover_epoch_publish preflight",
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
            "ec_distann_recover_epoch_publish",
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

        // Every T4 transaction, including an idempotent replay, recomputes the
        // complete candidate digest chain before consuming decision bytes.
        let candidate = load_build_candidate(index_oid, logical_index_uuid, build_id)?
            .ok_or_else(|| "EC_PUBLISH_DIGEST: no build candidate for recovery".to_owned())?;
        let candidate_digest = candidate.digest()?;

        let decision_table = extension_relation_name("ec_distann_publish_decision")?;
        let decision = Spi::connect(
            |client| -> Result<Option<(i64, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Option<Uuid>, Vec<u8>, Vec<u8>, PublishDecisionState, u32)>, String> {
                client
                    .select(
                        &format!(
                            "SELECT epoch, epoch_fingerprint, manifest_digest, epoch_manifest,
                                    candidate_digest, predecessor_build_id,
                                    successor_activation, successor_activation_digest,
                                    decision_state, xmin::text AS decision_xmin
                               FROM {decision_table}
                              WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                                AND build_id = $3::uuid"
                        ),
                        None,
                        &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                    )
                    .map_err(|error| {
                        format!("EC_EPOCH_STATE: publish decision lookup failed: {error}")
                    })?
                    .map(|row| {
                        let epoch = row["epoch"]
                            .value::<i64>()
                            .map_err(|_| "EC_EPOCH_STATE: decision epoch decode failed".to_owned())?
                            .ok_or_else(|| "EC_EPOCH_STATE: decision epoch is NULL".to_owned())?;
                        let field = |name: &str| -> Result<Vec<u8>, String> {
                            row[name]
                                .value::<Vec<u8>>()
                                .map_err(|_| format!("EC_EPOCH_STATE: decision {name} decode failed"))?
                                .ok_or_else(|| format!("EC_EPOCH_STATE: decision {name} is NULL"))
                        };
                        let predecessor_build_id = row["predecessor_build_id"]
                            .value::<Uuid>()
                            .map_err(|_| "EC_EPOCH_STATE: predecessor build id decode failed".to_owned())?;
                        let decision_xmin = row["decision_xmin"]
                            .value::<String>()
                            .map_err(|_| "EC_TRANSACTION_BOUNDARY: decision xmin decode failed".to_owned())?
                            .ok_or_else(|| "EC_TRANSACTION_BOUNDARY: decision xmin is NULL".to_owned())?
                            .parse::<u32>()
                            .map_err(|_| "EC_TRANSACTION_BOUNDARY: decision xmin is invalid".to_owned())?;
                        Ok((
                            epoch,
                            field("epoch_fingerprint")?,
                            field("manifest_digest")?,
                            field("epoch_manifest")?,
                            field("candidate_digest")?,
                            predecessor_build_id,
                            field("successor_activation")?,
                            field("successor_activation_digest")?,
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
                            decision_xmin,
                        ))
                    })
                    .next()
                    .transpose()
            },
        )?
        .ok_or_else(|| {
            "EC_EPOCH_STATE: no durable publish decision for this build id".to_owned()
        })?;
        let (epoch, epoch_fingerprint, manifest_digest, epoch_manifest, decision_candidate_digest, predecessor_build_id, activation_bytes, activation_digest, decision_state, decision_xmin) =
            decision;
        if unsafe {
            pg_sys::TransactionIdIsCurrentTransactionId(pg_sys::TransactionId::from(decision_xmin))
        } {
            return Err(
                "EC_TRANSACTION_BOUNDARY: publish decision must commit before recovery begins"
                    .to_owned(),
            );
        }
        if decision_state == PublishDecisionState::Cancelled {
            return Err(
                "EC_PUBLISH_CANCEL: cancelled publish decision cannot be recovered".to_owned(),
            );
        }
        if decision_candidate_digest != candidate_digest
            || epoch_fingerprint != candidate.epoch_fingerprint
            || manifest_digest != candidate.manifest_digest
            || epoch_manifest != candidate.epoch_manifest
        {
            return Err(
                "EC_PUBLISH_DIGEST: publish decision differs from the validated build candidate"
                    .to_owned(),
            );
        }

        // Lock and revalidate the registration before any participant RPC.
        // A new T4a recovery consumes Pending + Decided; an exact replay after
        // the atomic pointer/decision/registration transaction consumes only
        // Activated|Applied + Published.  Any other pair is durable state skew,
        // not an idempotent replay (FR-082:243-244).
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
        match (decision_state, registration_state) {
            (PublishDecisionState::Pending, Some(RegistrationState::Decided))
            | (
                PublishDecisionState::Activated | PublishDecisionState::Applied,
                Some(RegistrationState::Published),
            ) => {}
            (_, Some(registration_state)) => {
                return Err(format!(
                    "EC_EPOCH_STATE: publish decision is {decision_state} but registration is {registration_state}"
                ));
            }
            (_, None) => {
                return Err("EC_EPOCH_STATE: build registration is absent during recovery".to_owned());
            }
        }

        // Read the active pointer under the control lock.
        let active_table = extension_relation_name("ec_distann_active_epoch")?;
        let active_build = Spi::connect_mut(|client| -> Result<Option<Uuid>, String> {
            client
                .update(
                    &format!(
                        "SELECT build_id FROM {active_table}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                          FOR UPDATE"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into()],
                )
                .map_err(|error| {
                    format!("EC_EPOCH_STATE: active pointer lookup failed: {error}")
                })?
                .map(|row| {
                    row["build_id"]
                        .value::<Uuid>()
                        .map_err(|_| "EC_EPOCH_STATE: active build id decode failed".to_owned())?
                        .ok_or_else(|| "EC_EPOCH_STATE: active build id is NULL".to_owned())
                })
                .next()
                .transpose()
        })?;
        match (&active_build, &predecessor_build_id) {
            // Idempotent replay: this build is already the active pointer.
            (Some(active), _) if active.as_bytes() == build_id.as_bytes() => {
                if predecessor_build_id.is_some()
                    && decision_state == PublishDecisionState::Activated
                {
                    recover_predecessor_retirement(
                        index_oid,
                        logical_index_uuid,
                        build_id,
                        &activation_bytes,
                        &activation_digest,
                    )?;
                }
                super::super::traversal_replica::retire_superseded_replicas(
                    index_oid,
                    logical_index_uuid,
                    build_id,
                )?;
                if metadata.is_distributed_control() {
                    super::super::physical_dml::refresh_source_mapping(index_oid)?;
                }
                source_lock.release_after_commit();
                return Ok(epoch_fingerprint);
            }
            // A successor may swap only when the recorded predecessor is active.
            (Some(active), Some(predecessor))
                if active.as_bytes() == predecessor.as_bytes() => {}
            (Some(_), _) => {
                return Err(
                    "EC_EPOCH_STATE: the active pointer names an unexpected build id".to_owned(),
                );
            }
            // First epoch: no active pointer and no predecessor.
            (None, None) => {}
            (None, Some(_)) => {
                return Err(
                    "EC_EPOCH_STATE: a successor decision requires its predecessor to be active"
                        .to_owned(),
                );
            }
        }

        // Publish every successor participant before moving the coordinator
        // pointer. Exact replay is safe after a partial remote acknowledgement.
        let publish_fn = extension_relation_name("ec_distann_publish_epoch")?;
        let binding = extension_relation_name("ec_distann_build_participant_binding")?;
        let successor_bindings = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT is_local, conninfo_secret_name, remote_index_regclass
                           FROM {binding}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid
                          ORDER BY roster_ordinal"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| format!("EC_EPOCH_STATE: successor binding lookup failed: {error}"))?
                .map(|row| {
                    let text = |field: &str| -> Result<String, String> {
                        row[field]
                            .value::<String>()
                            .map_err(|_| format!("EC_EPOCH_STATE: {field} decode failed"))?
                            .ok_or_else(|| format!("EC_EPOCH_STATE: {field} is NULL"))
                    };
                    Ok((
                        row["is_local"]
                            .value::<bool>()
                            .map_err(|_| "EC_EPOCH_STATE: successor locality decode failed".to_owned())?
                            .ok_or_else(|| "EC_EPOCH_STATE: successor locality is NULL".to_owned())?,
                        text("conninfo_secret_name")?,
                        text("remote_index_regclass")?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()
        })?;
        if successor_bindings.is_empty() {
            return Err("EC_EPOCH_STATE: publish decision has no successor bindings".to_owned());
        }
        let build_id_text = build_id.to_string();
        for (is_local, secret_name, remote_index_regclass) in successor_bindings {
            let acked_fingerprint = if is_local {
                Spi::connect_mut(|client| -> Result<Vec<u8>, String> {
                    client
                        .update(
                            &format!(
                                "SELECT {publish_fn}(
                                     $1::oid::regclass, $2::uuid, $3::bytea, $4::bytea
                                 ) AS fingerprint"
                            ),
                            None,
                            &[
                                index_oid.into(), build_id.into(), epoch_manifest.clone().into(),
                                manifest_digest.clone().into(),
                            ],
                        )
                        .map_err(|error| format!("EC_EPOCH_STATE: local publish failed: {error}"))?
                        .map(|row| {
                            row["fingerprint"]
                                .value::<Vec<u8>>()
                                .map_err(|_| "EC_EPOCH_STATE: participant fingerprint decode failed".to_owned())?
                                .ok_or_else(|| "EC_EPOCH_STATE: participant fingerprint is NULL".to_owned())
                        })
                        .next()
                        .transpose()?
                        .ok_or_else(|| "EC_EPOCH_STATE: participant publish returned no fingerprint".to_owned())
                })?
            } else {
                let conninfo = super::super::node_registry::resolve_conninfo_secret(&secret_name)?;
                super::super::remote_transport::remote_publish_epoch(
                    &conninfo,
                    &remote_index_regclass,
                    &build_id_text,
                    &epoch_manifest,
                    &manifest_digest,
                )?
            };
            if acked_fingerprint != epoch_fingerprint {
                return Err(
                    "EC_EPOCH_STATE: participant acknowledged a different epoch fingerprint"
                        .to_owned(),
                );
            }
        }
        if super::super::options::debug_fail_recover_after_publish_ack() {
            return Err(
                "EC_FAULT_INJECTED: publish recovery stopped after participant acknowledgement and before active-pointer swap"
                    .to_owned(),
            );
        }

        // Swap the active pointer to the successor and publish the registration
        // to clear the gate. A first epoch records Applied immediately; a
        // successor CAS-swaps from its predecessor, records Activated, and opens
        // one Pending disposition per predecessor binding (T4b retires them).
        let disposition = extension_relation_name("ec_distann_predecessor_disposition")?;
        Spi::connect_mut(|client| -> Result<(), String> {
            match &predecessor_build_id {
                None => {
                    client
                        .update(
                            &format!(
                                "INSERT INTO {active_table} (
                                     index_oid, logical_index_uuid, build_id, epoch,
                                     epoch_fingerprint, manifest_digest, updated_at
                                 ) VALUES (
                                     $1::oid, $2::uuid, $3::uuid, $4::bigint,
                                     $5::bytea, $6::bytea, clock_timestamp()
                                 )"
                            ),
                            None,
                            &[
                                index_oid.into(),
                                logical_index_uuid.into(),
                                build_id.into(),
                                epoch.into(),
                                epoch_fingerprint.clone().into(),
                                manifest_digest.clone().into(),
                            ],
                        )
                        .map_err(|error| {
                            format!("EC_EPOCH_STATE: active pointer insert failed: {error}")
                        })?;
                    let transitioned = client
                        .update(
                            &format!(
                                "UPDATE {decision_table}
                                    SET decision_state = 'Applied',
                                        activated_at = clock_timestamp(),
                                        applied_at = clock_timestamp()
                                  WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                                    AND build_id = $3::uuid AND decision_state = 'Pending'
                                  RETURNING 1"
                            ),
                            None,
                            &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                        )
                        .map_err(|error| {
                            format!("EC_EPOCH_STATE: decision Applied transition failed: {error}")
                        })?
                        .len();
                    require_exact_transition(
                        PublishDecisionState::Pending,
                        PublishDecisionState::Applied,
                        transitioned,
                        "publish decision",
                    )?;
                }
                Some(predecessor) => {
                    let swapped = client
                        .update(
                            &format!(
                                "UPDATE {active_table}
                                    SET build_id = $3::uuid, epoch = $4::bigint,
                                        epoch_fingerprint = $5::bytea,
                                        manifest_digest = $6::bytea,
                                        updated_at = clock_timestamp()
                                  WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                                    AND build_id = $7::uuid"
                            ),
                            None,
                            &[
                                index_oid.into(),
                                logical_index_uuid.into(),
                                build_id.into(),
                                epoch.into(),
                                epoch_fingerprint.clone().into(),
                                manifest_digest.clone().into(),
                                (*predecessor).into(),
                            ],
                        )
                        .map_err(|error| {
                            format!("EC_EPOCH_STATE: active pointer swap failed: {error}")
                        })?
                        .len();
                    if swapped != 1 {
                        return Err(
                            "EC_EPOCH_STATE: active pointer no longer names the recorded predecessor"
                                .to_owned(),
                        );
                    }
                    let transitioned = client
                        .update(
                            &format!(
                                "UPDATE {decision_table}
                                    SET decision_state = 'Activated',
                                        activated_at = clock_timestamp()
                                  WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                                    AND build_id = $3::uuid AND decision_state = 'Pending'
                                  RETURNING 1"
                            ),
                            None,
                            &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                        )
                        .map_err(|error| {
                            format!("EC_EPOCH_STATE: decision Activated transition failed: {error}")
                        })?
                        .len();
                    require_exact_transition(
                        PublishDecisionState::Pending,
                        PublishDecisionState::Activated,
                        transitioned,
                        "publish decision",
                    )?;
                    // Open one Pending disposition per predecessor participant,
                    // pulling predecessor identity from the decision and
                    // participant identity from the predecessor's bindings.
                    client
                        .update(
                            &format!(
                                "INSERT INTO {disposition} (
                                     index_oid, logical_index_uuid, successor_build_id,
                                     predecessor_build_id, predecessor_epoch,
                                     predecessor_epoch_fingerprint, predecessor_manifest_digest,
                                     predecessor_roster_ordinal, node_id, endpoint_identity,
                                     remote_index_regclass, participant_logical_index_uuid,
                                     successor_activation_digest, disposition
                                 )
                                 SELECT d.index_oid, d.logical_index_uuid, d.build_id,
                                        d.predecessor_build_id, d.predecessor_epoch,
                                        d.predecessor_epoch_fingerprint, d.predecessor_manifest_digest,
                                        b.roster_ordinal, b.node_id, b.endpoint_identity,
                                        b.remote_index_regclass, b.participant_logical_index_uuid,
                                        d.successor_activation_digest, 'Pending'
                                   FROM {decision_table} d
                                   JOIN {binding} b
                                     ON b.index_oid = d.index_oid
                                    AND b.logical_index_uuid = d.logical_index_uuid
                                    AND b.build_id = d.predecessor_build_id
                                  WHERE d.index_oid = $1::oid
                                    AND d.logical_index_uuid = $2::uuid
                                    AND d.build_id = $3::uuid"
                            ),
                            None,
                            &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                        )
                        .map_err(|error| {
                            format!("EC_EPOCH_STATE: predecessor disposition insert failed: {error}")
                        })?;
                }
            }
            let transitioned = client
                .update(
                    &format!(
                        "UPDATE {registration} SET state = 'Published'
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid AND state = 'Decided'
                          RETURNING 1"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| {
                    format!("EC_EPOCH_STATE: registration Published transition failed: {error}")
                })?
                .len();
            require_exact_transition(
                RegistrationState::Decided,
                RegistrationState::Published,
                transitioned,
                "registration",
            )
        })?;

        super::super::traversal_replica::retire_superseded_replicas(
            index_oid,
            logical_index_uuid,
            build_id,
        )?;
        if metadata.is_distributed_control() {
            super::super::physical_dml::refresh_source_mapping(index_oid)?;
        }

        // Release the coordinator session locks after this swap commits.
        schedule_session_lock_release_for_control(index_oid);
        Ok(epoch_fingerprint)
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"))
}
