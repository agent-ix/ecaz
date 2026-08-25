//! Task 179 participant identity and coordinator desired-roster registration.
//!
//! Raw conninfo is resolved only long enough to authenticate and query the
//! participant control identity. The desired registry persists the secret
//! reference and values returned by that endpoint, never caller-supplied UUIDs,
//! locator spellings, or connection material. A later begin-build transaction
//! copies these rows into immutable build-specific private bindings.

#[cfg(feature = "pg_test")]
use std::cell::RefCell;
#[cfg(feature = "pg_test")]
use std::collections::HashMap;

use pgrx::datum::Uuid;
use pgrx::{pg_extern, pg_sys, PgRelation, Spi};
use sha2::{Digest, Sha256};

use super::canonical_wire::{fixed_digest, is_rfc4122_v4_uuid};
use super::generation_catalog::extension_relation_name;
use super::generation_descriptor::{
    validate_endpoint_identity, validate_roster, DistannRosterEntry,
};
use super::generation_store::{control_compatibility_digest, open_control_index};
use super::remote_transport::connect_distann_postgres;
use crate::am::common::remote_postgres_tls::{remote_security_fingerprint, RemoteTlsPolicy};

struct ControlRegistrationIdentity {
    logical_index_uuid: Uuid,
    format_version: i32,
    distributed_control: bool,
    compatibility_digest: [u8; 32],
    endpoint_identity: String,
    canonical_index_regclass: String,
}

#[cfg(feature = "pg_test")]
thread_local! {
    static TEST_CONNINFO_SECRET_OVERRIDES: RefCell<HashMap<String, String>> =
        RefCell::new(HashMap::new());
}

fn validate_secret_reference(value: &str) -> Result<(), String> {
    let bytes = value.as_bytes();
    let valid = bytes.len() <= 128
        && bytes.first().is_some_and(|byte| byte.is_ascii_uppercase())
        && bytes
            .iter()
            .skip(1)
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || *byte == b'_');
    if !valid {
        return Err(
            "EC_NODE_DESCRIPTOR: conninfo secret reference violates the canonical v1 grammar"
                .to_owned(),
        );
    }
    Ok(())
}

fn valid_locator_component(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() <= 63
        && bytes
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || *byte == b'_')
        && bytes
            .iter()
            .skip(1)
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
}

pub(crate) fn validate_canonical_index_locator(value: &str) -> Result<(), String> {
    let Some((schema, relation)) = value.split_once('.') else {
        return Err(
            "EC_NODE_DESCRIPTOR: remote index locator must be schema-qualified canonical v1 text"
                .to_owned(),
        );
    };
    if relation.contains('.')
        || !valid_locator_component(schema)
        || !valid_locator_component(relation)
    {
        return Err(
            "EC_NODE_DESCRIPTOR: remote index locator violates the canonical v1 grammar".to_owned(),
        );
    }
    Ok(())
}

#[derive(Clone)]
pub(crate) struct ResolvedConninfoSecret {
    raw_conninfo: String,
    secret_reference: Option<String>,
    secret_identity_fingerprint: [u8; 32],
    security_fingerprint: [u8; 32],
}

impl std::fmt::Debug for ResolvedConninfoSecret {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ResolvedConninfoSecret")
            .field(
                "secret_identity_fingerprint",
                &hex::encode(self.secret_identity_fingerprint),
            )
            .field(
                "security_fingerprint",
                &hex::encode(self.security_fingerprint),
            )
            .finish_non_exhaustive()
    }
}

impl ResolvedConninfoSecret {
    pub(crate) fn conninfo(&self) -> &str {
        &self.raw_conninfo
    }

    pub(crate) fn secret_identity_fingerprint(&self) -> [u8; 32] {
        self.secret_identity_fingerprint
    }

    pub(crate) fn security_fingerprint(&self) -> [u8; 32] {
        self.security_fingerprint
    }

    pub(crate) fn secret_reference(&self) -> Option<&str> {
        self.secret_reference.as_deref()
    }

