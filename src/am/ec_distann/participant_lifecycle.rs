//! FR-082 participant-side generation publication and retirement.
//!
//! These endpoints are deliberately local: authenticated transport and the
//! coordinator decision/recovery state machine call them, but do not live in
//! this module.  Every mutation is scoped by the live control relation OID and
//! its persisted logical UUID, and every replay compares the complete
//! canonical identity before returning success.

use pgrx::datum::Uuid;
use pgrx::iter::TableIterator;
use pgrx::{name, pg_extern, pg_sys, PgRelation, Spi};

use super::canonical_wire::{domain_digest, fixed_digest, is_rfc4122_v4_uuid, CanonicalEncoder};
use super::generation_catalog::{self, GenerationCatalogRow};
use super::generation_descriptor::{encode_roster, roster_digest, DistannGenerationDescriptor};
use super::generation_store::{drop_generation_relations, open_control_index, GenerationRelations};
use super::lifecycle_state::{require_exact_transition, GenerationState};
use super::lifecycle_wire::{
    DistannCancelPublishAuditV1, DistannRetireDecisionV1, DistannSuccessorActivationV1,
};
use super::manifest_v2::{
    DistannEpochFingerprint, DistannEpochManifestV2, DistannReadyReceipt,
    DISTANN_EPOCH_FINGERPRINT_BYTES,
};

const MANIFEST_DOMAIN: &[u8] = b"ec_distann_epoch_manifest_v2\0";
const SUCCESSOR_ACTIVATION_DOMAIN: &[u8] = b"ec_distann_successor_activation_v1\0";
const RETIRE_DECISION_DOMAIN: &[u8] = b"ec_distann_retire_decision_v1\0";

type GenerationStatusRow = (
    i64,
    String,
    Vec<u8>,
    Vec<u8>,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
    i64,
    i64,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
);

fn validate_build_id(build_id: Uuid) -> Result<(), String> {
    if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
        return Err("EC_EPOCH_STATE: build id must be an RFC 4122 v4 UUID".to_owned());
    }
    Ok(())
}

fn decode_manifest(
    bytes: &[u8],
    supplied_digest: [u8; 32],
) -> Result<DistannEpochManifestV2, String> {
    if domain_digest(MANIFEST_DOMAIN, bytes) != supplied_digest {
        return Err("EC_PUBLISH_DIGEST: epoch manifest digest mismatch".to_owned());
    }
    let manifest = DistannEpochManifestV2::decode(bytes)?;
    if manifest.digest()? != supplied_digest {
        return Err("EC_PUBLISH_DIGEST: non-canonical epoch manifest digest".to_owned());
    }
    Ok(manifest)
}

fn decode_activation(
    bytes: &[u8],
    supplied_digest: [u8; 32],
) -> Result<DistannSuccessorActivationV1, String> {
    if domain_digest(SUCCESSOR_ACTIVATION_DOMAIN, bytes) != supplied_digest {
        return Err("EC_PUBLISH_DIGEST: successor activation digest mismatch".to_owned());
    }
    let activation = DistannSuccessorActivationV1::decode(bytes)?;
    if activation.digest()? != supplied_digest {
        return Err("EC_PUBLISH_DIGEST: non-canonical successor activation".to_owned());
    }
    Ok(activation)
}

fn decode_retire_decision(
    bytes: &[u8],
    supplied_digest: [u8; 32],
) -> Result<DistannRetireDecisionV1, String> {
    if domain_digest(RETIRE_DECISION_DOMAIN, bytes) != supplied_digest {
        return Err("EC_PUBLISH_DIGEST: retire decision digest mismatch".to_owned());
    }
    let decision = DistannRetireDecisionV1::decode(bytes)?;
    if decision.digest()? != supplied_digest {
        return Err("EC_PUBLISH_DIGEST: non-canonical retire decision".to_owned());
    }
    Ok(decision)
}

fn decode_generation_descriptor(
    row: &GenerationCatalogRow,
) -> Result<DistannGenerationDescriptor, String> {
    let descriptor = DistannGenerationDescriptor::decode(&row.generation_descriptor)?;
    if descriptor.digest()? != row.generation_descriptor_digest {
        return Err("EC_PUBLISH_DIGEST: stored generation descriptor digest mismatch".to_owned());
    }
    if roster_digest(&descriptor.roster)? != row.roster_digest {
        return Err("EC_PUBLISH_DIGEST: stored generation roster digest mismatch".to_owned());
    }
    Ok(descriptor)
}

