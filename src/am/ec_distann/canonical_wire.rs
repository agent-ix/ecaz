//! Bounded little-endian helpers shared by EC-DistANN canonical formats.
//!
//! These formats are digested identities and/or remote handoff payloads. A
//! decoder must preflight declared lengths against the enclosing byte slice
//! before allocating, reject trailing bytes, and never reinterpret host endian.

use sha2::{Digest, Sha256};

pub(crate) const DISTANN_MAX_CANONICAL_BYTES: usize = 8 * 1024 * 1024;

pub(crate) fn domain_digest(domain: &[u8], canonical: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(canonical);
    hasher.finalize().into()
}

pub(crate) fn is_rfc4122_v4_uuid(value: &[u8; 16]) -> bool {
    value != &[0; 16] && value[6] & 0xf0 == 0x40 && value[8] & 0xc0 == 0x80
}

#[cfg(test)]
pub(crate) fn sample_rfc4122_v4_uuid(fill: u8) -> [u8; 16] {
    let mut value = [fill; 16];
    value[6] = (value[6] & 0x0f) | 0x40;
    value[8] = (value[8] & 0x3f) | 0x80;
    value
}

#[derive(Default)]
pub(crate) struct CanonicalEncoder {
    bytes: Vec<u8>,
}

impl CanonicalEncoder {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(capacity),
        }
    }

    pub(crate) fn put_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub(crate) fn put_u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn put_u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn put_i32(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn put_u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn put_f32(&mut self, value: f32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn put_fixed(&mut self, value: &[u8]) {
        self.bytes.extend_from_slice(value);
    }

    pub(crate) fn put_len_prefixed(&mut self, value: &[u8]) -> Result<(), String> {
        let length = u32::try_from(value.len())
            .map_err(|_| "ec_distann canonical field exceeds u32 length".to_owned())?;
        self.put_u32(length);
        self.put_fixed(value);
        Ok(())
    }

    pub(crate) fn put_string(&mut self, value: &str) -> Result<(), String> {
        self.put_len_prefixed(value.as_bytes())
    }

    pub(crate) fn len(&self) -> usize {
        self.bytes.len()
    }

    pub(crate) fn finish(self) -> Result<Vec<u8>, String> {
        if self.bytes.len() > DISTANN_MAX_CANONICAL_BYTES {
            return Err(format!(
                "ec_distann canonical value is {} bytes, exceeding {}",
                self.bytes.len(),
                DISTANN_MAX_CANONICAL_BYTES
            ));
        }
        Ok(self.bytes)
    }
}

pub(crate) struct CanonicalDecoder<'a> {
    input: &'a [u8],
    position: usize,
}

impl<'a> CanonicalDecoder<'a> {
    pub(crate) fn new(input: &'a [u8], format_name: &str) -> Result<Self, String> {
        if input.len() > DISTANN_MAX_CANONICAL_BYTES {
            return Err(format!(
                "{format_name} is {} bytes, exceeding {}",
                input.len(),
                DISTANN_MAX_CANONICAL_BYTES
            ));
        }
        Ok(Self { input, position: 0 })
    }

