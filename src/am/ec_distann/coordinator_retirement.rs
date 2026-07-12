//! Coordinator-side FR-082 retirement decision and replay.
//!
//! Decision creation is a short transaction under the logical index's exact
//! retirement fence. Participant reclaim is deliberately a later recovery
//! transaction so no physical relation is dropped before the decision commits.

use pgrx::datum::Uuid;
use pgrx::{pg_extern, pg_sys, PgRelation, Spi};

use super::canonical_wire::{CanonicalDecoder, CanonicalEncoder};
use super::generation_catalog::extension_relation_name;
use super::generation_descriptor::{decode_roster, encode_roster};
use super::generation_store::open_control_index;
use super::lifecycle_wire::{
    DistannAbandonedBinding, DistannAbandonedBindingSetV1, DistannRetireDecisionV1,
};
use super::manifest_v2::DistannEpochFingerprint;
use super::scan_registry::{acquire_retirement_fence_xact, live_token_count_for_fingerprint};

fn fixed<const N: usize>(bytes: Vec<u8>, field: &str) -> Result<[u8; N], String> {
    bytes
        .try_into()
        .map_err(|_| format!("EC_EPOCH_STATE: {field} must be {N} bytes"))
}

fn canonical_retire_roster(registration_snapshot: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = CanonicalDecoder::new(registration_snapshot, "build roster snapshot")?;
    if decoder.get_u16("build roster snapshot version")? != 1 {
        return Err("EC_EPOCH_STATE: unsupported build roster snapshot version".to_owned());
    }
    let roster = decode_roster(&mut decoder)?;
    decoder.finish("build roster snapshot")?;
    let mut encoder = CanonicalEncoder::with_capacity(registration_snapshot.len());
    encode_roster(&mut encoder, &roster)?;
    encoder.finish()
}

pub(crate) fn ensure_fingerprint_not_retiring(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    fingerprint: &[u8; 34],
) -> Result<(), String> {
    let retire = extension_relation_name("ec_distann_retire_decision")?;
    let exists = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT 1 FROM {retire}
                      WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                        AND epoch_fingerprint = $3::bytea"
                ),
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    fingerprint.to_vec().into(),
                ],
            )
            .map(|rows| !rows.is_empty())
            .map_err(|error| format!("EC_EPOCH_STATE: retire-decision lookup failed: {error}"))
    })?;
    if exists {
        Err("EC_EPOCH_STATE: fingerprint has a committed retire decision".to_owned())
    } else {
        Ok(())
    }
}

struct RetireTarget {
    build_id: Uuid,
    epoch: i64,
    fingerprint: [u8; 34],
    manifest_digest: [u8; 32],
    roster_snapshot: Vec<u8>,
    roster_digest: [u8; 32],
    covering_successor_build_id: Uuid,
    covering_successor_activation_digest: [u8; 32],
}