fn local_ready_receipt(
    row: &GenerationCatalogRow,
    descriptor: &DistannGenerationDescriptor,
    manifest: &DistannEpochManifestV2,
    local_logical_index_uuid: Uuid,
) -> Result<DistannReadyReceipt, String> {
    if manifest.epoch != row.epoch
        || manifest.build_spec_digest != row.build_spec_digest
        || manifest.generation_descriptor_digest != row.generation_descriptor_digest
        || manifest.roster != descriptor.roster
    {
        return Err(
            "EC_PUBLISH_DIGEST: manifest disagrees with local generation identity".to_owned(),
        );
    }
    manifest
        .codec_parameters
        .validate_artifact(&descriptor.codec_artifact)
        .map_err(|_| {
            "EC_PUBLISH_DIGEST: manifest codec parameters disagree with local codec artifact"
                .to_owned()
        })?;
    if manifest.row_schema_fingerprint != descriptor.row_schema.fingerprint()? {
        return Err("EC_PUBLISH_DIGEST: manifest row schema disagrees with descriptor".to_owned());
    }
    let descriptor_cover_digest = descriptor
        .payload_cover()
        .map(|cover| cover.digest())
        .transpose()?;
    if manifest.payload_cover_descriptor_digest != descriptor_cover_digest {
        return Err(
            "EC_PUBLISH_DIGEST: manifest payload cover disagrees with descriptor".to_owned(),
        );
    }

    let ordinal = usize::try_from(row.owner_ordinal)
        .map_err(|_| "EC_PUBLISH_INCOMPLETE: local owner ordinal is invalid".to_owned())?;
    let roster_entry = descriptor
        .roster
        .get(ordinal)
        .ok_or_else(|| "EC_PUBLISH_INCOMPLETE: local owner ordinal is outside roster".to_owned())?;
    let receipt = manifest
        .participant_receipts
        .get(ordinal)
        .ok_or_else(|| "EC_PUBLISH_INCOMPLETE: local Ready receipt is absent".to_owned())?;
    if roster_entry.logical_index_uuid != *local_logical_index_uuid.as_bytes()
        || roster_entry.node_id != row.node_id
        || receipt.node_id != row.node_id
        || receipt.epoch != row.epoch
        || receipt.build_spec_digest != row.build_spec_digest
        || receipt.generation_descriptor_digest != row.generation_descriptor_digest
        || receipt.owned_record_count != row.expected_owner_count
        || receipt.owned_record_count != row.cumulative_record_count
        || receipt.owner_stream_digest != row.expected_owner_digest
        || receipt.owner_stream_digest != row.cumulative_owner_digest
    {
        return Err(
            "EC_PUBLISH_INCOMPLETE: local roster/receipt/topology proof disagrees".to_owned(),
        );
    }
    let stored = row
        .ready_receipt
        .as_ref()
        .ok_or_else(|| "EC_PUBLISH_INCOMPLETE: local Ready receipt is not durable".to_owned())?;
    if receipt.encode()?.as_slice() != stored.as_slice() {
        return Err(
            "EC_PUBLISH_DIGEST: manifest does not contain the exact local Ready receipt".to_owned(),
        );
    }
    Ok(receipt.clone())
}

/// Participant Ready -> Published transition.  The 34-byte fingerprint is the
/// acknowledgement identity returned to the coordinator.
#[pg_extern(
    name = "ec_distann_publish_epoch",
    volatile,
    strict,
    parallel_restricted
)]
fn ec_distann_publish_generation(
    index_regclass: PgRelation,
    build_id: Uuid,
    epoch_manifest: Vec<u8>,
    manifest_digest: Vec<u8>,
) -> Vec<u8> {
    (|| -> Result<Vec<u8>, String> {
        super::lifecycle_guard::require_read_committed("ec_distann_publish_epoch")?;
        validate_build_id(build_id)?;
        let manifest_digest =
            fixed_digest(manifest_digest, "EC_PUBLISH_DIGEST", "manifest digest")?;
        let manifest = decode_manifest(&epoch_manifest, manifest_digest)?;
        if manifest.build_id != *build_id.as_bytes() {
            return Err(
                "EC_EPOCH_STATE: manifest build id differs from requested build".to_owned(),
            );
        }
        let fingerprint = manifest.fingerprint()?;
        let index_oid = index_regclass.oid();
        let (_guard, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_publish_epoch",
        )?;
        let row = generation_catalog::lookup_generation_for_update(
            index_oid,
            logical_index_uuid,
            build_id,
        )?
        .ok_or_else(|| "EC_GENERATION_MISSING: publication target does not exist".to_owned())?;
        let descriptor = decode_generation_descriptor(&row)?;
        local_ready_receipt(&row, &descriptor, &manifest, logical_index_uuid)?;

        if row.state == GenerationState::Published {
            let generation = generation_catalog::extension_relation_name("ec_distann_generation")?;
            let exact = Spi::get_one_with_args::<bool>(
                &format!(
                    "SELECT epoch_fingerprint = $4::bytea
                            AND manifest_digest = $5::bytea
                            AND epoch_manifest = $6::bytea
                       FROM {generation}
                      WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                        AND build_id = $3::uuid"
                ),
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    build_id.into(),
                    fingerprint.as_bytes().to_vec().into(),
                    manifest_digest.to_vec().into(),
                    epoch_manifest.clone().into(),
                ],
            )
            .map_err(|error| format!("EC_EPOCH_STATE: publication replay lookup failed: {error}"))?
            .unwrap_or(false);
            if exact {
                return Ok(fingerprint.as_bytes().to_vec());
            }
            return Err("EC_EPOCH_STATE: conflicting publication replay".to_owned());
        }
        if row.state != GenerationState::Ready {
            return Err(format!(
                "EC_EPOCH_STATE: generation state {} cannot transition to Published",
                row.state
            ));
        }
        let generation = generation_catalog::extension_relation_name("ec_distann_generation")?;
        let sql = format!(
            "UPDATE {generation}
                SET state = 'Published', epoch_fingerprint = $4::bytea,
                    manifest_digest = $5::bytea, epoch_manifest = $6::bytea,
                    published_at = clock_timestamp(), updated_at = clock_timestamp()
              WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                AND build_id = $3::uuid AND state = 'Ready'
                AND epoch = $7::bigint AND build_spec_digest = $8::bytea
                AND generation_descriptor_digest = $9::bytea
              RETURNING 1"
        );
        let epoch = i64::try_from(manifest.epoch)
            .map_err(|_| "EC_EPOCH_STATE: manifest epoch exceeds bigint".to_owned())?;
        let updated = Spi::connect_mut(|client| {
            client
                .update(
                    &sql,
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        build_id.into(),
                        fingerprint.as_bytes().to_vec().into(),
                        manifest_digest.to_vec().into(),
                        epoch_manifest.into(),
                        epoch.into(),
                        row.build_spec_digest.to_vec().into(),
                        row.generation_descriptor_digest.to_vec().into(),
                    ],
                )
                .map(|table| table.len())
                .map_err(|error| format!("EC_EPOCH_STATE: publication transition failed: {error}"))
        })?;
        require_exact_transition(
            GenerationState::Ready,
            GenerationState::Published,
            updated,
            "generation",
        )?;
        Ok(fingerprint.as_bytes().to_vec())
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"))
}

