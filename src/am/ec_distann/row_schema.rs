//! FR-078 canonical epoch-row-tier schema descriptor and fingerprint.

use super::canonical_wire::{domain_digest, CanonicalDecoder, CanonicalEncoder};

pub const DISTANN_ROW_SCHEMA_VERSION: u16 = 1;
pub const DISTANN_ROW_SCHEMA_DOMAIN: &[u8] = b"ec_distann_row_schema_v1\0";
pub const DISTANN_ROW_SCHEMA_VERSION_OFFSET: usize = 0;
const DISTANN_MAX_PHYSICAL_ATTRIBUTES: usize = 1664;

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
    }
}