    pub(crate) fn from_legacy_conninfo(raw_conninfo: String) -> Self {
        resolved_conninfo_secret(None, raw_conninfo)
    }

    #[cfg(test)]
    pub(crate) fn from_test_secret_reference(secret_reference: &str, raw_conninfo: String) -> Self {
        resolved_conninfo_secret(Some(secret_reference), raw_conninfo)
    }
}

pub(crate) fn resolve_conninfo_secret(
    conninfo_secret_name: &str,
) -> Result<ResolvedConninfoSecret, String> {
    validate_secret_reference(conninfo_secret_name)?;
    #[cfg(feature = "pg_test")]
    if let Some(raw_conninfo) = TEST_CONNINFO_SECRET_OVERRIDES
        .with(|overrides| overrides.borrow().get(conninfo_secret_name).cloned())
    {
        if raw_conninfo.is_empty() {
            return Err(
                "EC_NODE_DESCRIPTOR secret_empty: conninfo secret resolved to an empty value"
                    .to_owned(),
            );
        }
        return Ok(resolved_conninfo_secret(
            Some(conninfo_secret_name),
            raw_conninfo,
        ));
    }
    let key = crate::am::spire_remote_conninfo_secret_provider_lookup_key(conninfo_secret_name)
        .map_err(|_| {
            "EC_NODE_DESCRIPTOR secret_invalid: conninfo secret reference is invalid".to_owned()
        })?;
    let raw_conninfo = match std::env::var(key) {
        Ok(value) if !value.is_empty() => value,
        Ok(_) => {
            return Err(
                "EC_NODE_DESCRIPTOR secret_empty: conninfo secret resolved to an empty value"
                    .to_owned(),
            )
        }
        Err(_) => {
            return Err(
                "EC_NODE_DESCRIPTOR secret_missing: conninfo secret is unavailable".to_owned(),
            )
        }
    };
    Ok(resolved_conninfo_secret(
        Some(conninfo_secret_name),
        raw_conninfo,
    ))
}

fn resolved_conninfo_secret(
    conninfo_secret_name: Option<&str>,
    raw_conninfo: String,
) -> ResolvedConninfoSecret {
    let mut identity_hasher = Sha256::new();
    identity_hasher.update(b"ecaz-distann-secret-identity-v1\0");
    match conninfo_secret_name {
        Some(name) => identity_hasher.update(name.as_bytes()),
        None => identity_hasher.update(raw_conninfo.as_bytes()),
    }
    ResolvedConninfoSecret {
        security_fingerprint: remote_security_fingerprint(
            &raw_conninfo,
            RemoteTlsPolicy::DistannSecure,
        ),
        secret_reference: conninfo_secret_name.map(str::to_owned),
        raw_conninfo,
        secret_identity_fingerprint: identity_hasher.finalize().into(),
    }
}

#[cfg(feature = "pg_test")]
#[pg_extern(volatile, strict)]
fn ec_distann_test_set_conninfo_secret(conninfo_secret_name: String, conninfo: String) {
    validate_secret_reference(&conninfo_secret_name)
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    TEST_CONNINFO_SECRET_OVERRIDES.with(|overrides| {
        overrides
            .borrow_mut()
            .insert(conninfo_secret_name, conninfo);
    });
}

fn parse_uuid_text(value: &str) -> Result<Uuid, String> {
    if value.len() != 36
        || value.as_bytes().get(8) != Some(&b'-')
        || value.as_bytes().get(13) != Some(&b'-')
        || value.as_bytes().get(18) != Some(&b'-')
        || value.as_bytes().get(23) != Some(&b'-')
    {
        return Err("EC_NODE_DESCRIPTOR: remote control returned a malformed UUID".to_owned());
    }
    let compact = value.replace('-', "");
    let bytes = hex::decode(compact)
        .map_err(|_| "EC_NODE_DESCRIPTOR: remote control returned a malformed UUID".to_owned())?;
    let bytes: [u8; 16] = bytes
        .try_into()
        .map_err(|_| "EC_NODE_DESCRIPTOR: remote control returned a malformed UUID".to_owned())?;
    if !is_rfc4122_v4_uuid(&bytes) {
        return Err("EC_NODE_DESCRIPTOR: remote control returned a non-v4 logical UUID".to_owned());
    }
    Ok(Uuid::from_bytes(bytes))
}