fn optional_bytes(
    row: &pgrx::spi::SpiHeapTupleData<'_>,
    field: &str,
) -> Result<Option<Vec<u8>>, String> {
    row[field]
        .value::<Vec<u8>>()
        .map_err(|error| format!("EC_EPOCH_STATE: status {field} decode failed: {error}"))
}

/// Durable participant status, including the Reclaimed tombstone after every
/// generation relation and the live generation row have been removed.
#[pg_extern(stable, strict, parallel_restricted)]
fn ec_distann_epoch_generation_status(
    index_regclass: PgRelation,
    build_id: Uuid,
) -> TableIterator<
    'static,
    (
        name!(epoch, i64),
        name!(state, String),
        name!(build_spec_digest, Vec<u8>),
        name!(generation_descriptor_digest, Vec<u8>),
        name!(epoch_fingerprint, Option<Vec<u8>>),
        name!(manifest_digest, Option<Vec<u8>>),
        name!(record_count, i64),
        name!(row_count, i64),
        name!(successor_activation_digest, Option<Vec<u8>>),
        name!(retire_decision_digest, Option<Vec<u8>>),
        name!(cancellation_audit_digest, Option<Vec<u8>>),
    ),
> {
    let result = (|| -> Result<GenerationStatusRow, String> {
        validate_build_id(build_id)?;
        let index_oid = index_regclass.oid();
        let (_guard, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            "ec_distann_epoch_generation_status",
        )?;
        let generation = generation_catalog::extension_relation_name("ec_distann_generation")?;
        let reclaim = generation_catalog::extension_relation_name("ec_distann_generation_reclaim")?;
        let cancelled_reclaim =
            generation_catalog::extension_relation_name("ec_distann_cancelled_generation_reclaim")?;
        let sql = format!(
            "SELECT epoch, state, build_spec_digest, generation_descriptor_digest,
                    epoch_fingerprint, manifest_digest, record_count, row_count,
                    successor_activation_digest, retire_decision_digest,
                    cancellation_audit_digest
               FROM (
                   SELECT epoch, state, build_spec_digest,
                          generation_descriptor_digest, epoch_fingerprint,
                          manifest_digest, cumulative_record_count AS record_count,
                          cumulative_record_count AS row_count,
                          successor_activation_digest, NULL::bytea AS retire_decision_digest,
                          NULL::bytea AS cancellation_audit_digest
                     FROM {generation}
                    WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                      AND build_id = $3::uuid
                   UNION ALL
                   SELECT epoch, 'Reclaimed'::text, build_spec_digest,
                          generation_descriptor_digest, epoch_fingerprint,
                          manifest_digest, record_count, row_count,
                          successor_activation_digest, retire_decision_digest,
                          NULL::bytea AS cancellation_audit_digest
                     FROM {reclaim}
                    WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                      AND build_id = $3::uuid
                   UNION ALL
                   SELECT epoch, 'CancelledReclaimed'::text, build_spec_digest,
                          generation_descriptor_digest, epoch_fingerprint,
                          manifest_digest, record_count, row_count,
                          NULL::bytea AS successor_activation_digest,
                          NULL::bytea AS retire_decision_digest,
                          cancellation_audit_digest
                     FROM {cancelled_reclaim}
                    WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                      AND build_id = $3::uuid
               ) status"
        );
        Spi::connect(|client| {
            let mut rows = client
                .select(
                    &sql,
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| format!("EC_EPOCH_STATE: generation status failed: {error}"))?;
            let row = rows
                .next()
                .ok_or_else(|| "EC_GENERATION_MISSING: generation status is unknown".to_owned())?;
            if rows.next().is_some() {
                return Err(
                    "EC_EPOCH_STATE: live generation and reclaim tombstone overlap".to_owned(),
                );
            }
            let required_i64 = |field: &str| {
                row[field]
                    .value::<i64>()
                    .map_err(|error| {
                        format!("EC_EPOCH_STATE: status {field} decode failed: {error}")
                    })?
                    .ok_or_else(|| format!("EC_EPOCH_STATE: status {field} is NULL"))
            };
            let required_string = |field: &str| {
                row[field]
                    .value::<String>()
                    .map_err(|error| {
                        format!("EC_EPOCH_STATE: status {field} decode failed: {error}")
                    })?
                    .ok_or_else(|| format!("EC_EPOCH_STATE: status {field} is NULL"))
            };
            let required_bytes = |field: &str| {
                optional_bytes(&row, field)?
                    .ok_or_else(|| format!("EC_EPOCH_STATE: status {field} is NULL"))
            };
            Ok((
                required_i64("epoch")?,
                required_string("state")?,
                required_bytes("build_spec_digest")?,
                required_bytes("generation_descriptor_digest")?,
                optional_bytes(&row, "epoch_fingerprint")?,
                optional_bytes(&row, "manifest_digest")?,
                required_i64("record_count")?,
                required_i64("row_count")?,
                optional_bytes(&row, "successor_activation_digest")?,
                optional_bytes(&row, "retire_decision_digest")?,
                optional_bytes(&row, "cancellation_audit_digest")?,
            ))
        })
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
    TableIterator::once(result)
}

#[pg_extern(volatile, strict, parallel_restricted)]
fn ec_distann_mark_epoch_retired(
    index_regclass: PgRelation,
    successor_activation: Vec<u8>,
    successor_activation_digest: Vec<u8>,
) {
    (|| -> Result<(), String> {
        super::lifecycle_guard::require_read_committed("ec_distann_mark_epoch_retired")?;
        let supplied_digest = fixed_digest(
            successor_activation_digest,
            "EC_PUBLISH_DIGEST",
            "successor activation digest",
        )?;
        let activation = decode_activation(&successor_activation, supplied_digest)?;
        let predecessor = activation
            .predecessor
            .ok_or_else(|| "EC_EPOCH_STATE: retirement marker has no predecessor".to_owned())?;
        if activation.successor.epoch <= predecessor.epoch {
            return Err("EC_EPOCH_STATE: successor epoch does not advance predecessor".to_owned());
        }
        let build_id = Uuid::from_bytes(predecessor.build_id);
        let index_oid = index_regclass.oid();
        let (_guard, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_mark_epoch_retired",
        )?;
        let row = generation_catalog::lookup_generation_for_update(
            index_oid,
            logical_index_uuid,
            build_id,
        )?
        .ok_or_else(|| "EC_GENERATION_MISSING: retirement predecessor does not exist".to_owned())?;
        let descriptor = decode_generation_descriptor(&row)?;
        if activation.coordinator_logical_index_uuid != descriptor.coordinator_logical_index_uuid
            || row.epoch != predecessor.epoch
        {
            return Err(
                "EC_EPOCH_STATE: retirement marker coordinator/predecessor mismatch".to_owned(),
            );
        }
        let generation = generation_catalog::extension_relation_name("ec_distann_generation")?;
        let persisted = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT epoch_fingerprint, manifest_digest,
                                successor_activation, successor_activation_digest
                           FROM {generation}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| {
                    format!("EC_EPOCH_STATE: retirement identity lookup failed: {error}")
                })?
                .map(|catalog_row| {
                    Ok::<_, String>((
                        optional_bytes(&catalog_row, "epoch_fingerprint")?,
                        optional_bytes(&catalog_row, "manifest_digest")?,
                        optional_bytes(&catalog_row, "successor_activation")?,
                        optional_bytes(&catalog_row, "successor_activation_digest")?,
                    ))
                })
                .next()
                .transpose()?
                .ok_or_else(|| {
                    "EC_EPOCH_STATE: locked retirement generation disappeared".to_owned()
                })
        })?;
        if persisted.0.as_deref() != Some(predecessor.fingerprint.as_slice())
            || persisted.1.as_deref() != Some(predecessor.manifest_digest.as_slice())
        {
            return Err(
                "EC_EPOCH_STATE: retirement marker does not name exact Published predecessor"
                    .to_owned(),
            );
        }
        if row.state == GenerationState::Retired {
            if persisted.2.as_deref() == Some(successor_activation.as_slice())
                && persisted.3.as_deref() == Some(supplied_digest.as_slice())
            {
                return Ok(());
            }
            return Err("EC_EPOCH_STATE: conflicting successor activation replay".to_owned());
        }
        if row.state != GenerationState::Published {
            return Err(format!(
                "EC_EPOCH_STATE: generation state {} cannot transition to Retired",
                row.state
            ));
        }
        let updated = Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "UPDATE {generation}
                            SET state = 'Retired', successor_activation = $4::bytea,
                                successor_activation_digest = $5::bytea,
                                retired_at = clock_timestamp(), updated_at = clock_timestamp()
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid AND state = 'Published'
                            AND epoch_fingerprint = $6::bytea
                            AND manifest_digest = $7::bytea
                          RETURNING 1"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        build_id.into(),
                        successor_activation.into(),
                        supplied_digest.to_vec().into(),
                        predecessor.fingerprint.to_vec().into(),
                        predecessor.manifest_digest.to_vec().into(),
                    ],
                )
                .map(|table| table.len())
                .map_err(|error| format!("EC_EPOCH_STATE: retirement mark failed: {error}"))
        })?;
        require_exact_transition(
            GenerationState::Published,
            GenerationState::Retired,
            updated,
            "generation",
        )
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
}

