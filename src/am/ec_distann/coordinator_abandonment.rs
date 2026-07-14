//! Audited coordinator override for an unavailable predecessor binding.

use pgrx::datum::Uuid;
use pgrx::{pg_extern, pg_sys, PgRelation, Spi};

use super::generation_catalog::extension_relation_name;
use super::generation_store::open_control_index;
use super::lifecycle_state::{
    require_exact_transition_classified, require_transition, PredecessorDisposition,
    PublishDecisionState,
};
use super::lifecycle_wire::DistannAbandonBindingAuditV1;

fn required_bytes<const N: usize>(
    row: &pgrx::spi::SpiHeapTupleData<'_>,
    field: &str,
) -> Result<[u8; N], String> {
    row[field]
        .value::<Vec<u8>>()
        .map_err(|_| format!("EC_PREDECESSOR_ABANDON: {field} decode failed"))?
        .ok_or_else(|| format!("EC_PREDECESSOR_ABANDON: {field} is NULL"))?
        .try_into()
        .map_err(|_| format!("EC_PREDECESSOR_ABANDON: {field} must be {N} bytes"))
}

struct AbandonTarget {
    successor_epoch: i64,
    successor_fingerprint: [u8; 34],
    successor_activation_digest: [u8; 32],
    successor_state: PublishDecisionState,
    predecessor_build_id: Uuid,
    predecessor_epoch: i64,
    predecessor_fingerprint: [u8; 34],
    predecessor_manifest_digest: [u8; 32],
    node_id: i32,
    endpoint_identity: String,
    remote_index_regclass: String,
    participant_logical_index_uuid: Uuid,
    disposition: PredecessorDisposition,
    abandon_audit: Option<Vec<u8>>,
}

fn decode_target(row: pgrx::spi::SpiHeapTupleData<'_>) -> Result<AbandonTarget, String> {
    let uuid = |field: &str| -> Result<Uuid, String> {
        row[field]
            .value::<Uuid>()
            .map_err(|_| format!("EC_PREDECESSOR_ABANDON: {field} decode failed"))?
            .ok_or_else(|| format!("EC_PREDECESSOR_ABANDON: {field} is NULL"))
    };
    let i64_value = |field: &str| -> Result<i64, String> {
        row[field]
            .value::<i64>()
            .map_err(|_| format!("EC_PREDECESSOR_ABANDON: {field} decode failed"))?
            .ok_or_else(|| format!("EC_PREDECESSOR_ABANDON: {field} is NULL"))
    };
    let i32_value = |field: &str| -> Result<i32, String> {
        row[field]
            .value::<i32>()
            .map_err(|_| format!("EC_PREDECESSOR_ABANDON: {field} decode failed"))?
            .ok_or_else(|| format!("EC_PREDECESSOR_ABANDON: {field} is NULL"))
    };
    let text = |field: &str| -> Result<String, String> {
        row[field]
            .value::<String>()
            .map_err(|_| format!("EC_PREDECESSOR_ABANDON: {field} decode failed"))?
            .ok_or_else(|| format!("EC_PREDECESSOR_ABANDON: {field} is NULL"))
    };
    Ok(AbandonTarget {
        successor_epoch: i64_value("successor_epoch")?,
        successor_fingerprint: required_bytes(&row, "successor_fingerprint")?,
        successor_activation_digest: required_bytes(&row, "successor_activation_digest")?,
        successor_state: PublishDecisionState::parse(&text("decision_state")?)?,
        predecessor_build_id: uuid("predecessor_build_id")?,
        predecessor_epoch: i64_value("predecessor_epoch")?,
        predecessor_fingerprint: required_bytes(&row, "predecessor_epoch_fingerprint")?,
        predecessor_manifest_digest: required_bytes(&row, "predecessor_manifest_digest")?,
        node_id: i32_value("node_id")?,
        endpoint_identity: text("endpoint_identity")?,
        remote_index_regclass: text("remote_index_regclass")?,
        participant_logical_index_uuid: uuid("participant_logical_index_uuid")?,
        disposition: PredecessorDisposition::parse(&text("disposition")?)?,
        abandon_audit: row["abandon_audit"]
            .value::<Vec<u8>>()
            .map_err(|_| "EC_PREDECESSOR_ABANDON: existing audit decode failed".to_owned())?,
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
            .map_err(|error| {
                format!("EC_PREDECESSOR_ABANDON: audit lookup failed: {error}")
            })?
            .map(|row| {
                Ok::<_, String>((
                    row["caller_name"]
                        .value::<String>()
                        .map_err(|_| {
                            "EC_PREDECESSOR_ABANDON: caller name decode failed".to_owned()
                        })?
                        .ok_or_else(|| {
                            "EC_PREDECESSOR_ABANDON: caller name is NULL".to_owned()
                        })?,
                    row["decision_time"]
                        .value::<i64>()
                        .map_err(|_| {
                            "EC_PREDECESSOR_ABANDON: decision time decode failed".to_owned()
                        })?
                        .ok_or_else(|| {
                            "EC_PREDECESSOR_ABANDON: decision time is NULL".to_owned()
                        })?,
                ))
            })
            .next()
            .transpose()?
            .ok_or_else(|| "EC_PREDECESSOR_ABANDON: audit lookup returned no row".to_owned())
    })
}

