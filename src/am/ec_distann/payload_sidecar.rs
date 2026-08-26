//! Task 229 covering-payload format and schema resolution.

use super::row_schema::{DistannRowSchemaAttribute, DistannRowSchemaDescriptor};

pub(crate) const DISTANN_PAYLOAD_COVER_ENTRY_VERSION: u16 = 1;
pub(crate) const DISTANN_PAYLOAD_COVER_MAX_ATTRIBUTES: usize = 16;
pub(crate) const DISTANN_PAYLOAD_COVER_MAX_VALUE_BYTES: usize = 256;
pub(crate) const DISTANN_PAYLOAD_COVER_MAX_NULL_BYTES: usize = 2;
pub(crate) const DISTANN_PAYLOAD_COVER_MAX_PAYLOAD_BYTES: usize =
    DISTANN_PAYLOAD_COVER_MAX_VALUE_BYTES + DISTANN_PAYLOAD_COVER_MAX_NULL_BYTES;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedPayloadCoverAttribute {
    pub(crate) schema_attribute: DistannRowSchemaAttribute,
    pub(crate) binary_width: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedPayloadCover {
    pub(crate) entry_format_version: u16,
    pub(crate) maximum_attribute_count: u16,
    pub(crate) row_schema_fingerprint: [u8; 32],
    pub(crate) attributes: Vec<ResolvedPayloadCoverAttribute>,
    pub(crate) maximum_payload_bytes: u16,
}

fn fixed_binary_width(attribute: &DistannRowSchemaAttribute) -> Option<u16> {
    if attribute.type_namespace != "pg_catalog" {
        return None;
    }
    match attribute.type_name.as_str() {
        "bool" => Some(1),
        "int2" => Some(2),
        "int4" | "float4" | "date" => Some(4),
        "int8" | "float8" | "time" | "timestamp" | "timestamptz" => Some(8),
        "uuid" => Some(16),
        _ => None,
    }
}

pub(crate) fn resolve_payload_cover(
    row_schema: &DistannRowSchemaDescriptor,
    indexed_vector_attnum: u16,
    requested_attnums: Option<&[u16]>,
) -> Result<Option<ResolvedPayloadCover>, String> {
    let Some(requested_attnums) = requested_attnums else {
        return Ok(None);
    };
    row_schema.validate()?;
    if indexed_vector_attnum == 0 {
        return Err(
            "EC_SCHEMA_UNSUPPORTED: covering payload resolution needs a physical vector attnum"
                .to_owned(),
        );
    }
    if requested_attnums.is_empty()
        || requested_attnums.len() > DISTANN_PAYLOAD_COVER_MAX_ATTRIBUTES
    {
        return Err(format!(
            "EC_SCHEMA_UNSUPPORTED: covering payload attribute count must be 1..={DISTANN_PAYLOAD_COVER_MAX_ATTRIBUTES}"
        ));
    }

    let mut previous = 0_u16;
    let mut attributes = Vec::with_capacity(requested_attnums.len());
    let mut value_bytes = 0_usize;
    for &attnum in requested_attnums {
        if attnum == 0 || attnum <= previous {
            return Err(
                "EC_SCHEMA_UNSUPPORTED: covering payload attnums must be positive, strictly increasing, and unique"
                    .to_owned(),
            );
        }
        if attnum == indexed_vector_attnum {
            return Err(format!(
                "EC_SCHEMA_UNSUPPORTED: covering payload attnum {attnum} is the indexed vector attribute"
            ));
        }
        let attribute = row_schema
            .attributes
            .iter()
            .find(|attribute| attribute.attnum == attnum)
            .ok_or_else(|| {
                format!(
                    "EC_SCHEMA_UNSUPPORTED: covering payload attnum {attnum} is absent from the frozen row schema"
                )
            })?;
        if attribute.dropped {
            return Err(format!(
                "EC_SCHEMA_UNSUPPORTED: covering payload attnum {attnum} is dropped"
            ));
        }
        if attribute.generated_kind != 0 {
            return Err(format!(
                "EC_SCHEMA_UNSUPPORTED: covering payload attnum {attnum} is generated"
            ));
        }
        if attribute.send_function.is_empty() || attribute.receive_function.is_empty() {
            return Err(format!(
                "EC_SCHEMA_UNSUPPORTED: covering payload attnum {attnum} lacks binary send/receive identity"
            ));
        }
        let binary_width = fixed_binary_width(attribute).ok_or_else(|| {
            format!(
                "EC_SCHEMA_UNSUPPORTED: covering payload attnum {attnum} type {}.{} is outside the fixed PG18 scalar allowlist",
                attribute.type_namespace, attribute.type_name
            )
        })?;
        value_bytes = value_bytes
            .checked_add(usize::from(binary_width))
            .ok_or_else(|| "EC_SCHEMA_UNSUPPORTED: covering payload width overflow".to_owned())?;
        attributes.push(ResolvedPayloadCoverAttribute {
            schema_attribute: attribute.clone(),
            binary_width,
        });
        previous = attnum;
    }

    let null_bytes = requested_attnums.len().div_ceil(8);
    let maximum_payload_bytes = value_bytes
        .checked_add(null_bytes)
        .ok_or_else(|| "EC_SCHEMA_UNSUPPORTED: covering payload width overflow".to_owned())?;
    if value_bytes > DISTANN_PAYLOAD_COVER_MAX_VALUE_BYTES
        || null_bytes > DISTANN_PAYLOAD_COVER_MAX_NULL_BYTES
        || maximum_payload_bytes > DISTANN_PAYLOAD_COVER_MAX_PAYLOAD_BYTES
    {
        return Err(
            "EC_SCHEMA_UNSUPPORTED: covering payload exceeds the 258-byte bound".to_owned(),
        );
    }

    Ok(Some(ResolvedPayloadCover {
        entry_format_version: DISTANN_PAYLOAD_COVER_ENTRY_VERSION,
        maximum_attribute_count: DISTANN_PAYLOAD_COVER_MAX_ATTRIBUTES as u16,
        row_schema_fingerprint: row_schema.fingerprint()?,
        attributes,
        maximum_payload_bytes: u16::try_from(maximum_payload_bytes)
            .map_err(|_| "EC_SCHEMA_UNSUPPORTED: covering payload width exceeds u16".to_owned())?,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attribute(attnum: u16, type_name: &str) -> DistannRowSchemaAttribute {
        DistannRowSchemaAttribute {
            attnum,
            name: format!("column_{attnum}"),
            type_namespace: "pg_catalog".to_owned(),
            type_name: type_name.to_owned(),
            typmod: -1,
            collation_namespace: String::new(),
            collation_name: String::new(),
            dropped: false,
            generated_kind: 0,
            send_function: format!("pg_catalog.{type_name}_send"),
            receive_function: format!("pg_catalog.{type_name}_recv"),
        }
    }

    #[test]
    fn fixed_pg18_scalar_allowlist_has_expected_widths() {
        for (type_name, width) in [
            ("bool", 1),
            ("int2", 2),
            ("int4", 4),
            ("int8", 8),
            ("float4", 4),
            ("float8", 8),
            ("uuid", 16),
            ("date", 4),
            ("time", 8),
            ("timestamp", 8),
            ("timestamptz", 8),
        ] {
            assert_eq!(fixed_binary_width(&attribute(1, type_name)), Some(width));
        }
        assert_eq!(fixed_binary_width(&attribute(1, "text")), None);
        let mut domain = attribute(1, "int8");
        domain.type_namespace = "public".to_owned();
        assert_eq!(fixed_binary_width(&domain), None);
    }

    #[test]
    fn resolution_binds_schema_and_enforces_the_258_byte_bound() {
        let schema = DistannRowSchemaDescriptor {
            attributes: (1..=16).map(|attnum| attribute(attnum, "uuid")).collect(),
        };
        let requested = (1..=16).collect::<Vec<_>>();
        let cover = resolve_payload_cover(&schema, 17, Some(&requested))
            .unwrap()
            .expect("cover should resolve");
        assert_eq!(cover.entry_format_version, 1);
        assert_eq!(cover.maximum_attribute_count, 16);
        assert_eq!(cover.maximum_payload_bytes, 258);
        assert_eq!(cover.row_schema_fingerprint, schema.fingerprint().unwrap());
        assert_eq!(cover.attributes.len(), 16);
    }

    #[test]
    fn resolution_rejects_vector_generated_dropped_and_variable_width_attributes() {
        let valid = DistannRowSchemaDescriptor {
            attributes: vec![attribute(1, "int8"), attribute(2, "uuid")],
        };
        assert!(resolve_payload_cover(&valid, 1, Some(&[1])).is_err());

        let mut generated = valid.clone();
        generated.attributes[0].generated_kind = b's';
        assert!(resolve_payload_cover(&generated, 2, Some(&[1])).is_err());

        let mut dropped = valid.clone();
        dropped.attributes[0] = DistannRowSchemaAttribute {
            attnum: 1,
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
        };
        assert!(resolve_payload_cover(&dropped, 2, Some(&[1])).is_err());

        let variable = DistannRowSchemaDescriptor {
            attributes: vec![attribute(1, "text")],
        };
        assert!(resolve_payload_cover(&variable, 2, Some(&[1])).is_err());
        assert_eq!(resolve_payload_cover(&valid, 1, None).unwrap(), None);
    }
}
