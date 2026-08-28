//! Task 230 hot/cold row-tier descriptor and authoritative partition.
//!
//! This module is deliberately pure: it resolves and validates the canonical
//! layout identity but does not create relations or write generation bytes.

use super::canonical_wire::{domain_digest, CanonicalDecoder, CanonicalEncoder};
use super::row_schema::{
    fixed_binary_width, DistannRowSchemaAttribute, DistannRowSchemaDescriptor,
    DISTANN_MAX_PHYSICAL_ATTRIBUTES,
};

pub(crate) const DISTANN_ROW_TIER_LAYOUT_VERSION: u16 = 1;
pub(crate) const DISTANN_HOT_SCALAR_MAX_ATTRIBUTES: usize = 16;
pub(crate) const DISTANN_HOT_COLD_MAX_DIMENSIONS: u16 = 1_536;
pub(crate) const DISTANN_HOT_TUPLE_MAX_BYTES: u32 = 8_160;
pub(crate) const DISTANN_HOT_INTERNAL_COLUMNS: u16 = 1;
pub(crate) const DISTANN_COLD_INTERNAL_COLUMNS: u16 = 1;

const DISTANN_ROW_TIER_LAYOUT_DOMAIN: &[u8] = b"ec_distann_row_tier_layout_v1\0";
const POSTGRES_MAX_HEAP_ATTRIBUTES: usize = 1600;
const POSTGRES_HEAP_TUPLE_HEADER_BYTES: usize = 23;
const POSTGRES_MAXIMUM_ALIGNMENT: usize = 8;
const POSTGRES_VARLENA_HEADER_BYTES: usize = 4;
const DISTANN_SOURCE_IDENTITY_VALUE_BYTES: usize = 16;
const POSTGRES_SHORT_VARLENA_HEADER_BYTES: usize = 1;
const DISTANN_HOT_VEC_ID_BYTES: usize = 8;

fn identity_maximum_inline_bytes(attribute: &DistannRowSchemaAttribute) -> Option<usize> {
    match (
        attribute.type_namespace.as_str(),
        attribute.type_name.as_str(),
    ) {
        ("pg_catalog", "uuid") => Some(DISTANN_SOURCE_IDENTITY_VALUE_BYTES),
        ("pg_catalog", "bytea") => {
            Some(DISTANN_SOURCE_IDENTITY_VALUE_BYTES + POSTGRES_SHORT_VARLENA_HEADER_BYTES)
        }
        _ => None,
    }
}

