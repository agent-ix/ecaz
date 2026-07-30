//! FR-078 canonical generation descriptor, codec artifact, and build spec.

use std::collections::HashSet;
use std::mem::size_of;

use crate::am::common::training::GroupedPq4Model;

use super::canonical_wire::{
    domain_digest, is_rfc4122_v4_uuid, CanonicalDecoder, CanonicalEncoder,
};
use super::page::{
    DISTANN_NEIGHBOR_CODEC_GROUPED_PQ, DISTANN_NEIGHBOR_CODEC_RABITQ,
    DISTANN_NEIGHBOR_CODEC_TURBOQUANT, INDEX_FORMAT_V5_DISTANN_CONTROL,
};
use super::quantizer::{DISTANN_RABITQ_BITS, DISTANN_TURBOQUANT_BITS};
use super::row_schema::DistannRowSchemaDescriptor;

pub const DISTANN_GENERATION_DESCRIPTOR_VERSION: u16 = 2;
pub const DISTANN_CODEC_ARTIFACT_VERSION: u16 = 1;
pub const DISTANN_BUILD_SPEC_VERSION: u16 = 1;
pub const DISTANN_GRAPH_RECORD_VERSION: u16 = 1;
pub const DISTANN_HANDOFF_WIRE_VERSION: u16 = 1;
pub const DISTANN_PHYSICAL_INDEX_FORMAT_VERSION: u16 = INDEX_FORMAT_V5_DISTANN_CONTROL;
pub const DISTANN_PLACEMENT_HASH_VERSION: u16 = super::placement::DISTANN_PLACEMENT_HASH_V1;
pub const DISTANN_GENERATION_DESCRIPTOR_VERSION_OFFSET: usize = 0;
pub const DISTANN_GENERATION_DESCRIPTOR_COORDINATOR_UUID_OFFSET: usize = 2;
pub const DISTANN_GENERATION_DESCRIPTOR_INDEX_FORMAT_OFFSET: usize = 18;
pub const DISTANN_GENERATION_DESCRIPTOR_GRAPH_RECORD_OFFSET: usize = 20;
pub const DISTANN_GENERATION_DESCRIPTOR_HANDOFF_WIRE_OFFSET: usize = 22;
pub const DISTANN_GENERATION_DESCRIPTOR_DIMENSIONS_OFFSET: usize = 24;
pub const DISTANN_GENERATION_DESCRIPTOR_GRAPH_DEGREE_OFFSET: usize = 26;
pub const DISTANN_GENERATION_DESCRIPTOR_PLACEMENT_HASH_OFFSET: usize = 28;
pub const DISTANN_GENERATION_DESCRIPTOR_ROSTER_COUNT_OFFSET: usize = 30;
pub const DISTANN_CODEC_ARTIFACT_VERSION_OFFSET: usize = 0;
pub const DISTANN_BUILD_SPEC_VERSION_OFFSET: usize = 0;
pub const DISTANN_GENERATION_DESCRIPTOR_FIXED_PREFIX_BYTES: usize = 30;

const GENERATION_DESCRIPTOR_DOMAIN: &[u8] = b"ec_distann_generation_descriptor_v2\0";
const ROSTER_DOMAIN: &[u8] = b"ec_distann_roster_v1\0";
const BUILD_SPEC_DOMAIN: &[u8] = b"ec_distann_build_spec_v1\0";
const GROUPED_PQ_CENTROIDS: usize = 16;
const MAX_ROSTER_COUNT: usize = 4096;
const MAX_CODEC_VALUES: usize = 16 * 1024 * 1024 / size_of::<f32>();

// Frozen validity domain for generation/build format v1. Reloption tuning may
// change independently, but decoding already-persisted v1 bytes must not. Any
// change to these bounds requires a format-version bump and new fixtures.
pub(super) const DISTANN_FORMAT_V1_MIN_GRAPH_DEGREE: u16 = 4;
pub(super) const DISTANN_FORMAT_V1_MAX_GRAPH_DEGREE: u16 = 256;
const DISTANN_FORMAT_V1_MIN_BUILD_LIST_SIZE: u16 = 10;
const DISTANN_FORMAT_V1_MAX_BUILD_LIST_SIZE: u16 = 1000;
const DISTANN_FORMAT_V1_MIN_ALPHA: f32 = 1.0;
const DISTANN_FORMAT_V1_MAX_ALPHA: f32 = 2.0;
const DISTANN_FORMAT_V1_MIN_CLOSURE_EPSILON: f32 = 0.0;
const DISTANN_FORMAT_V1_MAX_CLOSURE_EPSILON: f32 = 1.0;
const DISTANN_FORMAT_V1_MIN_HEAD_INDEX_CAP: u32 = 16;
const DISTANN_FORMAT_V1_MAX_HEAD_INDEX_CAP: u32 = 1_048_576;
const DISTANN_FORMAT_V1_MAX_BUILD_SHARDS: u32 = 4096;
const DISTANN_BUILD_OPTIONS_V1_BYTES: usize = 26;
const DISTANN_BUILD_OPTIONS_V2_VERSION: u16 = 2;
const DISTANN_BUILD_OPTIONS_V2_BYTES: usize = 65;
pub const DISTANN_TRAINING_QUERY_COUNT: u32 = 200;
pub const DISTANN_TRAINED_HEAD_INDEX_CAP: u32 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DistannHeadPolicy {
    CurrentSampleGraph = 0,
    TrainingLandmarksExact = 1,
}