fn lock_retire_target(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    fingerprint: &[u8; 34],
) -> Result<RetireTarget, String> {
    let publish = extension_relation_name("ec_distann_publish_decision")?;
    let registration = extension_relation_name("ec_distann_build_registration")?;
    Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "SELECT predecessor.build_id, predecessor.epoch,
                            predecessor.epoch_fingerprint, predecessor.manifest_digest,
                            registration.roster_snapshot, registration.roster_digest,
                            successor.build_id AS successor_build_id,
                            successor.successor_activation_digest
                       FROM {publish} predecessor
                       JOIN {registration} registration
                         ON registration.index_oid = predecessor.index_oid
                        AND registration.logical_index_uuid = predecessor.logical_index_uuid
                        AND registration.build_id = predecessor.build_id
                       JOIN {publish} successor
                         ON successor.index_oid = predecessor.index_oid
                        AND successor.logical_index_uuid = predecessor.logical_index_uuid
                        AND successor.predecessor_build_id = predecessor.build_id
                        AND successor.predecessor_epoch = predecessor.epoch
                        AND successor.predecessor_epoch_fingerprint = predecessor.epoch_fingerprint
                        AND successor.predecessor_manifest_digest = predecessor.manifest_digest
                        AND successor.decision_state = 'Applied'
                      WHERE predecessor.index_oid = $1::oid
                        AND predecessor.logical_index_uuid = $2::uuid
                        AND predecessor.epoch_fingerprint = $3::bytea
                      FOR UPDATE OF predecessor, successor"
                ),
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    fingerprint.to_vec().into(),
                ],
            )
            .map_err(|error| format!("EC_EPOCH_STATE: retire target lookup failed: {error}"))?
            .map(|row| {
                let required = |field: &str| -> Result<Vec<u8>, String> {
                    row[field]
                        .value::<Vec<u8>>()
                        .map_err(|_| format!("EC_EPOCH_STATE: {field} decode failed"))?
                        .ok_or_else(|| format!("EC_EPOCH_STATE: {field} is NULL"))
                };
                Ok::<RetireTarget, String>(RetireTarget {
                    build_id: row["build_id"]
                        .value::<Uuid>()
                        .map_err(|_| "EC_EPOCH_STATE: retire build id decode failed".to_owned())?
                        .ok_or_else(|| "EC_EPOCH_STATE: retire build id is NULL".to_owned())?,
                    epoch: row["epoch"]
                        .value::<i64>()
                        .map_err(|_| "EC_EPOCH_STATE: retire epoch decode failed".to_owned())?
                        .ok_or_else(|| "EC_EPOCH_STATE: retire epoch is NULL".to_owned())?,
                    fingerprint: fixed(required("epoch_fingerprint")?, "epoch fingerprint")?,
                    manifest_digest: fixed(required("manifest_digest")?, "manifest digest")?,
                    roster_snapshot: canonical_retire_roster(&required("roster_snapshot")?)?,
                    roster_digest: fixed(required("roster_digest")?, "roster digest")?,
                    covering_successor_build_id: row["successor_build_id"]
                        .value::<Uuid>()
                        .map_err(|_| "EC_EPOCH_STATE: successor build id decode failed".to_owned())?
                        .ok_or_else(|| "EC_EPOCH_STATE: successor build id is NULL".to_owned())?,
                    covering_successor_activation_digest: fixed(
                        required("successor_activation_digest")?,
                        "successor activation digest",
                    )?,
                })
            })
            .next()
            .transpose()?
            .ok_or_else(|| {
                "EC_EPOCH_STATE: retirement requires a non-active predecessor covered by an Applied successor decision"
                    .to_owned()
            })
    })
}

fn abandoned_bindings(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    successor_build_id: Uuid,
) -> Result<DistannAbandonedBindingSetV1, String> {
    let disposition = extension_relation_name("ec_distann_predecessor_disposition")?;
    Spi::connect_mut(|client| {
        let rows = client
            .update(
                &format!(
                    "SELECT predecessor_roster_ordinal, disposition, abandon_audit_digest
                       FROM {disposition}
                      WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                        AND successor_build_id = $3::uuid
                      ORDER BY predecessor_roster_ordinal
                      FOR SHARE"
                ),
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    successor_build_id.into(),
                ],
            )
            .map_err(|error| {
                format!("EC_EPOCH_STATE: predecessor disposition lookup failed: {error}")
            })?;
        if rows.is_empty() {
            return Err(
                "EC_EPOCH_STATE: covering successor has no predecessor dispositions".to_owned(),
            );
        }
        let mut abandoned = Vec::new();
        for row in rows {
            let ordinal = row["predecessor_roster_ordinal"]
                .value::<i32>()
                .map_err(|_| "EC_EPOCH_STATE: predecessor ordinal decode failed".to_owned())?
                .ok_or_else(|| "EC_EPOCH_STATE: predecessor ordinal is NULL".to_owned())?;
            let state = row["disposition"]
                .value::<String>()
                .map_err(|_| "EC_EPOCH_STATE: predecessor disposition decode failed".to_owned())?
                .ok_or_else(|| "EC_EPOCH_STATE: predecessor disposition is NULL".to_owned())?;
            match state.as_str() {
                "Retired" => {}
                "Abandoned" => {
                    let digest = row["abandon_audit_digest"]
                        .value::<Vec<u8>>()
                        .map_err(|_| "EC_EPOCH_STATE: abandon audit digest decode failed".to_owned())?
                        .ok_or_else(|| "EC_EPOCH_STATE: abandon audit digest is NULL".to_owned())?;
                    abandoned.push(DistannAbandonedBinding {
                        roster_ordinal: u32::try_from(ordinal)
                            .map_err(|_| "EC_EPOCH_STATE: predecessor ordinal is negative".to_owned())?,
                        abandon_audit_digest: fixed(digest, "abandon audit digest")?,
                    });
                }
                _ => {
                    return Err(
                        "EC_EPOCH_STATE: retirement requires every predecessor disposition to be terminal"
                            .to_owned(),
                    )
                }
            }
        }
        Ok(DistannAbandonedBindingSetV1 { entries: abandoned })
    })
}