#[derive(Debug)]
struct ReclaimReplay {
    logical_index_uuid: Uuid,
    build_id: Uuid,
    epoch: i64,
    fingerprint: Vec<u8>,
    manifest_digest: Vec<u8>,
    retire_decision: Vec<u8>,
    retire_decision_digest: Vec<u8>,
}

fn lookup_reclaim_for_update(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    fingerprint: &[u8; DISTANN_EPOCH_FINGERPRINT_BYTES],
) -> Result<Option<ReclaimReplay>, String> {
    let reclaim = generation_catalog::extension_relation_name("ec_distann_generation_reclaim")?;
    Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "SELECT logical_index_uuid, build_id, epoch, epoch_fingerprint,
                            manifest_digest, retire_decision, retire_decision_digest
                       FROM {reclaim}
                      WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                        AND (build_id = $3::uuid OR epoch_fingerprint = $4::bytea)
                      FOR UPDATE"
                ),
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    build_id.into(),
                    fingerprint.to_vec().into(),
                ],
            )
            .map_err(|error| format!("EC_EPOCH_STATE: reclaim replay lookup failed: {error}"))?
            .map(|row| {
                let required_bytes = |field: &str| {
                    optional_bytes(&row, field)?
                        .ok_or_else(|| format!("EC_EPOCH_STATE: reclaim {field} is NULL"))
                };
                Ok(ReclaimReplay {
                    logical_index_uuid: row["logical_index_uuid"]
                        .value::<Uuid>()
                        .map_err(|error| {
                            format!("EC_EPOCH_STATE: reclaim UUID decode failed: {error}")
                        })?
                        .ok_or_else(|| "EC_EPOCH_STATE: reclaim UUID is NULL".to_owned())?,
                    build_id: row["build_id"]
                        .value::<Uuid>()
                        .map_err(|error| {
                            format!("EC_EPOCH_STATE: reclaim build decode failed: {error}")
                        })?
                        .ok_or_else(|| "EC_EPOCH_STATE: reclaim build is NULL".to_owned())?,
                    epoch: row["epoch"]
                        .value::<i64>()
                        .map_err(|error| {
                            format!("EC_EPOCH_STATE: reclaim epoch decode failed: {error}")
                        })?
                        .ok_or_else(|| "EC_EPOCH_STATE: reclaim epoch is NULL".to_owned())?,
                    fingerprint: required_bytes("epoch_fingerprint")?,
                    manifest_digest: required_bytes("manifest_digest")?,
                    retire_decision: required_bytes("retire_decision")?,
                    retire_decision_digest: required_bytes("retire_decision_digest")?,
                })
            })
            .next()
            .transpose()
    })
}

