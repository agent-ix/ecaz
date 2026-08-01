//! FR-082 canonical source snapshot, Ready receipt, manifest, and fingerprint.
//!
//! The physical-generation lane deliberately does not reuse the legacy v1
//! FNV manifest helper. Every value here is bounded, little-endian, and
//! independently versioned before it can become a durable generation identity.

use std::mem::size_of;

use crate::quant::{prod::mse_code_len, rabitq::code_len_for};

use super::canonical_wire::{
    domain_digest, is_rfc4122_v4_uuid, CanonicalDecoder, CanonicalEncoder,
};
use super::generation_descriptor::{
    decode_roster, encode_roster, validate_roster, DistannBuildOptions, DistannCodecArtifact,
    DistannRosterEntry, DISTANN_FORMAT_V1_MAX_GRAPH_DEGREE, DISTANN_FORMAT_V1_MIN_GRAPH_DEGREE,
    DISTANN_GRAPH_RECORD_VERSION, DISTANN_HANDOFF_WIRE_VERSION,
    DISTANN_PHYSICAL_INDEX_FORMAT_VERSION, DISTANN_PLACEMENT_HASH_VERSION,
};
use super::page::{
    DISTANN_NEIGHBOR_CODEC_GROUPED_PQ, DISTANN_NEIGHBOR_CODEC_RABITQ,
    DISTANN_NEIGHBOR_CODEC_TURBOQUANT,
};
use super::quantizer::{DISTANN_RABITQ_BITS, DISTANN_TURBOQUANT_BITS};

pub const DISTANN_SOURCE_SNAPSHOT_VERSION: u16 = 1;
pub const DISTANN_READY_RECEIPT_VERSION: u16 = 1;
pub const DISTANN_EPOCH_MANIFEST_VERSION: u16 = 2;
pub const DISTANN_MANIFEST_CODEC_PARAMETERS_VERSION: u16 = 1;
pub const DISTANN_MANIFEST_BUILD_OPTIONS_VERSION: u16 = 2;
pub const DISTANN_READY_RECEIPT_STATE: u8 = 1;
pub const DISTANN_EPOCH_FINGERPRINT_BYTES: usize = 34;
pub const DISTANN_SOURCE_SNAPSHOT_VERSION_OFFSET: usize = 0;
pub const DISTANN_READY_RECEIPT_VERSION_OFFSET: usize = 0;
pub const DISTANN_EPOCH_MANIFEST_VERSION_OFFSET: usize = 0;
pub const DISTANN_MANIFEST_CODEC_PARAMETERS_VERSION_OFFSET: usize = 0;
pub const DISTANN_MANIFEST_BUILD_OPTIONS_VERSION_OFFSET: usize = 0;
pub const DISTANN_READY_RECEIPT_BYTES: usize = 303;
pub const DISTANN_MANIFEST_CODEC_PARAMETERS_BYTES: usize = 31;
pub const DISTANN_MANIFEST_BUILD_OPTIONS_BYTES: usize = 73;

const SOURCE_SNAPSHOT_DOMAIN: &[u8] = b"ec_distann_source_snapshot_v1\0";
const READY_RECEIPT_DOMAIN: &[u8] = b"ec_distann_ready_receipt_v1\0";
const EPOCH_MANIFEST_DOMAIN: &[u8] = b"ec_distann_epoch_manifest_v2\0";
const DIGEST_BYTES: usize = 32;
const BUILD_OPTIONS_V1_BYTES: usize = 26;
const MANIFEST_BUILD_OPTIONS_V1_VERSION: u16 = 1;
const MAX_SNAPSHOT_XIDS: usize = 1_000_000;

fn validate_parent_fingerprint(parent: &[u8]) -> Result<(), String> {
    if parent.is_empty() {
        return Ok(());
    }
    DistannEpochFingerprint::decode(parent).map(|_| ())
}