fn caller_and_time() -> Result<(String, i64), String> {
    Spi::connect(|client| {
        client
            .select(
                "SELECT session_user::text AS caller_name,
                        floor(extract(epoch FROM clock_timestamp()) * 1000000)::bigint AS decision_time",
                None,
                &[],
            )
            .map_err(|error| format!("EC_EPOCH_STATE: retire audit lookup failed: {error}"))?
            .map(|row| {
                Ok::<(String, i64), String>((
                    row["caller_name"]
                        .value::<String>()
                        .map_err(|_| "EC_EPOCH_STATE: caller name decode failed".to_owned())?
                        .ok_or_else(|| "EC_EPOCH_STATE: caller name is NULL".to_owned())?,
                    row["decision_time"]
                        .value::<i64>()
                        .map_err(|_| "EC_EPOCH_STATE: decision time decode failed".to_owned())?
                        .ok_or_else(|| "EC_EPOCH_STATE: decision time is NULL".to_owned())?,
                ))
            })
            .next()
            .transpose()?
            .ok_or_else(|| "EC_EPOCH_STATE: retire audit lookup returned no row".to_owned())
    })
}

fn decide_retirement(
    index_oid: pg_sys::Oid,
    epoch_fingerprint: Vec<u8>,
    forced: bool,
    reason: String,
) -> Result<(), String> {
    super::build_gate::require_shared_preload()?;
    let fingerprint: [u8; 34] = fixed(epoch_fingerprint, "epoch fingerprint")?;
    DistannEpochFingerprint::decode(&fingerprint)?;
    let (control, _handle, _metadata, logical_index_uuid) = open_control_index(
        index_oid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        "ec_distann_retire_epoch preflight",
    )?;
    drop(control);
    acquire_retirement_fence_xact(
        unsafe { pg_sys::MyDatabaseId },
        *logical_index_uuid.as_bytes(),
    )
    .map_err(|error| error.stable_message().to_owned())?;
    let (mut control, _handle, _metadata, locked_uuid) = open_control_index(
        index_oid,
        pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
        "ec_distann_retire_epoch",
    )?;
    control.retain_lock_until_transaction_end();
    if locked_uuid != logical_index_uuid {
        return Err("EC_EPOCH_STATE: control identity changed during retirement".to_owned());
    }

    let active = extension_relation_name("ec_distann_active_epoch")?;
    let is_active = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT 1 FROM {active}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND epoch_fingerprint = $3::bytea"
                ),
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    fingerprint.to_vec().into(),
                ],
            )
            .map(|rows| !rows.is_empty())
            .map_err(|error| format!("EC_EPOCH_STATE: active pointer lookup failed: {error}"))
    })?;
    if is_active {
        return Err("EC_EPOCH_STATE: active epoch cannot be retired".to_owned());
    }

    let retire = extension_relation_name("ec_distann_retire_decision")?;
    if let Some(existing_decision) = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT retire_decision FROM {retire}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND epoch_fingerprint = $3::bytea"
                ),
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    fingerprint.to_vec().into(),
                ],
            )
            .map_err(|error| format!("EC_EPOCH_STATE: retire replay lookup failed: {error}"))?
            .map(|row| {
                row["retire_decision"]
                    .value::<Vec<u8>>()
                    .map_err(|_| "EC_EPOCH_STATE: retire replay decode failed".to_owned())?
                    .ok_or_else(|| "EC_EPOCH_STATE: retire replay is NULL".to_owned())
            })
            .next()
            .transpose()
    })? {
        let existing = DistannRetireDecisionV1::decode(&existing_decision)?;
        if existing.forced != forced || existing.reason != reason {
            return Err("EC_EPOCH_STATE: conflicting retire decision replay".to_owned());
        }
        return Ok(());
    }

    let target = lock_retire_target(index_oid, logical_index_uuid, &fingerprint)?;
    let live = live_token_count_for_fingerprint(logical_index_uuid, fingerprint)
        .map_err(|error| error.stable_message().to_owned())?;
    if live != 0 && !forced {
        return Err(format!(
            "EC_RETENTION_ACTIVE: retired fingerprint has {live} live scan registration(s)"
        ));
    }
    let abandoned = abandoned_bindings(
        index_oid,
        logical_index_uuid,
        target.covering_successor_build_id,
    )?;
    let (caller_name, decision_time_unix_micros) = caller_and_time()?;
    let decision = DistannRetireDecisionV1 {
        coordinator_logical_index_uuid: *logical_index_uuid.as_bytes(),
        target_build_id: *target.build_id.as_bytes(),
        epoch: u64::try_from(target.epoch)
            .map_err(|_| "EC_EPOCH_STATE: retire target epoch is invalid".to_owned())?,
        target_fingerprint: target.fingerprint,
        target_manifest_digest: target.manifest_digest,
        target_roster_snapshot: target.roster_snapshot.clone(),
        roster_digest: target.roster_digest,
        abandoned_bindings: abandoned,
        forced,
        overridden_in_flight_count: u64::from(live),
        decision_time_unix_micros,
        caller_name: caller_name.clone(),
        reason: reason.clone(),
    };
    let decision_bytes = decision.encode()?;
    let decision_digest = decision.digest()?;
    let abandoned_bytes = decision.abandoned_binding_set_bytes()?;
    let abandoned_digest = decision.abandoned_binding_set_digest()?;
    Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "INSERT INTO {retire} (
                             index_oid, logical_index_uuid, build_id, epoch,
                             epoch_fingerprint, manifest_digest, roster_snapshot,
                             roster_digest, abandoned_binding_set,
                             abandoned_binding_set_digest, retire_decision,
                             retire_decision_digest, forced,
                             overridden_in_flight_count, caller_name, reason,
                             decision_time_unix_micros,
                             covering_successor_build_id,
                             covering_successor_activation_digest, decision_state
                         ) VALUES (
                             $1::oid, $2::uuid, $3::uuid, $4::bigint,
                             $5::bytea, $6::bytea, $7::bytea, $8::bytea,
                             $9::bytea, $10::bytea, $11::bytea, $12::bytea,
                             $13::boolean, $14::bigint, $15::text, $16::text,
                             $17::bigint, $18::uuid, $19::bytea, 'Pending'
                         )"
                ),
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    target.build_id.into(),
                    target.epoch.into(),
                    target.fingerprint.to_vec().into(),
                    target.manifest_digest.to_vec().into(),
                    target.roster_snapshot.into(),
                    target.roster_digest.to_vec().into(),
                    abandoned_bytes.into(),
                    abandoned_digest.to_vec().into(),
                    decision_bytes.into(),
                    decision_digest.to_vec().into(),
                    forced.into(),
                    i64::from(live).into(),
                    caller_name.into(),
                    reason.into(),
                    decision_time_unix_micros.into(),
                    target.covering_successor_build_id.into(),
                    target.covering_successor_activation_digest.to_vec().into(),
                ],
            )
            .map_err(|error| format!("EC_EPOCH_STATE: retire decision insert failed: {error}"))?;
        Ok::<(), String>(())
    })
}

