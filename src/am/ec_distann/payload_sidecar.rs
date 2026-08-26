//! Task 229 covering-payload format, schema resolution, and compact entries.

use super::canonical_wire::{
    domain_digest, validate_null_bitmap, CanonicalDecoder, CanonicalEncoder,
};
use super::row_schema::{DistannRowSchemaAttribute, DistannRowSchemaDescriptor};
use crate::storage::page::ItemPointer;
use pgrx::pg_sys;

pub(crate) const DISTANN_PAYLOAD_COVER_ENTRY_VERSION: u16 = 1;
pub(crate) const DISTANN_PAYLOAD_COVER_MAX_ATTRIBUTES: usize = 16;
pub(crate) const DISTANN_PAYLOAD_COVER_MAX_VALUE_BYTES: usize = 256;
pub(crate) const DISTANN_PAYLOAD_COVER_MAX_NULL_BYTES: usize = 2;
pub(crate) const DISTANN_PAYLOAD_COVER_MAX_PAYLOAD_BYTES: usize =
    DISTANN_PAYLOAD_COVER_MAX_VALUE_BYTES + DISTANN_PAYLOAD_COVER_MAX_NULL_BYTES;
const DISTANN_PAYLOAD_COVER_DESCRIPTOR_DOMAIN: &[u8] = b"ec_distann_payload_cover_descriptor_v1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DistannPayloadCoverAttributeV1 {
    pub(crate) attnum: u16,
    pub(crate) binary_width: u16,
    pub(crate) type_namespace: String,
    pub(crate) type_name: String,
    pub(crate) typmod: i32,
    pub(crate) collation_namespace: String,
    pub(crate) collation_name: String,
    pub(crate) send_function: String,
    pub(crate) receive_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DistannPayloadCoverDescriptorV1 {
    pub(crate) entry_format_version: u16,
    pub(crate) maximum_attribute_count: u16,
    pub(crate) row_schema_fingerprint: [u8; 32],
    pub(crate) attributes: Vec<DistannPayloadCoverAttributeV1>,
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

impl DistannPayloadCoverDescriptorV1 {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.entry_format_version != DISTANN_PAYLOAD_COVER_ENTRY_VERSION
            || usize::from(self.maximum_attribute_count) != DISTANN_PAYLOAD_COVER_MAX_ATTRIBUTES
            || self.attributes.is_empty()
            || self.attributes.len() > DISTANN_PAYLOAD_COVER_MAX_ATTRIBUTES
        {
            return Err(
                "EC_GENERATION_DESCRIPTOR: invalid payload cover version or attribute count"
                    .to_owned(),
            );
        }
        let mut previous = 0_u16;
        for attribute in &self.attributes {
            if attribute.attnum == 0 || attribute.attnum <= previous {
                return Err(
                    "EC_GENERATION_DESCRIPTOR: payload cover attnums are not canonical".to_owned(),
                );
            }
            for (field, value) in [
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
                        "EC_GENERATION_DESCRIPTOR: payload cover {field} contains NUL"
                    ));
                }
            }
            if attribute.type_namespace.is_empty()
                || attribute.type_name.is_empty()
                || attribute.send_function.is_empty()
                || attribute.receive_function.is_empty()
                || !attribute.collation_namespace.is_empty()
                || !attribute.collation_name.is_empty()
            {
                return Err(format!(
                    "EC_GENERATION_DESCRIPTOR: payload cover attnum {} has incomplete or unsupported type identity",
                    attribute.attnum
                ));
            }
            let schema_attribute = DistannRowSchemaAttribute {
                attnum: attribute.attnum,
                name: "covered".to_owned(),
                type_namespace: attribute.type_namespace.clone(),
                type_name: attribute.type_name.clone(),
                typmod: attribute.typmod,
                collation_namespace: attribute.collation_namespace.clone(),
                collation_name: attribute.collation_name.clone(),
                dropped: false,
                generated_kind: 0,
                send_function: attribute.send_function.clone(),
                receive_function: attribute.receive_function.clone(),
            };
            if fixed_binary_width(&schema_attribute) != Some(attribute.binary_width) {
                return Err(format!(
                    "EC_GENERATION_DESCRIPTOR: payload cover attnum {} has unsupported type or binary width",
                    attribute.attnum
                ));
            }
            previous = attribute.attnum;
        }
        self.maximum_payload_bytes()?;
        Ok(())
    }

    pub(crate) fn maximum_payload_bytes(&self) -> Result<usize, String> {
        let value_bytes = self.attributes.iter().try_fold(0_usize, |sum, attribute| {
            sum.checked_add(usize::from(attribute.binary_width))
                .ok_or_else(|| "EC_GENERATION_DESCRIPTOR: payload cover width overflow".to_owned())
        })?;
        let null_bytes = self.attributes.len().div_ceil(8);
        let maximum = value_bytes
            .checked_add(null_bytes)
            .ok_or_else(|| "EC_GENERATION_DESCRIPTOR: payload cover width overflow".to_owned())?;
        if value_bytes > DISTANN_PAYLOAD_COVER_MAX_VALUE_BYTES
            || null_bytes > DISTANN_PAYLOAD_COVER_MAX_NULL_BYTES
            || maximum > DISTANN_PAYLOAD_COVER_MAX_PAYLOAD_BYTES
        {
            return Err(
                "EC_GENERATION_DESCRIPTOR: payload cover exceeds the 258-byte bound".to_owned(),
            );
        }
        Ok(maximum)
    }

    pub(crate) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::with_capacity(256);
        encoder.put_u16(self.entry_format_version);
        encoder.put_u16(self.maximum_attribute_count);
        encoder.put_u16(u16::try_from(self.attributes.len()).map_err(|_| {
            "EC_GENERATION_DESCRIPTOR: payload cover attribute count exceeds u16".to_owned()
        })?);
        encoder.put_fixed(&self.row_schema_fingerprint);
        for attribute in &self.attributes {
            encoder.put_u16(attribute.attnum);
            encoder.put_u16(attribute.binary_width);
            encoder.put_string(&attribute.type_namespace)?;
            encoder.put_string(&attribute.type_name)?;
            encoder.put_i32(attribute.typmod);
            encoder.put_string(&attribute.collation_namespace)?;
            encoder.put_string(&attribute.collation_name)?;
            encoder.put_string(&attribute.send_function)?;
            encoder.put_string(&attribute.receive_function)?;
        }
        encoder.finish()
    }

    pub(crate) fn decode(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "payload cover descriptor v1")?;
        let entry_format_version = decoder.get_u16("payload cover entry format version")?;
        let maximum_attribute_count = decoder.get_u16("payload cover maximum attribute count")?;
        let attribute_count = decoder.get_u16("payload cover attribute count")? as usize;
        if attribute_count == 0 || attribute_count > DISTANN_PAYLOAD_COVER_MAX_ATTRIBUTES {
            return Err(
                "EC_GENERATION_DESCRIPTOR: invalid payload cover attribute count".to_owned(),
            );
        }
        let row_schema_fingerprint = decoder.get_fixed("payload cover row schema fingerprint")?;
        let mut attributes = Vec::with_capacity(attribute_count);
        for _ in 0..attribute_count {
            attributes.push(DistannPayloadCoverAttributeV1 {
                attnum: decoder.get_u16("payload cover attnum")?,
                binary_width: decoder.get_u16("payload cover binary width")?,
                type_namespace: decoder.get_string("payload cover type namespace")?,
                type_name: decoder.get_string("payload cover type name")?,
                typmod: decoder.get_i32("payload cover typmod")?,
                collation_namespace: decoder.get_string("payload cover collation namespace")?,
                collation_name: decoder.get_string("payload cover collation name")?,
                send_function: decoder.get_string("payload cover send function")?,
                receive_function: decoder.get_string("payload cover receive function")?,
            });
        }
        decoder.finish("payload cover descriptor v1")?;
        let descriptor = Self {
            entry_format_version,
            maximum_attribute_count,
            row_schema_fingerprint,
            attributes,
        };
        descriptor.validate()?;
        Ok(descriptor)
    }

    pub(crate) fn digest(&self) -> Result<[u8; 32], String> {
        Ok(domain_digest(
            DISTANN_PAYLOAD_COVER_DESCRIPTOR_DOMAIN,
            &self.encode()?,
        ))
    }

    pub(crate) fn validate_row_schema(
        &self,
        row_schema: &DistannRowSchemaDescriptor,
    ) -> Result<(), String> {
        row_schema.validate()?;
        if self.row_schema_fingerprint != row_schema.fingerprint()? {
            return Err(
                "EC_SCHEMA_MISMATCH: payload cover row-schema fingerprint mismatch".to_owned(),
            );
        }
        for covered in &self.attributes {
            let source = row_schema
                .attributes
                .iter()
                .find(|attribute| attribute.attnum == covered.attnum)
                .ok_or_else(|| {
                    format!(
                        "EC_SCHEMA_MISMATCH: payload cover attnum {} is absent",
                        covered.attnum
                    )
                })?;
            if source.dropped
                || source.generated_kind != 0
                || source.type_namespace != covered.type_namespace
                || source.type_name != covered.type_name
                || source.typmod != covered.typmod
                || source.collation_namespace != covered.collation_namespace
                || source.collation_name != covered.collation_name
                || source.send_function != covered.send_function
                || source.receive_function != covered.receive_function
                || fixed_binary_width(source) != Some(covered.binary_width)
            {
                return Err(format!(
                    "EC_SCHEMA_MISMATCH: payload cover attnum {} differs from the frozen row schema",
                    covered.attnum
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn encode_payload(&self, values: &[Option<&[u8]>]) -> Result<Vec<u8>, String> {
        self.validate()?;
        if values.len() != self.attributes.len() {
            return Err(format!(
                "EC_GENERATION_CORRUPT: payload cover has {} values, expected {}",
                values.len(),
                self.attributes.len()
            ));
        }
        let null_bytes = values.len().div_ceil(8);
        let mut payload = vec![0_u8; null_bytes];
        for (index, (value, attribute)) in values.iter().zip(&self.attributes).enumerate() {
            match value {
                None => payload[index / 8] |= 1 << (index % 8),
                Some(value) => {
                    if value.len() != usize::from(attribute.binary_width) {
                        return Err(format!(
                            "EC_GENERATION_CORRUPT: payload cover attnum {} value is {} bytes, expected {}",
                            attribute.attnum,
                            value.len(),
                            attribute.binary_width
                        ));
                    }
                    payload.extend_from_slice(value);
                }
            }
        }
        if payload.len() > self.maximum_payload_bytes()? {
            return Err("EC_GENERATION_CORRUPT: payload cover exceeds its bound".to_owned());
        }
        Ok(payload)
    }

    pub(crate) fn decode_payload<'a>(
        &self,
        payload: &'a [u8],
    ) -> Result<Vec<Option<&'a [u8]>>, String> {
        self.validate()?;
        let null_bytes = self.attributes.len().div_ceil(8);
        if payload.len() < null_bytes {
            return Err("EC_GENERATION_CORRUPT: payload cover null bitmap is truncated".to_owned());
        }
        let bitmap = &payload[..null_bytes];
        validate_null_bitmap(
            bitmap,
            self.attributes.len(),
            "EC_GENERATION_CORRUPT",
            "payload cover null bitmap",
        )?;
        let mut position = null_bytes;
        let mut values = Vec::with_capacity(self.attributes.len());
        for (index, attribute) in self.attributes.iter().enumerate() {
            if bitmap[index / 8] & (1 << (index % 8)) != 0 {
                values.push(None);
                continue;
            }
            let end = position
                .checked_add(usize::from(attribute.binary_width))
                .ok_or_else(|| "EC_GENERATION_CORRUPT: payload cover offset overflow".to_owned())?;
            if end > payload.len() {
                return Err(format!(
                    "EC_GENERATION_CORRUPT: payload cover attnum {} is truncated",
                    attribute.attnum
                ));
            }
            values.push(Some(&payload[position..end]));
            position = end;
        }
        if position != payload.len() {
            return Err(format!(
                "EC_GENERATION_CORRUPT: payload cover has {} trailing bytes",
                payload.len() - position
            ));
        }
        Ok(values)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn decode_row<'a>(
        &self,
        requested_row_tid: ItemPointer,
        requested_vec_id: u64,
        returned_row_tid: ItemPointer,
        returned_vec_id: u64,
        payload: &'a [u8],
    ) -> Result<Vec<Option<&'a [u8]>>, String> {
        if requested_row_tid.block_number == pg_sys::InvalidBlockNumber
            || requested_row_tid.offset_number == pg_sys::InvalidOffsetNumber
        {
            return Err(
                "EC_GENERATION_CORRUPT: requested payload cover row TID is invalid".to_owned(),
            );
        }
        if returned_row_tid != requested_row_tid {
            return Err("EC_GENERATION_CORRUPT: payload cover row TID echo mismatch".to_owned());
        }
        if returned_vec_id != requested_vec_id {
            return Err("EC_GENERATION_CORRUPT: payload cover vec_id echo mismatch".to_owned());
        }
        self.decode_payload(payload)
    }
}

pub(crate) fn resolve_payload_cover(
    row_schema: &DistannRowSchemaDescriptor,
    indexed_vector_attnum: u16,
    requested_attnums: Option<&[u16]>,
) -> Result<Option<DistannPayloadCoverDescriptorV1>, String> {
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
        attributes.push(DistannPayloadCoverAttributeV1 {
            attnum,
            binary_width,
            type_namespace: attribute.type_namespace.clone(),
            type_name: attribute.type_name.clone(),
            typmod: attribute.typmod,
            collation_namespace: attribute.collation_namespace.clone(),
            collation_name: attribute.collation_name.clone(),
            send_function: attribute.send_function.clone(),
            receive_function: attribute.receive_function.clone(),
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

    let descriptor = DistannPayloadCoverDescriptorV1 {
        entry_format_version: DISTANN_PAYLOAD_COVER_ENTRY_VERSION,
        maximum_attribute_count: DISTANN_PAYLOAD_COVER_MAX_ATTRIBUTES as u16,
        row_schema_fingerprint: row_schema.fingerprint()?,
        attributes,
    };
    descriptor.validate()?;
    debug_assert_eq!(descriptor.maximum_payload_bytes()?, maximum_payload_bytes);
    descriptor.validate_row_schema(row_schema)?;
    Ok(Some(descriptor))
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
    fn resolution_binds_schema_and_attains_the_258_byte_bound() {
        let schema = DistannRowSchemaDescriptor {
            attributes: (1..=16).map(|attnum| attribute(attnum, "uuid")).collect(),
        };
        let requested = (1..=16).collect::<Vec<_>>();
        let cover = resolve_payload_cover(&schema, 17, Some(&requested))
            .unwrap()
            .expect("cover should resolve");
        assert_eq!(cover.entry_format_version, 1);
        assert_eq!(cover.maximum_attribute_count, 16);
        assert_eq!(cover.maximum_payload_bytes().unwrap(), 258);
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

    #[test]
    fn cover_descriptor_round_trips_and_binds_the_exact_row_schema() {
        let schema = DistannRowSchemaDescriptor {
            attributes: vec![attribute(1, "int8"), attribute(2, "uuid")],
        };
        let cover = resolve_payload_cover(&schema, 3, Some(&[1, 2]))
            .unwrap()
            .unwrap();
        let encoded = cover.encode().unwrap();
        assert_eq!(
            DistannPayloadCoverDescriptorV1::decode(&encoded).unwrap(),
            cover
        );
        assert_ne!(cover.digest().unwrap(), [0; 32]);
        cover.validate_row_schema(&schema).unwrap();

        let mut changed = schema.clone();
        changed.attributes[0].typmod = 9;
        assert!(cover.validate_row_schema(&changed).is_err());

        let mut trailing = encoded.clone();
        trailing.push(0);
        assert!(DistannPayloadCoverDescriptorV1::decode(&trailing).is_err());
        let mut wrong_version = encoded;
        wrong_version[..2].copy_from_slice(&2_u16.to_le_bytes());
        assert!(DistannPayloadCoverDescriptorV1::decode(&wrong_version).is_err());
    }

    #[test]
    fn compact_payload_codec_enforces_null_shape_width_and_identity_echoes() {
        let schema = DistannRowSchemaDescriptor {
            attributes: vec![attribute(1, "int8"), attribute(2, "uuid")],
        };
        let cover = resolve_payload_cover(&schema, 3, Some(&[1, 2]))
            .unwrap()
            .unwrap();
        let int8 = [0x11_u8; 8];
        let payload = cover.encode_payload(&[Some(&int8), None]).unwrap();
        assert_eq!(payload, [&[0b0000_0010][..], &int8[..]].concat());
        let decoded = cover.decode_payload(&payload).unwrap();
        assert_eq!(decoded, vec![Some(&int8[..]), None]);

        let tid = ItemPointer {
            block_number: 7,
            offset_number: 3,
        };
        assert_eq!(
            cover.decode_row(tid, 42, tid, 42, &payload).unwrap(),
            decoded
        );
        assert!(cover
            .decode_row(
                tid,
                42,
                ItemPointer {
                    block_number: 8,
                    offset_number: 3,
                },
                42,
                &payload,
            )
            .is_err());
        assert!(cover.decode_row(tid, 42, tid, 43, &payload).is_err());
        assert!(cover.encode_payload(&[Some(&int8)]).is_err());
        assert!(cover.encode_payload(&[Some(&int8[..7]), None]).is_err());

        let mut noncanonical_bitmap = payload.clone();
        noncanonical_bitmap[0] |= 1 << 7;
        assert!(cover.decode_payload(&noncanonical_bitmap).is_err());
        assert!(cover.decode_payload(&payload[..payload.len() - 1]).is_err());
        let mut trailing = payload;
        trailing.push(0);
        assert!(cover.decode_payload(&trailing).is_err());
    }
}
