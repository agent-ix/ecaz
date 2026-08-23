//! FR-076 canonical epoch handoff entry and batch wire formats.

use std::mem::size_of;
use std::ops::Range;

use sha2::{
    digest::{
        common::hazmat::{SerializableState, SerializedState},
        typenum::Unsigned,
    },
    Digest, Sha256,
};

use super::canonical_wire::{domain_digest, is_rfc4122_v4_uuid, CanonicalDecoder};
use super::generation_descriptor::{
    DISTANN_HANDOFF_WIRE_VERSION, DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
};
use super::page::{
    DISTANN_NEIGHBOR_CODEC_GROUPED_PQ, DISTANN_NEIGHBOR_CODEC_RABITQ,
    DISTANN_NEIGHBOR_CODEC_TURBOQUANT,
};

pub const DISTANN_HANDOFF_MAX_BYTES: usize = 8 * 1024 * 1024;
pub const DISTANN_HANDOFF_VERSION_OFFSET: usize = 0;
pub const DISTANN_HANDOFF_ENTRY_FIXED_PREFIX_BYTES: usize = 10;
pub const DISTANN_HANDOFF_BATCH_FIXED_PREFIX_BYTES: usize = 109;
const DISTANN_HANDOFF_BATCH_SEQUENCE_OFFSET: usize = 26;
const DISTANN_HANDOFF_BATCH_ENTRY_COUNT_OFFSET: usize = 101;
const DISTANN_HANDOFF_BATCH_ENTRY_BYTES_OFFSET: usize = 105;
const HANDOFF_ENTRY_DOMAIN: &[u8] = b"ec_distann_handoff_entry_v1\0";
const HANDOFF_BATCH_DOMAIN: &[u8] = b"ec_distann_handoff_batch_v1\0";
const OWNER_STREAM_DOMAIN: &[u8] = b"ec_distann_owner_stream_v1\0";
const HANDOFF_DIGEST_BYTES: usize = 32;
const OWNER_STREAM_HASH_STATE_VERSION: u16 = 1;
const OWNER_STREAM_HASH_IMPLEMENTATION_SHA2_0_11: u8 = 1;
const SHA256_SERIALIZED_STATE_BYTES: usize =
    <<Sha256 as SerializableState>::SerializedStateSize as Unsigned>::USIZE;
pub const DISTANN_OWNER_STREAM_HASH_STATE_BYTES: usize = 2 + 1 + SHA256_SERIALIZED_STATE_BYTES;
pub const DISTANN_OWNER_STREAM_HASH_STATE_VERSION_OFFSET: usize = 0;
pub const DISTANN_OWNER_STREAM_HASH_STATE_IMPLEMENTATION_OFFSET: usize = 2;
pub const DISTANN_OWNER_STREAM_HASH_STATE_CHAIN_OFFSET: usize = 3;
pub const DISTANN_OWNER_STREAM_HASH_STATE_BLOCK_COUNT_OFFSET: usize = 35;
pub const DISTANN_OWNER_STREAM_HASH_STATE_BUFFER_LENGTH_OFFSET: usize = 43;
pub const DISTANN_OWNER_STREAM_HASH_STATE_BUFFER_OFFSET: usize = 44;

const _: [(); 104] = [(); SHA256_SERIALIZED_STATE_BYTES];
const _: [(); 107] = [(); DISTANN_OWNER_STREAM_HASH_STATE_BYTES];

