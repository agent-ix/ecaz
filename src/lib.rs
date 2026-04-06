use pgrx::extension_sql_file;
use pgrx::ffi::CString;
use pgrx::prelude::*;
use pgrx::{pg_sys, Internal};

pgrx::pg_module_magic!();

#[allow(dead_code)]
mod am;
mod quant;

use quant::prod::{payload_len, ProdQuantizer};

/// Public API surface for benchmarks and integration tests.
/// This stays narrow and explicit so benchmark and integration code can reuse
/// storage/quantizer helpers without reaching through internal modules directly.
pub mod bench_api {
    // Quantizer core
    pub use crate::quant::prod::{
        mse_code_len, pack_mse_indices, pack_qjl_signs, payload_len, qjl_code_len,
        unpack_mse_indices, unpack_qjl_signs, EncodedTq, PreparedQuery, ProdQuantizer,
    };

    // Hadamard
    pub use crate::quant::hadamard::{fwht_in_place, orthonormal_fwht_in_place};

    // Rotation
    pub use crate::quant::rotation::{inverse_srht, pad_input, sign_vector, srht, transform_dim};

    // Codebook
    pub use crate::quant::codebook::{beta_pdf, lloyd_max};

    // MSE
    pub use crate::quant::mse::{decode_indices, nearest_centroid_index, quantize_to_indices};

    // QJL
    pub use crate::quant::qjl::{decode_mse_only, qjl_project};

    // Page codec
    pub use crate::am::page::{
        neighbor_slots, neighbor_tuple_encoded_len, DataPage, DataPageChain, ItemPointer,
        MetadataPage, TqElementTuple, TqNeighborTuple, HEAPTID_INLINE_CAPACITY, ITEM_POINTER_BYTES,
        PAGE_HEADER_BYTES,
    };

    // Text I/O
    pub use crate::{format_text, parse_text, HEADER_BYTES, MIN_BINARY_BYTES};
}

extension_sql_file!("../sql/bootstrap.sql", name = "bootstrap", bootstrap);

/// Number of datum header bytes: dim(2) + bits(1) + seed(8).
pub const HEADER_BYTES: usize = 11;
/// Minimum valid wire payload: header plus gamma.
pub const MIN_BINARY_BYTES: usize = HEADER_BYTES + 4;

fn validate_bits(bits: u8) -> Result<(), String> {
    if !(2..=8).contains(&bits) {
        return Err(format!("bits must be between 2 and 8, got {bits}"));
    }
    Ok(())
}

fn code_len(dim: usize, bits: u8) -> usize {
    payload_len(dim, bits) - 4
}

