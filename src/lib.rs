use pgrx::prelude::*;

pgrx::pg_module_magic!();

// ---------------------------------------------------------------------------
// Module structure:
//
//   src/
//   ├── lib.rs              ← pgrx entry, type I/O, encode, distance, operators
//   ├── quant/              ← quantizer core (extracted from TurboQuantDB)
//   │   ├── mod.rs
//   │   ├── codebook.rs     ← Lloyd-Max codebook generation
//   │   ├── mse.rs          ← MSE quantizer (SRHT rotation + codebook)
//   │   ├── qjl.rs          ← QJL quantizer (Gaussian projection, bit-packed)
//   │   ├── prod.rs         ← ProdQuantizer orchestrator (encode, LUT score, pack/unpack)
//   │   ├── hadamard.rs     ← Fast Walsh-Hadamard Transform (AVX2 + NEON + scalar)
//   │   └── rotation.rs     ← SRHT rotation (diagonal signs + FWHT)
//   ├── am/                 ← HNSW index access method (raw pg_sys FFI)
//   │   ├── mod.rs          ← tqhnsw_handler, capability flags
//   │   ├── build.rs        ← ambuild, ambuildempty (uses hnsw_rs for construction)
//   │   ├── insert.rs       ← aminsert (page-level graph update)
//   │   ├── scan.rs         ← ambeginscan, amrescan, amgettuple, amendscan
//   │   ├── vacuum.rs       ← ambulkdelete, amvacuumcleanup
//   │   ├── cost.rs         ← amcostestimate
//   │   └── page.rs         ← TqElementTuple, TqNeighborTuple, GenericXLog helpers
//   ├── storage.rs          ← Packed code ↔ bytes for Postgres varlena/index pages
//   └── distance.rs         ← Distance impl for hnsw_rs build + pg_extern wrappers
//
// Wire format (little-endian, per code):
//   [mse_packed: ceil(n*(bits-1)/8) bytes][qjl_packed: ceil(n/8) bytes]
//   where n = dim.next_power_of_two()
//
// Metadata (dim, bits, seed, gamma) stored per-code in the varlena header
// or per-index for the AM. Not repeated in the packed code bytes.
//
// Text representation: "[dim=1536,bits=4,seed=42]:<hex>"
// ---------------------------------------------------------------------------

mod quant;
// mod am;        // TODO: HNSW index access method
// mod storage;   // TODO: packed code ↔ bytes
// mod distance;  // TODO: distance wrappers

/// Minimum header size: dim(2) + bits(1) + seed(8) = 11 bytes.
const HEADER_BYTES: usize = 11;

/// Number of code bytes for a given (dim, bits) pair.
/// MSE stage uses (bits-1) bits per dimension; QJL adds 1 bit per dimension.
/// Total = ceil(dim*(bits-1)/8) + ceil(dim/8).
fn code_len(dim: usize, bits: u8) -> usize {
    let mse_bits = (bits as usize).saturating_sub(1);
    let mse_bytes = (dim * mse_bits + 7) / 8;
    let qjl_bytes = (dim + 7) / 8;
    mse_bytes + qjl_bytes
}

// ---------------------------------------------------------------------------
// Text parse / format
// ---------------------------------------------------------------------------

fn parse_text(s: &str) -> Result<(u16, u8, u64, Vec<u8>), String> {
    let s = s.trim();
    let bracket_end = s.find(']').ok_or("missing ']'")?;
    let header = &s[1..bracket_end];
    let rest = s[bracket_end + 1..].trim_start_matches(':');

    let mut dim: Option<u16> = None;
    let mut bits: Option<u8> = None;
    let mut seed: u64 = 42;

    for part in header.split(',') {
        let (k, v) = part.split_once('=').ok_or_else(|| format!("bad header field: {part}"))?;
        match k.trim() {
            "dim"  => dim  = Some(v.trim().parse().map_err(|e| format!("dim: {e}"))?),
            "bits" => bits = Some(v.trim().parse().map_err(|e| format!("bits: {e}"))?),
            "seed" => seed = v.trim().parse().map_err(|e| format!("seed: {e}"))?,
            other  => return Err(format!("unknown header field: {other}")),
        }
    }

    let dim  = dim.ok_or("missing dim")?;
    let bits = bits.ok_or("missing bits")?;

    let codes = hex::decode(rest).map_err(|e| format!("hex decode: {e}"))?;
    let expected = code_len(dim as usize, bits);
    if codes.len() != expected {
        return Err(format!(
            "code length mismatch: got {} bytes, expected {expected}",
            codes.len()
        ));
    }
    Ok((dim, bits, seed, codes))
}