fn append_len_prefixed(output: &mut Vec<u8>, value: &[u8]) -> Result<(), String> {
    let length = u32::try_from(value.len())
        .map_err(|_| "EC_HANDOFF_TOO_LARGE: canonical field exceeds u32".to_owned())?;
    output.extend_from_slice(&length.to_le_bytes());
    output.extend_from_slice(value);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistannHandoffShape {
    pub code_stride: usize,
    pub graph_degree: usize,
    pub non_dropped_attribute_count: usize,
}

impl DistannHandoffShape {
    pub(crate) fn new(
        code_stride: usize,
        graph_degree: usize,
        non_dropped_attribute_count: usize,
    ) -> Result<Self, String> {
        let shape = Self {
            code_stride,
            graph_degree,
            non_dropped_attribute_count,
        };
        shape.validate()?;
        Ok(shape)
    }

    pub(crate) fn from_descriptor(
        descriptor: &super::generation_descriptor::DistannGenerationDescriptor,
    ) -> Result<Self, String> {
        let binding =
            super::quantizer::DistannCodecBinding::from_artifact(&descriptor.codec_artifact)?;
        Self::new(
            binding.code_len(usize::from(descriptor.dimensions))?,
            usize::from(descriptor.graph_degree),
            descriptor.row_schema.non_dropped_count(),
        )
    }

    fn validate(self) -> Result<(), String> {
        if self.code_stride == 0 || self.graph_degree == 0 || self.graph_degree > u16::MAX as usize
        {
            return Err("EC_HANDOFF_FORMAT: invalid handoff code/degree shape".to_owned());
        }
        if self.non_dropped_attribute_count > u16::MAX as usize {
            return Err("EC_HANDOFF_FORMAT: handoff row attribute count exceeds u16".to_owned());
        }
        Ok(())
    }

    fn null_bitmap_bytes(self) -> usize {
        self.non_dropped_attribute_count.div_ceil(8)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistannHandoffEntry {
    pub vec_id: u64,
    pub source_identity: Vec<u8>,
    pub graph_flags: u16,
    pub search_code: Vec<u8>,
    pub neighbor_vec_ids: Vec<u64>,
    pub neighbor_codes: Vec<u8>,
    /// Bit=1 means NULL; the first non-dropped attnum is bit 0.
    pub row_null_bitmap: Vec<u8>,
    /// One PostgreSQL binary `typsend` value per non-NULL attribute, in
    /// ascending non-dropped attnum order.
    pub row_values: Vec<Vec<u8>>,
}

impl DistannHandoffEntry {
    pub fn validate(&self, shape: DistannHandoffShape) -> Result<(), String> {
        shape.validate()?;
        if self.source_identity.len() != 16 {
            return Err(format!(
                "EC_SOURCE_IDENTITY: handoff identity is {} bytes, expected 16",
                self.source_identity.len()
            ));
        }
        if self.graph_flags != 0 {
            return Err(format!(
                "EC_HANDOFF_FORMAT: build handoff graph flags must be zero, got {}",
                self.graph_flags
            ));
        }
        if self.search_code.len() != shape.code_stride {
            return Err(format!(
                "EC_HANDOFF_FORMAT: search code is {} bytes, expected {}",
                self.search_code.len(),
                shape.code_stride
            ));
        }
        if self.neighbor_vec_ids.len() > shape.graph_degree {
            return Err(format!(
                "EC_HANDOFF_FORMAT: {} neighbors exceed graph degree {}",
                self.neighbor_vec_ids.len(),
                shape.graph_degree
            ));
        }
        let expected_neighbor_codes = self
            .neighbor_vec_ids
            .len()
            .checked_mul(shape.code_stride)
            .ok_or_else(|| "EC_HANDOFF_FORMAT: neighbor code length overflow".to_owned())?;
        if self.neighbor_codes.len() != expected_neighbor_codes {
            return Err(format!(
                "EC_HANDOFF_FORMAT: neighbor codes are {} bytes, expected {expected_neighbor_codes}",
                self.neighbor_codes.len()
            ));
        }
        let expected_values = super::canonical_wire::validate_null_bitmap(
            &self.row_null_bitmap,
            shape.non_dropped_attribute_count,
            "EC_HANDOFF_FORMAT",
            "NULL bitmap",
        )?;
        if self.row_values.len() != expected_values {
            return Err(format!(
                "EC_HANDOFF_FORMAT: row has {} binary values, expected {expected_values}",
                self.row_values.len()
            ));
        }
        Ok(())
    }

    pub(crate) fn encoded_len(&self, shape: DistannHandoffShape) -> Result<usize, String> {
        self.validate(shape)?;
        u32::try_from(self.neighbor_vec_ids.len())
            .map_err(|_| "EC_HANDOFF_FORMAT: neighbor count exceeds u32".to_owned())?;
        u32::try_from(self.row_values.len())
            .map_err(|_| "EC_HANDOFF_FORMAT: row value count exceeds u32".to_owned())?;
        let mut length = 2_usize + 8 + 4 + self.source_identity.len() + 2;
        for bytes in [
            self.search_code.len(),
            self.neighbor_codes.len(),
            self.row_null_bitmap.len(),
        ] {
            u32::try_from(bytes)
                .map_err(|_| "EC_HANDOFF_TOO_LARGE: handoff entry field exceeds u32".to_owned())?;
            length = length
                .checked_add(4 + bytes)
                .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: handoff entry length overflow".to_owned())?;
        }
        length = length
            .checked_add(4 + self.neighbor_vec_ids.len() * size_of::<u64>())
            .and_then(|value| value.checked_add(4))
            .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: handoff entry length overflow".to_owned())?;
        for value in &self.row_values {
            u32::try_from(value.len())
                .map_err(|_| "EC_HANDOFF_TOO_LARGE: row value exceeds u32".to_owned())?;
            length = length
                .checked_add(4 + value.len())
                .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: handoff entry length overflow".to_owned())?;
        }
        if length > DISTANN_HANDOFF_MAX_BYTES {
            return Err("EC_HANDOFF_TOO_LARGE: one handoff entry exceeds 8 MiB".to_owned());
        }
        Ok(length)
    }

    pub(crate) fn encode_into(
        &self,
        shape: DistannHandoffShape,
        output: &mut Vec<u8>,
    ) -> Result<Range<usize>, String> {
        let encoded_len = self.encoded_len(shape)?;
        let start = output.len();
        output
            .try_reserve_exact(encoded_len)
            .map_err(|_| "EC_HANDOFF_TOO_LARGE: could not reserve handoff entry".to_owned())?;
        let result = (|| {
            output.extend_from_slice(&DISTANN_HANDOFF_WIRE_VERSION.to_le_bytes());
            output.extend_from_slice(&self.vec_id.to_le_bytes());
            append_len_prefixed(output, &self.source_identity)?;
            output.extend_from_slice(&self.graph_flags.to_le_bytes());
            append_len_prefixed(output, &self.search_code)?;
            output.extend_from_slice(
                &u32::try_from(self.neighbor_vec_ids.len())
                    .map_err(|_| "EC_HANDOFF_FORMAT: neighbor count exceeds u32".to_owned())?
                    .to_le_bytes(),
            );
            for neighbor in &self.neighbor_vec_ids {
                output.extend_from_slice(&neighbor.to_le_bytes());
            }
            append_len_prefixed(output, &self.neighbor_codes)?;
            append_len_prefixed(output, &self.row_null_bitmap)?;
            output.extend_from_slice(
                &u32::try_from(self.row_values.len())
                    .map_err(|_| "EC_HANDOFF_FORMAT: row value count exceeds u32".to_owned())?
                    .to_le_bytes(),
            );
            for value in &self.row_values {
                append_len_prefixed(output, value)?;
            }
            Ok::<(), String>(())
        })();
        if let Err(error) = result {
            output.truncate(start);
            return Err(error);
        }
        debug_assert_eq!(output.len() - start, encoded_len);
        Ok(start..output.len())
    }

    pub fn encode(&self, shape: DistannHandoffShape) -> Result<Vec<u8>, String> {
        let encoded_len = self.encoded_len(shape)?;
        let mut encoded = Vec::with_capacity(encoded_len);
        self.encode_into(shape, &mut encoded)?;
        debug_assert_eq!(encoded.len(), encoded_len);
        Ok(encoded)
    }

    pub fn decode(input: &[u8], shape: DistannHandoffShape) -> Result<Self, String> {
        shape.validate()?;
        if input.len() > DISTANN_HANDOFF_MAX_BYTES {
            return Err("EC_HANDOFF_TOO_LARGE: one handoff entry exceeds 8 MiB".to_owned());
        }
        let mut decoder = CanonicalDecoder::new(input, "handoff entry")?;
        let version = decoder.get_u16("handoff entry version")?;
        if version != DISTANN_HANDOFF_WIRE_VERSION {
            return Err(format!(
                "EC_HANDOFF_FORMAT: unsupported entry version {version}"
            ));
        }
        let vec_id = decoder.get_u64("vec id")?;
        let source_identity = decoder.get_owned_bytes("source identity")?;
        let graph_flags = decoder.get_u16("graph flags")?;
        let search_code = decoder.get_owned_bytes("search code")?;
        let neighbor_count = decoder.get_u32("neighbor count")? as usize;
        if neighbor_count > shape.graph_degree
            || neighbor_count
                .checked_mul(size_of::<u64>())
                .map_or(true, |bytes| bytes > decoder.remaining())
        {
            return Err("EC_HANDOFF_FORMAT: invalid declared neighbor count".to_owned());
        }
        let mut neighbor_vec_ids = Vec::with_capacity(neighbor_count);
        for _ in 0..neighbor_count {
            neighbor_vec_ids.push(decoder.get_u64("neighbor vec id")?);
        }
        let neighbor_codes = decoder.get_owned_bytes("neighbor codes")?;
        let row_null_bitmap = decoder.get_owned_bytes("row NULL bitmap")?;
        let row_value_count = decoder.get_u32("row value count")? as usize;
        if row_value_count > shape.non_dropped_attribute_count {
            return Err("EC_HANDOFF_FORMAT: invalid declared row value count".to_owned());
        }
        let mut row_values = Vec::with_capacity(row_value_count);
        for _ in 0..row_value_count {
            row_values.push(decoder.get_owned_bytes("row value")?);
        }
        decoder.finish("handoff entry")?;
        let entry = Self {
            vec_id,
            source_identity,
            graph_flags,
            search_code,
            neighbor_vec_ids,
            neighbor_codes,
            row_null_bitmap,
            row_values,
        };
        entry.validate(shape)?;
        Ok(entry)
    }

    pub fn digest(&self, shape: DistannHandoffShape) -> Result<[u8; 32], String> {
        Ok(domain_digest(HANDOFF_ENTRY_DOMAIN, &self.encode(shape)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistannHandoffBatch {
    pub epoch: u64,
    pub build_id: [u8; 16],
    pub batch_seq: u64,
    pub build_spec_digest: [u8; 32],
    pub row_schema_fingerprint: [u8; 32],
    pub index_format_version: u16,
    pub neighbor_codec_kind: u8,
    pub entries: Vec<DistannHandoffEntry>,
}

impl DistannHandoffBatch {
    /// Verify the shape-independent outer batch identity before descriptor
    /// lookup, decoding, allocation, or generation storage access. The
    /// returned digest is both the canonical trailer and the journal identity
    /// used for exact replay.
    pub(crate) fn verified_digest(input: &[u8]) -> Result<[u8; 32], String> {
        let minimum = DISTANN_HANDOFF_BATCH_FIXED_PREFIX_BYTES
            .checked_add(HANDOFF_DIGEST_BYTES)
            .expect("handoff minimum length fits usize");
        if input.len() < minimum {
            return Err(format!(
                "EC_HANDOFF_FORMAT: handoff batch is {} bytes, minimum is {minimum}",
                input.len()
            ));
        }
        if input.len() > DISTANN_HANDOFF_MAX_BYTES {
            return Err("EC_HANDOFF_TOO_LARGE: batch exceeds 8 MiB".to_owned());
        }
        let canonical_len = input.len() - HANDOFF_DIGEST_BYTES;
        let (canonical, supplied_digest) = input.split_at(canonical_len);
        let expected_digest = domain_digest(HANDOFF_BATCH_DOMAIN, canonical);
        if supplied_digest != expected_digest {
            return Err("EC_HANDOFF_DIGEST: batch digest mismatch".to_owned());
        }
        Ok(expected_digest)
    }

    fn validate(&self, shape: DistannHandoffShape) -> Result<(), String> {
        shape.validate()?;
        if self.epoch == 0 || !is_rfc4122_v4_uuid(&self.build_id) {
            return Err(
                "EC_HANDOFF_FORMAT: epoch must be non-zero and build id must be RFC 4122 v4"
                    .to_owned(),
            );
        }
        if self.index_format_version != DISTANN_PHYSICAL_INDEX_FORMAT_VERSION
            || !matches!(
                self.neighbor_codec_kind,
                DISTANN_NEIGHBOR_CODEC_GROUPED_PQ
                    | DISTANN_NEIGHBOR_CODEC_RABITQ
                    | DISTANN_NEIGHBOR_CODEC_TURBOQUANT
            )
        {
            return Err("EC_HANDOFF_FORMAT: unsupported index/codec version".to_owned());
        }
        let mut previous = None;
        for entry in &self.entries {
            entry.validate(shape)?;
            if previous == Some(entry.vec_id) {
                return Err("EC_DUPLICATE_VEC_ID: owner batch repeats a vec_id".to_owned());
            }
            if previous.is_some_and(|value| entry.vec_id < value) {
                return Err(
                    "EC_HANDOFF_FORMAT: owner batch vec_ids are not strictly increasing".to_owned(),
                );
            }
            previous = Some(entry.vec_id);
        }
        Ok(())
    }

    fn canonical_without_digest(&self, shape: DistannHandoffShape) -> Result<Vec<u8>, String> {
        self.validate(shape)?;
        let encoded_entries_bytes = self.entries.iter().try_fold(0_usize, |total, entry| {
            total
                .checked_add(4 + entry.encoded_len(shape)?)
                .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: batch length overflow".to_owned())
        })?;
        let canonical_len = DISTANN_HANDOFF_BATCH_FIXED_PREFIX_BYTES
            .checked_add(encoded_entries_bytes)
            .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: batch length overflow".to_owned())?;
        let total_len = canonical_len
            .checked_add(HANDOFF_DIGEST_BYTES)
            .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: batch length overflow".to_owned())?;
        if total_len > DISTANN_HANDOFF_MAX_BYTES {
            return Err(format!(
                "EC_HANDOFF_TOO_LARGE: batch is {total_len} bytes, maximum is {DISTANN_HANDOFF_MAX_BYTES}"
            ));
        }
        let mut output = Vec::with_capacity(total_len);
        output.extend_from_slice(&DISTANN_HANDOFF_WIRE_VERSION.to_le_bytes());
        output.extend_from_slice(&self.epoch.to_le_bytes());
        output.extend_from_slice(&self.build_id);
        output.extend_from_slice(&self.batch_seq.to_le_bytes());
        output.extend_from_slice(&self.build_spec_digest);
        output.extend_from_slice(&self.row_schema_fingerprint);
        output.extend_from_slice(&self.index_format_version.to_le_bytes());
        output.push(self.neighbor_codec_kind);
        output.extend_from_slice(
            &u32::try_from(self.entries.len())
                .map_err(|_| "EC_HANDOFF_TOO_LARGE: entry count exceeds u32".to_owned())?
                .to_le_bytes(),
        );
        output.extend_from_slice(
            &u32::try_from(encoded_entries_bytes)
                .map_err(|_| "EC_HANDOFF_TOO_LARGE: encoded entries exceed u32".to_owned())?
                .to_le_bytes(),
        );
        for entry in &self.entries {
            let encoded_len = entry.encoded_len(shape)?;
            output.extend_from_slice(&(encoded_len as u32).to_le_bytes());
            entry.encode_into(shape, &mut output)?;
        }
        debug_assert_eq!(output.len(), canonical_len);
        Ok(output)
    }

    pub fn encode(&self, shape: DistannHandoffShape) -> Result<Vec<u8>, String> {
        let canonical = self.canonical_without_digest(shape)?;
        let digest = domain_digest(HANDOFF_BATCH_DOMAIN, &canonical);
        let total = canonical
            .len()
            .checked_add(HANDOFF_DIGEST_BYTES)
            .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: batch length overflow".to_owned())?;
        if total > DISTANN_HANDOFF_MAX_BYTES {
            return Err(format!(
                "EC_HANDOFF_TOO_LARGE: batch is {total} bytes, maximum is {DISTANN_HANDOFF_MAX_BYTES}"
            ));
        }
        let mut output = canonical;
        output.extend_from_slice(&digest);
        Ok(output)
    }

    pub fn decode(input: &[u8], shape: DistannHandoffShape) -> Result<Self, String> {
        shape.validate()?;
        let expected_digest = Self::verified_digest(input)?;
        let canonical_len = input.len() - HANDOFF_DIGEST_BYTES;
        let canonical = &input[..canonical_len];
        debug_assert_eq!(&input[canonical_len..], expected_digest);

        let mut decoder = CanonicalDecoder::new(canonical, "handoff batch")?;
        let version = decoder.get_u16("handoff batch version")?;
        if version != DISTANN_HANDOFF_WIRE_VERSION {
            return Err(format!(
                "EC_HANDOFF_FORMAT: unsupported batch version {version}"
            ));
        }
        let epoch = decoder.get_u64("epoch")?;
        let build_id = decoder.get_fixed("build id")?;
        let batch_seq = decoder.get_u64("batch sequence")?;
        let build_spec_digest = decoder.get_fixed("build spec digest")?;
        let row_schema_fingerprint = decoder.get_fixed("row schema fingerprint")?;
        let index_format_version = decoder.get_u16("index format version")?;
        let neighbor_codec_kind = decoder.get_u8("neighbor codec kind")?;
        let entry_count = decoder.get_u32("entry count")? as usize;
        let encoded_entries_bytes = decoder.get_u32("encoded entries bytes")? as usize;
        if encoded_entries_bytes > decoder.remaining() || entry_count > encoded_entries_bytes / 4 {
            return Err("EC_HANDOFF_FORMAT: invalid declared entry section size".to_owned());
        }
        let entry_section = decoder.get_bytes(encoded_entries_bytes, "encoded entries")?;
        decoder.finish("handoff batch")?;

        let mut entry_decoder = CanonicalDecoder::new(entry_section, "handoff entry section")?;
        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            let entry = entry_decoder.get_len_prefixed("handoff entry")?;
            entries.push(DistannHandoffEntry::decode(entry, shape)?);
        }
        entry_decoder.finish("handoff entry section")?;
        let batch = Self {
            epoch,
            build_id,
            batch_seq,
            build_spec_digest,
            row_schema_fingerprint,
            index_format_version,
            neighbor_codec_kind,
            entries,
        };
        batch.validate(shape)?;
        Ok(batch)
    }

    pub fn digest(&self, shape: DistannHandoffShape) -> Result<[u8; 32], String> {
        Ok(domain_digest(
            HANDOFF_BATCH_DOMAIN,
            &self.canonical_without_digest(shape)?,
        ))
    }
}

/// The coordinator's single retained representation of one owner batch.
/// Entries are appended directly in canonical form, then count/length fields
/// are patched and the digest trailer is appended in the same allocation.
pub(crate) struct DistannEncodedHandoffBatch {
    wire: Vec<u8>,
    entry_count: u32,
    entry_section_bytes: u32,
    finalized_digest: Option<[u8; HANDOFF_DIGEST_BYTES]>,
    max_bytes: usize,
}

impl DistannEncodedHandoffBatch {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        epoch: u64,
        build_id: [u8; 16],
        batch_seq: u64,
        build_spec_digest: [u8; 32],
        row_schema_fingerprint: [u8; 32],
        index_format_version: u16,
        neighbor_codec_kind: u8,
        max_bytes: usize,
    ) -> Result<Self, String> {
        if max_bytes < DISTANN_HANDOFF_BATCH_FIXED_PREFIX_BYTES + HANDOFF_DIGEST_BYTES
            || max_bytes > DISTANN_HANDOFF_MAX_BYTES
        {
            return Err("EC_HANDOFF_TOO_LARGE: invalid owner batch bound".to_owned());
        }
        let mut wire = Vec::new();
        wire.try_reserve_exact(DISTANN_HANDOFF_BATCH_FIXED_PREFIX_BYTES)
            .map_err(|_| "EC_HANDOFF_TOO_LARGE: could not reserve batch prefix".to_owned())?;
        wire.extend_from_slice(&DISTANN_HANDOFF_WIRE_VERSION.to_le_bytes());
        wire.extend_from_slice(&epoch.to_le_bytes());
        wire.extend_from_slice(&build_id);
        wire.extend_from_slice(&batch_seq.to_le_bytes());
        wire.extend_from_slice(&build_spec_digest);
        wire.extend_from_slice(&row_schema_fingerprint);
        wire.extend_from_slice(&index_format_version.to_le_bytes());
        wire.push(neighbor_codec_kind);
        wire.extend_from_slice(&0_u32.to_le_bytes());
        wire.extend_from_slice(&0_u32.to_le_bytes());
        debug_assert_eq!(wire.len(), DISTANN_HANDOFF_BATCH_FIXED_PREFIX_BYTES);
        let batch = Self {
            wire,
            entry_count: 0,
            entry_section_bytes: 0,
            finalized_digest: None,
            max_bytes,
        };
        batch.check_capacity()?;
        Ok(batch)
    }

    fn check_capacity(&self) -> Result<(), String> {
        if self.wire.capacity() > self.max_bytes {
            return Err(format!(
                "EC_HANDOFF_TOO_LARGE: retained owner capacity {} exceeds {}",
                self.wire.capacity(),
                self.max_bytes
            ));
        }
        Ok(())
    }

    pub(crate) fn can_append(&self, encoded_entry_bytes: usize) -> Result<bool, String> {
        if self.finalized_digest.is_some() {
            return Ok(false);
        }
        self.wire
            .len()
            .checked_add(4)
            .and_then(|bytes| bytes.checked_add(encoded_entry_bytes))
            .and_then(|bytes| bytes.checked_add(HANDOFF_DIGEST_BYTES))
            .map(|bytes| bytes <= self.max_bytes)
            .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: owner batch length overflow".to_owned())
    }

    pub(crate) fn append_entry(
        &mut self,
        entry: &DistannHandoffEntry,
        shape: DistannHandoffShape,
    ) -> Result<Range<usize>, String> {
        let encoded_entry_bytes = entry.encoded_len(shape)?;
        if !self.can_append(encoded_entry_bytes)? {
            return Err("EC_HANDOFF_TOO_LARGE: entry does not fit owner batch".to_owned());
        }
        let framed_bytes = 4_usize
            .checked_add(encoded_entry_bytes)
            .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: owner entry length overflow".to_owned())?;
        self.wire
            .try_reserve_exact(framed_bytes)
            .map_err(|_| "EC_HANDOFF_TOO_LARGE: could not reserve owner entry".to_owned())?;
        self.check_capacity()?;
        let previous_len = self.wire.len();
        self.wire
            .extend_from_slice(&(encoded_entry_bytes as u32).to_le_bytes());
        let range = match entry.encode_into(shape, &mut self.wire) {
            Ok(range) => range,
            Err(error) => {
                self.wire.truncate(previous_len);
                return Err(error);
            }
        };
        self.entry_count = self
            .entry_count
            .checked_add(1)
            .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: owner entry count exceeds u32".to_owned())?;
        self.entry_section_bytes = self
            .entry_section_bytes
            .checked_add(framed_bytes as u32)
            .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: owner entry section exceeds u32".to_owned())?;
        self.check_capacity()?;
        Ok(range)
    }

    pub(crate) fn finalize(&mut self) -> Result<[u8; HANDOFF_DIGEST_BYTES], String> {
        if let Some(digest) = self.finalized_digest {
            return Ok(digest);
        }
        let expected_len = DISTANN_HANDOFF_BATCH_FIXED_PREFIX_BYTES
            .checked_add(self.entry_section_bytes as usize)
            .and_then(|bytes| bytes.checked_add(HANDOFF_DIGEST_BYTES))
            .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: owner batch length overflow".to_owned())?;
        if expected_len > self.max_bytes {
            return Err("EC_HANDOFF_TOO_LARGE: encoded owner batch exceeds its bound".to_owned());
        }
        self.wire[DISTANN_HANDOFF_BATCH_ENTRY_COUNT_OFFSET
            ..DISTANN_HANDOFF_BATCH_ENTRY_COUNT_OFFSET + 4]
            .copy_from_slice(&self.entry_count.to_le_bytes());
        self.wire[DISTANN_HANDOFF_BATCH_ENTRY_BYTES_OFFSET
            ..DISTANN_HANDOFF_BATCH_ENTRY_BYTES_OFFSET + 4]
            .copy_from_slice(&self.entry_section_bytes.to_le_bytes());
        let digest = domain_digest(HANDOFF_BATCH_DOMAIN, &self.wire);
        self.wire
            .try_reserve_exact(HANDOFF_DIGEST_BYTES)
            .map_err(|_| "EC_HANDOFF_TOO_LARGE: could not reserve batch digest".to_owned())?;
        self.check_capacity()?;
        self.wire.extend_from_slice(&digest);
        debug_assert_eq!(self.wire.len(), expected_len);
        self.finalized_digest = Some(digest);
        Ok(digest)
    }

    pub(crate) fn reset(&mut self, next_batch_seq: u64) {
        self.wire.truncate(DISTANN_HANDOFF_BATCH_FIXED_PREFIX_BYTES);
        self.wire[DISTANN_HANDOFF_BATCH_SEQUENCE_OFFSET..DISTANN_HANDOFF_BATCH_SEQUENCE_OFFSET + 8]
            .copy_from_slice(&next_batch_seq.to_le_bytes());
        self.wire[DISTANN_HANDOFF_BATCH_ENTRY_COUNT_OFFSET
            ..DISTANN_HANDOFF_BATCH_ENTRY_COUNT_OFFSET + 8]
            .fill(0);
        self.entry_count = 0;
        self.entry_section_bytes = 0;
        self.finalized_digest = None;
    }

    pub(crate) fn entry_count(&self) -> u32 {
        self.entry_count
    }

    pub(crate) fn is_finalized(&self) -> bool {
        self.finalized_digest.is_some()
    }

    pub(crate) fn wire(&self) -> &[u8] {
        &self.wire
    }

    pub(crate) fn rollback_unfinalized_append(&mut self, previous_len: usize, previous_count: u32) {
        debug_assert!(self.finalized_digest.is_none());
        debug_assert!(previous_len >= DISTANN_HANDOFF_BATCH_FIXED_PREFIX_BYTES);
        self.wire.truncate(previous_len);
        self.entry_count = previous_count;
        self.entry_section_bytes =
            u32::try_from(previous_len - DISTANN_HANDOFF_BATCH_FIXED_PREFIX_BYTES)
                .expect("bounded owner entry section fits u32");
    }

    pub(crate) fn retained_capacity(&self) -> usize {
        self.wire.capacity()
    }
}

/// Restart-serializable SHA-256 state for one canonical owner stream.
///
/// The persisted representation is deliberately separate from the handoff
/// wire version: two little-endian version bytes and one implementation tag
/// precede the pinned 104-byte `sha2` 0.11 `Sha256` state. The separately
/// persisted cumulative digest must be supplied when restoring so catalog
/// corruption is detected before the state is advanced.
#[derive(Debug, Clone)]
pub(crate) struct DistannOwnerStreamHasher {
    hasher: Sha256,
    message_bytes: u64,
}

impl DistannOwnerStreamHasher {
    pub(crate) fn new() -> Self {
        let mut hasher = Sha256::new();
        hasher.update(OWNER_STREAM_DOMAIN);
        Self {
            hasher,
            message_bytes: OWNER_STREAM_DOMAIN.len() as u64,
        }
    }

    pub(crate) fn digest(&self) -> [u8; 32] {
        self.hasher.clone().finalize().into()
    }

    pub(crate) fn serialize(&self) -> [u8; DISTANN_OWNER_STREAM_HASH_STATE_BYTES] {
        let serialized = <Sha256 as SerializableState>::serialize(&self.hasher);
        let mut output = [0_u8; DISTANN_OWNER_STREAM_HASH_STATE_BYTES];
        output[..2].copy_from_slice(&OWNER_STREAM_HASH_STATE_VERSION.to_le_bytes());
        output[2] = OWNER_STREAM_HASH_IMPLEMENTATION_SHA2_0_11;
        output[3..].copy_from_slice(serialized.as_slice());
        output
    }

    pub(crate) fn restore(
        input: &[u8],
        expected_cumulative_digest: [u8; 32],
    ) -> Result<Self, String> {
        if input.len() != DISTANN_OWNER_STREAM_HASH_STATE_BYTES {
            return Err("EC_HANDOFF_FORMAT: invalid owner-stream SHA-256 state length".to_owned());
        }
        let state_version = u16::from_le_bytes(
            input[..2]
                .try_into()
                .expect("owner-stream state version has two bytes"),
        );
        if state_version != OWNER_STREAM_HASH_STATE_VERSION {
            return Err(format!(
                "EC_HANDOFF_FORMAT: unsupported owner-stream SHA-256 state version {state_version}"
            ));
        }
        if input[2] != OWNER_STREAM_HASH_IMPLEMENTATION_SHA2_0_11 {
            return Err(format!(
                "EC_HANDOFF_FORMAT: unsupported owner-stream SHA-256 implementation {}",
                input[2]
            ));
        }

        let serialized = SerializedState::<Sha256>::try_from(&input[3..]).map_err(|_| {
            "EC_HANDOFF_FORMAT: invalid owner-stream SHA-256 state length".to_owned()
        })?;
        let hasher = <Sha256 as SerializableState>::deserialize(&serialized).map_err(|_| {
            "EC_HANDOFF_FORMAT: invalid owner-stream SHA-256 state encoding".to_owned()
        })?;
        if <Sha256 as SerializableState>::serialize(&hasher).as_slice() != serialized.as_slice() {
            return Err("EC_HANDOFF_FORMAT: non-canonical owner-stream SHA-256 state".to_owned());
        }

        // sha2 0.11's pinned canonical representation stores the compressed
        // block count in raw bytes 32..40 and the eager-buffer position in
        // byte 40. Bound the reconstructed message to SHA-256's 64-bit bit
        // length before finalization or any further update.
        let block_count = u64::from_le_bytes(
            serialized[32..40]
                .try_into()
                .expect("serialized SHA-256 block count has eight bytes"),
        );
        let buffered_bytes = u64::from(serialized[40]);
        let message_bytes = block_count
            .checked_mul(64)
            .and_then(|bytes| bytes.checked_add(buffered_bytes))
            .filter(|bytes| *bytes >= OWNER_STREAM_DOMAIN.len() as u64 && *bytes <= u64::MAX / 8)
            .ok_or_else(|| {
                "EC_HANDOFF_FORMAT: invalid owner-stream SHA-256 message length".to_owned()
            })?;
        let state = Self {
            hasher,
            message_bytes,
        };
        if state.digest() != expected_cumulative_digest {
            return Err(
                "EC_HANDOFF_DIGEST: owner-stream SHA-256 state disagrees with cumulative digest"
                    .to_owned(),
            );
        }
        Ok(state)
    }

    fn ensure_can_update(&self, additional_bytes: usize) -> Result<u64, String> {
        let additional_bytes = u64::try_from(additional_bytes)
            .map_err(|_| "EC_HANDOFF_TOO_LARGE: owner stream length exceeds u64".to_owned())?;
        self.message_bytes
            .checked_add(additional_bytes)
            .filter(|bytes| *bytes <= u64::MAX / 8)
            .ok_or_else(|| {
                "EC_HANDOFF_TOO_LARGE: owner stream exceeds SHA-256 message length".to_owned()
            })
    }

    #[cfg(test)]
    fn update_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        let new_message_bytes = self.ensure_can_update(bytes.len())?;
        self.hasher.update(bytes);
        self.message_bytes = new_message_bytes;
        Ok(())
    }

    /// Add an entry that the caller has already validated as the exact
    /// canonical `DistannHandoffEntry` encoding for this generation's shape.
    pub(crate) fn update_encoded_entry(&mut self, encoded_entry: &[u8]) -> Result<(), String> {
        let encoded_length = u32::try_from(encoded_entry.len())
            .map_err(|_| "EC_HANDOFF_TOO_LARGE: owner entry exceeds u32".to_owned())?;
        let additional_bytes = size_of::<u32>()
            .checked_add(encoded_entry.len())
            .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: owner stream length overflow".to_owned())?;
        let new_message_bytes = self.ensure_can_update(additional_bytes)?;
        self.hasher.update(encoded_length.to_le_bytes());
        self.hasher.update(encoded_entry);
        self.message_bytes = new_message_bytes;
        Ok(())
    }

    pub(crate) fn update_entry(
        &mut self,
        entry: &DistannHandoffEntry,
        shape: DistannHandoffShape,
    ) -> Result<(), String> {
        self.update_encoded_entry(&entry.encode(shape)?)
    }
}

pub(crate) fn restore_owner_stream_hash_state(
    input: &[u8],
    expected_cumulative_digest: [u8; 32],
) -> Result<[u8; 32], String> {
    DistannOwnerStreamHasher::restore(input, expected_cumulative_digest).map(|state| state.digest())
}

pub fn owner_stream_digest(
    entries: &[DistannHandoffEntry],
    shape: DistannHandoffShape,
) -> Result<[u8; 32], String> {
    shape.validate()?;
    let mut hasher = DistannOwnerStreamHasher::new();
    let mut previous = None;
    for entry in entries {
        if previous.is_some_and(|value| entry.vec_id <= value) {
            return Err(
                "EC_HANDOFF_FORMAT: owner stream vec_ids are not strictly increasing".to_owned(),
            );
        }
        hasher.update_entry(entry, shape)?;
        previous = Some(entry.vec_id);
    }
    Ok(hasher.digest())
}

#[cfg(test)]
pub(crate) fn sample_handoff_entry(vec_id: u64) -> DistannHandoffEntry {
    DistannHandoffEntry {
        vec_id,
        source_identity: vec![vec_id as u8; 16],
        graph_flags: 0,
        search_code: vec![0xA0 | vec_id as u8, 0x0F],
        neighbor_vec_ids: vec![vec_id + 10, vec_id + 20],
        neighbor_codes: vec![1, 2, 3, 4],
        row_null_bitmap: vec![0b0000_0010],
        row_values: vec![vec![0x11, 0x22], vec![0x33]],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape() -> DistannHandoffShape {
        DistannHandoffShape {
            code_stride: 2,
            graph_degree: 4,
            non_dropped_attribute_count: 3,
        }
    }

    #[test]
    fn handoff_entry_round_trip_preserves_graph_and_row_bytes() {
        let entry = sample_handoff_entry(7);
        let encoded = entry.encode(shape()).unwrap();
        assert_eq!(
            DistannHandoffEntry::decode(&encoded, shape()).unwrap(),
            entry
        );
        assert_eq!(
            entry.digest(shape()).unwrap(),
            entry.digest(shape()).unwrap()
        );
        assert!(!encoded.windows(6).any(|window| window == [0xFF; 6]));
    }

    #[test]
    fn handoff_batch_round_trip_verifies_digest_and_order() {
        let batch = DistannHandoffBatch {
            epoch: 9,
            build_id: super::super::canonical_wire::sample_rfc4122_v4_uuid(0xB1),
            batch_seq: 3,
            build_spec_digest: [1; 32],
            row_schema_fingerprint: [2; 32],
            index_format_version: DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
            neighbor_codec_kind: DISTANN_NEIGHBOR_CODEC_RABITQ,
            entries: vec![sample_handoff_entry(7), sample_handoff_entry(8)],
        };
        let encoded = batch.encode(shape()).unwrap();
        assert_eq!(
            DistannHandoffBatch::verified_digest(&encoded).unwrap(),
            batch.digest(shape()).unwrap()
        );
        assert_eq!(
            DistannHandoffBatch::decode(&encoded, shape()).unwrap(),
            batch
        );
        assert_eq!(
            batch.digest(shape()).unwrap(),
            batch.digest(shape()).unwrap()
        );

        let mut invalid_uuid = batch.clone();
        invalid_uuid.build_id = [0xB1; 16];
        assert!(invalid_uuid.encode(shape()).is_err());

        let mut corrupted = encoded;
        corrupted[20] ^= 1;
        assert!(DistannHandoffBatch::decode(&corrupted, shape())
            .unwrap_err()
            .contains("EC_HANDOFF_DIGEST"));

        assert!(DistannHandoffBatch::verified_digest(
            &corrupted[..DISTANN_HANDOFF_BATCH_FIXED_PREFIX_BYTES]
        )
        .unwrap_err()
        .contains("minimum"));
    }

    #[test]
    fn handoff_preflight_rejects_shape_nulls_order_and_oversize() {
        let mut entry = sample_handoff_entry(7);
        entry.neighbor_codes.pop();
        assert!(entry.encode(shape()).is_err());

        let mut entry = sample_handoff_entry(7);
        entry.row_null_bitmap[0] = 0b1110_0010;
        assert!(entry.encode(shape()).is_err());

        let batch = DistannHandoffBatch {
            epoch: 9,
            build_id: super::super::canonical_wire::sample_rfc4122_v4_uuid(0xB1),
            batch_seq: 0,
            build_spec_digest: [1; 32],
            row_schema_fingerprint: [2; 32],
            index_format_version: DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
            neighbor_codec_kind: DISTANN_NEIGHBOR_CODEC_RABITQ,
            entries: vec![sample_handoff_entry(8), sample_handoff_entry(7)],
        };
        assert!(batch.encode(shape()).is_err());

        let mut huge = sample_handoff_entry(7);
        huge.row_values[0] = vec![0; DISTANN_HANDOFF_MAX_BYTES];
        assert!(huge
            .encode(shape())
            .unwrap_err()
            .contains("EC_HANDOFF_TOO_LARGE"));
    }

    #[test]
    fn owner_stream_digest_is_stable_and_order_sensitive() {
        let entries = vec![sample_handoff_entry(7), sample_handoff_entry(8)];
        let digest = owner_stream_digest(&entries, shape()).unwrap();
        assert_eq!(digest, owner_stream_digest(&entries, shape()).unwrap());
        let reversed = vec![entries[1].clone(), entries[0].clone()];
        assert!(owner_stream_digest(&reversed, shape()).is_err());
    }

    fn canonical_owner_payload(entries: &[DistannHandoffEntry]) -> Vec<u8> {
        let mut payload = Vec::new();
        for entry in entries {
            let encoded = entry.encode(shape()).unwrap();
            payload.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
            payload.extend_from_slice(&encoded);
        }
        payload
    }

    #[test]
    fn owner_stream_hash_initial_state_golden_is_frozen() {
        let state = DistannOwnerStreamHasher::new();
        let serialized = state.serialize();
        assert_eq!(serialized.len(), DISTANN_OWNER_STREAM_HASH_STATE_BYTES);
        assert_eq!(
            hex::encode(serialized),
            "01000167e6096a85ae67bb72f36e3c3af54fa57f520e518c68059babd9831f19cde05b00000000000000001b65635f64697374616e6e5f6f776e65725f73747265616d5f763100000000000000000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(state.digest(), owner_stream_digest(&[], shape()).unwrap());

        let restored = DistannOwnerStreamHasher::restore(&serialized, state.digest()).unwrap();
        assert_eq!(restored.serialize(), serialized);
        assert_eq!(restored.digest(), state.digest());
    }

    #[test]
    fn owner_stream_hash_resume_matches_one_shot_at_every_byte_boundary() {
        let entries = vec![
            sample_handoff_entry(7),
            sample_handoff_entry(8),
            sample_handoff_entry(9),
        ];
        let expected = owner_stream_digest(&entries, shape()).unwrap();
        let payload = canonical_owner_payload(&entries);

        for boundary in 0..=payload.len() {
            let mut prefix = DistannOwnerStreamHasher::new();
            prefix.update_bytes(&payload[..boundary]).unwrap();
            let serialized = prefix.serialize();
            let mut resumed =
                DistannOwnerStreamHasher::restore(&serialized, prefix.digest()).unwrap();
            resumed.update_bytes(&payload[boundary..]).unwrap();
            assert_eq!(resumed.digest(), expected, "byte boundary {boundary}");
        }
    }

    #[test]
    fn owner_stream_hash_resume_matches_one_shot_at_every_entry_partition() {
        let entries = vec![
            sample_handoff_entry(7),
            sample_handoff_entry(8),
            sample_handoff_entry(9),
        ];
        let expected = owner_stream_digest(&entries, shape()).unwrap();

        for boundary in 0..=entries.len() {
            let mut prefix = DistannOwnerStreamHasher::new();
            for entry in &entries[..boundary] {
                prefix.update_entry(entry, shape()).unwrap();
            }
            let serialized = prefix.serialize();
            let mut resumed =
                DistannOwnerStreamHasher::restore(&serialized, prefix.digest()).unwrap();
            for entry in &entries[boundary..] {
                let encoded = entry.encode(shape()).unwrap();
                resumed.update_encoded_entry(&encoded).unwrap();
            }
            assert_eq!(resumed.digest(), expected, "entry boundary {boundary}");
        }
    }

    #[test]
    fn owner_stream_hash_restore_rejects_malformed_state_and_digest() {
        let initial = DistannOwnerStreamHasher::new();
        let expected_digest = initial.digest();
        let serialized = initial.serialize();

        let error = DistannOwnerStreamHasher::restore(
            &serialized[..DISTANN_OWNER_STREAM_HASH_STATE_BYTES - 1],
            expected_digest,
        )
        .unwrap_err();
        assert!(error.contains("state length"));

        let mut too_long = serialized.to_vec();
        too_long.push(0);
        let error = DistannOwnerStreamHasher::restore(&too_long, expected_digest).unwrap_err();
        assert!(error.contains("state length"));

        let mut wrong_version = serialized;
        wrong_version[..2].copy_from_slice(&2_u16.to_le_bytes());
        let error = DistannOwnerStreamHasher::restore(&wrong_version, expected_digest).unwrap_err();
        assert!(error.contains("state version"));

        let mut wrong_implementation = serialized;
        wrong_implementation[2] = 2;
        let error =
            DistannOwnerStreamHasher::restore(&wrong_implementation, expected_digest).unwrap_err();
        assert!(error.contains("implementation"));

        let mut invalid_buffer_position = serialized;
        invalid_buffer_position[3 + 40] = 64;
        let error = DistannOwnerStreamHasher::restore(&invalid_buffer_position, expected_digest)
            .unwrap_err();
        assert!(error.contains("state encoding"));

        let mut nonzero_buffer_tail = serialized;
        *nonzero_buffer_tail.last_mut().unwrap() = 1;
        let error =
            DistannOwnerStreamHasher::restore(&nonzero_buffer_tail, expected_digest).unwrap_err();
        assert!(error.contains("state encoding"));

        let mut excessive_message_length = serialized;
        excessive_message_length[3 + 32..3 + 40].copy_from_slice(&u64::MAX.to_le_bytes());
        let error = DistannOwnerStreamHasher::restore(&excessive_message_length, expected_digest)
            .unwrap_err();
        assert!(error.contains("message length"));

        let mut missing_domain = serialized;
        missing_domain[3 + 40] = 0;
        missing_domain[3 + 41..].fill(0);
        let bare_digest: [u8; 32] = Sha256::digest([]).into();
        let error = DistannOwnerStreamHasher::restore(&missing_domain, bare_digest).unwrap_err();
        assert!(error.contains("message length"));

        let mut digest_mismatch = serialized;
        digest_mismatch[3] ^= 1;
        let error =
            DistannOwnerStreamHasher::restore(&digest_mismatch, expected_digest).unwrap_err();
        assert!(error.contains("EC_HANDOFF_DIGEST"));

        let error = DistannOwnerStreamHasher::restore(&serialized, [0; 32]).unwrap_err();
        assert!(error.contains("EC_HANDOFF_DIGEST"));

        let mut at_message_limit = initial;
        at_message_limit.message_bytes = u64::MAX / 8;
        let error = at_message_limit.update_encoded_entry(&[]).unwrap_err();
        assert!(error.contains("SHA-256 message length"));
    }

    #[test]
    fn empty_owner_stream_round_trip_does_not_advance_state() {
        let initial = DistannOwnerStreamHasher::new();
        let serialized = initial.serialize();
        let digest = initial.digest();
        let restored = DistannOwnerStreamHasher::restore(&serialized, digest).unwrap();

        assert_eq!(restored.message_bytes, OWNER_STREAM_DOMAIN.len() as u64);
        assert_eq!(restored.serialize(), serialized);
        assert_eq!(
            restored.digest(),
            owner_stream_digest(&[], shape()).unwrap()
        );
    }
}
