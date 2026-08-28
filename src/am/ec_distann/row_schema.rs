//! FR-078 canonical epoch-row-tier schema descriptor and fingerprint.

use pgrx::{pg_sys, Spi};

use super::canonical_wire::{domain_digest, CanonicalDecoder, CanonicalEncoder};

pub const DISTANN_ROW_SCHEMA_VERSION: u16 = 1;
pub const DISTANN_ROW_SCHEMA_DOMAIN: &[u8] = b"ec_distann_row_schema_v1\0";
pub const DISTANN_ROW_SCHEMA_VERSION_OFFSET: usize = 0;
pub(crate) const DISTANN_MAX_PHYSICAL_ATTRIBUTES: usize = 1664;

/// Closed PG18 binary-I/O-stable scalar set shared by compact generation
/// layouts. Variable-width, collatable, domain, array, and user-defined types
/// deliberately return `None`.
pub(crate) fn fixed_binary_width(type_namespace: &str, type_name: &str) -> Option<u16> {
    if type_namespace != "pg_catalog" {
        return None;
    }
    match type_name {
        "bool" => Some(1),
        "int2" => Some(2),
        "int4" | "float4" | "date" => Some(4),
        "int8" | "float8" | "time" | "timestamp" | "timestamptz" => Some(8),
        "uuid" => Some(16),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistannRowSchemaAttribute {
    pub attnum: u16,
    pub name: String,
    pub type_namespace: String,
    pub type_name: String,
    pub typmod: i32,
    pub collation_namespace: String,
    pub collation_name: String,
    pub dropped: bool,
    /// PostgreSQL `attgenerated`: 0, `s` (stored), or `v` (virtual).
    pub generated_kind: u8,
    /// Catalog-resolved qualified identity (`namespace.name`), or empty only
    /// for a dropped attribute.
    pub send_function: String,
    /// Catalog-resolved qualified identity (`namespace.name`), or empty only
    /// for a dropped attribute.
    pub receive_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistannRowSchemaDescriptor {
    pub attributes: Vec<DistannRowSchemaAttribute>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedRowSchemaColumn {
    pub(crate) attnum: u16,
    pub(crate) name: String,
    pub(crate) sql_type: String,
    pub(crate) collation_sql: Option<String>,
    pub(crate) dropped: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedRowSchema {
    pub(crate) descriptor: DistannRowSchemaDescriptor,
    pub(crate) columns: Vec<ResolvedRowSchemaColumn>,
}

pub(crate) fn resolve_relation_schema(
    relation_oid: pg_sys::Oid,
) -> Result<ResolvedRowSchema, String> {
    let rows = Spi::connect(|client| {
        client
            .select(
                "SELECT a.attnum::int4 AS attnum,
                        CASE WHEN a.attisdropped THEN '' ELSE a.attname::text END AS attribute_name,
                        CASE WHEN a.attisdropped THEN '' ELSE tn.nspname::text END AS type_namespace,
                        CASE WHEN a.attisdropped THEN '' ELSE t.typname::text END AS type_name,
                        CASE WHEN a.attisdropped THEN -1 ELSE a.atttypmod END AS typmod,
                        CASE WHEN a.attisdropped OR a.attcollation = 0 THEN '' ELSE cn.nspname::text END AS collation_namespace,
                        CASE WHEN a.attisdropped OR a.attcollation = 0 THEN '' ELSE c.collname::text END AS collation_name,
                        a.attisdropped AS dropped,
                        CASE WHEN a.attisdropped THEN '' ELSE a.attgenerated::text END AS generated_kind,
                        CASE WHEN a.attisdropped OR send_proc.oid IS NULL THEN ''
                             ELSE format('%I.%I', send_ns.nspname, send_proc.proname) END AS send_function,
                        CASE WHEN a.attisdropped OR recv_proc.oid IS NULL THEN ''
                             ELSE format('%I.%I', recv_ns.nspname, recv_proc.proname) END AS receive_function,
                        CASE WHEN a.attisdropped THEN '' ELSE format_type(a.atttypid, a.atttypmod) END AS sql_type,
                        CASE WHEN a.attisdropped OR a.attcollation = 0 THEN NULL
                             ELSE format('%I.%I', cn.nspname, c.collname) END AS collation_sql
                   FROM pg_catalog.pg_attribute a
                   LEFT JOIN pg_catalog.pg_type t ON t.oid = a.atttypid
                   LEFT JOIN pg_catalog.pg_namespace tn ON tn.oid = t.typnamespace
                   LEFT JOIN pg_catalog.pg_collation c ON c.oid = a.attcollation
                   LEFT JOIN pg_catalog.pg_namespace cn ON cn.oid = c.collnamespace
                   LEFT JOIN pg_catalog.pg_proc send_proc ON send_proc.oid = t.typsend
                   LEFT JOIN pg_catalog.pg_namespace send_ns ON send_ns.oid = send_proc.pronamespace
                   LEFT JOIN pg_catalog.pg_proc recv_proc ON recv_proc.oid = t.typreceive
                   LEFT JOIN pg_catalog.pg_namespace recv_ns ON recv_ns.oid = recv_proc.pronamespace
                  WHERE a.attrelid = $1::oid
                    AND a.attnum > 0
                  ORDER BY a.attnum",
                None,
                &[relation_oid.into()],
            )
            .map_err(|error| format!("EC_SCHEMA_MISMATCH: local schema lookup failed: {error}"))?
            .map(|row| {
                let required_string = |name: &str| -> Result<String, String> {
                    row[name]
                        .value::<String>()
                        .map_err(|error| {
                            format!("EC_SCHEMA_MISMATCH: {name} decode failed: {error}")
                        })?
                        .ok_or_else(|| format!("EC_SCHEMA_MISMATCH: {name} is NULL"))
                };
                let attnum_i32 = row["attnum"]
                    .value::<i32>()
                    .map_err(|error| {
                        format!("EC_SCHEMA_MISMATCH: attnum decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_SCHEMA_MISMATCH: attnum is NULL".to_owned())?;
                let attnum = u16::try_from(attnum_i32).map_err(|_| {
                    format!("EC_SCHEMA_MISMATCH: attnum {attnum_i32} is outside u16")
                })?;
                let dropped = row["dropped"]
                    .value::<bool>()
                    .map_err(|error| {
                        format!("EC_SCHEMA_MISMATCH: dropped decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_SCHEMA_MISMATCH: dropped is NULL".to_owned())?;
                let generated = required_string("generated_kind")?;
                let generated_kind = match generated.as_bytes() {
                    [] => 0,
                    [kind] if matches!(*kind, b's' | b'v') => *kind,
                    _ => {
                        return Err(format!(
                            "EC_SCHEMA_UNSUPPORTED: attribute {attnum} has generated kind {generated:?}"
                        ))
                    }
                };
                let typmod = row["typmod"]
                    .value::<i32>()
                    .map_err(|error| {
                        format!("EC_SCHEMA_MISMATCH: typmod decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_SCHEMA_MISMATCH: typmod is NULL".to_owned())?;
                let collation_sql = row["collation_sql"]
                    .value::<String>()
                    .map_err(|error| {
                        format!("EC_SCHEMA_MISMATCH: collation SQL decode failed: {error}")
                    })?;
                Ok::<(DistannRowSchemaAttribute, ResolvedRowSchemaColumn), String>((
                    DistannRowSchemaAttribute {
                        attnum,
                        name: required_string("attribute_name")?,
                        type_namespace: required_string("type_namespace")?,
                        type_name: required_string("type_name")?,
                        typmod,
                        collation_namespace: required_string("collation_namespace")?,
                        collation_name: required_string("collation_name")?,
                        dropped,
                        generated_kind,
                        send_function: required_string("send_function")?,
                        receive_function: required_string("receive_function")?,
                    },
                    ResolvedRowSchemaColumn {
                        attnum,
                        name: required_string("attribute_name")?,
                        sql_type: required_string("sql_type")?,
                        collation_sql,
                        dropped,
                    },
                ))
            })
            .collect::<Result<Vec<_>, String>>()
    })?;

    if rows.is_empty() {
        return Err("EC_SCHEMA_MISMATCH: source relation has no physical attributes".to_owned());
    }
    let (attributes, columns): (Vec<_>, Vec<_>) = rows.into_iter().unzip();
    let descriptor = DistannRowSchemaDescriptor { attributes };
    descriptor.validate()?;
    Ok(ResolvedRowSchema {
        descriptor,
        columns,
    })
}

impl DistannRowSchemaDescriptor {
    pub fn validate(&self) -> Result<(), String> {
        if self.attributes.len() > DISTANN_MAX_PHYSICAL_ATTRIBUTES {
            return Err(format!(
                "EC_GENERATION_DESCRIPTOR: row schema has {} physical attributes, maximum is {DISTANN_MAX_PHYSICAL_ATTRIBUTES}",
                self.attributes.len()
            ));
        }
        for (index, attribute) in self.attributes.iter().enumerate() {
            let expected_attnum = u16::try_from(index + 1)
                .map_err(|_| "EC_GENERATION_DESCRIPTOR: row attnum overflow".to_owned())?;
            if attribute.attnum != expected_attnum {
                return Err(format!(
                    "EC_GENERATION_DESCRIPTOR: row schema attnum {} is not contiguous expected {expected_attnum}",
                    attribute.attnum
                ));
            }
            for (field, value) in [
                ("attribute name", attribute.name.as_str()),
                ("type namespace", attribute.type_namespace.as_str()),
                ("type name", attribute.type_name.as_str()),
                (
                    "collation namespace",
                    attribute.collation_namespace.as_str(),
                ),
                ("collation name", attribute.collation_name.as_str()),
                ("send function", attribute.send_function.as_str()),
                ("receive function", attribute.receive_function.as_str()),
            ] {
                if value.as_bytes().contains(&0) {
                    return Err(format!(
                        "EC_GENERATION_DESCRIPTOR: row schema {field} contains NUL"
                    ));
                }
            }
            if !matches!(attribute.generated_kind, 0 | b's' | b'v') {
                return Err(format!(
                    "EC_GENERATION_DESCRIPTOR: attribute {} has unsupported generated kind {}",
                    attribute.attnum, attribute.generated_kind
                ));
            }
            if attribute.collation_namespace.is_empty() != attribute.collation_name.is_empty() {
                return Err(format!(
                    "EC_GENERATION_DESCRIPTOR: attribute {} has incomplete collation identity",
                    attribute.attnum
                ));
            }
            if attribute.dropped
                && (!attribute.name.is_empty()
                    || !attribute.type_namespace.is_empty()
                    || !attribute.type_name.is_empty()
                    || attribute.typmod != -1
                    || !attribute.collation_namespace.is_empty()
                    || !attribute.collation_name.is_empty()
                    || attribute.generated_kind != 0
                    || !attribute.send_function.is_empty()
                    || !attribute.receive_function.is_empty())
            {
                return Err(format!(
                    "EC_GENERATION_DESCRIPTOR: dropped attribute {} is not in its canonical empty form",
                    attribute.attnum
                ));
            }
            if !attribute.dropped
                && (attribute.name.is_empty()
                    || attribute.type_namespace.is_empty()
                    || attribute.type_name.is_empty()
                    || attribute.send_function.is_empty()
                    || attribute.receive_function.is_empty())
            {
                return Err(format!(
                    "EC_SCHEMA_UNSUPPORTED: non-dropped attribute {} lacks a canonical type or binary function identity",
                    attribute.attnum
                ));
            }
        }
        Ok(())
    }

    pub fn non_dropped_count(&self) -> usize {
        self.attributes
            .iter()
            .filter(|attribute| !attribute.dropped)
            .count()
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let attribute_count = u16::try_from(self.attributes.len())
            .map_err(|_| "EC_GENERATION_DESCRIPTOR: row attribute count exceeds u16".to_owned())?;
        let mut encoder = CanonicalEncoder::with_capacity(4 + self.attributes.len() * 64);
        encoder.put_u16(DISTANN_ROW_SCHEMA_VERSION);
        encoder.put_u16(attribute_count);
        for attribute in &self.attributes {
            encoder.put_u16(attribute.attnum);
            encoder.put_string(&attribute.name)?;
            encoder.put_string(&attribute.type_namespace)?;
            encoder.put_string(&attribute.type_name)?;
            encoder.put_i32(attribute.typmod);
            encoder.put_string(&attribute.collation_namespace)?;
            encoder.put_string(&attribute.collation_name)?;
            encoder.put_u8(u8::from(attribute.dropped));
            encoder.put_u8(attribute.generated_kind);
            encoder.put_string(&attribute.send_function)?;
            encoder.put_string(&attribute.receive_function)?;
        }
        encoder.finish()
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "row schema descriptor")?;
        let version = decoder.get_u16("row schema descriptor version")?;
        if version != DISTANN_ROW_SCHEMA_VERSION {
            return Err(format!(
                "EC_GENERATION_DESCRIPTOR: unsupported row schema descriptor version {version}"
            ));
        }
        let attribute_count = decoder.get_u16("physical attribute count")? as usize;
        if attribute_count > DISTANN_MAX_PHYSICAL_ATTRIBUTES {
            return Err(format!(
                "EC_GENERATION_DESCRIPTOR: row schema declares {attribute_count} attributes"
            ));
        }
        let mut attributes = Vec::with_capacity(attribute_count);
        for _ in 0..attribute_count {
            attributes.push(DistannRowSchemaAttribute {
                attnum: decoder.get_u16("attribute attnum")?,
                name: decoder.get_string("attribute name")?,
                type_namespace: decoder.get_string("type namespace")?,
                type_name: decoder.get_string("type name")?,
                typmod: decoder.get_i32("attribute typmod")?,
                collation_namespace: decoder.get_string("collation namespace")?,
                collation_name: decoder.get_string("collation name")?,
                dropped: match decoder.get_u8("attribute dropped flag")? {
                    0 => false,
                    1 => true,
                    other => {
                        return Err(format!(
                            "EC_GENERATION_DESCRIPTOR: invalid dropped flag {other}"
                        ))
                    }
                },
                generated_kind: decoder.get_u8("attribute generated kind")?,
                send_function: decoder.get_string("send function identity")?,
                receive_function: decoder.get_string("receive function identity")?,
            });
        }
        decoder.finish("row schema descriptor")?;
        let descriptor = Self { attributes };
        descriptor.validate()?;
        Ok(descriptor)
    }

    pub fn fingerprint(&self) -> Result<[u8; 32], String> {
        Ok(domain_digest(DISTANN_ROW_SCHEMA_DOMAIN, &self.encode()?))
    }
}

#[cfg(test)]
pub(crate) fn sample_row_schema() -> DistannRowSchemaDescriptor {
    DistannRowSchemaDescriptor {
        attributes: vec![
            DistannRowSchemaAttribute {
                attnum: 1,
                name: "id".to_owned(),
                type_namespace: "pg_catalog".to_owned(),
                type_name: "uuid".to_owned(),
                typmod: -1,
                collation_namespace: String::new(),
                collation_name: String::new(),
                dropped: false,
                generated_kind: 0,
                send_function: "pg_catalog.uuid_send".to_owned(),
                receive_function: "pg_catalog.uuid_recv".to_owned(),
            },
            DistannRowSchemaAttribute {
                attnum: 2,
                name: String::new(),
                type_namespace: String::new(),
                type_name: String::new(),
                typmod: -1,
                collation_namespace: String::new(),
                collation_name: String::new(),
                dropped: true,
                generated_kind: 0,
                send_function: String::new(),
                receive_function: String::new(),
            },
            DistannRowSchemaAttribute {
                attnum: 3,
                name: "embedding".to_owned(),
                type_namespace: "public".to_owned(),
                type_name: "ecvector".to_owned(),
                typmod: 1536,
                collation_namespace: String::new(),
                collation_name: String::new(),
                dropped: false,
                generated_kind: 0,
                send_function: "public.ecvector_send".to_owned(),
                receive_function: "public.ecvector_recv".to_owned(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_schema_round_trip_and_fingerprint_are_canonical() {
        let descriptor = sample_row_schema();
        let encoded = descriptor.encode().unwrap();
        assert_eq!(
            DistannRowSchemaDescriptor::decode(&encoded).unwrap(),
            descriptor
        );
        assert_eq!(descriptor.non_dropped_count(), 2);
        assert_eq!(
            descriptor.fingerprint().unwrap(),
            descriptor.fingerprint().unwrap()
        );
    }

    #[test]
    fn row_schema_rejects_version_order_and_unsupported_binary_identity() {
        let mut encoded = sample_row_schema().encode().unwrap();
        encoded[0..2].copy_from_slice(&99_u16.to_le_bytes());
        assert!(DistannRowSchemaDescriptor::decode(&encoded).is_err());

        let mut descriptor = sample_row_schema();
        descriptor.attributes[2].attnum = 4;
        assert!(descriptor.encode().is_err());

        let mut descriptor = sample_row_schema();
        descriptor.attributes[0].receive_function.clear();
        assert!(descriptor
            .encode()
            .unwrap_err()
            .contains("EC_SCHEMA_UNSUPPORTED"));

        let mut descriptor = sample_row_schema();
        descriptor.attributes[1].name = "stale_dropped_name".to_owned();
        assert!(descriptor
            .encode()
            .unwrap_err()
            .contains("canonical empty form"));
    }
}