fn lock_registry_state(index_oid: pg_sys::Oid, logical_index_uuid: Uuid) -> Result<i64, String> {
    let registry_state = extension_relation_name("ec_distann_registry_state")?;
    let sql = format!(
        "SELECT revision
           FROM {registry_state}
          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
          FOR UPDATE"
    );
    Spi::connect_mut(|client| {
        client
            .update(&sql, None, &[index_oid.into(), logical_index_uuid.into()])
            .map_err(|_| "EC_NODE_DESCRIPTOR: registry state lock failed".to_owned())?
            .map(|row| {
                row["revision"]
                    .value::<i64>()
                    .map_err(|_| "EC_NODE_DESCRIPTOR: registry revision decode failed".to_owned())?
                    .ok_or_else(|| "EC_NODE_DESCRIPTOR: registry revision is NULL".to_owned())
            })
            .next()
            .transpose()?
            .ok_or_else(|| "EC_NODE_DESCRIPTOR: registry state is absent".to_owned())
    })
}

fn increment_registry_revision(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    expected_revision: i64,
) -> Result<(), String> {
    let registry_state = extension_relation_name("ec_distann_registry_state")?;
    let sql = format!(
        "UPDATE {registry_state}
            SET revision = revision + 1, updated_at = clock_timestamp()
          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
            AND revision = $3::bigint
          RETURNING revision"
    );
    let updated = Spi::connect_mut(|client| {
        client
            .update(
                &sql,
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    expected_revision.into(),
                ],
            )
            .map_err(|_| "EC_NODE_DESCRIPTOR: registry revision update failed".to_owned())
            .map(|table| table.len())
    })?;
    if updated != 1 {
        // Defensive only under the required control-relation SRE lock plus
        // registry-row lock. If a future caller weakens that discipline, fail
        // closed instead of silently reporting a stale successful mutation.
        return Err("EC_NODE_DESCRIPTOR: registry revision changed concurrently".to_owned());
    }
    Ok(())
}

fn local_control_identity(
    remote_index_regclass: &str,
) -> Result<ControlRegistrationIdentity, String> {
    Spi::connect(|client| {
        client
            .select(
                "SELECT logical_index_uuid, index_format_version,
                        distributed_control, compatibility_digest,
                        endpoint_identity, canonical_index_regclass
                   FROM ec_distann_control_identity(
                       pg_catalog.to_regclass($1::text)
                   )",
                None,
                &[remote_index_regclass.to_owned().into()],
            )
            .map_err(|_| "EC_NODE_DESCRIPTOR: local control identity query failed".to_owned())?
            .map(|row| {
                let logical_index_uuid = row["logical_index_uuid"]
                    .value::<Uuid>()
                    .map_err(|_| "EC_NODE_DESCRIPTOR: local logical UUID decode failed".to_owned())?
                    .ok_or_else(|| "EC_NODE_DESCRIPTOR: local logical UUID is NULL".to_owned())?;
                if !is_rfc4122_v4_uuid(logical_index_uuid.as_bytes()) {
                    return Err(
                        "EC_NODE_DESCRIPTOR: local control returned a non-v4 logical UUID"
                            .to_owned(),
                    );
                }
                let required_string = |name: &str| -> Result<String, String> {
                    row[name]
                        .value::<String>()
                        .map_err(|_| format!("EC_NODE_DESCRIPTOR: local {name} decode failed"))?
                        .ok_or_else(|| format!("EC_NODE_DESCRIPTOR: local {name} is unconfigured"))
                };
                Ok(ControlRegistrationIdentity {
                    logical_index_uuid,
                    format_version: row["index_format_version"]
                        .value::<i32>()
                        .map_err(|_| {
                            "EC_NODE_DESCRIPTOR: local format version decode failed".to_owned()
                        })?
                        .ok_or_else(|| {
                            "EC_NODE_DESCRIPTOR: local format version is NULL".to_owned()
                        })?,
                    distributed_control: row["distributed_control"]
                        .value::<bool>()
                        .map_err(|_| {
                            "EC_NODE_DESCRIPTOR: local control flag decode failed".to_owned()
                        })?
                        .ok_or_else(|| {
                            "EC_NODE_DESCRIPTOR: local control flag is NULL".to_owned()
                        })?,
                    compatibility_digest: fixed_digest(
                        row["compatibility_digest"]
                            .value::<Vec<u8>>()
                            .map_err(|_| {
                                "EC_NODE_DESCRIPTOR: local compatibility digest decode failed"
                                    .to_owned()
                            })?
                            .ok_or_else(|| {
                                "EC_NODE_DESCRIPTOR: local compatibility digest is NULL".to_owned()
                            })?,
                        "EC_NODE_DESCRIPTOR",
                        "local compatibility digest",
                    )?,
                    endpoint_identity: required_string("endpoint_identity")?,
                    canonical_index_regclass: required_string("canonical_index_regclass")?,
                })
            })
            .next()
            .transpose()?
            .ok_or_else(|| "EC_NODE_DESCRIPTOR: local control identity returned no row".to_owned())
    })
}

