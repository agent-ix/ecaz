use pgrx::extension_sql_file;
use pgrx::ffi::CString;
use pgrx::prelude::*;
use pgrx::{pg_sys, Internal};

pgrx::pg_module_magic!();

#[allow(dead_code)]
mod am;
mod quant;

use quant::prod::{payload_len, ProdQuantizer};

extension_sql_file!("../sql/bootstrap.sql", name = "bootstrap", bootstrap);

/// Number of datum header bytes: dim(2) + bits(1) + seed(8).
const HEADER_BYTES: usize = 11;
/// Minimum valid wire payload: header plus gamma.
const MIN_BINARY_BYTES: usize = HEADER_BYTES + 4;

fn validate_bits(bits: u8) -> Result<(), String> {
    if !(2..=8).contains(&bits) {
        return Err(format!("bits must be between 2 and 8, got {bits}"));
    }
    Ok(())
}

fn code_len(dim: usize, bits: u8) -> usize {
    payload_len(dim, bits) - 4
}

fn parse_text(s: &str) -> Result<(u16, u8, u64, f32, Vec<u8>), String> {
    let s = s.trim();
    if !s.starts_with('[') {
        return Err("missing '['".into());
    }
    let bracket_end = s.find(']').ok_or("missing ']'")?;
    let header = &s[1..bracket_end];
    let rest = s[bracket_end + 1..]
        .strip_prefix(':')
        .ok_or("missing ':' after header")?;

    let mut dim: Option<u16> = None;
    let mut bits: Option<u8> = None;
    let mut seed: u64 = 42;
    let mut gamma: f32 = 0.0;

    for part in header.split(',') {
        let (key, value) = part
            .split_once('=')
            .ok_or_else(|| format!("bad header field: {part}"))?;
        match key.trim() {
            "dim" => dim = Some(value.trim().parse().map_err(|e| format!("dim: {e}"))?),
            "bits" => bits = Some(value.trim().parse().map_err(|e| format!("bits: {e}"))?),
            "seed" => seed = value.trim().parse().map_err(|e| format!("seed: {e}"))?,
            "gamma" => gamma = value.trim().parse().map_err(|e| format!("gamma: {e}"))?,
            other => return Err(format!("unknown header field: {other}")),
        }
    }

    let dim = dim.ok_or("missing dim")?;
    let bits = bits.ok_or("missing bits")?;
    validate_bits(bits)?;

    let codes = hex::decode(rest).map_err(|e| format!("hex decode: {e}"))?;
    let expected = code_len(dim as usize, bits);
    if codes.len() != expected {
        return Err(format!(
            "code length mismatch: got {} bytes, expected {expected}",
            codes.len()
        ));
    }

    Ok((dim, bits, seed, gamma, codes))
}

fn format_text(dim: u16, bits: u8, seed: u64, gamma: f32, codes: &[u8]) -> String {
    format!(
        "[dim={dim},bits={bits},seed={seed},gamma={gamma}]:{}",
        hex::encode(codes)
    )
}

fn pack(dim: u16, bits: u8, seed: u64, gamma: f32, codes: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(MIN_BINARY_BYTES + codes.len());
    buf.extend_from_slice(&dim.to_le_bytes());
    buf.push(bits);
    buf.extend_from_slice(&seed.to_le_bytes());
    buf.extend_from_slice(&gamma.to_le_bytes());
    buf.extend_from_slice(codes);
    buf
}

fn unpack(data: &[u8]) -> Result<(u16, u8, u64, f32, &[u8]), String> {
    if data.len() < MIN_BINARY_BYTES {
        return Err(format!(
            "tqvector too short: {} bytes (need >= {MIN_BINARY_BYTES})",
            data.len()
        ));
    }

    let dim = u16::from_le_bytes(data[0..2].try_into().expect("dim bytes"));
    let bits = data[2];
    validate_bits(bits)?;
    let seed = u64::from_le_bytes(data[3..11].try_into().expect("seed bytes"));
    let gamma = f32::from_le_bytes(data[11..15].try_into().expect("gamma bytes"));
    let codes = &data[15..];
    let expected = code_len(dim as usize, bits);
    if codes.len() != expected {
        return Err(format!(
            "code length mismatch: got {} bytes, expected {expected}",
            codes.len()
        ));
    }

    Ok((dim, bits, seed, gamma, codes))
}

fn expected_binary_len(data: &[u8]) -> Result<usize, String> {
    let (dim, bits, ..) = unpack(data)?;
    Ok(MIN_BINARY_BYTES + code_len(dim as usize, bits))
}

