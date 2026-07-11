//! PostgreSQL-managed temporary spool for frozen source-row payloads.
//!
//! Vamana construction retains source vectors and stable identities, but full
//! PostgreSQL row payloads can be much larger than the graph workspace. Each
//! payload is therefore written once to a resource-owner-scoped `BufFile` and
//! addressed by a small `(file, offset, encoded length)` locator until the
//! stitched graph emits the corresponding owner handoff entry.

#[cfg(not(test))]
use std::ptr::NonNull;

#[cfg(not(test))]
use pgrx::pg_sys;

use super::canonical_wire::{CanonicalDecoder, CanonicalEncoder};
use super::handoff_wire::{DistannHandoffEntry, DistannHandoffShape, DISTANN_HANDOFF_MAX_BYTES};

const SOURCE_PAYLOAD_SPOOL_VERSION: u16 = 1;
const DISTANN_BUFFILE_SEEK_SET: i32 = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FrozenSourcePayload {
    pub(crate) vec_id: u64,
    pub(crate) source_identity: [u8; 16],
    pub(crate) row_null_bitmap: Vec<u8>,
    pub(crate) row_values: Vec<Vec<u8>>,
}

impl FrozenSourcePayload {
    fn validate_row_shape(&self, non_dropped_attribute_count: usize) -> Result<(), String> {
        if non_dropped_attribute_count > u16::MAX as usize {
            return Err("EC_SCHEMA_UNSUPPORTED: frozen row has too many attributes".to_owned());
        }
        let expected_bitmap_bytes = non_dropped_attribute_count.div_ceil(8);
        if self.row_null_bitmap.len() != expected_bitmap_bytes {
            return Err(format!(
                "EC_SOURCE_SNAPSHOT: frozen row NULL bitmap is {} bytes, expected {expected_bitmap_bytes}",
                self.row_null_bitmap.len()
            ));
        }
        if non_dropped_attribute_count % 8 != 0 && !self.row_null_bitmap.is_empty() {
            let used = non_dropped_attribute_count % 8;
            let padding_mask = !((1_u8 << used) - 1);
            if self.row_null_bitmap.last().copied().unwrap_or(0) & padding_mask != 0 {
                return Err(
                    "EC_SOURCE_SNAPSHOT: frozen row NULL bitmap padding is non-zero".to_owned(),
                );
            }
        }
        let null_count = (0..non_dropped_attribute_count)
            .filter(|attribute| self.row_null_bitmap[attribute / 8] & (1 << (attribute % 8)) != 0)
            .count();
        let expected_values = non_dropped_attribute_count - null_count;
        if self.row_values.len() != expected_values {
            return Err(format!(
                "EC_SOURCE_SNAPSHOT: frozen row has {} values, expected {expected_values}",
                self.row_values.len()
            ));
        }
        Ok(())
    }

    pub(super) fn encode(&self, non_dropped_attribute_count: usize) -> Result<Vec<u8>, String> {
        self.validate_row_shape(non_dropped_attribute_count)?;
        let mut encoder = CanonicalEncoder::with_capacity(64);
        encoder.put_u16(SOURCE_PAYLOAD_SPOOL_VERSION);
        encoder.put_u64(self.vec_id);
        encoder.put_fixed(&self.source_identity);
        encoder.put_len_prefixed(&self.row_null_bitmap)?;
        encoder
            .put_u32(u32::try_from(self.row_values.len()).map_err(|_| {
                "EC_SOURCE_SNAPSHOT: frozen row value count exceeds u32".to_owned()
            })?);
        for value in &self.row_values {
            encoder.put_len_prefixed(value)?;
        }
        encoder
            .finish()
            .map_err(|_| "EC_HANDOFF_TOO_LARGE: frozen source payload exceeds 8 MiB".to_owned())
    }