fn remote_control_identity(
    conninfo: &str,
    remote_index_regclass: &str,
    node_id: u32,
) -> Result<ControlRegistrationIdentity, String> {
    let mut client = connect_distann_postgres(conninfo, node_id, "ec_distann node registration")
        .map_err(|_| "EC_NODE_DESCRIPTOR: remote control connection failed".to_owned())?;
    let extension_schema = client
        .query_opt(
            "SELECT pg_catalog.quote_ident(n.nspname)
               FROM pg_catalog.pg_extension e
               JOIN pg_catalog.pg_namespace n ON n.oid = e.extnamespace
              WHERE e.extname = 'ecaz'",
            &[],
        )
        .map_err(|_| "EC_NODE_DESCRIPTOR: remote extension lookup failed".to_owned())?
        .ok_or_else(|| "EC_NODE_DESCRIPTOR: remote ecaz extension is absent".to_owned())?
        .try_get::<_, String>(0)
        .map_err(|_| "EC_NODE_DESCRIPTOR: remote extension schema decode failed".to_owned())?;
    let identity_sql = format!(
        "SELECT logical_index_uuid::text, index_format_version,
                distributed_control, compatibility_digest,
                endpoint_identity, canonical_index_regclass
           FROM {extension_schema}.ec_distann_control_identity($1::text::regclass)"
    );
    let row = client
        .query_opt(&identity_sql, &[&remote_index_regclass])
        .map_err(|_| "EC_NODE_DESCRIPTOR: remote control identity query failed".to_owned())?
        .ok_or_else(|| "EC_NODE_DESCRIPTOR: remote control identity returned no row".to_owned())?;
    let endpoint_identity = row
        .try_get::<_, Option<String>>(4)
        .map_err(|_| "EC_NODE_DESCRIPTOR: remote endpoint identity decode failed".to_owned())?
        .ok_or_else(|| "EC_NODE_DESCRIPTOR: remote endpoint identity is unconfigured".to_owned())?;
    let canonical_index_regclass = row
        .try_get::<_, String>(5)
        .map_err(|_| "EC_NODE_DESCRIPTOR: remote canonical index decode failed".to_owned())?;
    Ok(ControlRegistrationIdentity {
        logical_index_uuid: parse_uuid_text(
            &row.try_get::<_, String>(0)
                .map_err(|_| "EC_NODE_DESCRIPTOR: remote logical UUID decode failed".to_owned())?,
        )?,
        format_version: row
            .try_get::<_, i32>(1)
            .map_err(|_| "EC_NODE_DESCRIPTOR: remote format version decode failed".to_owned())?,
        distributed_control: row
            .try_get::<_, bool>(2)
            .map_err(|_| "EC_NODE_DESCRIPTOR: remote control flag decode failed".to_owned())?,
        compatibility_digest: fixed_digest(
            row.try_get::<_, Vec<u8>>(3).map_err(|_| {
                "EC_NODE_DESCRIPTOR: remote compatibility digest decode failed".to_owned()
            })?,
            "EC_NODE_DESCRIPTOR",
            "remote compatibility digest",
        )?,
        endpoint_identity,
        canonical_index_regclass,
    })
}