impl DistannHeadPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CurrentSampleGraph => "current_sample_graph",
            Self::TrainingLandmarksExact => "training_landmarks_exact",
        }
    }

    pub(crate) fn decode_wire(value: u8) -> Result<Self, String> {
        Self::decode(value)
    }

    fn decode(value: u8) -> Result<Self, String> {
        match value {
            0 => Ok(Self::CurrentSampleGraph),
            1 => Ok(Self::TrainingLandmarksExact),
            other => Err(format!(
                "EC_GENERATION_DESCRIPTOR: unsupported head policy {other}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistannRosterEntry {
    pub node_id: u32,
    pub logical_index_uuid: [u8; 16],
    pub endpoint_identity: String,
}

pub fn validate_roster(roster: &[DistannRosterEntry]) -> Result<(), String> {
    if roster.is_empty() || roster.len() > MAX_ROSTER_COUNT {
        return Err(format!(
            "EC_NODE_DESCRIPTOR: roster count {} is outside 1..={MAX_ROSTER_COUNT}",
            roster.len()
        ));
    }
    let mut node_ids = HashSet::with_capacity(roster.len());
    let mut logical_indexes = HashSet::with_capacity(roster.len());
    let mut endpoints = HashSet::with_capacity(roster.len());
    for entry in roster {
        if entry.node_id == 0 || entry.node_id > i32::MAX as u32 {
            return Err(format!(
                "EC_NODE_DESCRIPTOR: node id {} is outside 1..={}",
                entry.node_id,
                i32::MAX
            ));
        }
        if !node_ids.insert(entry.node_id) {
            return Err(format!(
                "EC_NODE_DESCRIPTOR: duplicate roster node id {}",
                entry.node_id
            ));
        }
        if !is_rfc4122_v4_uuid(&entry.logical_index_uuid)
            || !logical_indexes.insert(entry.logical_index_uuid)
        {
            return Err("EC_NODE_DESCRIPTOR: zero or duplicate logical index UUID".to_owned());
        }
        if validate_endpoint_identity(&entry.endpoint_identity).is_err()
            || !endpoints.insert(entry.endpoint_identity.clone())
        {
            return Err(
                "EC_NODE_DESCRIPTOR: malformed, duplicate, or secret-bearing endpoint identity"
                    .to_owned(),
            );
        }
    }
    Ok(())
}

pub(crate) fn validate_endpoint_identity(value: &str) -> Result<(), String> {
    let bytes = value.as_bytes();
    let first_is_alphanumeric = bytes
        .first()
        .is_some_and(|byte| byte.is_ascii_alphanumeric());
    let tail_is_canonical = bytes
        .iter()
        .skip(1)
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'.' | b'_' | b'/' | b'-'));
    if bytes.len() > 255 || !first_is_alphanumeric || !tail_is_canonical {
        return Err(
            "EC_NODE_DESCRIPTOR: endpoint identity violates the canonical v1 grammar".to_owned(),
        );
    }
    Ok(())
}

pub(crate) fn encode_roster(
    encoder: &mut CanonicalEncoder,
    roster: &[DistannRosterEntry],
) -> Result<(), String> {
    validate_roster(roster)?;
    encoder.put_u32(
        u32::try_from(roster.len())
            .map_err(|_| "EC_NODE_DESCRIPTOR: roster count exceeds u32".to_owned())?,
    );
    for entry in roster {
        encoder.put_u32(entry.node_id);
        encoder.put_fixed(&entry.logical_index_uuid);
        encoder.put_string(&entry.endpoint_identity)?;
    }
    Ok(())
}

pub(crate) fn decode_roster(
    decoder: &mut CanonicalDecoder<'_>,
) -> Result<Vec<DistannRosterEntry>, String> {
    let count = decoder.get_u32("roster count")? as usize;
    if count == 0 || count > MAX_ROSTER_COUNT {
        return Err(format!("EC_NODE_DESCRIPTOR: invalid roster count {count}"));
    }
    let mut roster = Vec::with_capacity(count);
    for _ in 0..count {
        roster.push(DistannRosterEntry {
            node_id: decoder.get_u32("roster node id")?,
            logical_index_uuid: decoder.get_fixed("logical index UUID")?,
            endpoint_identity: decoder.get_string("endpoint identity")?,
        });
    }
    validate_roster(&roster)?;
    Ok(roster)
}

pub fn roster_digest(roster: &[DistannRosterEntry]) -> Result<[u8; 32], String> {
    let mut encoder = CanonicalEncoder::with_capacity(4 + roster.len() * 32);
    encode_roster(&mut encoder, roster)?;
    Ok(domain_digest(ROSTER_DOMAIN, &encoder.finish()?))
}

#[derive(Debug, Clone, PartialEq)]
pub enum DistannCodecArtifact {
    RaBitQ {
        dimensions: u16,
        seed: u64,
        bits: u8,
    },
    TurboQuant {
        dimensions: u16,
        seed: u64,
        bits: u8,
    },
    GroupedPq4 {
        dimensions: u16,
        seed: u64,
        model: GroupedPq4Model,
    },
}

impl DistannCodecArtifact {
    pub fn codec_kind(&self) -> u8 {
        match self {
            Self::GroupedPq4 { .. } => DISTANN_NEIGHBOR_CODEC_GROUPED_PQ,
            Self::RaBitQ { .. } => DISTANN_NEIGHBOR_CODEC_RABITQ,
            Self::TurboQuant { .. } => DISTANN_NEIGHBOR_CODEC_TURBOQUANT,
        }
    }

    pub fn dimensions(&self) -> u16 {
        match self {
            Self::RaBitQ { dimensions, .. }
            | Self::TurboQuant { dimensions, .. }
            | Self::GroupedPq4 { dimensions, .. } => *dimensions,
        }
    }

    pub fn seed(&self) -> u64 {
        match self {
            Self::RaBitQ { seed, .. }
            | Self::TurboQuant { seed, .. }
            | Self::GroupedPq4 { seed, .. } => *seed,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let dimensions = usize::from(self.dimensions());
        if dimensions == 0 {
            return Err("EC_GENERATION_DESCRIPTOR: codec dimensions must be non-zero".to_owned());
        }
        match self {
            Self::RaBitQ { bits, .. } => {
                if *bits != DISTANN_RABITQ_BITS {
                    return Err(format!(
                        "EC_GENERATION_DESCRIPTOR: unsupported RaBitQ artifact bits {bits}"
                    ));
                }
                crate::quant::rabitq::code_len_for(dimensions, *bits)
                    .map(|_| ())
                    .map_err(|error| {
                        format!(
                            "EC_GENERATION_DESCRIPTOR: unsupported RaBitQ artifact shape: {error}"
                        )
                    })
            }
            Self::TurboQuant { seed, bits, .. } => {
                if *bits != DISTANN_TURBOQUANT_BITS {
                    return Err(format!(
                        "EC_GENERATION_DESCRIPTOR: unsupported TurboQuant artifact bits {bits}"
                    ));
                }
                let quantizer = crate::quant::prod::ProdQuantizer::cached(dimensions, *bits, *seed);
                if !quantizer.int8_approx_no_qjl_4bit_supported() {
                    return Err(
                        "EC_GENERATION_DESCRIPTOR: TurboQuant artifact lacks a supported no-QJL lane"
                            .to_owned(),
                    );
                }
                Ok(())
            }
            Self::GroupedPq4 { model, .. } => {
                let expected_transform =
                    crate::quant::rotation::effective_transform_dim(dimensions);
                if model.transform_dim != expected_transform
                    || model.signs.len() != model.transform_dim
                    || model.group_count == 0
                    || model.group_size == 0
                    || model.group_count.checked_mul(model.group_size) != Some(model.transform_dim)
                    || model.codebooks.len() != model.group_count
                {
                    return Err(
                        "EC_GENERATION_DESCRIPTOR: malformed GroupedPQ4 artifact shape".to_owned(),
                    );
                }
                if model.signs.iter().any(|sign| *sign != -1.0 && *sign != 1.0) {
                    return Err(
                        "EC_GENERATION_DESCRIPTOR: GroupedPQ4 signs must be -1 or +1".to_owned(),
                    );
                }
                let expected_centroids = model
                    .group_size
                    .checked_mul(GROUPED_PQ_CENTROIDS)
                    .ok_or_else(|| {
                        "EC_GENERATION_DESCRIPTOR: GroupedPQ4 centroid size overflow".to_owned()
                    })?;
                if model.codebooks.iter().any(|codebook| {
                    codebook.len() != expected_centroids
                        || codebook.iter().any(|value| !value.is_finite())
                }) {
                    return Err(
                        "EC_GENERATION_DESCRIPTOR: malformed GroupedPQ4 codebook".to_owned()
                    );
                }
                Ok(())
            }
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::with_capacity(32);
        encoder.put_u16(DISTANN_CODEC_ARTIFACT_VERSION);
        encoder.put_u8(self.codec_kind());
        encoder.put_u16(self.dimensions());
        encoder.put_u64(self.seed());
        match self {
            Self::RaBitQ { bits, .. } | Self::TurboQuant { bits, .. } => encoder.put_u8(*bits),
            Self::GroupedPq4 { model, .. } => {
                encoder.put_u32(u32::try_from(model.transform_dim).map_err(|_| {
                    "EC_GENERATION_DESCRIPTOR: transform dimension exceeds u32".to_owned()
                })?);
                encoder.put_u32(
                    u32::try_from(model.signs.len()).map_err(|_| {
                        "EC_GENERATION_DESCRIPTOR: sign count exceeds u32".to_owned()
                    })?,
                );
                for sign in &model.signs {
                    encoder.put_f32(*sign);
                }
                encoder.put_u32(
                    u32::try_from(model.group_count).map_err(|_| {
                        "EC_GENERATION_DESCRIPTOR: group count exceeds u32".to_owned()
                    })?,
                );
                encoder.put_u32(
                    u32::try_from(model.group_size).map_err(|_| {
                        "EC_GENERATION_DESCRIPTOR: group size exceeds u32".to_owned()
                    })?,
                );
                encoder.put_u16(GROUPED_PQ_CENTROIDS as u16);
                for codebook in &model.codebooks {
                    encoder.put_u32(u32::try_from(codebook.len()).map_err(|_| {
                        "EC_GENERATION_DESCRIPTOR: centroid value count exceeds u32".to_owned()
                    })?);
                    for value in codebook {
                        encoder.put_f32(*value);
                    }
                }
            }
        }
        encoder.finish()
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "codec artifact")?;
        let version = decoder.get_u16("codec artifact version")?;
        if version != DISTANN_CODEC_ARTIFACT_VERSION {
            return Err(format!(
                "EC_GENERATION_DESCRIPTOR: unsupported codec artifact version {version}"
            ));
        }
        let codec_kind = decoder.get_u8("codec kind")?;
        let dimensions = decoder.get_u16("codec dimensions")?;
        let seed = decoder.get_u64("codec seed")?;
        let artifact = match codec_kind {
            DISTANN_NEIGHBOR_CODEC_RABITQ => Self::RaBitQ {
                dimensions,
                seed,
                bits: decoder.get_u8("RaBitQ bits")?,
            },
            DISTANN_NEIGHBOR_CODEC_TURBOQUANT => Self::TurboQuant {
                dimensions,
                seed,
                bits: decoder.get_u8("TurboQuant bits")?,
            },
            DISTANN_NEIGHBOR_CODEC_GROUPED_PQ => {
                let transform_dim = decoder.get_u32("GroupedPQ4 transform dim")? as usize;
                let sign_count = decoder.get_u32("GroupedPQ4 sign count")? as usize;
                let expected_transform =
                    crate::quant::rotation::effective_transform_dim(usize::from(dimensions));
                if dimensions == 0
                    || transform_dim != expected_transform
                    || sign_count != transform_dim
                    || sign_count > MAX_CODEC_VALUES
                    || sign_count
                        .checked_mul(size_of::<f32>())
                        .map_or(true, |bytes| bytes > decoder.remaining())
                {
                    return Err(
                        "EC_GENERATION_DESCRIPTOR: invalid GroupedPQ4 sign count".to_owned()
                    );
                }
                let mut signs = Vec::with_capacity(sign_count);
                for _ in 0..sign_count {
                    signs.push(decoder.get_f32("GroupedPQ4 sign")?);
                }
                let group_count = decoder.get_u32("GroupedPQ4 group count")? as usize;
                let group_size = decoder.get_u32("GroupedPQ4 group size")? as usize;
                let centroids = decoder.get_u16("GroupedPQ4 centroid count")? as usize;
                if group_count == 0
                    || group_count > transform_dim
                    || group_size == 0
                    || group_count.checked_mul(group_size) != Some(transform_dim)
                    || centroids != GROUPED_PQ_CENTROIDS
                {
                    return Err(
                        "EC_GENERATION_DESCRIPTOR: invalid GroupedPQ4 dimensions".to_owned()
                    );
                }
                let expected_values = group_size.checked_mul(centroids).ok_or_else(|| {
                    "EC_GENERATION_DESCRIPTOR: GroupedPQ4 value count overflow".to_owned()
                })?;
                let mut codebooks = Vec::with_capacity(group_count);
                for _ in 0..group_count {
                    let value_count = decoder.get_u32("GroupedPQ4 centroid value count")? as usize;
                    if value_count != expected_values
                        || value_count > MAX_CODEC_VALUES
                        || value_count
                            .checked_mul(size_of::<f32>())
                            .map_or(true, |bytes| bytes > decoder.remaining())
                    {
                        return Err(
                            "EC_GENERATION_DESCRIPTOR: invalid GroupedPQ4 codebook length"
                                .to_owned(),
                        );
                    }
                    let mut codebook = Vec::with_capacity(value_count);
                    for _ in 0..value_count {
                        codebook.push(decoder.get_f32("GroupedPQ4 centroid value")?);
                    }
                    codebooks.push(codebook);
                }
                Self::GroupedPq4 {
                    dimensions,
                    seed,
                    model: GroupedPq4Model {
                        codebooks,
                        group_count,
                        group_size,
                        transform_dim,
                        signs,
                    },
                }
            }
            other => {
                return Err(format!(
                    "EC_GENERATION_DESCRIPTOR: unsupported codec kind {other}"
                ))
            }
        };
        decoder.finish("codec artifact")?;
        artifact.validate()?;
        Ok(artifact)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DistannGenerationDescriptor {
    pub coordinator_logical_index_uuid: [u8; 16],
    pub index_format_version: u16,
    pub graph_record_version: u16,
    pub handoff_wire_version: u16,
    pub dimensions: u16,
    pub graph_degree: u16,
    pub placement_hash_version: u16,
    pub roster: Vec<DistannRosterEntry>,
    pub neighbor_codec_kind: u8,
    pub codec_artifact: DistannCodecArtifact,
    pub row_schema: DistannRowSchemaDescriptor,
}

impl DistannGenerationDescriptor {
    pub fn validate(&self) -> Result<(), String> {
        if !is_rfc4122_v4_uuid(&self.coordinator_logical_index_uuid)
            || self.index_format_version != DISTANN_PHYSICAL_INDEX_FORMAT_VERSION
            || self.graph_record_version != DISTANN_GRAPH_RECORD_VERSION
            || self.handoff_wire_version != DISTANN_HANDOFF_WIRE_VERSION
            || self.placement_hash_version != DISTANN_PLACEMENT_HASH_VERSION
            || self.dimensions == 0
            || self.graph_degree < DISTANN_FORMAT_V1_MIN_GRAPH_DEGREE
            || self.graph_degree > DISTANN_FORMAT_V1_MAX_GRAPH_DEGREE
        {
            return Err(
                "EC_GENERATION_DESCRIPTOR: unsupported physical format or shape".to_owned(),
            );
        }
        validate_roster(&self.roster)?;
        self.codec_artifact.validate()?;
        if self.codec_artifact.codec_kind() != self.neighbor_codec_kind
            || self.codec_artifact.dimensions() != self.dimensions
        {
            return Err("EC_GENERATION_DESCRIPTOR: codec kind/dimension mismatch".to_owned());
        }
        self.row_schema.validate()?;
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let codec_artifact = self.codec_artifact.encode()?;
        let row_schema = self.row_schema.encode()?;
        let row_schema_fingerprint = self.row_schema.fingerprint()?;
        let mut encoder = CanonicalEncoder::with_capacity(
            32 + codec_artifact.len() + row_schema.len() + self.roster.len() * 32,
        );
        encoder.put_u16(DISTANN_GENERATION_DESCRIPTOR_VERSION);
        encoder.put_fixed(&self.coordinator_logical_index_uuid);
        encoder.put_u16(self.index_format_version);
        encoder.put_u16(self.graph_record_version);
        encoder.put_u16(self.handoff_wire_version);
        encoder.put_u16(self.dimensions);
        encoder.put_u16(self.graph_degree);
        encoder.put_u16(self.placement_hash_version);
        encode_roster(&mut encoder, &self.roster)?;
        encoder.put_u8(self.neighbor_codec_kind);
        encoder.put_len_prefixed(&codec_artifact)?;
        encoder.put_len_prefixed(&row_schema)?;
        encoder.put_fixed(&row_schema_fingerprint);
        encoder.finish()
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "generation descriptor")?;
        let version = decoder.get_u16("generation descriptor version")?;
        if version != DISTANN_GENERATION_DESCRIPTOR_VERSION {
            return Err(format!(
                "EC_GENERATION_DESCRIPTOR: unsupported descriptor version {version}"
            ));
        }
        let coordinator_logical_index_uuid = decoder.get_fixed("coordinator logical index UUID")?;
        let index_format_version = decoder.get_u16("index format version")?;
        let graph_record_version = decoder.get_u16("graph record version")?;
        let handoff_wire_version = decoder.get_u16("handoff wire version")?;
        let dimensions = decoder.get_u16("dimensions")?;
        let graph_degree = decoder.get_u16("graph degree")?;
        let placement_hash_version = decoder.get_u16("placement hash version")?;
        let roster = decode_roster(&mut decoder)?;
        let neighbor_codec_kind = decoder.get_u8("neighbor codec kind")?;
        let codec_artifact =
            DistannCodecArtifact::decode(decoder.get_len_prefixed("codec artifact")?)?;
        let row_schema_bytes = decoder.get_len_prefixed("row schema descriptor")?;
        let row_schema = DistannRowSchemaDescriptor::decode(row_schema_bytes)?;
        let expected_schema_fingerprint: [u8; 32] = decoder.get_fixed("row schema fingerprint")?;
        decoder.finish("generation descriptor")?;
        if row_schema.fingerprint()? != expected_schema_fingerprint {
            return Err("EC_GENERATION_DESCRIPTOR: row schema fingerprint mismatch".to_owned());
        }
        let descriptor = Self {
            coordinator_logical_index_uuid,
            index_format_version,
            graph_record_version,
            handoff_wire_version,
            dimensions,
            graph_degree,
            placement_hash_version,
            roster,
            neighbor_codec_kind,
            codec_artifact,
            row_schema,
        };
        descriptor.validate()?;
        Ok(descriptor)
    }

    pub fn digest(&self) -> Result<[u8; 32], String> {
        Ok(domain_digest(GENERATION_DESCRIPTOR_DOMAIN, &self.encode()?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistannBuildOptions {
    pub build_list_size: u16,
    pub alpha: f32,
    pub seed: u64,
    pub closure_epsilon: f32,
    pub head_index_cap: u32,
    pub build_shards: u32,
    pub head_policy: DistannHeadPolicy,
    pub training_query_count: u32,
    pub training_query_digest: [u8; 32],
}

impl DistannBuildOptions {
    pub fn encode(self) -> Result<Vec<u8>, String> {
        if self.build_list_size < DISTANN_FORMAT_V1_MIN_BUILD_LIST_SIZE
            || self.build_list_size > DISTANN_FORMAT_V1_MAX_BUILD_LIST_SIZE
            || !self.alpha.is_finite()
            || self.alpha < DISTANN_FORMAT_V1_MIN_ALPHA
            || self.alpha > DISTANN_FORMAT_V1_MAX_ALPHA
            || !self.closure_epsilon.is_finite()
            || self.closure_epsilon.to_bits() == (-0.0_f32).to_bits()
            || self.closure_epsilon < DISTANN_FORMAT_V1_MIN_CLOSURE_EPSILON
            || self.closure_epsilon > DISTANN_FORMAT_V1_MAX_CLOSURE_EPSILON
            || self.head_index_cap < DISTANN_FORMAT_V1_MIN_HEAD_INDEX_CAP
            || self.head_index_cap > DISTANN_FORMAT_V1_MAX_HEAD_INDEX_CAP
            || self.build_shards > DISTANN_FORMAT_V1_MAX_BUILD_SHARDS
        {
            return Err("EC_GENERATION_DESCRIPTOR: invalid canonical build options".to_owned());
        }
        match self.head_policy {
            DistannHeadPolicy::CurrentSampleGraph => {
                if self.training_query_count != 0 || self.training_query_digest != [0; 32] {
                    return Err(
                        "EC_GENERATION_DESCRIPTOR: current head policy cannot bind training input"
                            .to_owned(),
                    );
                }
            }
            DistannHeadPolicy::TrainingLandmarksExact => {
                if self.head_index_cap != DISTANN_TRAINED_HEAD_INDEX_CAP
                    || self.training_query_count != DISTANN_TRAINING_QUERY_COUNT
                    || self.training_query_digest == [0; 32]
                {
                    return Err(
                        "EC_GENERATION_DESCRIPTOR: trained head policy requires cap 4096 and 200 digest-bound training queries"
                            .to_owned(),
                    );
                }
            }
        }
        let mut encoder = CanonicalEncoder::with_capacity(match self.head_policy {
            DistannHeadPolicy::CurrentSampleGraph => DISTANN_BUILD_OPTIONS_V1_BYTES,
            DistannHeadPolicy::TrainingLandmarksExact => DISTANN_BUILD_OPTIONS_V2_BYTES,
        });
        if self.head_policy == DistannHeadPolicy::TrainingLandmarksExact {
            encoder.put_u16(DISTANN_BUILD_OPTIONS_V2_VERSION);
        }
        encoder.put_u16(self.build_list_size);
        encoder.put_f32(self.alpha);
        encoder.put_u64(self.seed);
        encoder.put_f32(self.closure_epsilon);
        encoder.put_u32(self.head_index_cap);
        encoder.put_u32(self.build_shards);
        if self.head_policy == DistannHeadPolicy::TrainingLandmarksExact {
            encoder.put_u8(self.head_policy as u8);
            encoder.put_u32(self.training_query_count);
            encoder.put_fixed(&self.training_query_digest);
        }
        encoder.finish()
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() == DISTANN_BUILD_OPTIONS_V1_BYTES {
            let mut decoder = CanonicalDecoder::new(input, "build options v1")?;
            let options = Self {
                build_list_size: decoder.get_u16("build list size")?,
                alpha: decoder.get_f32("alpha")?,
                seed: decoder.get_u64("seed")?,
                closure_epsilon: decoder.get_f32("closure epsilon")?,
                head_index_cap: decoder.get_u32("head index cap")?,
                build_shards: decoder.get_u32("build shards")?,
                head_policy: DistannHeadPolicy::CurrentSampleGraph,
                training_query_count: 0,
                training_query_digest: [0; 32],
            };
            decoder.finish("build options v1")?;
            options.encode()?;
            return Ok(options);
        }
        if input.len() != DISTANN_BUILD_OPTIONS_V2_BYTES {
            return Err(format!(
                "EC_GENERATION_DESCRIPTOR: build options are {} bytes, expected {DISTANN_BUILD_OPTIONS_V1_BYTES} or {DISTANN_BUILD_OPTIONS_V2_BYTES}",
                input.len()
            ));
        }
        let mut decoder = CanonicalDecoder::new(input, "build options")?;
        let version = decoder.get_u16("build options version")?;
        if version != DISTANN_BUILD_OPTIONS_V2_VERSION {
            return Err(format!(
                "EC_GENERATION_DESCRIPTOR: unsupported build options version {version}"
            ));
        }
        let options = Self {
            build_list_size: decoder.get_u16("build list size")?,
            alpha: decoder.get_f32("alpha")?,
            seed: decoder.get_u64("seed")?,
            closure_epsilon: decoder.get_f32("closure epsilon")?,
            head_index_cap: decoder.get_u32("head index cap")?,
            build_shards: decoder.get_u32("build shards")?,
            head_policy: DistannHeadPolicy::decode(decoder.get_u8("head policy")?)?,
            training_query_count: decoder.get_u32("training query count")?,
            training_query_digest: decoder.get_fixed("training query digest")?,
        };
        decoder.finish("build options")?;
        options.encode()?;
        Ok(options)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistannOwnerExpectation {
    pub node_id: u32,
    pub expected_count: u64,
    pub expected_owner_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq)]
pub struct DistannBuildSpec {
    pub epoch: u64,
    pub build_id: [u8; 16],
    pub parent_fingerprint: Vec<u8>,
    pub source_snapshot_digest: [u8; 32],
    pub generation_descriptor_digest: [u8; 32],
    pub build_options: DistannBuildOptions,
    pub expected_global_count: u64,
    pub expected_global_graph_digest: [u8; 32],
    pub expected_global_row_tier_digest: [u8; 32],
    pub head_sample_digest: [u8; 32],
    pub owner_expectations: Vec<DistannOwnerExpectation>,
}

impl DistannBuildSpec {
    pub fn validate(&self) -> Result<(), String> {
        if self.epoch == 0 || !is_rfc4122_v4_uuid(&self.build_id) {
            return Err(
                "EC_GENERATION_DESCRIPTOR: build epoch must be non-zero and build id must be RFC 4122 v4"
                    .to_owned(),
            );
        }
        if !self.parent_fingerprint.is_empty() {
            super::manifest_v2::DistannEpochFingerprint::decode(&self.parent_fingerprint)?;
        }
        self.build_options.encode()?;
        if self.owner_expectations.is_empty() || self.owner_expectations.len() > MAX_ROSTER_COUNT {
            return Err("EC_GENERATION_DESCRIPTOR: invalid owner expectation count".to_owned());
        }
        let mut owners = HashSet::with_capacity(self.owner_expectations.len());
        let mut count_sum = 0_u64;
        for owner in &self.owner_expectations {
            if owner.node_id == 0
                || owner.node_id > i32::MAX as u32
                || !owners.insert(owner.node_id)
            {
                return Err(
                    "EC_GENERATION_DESCRIPTOR: invalid or duplicate owner expectation".to_owned(),
                );
            }
            count_sum = count_sum
                .checked_add(owner.expected_count)
                .ok_or_else(|| "EC_GENERATION_DESCRIPTOR: owner count overflow".to_owned())?;
        }
        if count_sum != self.expected_global_count {
            return Err(format!(
                "EC_GENERATION_DESCRIPTOR: owner count sum {count_sum} differs from global count {}",
                self.expected_global_count
            ));
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let build_options = self.build_options.encode()?;
        let mut encoder = CanonicalEncoder::with_capacity(256);
        encoder.put_u16(DISTANN_BUILD_SPEC_VERSION);
        encoder.put_u64(self.epoch);
        encoder.put_fixed(&self.build_id);
        encoder.put_len_prefixed(&self.parent_fingerprint)?;
        encoder.put_fixed(&self.source_snapshot_digest);
        encoder.put_fixed(&self.generation_descriptor_digest);
        encoder.put_len_prefixed(&build_options)?;
        encoder.put_u64(self.expected_global_count);
        encoder.put_fixed(&self.expected_global_graph_digest);
        encoder.put_fixed(&self.expected_global_row_tier_digest);
        encoder.put_fixed(&self.head_sample_digest);
        encoder.put_u32(u32::try_from(self.owner_expectations.len()).map_err(|_| {
            "EC_GENERATION_DESCRIPTOR: owner expectation count exceeds u32".to_owned()
        })?);
        for owner in &self.owner_expectations {
            encoder.put_u32(owner.node_id);
            encoder.put_u64(owner.expected_count);
            encoder.put_fixed(&owner.expected_owner_digest);
        }
        encoder.finish()
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "build specification")?;
        let version = decoder.get_u16("build spec version")?;
        if version != DISTANN_BUILD_SPEC_VERSION {
            return Err(format!(
                "EC_GENERATION_DESCRIPTOR: unsupported build spec version {version}"
            ));
        }
        let epoch = decoder.get_u64("epoch")?;
        let build_id = decoder.get_fixed("build id")?;
        let parent_fingerprint = decoder.get_owned_bytes("parent fingerprint")?;
        let source_snapshot_digest = decoder.get_fixed("source snapshot digest")?;
        let generation_descriptor_digest = decoder.get_fixed("generation descriptor digest")?;
        let build_options =
            DistannBuildOptions::decode(decoder.get_len_prefixed("build options")?)?;
        let expected_global_count = decoder.get_u64("expected global count")?;
        let expected_global_graph_digest = decoder.get_fixed("global graph digest")?;
        let expected_global_row_tier_digest = decoder.get_fixed("global row-tier digest")?;
        let head_sample_digest = decoder.get_fixed("head sample digest")?;
        let owner_count = decoder.get_u32("owner expectation count")? as usize;
        if owner_count == 0 || owner_count > MAX_ROSTER_COUNT {
            return Err("EC_GENERATION_DESCRIPTOR: invalid owner expectation count".to_owned());
        }
        let mut owner_expectations = Vec::with_capacity(owner_count);
        for _ in 0..owner_count {
            owner_expectations.push(DistannOwnerExpectation {
                node_id: decoder.get_u32("owner node id")?,
                expected_count: decoder.get_u64("owner expected count")?,
                expected_owner_digest: decoder.get_fixed("owner digest")?,
            });
        }
        decoder.finish("build specification")?;
        let spec = Self {
            epoch,
            build_id,
            parent_fingerprint,
            source_snapshot_digest,
            generation_descriptor_digest,
            build_options,
            expected_global_count,
            expected_global_graph_digest,
            expected_global_row_tier_digest,
            head_sample_digest,
            owner_expectations,
        };
        spec.validate()?;
        Ok(spec)
    }

    pub fn digest(&self) -> Result<[u8; 32], String> {
        Ok(domain_digest(BUILD_SPEC_DOMAIN, &self.encode()?))
    }
}

#[cfg(test)]
pub(crate) fn sample_roster() -> Vec<DistannRosterEntry> {
    fn logical_uuid(fill: u8) -> [u8; 16] {
        let mut uuid = [fill; 16];
        uuid[6] = (uuid[6] & 0x0f) | 0x40;
        uuid[8] = (uuid[8] & 0x3f) | 0x80;
        uuid
    }

    vec![
        DistannRosterEntry {
            node_id: 10,
            logical_index_uuid: logical_uuid(0x10),
            endpoint_identity: "cluster-a/node-10".to_owned(),
        },
        DistannRosterEntry {
            node_id: 20,
            logical_index_uuid: logical_uuid(0x20),
            endpoint_identity: "cluster-a/node-20".to_owned(),
        },
    ]
}

#[cfg(test)]
pub(crate) fn sample_generation_descriptor() -> DistannGenerationDescriptor {
    DistannGenerationDescriptor {
        coordinator_logical_index_uuid: super::canonical_wire::sample_rfc4122_v4_uuid(0xC0),
        index_format_version: DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
        graph_record_version: DISTANN_GRAPH_RECORD_VERSION,
        handoff_wire_version: DISTANN_HANDOFF_WIRE_VERSION,
        dimensions: 8,
        graph_degree: 4,
        placement_hash_version: DISTANN_PLACEMENT_HASH_VERSION,
        roster: sample_roster(),
        neighbor_codec_kind: DISTANN_NEIGHBOR_CODEC_RABITQ,
        codec_artifact: DistannCodecArtifact::RaBitQ {
            dimensions: 8,
            seed: 42,
            bits: DISTANN_RABITQ_BITS,
        },
        row_schema: super::row_schema::sample_row_schema(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_codec_artifacts_round_trip() {
        for artifact in [
            DistannCodecArtifact::RaBitQ {
                dimensions: 128,
                seed: 42,
                bits: DISTANN_RABITQ_BITS,
            },
            DistannCodecArtifact::TurboQuant {
                dimensions: 1536,
                seed: 42,
                bits: DISTANN_TURBOQUANT_BITS,
            },
        ] {
            let encoded = artifact.encode().unwrap();
            assert_eq!(DistannCodecArtifact::decode(&encoded).unwrap(), artifact);
        }
    }

    #[test]
    fn grouped_pq_artifact_round_trip_preserves_trained_bytes() {
        let artifact = DistannCodecArtifact::GroupedPq4 {
            dimensions: 4,
            seed: 99,
            model: GroupedPq4Model {
                codebooks: vec![(0..32).map(|value| value as f32 / 32.0).collect()],
                group_count: 1,
                group_size: 2,
                transform_dim: 2,
                signs: vec![1.0, -1.0],
            },
        };
        // A 4-dimensional input transforms to 4, so the deliberately wrong
        // transform shape above must fail before any serialization.
        assert!(artifact.encode().is_err());

        let artifact = DistannCodecArtifact::GroupedPq4 {
            dimensions: 4,
            seed: 99,
            model: GroupedPq4Model {
                codebooks: vec![
                    (0..32).map(|value| value as f32 / 32.0).collect(),
                    (0..32).map(|value| -(value as f32) / 32.0).collect(),
                ],
                group_count: 2,
                group_size: 2,
                transform_dim: 4,
                signs: vec![1.0, -1.0, 1.0, -1.0],
            },
        };
        let encoded = artifact.encode().unwrap();
        assert_eq!(DistannCodecArtifact::decode(&encoded).unwrap(), artifact);
    }

    #[test]
    fn generation_descriptor_round_trip_binds_schema_codec_and_roster() {
        let descriptor = sample_generation_descriptor();
        let encoded = descriptor.encode().unwrap();
        assert_eq!(
            DistannGenerationDescriptor::decode(&encoded).unwrap(),
            descriptor
        );
        assert_eq!(descriptor.digest().unwrap(), descriptor.digest().unwrap());

        let mut raw_conninfo = descriptor.clone();
        raw_conninfo.roster[0].endpoint_identity = "host=/tmp port=5432".to_owned();
        assert!(raw_conninfo
            .encode()
            .unwrap_err()
            .contains("EC_NODE_DESCRIPTOR"));

        let mut noncanonical_endpoint = descriptor;
        noncanonical_endpoint.roster[0].endpoint_identity = " cluster-a/node-10".to_owned();
        assert!(noncanonical_endpoint.encode().is_err());
    }

    #[test]
    fn roster_digest_is_canonical_and_order_sensitive() {
        let roster = sample_roster();
        assert_eq!(
            roster_digest(&roster).unwrap(),
            roster_digest(&roster).unwrap()
        );
        let mut reordered = roster;
        reordered.reverse();
        assert_ne!(
            roster_digest(&reordered).unwrap(),
            roster_digest(&sample_roster()).unwrap()
        );
    }

    #[test]
    fn build_spec_round_trip_and_owner_sum_validation() {
        let descriptor = sample_generation_descriptor();
        let spec = DistannBuildSpec {
            epoch: 7,
            build_id: super::super::canonical_wire::sample_rfc4122_v4_uuid(0xAB),
            parent_fingerprint: Vec::new(),
            source_snapshot_digest: [1; 32],
            generation_descriptor_digest: descriptor.digest().unwrap(),
            build_options: DistannBuildOptions {
                build_list_size: 100,
                alpha: 1.2,
                seed: 42,
                closure_epsilon: 0.3,
                head_index_cap: 4096,
                build_shards: 4,
                head_policy: DistannHeadPolicy::CurrentSampleGraph,
                training_query_count: 0,
                training_query_digest: [0; 32],
            },
            expected_global_count: 10,
            expected_global_graph_digest: [2; 32],
            expected_global_row_tier_digest: [3; 32],
            head_sample_digest: [4; 32],
            owner_expectations: vec![
                DistannOwnerExpectation {
                    node_id: 10,
                    expected_count: 6,
                    expected_owner_digest: [5; 32],
                },
                DistannOwnerExpectation {
                    node_id: 20,
                    expected_count: 4,
                    expected_owner_digest: [6; 32],
                },
            ],
        };
        let encoded = spec.encode().unwrap();
        assert_eq!(DistannBuildSpec::decode(&encoded).unwrap(), spec);
        assert_eq!(spec.digest().unwrap(), spec.digest().unwrap());

        let mut invalid_uuid = spec.clone();
        invalid_uuid.build_id = [0xAB; 16];
        assert!(invalid_uuid.encode().is_err());

        let mut invalid_parent = spec.clone();
        invalid_parent.parent_fingerprint = vec![0; 34];
        assert!(invalid_parent
            .encode()
            .unwrap_err()
            .contains("EC_EPOCH_FINGERPRINT_VERSION"));

        let mut bad = spec;
        bad.owner_expectations[0].expected_count = 5;
        assert!(bad.encode().is_err());

        let mut bad_options = bad;
        bad_options.owner_expectations[0].expected_count = 6;
        bad_options.build_options.build_list_size = 9;
        assert!(bad_options.encode().is_err());

        let mut negative_zero = bad_options;
        negative_zero.build_options.build_list_size = 10;
        negative_zero.build_options.closure_epsilon = -0.0;
        assert!(negative_zero.encode().is_err());
    }

    #[test]
    fn build_options_preserve_v1_and_bind_trained_head_inputs() {
        let current = DistannBuildOptions {
            build_list_size: 100,
            alpha: 1.2,
            seed: 42,
            closure_epsilon: 0.3,
            head_index_cap: 4096,
            build_shards: 0,
            head_policy: DistannHeadPolicy::CurrentSampleGraph,
            training_query_count: 0,
            training_query_digest: [0; 32],
        };
        let current_bytes = current.encode().unwrap();
        assert_eq!(current_bytes.len(), DISTANN_BUILD_OPTIONS_V1_BYTES);
        assert_eq!(
            DistannBuildOptions::decode(&current_bytes).unwrap(),
            current
        );

        let trained = DistannBuildOptions {
            head_policy: DistannHeadPolicy::TrainingLandmarksExact,
            training_query_count: DISTANN_TRAINING_QUERY_COUNT,
            training_query_digest: [0x7a; 32],
            ..current
        };
        let trained_bytes = trained.encode().unwrap();
        assert_eq!(trained_bytes.len(), DISTANN_BUILD_OPTIONS_V2_BYTES);
        assert_eq!(
            DistannBuildOptions::decode(&trained_bytes).unwrap(),
            trained
        );

        let mut wrong_count = trained;
        wrong_count.training_query_count -= 1;
        assert!(wrong_count.encode().is_err());
        let mut wrong_cap = trained;
        wrong_cap.head_index_cap -= 1;
        assert!(wrong_cap.encode().is_err());
        let mut unsupported = trained_bytes;
        unsupported[2 + DISTANN_BUILD_OPTIONS_V1_BYTES] = 9;
        assert!(DistannBuildOptions::decode(&unsupported).is_err());
    }
}