fn minimum_hot_tuple_bytes(
    exact_vector_dimensions: u16,
    identity_inline_bytes: usize,
    hot_scalars: &[DistannHotScalarAttributeV1],
) -> Result<u32, String> {
    let hot_attribute_count = DISTANN_HOT_INTERNAL_COLUMNS as usize + 2 + hot_scalars.len();
    let null_bitmap_bytes = hot_attribute_count.div_ceil(8);
    let unaligned_header_bytes = POSTGRES_HEAP_TUPLE_HEADER_BYTES
        .checked_add(null_bitmap_bytes)
        .ok_or_else(|| "EC_GENERATION_DESCRIPTOR: hot tuple header overflow".to_owned())?;
    let aligned_header_bytes = unaligned_header_bytes
        .checked_add(POSTGRES_MAXIMUM_ALIGNMENT - 1)
        .ok_or_else(|| "EC_GENERATION_DESCRIPTOR: hot tuple header overflow".to_owned())?
        & !(POSTGRES_MAXIMUM_ALIGNMENT - 1);
    let vector_datum_bytes = usize::from(exact_vector_dimensions)
        .checked_mul(std::mem::size_of::<f32>())
        .and_then(|bytes| bytes.checked_add(POSTGRES_VARLENA_HEADER_BYTES))
        .ok_or_else(|| "EC_GENERATION_DESCRIPTOR: hot vector width overflow".to_owned())?;
    let scalar_bytes = hot_scalars.iter().try_fold(0_usize, |sum, scalar| {
        sum.checked_add(usize::from(scalar.binary_width))
            .ok_or_else(|| "EC_GENERATION_DESCRIPTOR: hot scalar width overflow".to_owned())
    })?;
    let minimum = aligned_header_bytes
        .checked_add(DISTANN_HOT_VEC_ID_BYTES)
        .and_then(|bytes| bytes.checked_add(vector_datum_bytes))
        .and_then(|bytes| bytes.checked_add(identity_inline_bytes))
        .and_then(|bytes| bytes.checked_add(scalar_bytes))
        .ok_or_else(|| "EC_GENERATION_DESCRIPTOR: hot tuple width overflow".to_owned())?;
    u32::try_from(minimum)
        .map_err(|_| "EC_GENERATION_DESCRIPTOR: hot tuple width exceeds u32".to_owned())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum DistannRowTierV1 {
    Hot = 1,
    Cold = 2,
}

impl DistannRowTierV1 {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(Self::Hot),
            2 => Ok(Self::Cold),
            other => Err(format!(
                "EC_GENERATION_DESCRIPTOR: unsupported row-tier placement {other}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DistannHotScalarAttributeV1 {
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

impl DistannHotScalarAttributeV1 {
    fn from_schema(attribute: &DistannRowSchemaAttribute) -> Result<Self, String> {
        let binary_width = fixed_binary_width(&attribute.type_namespace, &attribute.type_name)
            .ok_or_else(|| {
                format!(
                    "EC_SCHEMA_UNSUPPORTED: hot scalar attnum {} has unsupported type {}.{}",
                    attribute.attnum, attribute.type_namespace, attribute.type_name
                )
            })?;
        let resolved = Self {
            attnum: attribute.attnum,
            binary_width,
            type_namespace: attribute.type_namespace.clone(),
            type_name: attribute.type_name.clone(),
            typmod: attribute.typmod,
            collation_namespace: attribute.collation_namespace.clone(),
            collation_name: attribute.collation_name.clone(),
            send_function: attribute.send_function.clone(),
            receive_function: attribute.receive_function.clone(),
        };
        resolved.validate()?;
        Ok(resolved)
    }

    fn validate(&self) -> Result<(), String> {
        for (field, value) in [
            ("type namespace", self.type_namespace.as_str()),
            ("type name", self.type_name.as_str()),
            ("collation namespace", self.collation_namespace.as_str()),
            ("collation name", self.collation_name.as_str()),
            ("send function", self.send_function.as_str()),
            ("receive function", self.receive_function.as_str()),
        ] {
            if value.as_bytes().contains(&0) {
                return Err(format!(
                    "EC_GENERATION_DESCRIPTOR: hot scalar {field} contains NUL"
                ));
            }
        }
        if self.attnum == 0
            || self.type_namespace.is_empty()
            || self.type_name.is_empty()
            || self.send_function.is_empty()
            || self.receive_function.is_empty()
            || !self.collation_namespace.is_empty()
            || !self.collation_name.is_empty()
            || fixed_binary_width(&self.type_namespace, &self.type_name) != Some(self.binary_width)
        {
            return Err(format!(
                "EC_GENERATION_DESCRIPTOR: hot scalar attnum {} has incomplete or unsupported type identity",
                self.attnum
            ));
        }
        Ok(())
    }

    fn matches_schema(&self, attribute: &DistannRowSchemaAttribute) -> bool {
        !attribute.dropped
            && attribute.generated_kind == 0
            && self.attnum == attribute.attnum
            && self.type_namespace == attribute.type_namespace
            && self.type_name == attribute.type_name
            && self.typmod == attribute.typmod
            && self.collation_namespace == attribute.collation_namespace
            && self.collation_name == attribute.collation_name
            && self.send_function == attribute.send_function
            && self.receive_function == attribute.receive_function
            && fixed_binary_width(&attribute.type_namespace, &attribute.type_name)
                == Some(self.binary_width)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DistannRowTierPlacementV1 {
    pub(crate) attnum: u16,
    pub(crate) tier: DistannRowTierV1,
    /// One-based physical relation attnum, including the tier's internal
    /// prefix columns.
    pub(crate) physical_ordinal: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DistannRowTierLayoutDescriptorV1 {
    pub(crate) version: u16,
    pub(crate) maximum_hot_scalar_count: u16,
    pub(crate) exact_vector_dimensions: u16,
    pub(crate) maximum_hot_tuple_bytes: u32,
    pub(crate) source_identity_maximum_inline_bytes: u16,
    pub(crate) row_schema_fingerprint: [u8; 32],
    pub(crate) indexed_vector_attnum: u16,
    pub(crate) source_identity_attnum: u16,
    pub(crate) hot_scalars: Vec<DistannHotScalarAttributeV1>,
    pub(crate) placements: Vec<DistannRowTierPlacementV1>,
}

impl DistannRowTierLayoutDescriptorV1 {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.version != DISTANN_ROW_TIER_LAYOUT_VERSION
            || usize::from(self.maximum_hot_scalar_count) != DISTANN_HOT_SCALAR_MAX_ATTRIBUTES
            || self.exact_vector_dimensions == 0
            || self.exact_vector_dimensions > DISTANN_HOT_COLD_MAX_DIMENSIONS
            || self.maximum_hot_tuple_bytes == 0
            || self.maximum_hot_tuple_bytes > DISTANN_HOT_TUPLE_MAX_BYTES
            || self.indexed_vector_attnum == 0
            || self.source_identity_attnum == 0
            || self.indexed_vector_attnum == self.source_identity_attnum
            || self.hot_scalars.len() > DISTANN_HOT_SCALAR_MAX_ATTRIBUTES
            || self.placements.is_empty()
            || self.placements.len() > DISTANN_MAX_PHYSICAL_ATTRIBUTES
        {
            return Err(
                "EC_GENERATION_DESCRIPTOR: invalid hot/cold row-tier version or shape".to_owned(),
            );
        }

        let identity_inline_bytes = usize::from(self.source_identity_maximum_inline_bytes);
        if identity_inline_bytes != DISTANN_SOURCE_IDENTITY_VALUE_BYTES
            && identity_inline_bytes
                != DISTANN_SOURCE_IDENTITY_VALUE_BYTES + POSTGRES_SHORT_VARLENA_HEADER_BYTES
        {
            return Err(
                "EC_GENERATION_DESCRIPTOR: invalid source identity inline width".to_owned(),
            );
        }
        let minimum_hot_tuple_bytes = minimum_hot_tuple_bytes(
            self.exact_vector_dimensions,
            usize::from(self.source_identity_maximum_inline_bytes),
            &self.hot_scalars,
        )?;
        if self.maximum_hot_tuple_bytes < minimum_hot_tuple_bytes {
            return Err(format!(
                "EC_GENERATION_DESCRIPTOR: maximum hot tuple bytes {} is below descriptor-derived minimum {minimum_hot_tuple_bytes}",
                self.maximum_hot_tuple_bytes
            ));
        }

        let mut previous_hot = 0_u16;
        for scalar in &self.hot_scalars {
            scalar.validate()?;
            if scalar.attnum <= previous_hot
                || scalar.attnum == self.indexed_vector_attnum
                || scalar.attnum == self.source_identity_attnum
            {
                return Err(
                    "EC_GENERATION_DESCRIPTOR: hot scalar attnums are not canonical".to_owned(),
                );
            }
            previous_hot = scalar.attnum;
        }

        let mut previous_attnum = 0_u16;
        let mut next_hot = DISTANN_HOT_INTERNAL_COLUMNS + 1;
        let mut next_cold = DISTANN_COLD_INTERNAL_COLUMNS + 1;
        let mut vector_is_hot = false;
        let mut identity_is_hot = false;
        for placement in &self.placements {
            if placement.attnum == 0 || placement.attnum <= previous_attnum {
                return Err(
                    "EC_GENERATION_DESCRIPTOR: row-tier placements are not canonical".to_owned(),
                );
            }
            let scalar_is_hot = self
                .hot_scalars
                .binary_search_by_key(&placement.attnum, |scalar| scalar.attnum)
                .is_ok();
            match placement.tier {
                DistannRowTierV1::Hot => {
                    if placement.physical_ordinal != next_hot
                        || (placement.attnum != self.indexed_vector_attnum
                            && placement.attnum != self.source_identity_attnum
                            && !scalar_is_hot)
                    {
                        return Err(
                            "EC_GENERATION_DESCRIPTOR: invalid hot-tier physical mapping"
                                .to_owned(),
                        );
                    }
                    vector_is_hot |= placement.attnum == self.indexed_vector_attnum;
                    identity_is_hot |= placement.attnum == self.source_identity_attnum;
                    next_hot = next_hot.checked_add(1).ok_or_else(|| {
                        "EC_GENERATION_DESCRIPTOR: hot-tier ordinal overflow".to_owned()
                    })?;
                }
                DistannRowTierV1::Cold => {
                    if placement.physical_ordinal != next_cold || scalar_is_hot {
                        return Err(
                            "EC_GENERATION_DESCRIPTOR: invalid cold-tier physical mapping"
                                .to_owned(),
                        );
                    }
                    next_cold = next_cold.checked_add(1).ok_or_else(|| {
                        "EC_GENERATION_DESCRIPTOR: cold-tier ordinal overflow".to_owned()
                    })?;
                }
            }
            previous_attnum = placement.attnum;
        }
        if !vector_is_hot {
            return Err(
                "EC_GENERATION_DESCRIPTOR: indexed vector is not in the hot tier".to_owned(),
            );
        }
        if !identity_is_hot {
            return Err(
                "EC_GENERATION_DESCRIPTOR: source identity is not in the hot tier".to_owned(),
            );
        }
        if self.hot_scalars.iter().any(|scalar| {
            self.placements.iter().all(|placement| {
                placement.attnum != scalar.attnum || placement.tier != DistannRowTierV1::Hot
            })
        }) {
            return Err(
                "EC_GENERATION_DESCRIPTOR: declared hot scalar is not in the hot tier".to_owned(),
            );
        }
        if usize::from(next_hot - 1) > POSTGRES_MAX_HEAP_ATTRIBUTES
            || usize::from(next_cold - 1) > POSTGRES_MAX_HEAP_ATTRIBUTES
        {
            return Err(
                "EC_GENERATION_DESCRIPTOR: hot/cold tier exceeds PostgreSQL relation bounds"
                    .to_owned(),
            );
        }
        Ok(())
    }

    pub(crate) fn validate_row_schema(
        &self,
        row_schema: &DistannRowSchemaDescriptor,
    ) -> Result<(), String> {
        self.validate()?;
        row_schema.validate()?;
        if self.row_schema_fingerprint != row_schema.fingerprint()? {
            return Err(
                "EC_SCHEMA_MISMATCH: hot/cold layout row-schema fingerprint mismatch".to_owned(),
            );
        }
        let live = row_schema
            .attributes
            .iter()
            .filter(|attribute| !attribute.dropped)
            .collect::<Vec<_>>();
        if live.len() != self.placements.len() {
            return Err(
                "EC_SCHEMA_MISMATCH: hot/cold layout does not cover every live attribute"
                    .to_owned(),
            );
        }
        for (attribute, placement) in live.iter().zip(&self.placements) {
            if attribute.attnum != placement.attnum {
                return Err(
                    "EC_SCHEMA_MISMATCH: hot/cold layout attribute partition differs from the row schema"
                        .to_owned(),
                );
            }
        }
        let vector = live
            .iter()
            .find(|attribute| attribute.attnum == self.indexed_vector_attnum)
            .ok_or_else(|| {
                "EC_SCHEMA_MISMATCH: indexed vector is absent from the hot/cold schema".to_owned()
            })?;
        if vector.generated_kind != 0 || vector.type_name != "ecvector" {
            return Err(
                "EC_SCHEMA_MISMATCH: indexed vector is not a non-generated ecvector".to_owned(),
            );
        }
        let identity = live
            .iter()
            .find(|attribute| attribute.attnum == self.source_identity_attnum)
            .ok_or_else(|| {
                "EC_SCHEMA_MISMATCH: source identity is absent from the hot/cold schema".to_owned()
            })?;
        let identity_inline_bytes = identity_maximum_inline_bytes(identity).ok_or_else(|| {
            "EC_SCHEMA_MISMATCH: source identity is not UUID or bytea(16)".to_owned()
        })?;
        if identity.generated_kind != 0 {
            return Err("EC_SCHEMA_MISMATCH: source identity must not be generated".to_owned());
        }
        if usize::from(self.source_identity_maximum_inline_bytes) != identity_inline_bytes {
            return Err(
                "EC_SCHEMA_MISMATCH: source identity inline width differs from the row schema"
                    .to_owned(),
            );
        }
        for scalar in &self.hot_scalars {
            let attribute = live
                .iter()
                .find(|attribute| attribute.attnum == scalar.attnum)
                .ok_or_else(|| {
                    format!(
                        "EC_SCHEMA_MISMATCH: hot scalar attnum {} is absent",
                        scalar.attnum
                    )
                })?;
            if !scalar.matches_schema(attribute) {
                return Err(format!(
                    "EC_SCHEMA_MISMATCH: hot scalar attnum {} differs from the frozen row schema",
                    scalar.attnum
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::with_capacity(128 + self.placements.len() * 5);
        encoder.put_u16(self.version);
        encoder.put_u16(self.maximum_hot_scalar_count);
        encoder.put_u16(self.exact_vector_dimensions);
        encoder.put_u32(self.maximum_hot_tuple_bytes);
        encoder.put_u16(self.source_identity_maximum_inline_bytes);
        encoder.put_fixed(&self.row_schema_fingerprint);
        encoder.put_u16(self.indexed_vector_attnum);
        encoder.put_u16(self.source_identity_attnum);
        encoder
            .put_u16(u16::try_from(self.hot_scalars.len()).map_err(|_| {
                "EC_GENERATION_DESCRIPTOR: hot scalar count exceeds u16".to_owned()
            })?);
        for scalar in &self.hot_scalars {
            encoder.put_u16(scalar.attnum);
            encoder.put_u16(scalar.binary_width);
            encoder.put_string(&scalar.type_namespace)?;
            encoder.put_string(&scalar.type_name)?;
            encoder.put_i32(scalar.typmod);
            encoder.put_string(&scalar.collation_namespace)?;
            encoder.put_string(&scalar.collation_name)?;
            encoder.put_string(&scalar.send_function)?;
            encoder.put_string(&scalar.receive_function)?;
        }
        encoder.put_u16(u16::try_from(self.placements.len()).map_err(|_| {
            "EC_GENERATION_DESCRIPTOR: row-tier placement count exceeds u16".to_owned()
        })?);
        for placement in &self.placements {
            encoder.put_u16(placement.attnum);
            encoder.put_u8(placement.tier as u8);
            encoder.put_u16(placement.physical_ordinal);
        }
        encoder.finish()
    }

    pub(crate) fn decode(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "hot/cold row-tier descriptor")?;
        let version = decoder.get_u16("row-tier layout version")?;
        let maximum_hot_scalar_count = decoder.get_u16("maximum hot scalar count")?;
        let exact_vector_dimensions = decoder.get_u16("exact vector dimensions")?;
        let maximum_hot_tuple_bytes = decoder.get_u32("maximum hot tuple bytes")?;
        let source_identity_maximum_inline_bytes =
            decoder.get_u16("source identity maximum inline bytes")?;
        let row_schema_fingerprint = decoder.get_fixed("row schema fingerprint")?;
        let indexed_vector_attnum = decoder.get_u16("indexed vector attnum")?;
        let source_identity_attnum = decoder.get_u16("source identity attnum")?;
        let scalar_count = decoder.get_u16("hot scalar count")? as usize;
        if scalar_count > DISTANN_HOT_SCALAR_MAX_ATTRIBUTES {
            return Err(
                "EC_GENERATION_DESCRIPTOR: invalid hot scalar count in row-tier descriptor"
                    .to_owned(),
            );
        }
        let mut hot_scalars = Vec::with_capacity(scalar_count);
        for _ in 0..scalar_count {
            hot_scalars.push(DistannHotScalarAttributeV1 {
                attnum: decoder.get_u16("hot scalar attnum")?,
                binary_width: decoder.get_u16("hot scalar binary width")?,
                type_namespace: decoder.get_string("hot scalar type namespace")?,
                type_name: decoder.get_string("hot scalar type name")?,
                typmod: decoder.get_i32("hot scalar typmod")?,
                collation_namespace: decoder.get_string("hot scalar collation namespace")?,
                collation_name: decoder.get_string("hot scalar collation name")?,
                send_function: decoder.get_string("hot scalar send function")?,
                receive_function: decoder.get_string("hot scalar receive function")?,
            });
        }
        let placement_count = decoder.get_u16("row-tier placement count")? as usize;
        if placement_count == 0 || placement_count > DISTANN_MAX_PHYSICAL_ATTRIBUTES {
            return Err("EC_GENERATION_DESCRIPTOR: invalid row-tier placement count".to_owned());
        }
        let mut placements = Vec::with_capacity(placement_count);
        for _ in 0..placement_count {
            placements.push(DistannRowTierPlacementV1 {
                attnum: decoder.get_u16("row-tier placement attnum")?,
                tier: DistannRowTierV1::decode(decoder.get_u8("row-tier placement tier")?)?,
                physical_ordinal: decoder.get_u16("row-tier physical ordinal")?,
            });
        }
        decoder.finish("hot/cold row-tier descriptor")?;
        let descriptor = Self {
            version,
            maximum_hot_scalar_count,
            exact_vector_dimensions,
            maximum_hot_tuple_bytes,
            source_identity_maximum_inline_bytes,
            row_schema_fingerprint,
            indexed_vector_attnum,
            source_identity_attnum,
            hot_scalars,
            placements,
        };
        descriptor.validate()?;
        Ok(descriptor)
    }

    pub(crate) fn digest(&self) -> Result<[u8; 32], String> {
        Ok(domain_digest(
            DISTANN_ROW_TIER_LAYOUT_DOMAIN,
            &self.encode()?,
        ))
    }
}

pub(crate) fn resolve_hot_cold_layout(
    row_schema: &DistannRowSchemaDescriptor,
    indexed_vector_attnum: u16,
    source_identity_attnum: u16,
    exact_vector_dimensions: u16,
    maximum_hot_tuple_bytes: u32,
    requested_hot_attnums: &[u16],
) -> Result<DistannRowTierLayoutDescriptorV1, String> {
    row_schema.validate()?;
    if indexed_vector_attnum == 0
        || source_identity_attnum == 0
        || indexed_vector_attnum == source_identity_attnum
    {
        return Err(
            "EC_SCHEMA_UNSUPPORTED: hot/cold layout needs distinct physical vector and identity attnums"
                .to_owned(),
        );
    }
    if exact_vector_dimensions == 0 || exact_vector_dimensions > DISTANN_HOT_COLD_MAX_DIMENSIONS {
        return Err(format!(
            "EC_SCHEMA_UNSUPPORTED: hot/cold exact-vector dimensions must be 1..={DISTANN_HOT_COLD_MAX_DIMENSIONS}"
        ));
    }
    if maximum_hot_tuple_bytes == 0 || maximum_hot_tuple_bytes > DISTANN_HOT_TUPLE_MAX_BYTES {
        return Err(format!(
            "EC_SCHEMA_UNSUPPORTED: hot/cold maximum tuple bytes must be 1..={DISTANN_HOT_TUPLE_MAX_BYTES}"
        ));
    }
    if requested_hot_attnums.len() > DISTANN_HOT_SCALAR_MAX_ATTRIBUTES {
        return Err(format!(
            "EC_SCHEMA_UNSUPPORTED: hot scalar attribute count must be 0..={DISTANN_HOT_SCALAR_MAX_ATTRIBUTES}"
        ));
    }

    let mut hot_scalars = Vec::with_capacity(requested_hot_attnums.len());
    let mut previous = 0_u16;
    for &attnum in requested_hot_attnums {
        if attnum == 0 || attnum <= previous {
            return Err(
                "EC_SCHEMA_UNSUPPORTED: hot scalar attnums must be positive, strictly increasing, and unique"
                    .to_owned(),
            );
        }
        if attnum == indexed_vector_attnum || attnum == source_identity_attnum {
            return Err(format!(
                "EC_SCHEMA_UNSUPPORTED: hot scalar attnum {attnum} is an implicit hot attribute"
            ));
        }
        let attribute = row_schema
            .attributes
            .iter()
            .find(|attribute| attribute.attnum == attnum)
            .ok_or_else(|| {
                format!(
                    "EC_SCHEMA_UNSUPPORTED: hot scalar attnum {attnum} is absent from the frozen row schema"
                )
            })?;
        if attribute.dropped {
            return Err(format!(
                "EC_SCHEMA_UNSUPPORTED: hot scalar attnum {attnum} is dropped"
            ));
        }
        if attribute.generated_kind != 0 {
            return Err(format!(
                "EC_SCHEMA_UNSUPPORTED: hot scalar attnum {attnum} is generated"
            ));
        }
        hot_scalars.push(DistannHotScalarAttributeV1::from_schema(attribute)?);
        previous = attnum;
    }
    let vector = row_schema
        .attributes
        .iter()
        .find(|attribute| attribute.attnum == indexed_vector_attnum)
        .ok_or_else(|| {
            "EC_SCHEMA_UNSUPPORTED: indexed vector is absent from the frozen row schema".to_owned()
        })?;
    if vector.dropped || vector.generated_kind != 0 || vector.type_name != "ecvector" {
        return Err(
            "EC_SCHEMA_UNSUPPORTED: indexed vector must be a live, non-generated ecvector"
                .to_owned(),
        );
    }
    let identity = row_schema
        .attributes
        .iter()
        .find(|attribute| attribute.attnum == source_identity_attnum)
        .ok_or_else(|| {
            "EC_SCHEMA_UNSUPPORTED: source identity is absent from the frozen row schema".to_owned()
        })?;
    if identity.dropped || identity.generated_kind != 0 {
        return Err(
            "EC_SCHEMA_UNSUPPORTED: source identity must be live and non-generated".to_owned(),
        );
    }
    let source_identity_maximum_inline_bytes =
        identity_maximum_inline_bytes(identity).ok_or_else(|| {
            "EC_SCHEMA_UNSUPPORTED: source identity is not UUID or bytea(16)".to_owned()
        })?;
    let mut placements = Vec::with_capacity(row_schema.non_dropped_count());
    let mut next_hot = DISTANN_HOT_INTERNAL_COLUMNS + 1;
    let mut next_cold = DISTANN_COLD_INTERNAL_COLUMNS + 1;
    for attribute in row_schema
        .attributes
        .iter()
        .filter(|attribute| !attribute.dropped)
    {
        let tier = if attribute.attnum == indexed_vector_attnum
            || attribute.attnum == source_identity_attnum
            || requested_hot_attnums
                .binary_search(&attribute.attnum)
                .is_ok()
        {
            DistannRowTierV1::Hot
        } else {
            DistannRowTierV1::Cold
        };
        let physical_ordinal = match tier {
            DistannRowTierV1::Hot => {
                let ordinal = next_hot;
                next_hot = next_hot.checked_add(1).ok_or_else(|| {
                    "EC_GENERATION_DESCRIPTOR: hot-tier ordinal overflow".to_owned()
                })?;
                ordinal
            }
            DistannRowTierV1::Cold => {
                let ordinal = next_cold;
                next_cold = next_cold.checked_add(1).ok_or_else(|| {
                    "EC_GENERATION_DESCRIPTOR: cold-tier ordinal overflow".to_owned()
                })?;
                ordinal
            }
        };
        placements.push(DistannRowTierPlacementV1 {
            attnum: attribute.attnum,
            tier,
            physical_ordinal,
        });
    }

    let descriptor = DistannRowTierLayoutDescriptorV1 {
        version: DISTANN_ROW_TIER_LAYOUT_VERSION,
        maximum_hot_scalar_count: DISTANN_HOT_SCALAR_MAX_ATTRIBUTES as u16,
        exact_vector_dimensions,
        maximum_hot_tuple_bytes,
        source_identity_maximum_inline_bytes: u16::try_from(source_identity_maximum_inline_bytes)
            .map_err(|_| {
            "EC_SCHEMA_UNSUPPORTED: source identity width exceeds u16".to_owned()
        })?,
        row_schema_fingerprint: row_schema.fingerprint()?,
        indexed_vector_attnum,
        source_identity_attnum,
        hot_scalars,
        placements,
    };
    descriptor.validate_row_schema(row_schema)?;
    Ok(descriptor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::ec_distann::row_schema::DistannRowSchemaAttribute;

    fn attribute(
        attnum: u16,
        name: &str,
        type_namespace: &str,
        type_name: &str,
        send_function: &str,
        receive_function: &str,
    ) -> DistannRowSchemaAttribute {
        DistannRowSchemaAttribute {
            attnum,
            name: name.to_owned(),
            type_namespace: type_namespace.to_owned(),
            type_name: type_name.to_owned(),
            typmod: -1,
            collation_namespace: String::new(),
            collation_name: String::new(),
            dropped: false,
            generated_kind: 0,
            send_function: send_function.to_owned(),
            receive_function: receive_function.to_owned(),
        }
    }

    fn schema() -> DistannRowSchemaDescriptor {
        DistannRowSchemaDescriptor {
            attributes: vec![
                attribute(
                    1,
                    "id",
                    "pg_catalog",
                    "int8",
                    "pg_catalog.int8send",
                    "pg_catalog.int8recv",
                ),
                attribute(
                    2,
                    "source_id",
                    "pg_catalog",
                    "uuid",
                    "pg_catalog.uuid_send",
                    "pg_catalog.uuid_recv",
                ),
                attribute(
                    3,
                    "source",
                    "pg_catalog",
                    "_float4",
                    "pg_catalog.array_send",
                    "pg_catalog.array_recv",
                ),
                attribute(
                    4,
                    "embedding",
                    "public",
                    "ecvector",
                    "public.ecvector_send",
                    "public.ecvector_recv",
                ),
                attribute(
                    5,
                    "payload_note",
                    "pg_catalog",
                    "text",
                    "pg_catalog.textsend",
                    "pg_catalog.textrecv",
                ),
            ],
        }
    }

    fn derived_hot_tuple_bound(
        row_schema: &DistannRowSchemaDescriptor,
        exact_vector_dimensions: u16,
        source_identity_attnum: u16,
        requested_hot_attnums: &[u16],
    ) -> u32 {
        let identity = row_schema
            .attributes
            .iter()
            .find(|attribute| attribute.attnum == source_identity_attnum)
            .unwrap();
        let hot_scalars = requested_hot_attnums
            .iter()
            .map(|attnum| {
                let attribute = row_schema
                    .attributes
                    .iter()
                    .find(|attribute| attribute.attnum == *attnum)
                    .unwrap();
                DistannHotScalarAttributeV1::from_schema(attribute).unwrap()
            })
            .collect::<Vec<_>>();
        minimum_hot_tuple_bytes(
            exact_vector_dimensions,
            identity_maximum_inline_bytes(identity).unwrap(),
            &hot_scalars,
        )
        .unwrap()
    }

    #[test]
    fn layout_round_trip_partitions_every_attribute_once() {
        let schema = schema();
        let maximum_hot_tuple_bytes = derived_hot_tuple_bound(&schema, 1_536, 2, &[1]);
        let descriptor =
            resolve_hot_cold_layout(&schema, 4, 2, 1_536, maximum_hot_tuple_bytes, &[1]).unwrap();
        assert_eq!(descriptor.exact_vector_dimensions, 1_536);
        assert_eq!(descriptor.maximum_hot_tuple_bytes, maximum_hot_tuple_bytes);
        assert_eq!(
            descriptor
                .placements
                .iter()
                .map(|placement| (placement.attnum, placement.tier, placement.physical_ordinal,))
                .collect::<Vec<_>>(),
            vec![
                (1, DistannRowTierV1::Hot, 2),
                (2, DistannRowTierV1::Hot, 3),
                (3, DistannRowTierV1::Cold, 2),
                (4, DistannRowTierV1::Hot, 4),
                (5, DistannRowTierV1::Cold, 3),
            ]
        );
        let encoded = descriptor.encode().unwrap();
        let decoded = DistannRowTierLayoutDescriptorV1::decode(&encoded).unwrap();
        assert_eq!(decoded, descriptor);
        decoded.validate_row_schema(&schema).unwrap();
        assert_eq!(decoded.digest().unwrap(), descriptor.digest().unwrap());
    }

    #[test]
    fn layout_keeps_identity_implicit_and_rejects_implicit_or_variable_hot_values() {
        let schema = schema();
        let no_additional_bound = derived_hot_tuple_bound(&schema, 1_536, 2, &[]);
        let scalar_bound = derived_hot_tuple_bound(&schema, 1_536, 2, &[1]);
        let no_additional =
            resolve_hot_cold_layout(&schema, 4, 2, 1_536, no_additional_bound, &[]).unwrap();
        assert!(no_additional.hot_scalars.is_empty());
        assert!(
            resolve_hot_cold_layout(&schema, 4, 2, 1_536, scalar_bound, &[1, 2])
                .unwrap_err()
                .contains("implicit hot")
        );
        assert!(
            resolve_hot_cold_layout(&schema, 4, 2, 1_536, scalar_bound, &[1, 4])
                .unwrap_err()
                .contains("implicit hot")
        );
        assert!(
            resolve_hot_cold_layout(&schema, 4, 2, 1_536, scalar_bound, &[1, 5])
                .unwrap_err()
                .contains("unsupported type")
        );
    }

    #[test]
    fn layout_pins_dimension_tuple_and_bytea_identity_bounds() {
        let schema = schema();
        let uuid_bound = derived_hot_tuple_bound(&schema, 1_536, 2, &[1]);
        assert!(resolve_hot_cold_layout(&schema, 4, 2, 1_537, uuid_bound, &[1]).is_err());
        assert!(resolve_hot_cold_layout(&schema, 4, 2, 1_536, 8_161, &[1]).is_err());
        let descriptor = resolve_hot_cold_layout(&schema, 4, 2, 1_536, uuid_bound, &[1]).unwrap();
        let mut impossible = descriptor.clone();
        impossible.maximum_hot_tuple_bytes = 1;
        assert!(impossible
            .validate()
            .unwrap_err()
            .contains("descriptor-derived minimum"));

        let mut bytea_schema = schema;
        bytea_schema.attributes[1].type_name = "bytea".to_owned();
        bytea_schema.attributes[1].send_function = "pg_catalog.byteasend".to_owned();
        bytea_schema.attributes[1].receive_function = "pg_catalog.bytearecv".to_owned();
        let bytea_bound = derived_hot_tuple_bound(&bytea_schema, 1_536, 2, &[1]);
        assert_eq!(bytea_bound, uuid_bound + 1);
        assert!(
            resolve_hot_cold_layout(&bytea_schema, 4, 2, 1_536, uuid_bound, &[1])
                .unwrap_err()
                .contains("descriptor-derived minimum")
        );
        resolve_hot_cold_layout(&bytea_schema, 4, 2, 1_536, bytea_bound, &[1]).unwrap();
    }

    #[test]
    fn layout_decode_rejects_unknown_version_tier_and_trailing_bytes() {
        let schema = schema();
        let bound = derived_hot_tuple_bound(&schema, 1_536, 2, &[1]);
        let descriptor = resolve_hot_cold_layout(&schema, 4, 2, 1_536, bound, &[1]).unwrap();
        let mut unknown_version = descriptor.encode().unwrap();
        unknown_version[0..2].copy_from_slice(&99_u16.to_le_bytes());
        assert!(DistannRowTierLayoutDescriptorV1::decode(&unknown_version).is_err());

        let mut unknown_tier = descriptor.encode().unwrap();
        let placement_bytes = descriptor.placements.len() * 5;
        let first_tier = unknown_tier.len() - placement_bytes + 2;
        unknown_tier[first_tier] = 9;
        assert!(DistannRowTierLayoutDescriptorV1::decode(&unknown_tier).is_err());

        let mut trailing = descriptor.encode().unwrap();
        trailing.push(0);
        assert!(DistannRowTierLayoutDescriptorV1::decode(&trailing).is_err());
    }

    #[test]
    fn layout_schema_validation_detects_partition_and_type_drift() {
        let schema = schema();
        let bound = derived_hot_tuple_bound(&schema, 1_536, 2, &[1]);
        let descriptor = resolve_hot_cold_layout(&schema, 4, 2, 1_536, bound, &[1]).unwrap();

        let mut missing = descriptor.clone();
        missing.placements.pop();
        assert!(missing.validate_row_schema(&schema).is_err());

        let mut drifted = schema.clone();
        drifted.attributes[1].type_name = "int8".to_owned();
        assert!(descriptor.validate_row_schema(&drifted).is_err());

        let mut wrong_vector_schema = schema.clone();
        wrong_vector_schema.attributes[3].type_name = "text".to_owned();
        wrong_vector_schema.attributes[3].send_function = "pg_catalog.textsend".to_owned();
        wrong_vector_schema.attributes[3].receive_function = "pg_catalog.textrecv".to_owned();
        let mut wrong_vector = descriptor.clone();
        wrong_vector.row_schema_fingerprint = wrong_vector_schema.fingerprint().unwrap();
        assert!(wrong_vector
            .validate_row_schema(&wrong_vector_schema)
            .unwrap_err()
            .contains("non-generated ecvector"));

        let mut generated_vector_schema = schema.clone();
        generated_vector_schema.attributes[3].generated_kind = b's';
        let generated_bound = derived_hot_tuple_bound(&generated_vector_schema, 1_536, 2, &[1]);
        assert!(resolve_hot_cold_layout(
            &generated_vector_schema,
            4,
            2,
            1_536,
            generated_bound,
            &[1]
        )
        .unwrap_err()
        .contains("non-generated ecvector"));
    }
}