fn catalog_conflict(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    roster_ordinal: i32,
    node_id: i32,
    endpoint_identity: &str,
    participant_uuid: Uuid,
    is_local: bool,
) -> Result<Option<&'static str>, String> {
    let catalog = extension_relation_name("ec_distann_node_descriptor")?;
    let sql = format!(
        "SELECT EXISTS (
                    SELECT 1 FROM {catalog}
                     WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                       AND roster_ordinal = $3::integer
                ) AS ordinal_exists,
                EXISTS (
                    SELECT 1 FROM {catalog}
                     WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                       AND node_id = $4::integer
                ) AS node_exists,
                EXISTS (
                    SELECT 1 FROM {catalog}
                     WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                       AND endpoint_identity = $5::text
                ) AS endpoint_exists,
                EXISTS (
                    SELECT 1 FROM {catalog}
                     WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                       AND participant_logical_index_uuid = $6::uuid
                ) AS participant_exists,
                EXISTS (
                    SELECT 1 FROM {catalog}
                     WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                       AND is_local
                ) AS local_exists"
    );
    Spi::connect(|client| {
        client
            .select(
                &sql,
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    roster_ordinal.into(),
                    node_id.into(),
                    endpoint_identity.to_owned().into(),
                    participant_uuid.into(),
                ],
            )
            .map_err(|_| "EC_NODE_DESCRIPTOR: conflict lookup failed".to_owned())?
            .map(|row| {
                let value = |name: &str| -> Result<bool, String> {
                    row[name]
                        .value::<bool>()
                        .map_err(|_| format!("EC_NODE_DESCRIPTOR: {name} decode failed"))?
                        .ok_or_else(|| format!("EC_NODE_DESCRIPTOR: {name} is NULL"))
                };
                Ok(if value("ordinal_exists")? {
                    Some("roster ordinal already exists")
                } else if value("node_exists")? {
                    Some("node id already exists")
                } else if value("participant_exists")? {
                    Some("participant logical UUID already exists")
                } else if value("endpoint_exists")? {
                    Some("endpoint identity already exists")
                } else if is_local && value("local_exists")? {
                    Some("local participant already exists")
                } else {
                    None
                })
            })
            .next()
            .transpose()
            .map(|value| value.flatten())
    })
}

#[pg_extern(volatile, strict)]
fn ec_distann_configure_participant_identity(
    index_regclass: PgRelation,
    endpoint_identity: String,
) {
    let result = (|| -> Result<(), String> {
        validate_endpoint_identity(&endpoint_identity)?;
        let index_oid = index_regclass.oid();
        let (mut guard, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_configure_participant_identity",
        )
        .map_err(|_| {
            "EC_NODE_DESCRIPTOR: participant identity requires a v5 control index".to_owned()
        })?;
        guard.retain_lock_until_transaction_end();
        let _revision = lock_registry_state(index_oid, logical_index_uuid)?;
        let catalog = extension_relation_name("ec_distann_participant_identity")?;
        let existing = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT endpoint_identity FROM {catalog}
                          WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into()],
                )
                .map_err(|_| "EC_NODE_DESCRIPTOR: participant identity lookup failed".to_owned())?
                .map(|row| {
                    row["endpoint_identity"]
                        .value::<String>()
                        .map_err(|_| {
                            "EC_NODE_DESCRIPTOR: participant identity decode failed".to_owned()
                        })?
                        .ok_or_else(|| {
                            "EC_NODE_DESCRIPTOR: participant identity is NULL".to_owned()
                        })
                })
                .next()
                .transpose()
        })?;
        if let Some(existing) = existing {
            if existing == endpoint_identity {
                return Ok(());
            }
            return Err(
                "EC_NODE_DESCRIPTOR: participant identity is already configured differently"
                    .to_owned(),
            );
        }
        Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "INSERT INTO {catalog} (
                             index_oid, logical_index_uuid, endpoint_identity
                         ) VALUES ($1::oid, $2::uuid, $3::text)"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        endpoint_identity.into(),
                    ],
                )
                .map_err(|_| "EC_NODE_DESCRIPTOR: participant identity insert failed".to_owned())?;
            Ok::<(), String>(())
        })
    })();
    result.unwrap_or_else(|error| pgrx::error!("{error}"));
}