pub fn parse_text(s: &str) -> Result<(u16, u8, u64, f32, Vec<u8>), String> {
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

pub fn format_text(dim: u16, bits: u8, seed: u64, gamma: f32, codes: &[u8]) -> String {
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

pub(crate) fn score_code_inner_product(
    dim: usize,
    bits: u8,
    seed: u64,
    code_a: &[u8],
    code_b: &[u8],
) -> f32 {
    let quantizer = ProdQuantizer::cached(dim, bits, seed);
    quantizer.score_ip_codes_lite(code_a, code_b)
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

    let prefix = unsafe { pg_sys::pq_getmsgbytes(msg, MIN_BINARY_BYTES as i32) as *const u8 };
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
        let codes = unsafe { pg_sys::pq_getmsgbytes(msg, code_bytes_len as i32) as *const u8 };
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

fn encode_embedding_to_tqvector(
    embedding: Vec<f32>,
    bits: i32,
    seed: i64,
) -> Result<Vec<u8>, String> {
    if embedding.is_empty() {
        return Err("embedding must not be empty".into());
    }
    let bits = u8::try_from(bits).map_err(|_| "bits must fit into u8".to_string())?;
    validate_bits(bits)?;
    let seed = seed as u64;
    let dim = u16::try_from(embedding.len()).map_err(|_| {
        format!(
            "embedding dimension {} exceeds maximum 65535",
            embedding.len()
        )
    })?;

    let quantizer = ProdQuantizer::cached(embedding.len(), bits, seed);
    let encoded = quantizer.encode(&embedding);

    let mut code_bytes = encoded.mse_packed;
    code_bytes.extend_from_slice(&encoded.qjl_packed);

    Ok(pack(dim, bits, seed, encoded.gamma, &code_bytes))
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn encode_to_tqvector(embedding: Vec<f32>, bits: i32, seed: i64) -> Vec<u8> {
    encode_embedding_to_tqvector(embedding, bits, seed).unwrap_or_else(|e| pgrx::error!("{e}"))
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

    score_code_inner_product(dim_a as usize, bits_a, seed_a, codes_a, codes_b)
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn tqvector_negative_inner_product(a: Vec<u8>, b: Vec<u8>) -> f32 {
    -tqvector_inner_product(a, b)
}

fn score_query_inner_product(candidate: &[u8], query: &[f32]) -> Result<f32, String> {
    let (dim, bits, seed, gamma, codes) =
        unpack(candidate).map_err(|e| format!("tqvector_query_inner_product(candidate): {e}"))?;
    if query.len() != dim as usize {
        return Err(format!(
            "tqvector/query dimension mismatch: candidate dim {}, query dim {}",
            dim,
            query.len()
        ));
    }

    let quantizer = ProdQuantizer::cached(dim as usize, bits, seed);
    let prepared = quantizer.prepare_ip_query(query);
    let mut payload = Vec::with_capacity(4 + codes.len());
    payload.extend_from_slice(&gamma.to_le_bytes());
    payload.extend_from_slice(codes);
    Ok(quantizer.score_ip_encoded(&prepared, &payload))
}

fn score_negative_query_inner_product(candidate: &[u8], query: &[f32]) -> Result<f32, String> {
    Ok(-score_query_inner_product(candidate, query)?)
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn tqvector_query_inner_product(candidate: Vec<u8>, query: Vec<f32>) -> f32 {
    score_query_inner_product(&candidate, &query).unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn tqvector_negative_query_inner_product(candidate: Vec<u8>, query: Vec<f32>) -> f32 {
    score_negative_query_inner_product(&candidate, &query).unwrap_or_else(|e| pgrx::error!("{e}"))
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
    fn test_score_code_inner_product_matches_quantizer_lite() {
        let mut rng = ChaCha8Rng::seed_from_u64(17);
        let a = (0..32)
            .map(|_| rng.gen_range(-1.0_f32..1.0_f32))
            .collect::<Vec<_>>();
        let b = (0..32)
            .map(|_| rng.gen_range(-1.0_f32..1.0_f32))
            .collect::<Vec<_>>();
        let quantizer = ProdQuantizer::cached(32, 4, 42);
        let enc_a = quantizer.encode(&a);
        let enc_b = quantizer.encode(&b);
        let mut code_a = enc_a.mse_packed;
        code_a.extend_from_slice(&enc_a.qjl_packed);
        let mut code_b = enc_b.mse_packed;
        code_b.extend_from_slice(&enc_b.qjl_packed);

        let expected = quantizer.score_ip_codes_lite(&code_a, &code_b);
        let observed = score_code_inner_product(32, 4, 42, &code_a, &code_b);
        assert_eq!(observed, expected);
    }

    #[test]
    fn test_encode_embedding_to_tqvector_rejects_dimensions_over_u16_max() {
        let oversized = vec![0.0_f32; usize::from(u16::MAX) + 1];
        let err = encode_embedding_to_tqvector(oversized, 4, 42).unwrap_err();
        assert!(
            err.contains("exceeds maximum 65535"),
            "oversized embeddings should fail with an explicit dimension limit error"
        );
    }

    #[test]
    fn test_tqvector_query_inner_product_uses_persisted_gamma() {
        let vector = vec![0.5_f32, -1.0, 0.25, 0.75, -0.5, 1.25, -0.75, 0.125];
        let query = vec![-0.25_f32, 0.75, 1.0, -0.5, 0.25, -1.0, 0.5, 0.875];
        let quantizer = ProdQuantizer::cached(vector.len(), 4, 42);
        let encoded = quantizer.encode(&vector);

        let mut code_bytes = encoded.mse_packed.clone();
        code_bytes.extend_from_slice(&encoded.qjl_packed);
        let candidate = pack(vector.len() as u16, 4, 42, encoded.gamma, &code_bytes);

        let prepared = quantizer.prepare_ip_query(&query);
        let expected = quantizer.score_ip_encoded(&prepared, &quantizer.pack_payload(&encoded));
        let observed = score_query_inner_product(&candidate, &query)
            .expect("query inner product should accept a valid packed candidate");

        assert!(
            (observed - expected).abs() < 1e-6,
            "query inner product should score against the full persisted payload"
        );
    }

    #[test]
    fn test_tqvector_query_inner_product_rejects_dimension_mismatch() {
        let vector = vec![0.5_f32, -1.0, 0.25, 0.75];
        let quantizer = ProdQuantizer::cached(vector.len(), 4, 42);
        let encoded = quantizer.encode(&vector);

        let mut code_bytes = encoded.mse_packed.clone();
        code_bytes.extend_from_slice(&encoded.qjl_packed);
        let candidate = pack(vector.len() as u16, 4, 42, encoded.gamma, &code_bytes);

        let err = score_query_inner_product(&candidate, &[1.0, 0.0, -0.5]).unwrap_err();
        assert!(
            err.contains("candidate dim 4, query dim 3"),
            "dimension mismatch should report both candidate and query dimensions"
        );
    }

    #[test]
    fn test_tqvector_negative_query_inner_product_negates_query_score() {
        let vector = vec![0.5_f32, -1.0, 0.25, 0.75, -0.5, 1.25, -0.75, 0.125];
        let query = vec![-0.25_f32, 0.75, 1.0, -0.5, 0.25, -1.0, 0.5, 0.875];
        let candidate = encode_embedding_to_tqvector(vector, 4, 42)
            .expect("candidate should encode successfully");

        let base = score_query_inner_product(&candidate, &query)
            .expect("base query score should succeed for matching dimensions");
        let negated = score_negative_query_inner_product(&candidate, &query)
            .expect("negative query score should succeed for matching dimensions");

        assert_eq!(negated, -base);
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

    fn setup_rescan_scaffold_index(name: &str) -> pg_sys::Oid {
        Spi::run(&format!(
            "CREATE TABLE {name} (id bigint primary key, embedding tqvector)"
        ))
        .expect("table creation should succeed");
        Spi::run(&format!(
            "INSERT INTO {name} VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))"
        ))
        .expect("seed insert should succeed");
        Spi::run(&format!(
            "CREATE INDEX {name}_idx ON {name} USING tqhnsw (embedding tqvector_ip_ops)"
        ))
        .expect("index creation should succeed");

        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{name}_idx'::regclass::oid"))
            .expect("SPI query should succeed")
            .expect("index oid should exist")
    }

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
        let amname =
            Spi::get_one::<String>("SELECT amname::text FROM pg_am WHERE amname = 'tqhnsw'")
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
    fn test_tqhnsw_planner_surface_stays_disabled() {
        Spi::run("CREATE TABLE tqhnsw_no_scan_plan (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_no_scan_plan VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_no_scan_plan_idx ON tqhnsw_no_scan_plan USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM tqhnsw_no_scan_plan \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
                     LIMIT 1",
                    None,
                    &[],
                )
                .expect("EXPLAIN should succeed")
                .first();
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<String>(1)
                        .expect("plan row should decode")
                        .expect("plan row should not be NULL"),
                );
            }
            lines.join("\n")
        });

        assert!(
            !plan.contains("Index Scan") && !plan.contains("Index Only Scan"),
            "planner should not use tqhnsw for scans before scan callbacks exist: {plan}"
        );
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

        let first_page =
            Spi::get_one::<Vec<u8>>("SELECT get_raw_page('tqhnsw_empty_build_idx', 0)::bytea")
                .expect("SPI query should succeed")
                .expect("raw index page should exist");

        let metadata = am::page::MetadataPage::decode_page(&first_page)
            .expect("metadata page should decode from raw relation bytes");

        assert_eq!(
            reloptions,
            vec!["m=12".to_string(), "ef_construction=80".to_string()]
        );
        assert_eq!(metadata.m, 12);
        assert_eq!(metadata.ef_construction, 80);
        assert_eq!(metadata.entry_point, am::page::ItemPointer::INVALID);
        assert_eq!(metadata.dimensions, 0);
        assert_eq!(metadata.bits, 0);
        assert_eq!(metadata.max_level, 0);
        assert_eq!(metadata.seed, 0);
    }

    #[pg_test]
    fn test_non_empty_index_build_writes_minimal_data_pages() {
        Spi::run("CREATE TABLE tqhnsw_nonempty_build (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_nonempty_build VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_nonempty_build_idx ON tqhnsw_nonempty_build USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 10, ef_construction = 90)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_nonempty_build_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        let (block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };

        assert!(
            block_count >= 2,
            "non-empty build should allocate a data page"
        );
        assert_eq!(metadata.m, 10);
        assert_eq!(metadata.ef_construction, 90);
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);
        assert!(metadata.max_level <= am::page::default_max_level_cap(metadata.m));

        let page_tuples = data_pages
            .iter()
            .flat_map(|page| {
                page.tuples.iter().enumerate().map(move |(idx, tuple)| {
                    (
                        am::page::ItemPointer {
                            block_number: page.block_number,
                            offset_number: (idx + 1) as u16,
                        },
                        tuple.as_slice(),
                    )
                })
            })
            .collect::<Vec<_>>();

        assert_eq!(
            page_tuples.len(),
            6,
            "each heap row should emit one neighbor and one element tuple"
        );

        let neighbor_tids = page_tuples
            .iter()
            .filter_map(|(tid, tuple)| {
                if tuple.first().copied() == Some(am::page::TQ_NEIGHBOR_TAG) {
                    Some(*tid)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let elements = page_tuples
            .iter()
            .filter_map(|(tid, tuple)| {
                if tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG) {
                    Some((
                        *tid,
                        am::page::TqElementTuple::decode(tuple, code_len(4, 4))
                            .expect("element tuple should decode"),
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        assert_eq!(neighbor_tids.len(), 3);
        assert_eq!(elements.len(), 3);
        assert!(
            page_tuples.iter().any(|(tid, tuple)| {
                *tid == metadata.entry_point
                    && tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG)
            }),
            "entry point should identify an element tuple"
        );

        let neighbor_map = page_tuples
            .iter()
            .filter_map(|(tid, tuple)| {
                if tuple.first().copied() == Some(am::page::TQ_NEIGHBOR_TAG) {
                    Some((
                        *tid,
                        am::page::TqNeighborTuple::decode(tuple)
                            .expect("neighbor tuple should decode"),
                    ))
                } else {
                    None
                }
            })
            .collect::<std::collections::HashMap<_, _>>();
        let element_ids = elements.iter().map(|(tid, _)| *tid).collect::<Vec<_>>();
        let entry_element = elements
            .iter()
            .find(|(tid, _)| *tid == metadata.entry_point)
            .expect("entry point should identify an element tuple");
        assert_eq!(entry_element.1.level, metadata.max_level);
        for (element_tid, element) in &elements {
            assert!(element.level <= metadata.max_level);
            assert!(!element.deleted);
            assert_eq!(element.heaptids.len(), 1);
            assert_ne!(element.heaptids[0], am::page::ItemPointer::INVALID);
            assert!(neighbor_tids.contains(&element.neighbortid));
            let neighbor = neighbor_map
                .get(&element.neighbortid)
                .expect("neighbor tuple should exist");
            assert_eq!(neighbor.count as usize, neighbor.tids.len());
            assert!(neighbor.tids.len() <= am::page::neighbor_slots(element.level, metadata.m));
            assert!(!neighbor.tids.contains(element_tid));
            assert!(neighbor.tids.iter().all(|tid| element_ids.contains(tid)));
        }
    }

    #[pg_test]
    fn test_non_empty_index_build_supports_raw_source_column() {
        Spi::run(
            "CREATE TABLE tqhnsw_source_build (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_source_build VALUES
             (1, ARRAY[1.0, 0.0, 0.5, -1.0], encode_to_tqvector(ARRAY[0.2, 0.1, 0.0, -0.2], 4, 42)),
             (2, ARRAY[0.9, 0.1, 0.4, -0.8], encode_to_tqvector(ARRAY[-0.1, 0.9, 0.2, -0.3], 4, 42)),
             (3, ARRAY[-1.0, 0.5, 0.0, 1.0], encode_to_tqvector(ARRAY[0.8, -0.4, 0.1, 0.7], 4, 42)),
             (4, ARRAY[-0.8, 0.4, 0.2, 0.9], encode_to_tqvector(ARRAY[-0.7, -0.2, 0.3, 0.6], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_source_build_idx ON tqhnsw_source_build USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let reloptions = Spi::get_one::<Vec<String>>(
            "SELECT reloptions FROM pg_class WHERE oid = 'tqhnsw_source_build_idx'::regclass",
        )
        .expect("SPI query should succeed")
        .expect("reloptions should exist");
        assert!(reloptions.contains(&"build_source_column=source".to_string()));

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_source_build_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.m, 6);
        assert_eq!(metadata.ef_construction, 80);
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);

        let elements = data_pages
            .iter()
            .flat_map(|page| {
                page.tuples
                    .iter()
                    .enumerate()
                    .filter_map(move |(idx, tuple)| {
                        if tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG) {
                            Some((
                                am::page::ItemPointer {
                                    block_number: page.block_number,
                                    offset_number: (idx + 1) as u16,
                                },
                                am::page::TqElementTuple::decode(tuple, code_len(4, 4))
                                    .expect("element tuple should decode"),
                            ))
                        } else {
                            None
                        }
                    })
            })
            .collect::<Vec<_>>();

        assert_eq!(elements.len(), 4);
        assert!(
            elements.iter().any(|(tid, _)| *tid == metadata.entry_point),
            "entry point should identify an element tuple"
        );
        assert!(elements
            .iter()
            .all(|(_, element)| element.heaptids.len() == 1));
    }

    #[pg_test]
    fn test_raw_source_build_coalesces_duplicate_vectors() {
        Spi::run(
            "CREATE TABLE tqhnsw_duplicate_source_build (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_duplicate_source_build VALUES
             (1, ARRAY[1.0, 0.0, 0.0, 0.0], encode_to_tqvector(ARRAY[0.5, 0.2, 0.1, 0.0], 4, 42)),
             (2, ARRAY[0.0, 1.0, 0.0, 0.0], encode_to_tqvector(ARRAY[0.5, 0.2, 0.1, 0.0], 4, 42)),
             (3, ARRAY[-1.0, 0.0, 0.0, 0.0], encode_to_tqvector(ARRAY[-0.6, -0.1, 0.0, 0.3], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_duplicate_source_build_idx ON tqhnsw_duplicate_source_build USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_duplicate_source_build_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);

        let elements = data_pages
            .iter()
            .flat_map(|page| page.tuples.iter())
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG))
            .map(|tuple| {
                am::page::TqElementTuple::decode(tuple, code_len(4, 4))
                    .expect("element tuple should decode")
            })
            .collect::<Vec<_>>();

        assert_eq!(
            elements.len(),
            2,
            "duplicate encoded vectors should share one element tuple"
        );
        let mut heaptid_counts = elements
            .iter()
            .map(|element| element.heaptids.len())
            .collect::<Vec<_>>();
        heaptid_counts.sort_unstable();
        assert_eq!(heaptid_counts, vec![1, 2]);
    }

    #[pg_test]
    #[should_panic(expected = "does not name a user column")]
    fn test_raw_source_rejects_missing_column() {
        Spi::run(
            "CREATE TABLE tqhnsw_bad_source_column (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_bad_source_column VALUES
             (1, ARRAY[1.0, 0.0, 0.0, 0.0], encode_to_tqvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_bad_source_column_idx ON tqhnsw_bad_source_column USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (build_source_column = 'missing')",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    #[should_panic(expected = "must be real[]")]
    fn test_raw_source_rejects_wrong_type() {
        Spi::run(
            "CREATE TABLE tqhnsw_bad_source_type (
                id bigint primary key,
                source double precision[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_bad_source_type VALUES
             (1, ARRAY[1.0, 0.0, 0.0, 0.0]::double precision[], encode_to_tqvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_bad_source_type_idx ON tqhnsw_bad_source_type USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (build_source_column = 'source')",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    #[should_panic(expected = "dimension mismatch")]
    fn test_raw_source_rejects_dimension_mismatch() {
        Spi::run(
            "CREATE TABLE tqhnsw_bad_source_dim (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_bad_source_dim VALUES
             (1, ARRAY[1.0, 0.0, 0.0], encode_to_tqvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_bad_source_dim_idx ON tqhnsw_bad_source_dim USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (build_source_column = 'source')",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    #[should_panic(expected = "does not support NULL tqhnsw build_source_column")]
    fn test_raw_source_rejects_null_value() {
        Spi::run(
            "CREATE TABLE tqhnsw_null_source (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_null_source VALUES
             (1, NULL, encode_to_tqvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_null_source_idx ON tqhnsw_null_source USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (build_source_column = 'source')",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    #[should_panic(expected = "does not support expression indexes yet")]
    fn test_raw_source_rejects_expression_index() {
        Spi::run(
            "CREATE TABLE tqhnsw_expression_source (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_expression_source VALUES
             (1, ARRAY[1.0, 0.0, 0.0, 0.0], encode_to_tqvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_expression_source_idx ON tqhnsw_expression_source USING tqhnsw \
             (((embedding::text)::tqvector) tqvector_ip_ops) WITH (build_source_column = 'source')",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    #[should_panic(expected = "does not support partial indexes yet")]
    fn test_raw_source_rejects_partial_index() {
        Spi::run(
            "CREATE TABLE tqhnsw_partial_source (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_partial_source VALUES
             (1, ARRAY[1.0, 0.0, 0.0, 0.0], encode_to_tqvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42)),
             (2, ARRAY[0.0, 1.0, 0.0, 0.0], encode_to_tqvector(ARRAY[0.0, 1.0, 0.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_partial_source_idx ON tqhnsw_partial_source USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (build_source_column = 'source') WHERE id > 1",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    fn test_non_empty_index_build_spans_multiple_data_pages() {
        Spi::run("CREATE TABLE tqhnsw_multipage_build (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");

        let payload_len = code_len(4, 8);
        for id in 1..=128 {
            let code = (0..payload_len)
                .map(|offset| ((id * 17 + offset as i32) & 0xff) as u8)
                .collect::<Vec<_>>();
            Spi::run(&format!(
                "INSERT INTO tqhnsw_multipage_build VALUES \
                 ({id}, '[dim=4,bits=8,seed=42,gamma=0.5]:{payload}'::tqvector)",
                payload = hex::encode(code),
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_multipage_build_idx ON tqhnsw_multipage_build USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 4, ef_construction = 64)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_multipage_build_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        let (block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert!(block_count > 2, "build should span more than one data page");
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 8);
        assert_eq!(metadata.seed, 42);
        assert!(metadata.max_level <= am::page::default_max_level_cap(metadata.m));

        let page_tuples = data_pages
            .iter()
            .flat_map(|page| {
                page.tuples.iter().enumerate().map(move |(idx, tuple)| {
                    (
                        am::page::ItemPointer {
                            block_number: page.block_number,
                            offset_number: (idx + 1) as u16,
                        },
                        tuple.as_slice(),
                    )
                })
            })
            .collect::<Vec<_>>();

        let elements = page_tuples
            .iter()
            .filter_map(|(tid, tuple)| {
                if tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG) {
                    Some((
                        *tid,
                        am::page::TqElementTuple::decode(tuple, code_len(4, 8))
                            .expect("element tuple should decode"),
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let neighbors = page_tuples
            .iter()
            .filter_map(|(tid, tuple)| {
                if tuple.first().copied() == Some(am::page::TQ_NEIGHBOR_TAG) {
                    Some((
                        *tid,
                        am::page::TqNeighborTuple::decode(tuple)
                            .expect("neighbor tuple should decode"),
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let element_tids = elements.iter().map(|(tid, _)| *tid).collect::<Vec<_>>();
        let covered_heap_tids = elements
            .iter()
            .map(|(_, element)| element.heaptids.len())
            .sum::<usize>();
        let entry_element = elements
            .iter()
            .find(|(tid, _)| *tid == metadata.entry_point)
            .expect("entry point should identify an element tuple");

        assert!(elements.len() > 1);
        assert_eq!(neighbors.len(), elements.len());
        assert_eq!(covered_heap_tids, 128);
        assert_eq!(entry_element.1.level, metadata.max_level);
        assert!(
            data_pages
                .iter()
                .filter(|page| !page.tuples.is_empty())
                .count()
                > 1,
            "more than one populated data page should exist"
        );

        let neighbor_map = neighbors
            .into_iter()
            .collect::<std::collections::HashMap<_, _>>();
        for (_, element) in &elements {
            let neighbor = neighbor_map
                .get(&element.neighbortid)
                .expect("neighbor tuple should exist");
            assert_eq!(neighbor.count as usize, neighbor.tids.len());
            assert!(neighbor.tids.len() <= am::page::neighbor_slots(element.level, metadata.m));
            assert!(neighbor.tids.iter().all(|tid| element_tids.contains(tid)));
        }
    }

    #[pg_test]
    fn test_non_empty_index_build_coalesces_duplicate_vectors() {
        Spi::run("CREATE TABLE tqhnsw_duplicate_build (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_duplicate_build VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, -2.0, -3.0, -4.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_duplicate_build_idx ON tqhnsw_duplicate_build USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_duplicate_build_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);

        let elements = data_pages
            .iter()
            .flat_map(|page| page.tuples.iter())
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG))
            .map(|tuple| {
                am::page::TqElementTuple::decode(tuple, code_len(4, 4))
                    .expect("element tuple should decode")
            })
            .collect::<Vec<_>>();

        assert_eq!(
            elements.len(),
            2,
            "duplicate encoded vectors should share one element tuple"
        );
        let mut heaptid_counts = elements
            .iter()
            .map(|element| element.heaptids.len())
            .collect::<Vec<_>>();
        heaptid_counts.sort_unstable();
        assert_eq!(heaptid_counts, vec![1, 2]);
    }

    #[pg_test]
    fn test_non_empty_index_build_keeps_gamma_distinct() {
        Spi::run(
            "CREATE TABLE tqhnsw_duplicate_build_gamma (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_duplicate_build_gamma VALUES
             (1, '[dim=4,bits=4,seed=42,gamma=0.5]:112233'::tqvector),
             (2, '[dim=4,bits=4,seed=42,gamma=1.5]:112233'::tqvector)",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_duplicate_build_gamma_idx ON tqhnsw_duplicate_build_gamma USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_duplicate_build_gamma_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);

        let elements = data_pages
            .iter()
            .flat_map(|page| page.tuples.iter())
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG))
            .map(|tuple| {
                am::page::TqElementTuple::decode(tuple, code_len(4, 4))
                    .expect("element tuple should decode")
            })
            .collect::<Vec<_>>();

        assert_eq!(
            elements.len(),
            2,
            "same-code build inputs with distinct persisted gamma values must not coalesce"
        );
        assert!(elements.iter().all(|element| element.heaptids.len() == 1));
        let mut gammas = elements
            .iter()
            .map(|element| element.gamma.to_bits())
            .collect::<Vec<_>>();
        gammas.sort_unstable();
        assert_eq!(
            gammas,
            vec![0.5_f32.to_bits(), 1.5_f32.to_bits()],
            "build should persist element gamma values alongside same-code distinct tuples"
        );
    }

    #[pg_test]
    fn test_tqhnsw_insert_appends_new_element_tuple() {
        Spi::run("CREATE TABLE tqhnsw_insert_append (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_append VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_append_idx ON tqhnsw_insert_append USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_append VALUES
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42))",
        )
        .expect("insert should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_insert_append_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);

        let elements = data_pages
            .iter()
            .flat_map(|page| page.tuples.iter())
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG))
            .map(|tuple| {
                am::page::TqElementTuple::decode(tuple, code_len(4, 4))
                    .expect("element tuple should decode")
            })
            .collect::<Vec<_>>();

        assert_eq!(elements.len(), 2);
        assert!(elements
            .iter()
            .all(|element| element.level == 0 || element.level <= metadata.max_level));
        assert!(elements.iter().any(|element| element.heaptids.len() == 1));
    }

    #[pg_test]
    fn test_tqhnsw_insert_reuses_tail_page_when_space_remains() {
        Spi::run(
            "CREATE TABLE tqhnsw_insert_tail_reuse (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_tail_reuse VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_tail_reuse_idx ON tqhnsw_insert_tail_reuse USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_insert_tail_reuse_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (before_block_count, _metadata, _data_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(
            before_block_count, 2,
            "seed build should fit on one data page"
        );

        Spi::run(
            "INSERT INTO tqhnsw_insert_tail_reuse VALUES
             (4, encode_to_tqvector(ARRAY[0.5, -0.5, 0.1, 0.2], 4, 42))",
        )
        .expect("insert should succeed");

        let (after_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(
            after_block_count, before_block_count,
            "insert should reuse existing tail page"
        );
        assert_eq!(metadata.seed, 42);

        let tuple_count = data_pages
            .iter()
            .map(|page| page.tuples.len())
            .sum::<usize>();
        assert_eq!(
            tuple_count, 8,
            "three build tuples plus one inserted tuple should store four pairs"
        );
    }

    #[pg_test]
    fn test_tqhnsw_insert_allocates_new_page_when_tail_is_full() {
        Spi::run("CREATE TABLE tqhnsw_insert_new_page (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");

        let large_dim = (1_u16..=u16::MAX)
            .rev()
            .find(|dim| {
                let code_len = code_len(*dim as usize, 4);
                if !am::page::element_tuple_fits_on_page(code_len, pg_sys::BLCKSZ as usize) {
                    return false;
                }

                let mut staged_page = am::page::DataPage::new(
                    am::page::FIRST_DATA_BLOCK_NUMBER,
                    pg_sys::BLCKSZ as usize,
                );
                let neighbor = am::page::TqNeighborTuple {
                    count: 0,
                    tids: Vec::new(),
                };
                let element = am::page::TqElementTuple {
                    level: 0,
                    deleted: false,
                    heaptids: vec![am::page::ItemPointer {
                        block_number: 1,
                        offset_number: 1,
                    }],
                    gamma: 0.5,
                    neighbortid: am::page::ItemPointer::INVALID,
                    code: vec![0x11_u8; code_len],
                };
                let required_bytes = am::page::raw_tuple_storage_bytes(
                    neighbor
                        .encode()
                        .expect("neighbor tuple should encode")
                        .len(),
                ) + am::page::raw_tuple_storage_bytes(
                    am::page::TqElementTuple::encoded_len(code_len),
                );
                staged_page.insert_neighbor(&neighbor).is_ok()
                    && staged_page.insert_element(&element).is_ok()
                    && staged_page.free_bytes() < required_bytes
            })
            .expect("should find a dimension that saturates one data page");
        let large_code_len = code_len(large_dim as usize, 4);
        let first_code = vec![0x11_u8; large_code_len];
        let second_code = vec![0x22_u8; large_code_len];
        Spi::run(&format!(
            "INSERT INTO tqhnsw_insert_new_page VALUES
             (1, '[dim={large_dim},bits=4,seed=42,gamma=0.5]:{}'::tqvector)",
            hex::encode(first_code),
        ))
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_new_page_idx ON tqhnsw_insert_new_page USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_insert_new_page_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (before_block_count, metadata, _data_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(
            before_block_count, 2,
            "seed build should occupy one data page"
        );
        assert_eq!(metadata.dimensions, large_dim);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);

        Spi::run(&format!(
            "INSERT INTO tqhnsw_insert_new_page VALUES
             (2, '[dim={large_dim},bits=4,seed=42,gamma=0.5]:{}'::tqvector)",
            hex::encode(second_code),
        ))
        .expect("insert should succeed");

        let (after_block_count, _metadata, data_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert!(
            after_block_count > before_block_count,
            "insert should allocate a new data page when the tail page is full"
        );
        assert_eq!(data_pages.len(), 2, "index should now have two data pages");
    }

    #[pg_test]
    fn test_tqhnsw_insert_reuses_new_tail_page_after_rollover() {
        Spi::run(
            "CREATE TABLE tqhnsw_insert_rollover_reuse (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");

        let (dim, pairs_per_page) = (1_u16..=u16::MAX)
            .rev()
            .find_map(|dim| {
                let code_len = code_len(dim as usize, 4);
                if !am::page::element_tuple_fits_on_page(code_len, pg_sys::BLCKSZ as usize) {
                    return None;
                }

                let mut staged_page = am::page::DataPage::new(
                    am::page::FIRST_DATA_BLOCK_NUMBER,
                    pg_sys::BLCKSZ as usize,
                );
                let mut pairs = 0_usize;
                loop {
                    let neighbor = am::page::TqNeighborTuple {
                        count: 0,
                        tids: Vec::new(),
                    };
                    let element = am::page::TqElementTuple {
                        level: 0,
                        deleted: false,
                        heaptids: vec![am::page::ItemPointer {
                            block_number: 1,
                            offset_number: 1,
                        }],
                        gamma: 0.5,
                        neighbortid: am::page::ItemPointer::INVALID,
                        code: vec![0x11_u8; code_len],
                    };
                    if staged_page.insert_neighbor(&neighbor).is_err()
                        || staged_page.insert_element(&element).is_err()
                    {
                        break;
                    }
                    pairs += 1;
                }

                if pairs >= 2 {
                    Some((dim, pairs))
                } else {
                    None
                }
            })
            .expect("should find a dimension where one page fits multiple pairs");
        let code_len = code_len(dim as usize, 4);

        Spi::run(
            "CREATE INDEX tqhnsw_insert_rollover_reuse_idx ON tqhnsw_insert_rollover_reuse USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        for id in 1..=pairs_per_page {
            Spi::run(&format!(
                "INSERT INTO tqhnsw_insert_rollover_reuse VALUES
                 ({id}, '[dim={dim},bits=4,seed=42,gamma=0.5]:{}'::tqvector)",
                hex::encode(vec![id as u8; code_len]),
            ))
            .expect("live insert should succeed");
        }

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_insert_rollover_reuse_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (before_block_count, metadata, before_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.dimensions, dim);
        assert!(
            !before_pages.is_empty(),
            "live inserts should create at least one data page"
        );

        Spi::run(&format!(
            "INSERT INTO tqhnsw_insert_rollover_reuse VALUES
             ({}, '[dim={dim},bits=4,seed=42,gamma=0.5]:{}'::tqvector)",
            pairs_per_page + 1,
            hex::encode(vec![0xaa_u8; code_len]),
        ))
        .expect("rollover insert should succeed");

        let (after_rollover_block_count, _metadata, after_rollover_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert!(
            after_rollover_block_count > before_block_count,
            "insert should allocate a new page once the original tail page is full"
        );
        assert_eq!(after_rollover_pages.len(), 2);

        Spi::run(&format!(
            "INSERT INTO tqhnsw_insert_rollover_reuse VALUES
             ({}, '[dim={dim},bits=4,seed=42,gamma=0.5]:{}'::tqvector)",
            pairs_per_page + 2,
            hex::encode(vec![0xbb_u8; code_len]),
        ))
        .expect("post-rollover insert should succeed");

        let (after_reuse_block_count, _metadata, after_reuse_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(
            after_reuse_block_count, after_rollover_block_count,
            "insert after rollover should reuse the new tail page when space remains"
        );
        assert_eq!(after_reuse_pages.len(), 2);
    }

    #[pg_test]
    fn test_tqhnsw_insert_coalesces_duplicate_vectors() {
        Spi::run(
            "CREATE TABLE tqhnsw_insert_duplicate (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_duplicate VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[-1.0, -2.0, -3.0, -4.0], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_duplicate_idx ON tqhnsw_insert_duplicate USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_insert_duplicate_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (before_block_count, metadata, data_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.seed, 42);
        let before_tuple_count = data_pages
            .iter()
            .map(|page| page.tuples.len())
            .sum::<usize>();

        Spi::run(
            "INSERT INTO tqhnsw_insert_duplicate VALUES
             (3, encode_to_tqvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42))",
        )
        .expect("duplicate insert should succeed");

        let (after_block_count, _metadata, data_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(
            after_block_count, before_block_count,
            "duplicate insert should not allocate a new block"
        );
        let after_tuple_count = data_pages
            .iter()
            .map(|page| page.tuples.len())
            .sum::<usize>();
        assert_eq!(
            after_tuple_count, before_tuple_count,
            "duplicate insert should not add a new tuple pair"
        );

        let elements = data_pages
            .iter()
            .flat_map(|page| page.tuples.iter())
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG))
            .map(|tuple| {
                am::page::TqElementTuple::decode(tuple, code_len(4, 4))
                    .expect("element tuple should decode")
            })
            .collect::<Vec<_>>();
        let mut heaptid_counts = elements
            .iter()
            .map(|element| element.heaptids.len())
            .collect::<Vec<_>>();
        heaptid_counts.sort_unstable();
        assert_eq!(heaptid_counts, vec![1, 2]);
    }

    #[pg_test]
    fn test_tqhnsw_insert_keeps_gamma_distinct() {
        Spi::run(
            "CREATE TABLE tqhnsw_insert_duplicate_gamma (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_duplicate_gamma VALUES
             (1, '[dim=4,bits=4,seed=42,gamma=0.5]:112233'::tqvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_duplicate_gamma_idx ON tqhnsw_insert_duplicate_gamma USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_insert_duplicate_gamma_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (before_block_count, _metadata, before_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        let before_tuple_count = before_pages
            .iter()
            .map(|page| page.tuples.len())
            .sum::<usize>();

        Spi::run(
            "INSERT INTO tqhnsw_insert_duplicate_gamma VALUES
             (2, '[dim=4,bits=4,seed=42,gamma=1.5]:112233'::tqvector)",
        )
        .expect("gamma-distinct insert should succeed");

        let (after_block_count, _metadata, data_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(
            after_block_count, before_block_count,
            "gamma-distinct same-code inserts should stay on the current tail page in this narrow test"
        );
        let after_tuple_count = data_pages
            .iter()
            .map(|page| page.tuples.len())
            .sum::<usize>();
        assert_eq!(
            after_tuple_count,
            before_tuple_count + 2,
            "gamma-distinct same-code inserts should append a fresh element/neighbor tuple pair"
        );

        let elements = data_pages
            .iter()
            .flat_map(|page| page.tuples.iter())
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG))
            .map(|tuple| {
                am::page::TqElementTuple::decode(tuple, code_len(4, 4))
                    .expect("element tuple should decode")
            })
            .collect::<Vec<_>>();

        assert_eq!(
            elements.len(),
            2,
            "same-code inserts with distinct persisted gamma values must not coalesce"
        );
        assert!(elements.iter().all(|element| element.heaptids.len() == 1));
        let mut gammas = elements
            .iter()
            .map(|element| element.gamma.to_bits())
            .collect::<Vec<_>>();
        gammas.sort_unstable();
        assert_eq!(
            gammas,
            vec![0.5_f32.to_bits(), 1.5_f32.to_bits()],
            "live insert should persist element gamma values alongside same-code distinct tuples"
        );
    }

    #[pg_test]
    #[should_panic(
        expected = "tqhnsw aminsert supports at most 10 duplicate heap tids per encoded vector"
    )]
    fn test_tqhnsw_insert_rejects_duplicate_heaptid_overflow() {
        Spi::run("CREATE TABLE tqhnsw_insert_duplicate_overflow (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_duplicate_overflow VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_duplicate_overflow_idx ON tqhnsw_insert_duplicate_overflow USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        for id in 2..=10 {
            Spi::run(&format!(
                "INSERT INTO tqhnsw_insert_duplicate_overflow VALUES
                 ({id}, encode_to_tqvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42))"
            ))
            .expect("duplicate insert should succeed until inline heap tid capacity is exhausted");
        }

        Spi::run(
            "INSERT INTO tqhnsw_insert_duplicate_overflow VALUES
             (11, encode_to_tqvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42))",
        )
        .expect("insert should fail once duplicate heap tid capacity is exhausted");
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw aminsert does not support build_source_column indexes yet")]
    fn test_tqhnsw_insert_rejects_build_source_column_index() {
        Spi::run(
            "CREATE TABLE tqhnsw_insert_source_reject (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_source_reject VALUES
             (1, ARRAY[1.0, 0.0, 0.5, -1.0], encode_to_tqvector(ARRAY[0.2, 0.1, 0.0, -0.2], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_source_reject_idx ON tqhnsw_insert_source_reject USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (build_source_column = 'source')",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_source_reject VALUES
             (2, ARRAY[0.9, 0.1, 0.4, -0.8], encode_to_tqvector(ARRAY[-0.1, 0.9, 0.2, -0.3], 4, 42))",
        )
        .expect("insert should fail");
    }

    #[pg_test]
    fn test_tqhnsw_empty_index_insert_initializes_shape_metadata() {
        Spi::run("CREATE TABLE tqhnsw_empty_insert (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_empty_insert_idx ON tqhnsw_empty_insert USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_empty_insert VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("insert should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_empty_insert_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);
        assert_eq!(metadata.max_level, 0);
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);

        let tuple_count = data_pages
            .iter()
            .map(|page| page.tuples.len())
            .sum::<usize>();
        assert_eq!(
            tuple_count, 2,
            "aminsert should append one neighbor and one element tuple"
        );
    }

    #[pg_test]
    fn test_tqhnsw_empty_index_reuses_initialized_metadata() {
        Spi::run(
            "CREATE TABLE tqhnsw_empty_insert_reuse (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_empty_insert_reuse_idx ON tqhnsw_empty_insert_reuse USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_empty_insert_reuse VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42))",
        )
        .expect("sequential inserts should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_empty_insert_reuse_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);
        assert_eq!(metadata.max_level, 0);
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);

        let element_count = data_pages
            .iter()
            .flat_map(|page| page.tuples.iter())
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG))
            .count();
        assert_eq!(
            element_count, 2,
            "second insert into an initially empty index should validate against persisted shape metadata"
        );
    }

    #[pg_test]
    fn test_tqhnsw_vacuum_callbacks_are_benign_noops() {
        Spi::run("CREATE TABLE tqhnsw_vacuum_noop (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_vacuum_noop VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.25, 0.75], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_vacuum_noop_idx ON tqhnsw_vacuum_noop USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("DELETE FROM tqhnsw_vacuum_noop WHERE id = 2").expect("delete should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_vacuum_noop_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (block_count, _metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let element_tuple_count = data_pages
            .iter()
            .flat_map(|page| page.tuples.iter())
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG))
            .count();

        let stats = unsafe { am::debug_vacuum_stats(index_oid) };
        assert_eq!(stats.num_pages, block_count);
        assert!(
            !stats.estimated_count,
            "vacuum stats should report exact tuple counts"
        );
        assert_eq!(stats.num_index_tuples, element_tuple_count as f64);
        assert_eq!(stats.tuples_removed, 0.0);
        assert_eq!(stats.pages_newly_deleted, 0);
        assert_eq!(stats.pages_deleted, 0);
        assert_eq!(stats.pages_free, 0);
    }

    #[pg_test]
    fn test_tqhnsw_vacuum_callbacks_handle_empty_index() {
        Spi::run("CREATE TABLE tqhnsw_vacuum_empty (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_vacuum_empty_idx ON tqhnsw_vacuum_empty USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_vacuum_empty_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (block_count, _metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(data_pages.len(), 0, "empty index should have no data pages");

        let stats = unsafe { am::debug_vacuum_stats(index_oid) };
        assert_eq!(stats.num_pages, block_count);
        assert!(
            !stats.estimated_count,
            "vacuum stats should report exact tuple counts"
        );
        assert_eq!(stats.num_index_tuples, 0.0);
        assert_eq!(stats.tuples_removed, 0.0);
        assert_eq!(stats.pages_newly_deleted, 0);
        assert_eq!(stats.pages_deleted, 0);
        assert_eq!(stats.pages_free, 0);
    }

    #[pg_test]
    fn test_tqhnsw_vacuum_callbacks_are_stable_across_repeated_calls() {
        Spi::run("CREATE TABLE tqhnsw_vacuum_repeat (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_vacuum_repeat VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.25, 0.75], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_vacuum_repeat_idx ON tqhnsw_vacuum_repeat USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("DELETE FROM tqhnsw_vacuum_repeat WHERE id = 2").expect("delete should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_vacuum_repeat_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let first_stats = unsafe { am::debug_vacuum_stats(index_oid) };
        let second_stats = unsafe { am::debug_vacuum_stats(index_oid) };

        assert_eq!(second_stats.num_pages, first_stats.num_pages);
        assert_eq!(second_stats.estimated_count, first_stats.estimated_count);
        assert_eq!(second_stats.num_index_tuples, first_stats.num_index_tuples);
        assert_eq!(second_stats.tuples_removed, first_stats.tuples_removed);
        assert_eq!(
            second_stats.pages_newly_deleted,
            first_stats.pages_newly_deleted
        );
        assert_eq!(second_stats.pages_deleted, first_stats.pages_deleted);
        assert_eq!(second_stats.pages_free, first_stats.pages_free);
    }

    #[pg_test]
    fn test_tqhnsw_scan_scaffold_allocates_and_frees_state() {
        Spi::run("CREATE TABLE tqhnsw_scan_scaffold (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_scan_scaffold VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_scan_scaffold_idx ON tqhnsw_scan_scaffold USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_scan_scaffold_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (has_opaque, cleared_opaque) = unsafe { am::debug_begin_end_scan(index_oid) };
        assert!(has_opaque, "ambeginscan should allocate scan opaque state");
        assert!(cleared_opaque, "amendscan should release scan opaque state");
    }

    #[pg_test]
    fn test_tqhnsw_scan_scaffold_amendscan_is_idempotent() {
        Spi::run(
            "CREATE TABLE tqhnsw_scan_scaffold_idempotent (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_scan_scaffold_idempotent VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_scan_scaffold_idempotent_idx ON tqhnsw_scan_scaffold_idempotent USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_scan_scaffold_idempotent_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (has_opaque, cleared_after_first, cleared_after_second) =
            unsafe { am::debug_end_scan_twice(index_oid) };
        assert!(has_opaque, "ambeginscan should allocate scan opaque state");
        assert!(
            cleared_after_first,
            "first amendscan call should clear scan opaque state"
        );
        assert!(
            cleared_after_second,
            "second amendscan call should remain a benign no-op"
        );
    }

    #[pg_test]
    fn test_tqhnsw_rescan_scaffold_records_query_dimensions() {
        let index_oid = setup_rescan_scaffold_index("tqhnsw_rescan_scaffold");
        let expected_query = vec![1.0, 0.0, 0.5, -1.0];
        let (
            rescan_called,
            query_dimensions,
            stored_query,
            scan_dimensions,
            scan_bits,
            scan_code_len,
            scan_block_count,
            has_prepared_query,
            prepared_lut_len,
            prepared_sq_len,
        ) = unsafe { am::debug_rescan_query_dimensions(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
        assert!(
            rescan_called,
            "amrescan should mark scan state as initialized"
        );
        assert_eq!(query_dimensions, 4);
        assert_eq!(stored_query, expected_query);
        assert_eq!(scan_dimensions, 4);
        assert_eq!(scan_bits, 4);
        assert_eq!(scan_code_len, code_len(4, 4));
        assert!(
            scan_block_count >= 2,
            "rescan should cache the current index block count"
        );
        assert!(
            has_prepared_query,
            "non-empty rescans should cache prepared query state for future ordered search"
        );
        assert_eq!(prepared_lut_len, 32);
        assert_eq!(prepared_sq_len, 4);
    }

    #[pg_test]
    fn test_tqhnsw_rescan_repeat_overwrites_query_dimensions() {
        Spi::run("CREATE TABLE tqhnsw_rescan_repeat (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_rescan_repeat_idx ON tqhnsw_rescan_repeat USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_rescan_repeat_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let second_query = vec![1.0, 0.0, 0.5];
        let (
            rescan_called,
            query_dimensions,
            stored_query,
            scan_dimensions,
            scan_bits,
            scan_code_len,
            scan_block_count,
            has_prepared_query,
            prepared_lut_len,
            prepared_sq_len,
        ) = unsafe {
            am::debug_rescan_overwrites_query_dimensions(
                index_oid,
                vec![1.0, 0.0, 0.5, -1.0],
                second_query.clone(),
            )
        };
        assert!(
            rescan_called,
            "repeated amrescan should keep scan state initialized"
        );
        assert_eq!(
            query_dimensions, 3,
            "second amrescan should overwrite recorded query dimensions"
        );
        assert_eq!(
            stored_query, second_query,
            "second amrescan should overwrite the stored query payload"
        );
        assert_eq!(scan_dimensions, 0);
        assert_eq!(scan_bits, 0);
        assert_eq!(scan_code_len, 0);
        assert_eq!(scan_block_count, 1);
        assert!(
            !has_prepared_query,
            "empty-index rescans should not allocate prepared query state yet"
        );
        assert_eq!(prepared_lut_len, 0);
        assert_eq!(prepared_sq_len, 0);
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw scan query dimension mismatch")]
    fn test_tqhnsw_rescan_scaffold_rejects_wrong_query_dimensions() {
        let index_oid = setup_rescan_scaffold_index("tqhnsw_rescan_scaffold_mismatch");
        let _ = unsafe { am::debug_rescan_query_dimensions(index_oid, vec![1.0, 0.0, 0.5]) };
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw scan query must not be NULL")]
    fn test_tqhnsw_rescan_scaffold_rejects_null_query() {
        let index_oid = setup_rescan_scaffold_index("tqhnsw_rescan_scaffold_null");
        unsafe { am::debug_rescan_null_query(index_oid) };
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw scan query must not be empty")]
    fn test_tqhnsw_rescan_scaffold_rejects_empty_query() {
        let index_oid = setup_rescan_scaffold_index("tqhnsw_rescan_scaffold_empty");
        let _ = unsafe { am::debug_rescan_query_dimensions(index_oid, vec![]) };
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw scan query dimension 65536 exceeds maximum 65535")]
    fn test_tqhnsw_rescan_scaffold_rejects_oversized_query() {
        Spi::run(
            "CREATE TABLE tqhnsw_rescan_scaffold_oversized (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_rescan_scaffold_oversized_idx ON tqhnsw_rescan_scaffold_oversized USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_rescan_scaffold_oversized_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let oversized_query = vec![0.0_f32; (u16::MAX as usize) + 1];
        let _ = unsafe { am::debug_rescan_query_dimensions(index_oid, oversized_query) };
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw scan does not support index quals yet")]
    fn test_tqhnsw_rescan_scaffold_rejects_index_quals() {
        let index_oid = setup_rescan_scaffold_index("tqhnsw_rescan_scaffold_quals");
        unsafe { am::debug_rescan_with_index_qual(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw scan currently requires exactly one ORDER BY query")]
    fn test_tqhnsw_rescan_scaffold_rejects_multiple_orderbys() {
        let index_oid = setup_rescan_scaffold_index("tqhnsw_rescan_scaffold_multi_orderby");
        unsafe { am::debug_rescan_with_multiple_orderbys(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw amgettuple requires amrescan before scan execution")]
    fn test_tqhnsw_gettuple_scaffold_requires_rescan() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_scaffold (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_gettuple_scaffold VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_scaffold_idx ON tqhnsw_gettuple_scaffold USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_gettuple_scaffold_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        unsafe { am::debug_gettuple_without_rescan(index_oid) };
    }

    #[pg_test]
    fn test_tqhnsw_gettuple_returns_heap_tids() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_exec_scaffold (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_gettuple_exec_scaffold VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_exec_scaffold_idx ON tqhnsw_gettuple_exec_scaffold USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_gettuple_exec_scaffold_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let observed_tids =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
        let expected_tids = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                        split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number
                     FROM tqhnsw_gettuple_exec_scaffold
                     ORDER BY id",
                    None,
                    &[],
                )
                .expect("ctid query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    (
                        u32::try_from(block_number).expect("block number should be non-negative"),
                        u16::try_from(offset_number).expect("offset number should be positive"),
                    )
                })
                .collect::<Vec<_>>()
        });

        let mut observed_tids = observed_tids;
        let mut expected_tids = expected_tids;
        observed_tids.sort_unstable();
        expected_tids.sort_unstable();
        assert_eq!(
            observed_tids, expected_tids,
            "amgettuple should return each indexed heap tid exactly once while visible scan order evolves away from pure linear order"
        );
    }

    #[pg_test]
    fn test_tqhnsw_gettuple_tracks_current_result_state() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_result_state (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_gettuple_result_state VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_result_state_idx ON tqhnsw_gettuple_result_state USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_gettuple_result_state_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let query = vec![1.0, 0.0, 0.5, -1.0];
        let (
            before_found,
            before_tid,
            before_score,
            before_score_value,
            found,
            after_tid,
            after_score,
            after_score_value,
        ) = unsafe { am::debug_gettuple_current_result_state(index_oid, query.clone()) };
        let expected_score = Spi::get_one::<f32>(&format!(
            "SELECT embedding <#> ARRAY[{},{},{},{}]::real[] \
             FROM tqhnsw_gettuple_result_state WHERE id = 1",
            query[0], query[1], query[2], query[3],
        ))
        .expect("score query should succeed")
        .expect("score should exist");

        assert!(
            !before_found,
            "current result state should be clear immediately after amrescan"
        );
        assert_eq!(before_tid, (u32::MAX, u16::MAX));
        assert!(
            !before_score,
            "bootstrap scan should not expose a score before tuple production"
        );
        assert_eq!(
            before_score_value, 0.0,
            "current result score should clear to zero before tuple production"
        );
        assert!(
            found,
            "first gettuple call should produce a tuple for a non-empty index"
        );
        assert_ne!(
            after_tid,
            (u32::MAX, u16::MAX),
            "first tuple production should record the current result element tid"
        );
        assert!(
            after_score,
            "first tuple production should mark the current result score as valid"
        );
        assert_eq!(
            after_score_value, expected_score,
            "current result score should match the operator-facing <#> value for the representative heap tuple"
        );
    }

    #[pg_test]
    fn test_tqhnsw_gettuple_current_result_lifecycle() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_result_lifecycle (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_gettuple_result_lifecycle VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_result_lifecycle_idx ON tqhnsw_gettuple_result_lifecycle USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_gettuple_result_lifecycle_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (
            first_tid,
            second_tid,
            second_score,
            second_score_value,
            exhausted_tid,
            exhausted_score,
            exhausted_score_value,
            rescanned_tid,
            rescanned_score,
        ) = unsafe {
            am::debug_gettuple_current_result_lifecycle(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert_eq!(
            first_tid, second_tid,
            "duplicate heap tid draining should stay attached to the same current result tuple"
        );
        assert!(
            second_score,
            "duplicate heap tid draining should keep the current result score valid"
        );
        assert_ne!(
            second_score_value, 0.0,
            "duplicate heap tid draining should keep the current result score populated"
        );
        assert_eq!(
            exhausted_tid,
            (u32::MAX, u16::MAX),
            "current result state should clear after full scan exhaustion"
        );
        assert!(
            !exhausted_score,
            "exhaustion should clear any current result score-valid bit"
        );
        assert_eq!(
            exhausted_score_value, 0.0,
            "exhaustion should clear the current result score value"
        );
        assert_eq!(
            rescanned_tid,
            (u32::MAX, u16::MAX),
            "amrescan should clear current result state before the next tuple is produced"
        );
        assert!(
            !rescanned_score,
            "amrescan should clear any current result score-valid bit"
        );
    }

    #[pg_test]
    fn test_tqhnsw_gettuple_current_result_tracks_heap_progress() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_result_heap_progress (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_gettuple_result_heap_progress VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_result_heap_progress_idx ON tqhnsw_gettuple_result_heap_progress USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_gettuple_result_heap_progress_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (element_tid, first_heap_tid, second_heap_tid, first_score, second_score) = unsafe {
            am::debug_gettuple_current_result_heap_progress(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert_ne!(
            element_tid,
            (u32::MAX, u16::MAX),
            "current result should keep a concrete element tid while draining duplicates"
        );
        assert_ne!(
            first_heap_tid,
            (u32::MAX, u16::MAX),
            "first produced tuple should attach a concrete heap tid to current result state"
        );
        assert_ne!(
            second_heap_tid,
            (u32::MAX, u16::MAX),
            "second produced tuple should keep a concrete heap tid attached to current result state"
        );
        assert_ne!(
            first_heap_tid, second_heap_tid,
            "duplicate draining should advance the current-result heap tid while staying on one element"
        );
        assert_eq!(
            first_score, second_score,
            "duplicate draining should keep the current-result score stable for one element"
        );
    }

    #[pg_test]
    fn test_tqhnsw_rescan_seeds_entry_candidate_state() {
        Spi::run(
            "CREATE TABLE tqhnsw_entry_candidate_state (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_entry_candidate_state VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_entry_candidate_state_idx ON tqhnsw_entry_candidate_state USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_entry_candidate_state_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (_block_count, metadata, _data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let (before_valid, before_tid, before_score, after_valid, after_tid, after_score) =
            unsafe { am::debug_rescan_entry_candidate_state(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert!(
            before_valid,
            "amrescan should seed an entry candidate for a non-empty index"
        );
        assert_eq!(
            before_tid,
            (
                metadata.entry_point.block_number,
                metadata.entry_point.offset_number
            ),
            "entry candidate should point at the persisted metadata entry point"
        );
        assert_ne!(
            before_score, 0.0,
            "entry candidate should carry a computed score for future ordered traversal"
        );
        assert!(
            !after_valid,
            "entry candidate should clear once the bootstrap scan fully exhausts"
        );
        assert_eq!(
            after_tid,
            (u32::MAX, u16::MAX),
            "exhaustion should clear the entry candidate tuple pointer"
        );
        assert_eq!(
            after_score, 0.0,
            "exhaustion should clear the entry candidate score"
        );
    }

    #[pg_test]
    fn test_tqhnsw_entry_candidate_persists_until_exhaustion() {
        Spi::run(
            "CREATE TABLE tqhnsw_entry_candidate_lifecycle (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_entry_candidate_lifecycle VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_entry_candidate_lifecycle_idx ON tqhnsw_entry_candidate_lifecycle USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_entry_candidate_lifecycle_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (
            before_valid,
            _before_tid,
            _before_score,
            partial_valid,
            partial_tid,
            partial_score,
            partial_result_tid,
            exhausted_valid,
            exhausted_tid,
            exhausted_score,
        ) = unsafe { am::debug_entry_candidate_lifecycle(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert!(
            before_valid,
            "entry candidate should be seeded before tuple production"
        );
        assert!(
            partial_valid || partial_result_tid != (u32::MAX, u16::MAX),
            "partial scan progress should keep either a remaining frontier candidate or a concrete current result"
        );
        if partial_valid {
            assert_ne!(
                partial_tid,
                (u32::MAX, u16::MAX),
                "partial scan progress should keep a concrete frontier candidate tid when one remains"
            );
            assert_ne!(
                partial_score, 0.0,
                "partial scan progress should keep a concrete frontier candidate score when one remains"
            );
        } else {
            assert_ne!(
                partial_result_tid,
                (u32::MAX, u16::MAX),
                "when the frontier head materializes immediately, partial scan progress should keep a concrete current-result tid"
            );
        }
        assert!(
            !exhausted_valid,
            "entry candidate should clear once the bootstrap scan fully exhausts"
        );
        assert_eq!(
            exhausted_tid,
            (u32::MAX, u16::MAX),
            "entry candidate tid should clear on exhaustion"
        );
        assert_eq!(
            exhausted_score, 0.0,
            "entry candidate score should clear on exhaustion"
        );
    }

    #[pg_test]
    fn test_tqhnsw_successor_candidate_from_entry_adjacency() {
        Spi::run(
            "CREATE TABLE tqhnsw_successor_candidate_state (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_successor_candidate_state VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_successor_candidate_state_idx ON tqhnsw_successor_candidate_state USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_successor_candidate_state_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (entry_tid, entry_neighbors, successor_valid, successor_tid, successor_score) = unsafe {
            am::debug_rescan_successor_candidate_state(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert_ne!(
            entry_tid,
            (u32::MAX, u16::MAX),
            "non-empty index should expose a concrete entry point"
        );
        if entry_neighbors.is_empty() {
            assert!(
                !successor_valid,
                "successor candidate should stay empty when the persisted entry adjacency is empty"
            );
            assert_eq!(
                successor_tid,
                (u32::MAX, u16::MAX),
                "empty successor candidate should clear its tuple pointer"
            );
            assert_eq!(
                successor_score, 0.0,
                "empty successor candidate should clear its score"
            );
        } else {
            assert!(
                successor_valid,
                "successor candidate should seed from persisted entry adjacency when a live neighbor exists"
            );
            assert!(
                entry_neighbors.contains(&successor_tid),
                "successor candidate should target one of the persisted entry-point neighbor refs"
            );
            assert_ne!(
                successor_score, 0.0,
                "seeded successor candidate should carry a computed score"
            );
        }
    }

    #[pg_test]
    fn test_tqhnsw_rescan_builds_bootstrap_candidate_frontier() {
        Spi::run(
            "CREATE TABLE tqhnsw_candidate_frontier_state (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_candidate_frontier_state VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_candidate_frontier_state_idx ON tqhnsw_candidate_frontier_state USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_candidate_frontier_state_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (head, frontier, frontier_slots, frontier_provenance, expanded_sources) =
            unsafe { am::debug_rescan_candidate_frontier(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
        let valid_entry_neighbors = unsafe { am::debug_entry_point_neighbor_tids(index_oid) }
            .into_iter()
            .filter(|tid| *tid != (u32::MAX, u16::MAX))
            .collect::<Vec<_>>();

        assert!(
            frontier[0].0,
            "first frontier slot should hold the seeded entry candidate"
        );
        assert_ne!(
            frontier[0].1,
            (u32::MAX, u16::MAX),
            "first frontier slot should expose a concrete element tid"
        );
        assert_ne!(
            frontier[0].2, 0.0,
            "first frontier slot should carry a computed score"
        );

        if frontier[1].0 {
            assert_ne!(
                frontier[1].1,
                (u32::MAX, u16::MAX),
                "second frontier slot should expose a concrete element tid when present"
            );
            assert_ne!(
                frontier[1].2, 0.0,
                "second frontier slot should carry a computed score when present"
            );
        } else {
            assert_eq!(
                frontier[1].1,
                (u32::MAX, u16::MAX),
                "empty second frontier slot should clear its element tid"
            );
            assert_eq!(
                frontier[1].2, 0.0,
                "empty second frontier slot should clear its score"
            );
        }

        assert_eq!(
            frontier_slots.first().map(|slot| slot.1),
            Some(frontier_provenance[0].1),
            "frontier and provenance views should agree on the seeded entry candidate tid"
        );
        assert!(
            !frontier_slots.is_empty(),
            "bootstrap frontier should always contain the seeded entry candidate"
        );
        assert!(
            frontier_slots.len() <= 3,
            "bootstrap frontier should stay within the current bounded traversal width"
        );
        assert!(
            frontier_slots.len() > valid_entry_neighbors.len().min(1),
            "bootstrap frontier should at least seed the entry candidate and any immediately discoverable live neighbor"
        );

        let expected_head = frontier_slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.0)
            .min_by(|(left_index, left), (right_index, right)| {
                left.2.total_cmp(&right.2).then(left_index.cmp(right_index))
            })
            .map(|(_, slot)| slot.1);
        assert_eq!(
            head, expected_head,
            "frontier head should pick the best valid candidate across the current widened bootstrap frontier"
        );

        let entry_tid = frontier_provenance
            .first()
            .map(|slot| slot.1)
            .expect("frontier provenance should include the seeded entry candidate");
        assert_eq!(
            frontier_provenance[0].2,
            (u32::MAX, u16::MAX),
            "the seeded entry candidate should not claim discovery from another element"
        );
        for slot in frontier_provenance.iter().skip(1) {
            if slot.0 {
                assert!(
                    slot.2 == entry_tid
                        || frontier_provenance
                            .iter()
                            .any(|other| other.0 && other.1 == slot.2),
                    "bootstrap successor candidates should record either the entry candidate or another seeded candidate as their discovery source"
                );
            }
        }

        let seeded_candidate_tids = frontier_provenance
            .iter()
            .filter_map(|slot| slot.0.then_some(slot.1))
            .collect::<Vec<_>>();
        assert!(
            expanded_sources.contains(&entry_tid),
            "bootstrap expanded-source state should always include the entry candidate"
        );
        assert!(
            expanded_sources
                .iter()
                .all(|source_tid| seeded_candidate_tids.contains(source_tid)),
            "bootstrap expanded-source state should only contain seeded candidate tids"
        );
    }

    #[pg_test]
    fn test_tqhnsw_frontier_head_persists_until_exhaustion() {
        Spi::run(
            "CREATE TABLE tqhnsw_frontier_head_lifecycle (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_frontier_head_lifecycle VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_frontier_head_lifecycle_idx ON tqhnsw_frontier_head_lifecycle USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_frontier_head_lifecycle_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (
            before_head,
            before_frontier,
            partial_head,
            partial_frontier,
            exhausted_head,
            exhausted_frontier,
        ) = unsafe {
            am::debug_candidate_frontier_head_lifecycle(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert_ne!(
            before_head, None,
            "non-empty frontier should expose a concrete head immediately after rescan"
        );
        assert!(
            partial_head != before_head || partial_frontier != before_frontier,
            "partial bootstrap scan progress should now be allowed to advance bootstrap frontier state"
        );
        assert_eq!(
            partial_head.is_some(),
            partial_frontier.iter().any(|slot| slot.0),
            "frontier head presence should still track whether any bootstrap candidates remain after partial progress"
        );
        assert_eq!(
            exhausted_head, None,
            "frontier head should clear on full scan exhaustion"
        );
        assert_eq!(
            exhausted_frontier,
            [
                (false, (u32::MAX, u16::MAX), 0.0),
                (false, (u32::MAX, u16::MAX), 0.0),
            ],
            "full scan exhaustion should clear both frontier slots"
        );
    }

    #[pg_test]
    fn test_tqhnsw_consume_candidate_frontier_head_reselects_or_clears() {
        Spi::run(
            "CREATE TABLE tqhnsw_frontier_head_consume (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_frontier_head_consume VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_frontier_head_consume_idx ON tqhnsw_frontier_head_consume USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_frontier_head_consume_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (
            before_head,
            before_frontier,
            after_first_head,
            after_first_frontier,
            after_second_head,
            after_second_frontier,
        ) = unsafe {
            am::debug_consume_candidate_frontier_head(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert_ne!(
            before_head, None,
            "non-empty frontier should expose a concrete head before consumption"
        );
        let consumed_tid = before_head.expect("non-empty frontier should expose a head");
        let remaining_slot = before_frontier
            .iter()
            .position(|slot| slot.0 && slot.1 != consumed_tid)
            .unwrap_or(0);

        if before_frontier[remaining_slot].0 {
            assert_eq!(
                after_first_head,
                Some(before_frontier[remaining_slot].1),
                "when another candidate remains valid, consuming the head should expose that remaining candidate as the new head"
            );
            assert_eq!(
                after_first_frontier[0],
                before_frontier[remaining_slot],
                "consuming the current head should preserve the remaining candidate after compaction"
            );
            assert_eq!(
                after_first_frontier[1],
                (false, (u32::MAX, u16::MAX), 0.0),
                "the compacted candidate vector should leave no second frontier slot populated yet"
            );
        } else {
            assert_eq!(
                after_first_head, None,
                "consuming the only valid candidate should invalidate the frontier head"
            );
            assert_eq!(
                after_first_frontier,
                [
                    (false, (u32::MAX, u16::MAX), 0.0),
                    (false, (u32::MAX, u16::MAX), 0.0),
                ],
                "consuming the only valid candidate should leave the compacted frontier empty"
            );
        }

        assert_eq!(
            after_second_head, None,
            "consuming the frontier head again should leave the frontier empty"
        );
        assert_eq!(
            after_second_frontier,
            [
                (false, (u32::MAX, u16::MAX), 0.0),
                (false, (u32::MAX, u16::MAX), 0.0),
            ],
            "after consuming both available slots, the two-slot frontier should be fully cleared"
        );
    }

    #[pg_test]
    fn test_tqhnsw_frontier_head_refills_from_consumed_neighbors() {
        Spi::run(
            "CREATE TABLE tqhnsw_frontier_head_refill (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_frontier_head_refill VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42)),
             (4, encode_to_tqvector(ARRAY[0.5, -1.0, 1.0, 0.0], 4, 42)),
             (5, encode_to_tqvector(ARRAY[-0.5, 0.5, 1.0, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_frontier_head_refill_idx ON tqhnsw_frontier_head_refill USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_frontier_head_refill_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let valid_entry_neighbors = unsafe { am::debug_entry_point_neighbor_tids(index_oid) }
            .into_iter()
            .filter(|tid| *tid != (u32::MAX, u16::MAX))
            .collect::<Vec<_>>();
        let (
            before_head,
            before_slots,
            consumed_tid,
            consumed_neighbors,
            after_head,
            after_slots,
            after_provenance_slots,
        ) = unsafe {
            am::debug_consume_candidate_frontier_head_slots(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert_eq!(
            before_slots.len(),
            1 + valid_entry_neighbors.len().min(2),
            "bootstrap frontier should seed the entry candidate plus up to two live entry neighbors before head consumption"
        );
        let before_tids = before_slots
            .iter()
            .map(|slot| slot.1)
            .collect::<std::collections::BTreeSet<_>>();
        let after_tids = after_slots
            .iter()
            .map(|slot| slot.1)
            .collect::<std::collections::BTreeSet<_>>();
        let new_tids = after_tids
            .difference(&before_tids)
            .copied()
            .collect::<Vec<_>>();
        let has_unseen_consumed_neighbor = consumed_neighbors
            .iter()
            .any(|neighbor_tid| !before_tids.contains(neighbor_tid));

        assert_ne!(
            consumed_tid,
            (u32::MAX, u16::MAX),
            "non-empty frontier should expose an actually consumed candidate"
        );
        assert!(
            !after_tids.contains(&consumed_tid),
            "consuming the head should remove that candidate from the frontier"
        );
        assert!(
            after_slots.len() >= before_slots.len().saturating_sub(1),
            "consuming one candidate should not drop more than the consumed slot"
        );
        assert!(
            after_slots.len() <= before_slots.len(),
            "bootstrap refill should stay within the current bounded frontier width"
        );
        assert_eq!(
            after_head.is_some(),
            !after_slots.is_empty(),
            "frontier head presence should track whether any candidates remain after consume/refill"
        );
        assert!(
            before_head.is_some(),
            "non-empty frontier should expose a head before consume/refill"
        );

        if has_unseen_consumed_neighbor {
            assert_eq!(
                after_slots.len(),
                before_slots.len(),
                "when the consumed candidate exposes an unseen neighbor, refill should restore frontier width"
            );
            assert_eq!(
                new_tids.len(),
                1,
                "refill should introduce exactly one newly seeded frontier candidate after one consume"
            );
            assert!(
                consumed_neighbors.contains(&new_tids[0]),
                "refill should draw the newly added candidate from the consumed candidate's adjacency"
            );
            let new_slot = after_provenance_slots
                .iter()
                .find(|slot| slot.0 && slot.1 == new_tids[0])
                .expect(
                    "newly seeded candidate should remain present in the frontier provenance view",
                );
            assert!(
                new_slot.2 == consumed_tid
                    || (new_slot.2 != (u32::MAX, u16::MAX) && before_tids.contains(&new_slot.2)),
                "a candidate discovered during refill should record either the consumed frontier head or another still-seeded frontier source"
            );
        } else {
            if after_slots.len() == before_slots.len() {
                assert_eq!(
                    new_tids.len(),
                    1,
                    "bounded refill should add exactly one replacement candidate when another remaining frontier candidate can expand"
                );
                let new_slot = after_provenance_slots
                    .iter()
                    .find(|slot| slot.0 && slot.1 == new_tids[0])
                    .expect(
                        "newly seeded candidate should remain present in the frontier provenance view",
                    );
                assert_ne!(
                    new_slot.2, consumed_tid,
                    "when the consumed adjacency contributes nothing unseen, any replacement should come from another remaining frontier candidate"
                );
                assert!(
                    before_tids.contains(&new_slot.2) || after_tids.contains(&new_slot.2),
                    "replacement candidates should record a still-seeded frontier source"
                );
            } else {
                assert_eq!(
                    after_slots.len(),
                    before_slots.len() - 1,
                    "without any expandable remaining frontier candidate, the frontier should shrink by the consumed slot"
                );
                assert!(
                    new_tids.is_empty(),
                    "refill should not fabricate a new candidate when no eligible expansion source exists"
                );
            }
        }
    }

    #[pg_test]
    fn test_tqhnsw_gettuple_consumes_bootstrap_candidate_state() {
        Spi::run(
            "CREATE TABLE tqhnsw_bootstrap_consume_state (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_bootstrap_consume_state VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42)),
             (4, encode_to_tqvector(ARRAY[0.5, -1.0, 1.0, 0.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_bootstrap_consume_state_idx ON tqhnsw_bootstrap_consume_state USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_bootstrap_consume_state_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (
            before_head,
            before_slots,
            active_candidate,
            current_result_tid,
            after_head,
            after_slots,
        ) = unsafe {
            am::debug_gettuple_consumes_bootstrap_candidate(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert!(
            !active_candidate.0,
            "first amgettuple call should now materialize the consumed bootstrap candidate instead of leaving it active"
        );
        let consumed_slot = before_slots
            .into_iter()
            .find(|slot| slot.1 == current_result_tid)
            .expect("first amgettuple call should materialize one of the visible bootstrap frontier slots");
        assert_eq!(
            current_result_tid, consumed_slot.1,
            "first amgettuple call should attach current-result state to the consumed bootstrap candidate"
        );
        assert!(
            !after_slots.iter().any(|slot| slot.1 == consumed_slot.1),
            "consuming the bootstrap head should remove that candidate from the frontier"
        );
        assert_eq!(
            after_head.is_some(),
            !after_slots.is_empty(),
            "frontier-head presence should continue to track whether bootstrap candidates remain after first consumption"
        );
        assert!(
            before_head.is_some(),
            "non-empty bootstrap frontier should expose a head before first consumption"
        );
    }

    #[pg_test]
    fn test_tqhnsw_active_candidate_materializes_into_pending_drain() {
        Spi::run(
            "CREATE TABLE tqhnsw_active_candidate_materialize (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_active_candidate_materialize VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_active_candidate_materialize_idx ON tqhnsw_active_candidate_materialize USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_active_candidate_materialize_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (active_before, materialized, current_result_tid, pending_heap_tids, active_cleared) =
            unsafe {
                am::debug_materialize_active_candidate_result(
                    index_oid,
                    vec![1.0, 0.0, 0.5, -1.0],
                )
            };

        assert!(
            active_before.0,
            "bootstrap candidate consumption should produce an active candidate before materialization"
        );
        assert!(
            materialized,
            "active candidate should materialize into the pending heap-tid drain path"
        );
        assert_eq!(
            current_result_tid, active_before.1,
            "materializing the active candidate should attach current-result state to that candidate"
        );
        assert_eq!(
            pending_heap_tids.len(),
            2,
            "duplicate-coalesced active candidates should populate all duplicate heap tids into pending drain state"
        );
        assert!(
            active_cleared,
            "active candidate state should clear once it has been materialized into pending drain state"
        );
    }

    #[pg_test]
    fn test_tqhnsw_visited_seed_state_tracks_frontier_candidates() {
        Spi::run(
            "CREATE TABLE tqhnsw_visited_seed_state (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_visited_seed_state VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_visited_seed_state_idx ON tqhnsw_visited_seed_state USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_visited_seed_state_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        let (head, _frontier, frontier_slots, _frontier_provenance, _expanded_sources) =
            unsafe { am::debug_rescan_candidate_frontier(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
        let (before, partial, exhausted) =
            unsafe { am::debug_visited_seed_lifecycle(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        let mut expected = frontier_slots
            .into_iter()
            .filter_map(|(valid, tid, _)| valid.then_some(tid))
            .collect::<Vec<_>>();
        expected.sort_unstable();

        assert_ne!(
            head, None,
            "non-empty scan frontier should still expose at least one seeded candidate"
        );
        assert_eq!(
            before, expected,
            "visited state should seed from the currently valid frontier candidate tids"
        );
        assert_eq!(
            partial, before,
            "bootstrap linear scan progress should not mutate seeded visited state yet"
        );
        assert_eq!(
            exhausted, before,
            "visited state should remain stable through bootstrap scan exhaustion until rescan"
        );
    }

    #[pg_test]
    fn test_tqhnsw_gettuple_current_result_exposes_neighbor_refs() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_current_neighbors (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_gettuple_current_neighbors VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_current_neighbors_idx ON tqhnsw_gettuple_current_neighbors USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_gettuple_current_neighbors_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (current_result_tid, neighbor_count) = unsafe {
            am::debug_gettuple_current_result_neighbors(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };
        let (_block_count, metadata, _data_pages) = unsafe { am::debug_index_pages(index_oid) };

        assert_ne!(
            current_result_tid,
            (u32::MAX, u16::MAX),
            "neighbor debug helper should attach to a concrete current result tuple"
        );
        assert!(
            neighbor_count <= am::page::neighbor_slots(0, metadata.m),
            "current-result neighbor count should decode within level-0 neighbor capacity"
        );
    }

    #[pg_test]
    fn test_tqhnsw_entry_point_neighbor_refs_point_to_elements() {
        Spi::run(
            "CREATE TABLE tqhnsw_entry_point_neighbors (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_entry_point_neighbors VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_entry_point_neighbors_idx ON tqhnsw_entry_point_neighbors USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_entry_point_neighbors_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let neighbor_tids = unsafe { am::debug_entry_point_neighbor_tids(index_oid) };
        let (_block_count, _metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let element_tids = data_pages
            .iter()
            .flat_map(|page| {
                page.tuples.iter().enumerate().filter_map(|(idx, tuple)| {
                    (tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG)).then_some((
                        page.block_number,
                        u16::try_from(idx + 1).expect("page tuple offset should fit in u16"),
                    ))
                })
            })
            .collect::<std::collections::HashSet<_>>();

        for neighbor_tid in neighbor_tids {
            assert!(
                element_tids.contains(&neighbor_tid),
                "entry-point neighbor ref should target an element tuple"
            );
        }
    }

    #[pg_test]
    fn test_tqhnsw_gettuple_returns_all_duplicate_heap_tids() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_duplicate_exec (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_gettuple_duplicate_exec VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_duplicate_exec_idx ON tqhnsw_gettuple_duplicate_exec USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_gettuple_duplicate_exec_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let observed_tids =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
        let expected_tids = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                        split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number
                     FROM tqhnsw_gettuple_duplicate_exec
                     ORDER BY id",
                    None,
                    &[],
                )
                .expect("ctid query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    (
                        u32::try_from(block_number).expect("block number should be non-negative"),
                        u16::try_from(offset_number).expect("offset number should be positive"),
                    )
                })
                .collect::<Vec<_>>()
        });

        let mut observed_tids = observed_tids;
        let mut expected_tids = expected_tids;
        observed_tids.sort_unstable();
        expected_tids.sort_unstable();
        assert_eq!(
            observed_tids, expected_tids,
            "amgettuple should emit every heap tid stored in a duplicate-coalesced element tuple exactly once"
        );
    }

    #[pg_test]
    fn test_tqhnsw_gettuple_exhaustion_stays_false() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_exhaustion (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_gettuple_exhaustion VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_exhaustion_idx ON tqhnsw_gettuple_exhaustion USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_gettuple_exhaustion_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (observed_tids, exhausted_once, exhausted_twice) =
            unsafe { am::debug_gettuple_exhaustion_state(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        let expected_tids = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                        split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number
                     FROM tqhnsw_gettuple_exhaustion
                     ORDER BY id",
                    None,
                    &[],
                )
                .expect("ctid query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    (
                        u32::try_from(block_number).expect("block number should be non-negative"),
                        u16::try_from(offset_number).expect("offset number should be positive"),
                    )
                })
                .collect::<Vec<_>>()
        });

        let mut observed_tids = observed_tids;
        let mut expected_tids = expected_tids;
        observed_tids.sort_unstable();
        expected_tids.sort_unstable();
        assert_eq!(
            observed_tids, expected_tids,
            "scan exhaustion should occur only after returning every heap tid exactly once"
        );
        assert!(
            !exhausted_once,
            "first amgettuple call after exhausting the scan should return false"
        );
        assert!(
            !exhausted_twice,
            "repeated amgettuple calls after exhaustion should remain false"
        );
    }

    #[pg_test]
    fn test_tqhnsw_gettuple_rescan_after_exhaustion_restarts_scan() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_exhaustion_rescan (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_gettuple_exhaustion_rescan VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_exhaustion_rescan_idx ON tqhnsw_gettuple_exhaustion_rescan USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_gettuple_exhaustion_rescan_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (first_pass, rescanned_tids) = unsafe {
            am::debug_gettuple_rescan_after_exhaustion(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        let expected_tids = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                        split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number
                     FROM tqhnsw_gettuple_exhaustion_rescan
                     ORDER BY id",
                    None,
                    &[],
                )
                .expect("ctid query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    (
                        u32::try_from(block_number).expect("block number should be non-negative"),
                        u16::try_from(offset_number).expect("offset number should be positive"),
                    )
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(
            first_pass, expected_tids,
            "the first linear scan should exhaust only after returning every heap tid"
        );
        assert_eq!(
            rescanned_tids, expected_tids,
            "amrescan after exhaustion should restart tuple production from the beginning"
        );
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw amgettuple only supports forward scan direction")]
    fn test_tqhnsw_gettuple_rejects_backward_scan_direction() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_backward_scan (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_gettuple_backward_scan VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_backward_scan_idx ON tqhnsw_gettuple_backward_scan USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_gettuple_backward_scan_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        unsafe { am::debug_gettuple_backward_after_rescan(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
    }

    #[pg_test]
    fn test_tqhnsw_gettuple_rescan_resets_duplicate_progress() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_duplicate_rescan (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_gettuple_duplicate_rescan VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_duplicate_rescan_idx ON tqhnsw_gettuple_duplicate_rescan USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_gettuple_duplicate_rescan_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (first_tid, rescanned_tids) = unsafe {
            am::debug_gettuple_rescan_after_partial(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        let expected_tids = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                        split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number
                     FROM tqhnsw_gettuple_duplicate_rescan
                     ORDER BY id",
                    None,
                    &[],
                )
                .expect("ctid query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    (
                        u32::try_from(block_number).expect("block number should be non-negative"),
                        u16::try_from(offset_number).expect("offset number should be positive"),
                    )
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(
            first_tid, expected_tids[0],
            "the partial scan should consume the first duplicate heap tid before rescan"
        );
        assert_eq!(
            rescanned_tids, expected_tids,
            "amrescan should reset duplicate heap-tid progress back to the start of the linear scan"
        );
    }

    #[pg_test]
    fn test_tqhnsw_gettuple_duplicate_scan_spans_multiple_pages() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_duplicate_multipage (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");

        let payload_len = code_len(4, 8);
        let duplicate_payload = vec![0x11_u8; payload_len];
        for id in 1..=10 {
            Spi::run(&format!(
                "INSERT INTO tqhnsw_gettuple_duplicate_multipage VALUES \
                 ({id}, '[dim=4,bits=8,seed=42,gamma=0.5]:{payload}'::tqvector)",
                payload = hex::encode(&duplicate_payload),
            ))
            .expect("duplicate insert should succeed");
        }

        for id in 11..=128 {
            let code = (0..payload_len)
                .map(|offset| ((id * 17 + offset as i32) & 0xff) as u8)
                .collect::<Vec<_>>();
            Spi::run(&format!(
                "INSERT INTO tqhnsw_gettuple_duplicate_multipage VALUES \
                 ({id}, '[dim=4,bits=8,seed=42,gamma=0.5]:{payload}'::tqvector)",
                payload = hex::encode(code),
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_duplicate_multipage_idx ON tqhnsw_gettuple_duplicate_multipage USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 4, ef_construction = 64)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_gettuple_duplicate_multipage_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let (block_count, _metadata, _data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert!(
            block_count > 2,
            "duplicate-heavy linear scan coverage should span multiple data pages"
        );

        let observed_tids =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
        let expected_tids = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                        split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number
                     FROM tqhnsw_gettuple_duplicate_multipage
                     ORDER BY id",
                    None,
                    &[],
                )
                .expect("ctid query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    (
                        u32::try_from(block_number).expect("block number should be non-negative"),
                        u16::try_from(offset_number).expect("offset number should be positive"),
                    )
                })
                .collect::<Vec<_>>()
        });

        let mut observed_tids = observed_tids;
        let mut expected_tids = expected_tids;
        observed_tids.sort_unstable();
        expected_tids.sort_unstable();
        assert_eq!(
            observed_tids, expected_tids,
            "scan should drain duplicate heap tids and continue across later data pages without duplicating candidate-driven results"
        );
    }

    #[pg_test]
    fn test_tqhnsw_gettuple_scaffold_returns_false_for_empty_index() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_empty_scaffold (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_empty_scaffold_idx ON tqhnsw_gettuple_empty_scaffold USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_gettuple_empty_scaffold_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let found_tuple =
            unsafe { am::debug_gettuple_after_rescan_result(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
        assert!(
            !found_tuple,
            "amgettuple should report no tuples for a valid rescan on an empty index"
        );
    }

    #[pg_test]
    fn test_tqhnsw_empty_scan_stays_false() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_empty_repeated (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_empty_repeated_idx ON tqhnsw_gettuple_empty_repeated USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_gettuple_empty_repeated_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (observed_tids, exhausted_once, exhausted_twice) =
            unsafe { am::debug_gettuple_exhaustion_state(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert!(
            observed_tids.is_empty(),
            "empty indexes should not produce any heap tids before exhaustion"
        );
        assert!(
            !exhausted_once,
            "first amgettuple call on an empty index should return false"
        );
        assert!(
            !exhausted_twice,
            "repeated amgettuple calls on an empty index should remain false"
        );
    }

    #[pg_test]
    fn test_tqhnsw_empty_scan_rescan_stays_false() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_empty_rescan (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_empty_rescan_idx ON tqhnsw_gettuple_empty_rescan USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_gettuple_empty_rescan_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (first_pass, rescanned_tids) = unsafe {
            am::debug_gettuple_rescan_after_exhaustion(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert!(
            first_pass.is_empty(),
            "empty indexes should still produce no tuples before a repeated rescan"
        );
        assert!(
            rescanned_tids.is_empty(),
            "amrescan on an empty index should continue to return no tuples"
        );
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw aminsert requires matching tqvector shape")]
    fn test_tqhnsw_insert_rejects_mismatched_seed() {
        Spi::run(
            "CREATE TABLE tqhnsw_insert_seed_mismatch (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_seed_mismatch VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_seed_mismatch_idx ON tqhnsw_insert_seed_mismatch USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_seed_mismatch VALUES
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 43))",
        )
        .expect("insert should fail");
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