#[pg_extern(volatile, strict)]
fn ec_distann_abandon_predecessor_binding(
    index_regclass: PgRelation,
    successor_build_id: Uuid,
    predecessor_roster_ordinal: i32,
    reason: String,
) {
    (|| -> Result<(), String> {
        super::lifecycle_guard::require_read_committed("ec_distann_abandon_predecessor_binding")?;
        let ordinal = u32::try_from(predecessor_roster_ordinal).map_err(|_| {
            "EC_PREDECESSOR_ABANDON: predecessor roster ordinal is negative".to_owned()
        })?;
        let index_oid = index_regclass.oid();
        drop(index_regclass);
        let (mut control, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_abandon_predecessor_binding",
        )?;
        control.retain_lock_until_transaction_end();

        let disposition = extension_relation_name("ec_distann_predecessor_disposition")?;
        let publish = extension_relation_name("ec_distann_publish_decision")?;
        let row = Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "SELECT successor.epoch AS successor_epoch,
                                successor.epoch_fingerprint AS successor_fingerprint,
                                successor.successor_activation_digest,
                                successor.decision_state,
                                predecessor.predecessor_build_id,
                                predecessor.predecessor_epoch,
                                predecessor.predecessor_epoch_fingerprint,
                                predecessor.predecessor_manifest_digest,
                                predecessor.node_id, predecessor.endpoint_identity,
                                predecessor.remote_index_regclass,
                                predecessor.participant_logical_index_uuid,
                                predecessor.disposition, predecessor.abandon_audit
                           FROM {disposition} predecessor
                           JOIN {publish} successor
                             ON successor.index_oid = predecessor.index_oid
                            AND successor.logical_index_uuid = predecessor.logical_index_uuid
                            AND successor.build_id = predecessor.successor_build_id
                          WHERE predecessor.index_oid = $1::oid
                            AND predecessor.logical_index_uuid = $2::uuid
                            AND predecessor.successor_build_id = $3::uuid
                            AND predecessor.predecessor_roster_ordinal = $4::integer
                          FOR UPDATE OF predecessor, successor"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        successor_build_id.into(),
                        predecessor_roster_ordinal.into(),
                    ],
                )
                .map_err(|error| format!("EC_PREDECESSOR_ABANDON: binding lookup failed: {error}"))?
                .map(decode_target)
                .next()
                .transpose()?
                .ok_or_else(|| {
                    "EC_PREDECESSOR_ABANDON: predecessor binding does not exist".to_owned()
                })
        })?;
        if row.disposition == PredecessorDisposition::Abandoned {
            let existing = row
                .abandon_audit
                .as_deref()
                .ok_or_else(|| "EC_PREDECESSOR_ABANDON: existing audit is NULL".to_owned())?;
            let audit = DistannAbandonBindingAuditV1::decode(&existing)?;
            if audit.successor_build_id == *successor_build_id.as_bytes()
                && audit.predecessor_roster_ordinal == ordinal
                && audit.reason == reason
            {
                return Ok(());
            }
            return Err("EC_PREDECESSOR_ABANDON: conflicting abandonment replay".to_owned());
        }
        if row.disposition != PredecessorDisposition::Pending {
            return Err("EC_PREDECESSOR_ABANDON: binding is already Retired".to_owned());
        }
        if row.successor_state != PublishDecisionState::Activated {
            return Err("EC_PREDECESSOR_ABANDON: successor decision is not Activated".to_owned());
        }
        let (caller_name, decision_time_unix_micros) = caller_and_time()?;
        let audit = DistannAbandonBindingAuditV1 {
            coordinator_logical_index_uuid: *logical_index_uuid.as_bytes(),
            successor_build_id: *successor_build_id.as_bytes(),
            successor_epoch: u64::try_from(row.successor_epoch)
                .map_err(|_| "EC_PREDECESSOR_ABANDON: successor epoch is invalid".to_owned())?,
            successor_fingerprint: row.successor_fingerprint,
            predecessor_build_id: *row.predecessor_build_id.as_bytes(),
            predecessor_epoch: u64::try_from(row.predecessor_epoch)
                .map_err(|_| "EC_PREDECESSOR_ABANDON: predecessor epoch is invalid".to_owned())?,
            predecessor_fingerprint: row.predecessor_fingerprint,
            predecessor_manifest_digest: row.predecessor_manifest_digest,
            predecessor_roster_ordinal: ordinal,
            node_id: u32::try_from(row.node_id)
                .map_err(|_| "EC_PREDECESSOR_ABANDON: node id is invalid".to_owned())?,
            participant_logical_index_uuid: *row.participant_logical_index_uuid.as_bytes(),
            endpoint_identity: row.endpoint_identity,
            remote_index_regclass: row.remote_index_regclass,
            successor_activation_digest: row.successor_activation_digest,
            decision_time_unix_micros,
            caller_name: caller_name.clone(),
            reason: reason.clone(),
        };
        let audit_bytes = audit.encode()?;
        let audit_digest = audit.digest()?;
        let updated = Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "UPDATE {disposition}
                            SET disposition = 'Abandoned', abandon_audit = $5::bytea,
                                abandon_audit_digest = $6::bytea,
                                abandon_caller_name = $7::text, abandon_reason = $8::text,
                                abandon_time_unix_micros = $9::bigint,
                                updated_at = clock_timestamp()
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND successor_build_id = $3::uuid
                            AND predecessor_roster_ordinal = $4::integer
                            AND disposition = 'Pending'
                          RETURNING 1"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        successor_build_id.into(),
                        predecessor_roster_ordinal.into(),
                        audit_bytes.into(),
                        audit_digest.to_vec().into(),
                        caller_name.into(),
                        reason.into(),
                        decision_time_unix_micros.into(),
                    ],
                )
                .map(|rows| rows.len())
                .map_err(|error| format!("EC_PREDECESSOR_ABANDON: binding update failed: {error}"))
        })?;
        require_exact_transition_classified(
            PredecessorDisposition::Pending,
            PredecessorDisposition::Abandoned,
            updated,
            "predecessor disposition",
            "EC_PREDECESSOR_ABANDON",
        )?;
        require_transition(
            PublishDecisionState::Activated,
            PublishDecisionState::Applied,
            "publish decision",
        )?;
        Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "UPDATE {publish} successor
                            SET decision_state = 'Applied', applied_at = clock_timestamp()
                          WHERE successor.index_oid = $1::oid
                            AND successor.logical_index_uuid = $2::uuid
                            AND successor.build_id = $3::uuid
                            AND successor.decision_state = 'Activated'
                            AND NOT EXISTS (
                                SELECT 1 FROM {disposition} predecessor
                                 WHERE predecessor.index_oid = successor.index_oid
                                   AND predecessor.logical_index_uuid = successor.logical_index_uuid
                                   AND predecessor.successor_build_id = successor.build_id
                                   AND predecessor.disposition = 'Pending'
                            )"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        successor_build_id.into(),
                    ],
                )
                .map_err(|error| {
                    format!("EC_PREDECESSOR_ABANDON: successor apply failed: {error}")
                })?;
            Ok::<(), String>(())
        })
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
}