    fn take(&mut self, length: usize, field: &str) -> Result<&'a [u8], String> {
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| format!("ec_distann {field} offset overflow"))?;
        if end > self.input.len() {
            return Err(format!(
                "ec_distann truncated {field}: need {length} bytes, have {}",
                self.input.len().saturating_sub(self.position)
            ));
        }
        let value = &self.input[self.position..end];
        self.position = end;
        Ok(value)
    }

    pub(crate) fn get_u8(&mut self, field: &str) -> Result<u8, String> {
        Ok(self.take(1, field)?[0])
    }

    pub(crate) fn get_u16(&mut self, field: &str) -> Result<u16, String> {
        Ok(u16::from_le_bytes(
            self.take(2, field)?.try_into().expect("two bytes"),
        ))
    }

    pub(crate) fn get_u32(&mut self, field: &str) -> Result<u32, String> {
        Ok(u32::from_le_bytes(
            self.take(4, field)?.try_into().expect("four bytes"),
        ))
    }

    pub(crate) fn get_i32(&mut self, field: &str) -> Result<i32, String> {
        Ok(i32::from_le_bytes(
            self.take(4, field)?.try_into().expect("four bytes"),
        ))
    }

    pub(crate) fn get_u64(&mut self, field: &str) -> Result<u64, String> {
        Ok(u64::from_le_bytes(
            self.take(8, field)?.try_into().expect("eight bytes"),
        ))
    }

    pub(crate) fn get_f32(&mut self, field: &str) -> Result<f32, String> {
        Ok(f32::from_le_bytes(
            self.take(4, field)?.try_into().expect("four bytes"),
        ))
    }

    pub(crate) fn get_fixed<const N: usize>(&mut self, field: &str) -> Result<[u8; N], String> {
        Ok(self.take(N, field)?.try_into().expect("fixed-width bytes"))
    }

    pub(crate) fn get_bytes(&mut self, length: usize, field: &str) -> Result<&'a [u8], String> {
        self.take(length, field)
    }

    pub(crate) fn get_len_prefixed(&mut self, field: &str) -> Result<&'a [u8], String> {
        let length = self.get_u32(&format!("{field} length"))? as usize;
        self.take(length, field)
    }

    pub(crate) fn get_owned_bytes(&mut self, field: &str) -> Result<Vec<u8>, String> {
        Ok(self.get_len_prefixed(field)?.to_vec())
    }

    pub(crate) fn get_string(&mut self, field: &str) -> Result<String, String> {
        let bytes = self.get_len_prefixed(field)?;
        let value = std::str::from_utf8(bytes)
            .map_err(|_| format!("ec_distann {field} is not valid UTF-8"))?;
        Ok(value.to_owned())
    }

    pub(crate) fn remaining(&self) -> usize {
        self.input.len() - self.position
    }

    pub(crate) fn finish(self, format_name: &str) -> Result<(), String> {
        if self.position != self.input.len() {
            return Err(format!(
                "ec_distann {format_name} has {} trailing bytes",
                self.input.len() - self.position
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_scalar_and_length_round_trip() {
        let mut encoder = CanonicalEncoder::default();
        encoder.put_u8(7);
        encoder.put_u16(0x1234);
        encoder.put_u32(0x5566_7788);
        encoder.put_i32(-42);
        encoder.put_u64(0x0102_0304_0506_0708);
        encoder.put_f32(-1.25);
        encoder.put_string("schema.type").unwrap();
        let bytes = encoder.finish().unwrap();

        let mut decoder = CanonicalDecoder::new(&bytes, "test canonical").unwrap();
        assert_eq!(decoder.get_u8("u8").unwrap(), 7);
        assert_eq!(decoder.get_u16("u16").unwrap(), 0x1234);
        assert_eq!(decoder.get_u32("u32").unwrap(), 0x5566_7788);
        assert_eq!(decoder.get_i32("i32").unwrap(), -42);
        assert_eq!(decoder.get_u64("u64").unwrap(), 0x0102_0304_0506_0708);
        assert_eq!(
            decoder.get_f32("f32").unwrap().to_bits(),
            (-1.25_f32).to_bits()
        );
        assert_eq!(decoder.get_string("string").unwrap(), "schema.type");
        decoder.finish("test canonical").unwrap();
    }

    #[test]
    fn canonical_decoder_rejects_truncation_and_trailing_bytes() {
        let mut decoder = CanonicalDecoder::new(&[4, 0, 0, 0, 1], "test").unwrap();
        assert!(decoder.get_len_prefixed("payload").is_err());

        let mut decoder = CanonicalDecoder::new(&[1, 2], "test").unwrap();
        assert_eq!(decoder.get_u8("first").unwrap(), 1);
        assert!(decoder.finish("test").is_err());
    }
}