fn format_text(dim: u16, bits: u8, seed: u64, codes: &[u8]) -> String {
    format!("[dim={dim},bits={bits},seed={seed}]:{}", hex::encode(codes))
}

// ---------------------------------------------------------------------------
// Binary pack / unpack
// ---------------------------------------------------------------------------

fn pack(dim: u16, bits: u8, seed: u64, codes: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(HEADER_BYTES + codes.len());
    buf.extend_from_slice(&dim.to_le_bytes());
    buf.push(bits);
    buf.extend_from_slice(&seed.to_le_bytes());
    buf.extend_from_slice(codes);
    buf
}

fn unpack(data: &[u8]) -> Result<(u16, u8, u64, &[u8]), String> {
    if data.len() < HEADER_BYTES {
        return Err(format!(
            "tqvector too short: {} bytes (need >= {HEADER_BYTES})",
            data.len()
        ));
    }
    let dim  = u16::from_le_bytes(data[0..2].try_into().unwrap());
    let bits = data[2];
    let seed = u64::from_le_bytes(data[3..11].try_into().unwrap());
    let codes = &data[HEADER_BYTES..];
    let expected = code_len(dim as usize, bits);
    if codes.len() != expected {
        return Err(format!(
            "code length mismatch: got {} bytes, expected {expected}",
            codes.len()
        ));
    }
    Ok((dim, bits, seed, codes))
}

// ---------------------------------------------------------------------------
// 3a — Type I/O functions
// ---------------------------------------------------------------------------

#[pg_extern(immutable, strict, parallel_safe)]
fn tqvector_in(input: &core::ffi::CStr) -> Vec<u8> {
    let s = input.to_str().expect("invalid UTF-8 in tqvector input");
    let (dim, bits, seed, codes) =
        parse_text(s).unwrap_or_else(|e| pgrx::error!("invalid tqvector: {e}"));
    pack(dim, bits, seed, &codes)
}

#[pg_extern(immutable, strict, parallel_safe)]
fn tqvector_out(vec: Vec<u8>) -> &'static core::ffi::CStr {
    let (dim, bits, seed, codes) =
        unpack(&vec).unwrap_or_else(|e| pgrx::error!("corrupt tqvector: {e}"));
    let s = format_text(dim, bits, seed, codes);
    let cstr = std::ffi::CString::new(s).unwrap();
    unsafe {
        let ptr = pg_sys::palloc(cstr.as_bytes_with_nul().len()) as *mut i8;
        std::ptr::copy_nonoverlapping(cstr.as_ptr(), ptr, cstr.as_bytes_with_nul().len());
        core::ffi::CStr::from_ptr(ptr)
    }
}

#[pg_extern(immutable, strict, parallel_safe)]
fn tqvector_send(vec: Vec<u8>) -> Vec<u8> {
    vec
}

#[pg_extern(immutable, strict, parallel_safe)]
fn tqvector_recv(buf: Vec<u8>) -> Vec<u8> {
    unpack(&buf).unwrap_or_else(|e| pgrx::error!("invalid tqvector binary: {e}"));
    buf
}

// ---------------------------------------------------------------------------
// 3a — Encode helper: fp32 array → tqvector
// Called during INSERT to compress a raw float32 vector into a tqvector code.
//
// TODO: replace placeholder body with:
//   turbo_quant::encode(&embedding, bits, seed)
// once crate API is confirmed (`cargo add turbo-quant && cargo doc --open`).
// Check whether the crate exposes encode() at the top level or via a struct.
// ---------------------------------------------------------------------------

