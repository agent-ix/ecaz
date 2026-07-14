//! Durable publish cancellation and replayable cleanup SQL endpoints.

use pgrx::datum::Uuid;
use pgrx::{pg_extern, PgRelation};

use super::*;

#[pg_extern(volatile, strict)]
fn ec_distann_cancel_epoch_publish(index_regclass: PgRelation, build_id: Uuid, reason: String) {
    cancel_epoch_publish(index_regclass, build_id, reason)
}

#[pg_extern(volatile, strict)]
fn ec_distann_recover_cancelled_publish(index_regclass: PgRelation, build_id: Uuid) {
    recover_cancelled_publish(index_regclass, build_id)
}

pub(super) fn cancel_epoch_publish(index_regclass: PgRelation, build_id: Uuid, reason: String) {
    (|| -> Result<(), String> {
        super::super::lifecycle_guard::require_read_committed("ec_distann_cancel_epoch_publish")?;
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        if !(1..=1024).contains(&reason.len()) {
            return Err(
                "EC_PUBLISH_CANCEL: cancellation reason must contain 1 to 1024 bytes".to_owned(),
            );
        }

        let index_oid = index_regclass.oid();
        drop(index_regclass);
        let (preflight_source_oid, preflight_uuid) =
            preflight_build_lock_identity(index_oid, "ec_distann_cancel_epoch_publish preflight")?;
        let mut source_lock = SourceSessionLockGuard::acquire(
            preflight_source_oid,
            index_oid,
            preflight_uuid,
            build_id,
        )?;
        let (mut control, handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_cancel_epoch_publish",
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

        struct CancelTarget {
            state: PublishDecisionState,
            epoch: i64,
            fingerprint: Vec<u8>,
            manifest_digest: Vec<u8>,
            predecessor_build: Option<Uuid>,
            predecessor_epoch: Option<i64>,
            predecessor_fingerprint: Option<Vec<u8>>,
            predecessor_manifest: Option<Vec<u8>>,
            xmin: u32,
            reason: Option<String>,
            audit: Option<Vec<u8>>,
            audit_digest: Option<Vec<u8>>,
        }
        let decision_table = extension_relation_name("ec_distann_publish_decision")?;
        let target = Spi::connect_mut(|client| -> Result<Option<CancelTarget>, String> {
            client
                .update(
                    &format!(
                        "SELECT decision_state, epoch, epoch_fingerprint, manifest_digest,
                                predecessor_build_id, predecessor_epoch,
                                predecessor_epoch_fingerprint, predecessor_manifest_digest,
                                xmin::text AS decision_xmin, cancellation_reason,
                                cancellation_audit, cancellation_audit_digest
                           FROM {decision_table}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid
                          FOR UPDATE"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| {
                    format!("EC_PUBLISH_CANCEL: publish decision lookup failed: {error}")
                })?
                .map(|row| {
                    let state = PublishDecisionState::parse(
                        &row["decision_state"]
                            .value::<String>()
                            .map_err(|error| {
                                format!("EC_PUBLISH_CANCEL: decision state decode failed: {error}")
                            })?
                            .ok_or_else(|| {
                                "EC_PUBLISH_CANCEL: decision state is NULL".to_owned()
                            })?,
                    )?;
                    let xmin = row["decision_xmin"]
                        .value::<String>()
                        .map_err(|_| {
                            "EC_TRANSACTION_BOUNDARY: publish decision xmin decode failed"
                                .to_owned()
                        })?
                        .ok_or_else(|| {
                            "EC_TRANSACTION_BOUNDARY: publish decision xmin is NULL".to_owned()
                        })?
                        .parse::<u32>()
                        .map_err(|_| {
                            "EC_TRANSACTION_BOUNDARY: publish decision xmin is invalid".to_owned()
                        })?;
                    Ok(CancelTarget {
                        state,
                        epoch: row["epoch"]
                            .value::<i64>()
                            .map_err(|_| "EC_PUBLISH_CANCEL: epoch decode failed".to_owned())?
                            .ok_or_else(|| "EC_PUBLISH_CANCEL: epoch is NULL".to_owned())?,
                        fingerprint: row["epoch_fingerprint"]
                            .value::<Vec<u8>>()
                            .map_err(|_| "EC_PUBLISH_CANCEL: fingerprint decode failed".to_owned())?
                            .ok_or_else(|| "EC_PUBLISH_CANCEL: fingerprint is NULL".to_owned())?,
                        manifest_digest: row["manifest_digest"]
                            .value::<Vec<u8>>()
                            .map_err(|_| {
                                "EC_PUBLISH_CANCEL: manifest digest decode failed".to_owned()
                            })?
                            .ok_or_else(|| {
                                "EC_PUBLISH_CANCEL: manifest digest is NULL".to_owned()
                            })?,
                        predecessor_build: row["predecessor_build_id"].value::<Uuid>().map_err(
                            |_| "EC_PUBLISH_CANCEL: predecessor build id decode failed".to_owned(),
                        )?,
                        predecessor_epoch: row["predecessor_epoch"].value::<i64>().map_err(
                            |_| "EC_PUBLISH_CANCEL: predecessor epoch decode failed".to_owned(),
                        )?,
                        predecessor_fingerprint: row["predecessor_epoch_fingerprint"]
                            .value::<Vec<u8>>()
                            .map_err(|_| {
                                "EC_PUBLISH_CANCEL: predecessor fingerprint decode failed"
                                    .to_owned()
                            })?,
                        predecessor_manifest: row["predecessor_manifest_digest"]
                            .value::<Vec<u8>>()
                            .map_err(|_| {
                                "EC_PUBLISH_CANCEL: predecessor manifest decode failed".to_owned()
                            })?,
                        xmin,
                        reason: row["cancellation_reason"].value::<String>().map_err(|_| {
                            "EC_PUBLISH_CANCEL: cancellation reason decode failed".to_owned()
                        })?,
                        audit: row["cancellation_audit"].value::<Vec<u8>>().map_err(|_| {
                            "EC_PUBLISH_CANCEL: cancellation audit decode failed".to_owned()
                        })?,
                        audit_digest: row["cancellation_audit_digest"]
                            .value::<Vec<u8>>()
                            .map_err(|_| {
                                "EC_PUBLISH_CANCEL: cancellation audit digest decode failed"
                                    .to_owned()
                            })?,
                    })
                })
                .next()
                .transpose()
        })?
        .ok_or_else(|| "EC_PUBLISH_CANCEL: publish decision does not exist".to_owned())?;
        if unsafe {
            pg_sys::TransactionIdIsCurrentTransactionId(pg_sys::TransactionId::from(target.xmin))
        } {
            return Err(
                "EC_TRANSACTION_BOUNDARY: publish decision must commit before cancellation begins"
                    .to_owned(),
            );
        }
        if target.state == PublishDecisionState::Cancelled {
            if target.reason.as_deref() != Some(reason.as_str()) {
                return Err("EC_PUBLISH_CANCEL: conflicting cancellation replay".to_owned());
            }
            let stored_audit = DistannCancelPublishAuditV1::decode(
                target
                    .audit
                    .as_deref()
                    .ok_or_else(|| "EC_PUBLISH_CANCEL: stored audit is NULL".to_owned())?,
            )?;
            let stored_digest: [u8; 32] = target
                .audit_digest
                .ok_or_else(|| "EC_PUBLISH_CANCEL: stored audit digest is NULL".to_owned())?
                .try_into()
                .map_err(|_| "EC_PUBLISH_CANCEL: stored audit digest is not 32 bytes".to_owned())?;
            if stored_audit.digest()? != stored_digest {
                return Err("EC_PUBLISH_CANCEL: stored audit digest is corrupt".to_owned());
            }
            if stored_audit.coordinator_logical_index_uuid != *logical_index_uuid.as_bytes()
                || stored_audit.build_id != *build_id.as_bytes()
                || stored_audit.epoch != u64::try_from(target.epoch).unwrap_or(0)
                || stored_audit.epoch_fingerprint.as_slice() != target.fingerprint
                || stored_audit.manifest_digest.as_slice() != target.manifest_digest
                || stored_audit.reason != reason
            {
                return Err("EC_PUBLISH_CANCEL: stored audit identity is corrupt".to_owned());
            }
            source_lock.release_after_commit();
            return Ok(());
        }
        if target.state != PublishDecisionState::Pending {
            return Err(format!(
                "EC_PUBLISH_CANCEL: only a Pending decision can be cancelled; state is {}",
                target.state
            ));
        }

        let active_table = extension_relation_name("ec_distann_active_epoch")?;
        let active = Spi::connect_mut(
            |client| -> Result<Option<(Uuid, i64, Vec<u8>, Vec<u8>)>, String> {
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
                        format!("EC_PUBLISH_CANCEL: active pointer lookup failed: {error}")
                    })?
                    .map(|row| {
                        Ok((
                            row["build_id"]
                                .value::<Uuid>()
                                .map_err(|_| {
                                    "EC_PUBLISH_CANCEL: active build id decode failed".to_owned()
                                })?
                                .ok_or_else(|| {
                                    "EC_PUBLISH_CANCEL: active build id is NULL".to_owned()
                                })?,
                            row["epoch"]
                                .value::<i64>()
                                .map_err(|_| {
                                    "EC_PUBLISH_CANCEL: active epoch decode failed".to_owned()
                                })?
                                .ok_or_else(|| {
                                    "EC_PUBLISH_CANCEL: active epoch is NULL".to_owned()
                                })?,
                            row["epoch_fingerprint"]
                                .value::<Vec<u8>>()
                                .map_err(|_| {
                                    "EC_PUBLISH_CANCEL: active fingerprint decode failed".to_owned()
                                })?
                                .ok_or_else(|| {
                                    "EC_PUBLISH_CANCEL: active fingerprint is NULL".to_owned()
                                })?,
                            row["manifest_digest"]
                                .value::<Vec<u8>>()
                                .map_err(|_| {
                                    "EC_PUBLISH_CANCEL: active manifest decode failed".to_owned()
                                })?
                                .ok_or_else(|| {
                                    "EC_PUBLISH_CANCEL: active manifest is NULL".to_owned()
                                })?,
                        ))
                    })
                    .next()
                    .transpose()
            },
        )?;
        let predecessor_still_active = match (
            active,
            target.predecessor_build,
            target.predecessor_epoch,
            target.predecessor_fingerprint,
            target.predecessor_manifest,
        ) {
            (None, None, None, None, None) => true,
            (
                Some((active_build, active_epoch, active_fingerprint, active_manifest)),
                Some(predecessor_build),
                Some(predecessor_epoch),
                Some(predecessor_fingerprint),
                Some(predecessor_manifest),
            ) => {
                active_build == predecessor_build
                    && active_epoch == predecessor_epoch
                    && active_fingerprint == predecessor_fingerprint
                    && active_manifest == predecessor_manifest
            }
            _ => false,
        };
        if !predecessor_still_active {
            return Err(
                "EC_PUBLISH_CANCEL: active pointer no longer matches the decision predecessor"
                    .to_owned(),
            );
        }

        let (caller_name, decision_time_unix_micros) = Spi::connect(|client| {
            client
                .select(
                    "SELECT session_user::text AS caller_name,
                            floor(extract(epoch FROM clock_timestamp()) * 1000000)::bigint
                                AS decision_time",
                    None,
                    &[],
                )
                .map_err(|error| {
                    format!("EC_PUBLISH_CANCEL: cancellation audit lookup failed: {error}")
                })?
                .map(|row| {
                    Ok::<_, String>((
                        row["caller_name"]
                            .value::<String>()
                            .map_err(|_| {
                                "EC_PUBLISH_CANCEL: cancellation caller decode failed".to_owned()
                            })?
                            .ok_or_else(|| {
                                "EC_PUBLISH_CANCEL: cancellation caller is NULL".to_owned()
                            })?,
                        row["decision_time"]
                            .value::<i64>()
                            .map_err(|_| {
                                "EC_PUBLISH_CANCEL: cancellation time decode failed".to_owned()
                            })?
                            .ok_or_else(|| {
                                "EC_PUBLISH_CANCEL: cancellation time is NULL".to_owned()
                            })?,
                    ))
                })
                .next()
                .transpose()?
                .ok_or_else(|| "EC_PUBLISH_CANCEL: cancellation audit returned no row".to_owned())
        })?;
        let audit = DistannCancelPublishAuditV1 {
            coordinator_logical_index_uuid: *logical_index_uuid.as_bytes(),
            build_id: *build_id.as_bytes(),
            epoch: u64::try_from(target.epoch)
                .map_err(|_| "EC_PUBLISH_CANCEL: cancelled epoch is invalid".to_owned())?,
            epoch_fingerprint: target
                .fingerprint
                .as_slice()
                .try_into()
                .map_err(|_| "EC_PUBLISH_CANCEL: fingerprint is not 34 bytes".to_owned())?,
            manifest_digest: target
                .manifest_digest
                .as_slice()
                .try_into()
                .map_err(|_| "EC_PUBLISH_CANCEL: manifest digest is not 32 bytes".to_owned())?,
            decision_time_unix_micros,
            caller_name: caller_name.clone(),
            reason: reason.clone(),
        };
        let audit_bytes = audit.encode()?;
        let audit_digest = audit.digest()?;

        let registration = extension_relation_name("ec_distann_build_registration")?;
        Spi::connect_mut(|client| -> Result<(), String> {
            let cancelled = client
                .update(
                    &format!(
                        "UPDATE {decision_table}
                            SET decision_state = 'Cancelled',
                                cancelled_at = clock_timestamp(),
                                cancelled_by = $5::text,
                                cancellation_reason = $4::text,
                                cancellation_time_unix_micros = $6::bigint,
                                cancellation_audit = $7::bytea,
                                cancellation_audit_digest = $8::bytea
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid AND decision_state = 'Pending'
                          RETURNING 1"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        build_id.into(),
                        reason.into(),
                        caller_name.into(),
                        decision_time_unix_micros.into(),
                        audit_bytes.into(),
                        audit_digest.to_vec().into(),
                    ],
                )
                .map_err(|error| {
                    format!("EC_PUBLISH_CANCEL: decision cancellation failed: {error}")
                })?
                .len();
            require_exact_transition(
                PublishDecisionState::Pending,
                PublishDecisionState::Cancelled,
                cancelled,
                "publish decision",
            )?;
            let terminal = client
                .update(
                    &format!(
                        "UPDATE {registration} SET state = 'Cancelled'
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid AND state = 'Decided'
                          RETURNING 1"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| {
                    format!("EC_PUBLISH_CANCEL: registration cancellation failed: {error}")
                })?
                .len();
            require_exact_transition(
                RegistrationState::Decided,
                RegistrationState::Cancelled,
                terminal,
                "registration",
            )
        })?;
        source_lock.release_after_commit();
        Ok(())
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
}

/// Replayable cleanup for participant generations named by a durably
/// Cancelled publish decision. Remote participants may have committed
/// publication before another owner failed, so this uses the canonical
/// cancellation audit rather than ordinary unpublished abort semantics.
pub(super) fn recover_cancelled_publish(index_regclass: PgRelation, build_id: Uuid) {
    (|| -> Result<(), String> {
        super::super::lifecycle_guard::require_read_committed(
            "ec_distann_recover_cancelled_publish",
        )?;
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        let index_oid = index_regclass.oid();
        drop(index_regclass);
        let (mut control, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_recover_cancelled_publish",
        )?;
        control.retain_lock_until_transaction_end();

        let decision = extension_relation_name("ec_distann_publish_decision")?;
        let stored = Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "SELECT decision_state, cancellation_audit,
                                cancellation_audit_digest, cancellation_reclaimed_at IS NOT NULL
                                    AS reclaimed, xmin::text AS decision_xmin
                           FROM {decision}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid
                          FOR UPDATE"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| {
                    format!("EC_PUBLISH_CANCEL: cancelled decision lookup failed: {error}")
                })?
                .map(|row| {
                    let required = |field: &str| -> Result<Vec<u8>, String> {
                        row[field]
                            .value::<Vec<u8>>()
                            .map_err(|_| {
                                format!("EC_PUBLISH_CANCEL: decision {field} decode failed")
                            })?
                            .ok_or_else(|| {
                                format!("EC_PUBLISH_CANCEL: decision {field} is NULL")
                            })
                    };
                    Ok::<_, String>((
                        PublishDecisionState::parse(
                            &row["decision_state"]
                                .value::<String>()
                                .map_err(|error| {
                                    format!("EC_PUBLISH_CANCEL: decision state decode failed: {error}")
                                })?
                                .ok_or_else(|| {
                                    "EC_PUBLISH_CANCEL: decision state is NULL".to_owned()
                                })?,
                        )?,
                        required("cancellation_audit")?,
                        required("cancellation_audit_digest")?,
                        row["reclaimed"]
                            .value::<bool>()
                            .map_err(|_| {
                                "EC_PUBLISH_CANCEL: reclaimed flag decode failed".to_owned()
                            })?
                            .ok_or_else(|| {
                                "EC_PUBLISH_CANCEL: reclaimed flag is NULL".to_owned()
                            })?,
                        row["decision_xmin"]
                            .value::<String>()
                            .map_err(|_| {
                                "EC_TRANSACTION_BOUNDARY: cancelled decision xmin decode failed"
                                    .to_owned()
                            })?
                            .ok_or_else(|| {
                                "EC_TRANSACTION_BOUNDARY: cancelled decision xmin is NULL"
                                    .to_owned()
                            })?
                            .parse::<u32>()
                            .map_err(|_| {
                                "EC_TRANSACTION_BOUNDARY: cancelled decision xmin is invalid"
                                    .to_owned()
                            })?,
                    ))
                })
                .next()
                .transpose()
        })?
        .ok_or_else(|| "EC_PUBLISH_CANCEL: cancelled publish decision is absent".to_owned())?;
        if unsafe {
            pg_sys::TransactionIdIsCurrentTransactionId(pg_sys::TransactionId::from(stored.4))
        } {
            return Err(
                "EC_TRANSACTION_BOUNDARY: cancelled publish decision must commit before recovery begins"
                    .to_owned(),
            );
        }
        if stored.0 != PublishDecisionState::Cancelled {
            return Err(format!(
                "EC_PUBLISH_CANCEL: cleanup requires a Cancelled decision; state is {}",
                stored.0
            ));
        }
        let audit = DistannCancelPublishAuditV1::decode(&stored.1)?;
        let audit_digest: [u8; 32] = stored
            .2
            .as_slice()
            .try_into()
            .map_err(|_| "EC_PUBLISH_CANCEL: cancellation audit digest is not 32 bytes".to_owned())?;
        if audit.digest()? != audit_digest
            || audit.coordinator_logical_index_uuid != *logical_index_uuid.as_bytes()
            || audit.build_id != *build_id.as_bytes()
        {
            return Err("EC_PUBLISH_CANCEL: cancelled decision audit is corrupt".to_owned());
        }
        if stored.3 {
            return Ok(());
        }

        let active = extension_relation_name("ec_distann_active_epoch")?;
        let target_is_active = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT 1 FROM {active}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map(|rows| !rows.is_empty())
                .map_err(|error| {
                    format!("EC_PUBLISH_CANCEL: active pointer lookup failed: {error}")
                })
        })?;
        if target_is_active {
            return Err("EC_PUBLISH_CANCEL: active generation cannot be cancellation-reclaimed".to_owned());
        }

        let binding = extension_relation_name("ec_distann_build_participant_binding")?;
        let participants = Spi::connect(|client| {
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
                    format!("EC_PUBLISH_CANCEL: participant binding lookup failed: {error}")
                })?
                .map(|row| {
                    let text = |field: &str| -> Result<String, String> {
                        row[field]
                            .value::<String>()
                            .map_err(|_| {
                                format!("EC_PUBLISH_CANCEL: binding {field} decode failed")
                            })?
                            .ok_or_else(|| {
                                format!("EC_PUBLISH_CANCEL: binding {field} is NULL")
                            })
                    };
                    Ok::<_, String>((
                        row["is_local"]
                            .value::<bool>()
                            .map_err(|_| {
                                "EC_PUBLISH_CANCEL: binding locality decode failed".to_owned()
                            })?
                            .ok_or_else(|| {
                                "EC_PUBLISH_CANCEL: binding locality is NULL".to_owned()
                            })?,
                        text("conninfo_secret_name")?,
                        text("remote_index_regclass")?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()
        })?;
        if participants.is_empty() {
            return Err("EC_PUBLISH_CANCEL: cancelled decision has no participant bindings".to_owned());
        }
        let local_reclaim = extension_relation_name("ec_distann_reclaim_cancelled_generation")?;
        for (is_local, secret_name, remote_index_regclass) in participants {
            if is_local {
                Spi::connect_mut(|client| -> Result<(), String> {
                    client
                        .update(
                            &format!("SELECT {local_reclaim}($1::oid::regclass, $2::bytea, $3::bytea)"),
                            None,
                            &[
                                index_oid.into(),
                                stored.1.clone().into(),
                                stored.2.clone().into(),
                            ],
                        )
                        .map_err(|error| {
                            format!("EC_PUBLISH_CANCEL: local cancellation reclaim failed: {error}")
                        })?;
                    Ok(())
                })?;
            } else {
                let conninfo = super::super::node_registry::resolve_conninfo_secret(&secret_name)?;
                super::super::remote_transport::remote_reclaim_cancelled_generation(
                    &conninfo,
                    &remote_index_regclass,
                    &stored.1,
                    &stored.2,
                )?;
            }
        }

        let updated = Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "UPDATE {decision}
                            SET cancellation_reclaimed_at = clock_timestamp()
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid AND decision_state = 'Cancelled'
                            AND cancellation_reclaimed_at IS NULL
                          RETURNING 1"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map(|rows| rows.len())
                .map_err(|error| {
                    format!("EC_PUBLISH_CANCEL: cleanup completion update failed: {error}")
                })
        })?;
        if updated != 1 {
            return Err("EC_PUBLISH_CANCEL: cleanup completion changed concurrently".to_owned());
        }
        Ok(())
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
}