    pub(super) fn decode(input: &[u8], non_dropped_attribute_count: usize) -> Result<Self, String> {
        if input.len() > DISTANN_HANDOFF_MAX_BYTES {
            return Err("EC_HANDOFF_TOO_LARGE: frozen source payload exceeds 8 MiB".to_owned());
        }
        let mut decoder = CanonicalDecoder::new(input, "frozen source payload")?;
        let version = decoder.get_u16("frozen source payload version")?;
        if version != SOURCE_PAYLOAD_SPOOL_VERSION {
            return Err(format!(
                "EC_SOURCE_SNAPSHOT: unsupported frozen source payload version {version}"
            ));
        }
        let vec_id = decoder.get_u64("frozen source vec id")?;
        let source_identity = decoder.get_fixed("frozen source identity")?;
        let row_null_bitmap = decoder.get_owned_bytes("frozen row NULL bitmap")?;
        let value_count = decoder.get_u32("frozen row value count")? as usize;
        if value_count > non_dropped_attribute_count {
            return Err("EC_SOURCE_SNAPSHOT: invalid frozen row value count".to_owned());
        }
        let mut row_values = Vec::with_capacity(value_count);
        for _ in 0..value_count {
            row_values.push(decoder.get_owned_bytes("frozen row value")?);
        }
        decoder.finish("frozen source payload")?;
        let payload = Self {
            vec_id,
            source_identity,
            row_null_bitmap,
            row_values,
        };
        payload.validate_row_shape(non_dropped_attribute_count)?;
        Ok(payload)
    }

    /// Validate the complete largest possible v1 handoff entry for this row
    /// before graph construction or any participant begin call.
    pub(super) fn preflight_handoff_size(&self, shape: DistannHandoffShape) -> Result<(), String> {
        let entry = DistannHandoffEntry {
            vec_id: self.vec_id,
            source_identity: self.source_identity.to_vec(),
            graph_flags: 0,
            search_code: vec![0; shape.code_stride],
            neighbor_vec_ids: vec![0; shape.graph_degree],
            neighbor_codes: vec![0; shape.graph_degree.saturating_mul(shape.code_stride)],
            row_null_bitmap: self.row_null_bitmap.clone(),
            row_values: self.row_values.clone(),
        };
        entry.encode(shape).map(|_| ())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SourcePayloadLocator {
    file_number: i32,
    offset: i64,
    encoded_bytes: u32,
}

#[cfg(not(test))]
pub(super) struct SourcePayloadSpool {
    file: NonNull<pg_sys::BufFile>,
}

#[cfg(not(test))]
impl SourcePayloadSpool {
    pub(super) fn create() -> Result<Self, String> {
        let file = unsafe { pg_sys::BufFileCreateTemp(false) };
        let file = NonNull::new(file)
            .ok_or_else(|| "EC_SOURCE_SNAPSHOT: could not create source BufFile".to_owned())?;
        Ok(Self { file })
    }

    fn write_all(&mut self, bytes: &[u8]) {
        if !bytes.is_empty() {
            unsafe {
                pg_sys::BufFileWrite(
                    self.file.as_ptr(),
                    bytes.as_ptr().cast::<std::ffi::c_void>(),
                    bytes.len(),
                );
            }
        }
    }

    fn read_exact(&mut self, bytes: &mut [u8]) -> Result<(), String> {
        let mut read = 0;
        while read < bytes.len() {
            let count = unsafe {
                pg_sys::BufFileRead(
                    self.file.as_ptr(),
                    bytes[read..].as_mut_ptr().cast::<std::ffi::c_void>(),
                    bytes.len() - read,
                )
            };
            if count == 0 {
                return Err("EC_SOURCE_SNAPSHOT: truncated source BufFile entry".to_owned());
            }
            read += count;
        }
        Ok(())
    }

    pub(super) fn append(
        &mut self,
        payload: &FrozenSourcePayload,
        non_dropped_attribute_count: usize,
    ) -> Result<SourcePayloadLocator, String> {
        let encoded = payload.encode(non_dropped_attribute_count)?;
        let encoded_bytes = u32::try_from(encoded.len())
            .map_err(|_| "EC_HANDOFF_TOO_LARGE: frozen source payload exceeds u32".to_owned())?;
        let mut file_number = -1;
        let mut offset = -1;
        unsafe { pg_sys::BufFileTell(self.file.as_ptr(), &mut file_number, &mut offset) };
        if file_number < 0 || offset < 0 {
            return Err("EC_SOURCE_SNAPSHOT: invalid source BufFile position".to_owned());
        }
        self.write_all(&encoded_bytes.to_le_bytes());
        self.write_all(&encoded);
        Ok(SourcePayloadLocator {
            file_number,
            offset,
            encoded_bytes,
        })
    }

    pub(super) fn read(
        &mut self,
        locator: SourcePayloadLocator,
        non_dropped_attribute_count: usize,
    ) -> Result<FrozenSourcePayload, String> {
        let status = unsafe {
            pg_sys::BufFileSeek(
                self.file.as_ptr(),
                locator.file_number,
                locator.offset,
                DISTANN_BUFFILE_SEEK_SET,
            )
        };
        if status != 0 {
            return Err("EC_SOURCE_SNAPSHOT: could not seek source BufFile".to_owned());
        }
        let mut length = [0; 4];
        self.read_exact(&mut length)?;
        let stored_bytes = u32::from_le_bytes(length);
        if stored_bytes != locator.encoded_bytes
            || stored_bytes as usize > DISTANN_HANDOFF_MAX_BYTES
        {
            return Err("EC_SOURCE_SNAPSHOT: source BufFile locator length mismatch".to_owned());
        }
        let mut encoded = vec![0; stored_bytes as usize];
        self.read_exact(&mut encoded)?;
        FrozenSourcePayload::decode(&encoded, non_dropped_attribute_count)
    }
}

#[cfg(not(test))]
impl Drop for SourcePayloadSpool {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            unsafe { pg_sys::BufFileClose(self.file.as_ptr()) };
        }
    }
}