#[pg_extern(name = "ec_distann_retire_epoch", volatile, strict)]
fn ec_distann_retire_physical_epoch(index_regclass: PgRelation, epoch_fingerprint: Vec<u8>) {
    super::lifecycle_guard::require_read_committed("ec_distann_retire_epoch")
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    let index_oid = index_regclass.oid();
    drop(index_regclass);
    decide_retirement(index_oid, epoch_fingerprint, false, "normal".to_owned())
        .unwrap_or_else(|error| pgrx::error!("{error}"));
}

#[pg_extern(name = "ec_distann_force_retire_epoch", volatile, strict)]
fn ec_distann_force_retire_physical_epoch(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
    reason: String,
) {
    super::lifecycle_guard::require_read_committed("ec_distann_force_retire_epoch")
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    let index_oid = index_regclass.oid();
    drop(index_regclass);
    decide_retirement(index_oid, epoch_fingerprint, true, reason)
        .unwrap_or_else(|error| pgrx::error!("{error}"));
}

struct RecoveryDecision {
    build_id: Uuid,
    decision: Vec<u8>,
    digest: Vec<u8>,
    state: String,
    xmin: u32,
}

#[pg_extern(volatile, strict)]
fn ec_distann_recover_epoch_retire(index_regclass: PgRelation, epoch_fingerprint: Vec<u8>) {
    (|| -> Result<(), String> {
        super::lifecycle_guard::require_read_committed("ec_distann_recover_epoch_retire")?;
        let fingerprint: [u8; 34] = fixed(epoch_fingerprint, "epoch fingerprint")?;
        DistannEpochFingerprint::decode(&fingerprint)?;
        let index_oid = index_regclass.oid();
        drop(index_regclass);
        let (mut control, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_recover_epoch_retire",
        )?;
        control.retain_lock_until_transaction_end();
        let retire = extension_relation_name("ec_distann_retire_decision")?;
        let stored = Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "SELECT build_id, retire_decision, retire_decision_digest, decision_state,
                                xmin::text AS decision_xmin
                           FROM {retire}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND epoch_fingerprint = $3::bytea
                          FOR UPDATE"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        fingerprint.to_vec().into(),
                    ],
                )
                .map_err(|error| format!("EC_EPOCH_STATE: retire recovery lookup failed: {error}"))?
                .map(|row| {
                    Ok::<RecoveryDecision, String>(RecoveryDecision {
                        build_id: row["build_id"]
                            .value::<Uuid>()
                            .map_err(|_| {
                                "EC_EPOCH_STATE: retire build id decode failed".to_owned()
                            })?
                            .ok_or_else(|| "EC_EPOCH_STATE: retire build id is NULL".to_owned())?,
                        decision: row["retire_decision"]
                            .value::<Vec<u8>>()
                            .map_err(|_| {
                                "EC_EPOCH_STATE: retire decision decode failed".to_owned()
                            })?
                            .ok_or_else(|| "EC_EPOCH_STATE: retire decision is NULL".to_owned())?,
                        digest: row["retire_decision_digest"]
                            .value::<Vec<u8>>()
                            .map_err(|_| "EC_EPOCH_STATE: retire digest decode failed".to_owned())?
                            .ok_or_else(|| "EC_EPOCH_STATE: retire digest is NULL".to_owned())?,
                        state: row["decision_state"]
                            .value::<String>()
                            .map_err(|_| "EC_EPOCH_STATE: retire state decode failed".to_owned())?
                            .ok_or_else(|| "EC_EPOCH_STATE: retire state is NULL".to_owned())?,
                        xmin: row["decision_xmin"]
                            .value::<String>()
                            .map_err(|_| {
                                "EC_TRANSACTION_BOUNDARY: retire decision xmin decode failed"
                                    .to_owned()
                            })?
                            .ok_or_else(|| {
                                "EC_TRANSACTION_BOUNDARY: retire decision xmin is NULL".to_owned()
                            })?
                            .parse::<u32>()
                            .map_err(|_| {
                                "EC_TRANSACTION_BOUNDARY: retire decision xmin is invalid"
                                    .to_owned()
                            })?,
                    })
                })
                .next()
                .transpose()?
                .ok_or_else(|| {
                    "EC_EPOCH_STATE: no durable retire decision for fingerprint".to_owned()
                })
        })?;
        if unsafe {
            pg_sys::TransactionIdIsCurrentTransactionId(pg_sys::TransactionId::from(stored.xmin))
        } {
            return Err(
                "EC_TRANSACTION_BOUNDARY: retire decision must commit before recovery begins"
                    .to_owned(),
            );
        }
        if stored.state == "Applied" {
            return Ok(());
        }
        let decoded = DistannRetireDecisionV1::decode(&stored.decision)?;
        if decoded.digest()?.as_slice() != stored.digest.as_slice()
            || decoded.target_build_id != *stored.build_id.as_bytes()
            || decoded.target_fingerprint != fingerprint
            || decoded.coordinator_logical_index_uuid != *logical_index_uuid.as_bytes()
        {
            return Err("EC_EPOCH_STATE: stored retire decision identity is corrupt".to_owned());
        }

        let binding = extension_relation_name("ec_distann_build_participant_binding")?;
        let participants = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT roster_ordinal, remote_index_regclass, is_local,
                                conninfo_secret_name
                           FROM {binding}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid
                          ORDER BY roster_ordinal"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        stored.build_id.into(),
                    ],
                )
                .map_err(|error| format!("EC_EPOCH_STATE: retire binding lookup failed: {error}"))?
                .map(|row| {
                    let text = |field: &str| -> Result<String, String> {
                        row[field]
                            .value::<String>()
                            .map_err(|_| format!("EC_EPOCH_STATE: retire {field} decode failed"))?
                            .ok_or_else(|| format!("EC_EPOCH_STATE: retire {field} is NULL"))
                    };
                    Ok((
                        row["roster_ordinal"]
                            .value::<i32>()
                            .map_err(|_| "EC_EPOCH_STATE: retire ordinal decode failed".to_owned())?
                            .ok_or_else(|| "EC_EPOCH_STATE: retire ordinal is NULL".to_owned())?,
                        text("remote_index_regclass")?,
                        row["is_local"]
                            .value::<bool>()
                            .map_err(|_| {
                                "EC_EPOCH_STATE: retire locality decode failed".to_owned()
                            })?
                            .ok_or_else(|| "EC_EPOCH_STATE: retire locality is NULL".to_owned())?,
                        text("conninfo_secret_name")?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()
        })?;
        if participants.is_empty() {
            return Err("EC_EPOCH_STATE: retire decision has no participant bindings".to_owned());
        }
        let apply = extension_relation_name("ec_distann_apply_epoch_retire")?;
        for (ordinal, remote_regclass, is_local, secret_name) in participants {
            if decoded
                .abandoned_bindings
                .entries
                .iter()
                .any(|entry| entry.roster_ordinal == ordinal as u32)
            {
                continue;
            }
            if !is_local {
                let conninfo = super::node_registry::resolve_conninfo_secret(&secret_name)?;
                super::remote_transport::remote_apply_epoch_retire(
                    &conninfo,
                    &remote_regclass,
                    &stored.decision,
                    &stored.digest,
                )?;
                continue;
            }
            Spi::connect_mut(|client| {
                client
                    .update(
                        &format!("SELECT {apply}($1::text::regclass, $2::bytea, $3::bytea)"),
                        None,
                        &[
                            remote_regclass.into(),
                            stored.decision.clone().into(),
                            stored.digest.clone().into(),
                        ],
                    )
                    .map_err(|error| {
                        format!("EC_EPOCH_STATE: participant reclaim failed: {error}")
                    })?;
                Ok::<(), String>(())
            })?;
        }
        // No new scan can register after the retire decision is durable, and
        // this recovery runs only after the exact fingerprint's scan tokens
        // drained. Remove the coordinator-local FR-080 object in the same
        // transaction that marks reclaim Applied; replay tolerates its absence.
        let head_state = extension_relation_name("ec_distann_generation_head_state")?;
        Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "DELETE FROM {head_state}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        stored.build_id.into(),
                    ],
                )
                .map_err(|error| {
                    format!("EC_EPOCH_STATE: head-sample reclaim failed: {error}")
                })?;
            Ok::<(), String>(())
        })?;
        let updated = Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "UPDATE {retire}
                            SET decision_state = 'Applied', applied_at = clock_timestamp()
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND epoch_fingerprint = $3::bytea
                            AND decision_state = 'Pending'
                          RETURNING 1"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        fingerprint.to_vec().into(),
                    ],
                )
                .map(|rows| rows.len())
                .map_err(|error| format!("EC_EPOCH_STATE: retire apply update failed: {error}"))
        })?;
        if updated != 1 {
            return Err("EC_EPOCH_STATE: retire decision changed concurrently".to_owned());
        }
        Ok(())
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
}