fn validate_strictly_ascending(values: &[u64], field: &str) -> Result<(), String> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(format!(
            "EC_SOURCE_SNAPSHOT: {field} must be strictly ascending"
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistannSourceSnapshot {
    pub system_identifier: u64,
    pub database_name: String,
    pub xmin_full: u64,
    pub xmax_full: u64,
    pub curcid: u32,
    pub xip: Vec<u64>,
    pub subxip: Vec<u64>,
    pub suboverflowed: bool,
    pub taken_during_recovery: bool,
}

impl DistannSourceSnapshot {
    pub fn validate(&self) -> Result<(), String> {
        if self.system_identifier == 0 {
            return Err("EC_SOURCE_SNAPSHOT: cluster system identifier is zero".to_owned());
        }
        if self.database_name.is_empty() || self.database_name.as_bytes().contains(&0) {
            return Err("EC_SOURCE_SNAPSHOT: database name is empty or contains NUL".to_owned());
        }
        if self.xip.len() > MAX_SNAPSHOT_XIDS || self.subxip.len() > MAX_SNAPSHOT_XIDS {
            return Err("EC_SOURCE_SNAPSHOT: snapshot xid array exceeds its bound".to_owned());
        }
        if self.xmin_full > self.xmax_full {
            return Err("EC_SOURCE_SNAPSHOT: xmin_full exceeds xmax_full".to_owned());
        }
        if self
            .xip
            .iter()
            .chain(&self.subxip)
            .any(|xid| *xid < self.xmin_full || *xid >= self.xmax_full)
        {
            return Err(
                "EC_SOURCE_SNAPSHOT: in-progress full XID is outside [xmin_full, xmax_full)"
                    .to_owned(),
            );
        }
        validate_strictly_ascending(&self.xip, "xip")?;
        validate_strictly_ascending(&self.subxip, "subxip")?;
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::with_capacity(
            44 + self.database_name.len() + (self.xip.len() + self.subxip.len()) * 8,
        );
        encoder.put_u16(DISTANN_SOURCE_SNAPSHOT_VERSION);
        encoder.put_u64(self.system_identifier);
        encoder.put_string(&self.database_name)?;
        encoder.put_u64(self.xmin_full);
        encoder.put_u64(self.xmax_full);
        encoder.put_u32(self.curcid);
        encoder.put_u32(
            u32::try_from(self.xip.len())
                .map_err(|_| "EC_SOURCE_SNAPSHOT: xip count exceeds u32".to_owned())?,
        );
        for xid in &self.xip {
            encoder.put_u64(*xid);
        }
        encoder.put_u32(
            u32::try_from(self.subxip.len())
                .map_err(|_| "EC_SOURCE_SNAPSHOT: subxip count exceeds u32".to_owned())?,
        );
        for xid in &self.subxip {
            encoder.put_u64(*xid);
        }
        encoder.put_u8(u8::from(self.suboverflowed));
        encoder.put_u8(u8::from(self.taken_during_recovery));
        encoder.finish()
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "source snapshot")?;
        let version = decoder.get_u16("source snapshot version")?;
        if version != DISTANN_SOURCE_SNAPSHOT_VERSION {
            return Err(format!(
                "EC_SOURCE_SNAPSHOT: unsupported snapshot version {version}"
            ));
        }
        let system_identifier = decoder.get_u64("cluster system identifier")?;
        let database_name = decoder.get_string("database name")?;
        let xmin_full = decoder.get_u64("snapshot full xmin")?;
        let xmax_full = decoder.get_u64("snapshot full xmax")?;
        let curcid = decoder.get_u32("snapshot curcid")?;
        let xip_count = decoder.get_u32("snapshot xip count")? as usize;
        if xip_count > MAX_SNAPSHOT_XIDS
            || xip_count
                .checked_mul(size_of::<u64>())
                .map_or(true, |bytes| bytes > decoder.remaining())
        {
            return Err("EC_SOURCE_SNAPSHOT: invalid xip count".to_owned());
        }
        let mut xip = Vec::with_capacity(xip_count);
        for _ in 0..xip_count {
            xip.push(decoder.get_u64("snapshot xip")?);
        }
        let subxip_count = decoder.get_u32("snapshot subxip count")? as usize;
        if subxip_count > MAX_SNAPSHOT_XIDS
            || subxip_count
                .checked_mul(size_of::<u64>())
                .map_or(true, |bytes| bytes > decoder.remaining())
        {
            return Err("EC_SOURCE_SNAPSHOT: invalid subxip count".to_owned());
        }
        let mut subxip = Vec::with_capacity(subxip_count);
        for _ in 0..subxip_count {
            subxip.push(decoder.get_u64("snapshot subxip")?);
        }
        let suboverflowed = match decoder.get_u8("snapshot suboverflowed")? {
            0 => false,
            1 => true,
            other => {
                return Err(format!(
                    "EC_SOURCE_SNAPSHOT: invalid suboverflowed flag {other}"
                ))
            }
        };
        let taken_during_recovery = match decoder.get_u8("snapshot recovery flag")? {
            0 => false,
            1 => true,
            other => return Err(format!("EC_SOURCE_SNAPSHOT: invalid recovery flag {other}")),
        };
        decoder.finish("source snapshot")?;
        let snapshot = Self {
            system_identifier,
            database_name,
            xmin_full,
            xmax_full,
            curcid,
            xip,
            subxip,
            suboverflowed,
            taken_during_recovery,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    pub fn digest(&self) -> Result<[u8; DIGEST_BYTES], String> {
        Ok(domain_digest(SOURCE_SNAPSHOT_DOMAIN, &self.encode()?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistannManifestCodecParameters {
    pub codec_kind: u8,
    pub dimensions: u16,
    pub code_stride: u32,
    pub seed: u64,
    pub transform_dim: u32,
    pub group_count: u32,
    pub group_size: u32,
    pub centroids_per_group: u16,
}

impl DistannManifestCodecParameters {
    pub fn validate(self) -> Result<(), String> {
        if self.dimensions == 0 || self.code_stride == 0 {
            return Err("EC_EPOCH_MANIFEST: codec dimension/stride is zero".to_owned());
        }
        match self.codec_kind {
            DISTANN_NEIGHBOR_CODEC_RABITQ | DISTANN_NEIGHBOR_CODEC_TURBOQUANT => {
                if self.transform_dim != 0
                    || self.group_count != 0
                    || self.group_size != 0
                    || self.centroids_per_group != 0
                {
                    return Err(
                        "EC_EPOCH_MANIFEST: seeded codec carries GroupedPQ shape".to_owned()
                    );
                }
                let expected_stride = if self.codec_kind == DISTANN_NEIGHBOR_CODEC_RABITQ {
                    code_len_for(usize::from(self.dimensions), DISTANN_RABITQ_BITS).map_err(
                        |error| format!("EC_EPOCH_MANIFEST: invalid RaBitQ shape: {error}"),
                    )?
                } else {
                    mse_code_len(usize::from(self.dimensions), DISTANN_TURBOQUANT_BITS)
                };
                if usize::try_from(self.code_stride).ok() != Some(expected_stride) {
                    return Err(format!(
                        "EC_EPOCH_MANIFEST: codec stride {} differs from canonical {expected_stride}",
                        self.code_stride
                    ));
                }
            }
            DISTANN_NEIGHBOR_CODEC_GROUPED_PQ => {
                let expected_transform =
                    crate::quant::rotation::effective_transform_dim(usize::from(self.dimensions));
                if self.transform_dim == 0
                    || usize::try_from(self.transform_dim).ok() != Some(expected_transform)
                    || self.group_count == 0
                    || self.group_size == 0
                    || self.group_count.checked_mul(self.group_size) != Some(self.transform_dim)
                    || self.code_stride != self.group_count.div_ceil(2)
                    || self.centroids_per_group != 16
                {
                    return Err("EC_EPOCH_MANIFEST: invalid GroupedPQ codec shape".to_owned());
                }
            }
            other => return Err(format!("EC_EPOCH_MANIFEST: unsupported codec kind {other}")),
        }
        Ok(())
    }

    pub fn encode(self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::with_capacity(31);
        encoder.put_u16(DISTANN_MANIFEST_CODEC_PARAMETERS_VERSION);
        encoder.put_u8(self.codec_kind);
        encoder.put_u16(self.dimensions);
        encoder.put_u32(self.code_stride);
        encoder.put_u64(self.seed);
        encoder.put_u32(self.transform_dim);
        encoder.put_u32(self.group_count);
        encoder.put_u32(self.group_size);
        encoder.put_u16(self.centroids_per_group);
        encoder.finish()
    }

    /// The manifest repeats a compact codec summary while the generation
    /// descriptor owns the complete artifact. Publication must compare the
    /// plaintext fields, not merely trust that two unrelated digests exist.
    pub fn validate_artifact(self, artifact: &DistannCodecArtifact) -> Result<(), String> {
        artifact.validate()?;
        if self.codec_kind != artifact.codec_kind()
            || self.dimensions != artifact.dimensions()
            || self.seed != artifact.seed()
        {
            return Err(
                "EC_EPOCH_MANIFEST: codec parameters disagree with generation artifact identity"
                    .to_owned(),
            );
        }
        let artifact_stride = match artifact {
            DistannCodecArtifact::RaBitQ { bits, .. } => {
                code_len_for(usize::from(self.dimensions), *bits).map_err(|error| {
                    format!("EC_EPOCH_MANIFEST: invalid RaBitQ artifact stride: {error}")
                })?
            }
            DistannCodecArtifact::TurboQuant { bits, .. } => {
                mse_code_len(usize::from(self.dimensions), *bits)
            }
            DistannCodecArtifact::GroupedPq4 { model, .. } => model.group_count.div_ceil(2),
        };
        if usize::try_from(self.code_stride).ok() != Some(artifact_stride) {
            return Err(
                "EC_EPOCH_MANIFEST: codec stride disagrees with generation artifact".to_owned(),
            );
        }
        if let DistannCodecArtifact::GroupedPq4 { model, .. } = artifact {
            if usize::try_from(self.transform_dim).ok() != Some(model.transform_dim)
                || usize::try_from(self.group_count).ok() != Some(model.group_count)
                || usize::try_from(self.group_size).ok() != Some(model.group_size)
                || self.centroids_per_group != 16
            {
                return Err(
                    "EC_EPOCH_MANIFEST: GroupedPQ parameters disagree with generation artifact"
                        .to_owned(),
                );
            }
        }
        self.validate()
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "manifest codec parameters")?;
        let version = decoder.get_u16("manifest codec parameter version")?;
        if version != DISTANN_MANIFEST_CODEC_PARAMETERS_VERSION {
            return Err(format!(
                "EC_EPOCH_MANIFEST: unsupported codec parameter version {version}"
            ));
        }
        let parameters = Self {
            codec_kind: decoder.get_u8("codec kind")?,
            dimensions: decoder.get_u16("codec dimensions")?,
            code_stride: decoder.get_u32("codec stride")?,
            seed: decoder.get_u64("codec seed")?,
            transform_dim: decoder.get_u32("codec transform dimension")?,
            group_count: decoder.get_u32("codec group count")?,
            group_size: decoder.get_u32("codec group size")?,
            centroids_per_group: decoder.get_u16("codec centroids per group")?,
        };
        decoder.finish("manifest codec parameters")?;
        parameters.validate()?;
        Ok(parameters)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistannManifestBuildOptions {
    pub graph_degree: u16,
    pub options: DistannBuildOptions,
}

impl DistannManifestBuildOptions {
    pub fn encode(self) -> Result<Vec<u8>, String> {
        if self.graph_degree < DISTANN_FORMAT_V1_MIN_GRAPH_DEGREE
            || self.graph_degree > DISTANN_FORMAT_V1_MAX_GRAPH_DEGREE
        {
            return Err(
                "EC_EPOCH_MANIFEST: graph/build options are outside the frozen format-v1 bounds"
                    .to_owned(),
            );
        }
        let options = self.options.encode()?;
        let legacy = options.len() == BUILD_OPTIONS_V1_BYTES;
        let mut encoder = CanonicalEncoder::with_capacity(8 + options.len());
        encoder.put_u16(if legacy {
            MANIFEST_BUILD_OPTIONS_V1_VERSION
        } else {
            DISTANN_MANIFEST_BUILD_OPTIONS_VERSION
        });
        encoder.put_u16(self.graph_degree);
        if legacy {
            encoder.put_fixed(&options);
        } else {
            encoder.put_len_prefixed(&options)?;
        }
        encoder.finish()
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "manifest build options")?;
        let version = decoder.get_u16("manifest build options version")?;
        let graph_degree = decoder.get_u16("graph degree")?;
        let options = match version {
            MANIFEST_BUILD_OPTIONS_V1_VERSION => DistannBuildOptions::decode(
                decoder.get_bytes(BUILD_OPTIONS_V1_BYTES, "canonical build options")?,
            )?,
            DISTANN_MANIFEST_BUILD_OPTIONS_VERSION => {
                DistannBuildOptions::decode(decoder.get_len_prefixed("canonical build options")?)?
            }
            _ => {
                return Err(format!(
                    "EC_EPOCH_MANIFEST: unsupported build options version {version}"
                ))
            }
        };
        decoder.finish("manifest build options")?;
        let value = Self {
            graph_degree,
            options,
        };
        value.encode()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistannReadyReceipt {
    pub node_id: u32,
    pub epoch: u64,
    pub build_id: [u8; 16],
    pub build_spec_digest: [u8; DIGEST_BYTES],
    pub generation_descriptor_digest: [u8; DIGEST_BYTES],
    pub last_acknowledged_batch_sequence: u64,
    pub owned_record_count: u64,
    pub row_count: u64,
    pub owner_stream_digest: [u8; DIGEST_BYTES],
    pub persisted_graph_digest: [u8; DIGEST_BYTES],
    pub persisted_row_tier_digest: [u8; DIGEST_BYTES],
    pub local_directory_digest: [u8; DIGEST_BYTES],
    pub graph_bytes: u64,
    pub row_tier_bytes: u64,
    pub directory_bytes: u64,
    pub state: u8,
}

impl DistannReadyReceipt {
    pub fn validate(&self) -> Result<(), String> {
        if self.node_id == 0 || self.node_id > i32::MAX as u32 {
            return Err(format!(
                "EC_READY_RECEIPT: node id {} is outside 1..={}",
                self.node_id,
                i32::MAX
            ));
        }
        if self.epoch == 0 || !is_rfc4122_v4_uuid(&self.build_id) {
            return Err(
                "EC_READY_RECEIPT: epoch must be non-zero and build id must be RFC 4122 v4"
                    .to_owned(),
            );
        }
        if self.owned_record_count != self.row_count {
            return Err(format!(
                "EC_READY_RECEIPT: record count {} differs from row count {}",
                self.owned_record_count, self.row_count
            ));
        }
        if self.state != DISTANN_READY_RECEIPT_STATE {
            return Err(format!(
                "EC_READY_RECEIPT: state {} is not Ready",
                self.state
            ));
        }
        Ok(())
    }

    fn canonical_without_digest(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::with_capacity(271);
        encoder.put_u16(DISTANN_READY_RECEIPT_VERSION);
        encoder.put_u32(self.node_id);
        encoder.put_u64(self.epoch);
        encoder.put_fixed(&self.build_id);
        encoder.put_fixed(&self.build_spec_digest);
        encoder.put_fixed(&self.generation_descriptor_digest);
        encoder.put_u64(self.last_acknowledged_batch_sequence);
        encoder.put_u64(self.owned_record_count);
        encoder.put_u64(self.row_count);
        encoder.put_fixed(&self.owner_stream_digest);
        encoder.put_fixed(&self.persisted_graph_digest);
        encoder.put_fixed(&self.persisted_row_tier_digest);
        encoder.put_fixed(&self.local_directory_digest);
        encoder.put_u64(self.graph_bytes);
        encoder.put_u64(self.row_tier_bytes);
        encoder.put_u64(self.directory_bytes);
        encoder.put_u8(self.state);
        encoder.finish()
    }

    pub fn digest(&self) -> Result<[u8; DIGEST_BYTES], String> {
        Ok(domain_digest(
            READY_RECEIPT_DOMAIN,
            &self.canonical_without_digest()?,
        ))
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        let mut encoded = self.canonical_without_digest()?;
        encoded.extend_from_slice(&domain_digest(READY_RECEIPT_DOMAIN, &encoded));
        Ok(encoded)
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() < DIGEST_BYTES {
            return Err("EC_READY_RECEIPT: receipt is shorter than its digest".to_owned());
        }
        let canonical_length = input.len() - DIGEST_BYTES;
        let (canonical, supplied_digest) = input.split_at(canonical_length);
        if supplied_digest != domain_digest(READY_RECEIPT_DOMAIN, canonical) {
            return Err("EC_READY_RECEIPT: receipt digest mismatch".to_owned());
        }
        let mut decoder = CanonicalDecoder::new(canonical, "Ready receipt")?;
        let version = decoder.get_u16("Ready receipt version")?;
        if version != DISTANN_READY_RECEIPT_VERSION {
            return Err(format!(
                "EC_READY_RECEIPT: unsupported receipt version {version}"
            ));
        }
        let receipt = Self {
            node_id: decoder.get_u32("Ready receipt node id")?,
            epoch: decoder.get_u64("Ready receipt epoch")?,
            build_id: decoder.get_fixed("Ready receipt build id")?,
            build_spec_digest: decoder.get_fixed("Ready receipt build spec digest")?,
            generation_descriptor_digest: decoder
                .get_fixed("Ready receipt generation descriptor digest")?,
            last_acknowledged_batch_sequence: decoder
                .get_u64("Ready receipt last batch sequence")?,
            owned_record_count: decoder.get_u64("Ready receipt record count")?,
            row_count: decoder.get_u64("Ready receipt row count")?,
            owner_stream_digest: decoder.get_fixed("Ready receipt owner stream digest")?,
            persisted_graph_digest: decoder.get_fixed("Ready receipt graph digest")?,
            persisted_row_tier_digest: decoder.get_fixed("Ready receipt row-tier digest")?,
            local_directory_digest: decoder.get_fixed("Ready receipt directory digest")?,
            graph_bytes: decoder.get_u64("Ready receipt graph bytes")?,
            row_tier_bytes: decoder.get_u64("Ready receipt row-tier bytes")?,
            directory_bytes: decoder.get_u64("Ready receipt directory bytes")?,
            state: decoder.get_u8("Ready receipt state")?,
        };
        decoder.finish("Ready receipt")?;
        receipt.validate()?;
        Ok(receipt)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DistannEpochFingerprint([u8; DISTANN_EPOCH_FINGERPRINT_BYTES]);

impl DistannEpochFingerprint {
    pub fn from_manifest_digest(digest: [u8; DIGEST_BYTES]) -> Self {
        let mut bytes = [0_u8; DISTANN_EPOCH_FINGERPRINT_BYTES];
        bytes[0..2].copy_from_slice(&DISTANN_EPOCH_MANIFEST_VERSION.to_le_bytes());
        bytes[2..].copy_from_slice(&digest);
        Self(bytes)
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != DISTANN_EPOCH_FINGERPRINT_BYTES {
            return Err(format!(
                "EC_EPOCH_FINGERPRINT_VERSION: fingerprint is {} bytes, expected {DISTANN_EPOCH_FINGERPRINT_BYTES}",
                input.len()
            ));
        }
        let version = u16::from_le_bytes(input[0..2].try_into().expect("fingerprint version"));
        if version != DISTANN_EPOCH_MANIFEST_VERSION {
            return Err(format!(
                "EC_EPOCH_FINGERPRINT_VERSION: unsupported fingerprint version {version}"
            ));
        }
        Ok(Self(
            input
                .try_into()
                .expect("fingerprint length validated before conversion"),
        ))
    }

    pub fn as_bytes(&self) -> &[u8; DISTANN_EPOCH_FINGERPRINT_BYTES] {
        &self.0
    }

    pub fn manifest_digest(&self) -> [u8; DIGEST_BYTES] {
        self.0[2..]
            .try_into()
            .expect("fingerprint always contains a 32-byte digest")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DistannEpochManifestV2 {
    pub epoch: u64,
    pub build_id: [u8; 16],
    pub parent_fingerprint: Vec<u8>,
    pub source_snapshot_digest: [u8; DIGEST_BYTES],
    pub build_spec_digest: [u8; DIGEST_BYTES],
    pub generation_descriptor_digest: [u8; DIGEST_BYTES],
    pub placement_hash_version: u16,
    pub roster: Vec<DistannRosterEntry>,
    pub index_format_version: u16,
    pub graph_record_version: u16,
    pub handoff_wire_version: u16,
    pub codec_parameters: DistannManifestCodecParameters,
    pub build_options: DistannManifestBuildOptions,
    pub row_schema_fingerprint: [u8; DIGEST_BYTES],
    pub head_sample_digest: [u8; DIGEST_BYTES],
    pub global_record_count: u64,
    pub global_graph_digest: [u8; DIGEST_BYTES],
    pub global_row_tier_digest: [u8; DIGEST_BYTES],
    pub participant_receipts: Vec<DistannReadyReceipt>,
}

impl DistannEpochManifestV2 {
    pub fn validate(&self) -> Result<(), String> {
        if self.epoch == 0 || !is_rfc4122_v4_uuid(&self.build_id) {
            return Err(
                "EC_EPOCH_MANIFEST: epoch must be non-zero and build id must be RFC 4122 v4"
                    .to_owned(),
            );
        }
        validate_parent_fingerprint(&self.parent_fingerprint)?;
        if self.placement_hash_version != DISTANN_PLACEMENT_HASH_VERSION
            || self.index_format_version != DISTANN_PHYSICAL_INDEX_FORMAT_VERSION
            || self.graph_record_version != DISTANN_GRAPH_RECORD_VERSION
            || self.handoff_wire_version != DISTANN_HANDOFF_WIRE_VERSION
        {
            return Err("EC_EPOCH_MANIFEST: unsupported physical format version".to_owned());
        }
        validate_roster(&self.roster)?;
        self.codec_parameters.validate()?;
        self.build_options.encode()?;
        if self.participant_receipts.len() != self.roster.len() {
            return Err(format!(
                "EC_EPOCH_MANIFEST: {} receipts differ from {} roster entries",
                self.participant_receipts.len(),
                self.roster.len()
            ));
        }

        let mut record_sum = 0_u64;
        for (roster, receipt) in self.roster.iter().zip(&self.participant_receipts) {
            receipt.validate()?;
            if receipt.node_id != roster.node_id
                || receipt.epoch != self.epoch
                || receipt.build_id != self.build_id
                || receipt.build_spec_digest != self.build_spec_digest
                || receipt.generation_descriptor_digest != self.generation_descriptor_digest
            {
                return Err(format!(
                    "EC_EPOCH_MANIFEST: receipt for node {} disagrees with its roster/epoch/build identities",
                    receipt.node_id
                ));
            }
            record_sum = record_sum
                .checked_add(receipt.owned_record_count)
                .ok_or_else(|| "EC_EPOCH_MANIFEST: participant record count overflow".to_owned())?;
        }
        if record_sum != self.global_record_count {
            return Err(format!(
                "EC_EPOCH_MANIFEST: participant record sum {record_sum} differs from global count {}",
                self.global_record_count
            ));
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let codec_parameters = self.codec_parameters.encode()?;
        let build_options = self.build_options.encode()?;
        let encoded_receipts = self
            .participant_receipts
            .iter()
            .map(DistannReadyReceipt::encode)
            .collect::<Result<Vec<_>, _>>()?;
        let mut encoder = CanonicalEncoder::with_capacity(
            320 + codec_parameters.len()
                + build_options.len()
                + self.roster.len() * 48
                + encoded_receipts.iter().map(Vec::len).sum::<usize>(),
        );
        encoder.put_u16(DISTANN_EPOCH_MANIFEST_VERSION);
        encoder.put_u64(self.epoch);
        encoder.put_fixed(&self.build_id);
        encoder.put_len_prefixed(&self.parent_fingerprint)?;
        encoder.put_fixed(&self.source_snapshot_digest);
        encoder.put_fixed(&self.build_spec_digest);
        encoder.put_fixed(&self.generation_descriptor_digest);
        encoder.put_u16(self.placement_hash_version);
        encode_roster(&mut encoder, &self.roster)?;
        encoder.put_u16(self.index_format_version);
        encoder.put_u16(self.graph_record_version);
        encoder.put_u16(self.handoff_wire_version);
        encoder.put_len_prefixed(&codec_parameters)?;
        encoder.put_len_prefixed(&build_options)?;
        encoder.put_fixed(&self.row_schema_fingerprint);
        encoder.put_fixed(&self.head_sample_digest);
        encoder.put_u64(self.global_record_count);
        encoder.put_fixed(&self.global_graph_digest);
        encoder.put_fixed(&self.global_row_tier_digest);
        encoder.put_u32(
            u32::try_from(encoded_receipts.len())
                .map_err(|_| "EC_EPOCH_MANIFEST: receipt count exceeds u32".to_owned())?,
        );
        for receipt in encoded_receipts {
            encoder.put_len_prefixed(&receipt)?;
        }
        encoder.finish()
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "epoch manifest v2")?;
        let version = decoder.get_u16("epoch manifest version")?;
        if version != DISTANN_EPOCH_MANIFEST_VERSION {
            return Err(format!(
                "EC_EPOCH_MANIFEST: unsupported manifest version {version}"
            ));
        }
        let epoch = decoder.get_u64("manifest epoch")?;
        let build_id = decoder.get_fixed("manifest build id")?;
        let parent_fingerprint = decoder.get_owned_bytes("manifest parent fingerprint")?;
        let source_snapshot_digest = decoder.get_fixed("manifest source snapshot digest")?;
        let build_spec_digest = decoder.get_fixed("manifest build spec digest")?;
        let generation_descriptor_digest =
            decoder.get_fixed("manifest generation descriptor digest")?;
        let placement_hash_version = decoder.get_u16("manifest placement hash version")?;
        let roster = decode_roster(&mut decoder)?;
        let index_format_version = decoder.get_u16("manifest index format version")?;
        let graph_record_version = decoder.get_u16("manifest graph record version")?;
        let handoff_wire_version = decoder.get_u16("manifest handoff wire version")?;
        let codec_parameters = DistannManifestCodecParameters::decode(
            decoder.get_len_prefixed("manifest codec parameters")?,
        )?;
        let build_options = DistannManifestBuildOptions::decode(
            decoder.get_len_prefixed("manifest build options")?,
        )?;
        let row_schema_fingerprint = decoder.get_fixed("manifest row schema fingerprint")?;
        let head_sample_digest = decoder.get_fixed("manifest head sample digest")?;
        let global_record_count = decoder.get_u64("manifest global record count")?;
        let global_graph_digest = decoder.get_fixed("manifest global graph digest")?;
        let global_row_tier_digest = decoder.get_fixed("manifest global row-tier digest")?;
        let receipt_count = decoder.get_u32("manifest receipt count")? as usize;
        if receipt_count != roster.len()
            || receipt_count
                .checked_mul(DIGEST_BYTES + 4)
                .map_or(true, |minimum| minimum > decoder.remaining())
        {
            return Err("EC_EPOCH_MANIFEST: invalid participant receipt count".to_owned());
        }
        let mut participant_receipts = Vec::with_capacity(receipt_count);
        for _ in 0..receipt_count {
            participant_receipts.push(DistannReadyReceipt::decode(
                decoder.get_len_prefixed("participant Ready receipt")?,
            )?);
        }
        decoder.finish("epoch manifest v2")?;
        let manifest = Self {
            epoch,
            build_id,
            parent_fingerprint,
            source_snapshot_digest,
            build_spec_digest,
            generation_descriptor_digest,
            placement_hash_version,
            roster,
            index_format_version,
            graph_record_version,
            handoff_wire_version,
            codec_parameters,
            build_options,
            row_schema_fingerprint,
            head_sample_digest,
            global_record_count,
            global_graph_digest,
            global_row_tier_digest,
            participant_receipts,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn digest(&self) -> Result<[u8; DIGEST_BYTES], String> {
        Ok(domain_digest(EPOCH_MANIFEST_DOMAIN, &self.encode()?))
    }

    pub fn fingerprint(&self) -> Result<DistannEpochFingerprint, String> {
        Ok(DistannEpochFingerprint::from_manifest_digest(
            self.digest()?,
        ))
    }
}

#[cfg(test)]
fn sample_receipt(node_id: u32, count: u64) -> DistannReadyReceipt {
    DistannReadyReceipt {
        node_id,
        epoch: 7,
        build_id: super::canonical_wire::sample_rfc4122_v4_uuid(0xAB),
        build_spec_digest: [0x11; DIGEST_BYTES],
        generation_descriptor_digest: [0x22; DIGEST_BYTES],
        last_acknowledged_batch_sequence: node_id as u64,
        owned_record_count: count,
        row_count: count,
        owner_stream_digest: [node_id as u8; DIGEST_BYTES],
        persisted_graph_digest: [0x33; DIGEST_BYTES],
        persisted_row_tier_digest: [0x44; DIGEST_BYTES],
        local_directory_digest: [0x55; DIGEST_BYTES],
        graph_bytes: count * 100,
        row_tier_bytes: count * 200,
        directory_bytes: count * 10,
        state: DISTANN_READY_RECEIPT_STATE,
    }
}

#[cfg(test)]
pub(crate) fn sample_manifest_v2() -> DistannEpochManifestV2 {
    DistannEpochManifestV2 {
        epoch: 7,
        build_id: super::canonical_wire::sample_rfc4122_v4_uuid(0xAB),
        parent_fingerprint: Vec::new(),
        source_snapshot_digest: [0x01; DIGEST_BYTES],
        build_spec_digest: [0x11; DIGEST_BYTES],
        generation_descriptor_digest: [0x22; DIGEST_BYTES],
        placement_hash_version: DISTANN_PLACEMENT_HASH_VERSION,
        roster: super::generation_descriptor::sample_roster(),
        index_format_version: DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
        graph_record_version: DISTANN_GRAPH_RECORD_VERSION,
        handoff_wire_version: DISTANN_HANDOFF_WIRE_VERSION,
        codec_parameters: DistannManifestCodecParameters {
            codec_kind: DISTANN_NEIGHBOR_CODEC_RABITQ,
            dimensions: 8,
            code_stride: 13,
            seed: 42,
            transform_dim: 0,
            group_count: 0,
            group_size: 0,
            centroids_per_group: 0,
        },
        build_options: DistannManifestBuildOptions {
            graph_degree: 4,
            options: DistannBuildOptions {
                build_list_size: 100,
                alpha: 1.2,
                seed: 42,
                closure_epsilon: 0.3,
                head_index_cap: 4096,
                build_shards: 0,
                head_policy:
                    crate::am::ec_distann::generation_descriptor::DistannHeadPolicy::CurrentSampleGraph,
                training_query_count: 0,
                training_query_digest: [0; 32],
                head_sizing: None,
            },
        },
        row_schema_fingerprint: [0x66; DIGEST_BYTES],
        head_sample_digest: [0x77; DIGEST_BYTES],
        global_record_count: 10,
        global_graph_digest: [0x88; DIGEST_BYTES],
        global_row_tier_digest: [0x99; DIGEST_BYTES],
        participant_receipts: vec![sample_receipt(10, 6), sample_receipt(20, 4)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_snapshot() -> DistannSourceSnapshot {
        DistannSourceSnapshot {
            system_identifier: 0x0102_0304_0506_0708,
            database_name: "ecaz".to_owned(),
            xmin_full: 100,
            xmax_full: 200,
            curcid: 3,
            xip: vec![101, 103, 107],
            subxip: vec![109, 113],
            suboverflowed: false,
            taken_during_recovery: true,
        }
    }

    #[test]
    fn source_snapshot_round_trip_and_digest_are_canonical() {
        let snapshot = sample_snapshot();
        let encoded = snapshot.encode().unwrap();
        assert_eq!(DistannSourceSnapshot::decode(&encoded).unwrap(), snapshot);
        assert_eq!(snapshot.digest().unwrap(), snapshot.digest().unwrap());

        let mut unsorted = snapshot;
        unsorted.xip.swap(0, 1);
        assert!(unsorted.encode().is_err());

        let mut inverted = sample_snapshot();
        inverted.xmin_full = 201;
        assert!(inverted.encode().is_err());

        let mut out_of_range = sample_snapshot();
        out_of_range.subxip[0] = out_of_range.xmax_full;
        assert!(out_of_range.encode().is_err());
    }

    #[test]
    fn source_snapshot_digest_binds_full_xid_epoch_bits() {
        let snapshot = sample_snapshot();
        let mut next_xid_epoch = snapshot.clone();
        next_xid_epoch.xmin_full += 1_u64 << 32;
        next_xid_epoch.xmax_full += 1_u64 << 32;
        for xid in &mut next_xid_epoch.xip {
            *xid += 1_u64 << 32;
        }
        for xid in &mut next_xid_epoch.subxip {
            *xid += 1_u64 << 32;
        }
        assert_ne!(
            snapshot.digest().unwrap(),
            next_xid_epoch.digest().unwrap(),
            "identical 32-bit XIDs in another wrap epoch must not alias"
        );
    }

    #[test]
    fn ready_receipt_round_trip_verifies_digest_state_and_counts() {
        let receipt = sample_receipt(10, 6);
        let encoded = receipt.encode().unwrap();
        assert_eq!(DistannReadyReceipt::decode(&encoded).unwrap(), receipt);
        assert_eq!(receipt.digest().unwrap(), receipt.digest().unwrap());

        let mut corrupted = encoded;
        corrupted[20] ^= 1;
        assert!(DistannReadyReceipt::decode(&corrupted).is_err());

        let mut bad = receipt;
        bad.row_count += 1;
        assert!(bad.encode().is_err());

        let mut invalid_uuid = sample_receipt(10, 6);
        invalid_uuid.build_id = [0xAB; 16];
        assert!(invalid_uuid.encode().is_err());

        let mut invalid_node = sample_receipt(10, 6);
        invalid_node.node_id = 0;
        assert!(invalid_node.encode().is_err());
    }

    #[test]
    fn manifest_round_trip_binds_roster_receipts_and_v2_fingerprint() {
        let manifest = sample_manifest_v2();
        let encoded = manifest.encode().unwrap();
        assert_eq!(DistannEpochManifestV2::decode(&encoded).unwrap(), manifest);
        let fingerprint = manifest.fingerprint().unwrap();
        assert_eq!(
            fingerprint.as_bytes().len(),
            DISTANN_EPOCH_FINGERPRINT_BYTES
        );
        assert_eq!(fingerprint.manifest_digest(), manifest.digest().unwrap());
        assert_eq!(
            DistannEpochFingerprint::decode(fingerprint.as_bytes()).unwrap(),
            fingerprint
        );

        let mut invalid_uuid = manifest;
        invalid_uuid.build_id = [0xAB; 16];
        assert!(invalid_uuid.encode().is_err());
    }

    #[test]
    fn manifest_build_options_preserve_legacy_bytes_and_round_trip_trained_policy() {
        let legacy = sample_manifest_v2().build_options;
        let legacy_bytes = legacy.encode().unwrap();
        assert_eq!(legacy_bytes.len(), 30);
        assert_eq!(
            u16::from_le_bytes(legacy_bytes[0..2].try_into().unwrap()),
            MANIFEST_BUILD_OPTIONS_V1_VERSION
        );
        assert_eq!(
            DistannManifestBuildOptions::decode(&legacy_bytes).unwrap(),
            legacy
        );

        let mut trained = legacy;
        trained.options.head_policy =
            super::super::generation_descriptor::DistannHeadPolicy::TrainingLandmarksExact;
        trained.options.training_query_count =
            super::super::generation_descriptor::DISTANN_TRAINING_QUERY_COUNT;
        trained.options.training_query_digest = [0x7a; 32];
        let trained_bytes = trained.encode().unwrap();
        assert_eq!(trained_bytes.len(), DISTANN_MANIFEST_BUILD_OPTIONS_BYTES);
        assert_eq!(
            u16::from_le_bytes(trained_bytes[0..2].try_into().unwrap()),
            DISTANN_MANIFEST_BUILD_OPTIONS_VERSION
        );
        assert_eq!(
            DistannManifestBuildOptions::decode(&trained_bytes).unwrap(),
            trained
        );
    }

    #[test]
    fn manifest_rejects_unknown_versions_receipt_reorder_and_bad_parent() {
        let mut encoded = sample_manifest_v2().encode().unwrap();
        encoded[0..2].copy_from_slice(&99_u16.to_le_bytes());
        assert!(DistannEpochManifestV2::decode(&encoded).is_err());

        let mut reordered = sample_manifest_v2();
        reordered.participant_receipts.swap(0, 1);
        assert!(reordered.encode().is_err());

        let mut bad_parent = sample_manifest_v2();
        bad_parent.parent_fingerprint = vec![0; 34];
        assert!(bad_parent.encode().is_err());
    }

    #[test]
    fn codec_and_build_parameter_subrecords_reject_unknown_versions() {
        let manifest = sample_manifest_v2();
        let mut codec = manifest.codec_parameters.encode().unwrap();
        codec[0..2].copy_from_slice(&2_u16.to_le_bytes());
        assert!(DistannManifestCodecParameters::decode(&codec).is_err());

        let mut build = manifest.build_options.encode().unwrap();
        build[0..2].copy_from_slice(&2_u16.to_le_bytes());
        assert!(DistannManifestBuildOptions::decode(&build).is_err());

        let mut bad_codec = manifest.codec_parameters;
        bad_codec.code_stride -= 1;
        assert!(bad_codec.encode().is_err());

        let mut bad_build = manifest.build_options;
        bad_build.graph_degree = 3;
        assert!(bad_build.encode().is_err());

        let descriptor = super::super::generation_descriptor::sample_generation_descriptor();
        manifest
            .codec_parameters
            .validate_artifact(&descriptor.codec_artifact)
            .unwrap();
        let mut wrong_seed = manifest.codec_parameters;
        wrong_seed.seed += 1;
        assert!(wrong_seed
            .validate_artifact(&descriptor.codec_artifact)
            .is_err());
    }

    /// Regeneration helper for the NFR-016 golden fixtures. Run explicitly:
    /// `cargo test --lib --no-default-features --features pg18 emit_distann_golden_fixture_hex -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn emit_distann_golden_fixture_hex() {
        use crate::am::common::training::GroupedPq4Model;
        use crate::am::ec_distann::generation_descriptor::{
            sample_generation_descriptor, DistannBuildSpec, DistannCodecArtifact,
            DistannOwnerExpectation,
        };
        use crate::am::ec_distann::handoff_wire::{
            sample_handoff_entry, DistannHandoffBatch, DistannHandoffShape,
        };
        use crate::am::ec_distann::page::{DistannMetadataPage, DISTANN_NEIGHBOR_CODEC_RABITQ};
        use crate::am::ec_distann::row_schema::sample_row_schema;
        use crate::am::ec_distann::tuple::DistannNodeTuple;
        use crate::storage::page::ItemPointer;

        fn emit(name: &str, bytes: &[u8]) {
            println!("FIXTURE {name} {}", hex::encode(bytes));
        }

        let mut legacy = DistannMetadataPage::empty(
            32,
            100,
            1.2,
            128,
            0x0102_0304_0506_0708,
            DISTANN_NEIGHBOR_CODEC_RABITQ,
            4096,
            0.3,
        );
        legacy.node_count = 42;
        legacy.content_digest = 0xA1A2_A3A4_A5A6_A7A8;
        emit("distann_metadata_v4.hex", &legacy.encode());

        let mut logical_index_uuid = [0xA5; 16];
        logical_index_uuid[6] = 0x45;
        logical_index_uuid[8] = 0x85;
        let control = DistannMetadataPage::distributed_control(
            32,
            100,
            1.2,
            0x0102_0304_0506_0708,
            DISTANN_NEIGHBOR_CODEC_RABITQ,
            4096,
            0.3,
            logical_index_uuid,
        );
        emit("distann_control_metadata_v5.hex", &control.encode());

        let graph = DistannNodeTuple {
            tombstoned: false,
            vec_id: 0x1122_3344_5566_7788,
            heap_tid: ItemPointer {
                block_number: 9,
                offset_number: 3,
            },
            neighbor_count: 2,
            search_code: vec![0xA1, 0xA2],
            neighbor_vec_ids: vec![101, 202, 0, 0],
            neighbor_codes: vec![1, 2, 3, 4, 0, 0, 0, 0],
        };
        emit(
            "distann_graph_record_v1.hex",
            &graph.encode_physical_v1(4, 2).unwrap(),
        );

        let row_schema = sample_row_schema();
        emit("distann_row_schema_v1.hex", &row_schema.encode().unwrap());

        let codec = DistannCodecArtifact::GroupedPq4 {
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
        emit("distann_codec_artifact_v1.hex", &codec.encode().unwrap());

        let descriptor = sample_generation_descriptor();
        emit(
            "distann_generation_descriptor_v2.hex",
            &descriptor.encode().unwrap(),
        );
        let build_spec = DistannBuildSpec {
            epoch: 7,
            build_id: crate::am::ec_distann::canonical_wire::sample_rfc4122_v4_uuid(0xAB),
            parent_fingerprint: Vec::new(),
            source_snapshot_digest: sample_snapshot().digest().unwrap(),
            generation_descriptor_digest: descriptor.digest().unwrap(),
            build_options: DistannBuildOptions {
                build_list_size: 100,
                alpha: 1.2,
                seed: 42,
                closure_epsilon: 0.3,
                head_index_cap: 4096,
                build_shards: 0,
                head_policy:
                    crate::am::ec_distann::generation_descriptor::DistannHeadPolicy::CurrentSampleGraph,
                training_query_count: 0,
                training_query_digest: [0; 32],
                head_sizing: None,
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
        emit("distann_build_spec_v1.hex", &build_spec.encode().unwrap());

        let shape = DistannHandoffShape {
            code_stride: 2,
            graph_degree: 4,
            non_dropped_attribute_count: 3,
        };
        let entry = sample_handoff_entry(7);
        emit(
            "distann_handoff_entry_v1.hex",
            &entry.encode(shape).unwrap(),
        );
        let batch = DistannHandoffBatch {
            epoch: 7,
            build_id: crate::am::ec_distann::canonical_wire::sample_rfc4122_v4_uuid(0xAB),
            batch_seq: 0,
            build_spec_digest: build_spec.digest().unwrap(),
            row_schema_fingerprint: row_schema.fingerprint().unwrap(),
            index_format_version: DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
            neighbor_codec_kind: DISTANN_NEIGHBOR_CODEC_RABITQ,
            entries: vec![entry, sample_handoff_entry(8)],
        };
        emit(
            "distann_handoff_batch_v1.hex",
            &batch.encode(shape).unwrap(),
        );

        emit(
            "distann_source_snapshot_v1.hex",
            &sample_snapshot().encode().unwrap(),
        );
        emit(
            "distann_ready_receipt_v1.hex",
            &sample_receipt(10, 6).encode().unwrap(),
        );
        let manifest = sample_manifest_v2();
        emit(
            "distann_manifest_codec_parameters_v1.hex",
            &manifest.codec_parameters.encode().unwrap(),
        );
        emit(
            "distann_manifest_build_options_v1.hex",
            &manifest.build_options.encode().unwrap(),
        );
        emit("distann_epoch_manifest_v2.hex", &manifest.encode().unwrap());
    }
}