#[cfg(test)]
pub(super) struct SourcePayloadSpool {
    bytes: Vec<u8>,
}

#[cfg(test)]
impl SourcePayloadSpool {
    pub(super) fn create() -> Result<Self, String> {
        Ok(Self { bytes: Vec::new() })
    }

    pub(super) fn append(
        &mut self,
        payload: &FrozenSourcePayload,
        non_dropped_attribute_count: usize,
    ) -> Result<SourcePayloadLocator, String> {
        let encoded = payload.encode(non_dropped_attribute_count)?;
        let encoded_bytes = u32::try_from(encoded.len()).unwrap();
        let offset = i64::try_from(self.bytes.len()).unwrap();
        self.bytes.extend_from_slice(&encoded_bytes.to_le_bytes());
        self.bytes.extend_from_slice(&encoded);
        Ok(SourcePayloadLocator {
            file_number: 0,
            offset,
            encoded_bytes,
        })
    }

    pub(super) fn read(
        &mut self,
        locator: SourcePayloadLocator,
        non_dropped_attribute_count: usize,
    ) -> Result<FrozenSourcePayload, String> {
        if locator.file_number != 0 || locator.offset < 0 {
            return Err("EC_SOURCE_SNAPSHOT: invalid test source locator".to_owned());
        }
        let start = usize::try_from(locator.offset).unwrap();
        let length_end = start
            .checked_add(4)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| "EC_SOURCE_SNAPSHOT: truncated test source length".to_owned())?;
        let stored_bytes = u32::from_le_bytes(self.bytes[start..length_end].try_into().unwrap());
        if stored_bytes != locator.encoded_bytes {
            return Err("EC_SOURCE_SNAPSHOT: test source locator length mismatch".to_owned());
        }
        let end = length_end
            .checked_add(stored_bytes as usize)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| "EC_SOURCE_SNAPSHOT: truncated test source payload".to_owned())?;
        FrozenSourcePayload::decode(&self.bytes[length_end..end], non_dropped_attribute_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload() -> FrozenSourcePayload {
        FrozenSourcePayload {
            vec_id: 7,
            source_identity: [0x77; 16],
            row_null_bitmap: vec![0b0000_0010],
            row_values: vec![vec![1, 2], vec![3], vec![4, 5, 6]],
        }
    }

    #[test]
    fn source_spool_round_trips_multiple_random_access_rows() {
        let mut spool = SourcePayloadSpool::create().unwrap();
        let first = payload();
        let first_locator = spool.append(&first, 4).unwrap();
        let mut second = payload();
        second.vec_id = 9;
        second.source_identity = [0x99; 16];
        let second_locator = spool.append(&second, 4).unwrap();

        assert_eq!(spool.read(second_locator, 4).unwrap(), second);
        assert_eq!(spool.read(first_locator, 4).unwrap(), first);
    }

    #[test]
    fn source_spool_rejects_shape_corruption_and_oversize_handoff() {
        let payload = payload();
        assert!(payload.encode(5).is_err());

        let mut corrupt = payload.encode(4).unwrap();
        corrupt[0] = 2;
        assert!(FrozenSourcePayload::decode(&corrupt, 4).is_err());

        let mut huge = payload;
        huge.row_values[0] = vec![0; DISTANN_HANDOFF_MAX_BYTES];
        let shape = DistannHandoffShape {
            code_stride: 8,
            graph_degree: 32,
            non_dropped_attribute_count: 4,
        };
        assert!(huge
            .preflight_handoff_size(shape)
            .unwrap_err()
            .contains("EC_HANDOFF_TOO_LARGE"));
    }
}
