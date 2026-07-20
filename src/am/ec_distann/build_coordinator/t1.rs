//! T1 registration and pre-decision abort SQL endpoints.

use pgrx::datum::Uuid;
use pgrx::{pg_extern, PgRelation};

use super::*;

#[pg_extern(volatile, strict)]
fn ec_distann_begin_epoch_build(index_regclass: PgRelation, epoch: i64, build_id: Uuid) -> Vec<u8> {
    begin_epoch_build(index_regclass, epoch, build_id)
}

#[pg_extern(volatile, strict)]
fn ec_distann_abort_epoch_build(index_regclass: PgRelation, build_id: Uuid) {
    abort_epoch_build(index_regclass, build_id)
}

pub(super) fn abort_epoch_build(index_regclass: PgRelation, build_id: Uuid) {
    (|| -> Result<(), String> {
        super::super::lifecycle_guard::require_read_committed("ec_distann_abort_epoch_build")?;
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        let index_oid = index_regclass.oid();
        drop(index_regclass);
        let (preflight_source_oid, preflight_uuid) =
            preflight_build_lock_identity(index_oid, "ec_distann_abort_epoch_build preflight")?;
        let mut source_lock = SourceSessionLockGuard::acquire(
            preflight_source_oid,
            index_oid,
            preflight_uuid,
            build_id,
        )?;
        let (_control_guard, handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_abort_epoch_build",
        )?;
        if logical_index_uuid != preflight_uuid
            || index_heap_relation_oid_handle(handle) != preflight_source_oid
        {
            return Err(
                "EC_BUILD_ID_CONFLICT: control identity changed while acquiring source lock"
                    .to_owned(),
            );
        }
        let _registry_revision = lock_registry_revision(index_oid, logical_index_uuid)?;
        let registration = extension_relation_name("ec_distann_build_registration")?;
        let binding = extension_relation_name("ec_distann_build_participant_binding")?;

        // Lock the registration row and read its state. Absence is idempotent.
        let state = Spi::connect_mut(|client| -> Result<Option<RegistrationState>, String> {
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
                .map_err(|error| format!("EC_BUILD_STATE: registration lookup failed: {error}"))?
                .map(|row| {
                    let state = row["state"]
                        .value::<String>()
                        .map_err(|error| {
                            format!("EC_BUILD_STATE: registration state decode failed: {error}")
                        })?
                        .ok_or_else(|| "EC_BUILD_STATE: registration state is NULL".to_owned())?;
                    RegistrationState::parse(&state)
                })
                .next()
                .transpose()
        })?;
        let Some(state) = state else {
            return Ok(());
        };
        match state {
            RegistrationState::Aborted => return Ok(()),
            RegistrationState::Registered
            | RegistrationState::Building
            | RegistrationState::Ready => {}
            other => {
                return Err(format!(
                    "EC_BUILD_STATE: cannot abort a build in state {other}"
                ));
            }
        }
        source_lock.retain();
        // A build with a durable Pending/Activated publish decision cannot be
        // aborted — destroying its generation would strand the decision (there
        // is no decision-abort). Decide moves the registration to 'Decided',
        // which the state check above already rejects; this guards the boundary
        // explicitly (FR-082 P1-1).
        let decision = extension_relation_name("ec_distann_publish_decision")?;
        let has_active_decision = Spi::connect(|client| -> Result<bool, String> {
            client
                .select(
                    &format!(
                        "SELECT EXISTS (
                             SELECT 1 FROM {decision}
                              WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                                AND build_id = $3::uuid
                                AND decision_state IN ('Pending', 'Activated')
                         ) AS present"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| {
                    format!("EC_BUILD_STATE: decision presence lookup failed: {error}")
                })?
                .map(|row| {
                    row["present"]
                        .value::<bool>()
                        .map_err(|_| "EC_BUILD_STATE: decision presence decode failed".to_owned())?
                        .ok_or_else(|| "EC_BUILD_STATE: decision presence is NULL".to_owned())
                })
                .next()
                .transpose()?
                .ok_or_else(|| "EC_BUILD_STATE: decision presence returned no row".to_owned())
        })?;
        if has_active_decision {
            return Err(
                "EC_BUILD_STATE: cannot abort a build with a durable publish decision".to_owned(),
            );
        }

        // Abort each participant's unpublished generation from the immutable
        // private binding snapshot, including owners removed from the desired
        // roster after begin-build.
        let bindings = Spi::connect(|client| -> Result<Vec<(bool, String, String)>, String> {
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
                .map_err(|error| {
                    format!("EC_BUILD_STATE: participant binding lookup failed: {error}")
                })?
                .map(|row| {
                    let is_local = row["is_local"]
                        .value::<bool>()
                        .map_err(|_| {
                            "EC_BUILD_STATE: participant locality decode failed".to_owned()
                        })?
                        .ok_or_else(|| "EC_BUILD_STATE: participant locality is NULL".to_owned())?;
                    let text = |field: &str| -> Result<String, String> {
                        row[field]
                            .value::<String>()
                            .map_err(|_| format!("EC_BUILD_STATE: {field} decode failed"))?
                            .ok_or_else(|| format!("EC_BUILD_STATE: {field} is NULL"))
                    };
                    Ok((
                        is_local,
                        text("conninfo_secret_name")?,
                        text("remote_index_regclass")?,
                    ))
                })
                .collect::<Result<Vec<_>, _>>()
        })?;
        let abort_handoff = extension_relation_name("ec_distann_abort_epoch_handoff")?;
        let build_id_text = build_id.to_string();
        for (is_local, secret_name, remote_index_regclass) in bindings {
            if is_local {
                Spi::connect_mut(|client| -> Result<(), String> {
                    client
                        .update(
                            &format!("SELECT {abort_handoff}($1::oid::regclass, $2::uuid)"),
                            None,
                            &[index_oid.into(), build_id.into()],
                        )
                        .map_err(|error| {
                            format!("EC_BUILD_STATE: local generation abort failed: {error}")
                        })?;
                    Ok(())
                })?;
            } else {
                let conninfo = super::super::node_registry::resolve_conninfo_secret(&secret_name)?;
                super::super::remote_transport::remote_abort_epoch_handoff(
                    &conninfo,
                    &remote_index_regclass,
                    &build_id_text,
                )?;
            }
        }

        // Move the registration to Aborted. The durable gate only matches
        // Registered/Building/Ready/Decided, so this clears it.
        let head_state = extension_relation_name("ec_distann_generation_head_state")?;
        Spi::connect_mut(|client| -> Result<(), String> {
            client
                .update(
                    &format!(
                        "DELETE FROM {head_state}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| {
                    format!("EC_BUILD_STATE: head-sample abort cleanup failed: {error}")
                })?;
            let transitioned = client
                .update(
                    &format!(
                        "UPDATE {registration} SET state = 'Aborted'
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid AND state = $4::text
                          RETURNING 1"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        build_id.into(),
                        state.as_str().into(),
                    ],
                )
                .map_err(|error| {
                    format!("EC_BUILD_STATE: registration abort update failed: {error}")
                })?
                .len();
            require_exact_transition(
                state,
                RegistrationState::Aborted,
                transitioned,
                "registration",
            )
        })?;

        // Release the coordinator's held session locks, but only after this
        // gate-clearing transaction commits.
        schedule_session_lock_release_for_control(index_oid);
        Ok(())
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"))
}

pub(super) fn begin_epoch_build(index_regclass: PgRelation, epoch: i64, build_id: Uuid) -> Vec<u8> {
    (|| -> Result<Vec<u8>, String> {
        super::super::lifecycle_guard::require_read_committed("ec_distann_begin_epoch_build")?;
        super::super::build_gate::require_shared_preload()?;
        super::super::build_gate::lock_global_gate_serialization(false)?;
        let epoch = u64::try_from(epoch)
            .ok()
            .filter(|epoch| *epoch > 0)
            .ok_or_else(|| "EC_BUILD_STATE: epoch must be positive".to_owned())?;
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be RFC 4122 v4".to_owned());
        }
        let index_oid = index_regclass.oid();
        // PgRelation conversion holds AccessShareLock. Use it only for the
        // short preflight, then release every control-index lock before taking
        // the source session lock. Source DDL otherwise has the inverse
        // source→index order and can deadlock begin-build.
        let (preflight_control, preflight_handle, _metadata, preflight_uuid) =
            open_control_index(
                index_oid,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
                "ec_distann_begin_epoch_build preflight",
            )?;
        let source_relation_oid = index_heap_relation_oid_handle(preflight_handle);
        drop(preflight_control);
        drop(index_regclass);

        let mut source_lock = SourceSessionLockGuard::acquire(
            source_relation_oid,
            index_oid,
            preflight_uuid,
            build_id,
        )?;
        let has_inheritance_edge = Spi::get_one_with_args::<bool>(
            "SELECT EXISTS (
                 SELECT 1 FROM pg_catalog.pg_inherits
                  WHERE inhrelid = $1::oid OR inhparent = $1::oid
             )",
            &[source_relation_oid.into()],
        )
        .map_err(|error| {
            format!("EC_BUILD_STATE: source inheritance topology lookup failed: {error}")
        })?
        .ok_or_else(|| "EC_BUILD_STATE: source inheritance topology lookup returned NULL".to_owned())?;
        if has_inheritance_edge {
            return Err(
                "EC_BUILD_STATE: distributed build sources may not be partitioned or participate in table inheritance in format v1"
                    .to_owned(),
            );
        }
        let (mut control, handle, metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_begin_epoch_build",
        )?;
        if logical_index_uuid != preflight_uuid
            || index_heap_relation_oid_handle(handle) != source_relation_oid
        {
            return Err(
                "EC_BUILD_ID_CONFLICT: control identity changed while acquiring source lock"
                    .to_owned(),
            );
        }
        control.retain_lock_until_transaction_end();
        // Registry operations and begin-build share this row lock after the
        // retained coordinator-control relation lock. This includes replay.
        let registry_revision = lock_registry_revision(index_oid, logical_index_uuid)?;
        let row_schema_fingerprint = resolve_relation_schema(source_relation_oid)?
            .descriptor
            .fingerprint()?;
        let compatibility_digest = control_compatibility_digest(handle, &metadata)?;

        if let Some((existing, requires_source_lock)) = replay_registration(
            index_oid,
            logical_index_uuid,
            build_id,
            epoch,
            row_schema_fingerprint,
            compatibility_digest,
            source_relation_oid,
        )? {
            if requires_source_lock {
                source_lock.retain();
            } else {
                source_lock.release_after_commit();
            }
            return Ok(existing.to_vec());
        }

        ensure_build_slot_available(index_oid, logical_index_uuid)?;

        let participants = desired_participants(index_oid, logical_index_uuid)?;
        if participants.is_empty() {
            return Err("EC_NODE_DESCRIPTOR: cannot register a build with no owners".to_owned());
        }
        let roster = public_roster(&participants)?;
        let roster_snapshot = encode_roster_snapshot(&roster)?;
        let frozen_roster_digest = roster_digest(&roster)?;
        if participants
            .iter()
            .any(|participant| participant.compatibility_digest != compatibility_digest)
        {
            return Err(
                "EC_NODE_DESCRIPTOR: desired participant compatibility changed before build registration"
                    .to_owned(),
            );
        }
        let digest = registration_digest(
            index_oid,
            logical_index_uuid,
            source_relation_oid,
            epoch,
            build_id,
            registry_revision,
            &roster_snapshot,
            frozen_roster_digest,
            row_schema_fingerprint,
            compatibility_digest,
            &participants,
        )?;
        let registration = extension_relation_name("ec_distann_build_registration")?;
        let binding = extension_relation_name("ec_distann_build_participant_binding")?;
        Spi::connect_mut(|client| -> Result<(), String> {
            client
                .update(
                    &format!(
                        "INSERT INTO {registration} (
                             index_oid, logical_index_uuid, source_relid, build_id, epoch, state,
                             registry_revision, roster_snapshot, roster_digest,
                             row_schema_fingerprint, registration_digest
                         ) VALUES (
                             $1::oid, $2::uuid, $3::oid, $4::uuid, $5::bigint, 'Registered',
                             $6::bigint, $7::bytea, $8::bytea, $9::bytea, $10::bytea
                         )"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        source_relation_oid.into(),
                        build_id.into(),
                        i64::try_from(epoch)
                            .map_err(|_| "EC_BUILD_STATE: epoch exceeds bigint".to_owned())?
                            .into(),
                        i64::try_from(registry_revision)
                            .map_err(|_| {
                                "EC_BUILD_STATE: registry revision exceeds bigint".to_owned()
                            })?
                            .into(),
                        roster_snapshot.clone().into(),
                        frozen_roster_digest.to_vec().into(),
                        row_schema_fingerprint.to_vec().into(),
                        digest.to_vec().into(),
                    ],
                )
                .map_err(|error| {
                    format!("EC_BUILD_STATE: build registration insert failed: {error}")
                })?;
            for participant in &participants {
                client
                    .update(
                        &format!(
                            "INSERT INTO {binding} (
                                 index_oid, logical_index_uuid, build_id, roster_ordinal,
                                 node_id, endpoint_identity, conninfo_secret_name,
                                 remote_index_regclass, participant_logical_index_uuid,
                                 compatibility_digest, is_local
                             ) VALUES (
                                 $1::oid, $2::uuid, $3::uuid, $4::integer,
                                 $5::integer, $6::text, $7::text, $8::text,
                                 $9::uuid, $10::bytea, $11::boolean
                             )"
                        ),
                        None,
                        &[
                            index_oid.into(),
                            logical_index_uuid.into(),
                            build_id.into(),
                            i32::try_from(participant.roster_ordinal)
                                .expect("validated roster ordinal fits i32")
                                .into(),
                            i32::try_from(participant.node_id)
                                .expect("validated node id fits i32")
                                .into(),
                            participant.endpoint_identity.clone().into(),
                            participant.conninfo_secret_name.clone().into(),
                            participant.remote_index_regclass.clone().into(),
                            participant.participant_logical_index_uuid.into(),
                            participant.compatibility_digest.to_vec().into(),
                            participant.is_local.into(),
                        ],
                    )
                    .map_err(|error| {
                        format!(
                            "EC_BUILD_STATE: private participant binding insert failed: {error}"
                        )
                    })?;
            }
            Ok(())
        })?;
        source_lock.retain();
        Ok(digest.to_vec())
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"))
}