fn canonical_roster_bytes(descriptor: &DistannGenerationDescriptor) -> Result<Vec<u8>, String> {
    let mut encoder = CanonicalEncoder::with_capacity(4 + descriptor.roster.len() * 32);
    encode_roster(&mut encoder, &descriptor.roster)?;
    encoder.finish()
}

#[pg_extern(volatile, strict, parallel_restricted)]
fn ec_distann_apply_epoch_retire(
    index_regclass: PgRelation,
    retire_decision: Vec<u8>,
    retire_decision_digest: Vec<u8>,
) {
    (|| -> Result<(), String> {
        super::lifecycle_guard::require_read_committed("ec_distann_apply_epoch_retire")?;
        let supplied_digest = fixed_digest(
            retire_decision_digest,
            "EC_PUBLISH_DIGEST",
            "retire decision digest",
        )?;
        let decision = decode_retire_decision(&retire_decision, supplied_digest)?;
        let build_id = Uuid::from_bytes(decision.target_build_id);
        let index_oid = index_regclass.oid();
        let (_guard, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_apply_epoch_retire",
        )?;

        if let Some(existing) = lookup_reclaim_for_update(
            index_oid,
            logical_index_uuid,
            build_id,
            &decision.target_fingerprint,
        )? {
            let epoch = i64::try_from(decision.epoch)
                .map_err(|_| "EC_EPOCH_STATE: retire target epoch exceeds bigint".to_owned())?;
            if existing.logical_index_uuid == logical_index_uuid
                && existing.build_id == build_id
                && existing.epoch == epoch
                && existing.fingerprint == decision.target_fingerprint
                && existing.manifest_digest == decision.target_manifest_digest
                && existing.retire_decision == retire_decision
                && existing.retire_decision_digest == supplied_digest
            {
                return Ok(());
            }
            return Err("EC_EPOCH_STATE: conflicting reclaim tombstone replay".to_owned());
        }

        let row = generation_catalog::lookup_generation_for_update(
            index_oid,
            logical_index_uuid,
            build_id,
        )?
        .ok_or_else(|| "EC_GENERATION_MISSING: retire target does not exist".to_owned())?;
        if row.state != GenerationState::Retired {
            return Err(format!(
                "EC_EPOCH_STATE: generation state {} cannot be reclaimed",
                row.state
            ));
        }
        let descriptor = decode_generation_descriptor(&row)?;
        let epoch = i64::try_from(decision.epoch)
            .map_err(|_| "EC_EPOCH_STATE: retire target epoch exceeds bigint".to_owned())?;
        if decision.coordinator_logical_index_uuid != descriptor.coordinator_logical_index_uuid
            || decision.epoch != row.epoch
            || decision.target_manifest_digest
                != DistannEpochFingerprint::decode(&decision.target_fingerprint)?.manifest_digest()
            || decision.roster_digest != row.roster_digest
            || decision.target_roster_snapshot != canonical_roster_bytes(&descriptor)?
        {
            return Err(
                "EC_EPOCH_STATE: retire decision disagrees with local generation identity"
                    .to_owned(),
            );
        }
        let ordinal = usize::try_from(row.owner_ordinal)
            .map_err(|_| "EC_EPOCH_STATE: local owner ordinal is invalid".to_owned())?;
        let member = descriptor.roster.get(ordinal).ok_or_else(|| {
            "EC_EPOCH_STATE: local owner ordinal is outside retire roster".to_owned()
        })?;
        if member.logical_index_uuid != *logical_index_uuid.as_bytes()
            || member.node_id != row.node_id
            || decision
                .abandoned_bindings
                .entries
                .iter()
                .any(|entry| entry.roster_ordinal as usize == ordinal)
        {
            return Err(
                "EC_EPOCH_STATE: participant is absent or abandoned in retire decision".to_owned(),
            );
        }
        let generation = generation_catalog::extension_relation_name("ec_distann_generation")?;
        let persisted = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT epoch_fingerprint, manifest_digest,
                                successor_activation_digest
                           FROM {generation}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| format!("EC_EPOCH_STATE: retire identity lookup failed: {error}"))?
                .map(|catalog_row| {
                    Ok::<_, String>((
                        optional_bytes(&catalog_row, "epoch_fingerprint")?,
                        optional_bytes(&catalog_row, "manifest_digest")?,
                        optional_bytes(&catalog_row, "successor_activation_digest")?,
                    ))
                })
                .next()
                .transpose()?
                .ok_or_else(|| "EC_EPOCH_STATE: locked retire generation disappeared".to_owned())
        })?;
        if persisted.0.as_deref() != Some(decision.target_fingerprint.as_slice())
            || persisted.1.as_deref() != Some(decision.target_manifest_digest.as_slice())
        {
            return Err(
                "EC_EPOCH_STATE: retire decision does not name exact Retired generation".to_owned(),
            );
        }
        let ready_receipt =
            DistannReadyReceipt::decode(row.ready_receipt.as_ref().ok_or_else(|| {
                "EC_EPOCH_STATE: Retired generation has no Ready receipt".to_owned()
            })?)?;
        if ready_receipt.node_id != row.node_id
            || ready_receipt.epoch != row.epoch
            || ready_receipt.build_id != *build_id.as_bytes()
            || ready_receipt.build_spec_digest != row.build_spec_digest
            || ready_receipt.generation_descriptor_digest != row.generation_descriptor_digest
            || ready_receipt.owned_record_count != row.expected_owner_count
            || ready_receipt.owned_record_count != row.cumulative_record_count
            || ready_receipt.row_count != row.cumulative_record_count
            || ready_receipt.owner_stream_digest != row.expected_owner_digest
            || ready_receipt.owner_stream_digest != row.cumulative_owner_digest
        {
            return Err(
                "EC_EPOCH_STATE: stored Ready receipt disagrees with Retired generation".to_owned(),
            );
        }
        let reclaim = generation_catalog::extension_relation_name("ec_distann_generation_reclaim")?;
        // Insert the immutable success evidence before physical deletion.  All
        // following DDL/catalog mutations are transactional, so an error rolls
        // this insert and every relation deletion back together.
        Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "INSERT INTO {reclaim} (
                             index_oid, logical_index_uuid, build_id, epoch,
                             epoch_fingerprint, manifest_digest, build_spec_digest,
                             generation_descriptor_digest, record_count, row_count,
                             successor_activation_digest, retire_decision,
                             retire_decision_digest
                         ) VALUES (
                             $1::oid, $2::uuid, $3::uuid, $4::bigint,
                             $5::bytea, $6::bytea, $7::bytea, $8::bytea,
                             $9::bigint, $10::bigint, $11::bytea, $12::bytea,
                             $13::bytea
                         )"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        build_id.into(),
                        epoch.into(),
                        decision.target_fingerprint.to_vec().into(),
                        decision.target_manifest_digest.to_vec().into(),
                        row.build_spec_digest.to_vec().into(),
                        row.generation_descriptor_digest.to_vec().into(),
                        i64::try_from(ready_receipt.owned_record_count)
                            .map_err(|_| "EC_EPOCH_STATE: record count exceeds bigint".to_owned())?
                            .into(),
                        i64::try_from(ready_receipt.row_count)
                            .map_err(|_| "EC_EPOCH_STATE: row count exceeds bigint".to_owned())?
                            .into(),
                        persisted.2.clone().into(),
                        retire_decision.clone().into(),
                        supplied_digest.to_vec().into(),
                    ],
                )
                .map_err(|error| {
                    format!("EC_EPOCH_STATE: reclaim tombstone insert failed: {error}")
                })?;
            Ok::<(), String>(())
        })?;
        // The retire decision's canonical bytes and abandoned-binding set are
        // already digest-verified against the supplied digest in
        // decode_retire_decision (raw-byte domain digest plus a re-encode
        // round-trip), so no further self-comparison of the abandoned set is
        // meaningful here.
        drop_generation_relations(
            index_oid,
            GenerationRelations {
                row_tier_relid: row.row_tier_relid,
                cold_tier_relid: row.cold_tier_relid,
                graph_store_relid: row.graph_store_relid,
                node_store_relid: row.node_store_relid,
                directory_relid: row.directory_relid,
                payload_sidecar_relid: row.payload_sidecar_relid,
                payload_sidecar_directory_relid: row.payload_sidecar_directory_relid,
            },
        )?;
        let deleted = Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "DELETE FROM {generation}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid AND state = 'Retired'
                            AND epoch_fingerprint = $4::bytea
                            AND manifest_digest = $5::bytea
                          RETURNING 1"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        build_id.into(),
                        decision.target_fingerprint.to_vec().into(),
                        decision.target_manifest_digest.to_vec().into(),
                    ],
                )
                .map(|table| table.len())
                .map_err(|error| {
                    format!("EC_EPOCH_STATE: reclaimed generation delete failed: {error}")
                })
        })?;
        if deleted != 1 {
            return Err("EC_EPOCH_STATE: reclaimed generation changed concurrently".to_owned());
        }
        Ok(())
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
}