#[pg_extern(volatile, strict)]
#[allow(clippy::too_many_arguments)]
fn ec_distann_register_node_descriptor(
    index_regclass: PgRelation,
    roster_ordinal: i32,
    node_id: i32,
    endpoint_identity: String,
    conninfo_secret_name: String,
    remote_index_regclass: String,
    is_local: bool,
) {
    let result = (|| -> Result<(), String> {
        if roster_ordinal < 0 || node_id <= 0 {
            return Err(
                "EC_NODE_DESCRIPTOR: roster ordinal must be nonnegative and node id positive"
                    .to_owned(),
            );
        }
        validate_endpoint_identity(&endpoint_identity)?;
        validate_secret_reference(&conninfo_secret_name)?;
        validate_canonical_index_locator(&remote_index_regclass)?;
        let conninfo = resolve_conninfo_secret(&conninfo_secret_name)?;
        let index_oid = index_regclass.oid();
        let (mut guard, coordinator_handle, coordinator_metadata, logical_index_uuid) =
            open_control_index(
                index_oid,
                pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
                "ec_distann_register_node_descriptor coordinator",
            )
            .map_err(|_| {
                "EC_NODE_DESCRIPTOR: registration coordinator is not a v5 control".to_owned()
            })?;
        guard.retain_lock_until_transaction_end();
        let revision = lock_registry_state(index_oid, logical_index_uuid)?;
        let coordinator_compatibility =
            control_compatibility_digest(coordinator_handle, &coordinator_metadata)?;
        let participant = if is_local {
            local_control_identity(&remote_index_regclass)?
        } else {
            remote_control_identity(
                conninfo.conninfo(),
                &remote_index_regclass,
                u32::try_from(node_id).expect("positive i32 fits u32"),
            )?
        };
        if participant.format_version != i32::from(super::page::INDEX_FORMAT_V5_DISTANN_CONTROL)
            || !participant.distributed_control
        {
            return Err(
                "EC_NODE_DESCRIPTOR: participant is not a v5 distributed control".to_owned(),
            );
        }
        if participant.endpoint_identity != endpoint_identity {
            return Err(
                "EC_NODE_DESCRIPTOR: participant endpoint identity does not match the requested identity"
                    .to_owned(),
            );
        }
        validate_canonical_index_locator(&participant.canonical_index_regclass)?;
        if participant.canonical_index_regclass != remote_index_regclass {
            return Err(
                "EC_NODE_DESCRIPTOR: participant canonical index locator does not match the requested locator"
                    .to_owned(),
            );
        }
        if participant.compatibility_digest != coordinator_compatibility {
            return Err(
                "EC_NODE_DESCRIPTOR: participant schema/reloptions are incompatible with the coordinator"
                    .to_owned(),
            );
        }
        validate_roster(&[DistannRosterEntry {
            node_id: u32::try_from(node_id).expect("positive i32 fits u32"),
            logical_index_uuid: *participant.logical_index_uuid.as_bytes(),
            endpoint_identity: participant.endpoint_identity.clone(),
        }])?;
        if let Some(conflict) = catalog_conflict(
            index_oid,
            logical_index_uuid,
            roster_ordinal,
            node_id,
            &participant.endpoint_identity,
            participant.logical_index_uuid,
            is_local,
        )? {
            return Err(format!("EC_NODE_DESCRIPTOR: {conflict}"));
        }

        let catalog = extension_relation_name("ec_distann_node_descriptor")?;
        let sql = format!(
            "INSERT INTO {catalog} (
                 index_oid, logical_index_uuid, roster_ordinal, node_id,
                 endpoint_identity, conninfo_secret_name, remote_index_regclass,
                 participant_logical_index_uuid, compatibility_digest, is_local
             ) VALUES (
                 $1::oid, $2::uuid, $3::integer, $4::integer,
                 $5::text, $6::text, $7::text, $8::uuid, $9::bytea, $10::boolean
             )"
        );
        Spi::connect_mut(|client| {
            client
                .update(
                    &sql,
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        roster_ordinal.into(),
                        node_id.into(),
                        participant.endpoint_identity.into(),
                        conninfo_secret_name.into(),
                        participant.canonical_index_regclass.into(),
                        participant.logical_index_uuid.into(),
                        participant.compatibility_digest.to_vec().into(),
                        is_local.into(),
                    ],
                )
                .map_err(|_| "EC_NODE_DESCRIPTOR: catalog insert failed".to_owned())?;
            Ok::<(), String>(())
        })?;
        increment_registry_revision(index_oid, logical_index_uuid, revision)
    })();
    result.unwrap_or_else(|error| pgrx::error!("{error}"));
}