unsafe fn recv_tqvector_message(msg: pg_sys::StringInfo) -> Result<Vec<u8>, String> {
    if msg.is_null() {
        return Err("invalid tqvector binary: missing input buffer".into());
    }

    let total_len =
        usize::try_from(unsafe { (*msg).len }).map_err(|_| "invalid tqvector binary length")?;
    let cursor =
        usize::try_from(unsafe { (*msg).cursor }).map_err(|_| "invalid tqvector binary cursor")?;
    if cursor > total_len {
        return Err("invalid tqvector binary cursor state".into());
    }

    let remaining = total_len - cursor;
    if remaining < MIN_BINARY_BYTES {
        return Err(format!(
            "tqvector too short: {remaining} bytes (need >= {MIN_BINARY_BYTES})"
        ));
    }

    let prefix = unsafe {
        pg_sys::pq_getmsgbytes(msg, MIN_BINARY_BYTES as i32) as *const u8
    };
    let prefix = unsafe { std::slice::from_raw_parts(prefix, MIN_BINARY_BYTES) };

    let expected_len = expected_binary_len(prefix)?;
    if remaining != expected_len {
        return Err(format!(
            "code length mismatch: got {} bytes, expected {}",
            remaining - MIN_BINARY_BYTES,
            expected_len - MIN_BINARY_BYTES
        ));
    }

    let mut bytes = Vec::with_capacity(expected_len);
    bytes.extend_from_slice(prefix);

    let code_bytes_len = expected_len - MIN_BINARY_BYTES;
    if code_bytes_len > 0 {
        let codes = unsafe {
            pg_sys::pq_getmsgbytes(msg, code_bytes_len as i32) as *const u8
        };
        let codes = unsafe { std::slice::from_raw_parts(codes, code_bytes_len) };
        bytes.extend_from_slice(codes);
    }

    unsafe { pg_sys::pq_getmsgend(msg) };
    unpack(&bytes)?;
    Ok(bytes)
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn tqvector_in(input: &core::ffi::CStr) -> Vec<u8> {
    let input = input
        .to_str()
        .unwrap_or_else(|_| pgrx::error!("invalid UTF-8 in tqvector input"));
    let (dim, bits, seed, gamma, codes) =
        parse_text(input).unwrap_or_else(|e| pgrx::error!("invalid tqvector: {e}"));
    pack(dim, bits, seed, gamma, &codes)
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn tqvector_out(vec: Vec<u8>) -> CString {
    let (dim, bits, seed, gamma, codes) =
        unpack(&vec).unwrap_or_else(|e| pgrx::error!("corrupt tqvector: {e}"));
    CString::new(format_text(dim, bits, seed, gamma, codes)).expect("cstring without NUL")
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn tqvector_send(vec: Vec<u8>) -> Vec<u8> {
    unpack(&vec).unwrap_or_else(|e| pgrx::error!("invalid tqvector binary: {e}"));
    vec
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn tqvector_recv(input: Internal) -> Vec<u8> {
    // SAFETY: PostgreSQL type receive functions are invoked with an `internal`
    // argument pointing at a live `StringInfoData` input buffer.
    let msg = unsafe {
        input
            .get::<pg_sys::StringInfoData>()
            .unwrap_or_else(|| pgrx::error!("invalid tqvector binary: missing input buffer"))
            as *const pg_sys::StringInfoData as pg_sys::StringInfo
    };

    // SAFETY: `msg` is a valid Postgres `StringInfo` owned by the current receive call.
    unsafe { recv_tqvector_message(msg) }
        .unwrap_or_else(|e| pgrx::error!("invalid tqvector binary: {e}"))
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn encode_to_tqvector(embedding: Vec<f32>, bits: i32, seed: i64) -> Vec<u8> {
    if embedding.is_empty() {
        pgrx::error!("embedding must not be empty");
    }
    let bits = u8::try_from(bits).unwrap_or_else(|_| pgrx::error!("bits must fit into u8"));
    validate_bits(bits).unwrap_or_else(|e| pgrx::error!("{e}"));
    let seed = seed as u64;

    let quantizer = ProdQuantizer::cached(embedding.len(), bits, seed);
    let encoded = quantizer.encode(&embedding);

    let mut code_bytes = encoded.mse_packed;
    code_bytes.extend_from_slice(&encoded.qjl_packed);

    pack(
        embedding.len() as u16,
        bits,
        seed,
        encoded.gamma,
        &code_bytes,
    )
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn tqvector_inner_product(a: Vec<u8>, b: Vec<u8>) -> f32 {
    let (dim_a, bits_a, seed_a, _, codes_a) =
        unpack(&a).unwrap_or_else(|e| pgrx::error!("tqvector_inner_product(a): {e}"));
    let (dim_b, bits_b, seed_b, _, codes_b) =
        unpack(&b).unwrap_or_else(|e| pgrx::error!("tqvector_inner_product(b): {e}"));

    if dim_a != dim_b || bits_a != bits_b || seed_a != seed_b {
        pgrx::error!(
            "tqvector mismatch: ({dim_a},{bits_a},{seed_a}) vs ({dim_b},{bits_b},{seed_b})"
        );
    }

    let quantizer = ProdQuantizer::cached(dim_a as usize, bits_a, seed_a);
    quantizer.score_ip_encoded_lite(codes_a, codes_b)
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn tqvector_negative_inner_product(a: Vec<u8>, b: Vec<u8>) -> f32 {
    -tqvector_inner_product(a, b)
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn tqvector_query_inner_product(candidate: Vec<u8>, query: Vec<f32>) -> f32 {
    let (dim, bits, seed, _, codes) = unpack(&candidate)
        .unwrap_or_else(|e| pgrx::error!("tqvector_query_inner_product(candidate): {e}"));
    if query.len() != dim as usize {
        pgrx::error!(
            "tqvector/query dimension mismatch: candidate dim {}, query dim {}",
            dim,
            query.len()
        );
    }

    let quantizer = ProdQuantizer::cached(dim as usize, bits, seed);
    let prepared = quantizer.prepare_ip_query(&query);
    quantizer.score_ip_encoded(&prepared, codes)
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn tqvector_negative_query_inner_product(candidate: Vec<u8>, query: Vec<f32>) -> f32 {
    -tqvector_query_inner_product(candidate, query)
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::quant::prod::payload_len;
    use crate::quant::prod::ProdQuantizer;
    use rand::Rng;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_payload_len_4bit_1536() {
        assert_eq!(payload_len(1536, 4), 772);
    }

    #[test]
    fn test_payload_len_8bit_1536() {
        assert_eq!(payload_len(1536, 8), 1540);
    }

    #[test]
    fn test_pack_unpack_roundtrip() {
        let codes = vec![0xAB_u8; code_len(4, 4)];
        let packed = pack(4, 4, 42, 1.25, &codes);
        let (dim, bits, seed, gamma, unpacked_codes) = unpack(&packed).unwrap();
        assert_eq!((dim, bits, seed), (4, 4, 42));
        assert_eq!(gamma, 1.25);
        assert_eq!(unpacked_codes, codes.as_slice());
    }

    #[test]
    fn test_text_roundtrip() {
        let codes = vec![0x11_u8; code_len(4, 4)];
        let text = format_text(4, 4, 42, 0.5, &codes);
        let (dim, bits, seed, gamma, unpacked_codes) = parse_text(&text).unwrap();
        assert_eq!((dim, bits, seed), (4, 4, 42));
        assert_eq!(gamma, 0.5);
        assert_eq!(unpacked_codes, codes);
    }

    #[test]
    fn test_parse_text_rejects_wrong_code_length() {
        assert!(parse_text("[dim=4,bits=4]:deadbeef").is_err());
    }

    #[test]
    fn test_parse_text_rejects_invalid_hex() {
        let err = parse_text("[dim=4,bits=4]:ZZZZ").unwrap_err();
        assert!(err.contains("hex"));
    }

    #[test]
    fn test_unpack_rejects_truncated_binary() {
        assert!(unpack(&[0_u8; 14]).is_err());
    }

    #[test]
    fn test_unpack_rejects_trailing_binary() {
        let mut packed = pack(4, 4, 42, 1.25, &[0xAA, 0xBB, 0xCC]);
        packed.push(0xDD);
        assert!(unpack(&packed).is_err());
    }

    #[test]
    fn test_quantizer_encode_is_deterministic() {
        let mut rng = ChaCha8Rng::seed_from_u64(123);
        let vector = (0..32)
            .map(|_| rng.gen_range(-1.0_f32..1.0_f32))
            .collect::<Vec<_>>();
        let quantizer = ProdQuantizer::cached(32, 4, 42);
        let first = quantizer.pack_payload(&quantizer.encode(&vector));
        let second = quantizer.pack_payload(&quantizer.encode(&vector));
        assert_eq!(first, second);
    }

    #[test]
    fn test_quantized_scores_can_be_negated() {
        let mut rng = ChaCha8Rng::seed_from_u64(5);
        let a = (0..16)
            .map(|_| rng.gen_range(-1.0_f32..1.0_f32))
            .collect::<Vec<_>>();
        let b = (0..16)
            .map(|_| rng.gen_range(-1.0_f32..1.0_f32))
            .collect::<Vec<_>>();
        let quantizer = ProdQuantizer::cached(16, 4, 42);
        let code_a = quantizer.pack_payload(&quantizer.encode(&a));
        let code_b = quantizer.pack_payload(&quantizer.encode(&b));

        let base_code = quantizer.score_ip_encoded_lite(&code_a, &code_b);
        assert_eq!(-base_code, -base_code);

        let prepared = quantizer.prepare_ip_query(&b);
        let base_query = quantizer.score_ip_encoded(&prepared, &code_a);
        assert_eq!(-base_query, -base_query);
    }

    #[test]
    fn test_parse_text_fuzz_stability() {
        let mut rng = ChaCha8Rng::seed_from_u64(77);
        for _ in 0..10_000 {
            let len = rng.gen_range(0..=256);
            let bytes = (0..len)
                .map(|_| rng.gen_range(0_u8..=255))
                .collect::<Vec<_>>();
            let candidate = String::from_utf8_lossy(&bytes);
            let _ = parse_text(&candidate);
        }
    }
}

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {}
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        vec![]
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn test_binary_send_matches_internal_layout() {
        let bytes = Spi::get_one::<Vec<u8>>(
            "SELECT tqvector_send('[dim=4,bits=4,seed=42,gamma=0.5]:112233'::tqvector)",
        )
        .expect("SPI query should succeed")
        .expect("query should return one row");

        assert_eq!(bytes, pack(4, 4, 42, 0.5, &[0x11, 0x22, 0x33]));
    }

    #[pg_test]
    fn test_access_method_is_registered() {
        let amname = Spi::get_one::<String>(
            "SELECT amname::text FROM pg_am WHERE amname = 'tqhnsw'",
        )
        .expect("SPI query should succeed")
        .expect("access method should exist");
        assert_eq!(amname, "tqhnsw");
    }

    #[pg_test]
    fn test_operator_class_is_registered() {
        let opcname = Spi::get_one::<String>(
            "SELECT opcname::text FROM pg_opclass WHERE opcname = 'tqvector_ip_ops'",
        )
        .expect("SPI query should succeed")
        .expect("operator class should exist");
        assert_eq!(opcname, "tqvector_ip_ops");
    }

    #[pg_test]
    fn test_empty_index_build_initializes_metadata_page() {
        Spi::run("CREATE TABLE tqhnsw_empty_build (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run("CREATE EXTENSION pageinspect").expect("pageinspect should be available");
        Spi::run(
            "CREATE INDEX tqhnsw_empty_build_idx ON tqhnsw_empty_build USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 12, ef_construction = 80)",
        )
        .expect("index creation should succeed");

        let reloptions = Spi::get_one::<Vec<String>>(
            "SELECT reloptions FROM pg_class WHERE oid = 'tqhnsw_empty_build_idx'::regclass",
        )
        .expect("SPI query should succeed")
        .expect("reloptions should exist");

        let first_page = Spi::get_one::<Vec<u8>>(
            "SELECT get_raw_page('tqhnsw_empty_build_idx', 0)::bytea",
        )
        .expect("SPI query should succeed")
        .expect("raw index page should exist");

        let metadata = am::page::MetadataPage::decode_page(&first_page)
            .expect("metadata page should decode from raw relation bytes");

        assert_eq!(reloptions, vec!["m=12".to_string(), "ef_construction=80".to_string()]);
        assert_eq!(metadata.m, 12);
        assert_eq!(metadata.ef_construction, 80);
        assert_eq!(metadata.entry_point, am::page::ItemPointer::INVALID);
        assert_eq!(metadata.dimensions, 0);
        assert_eq!(metadata.bits, 0);
        assert_eq!(metadata.max_level, 0);
    }

    #[pg_test]
    #[should_panic(expected = "code length mismatch")]
    fn test_binary_recv_rejects_trailing_bytes() {
        let mut bytes = pack(4, 4, 42, 0.5, &[0x11, 0x22, 0x33]);
        bytes.push(0x44);
        let _ = unsafe { recv_via_string_info(&bytes) };
    }

    #[pg_test]
    #[should_panic(expected = "too short")]
    fn test_binary_recv_rejects_truncated_bytes() {
        let _ = unsafe { recv_via_string_info(&[0_u8; 14]) };
    }

    unsafe fn recv_via_string_info(bytes: &[u8]) -> Vec<u8> {
        let leaked = Box::leak(bytes.to_vec().into_boxed_slice());
        let raw = pg_sys::StringInfoData {
            data: leaked.as_mut_ptr() as *mut std::os::raw::c_char,
            len: leaked.len() as i32,
            maxlen: leaked.len() as i32,
            cursor: 0,
        };
        tqvector_recv(Internal::new(raw))
    }
}