/// Explicitly reclaim a Ready or partially Published generation whose
/// coordinator publish decision was durably cancelled before activation.
/// The canonical cancellation audit is the authenticated authorization; an
/// immutable local tombstone makes partial remote cleanup replayable.
#[pg_extern(volatile, strict, parallel_restricted)]
fn ec_distann_reclaim_cancelled_generation(
    index_regclass: PgRelation,
    cancellation_audit: Vec<u8>,
    cancellation_audit_digest: Vec<u8>,
) {
    (|| -> Result<(), String> {
        super::lifecycle_guard::require_read_committed("ec_distann_reclaim_cancelled_generation")?;
        let supplied_digest = fixed_digest(
            cancellation_audit_digest,
            "EC_PUBLISH_CANCEL",
            "cancellation audit digest",
        )?;
        let audit = DistannCancelPublishAuditV1::decode(&cancellation_audit)?;
        if audit.digest()? != supplied_digest {
            return Err("EC_PUBLISH_CANCEL: cancellation audit digest mismatch".to_owned());
        }
        let build_id = Uuid::from_bytes(audit.build_id);
        let index_oid = index_regclass.oid();
        let (_guard, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_reclaim_cancelled_generation",
        )?;

        let tombstone =
            generation_catalog::extension_relation_name("ec_distann_cancelled_generation_reclaim")?;
        let replay = Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "SELECT epoch, epoch_fingerprint, manifest_digest,
                                cancellation_audit, cancellation_audit_digest
                           FROM {tombstone}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid
                          FOR UPDATE"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| {
                    format!("EC_PUBLISH_CANCEL: cancellation tombstone lookup failed: {error}")
                })?
                .map(|row| {
                    let required = |field: &str| -> Result<Vec<u8>, String> {
                        row[field]
                            .value::<Vec<u8>>()
                            .map_err(|_| {
                                format!("EC_PUBLISH_CANCEL: tombstone {field} decode failed")
                            })?
                            .ok_or_else(|| format!("EC_PUBLISH_CANCEL: tombstone {field} is NULL"))
                    };
                    Ok::<_, String>((
                        row["epoch"]
                            .value::<i64>()
                            .map_err(|_| {
                                "EC_PUBLISH_CANCEL: tombstone epoch decode failed".to_owned()
                            })?
                            .ok_or_else(|| {
                                "EC_PUBLISH_CANCEL: tombstone epoch is NULL".to_owned()
                            })?,
                        required("epoch_fingerprint")?,
                        required("manifest_digest")?,
                        required("cancellation_audit")?,
                        required("cancellation_audit_digest")?,
                    ))
                })
                .next()
                .transpose()
        })?;
        if let Some((epoch, fingerprint, manifest_digest, stored_audit, stored_digest)) = replay {
            if epoch == i64::try_from(audit.epoch).unwrap_or(i64::MIN)
                && fingerprint == audit.epoch_fingerprint
                && manifest_digest == audit.manifest_digest
                && stored_audit == cancellation_audit
                && stored_digest == supplied_digest
            {
                return Ok(());
            }
            return Err("EC_PUBLISH_CANCEL: conflicting cancellation reclaim replay".to_owned());
        }

        let row = generation_catalog::lookup_generation_for_update(
            index_oid,
            logical_index_uuid,
            build_id,
        )?
        .ok_or_else(|| {
            "EC_GENERATION_MISSING: cancelled generation and tombstone are absent".to_owned()
        })?;
        if !matches!(
            row.state,
            GenerationState::Ready | GenerationState::Published
        ) {
            return Err(format!(
                "EC_PUBLISH_CANCEL: generation state {} cannot be cancellation-reclaimed",
                row.state
            ));
        }
        let descriptor = decode_generation_descriptor(&row)?;
        if descriptor.coordinator_logical_index_uuid != audit.coordinator_logical_index_uuid
            || row.epoch != audit.epoch
        {
            return Err(
                "EC_PUBLISH_CANCEL: cancellation audit generation identity mismatch".to_owned(),
            );
        }
        let owner_ordinal = usize::try_from(row.owner_ordinal)
            .map_err(|_| "EC_PUBLISH_CANCEL: local owner ordinal is invalid".to_owned())?;
        let local_member = descriptor.roster.get(owner_ordinal).ok_or_else(|| {
            "EC_PUBLISH_CANCEL: local owner ordinal is outside the cancellation roster".to_owned()
        })?;
        if local_member.logical_index_uuid != *logical_index_uuid.as_bytes()
            || local_member.node_id != row.node_id
        {
            return Err(
                "EC_PUBLISH_CANCEL: cancellation audit participant identity mismatch".to_owned(),
            );
        }
        if row.state == GenerationState::Published {
            let generation = generation_catalog::extension_relation_name("ec_distann_generation")?;
            let persisted = Spi::connect(|client| {
                client
                    .select(
                        &format!(
                            "SELECT epoch_fingerprint, manifest_digest
                               FROM {generation}
                              WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                                AND build_id = $3::uuid"
                        ),
                        None,
                        &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                    )
                    .map_err(|error| {
                        format!("EC_PUBLISH_CANCEL: Published identity lookup failed: {error}")
                    })?
                    .map(|catalog_row| {
                        Ok::<_, String>((
                            optional_bytes(&catalog_row, "epoch_fingerprint")?,
                            optional_bytes(&catalog_row, "manifest_digest")?,
                        ))
                    })
                    .next()
                    .transpose()?
                    .ok_or_else(|| {
                        "EC_PUBLISH_CANCEL: locked Published generation disappeared".to_owned()
                    })
            })?;
            if persisted.0.as_deref() != Some(audit.epoch_fingerprint.as_slice())
                || persisted.1.as_deref() != Some(audit.manifest_digest.as_slice())
            {
                return Err(
                    "EC_PUBLISH_CANCEL: cancellation audit differs from Published generation"
                        .to_owned(),
                );
            }
        }
        let ready_receipt = DistannReadyReceipt::decode(
            row.ready_receipt
                .as_ref()
                .ok_or_else(|| "EC_PUBLISH_CANCEL: generation has no Ready receipt".to_owned())?,
        )?;
        if ready_receipt.build_id != audit.build_id
            || ready_receipt.epoch != audit.epoch
            || ready_receipt.owned_record_count != row.cumulative_record_count
            || ready_receipt.row_count != row.cumulative_record_count
        {
            return Err(
                "EC_PUBLISH_CANCEL: Ready receipt differs from cancelled generation".to_owned(),
            );
        }

        let active = generation_catalog::extension_relation_name("ec_distann_active_epoch")?;
        let is_active = Spi::connect(|client| {
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
        if is_active {
            return Err("EC_PUBLISH_CANCEL: active generation cannot be reclaimed".to_owned());
        }

        let epoch = i64::try_from(audit.epoch)
            .map_err(|_| "EC_PUBLISH_CANCEL: cancelled epoch exceeds bigint".to_owned())?;
        Spi::connect_mut(|client| -> Result<(), String> {
            client
                .update(
                    &format!(
                        "INSERT INTO {tombstone} (
                             index_oid, logical_index_uuid, build_id, epoch,
                             epoch_fingerprint, manifest_digest, prior_state,
                             build_spec_digest, generation_descriptor_digest,
                             record_count, row_count, cancellation_audit,
                             cancellation_audit_digest
                         ) VALUES (
                             $1::oid, $2::uuid, $3::uuid, $4::bigint,
                             $5::bytea, $6::bytea, $7::text,
                             $8::bytea, $9::bytea, $10::bigint, $11::bigint,
                             $12::bytea, $13::bytea
                         )"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        build_id.into(),
                        epoch.into(),
                        audit.epoch_fingerprint.to_vec().into(),
                        audit.manifest_digest.to_vec().into(),
                        row.state.as_str().into(),
                        row.build_spec_digest.to_vec().into(),
                        row.generation_descriptor_digest.to_vec().into(),
                        i64::try_from(ready_receipt.owned_record_count)
                            .map_err(|_| {
                                "EC_PUBLISH_CANCEL: record count exceeds bigint".to_owned()
                            })?
                            .into(),
                        i64::try_from(ready_receipt.row_count)
                            .map_err(|_| "EC_PUBLISH_CANCEL: row count exceeds bigint".to_owned())?
                            .into(),
                        cancellation_audit.clone().into(),
                        supplied_digest.to_vec().into(),
                    ],
                )
                .map_err(|error| {
                    format!("EC_PUBLISH_CANCEL: cancellation tombstone insert failed: {error}")
                })?;
            Ok(())
        })?;
        drop_generation_relations(
            index_oid,
            GenerationRelations {
                row_tier_relid: row.row_tier_relid,
                cold_tier_relid: row.cold_tier_relid,
                graph_store_relid: row.graph_store_relid,
                node_store_relid: row.node_store_relid,
                directory_relid: row.directory_relid,
                payload_sidecar_relid: row.payload_sidecar_relid,
                payload_sidecar_directory_relid: row.payload_sidecar_directory_relid,
            },
        )?;
        let generation = generation_catalog::extension_relation_name("ec_distann_generation")?;
        let deleted = Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "DELETE FROM {generation}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                            AND build_id = $3::uuid AND state IN ('Ready', 'Published')
                          RETURNING 1"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map(|rows| rows.len())
                .map_err(|error| {
                    format!("EC_PUBLISH_CANCEL: cancelled generation delete failed: {error}")
                })
        })?;
        if deleted != 1 {
            return Err("EC_PUBLISH_CANCEL: cancelled generation changed concurrently".to_owned());
        }
        Ok(())
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
}