#[pg_extern(volatile, strict)]
fn ec_distann_unregister_node_descriptor(index_regclass: PgRelation, roster_ordinal: i32) {
    let result = (|| -> Result<(), String> {
        if roster_ordinal < 0 {
            return Err("EC_NODE_DESCRIPTOR: roster ordinal must be nonnegative".to_owned());
        }
        let index_oid = index_regclass.oid();
        let (mut guard, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_unregister_node_descriptor",
        )
        .map_err(|_| "EC_NODE_DESCRIPTOR: unregister coordinator is not a v5 control".to_owned())?;
        guard.retain_lock_until_transaction_end();
        let revision = lock_registry_state(index_oid, logical_index_uuid)?;
        let descriptor_catalog = extension_relation_name("ec_distann_node_descriptor")?;
        let binding_catalog = extension_relation_name("ec_distann_build_participant_binding")?;
        let build_catalog = extension_relation_name("ec_distann_build_registration")?;
        let publish_catalog = extension_relation_name("ec_distann_publish_decision")?;
        let mutation_sql = format!(
            "WITH target AS MATERIALIZED (
                 SELECT 1 FROM {descriptor_catalog}
                  WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                    AND roster_ordinal = $3::integer
             ), in_progress AS MATERIALIZED (
                    SELECT 1
                      FROM {binding_catalog} bp
                      JOIN {build_catalog} br
                        USING (index_oid, logical_index_uuid, build_id)
                     WHERE bp.index_oid = $1::oid
                       AND bp.logical_index_uuid = $2::uuid
                       AND bp.roster_ordinal = $3::integer
                       AND (
                           br.state IN ('Registered', 'Building', 'Ready', 'Decided')
                           OR EXISTS (
                               SELECT 1 FROM {publish_catalog} pd
                                WHERE pd.index_oid = br.index_oid
                                  AND pd.logical_index_uuid = br.logical_index_uuid
                                  AND pd.build_id = br.build_id
                                  AND pd.decision_state IN ('Pending', 'Activated')
                           )
                       )
             ), deleted AS (
                 DELETE FROM {descriptor_catalog} descriptor
                  WHERE descriptor.index_oid = $1::oid
                    AND descriptor.logical_index_uuid = $2::uuid
                    AND descriptor.roster_ordinal = $3::integer
                    AND EXISTS (SELECT 1 FROM target)
                    AND NOT EXISTS (SELECT 1 FROM in_progress)
                 RETURNING 1
             )
             SELECT EXISTS (SELECT 1 FROM target) AS descriptor_exists,
                    EXISTS (SELECT 1 FROM in_progress) AS in_progress_reference,
                    EXISTS (SELECT 1 FROM deleted) AS descriptor_deleted"
        );
        let (exists, referenced, deleted) = Spi::connect_mut(|client| {
            client
                .update(
                    &mutation_sql,
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        roster_ordinal.into(),
                    ],
                )
                .map_err(|_| "EC_NODE_DESCRIPTOR: unregister guard failed".to_owned())?
                .map(|row| {
                    let decode = |name: &str| -> Result<bool, String> {
                        row[name]
                            .value::<bool>()
                            .map_err(|_| format!("EC_NODE_DESCRIPTOR: {name} decode failed"))?
                            .ok_or_else(|| format!("EC_NODE_DESCRIPTOR: {name} is NULL"))
                    };
                    Ok::<(bool, bool, bool), String>((
                        decode("descriptor_exists")?,
                        decode("in_progress_reference")?,
                        decode("descriptor_deleted")?,
                    ))
                })
                .next()
                .transpose()?
                .ok_or_else(|| "EC_NODE_DESCRIPTOR: unregister mutation returned no row".to_owned())
        })?;
        if !exists {
            return Err("EC_NODE_DESCRIPTOR: roster ordinal is not registered".to_owned());
        }
        if referenced {
            return Err(
                "EC_BUILD_STATE: node descriptor is captured by an in-progress build".to_owned(),
            );
        }
        if !deleted {
            return Err(
                "EC_NODE_DESCRIPTOR: roster ordinal changed concurrently before deletion"
                    .to_owned(),
            );
        }
        increment_registry_revision(index_oid, logical_index_uuid, revision)
    })();
    result.unwrap_or_else(|error| pgrx::error!("{error}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_secret_separates_identity_rotation_and_redacts_debug() {
        let first = resolved_conninfo_secret(
            Some("DISTANN_NODE_TWO"),
            "host=db.example user=alice password=first sslmode=require".to_owned(),
        );
        let rotated = resolved_conninfo_secret(
            Some("DISTANN_NODE_TWO"),
            "host=db.example user=alice password=second sslmode=require".to_owned(),
        );
        assert_eq!(
            first.secret_identity_fingerprint(),
            rotated.secret_identity_fingerprint()
        );
        assert_ne!(first.security_fingerprint(), rotated.security_fingerprint());
        assert_eq!(first.secret_reference(), Some("DISTANN_NODE_TWO"));
        let debug = format!("{first:?}");
        for forbidden in ["DISTANN_NODE_TWO", "db.example", "alice", "first"] {
            assert!(!debug.contains(forbidden));
        }
    }

    #[cfg(feature = "pg_test")]
    #[test]
    fn pg_test_secret_override_is_backend_local_and_rotation_aware() {
        TEST_CONNINFO_SECRET_OVERRIDES.with(|overrides| {
            overrides.borrow_mut().insert(
                "DISTANN_NODE_TWO".to_owned(),
                "host=127.0.0.1 user=first sslmode=disable".to_owned(),
            );
        });
        let first = resolve_conninfo_secret("DISTANN_NODE_TWO").unwrap();
        TEST_CONNINFO_SECRET_OVERRIDES.with(|overrides| {
            overrides.borrow_mut().insert(
                "DISTANN_NODE_TWO".to_owned(),
                "host=127.0.0.1 user=second sslmode=disable".to_owned(),
            );
        });
        let rotated = resolve_conninfo_secret("DISTANN_NODE_TWO").unwrap();
        TEST_CONNINFO_SECRET_OVERRIDES.with(|overrides| overrides.borrow_mut().clear());

        assert_eq!(
            first.secret_identity_fingerprint(),
            rotated.secret_identity_fingerprint()
        );
        assert_ne!(first.security_fingerprint(), rotated.security_fingerprint());
        assert_eq!(
            rotated.conninfo(),
            "host=127.0.0.1 user=second sslmode=disable"
        );
    }
}