#[pg_extern(immutable, strict, parallel_safe)]
fn encode_to_tqvector(embedding: Vec<f32>, bits: i32, seed: i64) -> Vec<u8> {
    if !(2..=8).contains(&bits) {
        pgrx::error!("bits must be 2–8, got {bits}");
    }
    let dim   = embedding.len();
    let bits  = bits as u8;
    let seed  = seed as u64;

    // TODO: let codes = turbo_quant::encode(&embedding, bits, seed);
    let codes = vec![0u8; code_len(dim, bits)]; // placeholder

    pack(dim as u16, bits, seed, &codes)
}

// ---------------------------------------------------------------------------
// 3b — Distance functions
//
// tqvector_inner_product    — used inside HNSW graph traversal
// tqvector_negative_inner_product — for ORDER BY col <#> query ASC
//
// TODO: replace placeholder with:
//   turbo_quant::score_ip(codes_a, codes_b, dim, bits)
// or if the crate requires one decoded side:
//   let decoded = turbo_quant::decode_approximate(codes_b, dim, bits, seed);
//   turbo_quant::inner_product_estimate(codes_a, &decoded, dim, bits)
// ---------------------------------------------------------------------------

#[pg_extern(immutable, strict, parallel_safe)]
fn tqvector_inner_product(a: Vec<u8>, b: Vec<u8>) -> f32 {
    let (dim_a, bits_a, _, codes_a) =
        unpack(&a).unwrap_or_else(|e| pgrx::error!("tqvector_inner_product (a): {e}"));
    let (dim_b, bits_b, _, codes_b) =
        unpack(&b).unwrap_or_else(|e| pgrx::error!("tqvector_inner_product (b): {e}"));

    if dim_a != dim_b || bits_a != bits_b {
        pgrx::error!(
            "tqvector dimension/bits mismatch: ({dim_a},{bits_a}) vs ({dim_b},{bits_b})"
        );
    }

    // TODO: turbo_quant::score_ip(codes_a, codes_b, dim_a as usize, bits_a)
    let _ = (codes_a, codes_b);
    0.0f32
}

#[pg_extern(immutable, strict, parallel_safe)]
fn tqvector_negative_inner_product(a: Vec<u8>, b: Vec<u8>) -> f32 {
    -tqvector_inner_product(a, b)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[test]
    fn test_code_len_4bit_1536() {
        // 4-bit, 1536-dim: MSE=3 bits/dim → 576 bytes; QJL=1 bit/dim → 192 bytes
        assert_eq!(code_len(1536, 4), 768);
    }

    #[test]
    fn test_code_len_8bit_1536() {
        // 8-bit: MSE=7 bits/dim → 1344 bytes; QJL → 192 bytes
        assert_eq!(code_len(1536, 8), 1536);
    }

    #[test]
    fn test_pack_unpack_roundtrip() {
        let codes = vec![0xABu8; code_len(4, 4)];
        let packed = pack(4, 4, 42, &codes);
        let (dim, bits, seed, c) = unpack(&packed).unwrap();
        assert_eq!((dim, bits, seed), (4u16, 4u8, 42u64));
        assert_eq!(c, codes.as_slice());
    }

    #[test]
    fn test_text_roundtrip() {
        let codes = vec![0u8; code_len(4, 4)];
        let text  = format_text(4, 4, 42, &codes);
        let (dim, bits, seed, c) = parse_text(&text).unwrap();
        assert_eq!((dim, bits, seed), (4u16, 4u8, 42u64));
        assert_eq!(c, codes);
    }

    #[test]
    fn test_parse_text_rejects_wrong_code_length() {
        // Manually construct text with wrong hex length
        let text = "[dim=4,bits=4]:deadbeef"; // 4 bytes, but code_len(4,4) != 4
        assert!(parse_text(text).is_err());
    }
}

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {}
    pub fn postgresql_conf_options() -> Vec<&'static str> { vec![] }
}
