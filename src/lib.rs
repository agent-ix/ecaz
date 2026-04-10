use pgrx::extension_sql_file;
use pgrx::ffi::CString;
use pgrx::prelude::*;
use pgrx::{pg_sys, Internal};

pgrx::pg_module_magic!();

#[allow(dead_code)]
mod am;
mod quant;

use quant::prod::{payload_len, ProdQuantizer};

#[pg_guard]
pub extern "C-unwind" fn _PG_init() {
    am::register_gucs();
}

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

fn tqhnsw_access_method_oid() -> pg_sys::Oid {
    Spi::get_one::<pg_sys::Oid>("SELECT oid FROM pg_am WHERE amname = 'tqhnsw'")
        .expect("SPI query should succeed")
        .expect("tqhnsw access method should exist")
}

unsafe fn open_valid_tqhnsw_index(
    index_oid: pg_sys::Oid,
    caller_name: &'static str,
) -> pg_sys::Relation {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let rd_rel = unsafe { (*index_relation).rd_rel.as_ref() }
        .expect("opened index relation should expose pg_class metadata");
    if rd_rel.relkind != pg_sys::RELKIND_INDEX as i8 as std::ffi::c_char {
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        pgrx::error!("{caller_name} requires an index relation");
    }
    if rd_rel.relam != tqhnsw_access_method_oid() {
        let relation_name = unsafe { std::ffi::CStr::from_ptr(rd_rel.relname.data.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        pgrx::error!("{caller_name} requires a tqhnsw index, got relation \"{relation_name}\"");
    }
    index_relation
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

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn tqhnsw_index_cost_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(planner_scan_enabled, bool),
        name!(planner_gate_reason, String),
        name!(relation_ef_search, i32),
        name!(session_ef_search, Option<i32>),
        name!(effective_ef_search, i32),
        name!(effective_source, String),
        name!(m, i32),
        name!(dimensions, i32),
        name!(max_level, i32),
        name!(resolved_tree_height, f64),
        name!(tree_height_source, String),
        name!(pg18_tree_height_callback_ready, bool),
        name!(index_pages, f64),
        name!(reltuples, f64),
        name!(random_page_cost, f64),
        name!(seq_page_cost, f64),
        name!(cpu_operator_cost, f64),
        name!(modeled_startup_cost, f64),
        name!(modeled_total_cost, f64),
        name!(modeled_selectivity, f64),
        name!(modeled_correlation, f64),
        name!(gated_startup_cost, f64),
        name!(gated_total_cost, f64),
        name!(gated_selectivity, f64),
        name!(gated_correlation, f64),
    ),
> {
    let index_relation =
        unsafe { open_valid_tqhnsw_index(index_oid, "tqhnsw_index_cost_snapshot") };
    let snapshot = unsafe { am::index_cost_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        snapshot.planner_scan_enabled,
        snapshot.planner_gate_reason.to_owned(),
        snapshot.relation_ef_search,
        snapshot.session_ef_search,
        snapshot.effective_ef_search,
        snapshot.effective_source.to_owned(),
        snapshot.m,
        i32::from(snapshot.dimensions),
        i32::from(snapshot.max_level),
        snapshot.resolved_tree_height,
        snapshot.tree_height_source.to_owned(),
        snapshot.pg18_tree_height_callback_ready,
        snapshot.index_pages,
        snapshot.reltuples,
        snapshot.random_page_cost,
        snapshot.seq_page_cost,
        snapshot.cpu_operator_cost,
        snapshot.modeled_startup_cost,
        snapshot.modeled_total_cost,
        snapshot.modeled_selectivity,
        snapshot.modeled_correlation,
        snapshot.gated_startup_cost,
        snapshot.gated_total_cost,
        snapshot.gated_selectivity,
        snapshot.gated_correlation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn tqhnsw_planner_integration_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(planner_scan_enabled, bool),
        name!(ordered_scan_ready, bool),
        name!(runtime_ordered_scan_ready, bool),
        name!(planner_cost_model_ready, bool),
        name!(planner_cost_callback_live, bool),
        name!(pg18_callback_surface_ready, bool),
        name!(pg18_diagnostics_surface_ready, bool),
        name!(pg18_read_stream_surface_ready, bool),
        name!(effective_ef_search, i32),
        name!(effective_source, String),
        name!(planner_gate_reason, String),
        name!(next_runtime_blocker, String),
        name!(next_pg18_blocker, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_tqhnsw_index(index_oid, "tqhnsw_planner_integration_snapshot") };
    let snapshot = unsafe { am::planner_integration_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        snapshot.planner_scan_enabled,
        snapshot.ordered_scan_ready,
        snapshot.runtime_ordered_scan_ready,
        snapshot.planner_cost_model_ready,
        snapshot.planner_cost_callback_live,
        snapshot.pg18_callback_surface_ready,
        snapshot.pg18_diagnostics_surface_ready,
        snapshot.pg18_read_stream_surface_ready,
        snapshot.effective_ef_search,
        snapshot.effective_source.to_owned(),
        snapshot.planner_gate_reason.to_owned(),
        snapshot.next_runtime_blocker.to_owned(),
        snapshot.next_pg18_blocker.to_owned(),
    ))
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
    use rand::Rng;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::collections::{HashMap, HashSet};
    use std::time::Instant;

    const RECALL_BITS: i32 = 4;
    const RECALL_SEED: i64 = 42;
    const RECALL_DIM: usize = 1536;
    const RECALL_CORPUS_SIZE: usize = 10_000;
    const RECALL_QUERY_COUNT: usize = 100;
    const RECALL_K: usize = 10;
    const RECALL_EF_CONSTRUCTION: i32 = 128;
    const RECALL_INSERT_BATCH_SIZE: usize = 32;
    const RECALL_GATE_CONFIGS: [(i32, i32, Option<f32>); 4] = [
        (8, 40, None),
        (8, 128, Some(0.89_f32)),
        (8, 200, None),
        (16, 200, None),
    ];

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

    fn random_unit_vectors(n: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut vectors = Vec::with_capacity(n);

        for _ in 0..n {
            let mut values: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0f32..1.0f32)).collect();
            let norm = values.iter().map(|v| v * v).sum::<f32>().sqrt();
            for value in &mut values {
                *value /= norm.max(f32::EPSILON);
            }
            vectors.push(values);
        }

        vectors
    }

    fn dot_product(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b).map(|(x, y)| x * y).sum()
    }

    fn brute_force_top_k(corpus: &[Vec<f32>], query: &[f32], k: usize) -> Vec<usize> {
        let mut scores: Vec<(usize, f32)> = corpus
            .iter()
            .enumerate()
            .map(|(i, vector)| (i, dot_product(query, vector)))
            .collect();
        scores.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        scores.truncate(k);
        scores.into_iter().map(|(i, _)| i).collect()
    }

    fn encoded_code_bytes(encoded: crate::bench_api::EncodedTq) -> Vec<u8> {
        let mut code_bytes = encoded.mse_packed;
        code_bytes.extend_from_slice(&encoded.qjl_packed);
        code_bytes
    }

    fn encode_recall_query_code(query: &[f32]) -> Vec<u8> {
        let quantizer = ProdQuantizer::cached(
            RECALL_DIM,
            u8::try_from(RECALL_BITS).expect("recall bits should fit into u8"),
            RECALL_SEED as u64,
        );
        encoded_code_bytes(quantizer.encode(query))
    }

    fn encode_recall_corpus_codes(corpus: &[Vec<f32>]) -> Vec<Vec<u8>> {
        let quantizer = ProdQuantizer::cached(
            RECALL_DIM,
            u8::try_from(RECALL_BITS).expect("recall bits should fit into u8"),
            RECALL_SEED as u64,
        );
        corpus
            .iter()
            .map(|vector| encoded_code_bytes(quantizer.encode(vector)))
            .collect()
    }

    fn brute_force_top_k_code_inner_product(
        corpus_codes: &[Vec<u8>],
        query_code: &[u8],
        k: usize,
    ) -> Vec<usize> {
        let mut scores = corpus_codes
            .iter()
            .enumerate()
            .map(|(i, code)| {
                (
                    i,
                    score_code_inner_product(
                        RECALL_DIM,
                        u8::try_from(RECALL_BITS).expect("recall bits should fit into u8"),
                        RECALL_SEED as u64,
                        query_code,
                        code,
                    ),
                )
            })
            .collect::<Vec<_>>();
        scores.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        scores.truncate(k);
        scores.into_iter().map(|(i, _)| i).collect()
    }

    fn create_recall_table(table_name: &str) {
        Spi::run(&format!(
            "CREATE TABLE {table_name} (id bigint primary key, embedding tqvector)"
        ))
        .expect("recall benchmark table creation should succeed");
    }

    fn create_recall_table_with_source(table_name: &str) {
        Spi::run(&format!(
            "CREATE TABLE {table_name} (
                id bigint primary key,
                source real[] NOT NULL,
                embedding tqvector
            )"
        ))
        .expect("recall benchmark source table creation should succeed");
    }

    fn insert_recall_corpus(table_name: &str, corpus: &[Vec<f32>]) {
        for batch in corpus.chunks(RECALL_INSERT_BATCH_SIZE).enumerate() {
            let (batch_index, embeddings) = batch;
            let batch_offset = batch_index * RECALL_INSERT_BATCH_SIZE;
            let values_sql = embeddings
                .iter()
                .enumerate()
                .map(|(batch_row, embedding)| {
                    format!(
                        "({}, encode_to_tqvector({}, {RECALL_BITS}, {RECALL_SEED}))",
                        batch_offset + batch_row,
                        format_recall_vector_sql_literal(embedding),
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO {table_name} (id, embedding) VALUES {values_sql}"
            ))
            .expect("recall benchmark batch insert should succeed");
        }
    }

    fn insert_recall_corpus_with_source(table_name: &str, corpus: &[Vec<f32>]) {
        for batch in corpus.chunks(RECALL_INSERT_BATCH_SIZE).enumerate() {
            let (batch_index, embeddings) = batch;
            let batch_offset = batch_index * RECALL_INSERT_BATCH_SIZE;
            let values_sql = embeddings
                .iter()
                .enumerate()
                .map(|(batch_row, embedding)| {
                    let source = format_recall_vector_sql_literal(embedding);
                    format!(
                        "({}, {}, encode_to_tqvector({}, {RECALL_BITS}, {RECALL_SEED}))",
                        batch_offset + batch_row,
                        source,
                        source,
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO {table_name} (id, source, embedding) VALUES {values_sql}"
            ))
            .expect("recall benchmark source batch insert should succeed");
        }
    }

    fn format_recall_vector_sql_literal(embedding: &[f32]) -> String {
        format!(
            "ARRAY[{}]::real[]",
            embedding
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    fn ctid_id_map(table_name: &str) -> HashMap<(u32, u16), usize> {
        Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT
                            split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                            split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number,
                            id
                         FROM {table_name}"
                    ),
                    None,
                    &[],
                )
                .expect("ctid/id map query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    let id = row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null");
                    (
                        (
                            u32::try_from(block_number)
                                .expect("block number should be non-negative"),
                            u16::try_from(offset_number)
                                .expect("offset number should be positive"),
                        ),
                        usize::try_from(id).expect("id should fit into usize"),
                    )
                })
                .collect::<HashMap<_, _>>()
        })
    }

    fn create_recall_index(table_name: &str, index_name: &str, m: i32) -> pg_sys::Oid {
        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = {m}, ef_construction = {RECALL_EF_CONSTRUCTION})"
        ))
        .expect("recall benchmark index creation should succeed");

        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
            .expect("recall benchmark index oid query should succeed")
            .expect("recall benchmark index oid should exist")
    }

    fn create_recall_index_with_source_build(
        table_name: &str,
        index_name: &str,
        m: i32,
    ) -> pg_sys::Oid {
        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (
                 m = {m},
                 ef_construction = {RECALL_EF_CONSTRUCTION},
                 build_source_column = 'source'
             )"
        ))
        .expect("recall benchmark source-build index creation should succeed");

        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
            .expect("recall benchmark source-build index oid query should succeed")
            .expect("recall benchmark source-build index oid should exist")
    }

    fn recall_index_block_count(index_oid: pg_sys::Oid, caller_name: &'static str) -> i32 {
        let index_relation = unsafe { open_valid_tqhnsw_index(index_oid, caller_name) };
        let index_block_count = unsafe {
            i32::try_from(pg_sys::RelationGetNumberOfBlocksInFork(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
            ))
            .expect("block count should fit into int")
        };
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        index_block_count
    }

    fn recall_fixture_ident(label: &str) -> String {
        assert!(!label.is_empty(), "recall fixture names must not be empty");
        assert!(
            label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'),
            "recall fixture names must be ASCII alphanumeric or underscore only"
        );
        label.to_owned()
    }

    fn reset_graph_scan_recall_fixture(fixture_name: &str, m: i32, corpus_size: usize) -> i32 {
        assert!(corpus_size >= RECALL_K);

        let fixture_name = recall_fixture_ident(fixture_name);
        let index_name = format!("{fixture_name}_idx");
        let corpus = random_unit_vectors(corpus_size, RECALL_DIM, RECALL_SEED as u64);

        Spi::run(&format!("DROP TABLE IF EXISTS {fixture_name} CASCADE"))
            .expect("recall fixture cleanup should succeed");
        create_recall_table(&fixture_name);
        insert_recall_corpus(&fixture_name, &corpus);
        let index_oid = create_recall_index(&fixture_name, &index_name, m);
        recall_index_block_count(index_oid, "reset_graph_scan_recall_fixture")
    }

    fn gate_fixture_already_exists(
        table_name: &str,
        fixture_prefix: &str,
        corpus_size: usize,
    ) -> Option<Vec<(i32, i32)>> {
        let table_exists = Spi::get_one::<bool>(&format!(
            "SELECT EXISTS (
                 SELECT 1
                 FROM pg_class
                 WHERE relname = '{table_name}'
                   AND relkind = 'r'
             )"
        ))
        .expect("table existence check should succeed")
        .unwrap_or(false);
        if !table_exists {
            return None;
        }

        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) FROM {table_name}"))
            .expect("row count query should succeed")
            .unwrap_or(0);
        if row_count != i64::try_from(corpus_size).expect("corpus size should fit into i64") {
            return None;
        }

        let mut results = Vec::new();
        for m in [8, 16] {
            let index_name = format!("{fixture_prefix}_m{m}_idx");
            let expected_m = format!("m={m}");
            let expected_ef = format!("ef_construction={RECALL_EF_CONSTRUCTION}");
            let index_ok = Spi::get_one::<bool>(&format!(
                "SELECT EXISTS (
                     SELECT 1
                     FROM pg_class
                     WHERE relname = '{index_name}'
                       AND relkind = 'i'
                       AND reloptions @> ARRAY['{expected_m}', '{expected_ef}']
                 )"
            ))
            .expect("index existence check should succeed")
            .unwrap_or(false);
            if !index_ok {
                return None;
            }

            let index_oid =
                Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                    .expect("index oid query should succeed")
                    .expect("index oid should exist");
            let block_count = recall_index_block_count(index_oid, "gate_fixture_already_exists");
            results.push((m, block_count));
        }

        Some(results)
    }

    fn reset_graph_scan_recall_gate_fixtures(
        fixture_prefix: &str,
        corpus_size: usize,
    ) -> Vec<(i32, i32)> {
        let fixture_prefix = recall_fixture_ident(fixture_prefix);
        let table_name = format!("{fixture_prefix}_corpus");

        if let Some(existing) =
            gate_fixture_already_exists(&table_name, &fixture_prefix, corpus_size)
        {
            pgrx::log!("fixture already exists, skipping rebuild: {table_name}");
            return existing;
        }

        let corpus = random_unit_vectors(corpus_size, RECALL_DIM, RECALL_SEED as u64);

        Spi::run(&format!("DROP TABLE IF EXISTS {table_name} CASCADE"))
            .expect("recall gate fixture cleanup should succeed");
        create_recall_table(&table_name);
        insert_recall_corpus(&table_name, &corpus);

        [8, 16]
            .into_iter()
            .map(|m| {
                let index_name = format!("{fixture_prefix}_m{m}_idx");
                let index_oid = create_recall_index(&table_name, &index_name, m);
                let index_block_count =
                    recall_index_block_count(index_oid, "reset_graph_scan_recall_gate_fixtures");
                (m, index_block_count)
            })
            .collect()
    }

    fn reset_graph_scan_recall_gate_source_fixtures(
        fixture_prefix: &str,
        corpus_size: usize,
    ) -> Vec<(i32, i32)> {
        let fixture_prefix = recall_fixture_ident(fixture_prefix);
        let table_name = format!("{fixture_prefix}_corpus");
        let corpus = random_unit_vectors(corpus_size, RECALL_DIM, RECALL_SEED as u64);

        Spi::run(&format!("DROP TABLE IF EXISTS {table_name} CASCADE"))
            .expect("recall gate source fixture cleanup should succeed");
        create_recall_table_with_source(&table_name);
        insert_recall_corpus_with_source(&table_name, &corpus);

        [8, 16]
            .into_iter()
            .map(|m| {
                let index_name = format!("{fixture_prefix}_m{m}_idx");
                let index_oid = create_recall_index_with_source_build(&table_name, &index_name, m);
                let index_block_count = recall_index_block_count(
                    index_oid,
                    "reset_graph_scan_recall_gate_source_fixtures",
                );
                (m, index_block_count)
            })
            .collect()
    }

    fn measure_graph_scan_recall(
        index_oid: pg_sys::Oid,
        ctid_to_id: &HashMap<(u32, u16), usize>,
        queries: &[Vec<f32>],
        ground_truth: &[Vec<usize>],
        ef_search: i32,
    ) -> f32 {
        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let hits = queries
            .iter()
            .zip(ground_truth.iter())
            .map(|(query, true_top_k)| {
                let predicted =
                    unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) };
                let predicted_top_k: HashSet<usize> = predicted
                    .iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        *ctid_to_id
                            .get(heap_tid)
                            .expect("emitted heap tid should map back to a benchmark row id")
                    })
                    .collect();

                true_top_k
                    .iter()
                    .filter(|id| predicted_top_k.contains(id))
                    .count()
            })
            .sum::<usize>();

        hits as f32 / (queries.len() * RECALL_K) as f32
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
    fn test_tqhnsw_index_cost_snapshot_reports_modeled_and_gated_costs() {
        Spi::run("CREATE TABLE tqhnsw_cost_snapshot (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_cost_snapshot VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.5, 0.5, -0.5, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_cost_snapshot_idx ON tqhnsw_cost_snapshot USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 12, ef_search = 77)",
        )
        .expect("index creation should succeed");
        Spi::run("SET tqhnsw.ef_search = 19").expect("set should succeed");
        Spi::run("ANALYZE tqhnsw_cost_snapshot").expect("analyze should succeed");

        assert!(
            !Spi::get_one::<bool>(
                "SELECT planner_scan_enabled FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("planner flag should be non-null"),
            "planner gate should still be off"
        );
        assert!(
            Spi::get_one::<String>(
                "SELECT planner_gate_reason FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("gate reason should be non-null")
                .contains("disabled"),
            "cost snapshot should explain why live planner costing is still gated"
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT relation_ef_search FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("relation ef_search should be non-null"),
            77
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT session_ef_search FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed"),
            Some(19)
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT effective_ef_search FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective ef_search should be non-null"),
            19
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT effective_source FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective source should be non-null"),
            "session"
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT m FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("m should be non-null"),
            12
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT dimensions FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("dimensions should be non-null"),
            4
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT resolved_tree_height FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("resolved tree height should be non-null"),
            0.0
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT tree_height_source FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("tree height source should be non-null"),
            "metadata_fallback"
        );
        assert!(
            !Spi::get_one::<bool>(
                "SELECT pg18_tree_height_callback_ready FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("pg18 tree-height callback flag should be non-null")
        );
        assert!(
            Spi::get_one::<f64>(
                "SELECT index_pages FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("index pages should be non-null")
                >= 1.0
        );
        assert!(
            Spi::get_one::<f64>(
                "SELECT reltuples FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("reltuples should be non-null")
                >= 3.0
        );
        let modeled_startup = Spi::get_one::<f64>(
            "SELECT modeled_startup_cost FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("modeled startup should be non-null");
        let modeled_total = Spi::get_one::<f64>(
            "SELECT modeled_total_cost FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("modeled total should be non-null");
        assert!(
            modeled_startup.is_finite(),
            "modeled startup should be finite"
        );
        assert!(modeled_total.is_finite(), "modeled total should be finite");
        assert!(
            modeled_total >= modeled_startup,
            "modeled total cost should include startup cost"
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT modeled_selectivity FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("modeled selectivity should be non-null"),
            1.0
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT modeled_correlation FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("modeled correlation should be non-null"),
            0.0
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT gated_startup_cost FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("gated startup should be non-null"),
            f64::MAX
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT gated_total_cost FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("gated total should be non-null"),
            f64::MAX
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT gated_selectivity FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("gated selectivity should be non-null"),
            0.0
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT gated_correlation FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("gated correlation should be non-null"),
            0.0
        );

        Spi::run("RESET tqhnsw.ef_search").expect("reset should succeed");
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw_index_cost_snapshot requires a tqhnsw index")]
    fn test_tqhnsw_index_cost_snapshot_rejects_non_tqhnsw_index() {
        Spi::run(
            "CREATE TABLE tqhnsw_cost_snapshot_wrong_am (id bigint primary key, value bigint)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_cost_snapshot_wrong_am_idx ON tqhnsw_cost_snapshot_wrong_am USING btree (value)",
        )
        .expect("index creation should succeed");

        let _ = Spi::get_one::<f64>(
            "SELECT modeled_total_cost FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_wrong_am_idx'::regclass)",
        );
    }

    #[pg_test]
    fn test_tqhnsw_planner_integration_snapshot_reports_blockers() {
        Spi::run(
            "CREATE TABLE tqhnsw_planner_integration_snapshot (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_planner_integration_snapshot_idx ON tqhnsw_planner_integration_snapshot USING tqhnsw (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        assert!(
            !Spi::get_one::<bool>(
                "SELECT planner_scan_enabled FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("planner scan flag should be non-null")
        );
        assert!(
            !Spi::get_one::<bool>(
                "SELECT ordered_scan_ready FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("ordered scan readiness should be non-null")
        );
        assert!(
            !Spi::get_one::<bool>(
                "SELECT runtime_ordered_scan_ready FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("runtime ordered scan readiness should be non-null")
        );
        assert!(
            Spi::get_one::<bool>(
                "SELECT planner_cost_model_ready FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("planner cost model readiness should be non-null")
        );
        assert!(
            !Spi::get_one::<bool>(
                "SELECT planner_cost_callback_live FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("planner cost callback live flag should be non-null")
        );
        assert!(
            !Spi::get_one::<bool>(
                "SELECT pg18_callback_surface_ready FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("pg18 callback readiness should be non-null")
        );
        assert!(
            !Spi::get_one::<bool>(
                "SELECT pg18_diagnostics_surface_ready FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("pg18 diagnostics readiness should be non-null")
        );
        assert!(
            !Spi::get_one::<bool>(
                "SELECT pg18_read_stream_surface_ready FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("pg18 read stream readiness should be non-null")
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT effective_ef_search FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective ef_search should be non-null"),
            40
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT effective_source FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective source should be non-null"),
            "relation"
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT planner_gate_reason FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("planner gate reason should be non-null"),
            "planner scan selection is disabled until ordered tqhnsw execution is credible"
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT next_runtime_blocker FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("runtime blocker should be non-null"),
            "ordered tqhnsw scan semantics and recall validation are not yet credible"
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT next_pg18_blocker FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("pg18 blocker should be non-null"),
            "pgrx pg18 feature support and callback bindings are not yet implemented"
        );
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw_planner_integration_snapshot requires a tqhnsw index")]
    fn test_tqhnsw_planner_integration_snapshot_rejects_wrong_am() {
        Spi::run(
            "CREATE TABLE tqhnsw_planner_integration_snapshot_wrong_am (id bigint primary key, value bigint)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_planner_integration_snapshot_wrong_am_idx ON tqhnsw_planner_integration_snapshot_wrong_am USING btree (value)",
        )
        .expect("index creation should succeed");

        let _ = Spi::get_one::<bool>(
            "SELECT planner_scan_enabled FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_wrong_am_idx'::regclass)",
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
            assert_eq!(
                neighbor.count as usize,
                neighbor.tids.len(),
                "neighbor tuples should persist every logical layer slot so runtime layer slicing stays stable",
            );
            assert_eq!(
                neighbor.tids.len(),
                am::page::neighbor_slots(element.level, metadata.m),
                "neighbor tuples should carry the full 2M / M slot payload for the node level instead of compacting active neighbors",
            );
            assert!(!neighbor.tids.contains(element_tid));
            assert!(neighbor.tids.iter().all(|tid| {
                *tid == am::page::ItemPointer::INVALID || element_ids.contains(tid)
            }));
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
            assert!(neighbor.tids.iter().all(|tid| {
                *tid == am::page::ItemPointer::INVALID || element_tids.contains(tid)
            }));
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

        let default_m = 8_u16;
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
                let neighbor_slot_count = am::page::neighbor_slots(0, default_m);
                let neighbor = am::page::TqNeighborTuple {
                    count: neighbor_slot_count as u16,
                    tids: vec![am::page::ItemPointer::INVALID; neighbor_slot_count],
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

        assert!(
            observed_tids.contains(&expected_tids[0]),
            "graph-first scan should return the nearest indexed heap tid for the query"
        );
        assert_eq!(
            observed_tids.len(),
            observed_tids
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            "graph-first scan should not emit duplicate heap tids"
        );
        assert!(
            observed_tids
                .iter()
                .all(|heap_tid| expected_tids.contains(heap_tid)),
            "every emitted heap tid should still belong to the indexed table"
        );
    }

    #[pg_test]
    fn test_tqhnsw_graph_first_scan_emits_distance_sorted_scores() {
        Spi::run(
            "CREATE TABLE tqhnsw_graph_first_ordered_scores (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_graph_first_ordered_scores VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.92, 0.08, 0.0, 0.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.75, 0.25, 0.0, 0.0], 4, 42)),
             (4, encode_to_tqvector(ARRAY[0.55, 0.45, 0.0, 0.0], 4, 42)),
             (5, encode_to_tqvector(ARRAY[0.35, 0.65, 0.0, 0.0], 4, 42)),
             (6, encode_to_tqvector(ARRAY[0.15, 0.85, 0.0, 0.0], 4, 42)),
             (7, encode_to_tqvector(ARRAY[-0.2, 0.98, 0.0, 0.0], 4, 42)),
             (8, encode_to_tqvector(ARRAY[-0.7, 0.3, 0.0, 0.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_graph_first_ordered_scores_idx ON tqhnsw_graph_first_ordered_scores USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 4, ef_construction = 64)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_graph_first_ordered_scores_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let ctid_to_id = ctid_id_map("tqhnsw_graph_first_ordered_scores");
        let observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_scores(index_oid, vec![1.0, 0.05, 0.0, 0.0])
        };

        assert!(
            observed.len() >= 3,
            "non-trivial built indexes should emit multiple graph-first ordered results"
        );
        assert_eq!(
            observed.len(),
            observed
                .iter()
                .map(|(heap_tid, _)| *heap_tid)
                .collect::<HashSet<_>>()
                .len(),
            "graph-first ordered scans should not emit the same heap tid twice"
        );
        assert!(
            observed
                .windows(2)
                .all(|pair| pair[0].1 <= pair[1].1 + f32::EPSILON),
            "graph-first scan should emit tuples in nondecreasing operator-facing <#> score order"
        );
        assert!(
            observed
                .iter()
                .all(|(heap_tid, _)| ctid_to_id.contains_key(heap_tid)),
            "every emitted heap tid should map back to a row in the built index table"
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
            before_found,
            "seeded graph-first rescans should prefill the first ordered result before amgettuple runs"
        );
        assert_ne!(
            before_tid,
            (u32::MAX, u16::MAX),
            "seeded graph-first rescans should expose a concrete current-result element tid immediately"
        );
        assert!(
            before_score,
            "seeded graph-first rescans should expose an order-by score before the first tuple drain"
        );
        assert_eq!(
            before_score_value, expected_score,
            "prefilled graph-first current-result score should already match the operator-facing <#> value"
        );
        assert!(
            found,
            "first gettuple call should produce a tuple for a non-empty index"
        );
        if after_tid == (u32::MAX, u16::MAX) {
            assert!(
                !after_score,
                "if the graph lane exhausts immediately after the first tuple drain, it should clear the current-result score-valid bit too"
            );
            assert_eq!(
                after_score_value, 0.0,
                "if the graph lane exhausts immediately after the first tuple drain, it should clear the current-result score value too"
            );
        } else {
            assert!(
                after_score,
                "when graph traversal stays hot after the first tuple drain, it should keep the current result score valid"
            );
            assert_ne!(
                after_score_value, 0.0,
                "when graph traversal stays hot after the first tuple drain, it should keep a concrete current-result score populated"
            );
            assert_ne!(
                after_tid, before_tid,
                "when graph traversal has another result ready, the current result should advance to the next prefetched candidate"
            );
        }
    }

    #[pg_test]
    fn test_tqhnsw_gettuple_emits_orderby_score() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_orderby_score (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_gettuple_orderby_score VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_orderby_score_idx ON tqhnsw_gettuple_orderby_score USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_gettuple_orderby_score_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![1.0, 0.0, 0.5, -1.0];
        let (found, orderby_is_null, orderby_score) =
            unsafe { am::debug_gettuple_orderby_score(index_oid, query.clone()) };
        let expected_score = Spi::get_one::<f32>(&format!(
            "SELECT embedding <#> ARRAY[{},{},{},{}]::real[] \
             FROM tqhnsw_gettuple_orderby_score WHERE id = 1",
            query[0], query[1], query[2], query[3],
        ))
        .expect("score query should succeed")
        .expect("score should exist");

        assert!(
            found,
            "first gettuple call should produce a tuple for a non-empty index"
        );
        assert!(
            !orderby_is_null,
            "visible tuple production should populate xs_orderbynulls[0] as non-null"
        );
        assert_eq!(
            orderby_score, expected_score,
            "amgettuple should publish the current result score through xs_orderbyvals[0]"
        );
    }

    #[pg_test]
    fn test_tqhnsw_gettuple_clears_orderby_score_on_rescan() {
        Spi::run(
            "CREATE TABLE tqhnsw_gettuple_orderby_lifecycle (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_gettuple_orderby_lifecycle VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_gettuple_orderby_lifecycle_idx ON tqhnsw_gettuple_orderby_lifecycle USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_gettuple_orderby_lifecycle_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (before, after_first, exhausted, rescanned) = unsafe {
            am::debug_gettuple_orderby_score_lifecycle(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert_eq!(
            before, None,
            "order-by output should start empty before tuple production"
        );
        assert!(
            after_first.is_some(),
            "first tuple production should publish a non-null order-by score"
        );
        assert_eq!(
            exhausted, None,
            "exhaustion should clear the visible order-by score instead of leaving stale output"
        );
        assert_eq!(
            rescanned, None,
            "amrescan should clear any prior order-by score before the next tuple is produced"
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

        assert_ne!(
            second_tid, first_tid,
            "after the last duplicate drain, graph traversal should either prefill the next current result or clear the old one"
        );
        if second_tid == (u32::MAX, u16::MAX) {
            assert!(
                !second_score,
                "if graph traversal exhausts after the last duplicate drain, it should clear the current-result score-valid bit"
            );
            assert_eq!(
                second_score_value, 0.0,
                "if graph traversal exhausts after the last duplicate drain, it should clear the current-result score value"
            );
        } else {
            assert!(
                second_score,
                "prefilling the next graph result should keep the current result score valid"
            );
            assert_ne!(
                second_score_value, 0.0,
                "prefilling the next graph result should keep a concrete score populated"
            );
        }
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
            first_tid,
            "amrescan should prefill the first graph-ordered current result again before the next tuple is produced"
        );
        assert!(
            rescanned_score,
            "amrescan should restore the current result score-valid bit for the prefetched graph result"
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
        let (
            element_tid,
            first_heap_tid,
            second_element_tid,
            second_heap_tid,
            first_score,
            second_score,
        ) = unsafe {
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
            second_element_tid,
            element_tid,
            "after the last duplicate drain, the graph lane should prefill the next current result element"
        );
        assert_ne!(
            second_heap_tid, first_heap_tid,
            "the prefetched next result should not leave the old duplicate heap tid attached"
        );
        assert_eq!(
            second_heap_tid,
            (u32::MAX, u16::MAX),
            "a freshly prefetched next result should not yet have a heap tid attached"
        );
        assert_ne!(
            first_score, second_score,
            "prefilling the next result should allow the current-result score to advance with the graph order"
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
        let (before_valid, before_tid, before_score, after_valid, after_tid, after_score) =
            unsafe { am::debug_rescan_entry_candidate_state(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert!(
            before_valid,
            "amrescan should seed a concrete graph-ordered starting result for a non-empty index"
        );
        assert_ne!(before_tid, (u32::MAX, u16::MAX));
        assert_ne!(
            before_score, 0.0,
            "the initial graph-ordered result should carry a computed score for future tuple production"
        );
        assert!(
            !after_valid,
            "the initial graph result state should clear once the bootstrap scan fully exhausts"
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
            partial_exhausted,
            exhausted_valid,
            exhausted_tid,
            exhausted_score,
        ) = unsafe { am::debug_entry_candidate_lifecycle(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert!(
            before_valid,
            "entry candidate should be seeded before tuple production"
        );
        assert!(
            partial_valid || partial_result_tid != (u32::MAX, u16::MAX) || partial_exhausted,
            "partial scan progress should keep either a remaining frontier candidate, a concrete current result, or an explicit exhausted state"
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
            if partial_result_tid == (u32::MAX, u16::MAX) {
                assert!(
                    partial_exhausted,
                    "if partial scan progress no longer exposes a frontier head or current result, the graph lane should already be exhausted"
                );
            } else {
                assert_ne!(
                    partial_result_tid,
                    (u32::MAX, u16::MAX),
                    "when the frontier head materializes immediately, partial scan progress should keep a concrete current-result tid"
                );
            }
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
        let (
            entry_tid,
            entry_neighbors,
            successor_valid,
            successor_tid,
            _successor_source_tid,
            successor_score,
        ) = unsafe {
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
                successor_tid != (u32::MAX, u16::MAX),
                "after amrescan prefill, a live entry adjacency should leave a concrete next ordered slot"
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
            !frontier.is_empty(),
            "bootstrap frontier should not be empty immediately after rescan"
        );
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

        if let Some(second) = frontier.get(1) {
            assert_ne!(
                second.1,
                (u32::MAX, u16::MAX),
                "second frontier slot should expose a concrete element tid when present"
            );
            assert_ne!(
                second.2, 0.0,
                "second frontier slot should carry a computed score when present"
            );
        } else {
            assert_eq!(frontier.len(), 1, "a missing second frontier slot should mean the Vec contains only the seeded entry candidate");
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
    fn test_tqhnsw_rescan_respects_ef_search_frontier_limit() {
        Spi::run(
            "CREATE TABLE tqhnsw_candidate_frontier_limit (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_candidate_frontier_limit VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_candidate_frontier_limit_idx ON tqhnsw_candidate_frontier_limit USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (ef_search = 1)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_candidate_frontier_limit_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (_head, frontier, frontier_slots, frontier_provenance, _expanded_sources) =
            unsafe { am::debug_rescan_candidate_frontier(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert_eq!(
            frontier.len(),
            1,
            "ef_search=1 should cap the visible bootstrap frontier at one candidate"
        );
        assert_eq!(
            frontier_slots.len(),
            1,
            "debug frontier slots should match the configured bootstrap frontier limit"
        );
        assert_eq!(
            frontier_provenance.len(),
            1,
            "frontier provenance should track only the single retained candidate"
        );
    }

    #[pg_test]
    fn test_tqhnsw_session_ef_search_override_limits_runtime_frontier() {
        Spi::run(
            "CREATE TABLE tqhnsw_session_runtime_frontier_limit (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_session_runtime_frontier_limit VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_session_runtime_frontier_limit_idx ON tqhnsw_session_runtime_frontier_limit USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (ef_search = 3)",
        )
        .expect("index creation should succeed");
        Spi::run("SET tqhnsw.ef_search = 1").expect("session override should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_session_runtime_frontier_limit_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (_head, frontier, frontier_slots, frontier_provenance, _expanded_sources) =
            unsafe { am::debug_rescan_candidate_frontier(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert_eq!(
            frontier.len(),
            1,
            "non-default session ef_search should override the reloption during scan bootstrap"
        );
        assert_eq!(
            frontier_slots.len(),
            1,
            "runtime frontier slots should honor the resolved session override width"
        );
        assert_eq!(
            frontier_provenance.len(),
            1,
            "runtime frontier provenance should also honor the resolved session override width"
        );

        Spi::run("RESET tqhnsw.ef_search").expect("reset should succeed");
    }

    #[pg_test]
    fn test_tqhnsw_session_ef_search_defaults_to_relation_setting() {
        Spi::run(
            "CREATE TABLE tqhnsw_session_ef_search_reloption (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_session_ef_search_reloption VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_session_ef_search_reloption_idx ON tqhnsw_session_ef_search_reloption USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (ef_search = 111)",
        )
        .expect("index creation should succeed");
        Spi::run("RESET tqhnsw.ef_search").expect("reset should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_session_ef_search_reloption_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let snapshot = unsafe { am::debug_planner_tuning_snapshot(index_oid) };

        assert_eq!(snapshot.relation_ef_search, 111);
        assert_eq!(snapshot.session_ef_search, None);
        assert_eq!(
            snapshot.effective_ef_search, 111,
            "default session setting should fall back to the index reloption"
        );
        assert_eq!(snapshot.effective_source, "relation");
        assert!(
            !snapshot.planner_scan_enabled,
            "planner-facing scan selection should remain explicitly disabled"
        );
    }

    #[pg_test]
    fn test_tqhnsw_session_ef_search_overrides_reloption() {
        Spi::run(
            "CREATE TABLE tqhnsw_session_ef_search_override (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_session_ef_search_override VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_session_ef_search_override_idx ON tqhnsw_session_ef_search_override USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (ef_search = 111)",
        )
        .expect("index creation should succeed");
        Spi::run("SET tqhnsw.ef_search = 7").expect("session override should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_session_ef_search_override_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let snapshot = unsafe { am::debug_planner_tuning_snapshot(index_oid) };

        assert_eq!(snapshot.relation_ef_search, 111);
        assert_eq!(snapshot.session_ef_search, Some(7));
        assert_eq!(
            snapshot.effective_ef_search, 7,
            "non-default session setting should override the index reloption"
        );
        assert_eq!(snapshot.effective_source, "session");

        Spi::run("RESET tqhnsw.ef_search").expect("reset should succeed");
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
            "seeded graph-first rescans should expose a concrete ordered head immediately after amrescan"
        );
        assert_eq!(
            partial_head.is_some(),
            partial_frontier.iter().any(|slot| slot.0),
            "ordered-head presence should still track whether any graph-ordered candidates remain after partial progress"
        );
        assert!(
            partial_frontier.len() <= before_frontier.len(),
            "draining the first graph-ordered tuple should not grow the remaining ordered runtime state"
        );
        assert_eq!(
            exhausted_head, None,
            "frontier head should clear on full scan exhaustion"
        );
        assert_eq!(
            exhausted_frontier,
            Vec::<(bool, (u32, u16), f32)>::new(),
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

        if let Some(consumed_tid) = before_head {
            let remaining_slot = before_frontier
                .iter()
                .position(|slot| slot.0 && slot.1 != consumed_tid);

            if let Some(remaining_slot) = remaining_slot {
                assert_eq!(
                    after_first_head,
                    Some(before_frontier[remaining_slot].1),
                    "when another candidate remains valid, consuming the head should expose that remaining candidate as the new head"
                );
                assert!(
                    after_first_frontier
                        .iter()
                        .any(|slot| slot.1 == before_frontier[remaining_slot].1),
                    "consuming the current head should preserve the remaining candidate after compaction"
                );
                assert!(
                    !after_first_frontier
                        .iter()
                        .any(|slot| slot.1 == consumed_tid),
                    "consuming the current head should remove that candidate from the frontier Vec"
                );
            } else {
                assert_eq!(
                    after_first_head, None,
                    "consuming the only valid candidate should invalidate the frontier head"
                );
                assert!(
                    after_first_frontier.is_empty(),
                    "consuming the only valid candidate should leave the compacted frontier empty"
                );
            }
        } else {
            assert_eq!(
                after_first_head, None,
                "after amrescan prefill, a tiny index may have no remaining raw frontier candidate to consume"
            );
            assert!(
                after_first_frontier.is_empty(),
                "without any remaining raw frontier candidates, the consume helper should keep the frontier empty"
            );
        }

        assert_eq!(
            after_second_head, None,
            "consuming the frontier head again should leave the frontier empty"
        );
        assert_eq!(
            after_second_frontier,
            Vec::<(bool, (u32, u16), f32)>::new(),
            "after consuming both available slots, the frontier Vec should be fully cleared"
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
             (embedding tqvector_ip_ops) WITH (ef_search = 3)",
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

        let expected_visible_width = 1 + valid_entry_neighbors.len().min(2);
        assert!(
            before_slots.len() <= expected_visible_width,
            "prefilled graph-first state should never expose more raw frontier slots than the configured width"
        );
        assert!(
            before_slots.len() >= expected_visible_width.saturating_sub(1),
            "prefilled graph-first state may leave the raw frontier one slot narrower once the current result has already been materialized"
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

        if before_slots.is_empty() {
            assert!(
                before_head.is_none(),
                "without any remaining raw frontier slots, the raw frontier head should already be empty"
            );
            assert_eq!(
                consumed_tid,
                (u32::MAX, u16::MAX),
                "when amrescan has already materialized the ordered head into current-result state, the raw frontier helper may have nothing left to consume"
            );
            assert!(
                after_head.is_none() && after_slots.is_empty(),
                "without any remaining raw frontier slots, manual frontier consume/refill should leave the raw frontier empty"
            );
            return;
        }

        assert_ne!(
            consumed_tid,
            (u32::MAX, u16::MAX),
            "non-empty frontier should expose an actually consumed candidate"
        );
        assert!(
            !after_tids.contains(&consumed_tid),
            "consuming the head should remove that candidate from the frontier"
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
        assert_eq!(
            after_tids.len(),
            after_slots.len(),
            "manual consume/refill should keep the raw frontier deduplicated"
        );

        for new_tid in new_tids {
            let new_slot = after_provenance_slots
                .iter()
                .find(|slot| slot.0 && slot.1 == new_tid)
                .expect(
                    "newly seeded candidate should remain present in the frontier provenance view",
                );
            assert!(
                (has_unseen_consumed_neighbor && consumed_neighbors.contains(&new_tid))
                    || (new_slot.2 != (u32::MAX, u16::MAX)
                        && (before_tids.contains(&new_slot.2) || after_tids.contains(&new_slot.2))),
                "manual consume/refill should only surface candidates from the consumed adjacency or another still-visible frontier source",
            );
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
        let (before_head, before_slots, current_result_tid, after_head, after_slots) = unsafe {
            am::debug_gettuple_consumes_bootstrap_candidate(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        let consumed_slot = before_slots
            .first()
            .copied()
            .expect("seeded graph-first rescans should prefill an ordered slot before the first tuple drain");
        assert_eq!(
            before_head,
            Some(consumed_slot.1),
            "seeded graph-first rescans should expose the prefetched current result as the ordered head before the first tuple drain"
        );
        assert_eq!(
            current_result_tid, consumed_slot.1,
            "the first amgettuple call should drain the already-prefilled ordered result"
        );
        assert_eq!(
            after_head.is_some(),
            !after_slots.is_empty(),
            "ordered-head presence should continue to track whether graph-ordered candidates remain after first tuple drain"
        );
        assert!(
            after_slots
                .iter()
                .all(|slot| slot.1 != consumed_slot.1 || slot.2 != consumed_slot.2),
            "after the first tuple drain, the previously emitted ordered slot should not remain queued as if it were still unseen"
        );
    }

    #[pg_test]
    fn test_tqhnsw_bootstrap_candidate_materializes_into_pending_drain() {
        Spi::run(
            "CREATE TABLE tqhnsw_bootstrap_candidate_materialize (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_bootstrap_candidate_materialize VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_bootstrap_candidate_materialize_idx ON tqhnsw_bootstrap_candidate_materialize USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_bootstrap_candidate_materialize_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (candidate_before, current_result_tid, pending_heap_tids, materialized) = unsafe {
            am::debug_materialize_bootstrap_candidate_result(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert!(
            candidate_before.0,
            "bootstrap frontier should yield a candidate before direct materialization"
        );
        assert!(
            materialized,
            "bootstrap candidate should materialize into the pending heap-tid drain path"
        );
        assert_eq!(
            current_result_tid, candidate_before.1,
            "materializing the bootstrap candidate should attach current-result state to that candidate"
        );
        assert_eq!(
            pending_heap_tids.len(),
            2,
            "duplicate-coalesced bootstrap candidates should populate all duplicate heap tids into pending drain state"
        );
    }

    #[pg_test]
    fn test_tqhnsw_bootstrap_phase_completes_and_resets_on_rescan() {
        Spi::run(
            "CREATE TABLE tqhnsw_bootstrap_phase_transition (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_bootstrap_phase_transition VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_bootstrap_phase_transition_idx ON tqhnsw_bootstrap_phase_transition USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_bootstrap_phase_transition_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (before_complete, after_complete, after_head, after_frontier, rescanned_complete) =
            unsafe { am::debug_bootstrap_phase_transition(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert!(
            !before_complete,
            "amrescan should always start with bootstrap traversal enabled"
        );
        assert!(
            after_complete,
            "non-empty scan execution should eventually complete the current bootstrap phase"
        );
        assert_eq!(
            after_head, None,
            "once bootstrap phase completes, the visible frontier head should stay cleared"
        );
        assert_eq!(
            after_frontier,
            Vec::<(bool, (u32, u16), f32)>::new(),
            "once bootstrap phase completes, the visible frontier should be cleared too"
        );
        assert!(
            !rescanned_complete,
            "amrescan should reset bootstrap-phase completion for the next execution"
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
    fn test_tqhnsw_gettuple_drains_selected_duplicate_heap_tids() {
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

        let selected_duplicate_heaptids = expected_tids[..2].to_vec();
        assert!(
            selected_duplicate_heaptids
                .iter()
                .all(|heap_tid| observed_tids.contains(heap_tid)),
            "graph-first scan should still drain every heap tid stored in the selected duplicate-coalesced element"
        );
        assert_eq!(
            observed_tids.len(),
            observed_tids
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            "graph-first scan should not emit any heap tid twice while draining duplicate-backed results"
        );
        assert!(
            observed_tids
                .iter()
                .all(|heap_tid| expected_tids.contains(heap_tid)),
            "every emitted heap tid should still come from the indexed table"
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
        assert!(
            !observed_tids.is_empty(),
            "graph-first scans should still return at least one heap tid before exhaustion"
        );
        assert!(
            observed_tids.contains(&expected_tids[0]),
            "graph-first exhaustion should still include the nearest indexed heap tid"
        );
        assert_eq!(
            observed_tids.len(),
            observed_tids
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            "graph-first exhaustion should not emit duplicate heap tids before the scan ends"
        );
        assert!(
            observed_tids
                .iter()
                .all(|heap_tid| expected_tids.contains(heap_tid)),
            "every emitted heap tid should still belong to the indexed table"
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

        assert!(
            !first_pass.is_empty(),
            "the first graph-first scan should return at least one heap tid before exhaustion"
        );
        assert!(
            first_pass.contains(&expected_tids[0]),
            "the first graph-first scan should include the nearest indexed heap tid before exhaustion"
        );
        assert_eq!(
            first_pass.len(),
            first_pass
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            "the first graph-first scan should not emit duplicate heap tids before exhaustion"
        );
        assert!(
            first_pass
                .iter()
                .all(|heap_tid| expected_tids.contains(heap_tid)),
            "the first graph-first scan should only emit heap tids from the indexed table"
        );
        assert_eq!(
            rescanned_tids, first_pass,
            "amrescan after exhaustion should restart tuple production from the beginning of the graph-first output"
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
        assert!(
            rescanned_tids.contains(&expected_tids[0]) && rescanned_tids.contains(&expected_tids[1]),
            "amrescan should reset duplicate heap-tid progress back to the start of the graph-ordered duplicate drain"
        );
        assert_eq!(
            rescanned_tids.len(),
            rescanned_tids
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            "rescanning after partial progress should not introduce duplicate heap tid emission"
        );
        assert!(
            rescanned_tids
                .iter()
                .all(|heap_tid| expected_tids.contains(heap_tid)),
            "rescanned heap tids should still belong to the indexed table"
        );
    }

    #[pg_test]
    fn test_tqhnsw_gettuple_duplicate_scan_drains_selected_duplicates() {
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

        assert!(
            !observed_tids.is_empty(),
            "graph-first scan should still return at least one heap tid from a duplicate-heavy multipage index"
        );
        assert_eq!(
            observed_tids.len(),
            observed_tids
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            "graph-first duplicate draining should not emit the same heap tid twice"
        );
        assert!(
            observed_tids
                .iter()
                .all(|heap_tid| expected_tids.contains(heap_tid)),
            "every emitted heap tid should still belong to the indexed table"
        );
        assert!(
            observed_tids.len() < expected_tids.len(),
            "this staged A3 execution path should stop after graph-ordered traversal instead of silently falling back to a full linear tail"
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

    fn run_graph_scan_recall_gate() -> Vec<(i32, i32, f32, Option<f32>, bool)> {
        let corpus = random_unit_vectors(RECALL_CORPUS_SIZE, RECALL_DIM, RECALL_SEED as u64);
        let queries = random_unit_vectors(
            RECALL_QUERY_COUNT,
            RECALL_DIM,
            (RECALL_SEED as u64) + 1_000_000,
        );
        let ground_truth = queries
            .iter()
            .map(|query| brute_force_top_k(&corpus, query, RECALL_K))
            .collect::<Vec<_>>();

        create_recall_table("tqhnsw_graph_scan_recall_gate");
        insert_recall_corpus("tqhnsw_graph_scan_recall_gate", &corpus);
        let ctid_to_id = ctid_id_map("tqhnsw_graph_scan_recall_gate");

        let mut results = Vec::new();

        for m in [8, 16] {
            let index_name = format!("tqhnsw_graph_scan_recall_gate_m{m}_idx");
            let index_oid = create_recall_index("tqhnsw_graph_scan_recall_gate", &index_name, m);

            for (config_m, ef_search, target) in RECALL_GATE_CONFIGS
                .iter()
                .copied()
                .filter(|(cfg_m, _, _)| *cfg_m == m)
            {
                let recall = measure_graph_scan_recall(
                    index_oid,
                    &ctid_to_id,
                    &queries,
                    &ground_truth,
                    ef_search,
                );
                let passed = target.map(|gate| recall >= gate).unwrap_or(true);
                results.push((config_m, ef_search, recall, target, passed));
            }

            Spi::run(&format!("DROP INDEX {index_name}"))
                .expect("recall benchmark index cleanup should succeed");
        }

        Spi::run("DROP TABLE tqhnsw_graph_scan_recall_gate")
            .expect("recall benchmark table cleanup should succeed");

        results
    }

    fn run_graph_scan_recall_gate_from_fixtures(
        fixture_prefix: &str,
        query_count: usize,
    ) -> Vec<(i32, i32, f32, Option<f32>, bool)> {
        assert!(query_count > 0, "query_count must be positive");

        let fixture_prefix = recall_fixture_ident(fixture_prefix);
        let table_name = format!("{fixture_prefix}_corpus");
        RECALL_GATE_CONFIGS
            .iter()
            .copied()
            .map(|(m, ef_search, target)| {
                let index_name = format!("{fixture_prefix}_m{m}_idx");
                let (_, _, _, graph_recall_at_10, _, _, _, _, _, _, _) =
                    probe_graph_scan_recall_fixture_summary_for_relation(
                        &table_name,
                        &index_name,
                        m,
                        ef_search,
                        query_count,
                    );
                let passed = target
                    .map(|gate| graph_recall_at_10 >= gate)
                    .unwrap_or(true);
                (m, ef_search, graph_recall_at_10, target, passed)
            })
            .collect()
    }

    type GraphScanRecallProbeRow = (i32, i32, i32, i32, bool, Vec<i64>, Vec<i64>, Vec<i64>);
    type GraphScanRecallFrontierTranscriptRow = (
        i32,
        i32,
        i32,
        i32,
        bool,
        i32,
        i32,
        Vec<i64>,
        Vec<i64>,
        Vec<i64>,
        Option<String>,
        Vec<String>,
        Vec<String>,
    );
    type GraphScanRecallProbeRanksRow = (
        i32,
        i32,
        i32,
        i32,
        bool,
        Vec<i64>,
        Vec<i64>,
        Vec<i64>,
        Vec<i32>,
        Vec<i32>,
    );
    type GraphScanRecallScoreAuditRow = (i32, i32, Vec<i64>, Vec<f32>, Vec<i32>, Vec<f32>);
    type GraphScanRecallFixtureQueryOverlapRow = (i32, i32, i32, i32, i32, i32, i32);
    type GraphScanRecallFixtureSummaryRow = (i32, i32, i32, f32, f32, f32, i32, i32, i32, i32, i32);
    type GraphScanRecallTopLevelOracleSummaryRow = (i32, i32, i32, f32, f32, f32, i32, i32, i32);
    type GraphScanRecallTopLevelOracleKSummaryRow =
        (i32, i32, i32, i32, f32, f32, f32, i32, i32, i32);
    type GraphScanRecallLayerOracleKCarrydownSummaryRow =
        (i32, i32, i32, i32, i32, f32, f32, f32, i32, i32, i32);
    type GraphScanRecallLayerNeighborCoverageSummaryRow =
        (i32, i32, i32, i32, i32, f32, f32, f32, i32, i32, i32);
    type GraphScanRecallTopLevelSeedCoverageRow = (
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
        f32,
        i32,
        Vec<i64>,
        Vec<i32>,
    );
    type GraphScanRecallExactSeedSummaryRow = (i32, i32, i32, f32, f32, f32, f32, i32, i32, i32);

    fn recall_top_k_overlap(left: &[i64], right: &[i64]) -> i32 {
        i32::try_from(left.iter().filter(|id| right.contains(id)).count())
            .expect("top-k overlap should fit into int")
    }

    fn format_heap_tid_coords((block_number, offset_number): (u32, u16)) -> String {
        format!("{block_number}:{offset_number}")
    }

    fn format_frontier_provenance_slot(
        (valid, node, source, score): (bool, (u32, u16), (u32, u16), f32),
    ) -> Option<String> {
        if !valid {
            return None;
        }

        let source = if source == (u32::MAX, u16::MAX) {
            "-".to_owned()
        } else {
            format_heap_tid_coords(source)
        };
        Some(format!(
            "{}<-{}@{score:.6}",
            format_heap_tid_coords(node),
            source
        ))
    }

    fn probe_graph_scan_recall_fixture(
        fixture_name: &str,
        m: i32,
        ef_search: i32,
        query_index: usize,
        query_count: usize,
    ) -> GraphScanRecallProbeRow {
        assert!(query_count > query_index);

        let fixture_name = recall_fixture_ident(fixture_name);
        let index_name = format!("{fixture_name}_idx");
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let corpus = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {fixture_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let query = queries
            .get(query_index)
            .expect("query index should be within the generated query set");
        let ctid_to_id = ctid_id_map(&fixture_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("recall fixture index oid query should succeed")
                .expect("recall fixture index oid should exist");
        let index_block_count =
            recall_index_block_count(index_oid, "build_graph_scan_recall_probe_with_sizes");
        let truth = brute_force_top_k(
            &random_unit_vectors(
                usize::try_from(corpus).expect("fixture corpus size should fit usize"),
                RECALL_DIM,
                RECALL_SEED as u64,
            ),
            query,
            RECALL_K,
        )
        .into_iter()
        .map(|id| i64::try_from(id).expect("truth id should fit into bigint"))
        .collect::<Vec<_>>();

        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");
        let (prefill_found, _, _, _, _, _, _, _) =
            unsafe { am::debug_gettuple_current_result_state(index_oid, query.clone()) };
        let predicted_heap_tids =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) };
        let predicted_ids = predicted_heap_tids
            .iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(heap_tid)
                        .expect("probe heap tid should map back to a benchmark row id"),
                )
                .expect("predicted id should fit into bigint")
            })
            .collect::<Vec<_>>();
        let exact_quantized_ids = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT id
                         FROM {fixture_name}
                         ORDER BY embedding <#> $1
                         LIMIT 10"
                    ),
                    None,
                    &[query.clone().into()],
                )
                .expect("exact quantized probe query should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });

        (
            m,
            ef_search,
            index_block_count,
            i32::try_from(predicted_heap_tids.len()).expect("row count should fit into int"),
            prefill_found,
            truth,
            predicted_ids,
            exact_quantized_ids,
        )
    }

    fn probe_graph_scan_recall_fixture_transcript(
        fixture_name: &str,
        m: i32,
        ef_search: i32,
        query_index: usize,
        query_count: usize,
    ) -> GraphScanRecallFrontierTranscriptRow {
        let probe =
            probe_graph_scan_recall_fixture(fixture_name, m, ef_search, query_index, query_count);
        let fixture_name = recall_fixture_ident(fixture_name);
        let index_name = format!("{fixture_name}_idx");
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("recall fixture index oid query should succeed")
                .expect("recall fixture index oid should exist");
        let query = random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000)
            .into_iter()
            .nth(query_index)
            .expect("query index should be within the generated query set");

        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");
        let (frontier_head, _, _, frontier_provenance, expanded_sources) =
            unsafe { am::debug_rescan_candidate_frontier(index_oid, query) };

        (
            probe.0,
            probe.1,
            probe.2,
            probe.3,
            probe.4,
            recall_top_k_overlap(&probe.5, &probe.6),
            recall_top_k_overlap(&probe.5, &probe.7),
            probe.5,
            probe.6,
            probe.7,
            frontier_head.map(format_heap_tid_coords),
            frontier_provenance
                .into_iter()
                .filter_map(format_frontier_provenance_slot)
                .collect::<Vec<_>>(),
            expanded_sources
                .into_iter()
                .map(format_heap_tid_coords)
                .collect::<Vec<_>>(),
        )
    }

    fn probe_graph_scan_recall_fixture_ranks(
        fixture_name: &str,
        m: i32,
        ef_search: i32,
        query_index: usize,
        query_count: usize,
    ) -> GraphScanRecallProbeRanksRow {
        assert!(query_count > query_index);

        let fixture_name = recall_fixture_ident(fixture_name);
        let index_name = format!("{fixture_name}_idx");
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let corpus = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {fixture_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let query = queries
            .get(query_index)
            .expect("query index should be within the generated query set");
        let ctid_to_id = ctid_id_map(&fixture_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("recall fixture index oid query should succeed")
                .expect("recall fixture index oid should exist");
        let index_block_count =
            recall_index_block_count(index_oid, "probe_graph_scan_recall_fixture_ranks");
        let truth = brute_force_top_k(
            &random_unit_vectors(
                usize::try_from(corpus).expect("fixture corpus size should fit usize"),
                RECALL_DIM,
                RECALL_SEED as u64,
            ),
            query,
            RECALL_K,
        )
        .into_iter()
        .map(|id| i64::try_from(id).expect("truth id should fit into bigint"))
        .collect::<Vec<_>>();

        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");
        let (prefill_found, _, _, _, _, _, _, _) =
            unsafe { am::debug_gettuple_current_result_state(index_oid, query.clone()) };
        let predicted_heap_tids =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) };
        let predicted_ids_full = predicted_heap_tids
            .iter()
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(heap_tid)
                        .expect("probe heap tid should map back to a benchmark row id"),
                )
                .expect("predicted id should fit into bigint")
            })
            .collect::<Vec<_>>();
        let predicted_top10_ids = predicted_ids_full
            .iter()
            .copied()
            .take(RECALL_K)
            .collect::<Vec<_>>();
        let exact_quantized_ids = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT id
                         FROM {fixture_name}
                         ORDER BY embedding <#> $1
                         LIMIT 10"
                    ),
                    None,
                    &[query.clone().into()],
                )
                .expect("exact quantized probe query should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });
        let truth_ranks = truth
            .iter()
            .map(|id| {
                predicted_ids_full
                    .iter()
                    .position(|candidate| candidate == id)
                    .map(|rank| i32::try_from(rank).expect("rank should fit into int"))
                    .unwrap_or(-1)
            })
            .collect::<Vec<_>>();
        let exact_ranks = exact_quantized_ids
            .iter()
            .map(|id| {
                predicted_ids_full
                    .iter()
                    .position(|candidate| candidate == id)
                    .map(|rank| i32::try_from(rank).expect("rank should fit into int"))
                    .unwrap_or(-1)
            })
            .collect::<Vec<_>>();

        (
            m,
            ef_search,
            index_block_count,
            i32::try_from(predicted_ids_full.len()).expect("row count should fit into int"),
            prefill_found,
            truth,
            predicted_top10_ids,
            exact_quantized_ids,
            truth_ranks,
            exact_ranks,
        )
    }

    fn probe_graph_scan_recall_fixture_score_audit(
        fixture_name: &str,
        m: i32,
        ef_search: i32,
        query_index: usize,
        query_count: usize,
    ) -> GraphScanRecallScoreAuditRow {
        assert!(query_count > query_index);

        let fixture_name = recall_fixture_ident(fixture_name);
        let index_name = format!("{fixture_name}_idx");
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let query = queries
            .get(query_index)
            .expect("query index should be within the generated query set")
            .clone();
        let ctid_to_id = ctid_id_map(&fixture_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("recall fixture index oid query should succeed")
                .expect("recall fixture index oid should exist");

        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");
        let predicted_with_scores =
            unsafe { am::debug_gettuple_scan_heap_tids_with_scores(index_oid, query.clone()) };
        let predicted_id_scores = predicted_with_scores
            .into_iter()
            .map(|(heap_tid, score)| {
                (
                    i64::try_from(
                        *ctid_to_id
                            .get(&heap_tid)
                            .expect("probe heap tid should map back to a benchmark row id"),
                    )
                    .expect("predicted id should fit into bigint"),
                    score,
                )
            })
            .collect::<Vec<_>>();

        let exact_ids = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT id
                         FROM {fixture_name}
                         ORDER BY embedding <#> $1
                         LIMIT 10"
                    ),
                    None,
                    &[query.clone().into()],
                )
                .expect("exact quantized probe query should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });
        let exact_scores = exact_ids
            .iter()
            .map(|id| {
                Spi::get_one::<f32>(&format!(
                    "SELECT embedding <#> ARRAY[{}]::real[] FROM {fixture_name} WHERE id = {id}",
                    query
                        .iter()
                        .map(|value| value.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ))
                .expect("exact score query should succeed")
                .expect("exact score should exist")
            })
            .collect::<Vec<_>>();
        let emitted_ranks = exact_ids
            .iter()
            .map(|id| {
                predicted_id_scores
                    .iter()
                    .position(|(candidate_id, _)| candidate_id == id)
                    .map(|rank| i32::try_from(rank).expect("rank should fit into int"))
                    .unwrap_or(-1)
            })
            .collect::<Vec<_>>();
        let emitted_scores = exact_ids
            .iter()
            .map(|id| {
                predicted_id_scores
                    .iter()
                    .find_map(|(candidate_id, score)| (*candidate_id == *id).then_some(*score))
                    .unwrap_or(f32::NAN)
            })
            .collect::<Vec<_>>();

        (
            m,
            ef_search,
            exact_ids,
            exact_scores,
            emitted_ranks,
            emitted_scores,
        )
    }

    fn collect_graph_scan_recall_fixture_query_overlaps_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
    ) -> Vec<GraphScanRecallFixtureQueryOverlapRow> {
        assert!(query_count > 0);

        let table_name = recall_fixture_ident(table_name);
        let index_name = recall_fixture_ident(index_name);
        let corpus_size = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {table_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let corpus = random_unit_vectors(
            usize::try_from(corpus_size).expect("fixture corpus size should fit usize"),
            RECALL_DIM,
            RECALL_SEED as u64,
        );
        let corpus_codes = encode_recall_corpus_codes(&corpus);
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ground_truth = queries
            .iter()
            .map(|query| brute_force_top_k(&corpus, query, RECALL_K))
            .collect::<Vec<_>>();
        let ctid_to_id = ctid_id_map(&table_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("recall fixture index oid query should succeed")
                .expect("recall fixture index oid should exist");

        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");
        let mut rows = Vec::with_capacity(query_count);

        for (query_index, (query, truth)) in queries.iter().zip(ground_truth.iter()).enumerate() {
            let truth_ids = truth
                .iter()
                .map(|id| i64::try_from(*id).expect("truth id should fit into bigint"))
                .collect::<Vec<_>>();
            let predicted_ids =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        i64::try_from(
                            *ctid_to_id
                                .get(&heap_tid)
                                .expect("probe heap tid should map back to a benchmark row id"),
                        )
                        .expect("predicted id should fit into bigint")
                    })
                    .collect::<Vec<_>>();
            let exact_quantized_ids = Spi::connect(|client| {
                client
                    .select(
                        &format!(
                            "SELECT id
                             FROM {table_name}
                             ORDER BY embedding <#> $1
                             LIMIT 10"
                        ),
                        None,
                        &[query.clone().into()],
                    )
                    .expect("exact quantized probe query should succeed")
                    .map(|row| {
                        row["id"]
                            .value::<i64>()
                            .expect("id should decode")
                            .expect("id should be non-null")
                    })
                    .collect::<Vec<_>>()
            });
            let build_code_ids = brute_force_top_k_code_inner_product(
                &corpus_codes,
                &encode_recall_query_code(query),
                RECALL_K,
            )
            .into_iter()
            .map(|id| i64::try_from(id).expect("build-code id should fit into bigint"))
            .collect::<Vec<_>>();

            let graph_overlap = recall_top_k_overlap(&truth_ids, &predicted_ids);
            let exact_overlap = recall_top_k_overlap(&truth_ids, &exact_quantized_ids);
            let build_code_overlap = recall_top_k_overlap(&truth_ids, &build_code_ids);

            rows.push((
                m,
                ef_search,
                i32::try_from(query_count).expect("query count should fit into int"),
                i32::try_from(query_index).expect("query index should fit into int"),
                graph_overlap,
                exact_overlap,
                build_code_overlap,
            ));
        }

        rows
    }

    fn collect_graph_scan_recall_fixture_query_overlaps(
        fixture_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
    ) -> Vec<GraphScanRecallFixtureQueryOverlapRow> {
        let fixture_name = recall_fixture_ident(fixture_name);
        let index_name = format!("{fixture_name}_idx");
        collect_graph_scan_recall_fixture_query_overlaps_for_relation(
            &fixture_name,
            &index_name,
            m,
            ef_search,
            query_count,
        )
    }

    fn probe_graph_scan_recall_fixture_summary(
        fixture_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
    ) -> GraphScanRecallFixtureSummaryRow {
        let rows = collect_graph_scan_recall_fixture_query_overlaps(
            fixture_name,
            m,
            ef_search,
            query_count,
        );
        summarize_graph_scan_recall_fixture_query_overlaps(rows, m, ef_search, query_count)
    }

    fn probe_graph_scan_recall_fixture_summary_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
    ) -> GraphScanRecallFixtureSummaryRow {
        let rows = collect_graph_scan_recall_fixture_query_overlaps_for_relation(
            table_name,
            index_name,
            m,
            ef_search,
            query_count,
        );
        summarize_graph_scan_recall_fixture_query_overlaps(rows, m, ef_search, query_count)
    }

    fn probe_graph_scan_recall_top_level_oracle_summary_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
    ) -> GraphScanRecallTopLevelOracleSummaryRow {
        let table_name = recall_fixture_ident(table_name);
        let index_name = recall_fixture_ident(index_name);
        let corpus = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {table_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let corpus = random_unit_vectors(
            usize::try_from(corpus).expect("fixture corpus size should fit usize"),
            RECALL_DIM,
            RECALL_SEED as u64,
        );
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ground_truth = queries
            .iter()
            .map(|query| brute_force_top_k(&corpus, query, RECALL_K))
            .collect::<Vec<_>>();
        let ctid_to_id = ctid_id_map(&table_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("oracle summary fixture index oid query should succeed")
                .expect("oracle summary fixture index oid should exist");

        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut graph_hits = 0_i32;
        let mut oracle_hits = 0_i32;
        let mut exact_hits = 0_i32;
        let mut graph_below_oracle_queries = 0_i32;
        let mut oracle_below_exact_queries = 0_i32;
        let mut worst_oracle_gap = 0_i32;

        for (query, truth) in queries.iter().zip(ground_truth.iter()) {
            let truth_ids = truth
                .iter()
                .map(|id| i64::try_from(*id).expect("truth id should fit into bigint"))
                .collect::<Vec<_>>();
            let predicted_ids =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        i64::try_from(
                            *ctid_to_id
                                .get(&heap_tid)
                                .expect("graph heap tid should map back to a benchmark row id"),
                        )
                        .expect("graph id should fit into bigint")
                    })
                    .collect::<Vec<_>>();
            let oracle_ids = unsafe {
                am::debug_top_level_oracle_scan_heap_tids(
                    index_oid,
                    query.clone(),
                    usize::try_from(ef_search).expect("ef_search should fit into usize"),
                )
            }
            .into_iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("oracle heap tid should map back to a benchmark row id"),
                )
                .expect("oracle id should fit into bigint")
            })
            .collect::<Vec<_>>();
            let exact_quantized_ids = Spi::connect(|client| {
                client
                    .select(
                        &format!(
                            "SELECT id
                             FROM {table_name}
                             ORDER BY embedding <#> $1
                             LIMIT 10"
                        ),
                        None,
                        &[query.clone().into()],
                    )
                    .expect("exact quantized oracle summary query should succeed")
                    .map(|row| {
                        row["id"]
                            .value::<i64>()
                            .expect("id should decode")
                            .expect("id should be non-null")
                    })
                    .collect::<Vec<_>>()
            });

            let graph_overlap = recall_top_k_overlap(&truth_ids, &predicted_ids);
            let oracle_overlap = recall_top_k_overlap(&truth_ids, &oracle_ids);
            let exact_overlap = recall_top_k_overlap(&truth_ids, &exact_quantized_ids);

            graph_hits += graph_overlap;
            oracle_hits += oracle_overlap;
            exact_hits += exact_overlap;

            if graph_overlap < oracle_overlap {
                graph_below_oracle_queries += 1;
                worst_oracle_gap = worst_oracle_gap.max(oracle_overlap - graph_overlap);
            }
            if oracle_overlap < exact_overlap {
                oracle_below_exact_queries += 1;
            }
        }

        let recall_denominator = (query_count as f32) * (RECALL_K as f32);
        (
            m,
            ef_search,
            i32::try_from(query_count).expect("query count should fit into int"),
            graph_hits as f32 / recall_denominator,
            oracle_hits as f32 / recall_denominator,
            exact_hits as f32 / recall_denominator,
            graph_below_oracle_queries,
            oracle_below_exact_queries,
            worst_oracle_gap,
        )
    }

    fn probe_graph_scan_recall_top_level_oracle_k_summary_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
        seed_count: usize,
    ) -> GraphScanRecallTopLevelOracleKSummaryRow {
        let table_name = recall_fixture_ident(table_name);
        let index_name = recall_fixture_ident(index_name);
        let corpus = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {table_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let corpus = random_unit_vectors(
            usize::try_from(corpus).expect("fixture corpus size should fit usize"),
            RECALL_DIM,
            RECALL_SEED as u64,
        );
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ground_truth = queries
            .iter()
            .map(|query| brute_force_top_k(&corpus, query, RECALL_K))
            .collect::<Vec<_>>();
        let ctid_to_id = ctid_id_map(&table_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("oracle-k summary fixture index oid query should succeed")
                .expect("oracle-k summary fixture index oid should exist");

        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut graph_hits = 0_i32;
        let mut oracle_hits = 0_i32;
        let mut exact_hits = 0_i32;
        let mut graph_below_oracle_queries = 0_i32;
        let mut oracle_below_exact_queries = 0_i32;
        let mut worst_oracle_gap = 0_i32;

        for (query, truth) in queries.iter().zip(ground_truth.iter()) {
            let truth_ids = truth
                .iter()
                .map(|id| i64::try_from(*id).expect("truth id should fit into bigint"))
                .collect::<Vec<_>>();
            let predicted_ids =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        i64::try_from(
                            *ctid_to_id
                                .get(&heap_tid)
                                .expect("graph heap tid should map back to a benchmark row id"),
                        )
                        .expect("graph id should fit into bigint")
                    })
                    .collect::<Vec<_>>();
            let oracle_ids = unsafe {
                am::debug_top_level_oracle_k_seed_scan_heap_tids(
                    index_oid,
                    query.clone(),
                    usize::try_from(ef_search).expect("ef_search should fit into usize"),
                    seed_count,
                )
            }
            .into_iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("oracle-k heap tid should map back to a benchmark row id"),
                )
                .expect("oracle-k id should fit into bigint")
            })
            .collect::<Vec<_>>();
            let exact_quantized_ids = Spi::connect(|client| {
                client
                    .select(
                        &format!(
                            "SELECT id
                             FROM {table_name}
                             ORDER BY embedding <#> $1
                             LIMIT 10"
                        ),
                        None,
                        &[query.clone().into()],
                    )
                    .expect("exact quantized oracle-k summary query should succeed")
                    .map(|row| {
                        row["id"]
                            .value::<i64>()
                            .expect("id should decode")
                            .expect("id should be non-null")
                    })
                    .collect::<Vec<_>>()
            });

            let graph_overlap = recall_top_k_overlap(&truth_ids, &predicted_ids);
            let oracle_overlap = recall_top_k_overlap(&truth_ids, &oracle_ids);
            let exact_overlap = recall_top_k_overlap(&truth_ids, &exact_quantized_ids);

            graph_hits += graph_overlap;
            oracle_hits += oracle_overlap;
            exact_hits += exact_overlap;

            if graph_overlap < oracle_overlap {
                graph_below_oracle_queries += 1;
                worst_oracle_gap = worst_oracle_gap.max(oracle_overlap - graph_overlap);
            }
            if oracle_overlap < exact_overlap {
                oracle_below_exact_queries += 1;
            }
        }

        let recall_denominator = (query_count as f32) * (RECALL_K as f32);
        (
            m,
            ef_search,
            i32::try_from(query_count).expect("query count should fit into int"),
            i32::try_from(seed_count).expect("seed count should fit into int"),
            graph_hits as f32 / recall_denominator,
            oracle_hits as f32 / recall_denominator,
            exact_hits as f32 / recall_denominator,
            graph_below_oracle_queries,
            oracle_below_exact_queries,
            worst_oracle_gap,
        )
    }

    fn probe_graph_scan_recall_layer_oracle_k_carrydown_summary_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
        layer: u8,
        seed_count: usize,
    ) -> GraphScanRecallLayerOracleKCarrydownSummaryRow {
        let table_name = recall_fixture_ident(table_name);
        let index_name = recall_fixture_ident(index_name);
        let corpus = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {table_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let corpus = random_unit_vectors(
            usize::try_from(corpus).expect("fixture corpus size should fit usize"),
            RECALL_DIM,
            RECALL_SEED as u64,
        );
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ground_truth = queries
            .iter()
            .map(|query| brute_force_top_k(&corpus, query, RECALL_K))
            .collect::<Vec<_>>();
        let ctid_to_id = ctid_id_map(&table_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("layer-oracle summary fixture index oid query should succeed")
                .expect("layer-oracle summary fixture index oid should exist");

        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut graph_hits = 0_i32;
        let mut oracle_hits = 0_i32;
        let mut exact_hits = 0_i32;
        let mut graph_below_oracle_queries = 0_i32;
        let mut oracle_below_exact_queries = 0_i32;
        let mut worst_oracle_gap = 0_i32;

        for (query, truth) in queries.iter().zip(ground_truth.iter()) {
            let truth_ids = truth
                .iter()
                .map(|id| i64::try_from(*id).expect("truth id should fit into bigint"))
                .collect::<Vec<_>>();
            let predicted_ids =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        i64::try_from(
                            *ctid_to_id
                                .get(&heap_tid)
                                .expect("graph heap tid should map back to a benchmark row id"),
                        )
                        .expect("graph id should fit into bigint")
                    })
                    .collect::<Vec<_>>();
            let oracle_ids = unsafe {
                am::debug_layer_oracle_k_carrydown_scan_heap_tids(
                    index_oid,
                    query.clone(),
                    usize::try_from(ef_search).expect("ef_search should fit into usize"),
                    layer,
                    seed_count,
                )
            }
            .into_iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("layer-oracle heap tid should map back to a benchmark row id"),
                )
                .expect("layer-oracle id should fit into bigint")
            })
            .collect::<Vec<_>>();
            let exact_quantized_ids = Spi::connect(|client| {
                client
                    .select(
                        &format!(
                            "SELECT id
                             FROM {table_name}
                             ORDER BY embedding <#> $1
                             LIMIT 10"
                        ),
                        None,
                        &[query.clone().into()],
                    )
                    .expect("exact quantized layer-oracle summary query should succeed")
                    .map(|row| {
                        row["id"]
                            .value::<i64>()
                            .expect("id should decode")
                            .expect("id should be non-null")
                    })
                    .collect::<Vec<_>>()
            });

            let graph_overlap = recall_top_k_overlap(&truth_ids, &predicted_ids);
            let oracle_overlap = recall_top_k_overlap(&truth_ids, &oracle_ids);
            let exact_overlap = recall_top_k_overlap(&truth_ids, &exact_quantized_ids);

            graph_hits += graph_overlap;
            oracle_hits += oracle_overlap;
            exact_hits += exact_overlap;

            if graph_overlap < oracle_overlap {
                graph_below_oracle_queries += 1;
                worst_oracle_gap = worst_oracle_gap.max(oracle_overlap - graph_overlap);
            }
            if oracle_overlap < exact_overlap {
                oracle_below_exact_queries += 1;
            }
        }

        let recall_denominator = (query_count as f32) * (RECALL_K as f32);
        (
            m,
            ef_search,
            i32::from(layer),
            i32::try_from(query_count).expect("query count should fit into int"),
            i32::try_from(seed_count).expect("seed count should fit into int"),
            graph_hits as f32 / recall_denominator,
            oracle_hits as f32 / recall_denominator,
            exact_hits as f32 / recall_denominator,
            graph_below_oracle_queries,
            oracle_below_exact_queries,
            worst_oracle_gap,
        )
    }

    fn probe_graph_scan_recall_layer_neighbor_coverage_summary_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
        layer: u8,
        seed_count: usize,
    ) -> GraphScanRecallLayerNeighborCoverageSummaryRow {
        let table_name = recall_fixture_ident(table_name);
        let index_name = recall_fixture_ident(index_name);
        let corpus = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {table_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let corpus = random_unit_vectors(
            usize::try_from(corpus).expect("fixture corpus size should fit usize"),
            RECALL_DIM,
            RECALL_SEED as u64,
        );
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ground_truth = queries
            .iter()
            .map(|query| brute_force_top_k(&corpus, query, RECALL_K))
            .collect::<Vec<_>>();
        let ctid_to_id = ctid_id_map(&table_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("layer-neighbor summary fixture index oid query should succeed")
                .expect("layer-neighbor summary fixture index oid should exist");

        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut graph_hits = 0_i32;
        let mut neighbor_hits = 0_i32;
        let mut exact_hits = 0_i32;
        let mut graph_below_neighbor_queries = 0_i32;
        let mut neighbor_below_exact_queries = 0_i32;
        let mut worst_neighbor_gap = 0_i32;

        for (query, truth) in queries.iter().zip(ground_truth.iter()) {
            let truth_ids = truth
                .iter()
                .map(|id| i64::try_from(*id).expect("truth id should fit into bigint"))
                .collect::<Vec<_>>();
            let predicted_ids =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        i64::try_from(
                            *ctid_to_id
                                .get(&heap_tid)
                                .expect("graph heap tid should map back to a benchmark row id"),
                        )
                        .expect("graph id should fit into bigint")
                    })
                    .collect::<Vec<_>>();
            let neighbor_ids = unsafe {
                am::debug_layer_oracle_k_seed_layer0_neighbor_heap_tids(
                    index_oid,
                    query.clone(),
                    layer,
                    seed_count,
                )
            }
            .into_iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("layer-neighbor heap tid should map back to a benchmark row id"),
                )
                .expect("layer-neighbor id should fit into bigint")
            })
            .collect::<Vec<_>>();
            let exact_quantized_ids = Spi::connect(|client| {
                client
                    .select(
                        &format!(
                            "SELECT id
                             FROM {table_name}
                             ORDER BY embedding <#> $1
                             LIMIT 10"
                        ),
                        None,
                        &[query.clone().into()],
                    )
                    .expect("exact quantized layer-neighbor summary query should succeed")
                    .map(|row| {
                        row["id"]
                            .value::<i64>()
                            .expect("id should decode")
                            .expect("id should be non-null")
                    })
                    .collect::<Vec<_>>()
            });

            let graph_overlap = recall_top_k_overlap(&truth_ids, &predicted_ids);
            let neighbor_overlap = recall_top_k_overlap(&truth_ids, &neighbor_ids);
            let exact_overlap = recall_top_k_overlap(&truth_ids, &exact_quantized_ids);

            graph_hits += graph_overlap;
            neighbor_hits += neighbor_overlap;
            exact_hits += exact_overlap;

            if graph_overlap < neighbor_overlap {
                graph_below_neighbor_queries += 1;
                worst_neighbor_gap = worst_neighbor_gap.max(neighbor_overlap - graph_overlap);
            }
            if neighbor_overlap < exact_overlap {
                neighbor_below_exact_queries += 1;
            }
        }

        let recall_denominator = (query_count as f32) * (RECALL_K as f32);
        (
            m,
            ef_search,
            i32::from(layer),
            i32::try_from(query_count).expect("query count should fit into int"),
            i32::try_from(seed_count).expect("seed count should fit into int"),
            graph_hits as f32 / recall_denominator,
            neighbor_hits as f32 / recall_denominator,
            exact_hits as f32 / recall_denominator,
            graph_below_neighbor_queries,
            neighbor_below_exact_queries,
            worst_neighbor_gap,
        )
    }

    fn probe_graph_scan_recall_top_level_seed_coverage_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
        seed_count: usize,
    ) -> GraphScanRecallTopLevelSeedCoverageRow {
        let table_name = recall_fixture_ident(table_name);
        let index_name = recall_fixture_ident(index_name);
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ctid_to_id = ctid_id_map(&table_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("seed coverage fixture index oid query should succeed")
                .expect("seed coverage fixture index oid should exist");

        let all_top_level_ids = unsafe { am::debug_all_top_level_heap_tids(index_oid) }
            .into_iter()
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("top-level heap tid should map back to a benchmark row id"),
                )
                .expect("top-level id should fit into bigint")
            })
            .collect::<std::collections::HashSet<_>>();
        let reachable_top_level_ids = unsafe { am::debug_top_level_reachable_heap_tids(index_oid) }
            .into_iter()
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("reachable heap tid should map back to a benchmark row id"),
                )
                .expect("reachable id should fit into bigint")
            })
            .collect::<std::collections::HashSet<_>>();

        let mut oracle_seed_frequency = std::collections::HashMap::<i64, i32>::new();
        let mut reachable_seed_slots = 0_i32;
        let mut total_seed_slots = 0_i32;
        let mut fully_reachable_queries = 0_i32;

        for query in &queries {
            let oracle_seed_ids = unsafe {
                am::debug_top_level_oracle_k_seed_heap_tids(index_oid, query.clone(), seed_count)
            }
            .into_iter()
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("oracle seed heap tid should map back to a benchmark row id"),
                )
                .expect("oracle seed id should fit into bigint")
            })
            .collect::<Vec<_>>();

            total_seed_slots +=
                i32::try_from(oracle_seed_ids.len()).expect("oracle seed slot count should fit");
            let reachable_for_query = oracle_seed_ids
                .iter()
                .filter(|id| reachable_top_level_ids.contains(id))
                .count();
            reachable_seed_slots +=
                i32::try_from(reachable_for_query).expect("reachable query count should fit");
            if reachable_for_query == oracle_seed_ids.len() {
                fully_reachable_queries += 1;
            }
            for id in oracle_seed_ids {
                *oracle_seed_frequency.entry(id).or_insert(0) += 1;
            }
        }

        let unique_oracle_seed_ids =
            i32::try_from(oracle_seed_frequency.len()).expect("unique oracle seed ids should fit");
        let reachable_unique_oracle_seed_ids = i32::try_from(
            oracle_seed_frequency
                .keys()
                .filter(|id| reachable_top_level_ids.contains(id))
                .count(),
        )
        .expect("reachable unique oracle seed ids should fit");
        let mut frequent_oracle_seeds = oracle_seed_frequency.into_iter().collect::<Vec<_>>();
        frequent_oracle_seeds.sort_by(|left, right| {
            right
                .1
                .cmp(&left.1)
                .then_with(|| left.0.cmp(&right.0))
        });
        frequent_oracle_seeds.truncate(seed_count.max(10));
        let (top_seed_ids, top_seed_query_counts): (Vec<_>, Vec<_>) =
            frequent_oracle_seeds.into_iter().unzip();

        (
            m,
            ef_search,
            i32::try_from(query_count).expect("query count should fit into int"),
            i32::try_from(seed_count).expect("seed count should fit into int"),
            i32::try_from(all_top_level_ids.len()).expect("top-level node count should fit"),
            i32::try_from(reachable_top_level_ids.len())
                .expect("reachable top-level node count should fit"),
            unique_oracle_seed_ids,
            reachable_unique_oracle_seed_ids,
            if total_seed_slots == 0 {
                0.0
            } else {
                reachable_seed_slots as f32 / total_seed_slots as f32
            },
            fully_reachable_queries,
            top_seed_ids,
            top_seed_query_counts,
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_hierarchy_summary(
        index_oid: pg_sys::Oid,
    ) -> TableIterator<
        'static,
        (
            name!(level, i32),
            name!(node_count, i32),
            name!(avg_neighbor_count, f64),
            name!(min_neighbor_count, i32),
            name!(max_neighbor_count, i32),
            name!(expected_max_neighbors, i32),
        ),
    > {
        let index_relation =
            unsafe { open_valid_tqhnsw_index(index_oid, "tqhnsw_graph_hierarchy_summary") };
        unsafe {
            pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        }

        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let m = metadata.m as usize;
        let code_len = code_len(metadata.dimensions as usize, metadata.bits);

        let neighbor_map: HashMap<am::page::ItemPointer, am::page::TqNeighborTuple> = data_pages
            .iter()
            .flat_map(|page| {
                page.tuples.iter().enumerate().filter_map(move |(idx, tuple)| {
                    if tuple.first().copied() == Some(am::page::TQ_NEIGHBOR_TAG) {
                        Some((
                            am::page::ItemPointer {
                                block_number: page.block_number,
                                offset_number: (idx + 1) as u16,
                            },
                            am::page::TqNeighborTuple::decode(tuple)
                                .expect("neighbor tuple should decode"),
                        ))
                    } else {
                        None
                    }
                })
            })
            .collect();

        struct LevelStats {
            node_count: usize,
            total_neighbors: usize,
            min_neighbors: usize,
            max_neighbors: usize,
        }

        let mut level_stats: HashMap<u8, LevelStats> = HashMap::new();

        for page in &data_pages {
            for tuple_bytes in &page.tuples {
                if tuple_bytes.first().copied() != Some(am::page::TQ_ELEMENT_TAG) {
                    continue;
                }

                let element = am::page::TqElementTuple::decode(tuple_bytes, code_len)
                    .expect("element tuple should decode");
                let neighbor = neighbor_map
                    .get(&element.neighbortid)
                    .expect("element neighbor TID should resolve");

                for layer in 0..=element.level {
                    let (start, end) = if layer == 0 {
                        (0, m * 2)
                    } else {
                        let start = m * 2 + (usize::from(layer) - 1) * m;
                        (start, start + m)
                    };

                    let valid_count = neighbor
                        .tids
                        .iter()
                        .skip(start)
                        .take(end.saturating_sub(start))
                        .filter(|tid| **tid != am::page::ItemPointer::INVALID)
                        .count();

                    let stats = level_stats.entry(layer).or_insert(LevelStats {
                        node_count: 0,
                        total_neighbors: 0,
                        min_neighbors: usize::MAX,
                        max_neighbors: 0,
                    });
                    stats.node_count += 1;
                    stats.total_neighbors += valid_count;
                    if valid_count < stats.min_neighbors {
                        stats.min_neighbors = valid_count;
                    }
                    if valid_count > stats.max_neighbors {
                        stats.max_neighbors = valid_count;
                    }
                }
            }
        }

        let mut rows: Vec<(i32, i32, f64, i32, i32, i32)> = level_stats
            .iter()
            .map(|(&level, stats)| {
                let avg = if stats.node_count > 0 {
                    stats.total_neighbors as f64 / stats.node_count as f64
                } else {
                    0.0
                };
                let expected_max = if level == 0 { (m * 2) as i32 } else { m as i32 };
                (
                    i32::from(level),
                    stats.node_count as i32,
                    avg,
                    stats.min_neighbors as i32,
                    stats.max_neighbors as i32,
                    expected_max,
                )
            })
            .collect();
        rows.sort_by_key(|(level, _, _, _, _, _)| *level);

        TableIterator::new(rows)
    }

    fn id_heap_tid_map(table_name: &str) -> HashMap<i64, (u32, u16)> {
        Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT
                            split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                            split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number,
                            id
                         FROM {table_name}"
                    ),
                    None,
                    &[],
                )
                .expect("id/ctid map query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    let id = row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null");
                    (
                        id,
                        (
                            u32::try_from(block_number)
                                .expect("block number should be non-negative"),
                            u16::try_from(offset_number)
                                .expect("offset number should be positive"),
                        ),
                    )
                })
                .collect::<HashMap<_, _>>()
        })
    }

    fn probe_graph_scan_recall_exact_seed_summary_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
    ) -> GraphScanRecallExactSeedSummaryRow {
        let table_name = recall_fixture_ident(table_name);
        let index_name = recall_fixture_ident(index_name);
        let corpus = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {table_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let corpus = random_unit_vectors(
            usize::try_from(corpus).expect("fixture corpus size should fit usize"),
            RECALL_DIM,
            RECALL_SEED as u64,
        );
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ground_truth = queries
            .iter()
            .map(|query| brute_force_top_k(&corpus, query, RECALL_K))
            .collect::<Vec<_>>();
        let ctid_to_id = ctid_id_map(&table_name);
        let id_to_heap_tid = id_heap_tid_map(&table_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("exact-seed fixture index oid query should succeed")
                .expect("exact-seed fixture index oid should exist");

        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut graph_hits = 0_i32;
        let mut exact_seed1_hits = 0_i32;
        let mut exact_seed10_hits = 0_i32;
        let mut exact_hits = 0_i32;
        let mut graph_below_exact_seed10_queries = 0_i32;
        let mut exact_seed10_below_exact_queries = 0_i32;
        let mut worst_exact_seed10_gap = 0_i32;

        for (query, truth) in queries.iter().zip(ground_truth.iter()) {
            let truth_ids = truth
                .iter()
                .map(|id| i64::try_from(*id).expect("truth id should fit into bigint"))
                .collect::<Vec<_>>();
            let predicted_ids =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        i64::try_from(
                            *ctid_to_id
                                .get(&heap_tid)
                                .expect("graph heap tid should map back to a benchmark row id"),
                        )
                        .expect("graph id should fit into bigint")
                    })
                    .collect::<Vec<_>>();
            let exact_quantized_ids = Spi::connect(|client| {
                client
                    .select(
                        &format!(
                            "SELECT id
                             FROM {table_name}
                             ORDER BY embedding <#> $1
                             LIMIT 10"
                        ),
                        None,
                        &[query.to_vec().into()],
                    )
                    .expect("exact quantized exact-seed summary query should succeed")
                    .map(|row| {
                        row["id"]
                            .value::<i64>()
                            .expect("id should decode")
                            .expect("id should be non-null")
                    })
                    .collect::<Vec<_>>()
            });
            let exact_seed_heap_tids = exact_quantized_ids
                .iter()
                .filter_map(|id| id_to_heap_tid.get(id))
                .copied()
                .collect::<Vec<_>>();
            let exact_seed1_input = exact_seed_heap_tids
                .iter()
                .copied()
                .take(1)
                .collect::<Vec<_>>();
            let exact_seed1_ids = unsafe {
                am::debug_exact_seed_scan_heap_tids(
                    index_oid,
                    query.clone(),
                    exact_seed1_input,
                    usize::try_from(ef_search).expect("ef_search should fit into usize"),
                )
            }
            .into_iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("exact-seed1 heap tid should map back to a benchmark row id"),
                )
                .expect("exact-seed1 id should fit into bigint")
            })
            .collect::<Vec<_>>();
            let exact_seed10_ids = unsafe {
                am::debug_exact_seed_scan_heap_tids(
                    index_oid,
                    query.clone(),
                    exact_seed_heap_tids,
                    usize::try_from(ef_search).expect("ef_search should fit into usize"),
                )
            }
            .into_iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("exact-seed10 heap tid should map back to a benchmark row id"),
                )
                .expect("exact-seed10 id should fit into bigint")
            })
            .collect::<Vec<_>>();

            let graph_overlap = recall_top_k_overlap(&truth_ids, &predicted_ids);
            let exact_seed1_overlap = recall_top_k_overlap(&truth_ids, &exact_seed1_ids);
            let exact_seed10_overlap = recall_top_k_overlap(&truth_ids, &exact_seed10_ids);
            let exact_overlap = recall_top_k_overlap(&truth_ids, &exact_quantized_ids);

            graph_hits += graph_overlap;
            exact_seed1_hits += exact_seed1_overlap;
            exact_seed10_hits += exact_seed10_overlap;
            exact_hits += exact_overlap;

            if graph_overlap < exact_seed10_overlap {
                graph_below_exact_seed10_queries += 1;
                worst_exact_seed10_gap =
                    worst_exact_seed10_gap.max(exact_seed10_overlap - graph_overlap);
            }
            if exact_seed10_overlap < exact_overlap {
                exact_seed10_below_exact_queries += 1;
            }
        }

        let recall_denominator = (query_count as f32) * (RECALL_K as f32);
        (
            m,
            ef_search,
            i32::try_from(query_count).expect("query count should fit into int"),
            graph_hits as f32 / recall_denominator,
            exact_seed1_hits as f32 / recall_denominator,
            exact_seed10_hits as f32 / recall_denominator,
            exact_hits as f32 / recall_denominator,
            graph_below_exact_seed10_queries,
            exact_seed10_below_exact_queries,
            worst_exact_seed10_gap,
        )
    }

    fn summarize_graph_scan_recall_fixture_query_overlaps(
        rows: Vec<GraphScanRecallFixtureQueryOverlapRow>,
        m: i32,
        ef_search: i32,
        query_count: usize,
    ) -> GraphScanRecallFixtureSummaryRow {
        let mut graph_hits = 0_i32;
        let mut exact_hits = 0_i32;
        let mut build_code_hits = 0_i32;
        let mut graph_below_exact_queries = 0_i32;
        let mut graph_below_build_code_queries = 0_i32;
        let mut build_code_below_exact_queries = 0_i32;
        let mut worst_exact_gap = 0_i32;
        let mut worst_build_code_gap = 0_i32;

        for (_, _, _, _, graph_overlap, exact_overlap, build_code_overlap) in &rows {
            graph_hits += *graph_overlap;
            exact_hits += *exact_overlap;
            build_code_hits += *build_code_overlap;

            if *graph_overlap < *exact_overlap {
                graph_below_exact_queries += 1;
                worst_exact_gap = worst_exact_gap.max(*exact_overlap - *graph_overlap);
            }
            if *graph_overlap < *build_code_overlap {
                graph_below_build_code_queries += 1;
                worst_build_code_gap =
                    worst_build_code_gap.max(*build_code_overlap - *graph_overlap);
            }
            if *build_code_overlap < *exact_overlap {
                build_code_below_exact_queries += 1;
            }
        }

        let recall_denominator = (query_count as f32) * (RECALL_K as f32);
        (
            m,
            ef_search,
            i32::try_from(query_count).expect("query count should fit into int"),
            graph_hits as f32 / recall_denominator,
            exact_hits as f32 / recall_denominator,
            build_code_hits as f32 / recall_denominator,
            graph_below_exact_queries,
            graph_below_build_code_queries,
            build_code_below_exact_queries,
            worst_exact_gap,
            worst_build_code_gap,
        )
    }

    fn build_graph_scan_recall_probe_with_sizes(
        m: i32,
        ef_search: i32,
        query_index: usize,
        corpus_size: usize,
        query_count: usize,
    ) -> GraphScanRecallProbeRow {
        assert!(corpus_size >= RECALL_K);
        assert!(query_count > query_index);

        let corpus = random_unit_vectors(corpus_size, RECALL_DIM, RECALL_SEED as u64);
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let query = queries
            .get(query_index)
            .expect("query index should be within the generated query set");
        let truth = brute_force_top_k(&corpus, query, RECALL_K)
            .into_iter()
            .map(|id| i64::try_from(id).expect("truth id should fit into bigint"))
            .collect::<Vec<_>>();

        create_recall_table("tqhnsw_graph_scan_recall_probe");
        insert_recall_corpus("tqhnsw_graph_scan_recall_probe", &corpus);
        let ctid_to_id = ctid_id_map("tqhnsw_graph_scan_recall_probe");
        let index_oid = create_recall_index(
            "tqhnsw_graph_scan_recall_probe",
            "tqhnsw_graph_scan_recall_probe_idx",
            m,
        );
        let index_relation =
            unsafe { open_valid_tqhnsw_index(index_oid, "tqhnsw_graph_scan_recall_probe") };
        let index_block_count = unsafe {
            i32::try_from(pg_sys::RelationGetNumberOfBlocksInFork(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
            ))
            .expect("block count should fit into int")
        };
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");
        let (prefill_found, _, _, _, _, _, _, _) =
            unsafe { am::debug_gettuple_current_result_state(index_oid, query.clone()) };
        let predicted_heap_tids =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) };
        let predicted_ids = predicted_heap_tids
            .iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(heap_tid)
                        .expect("probe heap tid should map back to a benchmark row id"),
                )
                .expect("predicted id should fit into bigint")
            })
            .collect::<Vec<_>>();
        let exact_quantized_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id
                     FROM tqhnsw_graph_scan_recall_probe
                     ORDER BY embedding <#> $1
                     LIMIT 10",
                    None,
                    &[query.clone().into()],
                )
                .expect("exact quantized probe query should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });

        Spi::run("DROP INDEX tqhnsw_graph_scan_recall_probe_idx")
            .expect("probe index cleanup should succeed");
        Spi::run("DROP TABLE tqhnsw_graph_scan_recall_probe")
            .expect("probe table cleanup should succeed");

        (
            m,
            ef_search,
            index_block_count,
            i32::try_from(predicted_heap_tids.len()).expect("row count should fit into int"),
            prefill_found,
            truth,
            predicted_ids,
            exact_quantized_ids,
        )
    }

    fn build_graph_scan_recall_probe(
        m: i32,
        ef_search: i32,
        query_index: usize,
    ) -> GraphScanRecallProbeRow {
        build_graph_scan_recall_probe_with_sizes(
            m,
            ef_search,
            query_index,
            RECALL_CORPUS_SIZE,
            RECALL_QUERY_COUNT,
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_gate_report() -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(recall_at_10, f32),
            name!(gate_recall_at_10, Option<f32>),
            name!(passes_gate, bool),
        ),
    > {
        TableIterator::new(run_graph_scan_recall_gate())
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_fixture_gate_reset(
        fixture_prefix: String,
        corpus_size: i32,
    ) -> TableIterator<'static, (name!(m, i32), name!(index_block_count, i32))> {
        TableIterator::new(reset_graph_scan_recall_gate_fixtures(
            &fixture_prefix,
            usize::try_from(corpus_size).expect("corpus size should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_fixture_gate_source_build_reset(
        fixture_prefix: String,
        corpus_size: i32,
    ) -> TableIterator<'static, (name!(m, i32), name!(index_block_count, i32))> {
        TableIterator::new(reset_graph_scan_recall_gate_source_fixtures(
            &fixture_prefix,
            usize::try_from(corpus_size).expect("corpus size should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_fixture_gate_report(
        fixture_prefix: String,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(recall_at_10, f32),
            name!(gate_recall_at_10, Option<f32>),
            name!(passes_gate, bool),
        ),
    > {
        TableIterator::new(run_graph_scan_recall_gate_from_fixtures(
            &fixture_prefix,
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_probe(
        m: i32,
        ef_search: i32,
        query_index: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(index_block_count, i32),
            name!(predicted_count, i32),
            name!(prefill_found, bool),
            name!(truth_top10_ids, Vec<i64>),
            name!(predicted_top10_ids, Vec<i64>),
            name!(exact_quantized_top10_ids, Vec<i64>),
        ),
    > {
        TableIterator::once(build_graph_scan_recall_probe(
            m,
            ef_search,
            usize::try_from(query_index).expect("query index should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_probe_sized(
        m: i32,
        ef_search: i32,
        query_index: i32,
        corpus_size: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(index_block_count, i32),
            name!(predicted_count, i32),
            name!(prefill_found, bool),
            name!(truth_top10_ids, Vec<i64>),
            name!(predicted_top10_ids, Vec<i64>),
            name!(exact_quantized_top10_ids, Vec<i64>),
        ),
    > {
        TableIterator::once(build_graph_scan_recall_probe_with_sizes(
            m,
            ef_search,
            usize::try_from(query_index).expect("query index should be non-negative"),
            usize::try_from(corpus_size).expect("corpus size should be non-negative"),
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    fn tqhnsw_graph_scan_recall_fixture_reset(
        fixture_name: String,
        m: i32,
        corpus_size: i32,
    ) -> i32 {
        reset_graph_scan_recall_fixture(
            &fixture_name,
            m,
            usize::try_from(corpus_size).expect("corpus size should be non-negative"),
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_fixture_probe(
        fixture_name: String,
        m: i32,
        ef_search: i32,
        query_index: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(index_block_count, i32),
            name!(predicted_count, i32),
            name!(prefill_found, bool),
            name!(truth_top10_ids, Vec<i64>),
            name!(predicted_top10_ids, Vec<i64>),
            name!(exact_quantized_top10_ids, Vec<i64>),
        ),
    > {
        TableIterator::once(probe_graph_scan_recall_fixture(
            &fixture_name,
            m,
            ef_search,
            usize::try_from(query_index).expect("query index should be non-negative"),
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_fixture_transcript(
        fixture_name: String,
        m: i32,
        ef_search: i32,
        query_index: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(index_block_count, i32),
            name!(predicted_count, i32),
            name!(prefill_found, bool),
            name!(graph_overlap, i32),
            name!(exact_overlap, i32),
            name!(truth_top10_ids, Vec<i64>),
            name!(predicted_top10_ids, Vec<i64>),
            name!(exact_quantized_top10_ids, Vec<i64>),
            name!(frontier_head, Option<String>),
            name!(frontier_provenance, Vec<String>),
            name!(expanded_sources, Vec<String>),
        ),
    > {
        TableIterator::once(probe_graph_scan_recall_fixture_transcript(
            &fixture_name,
            m,
            ef_search,
            usize::try_from(query_index).expect("query index should be non-negative"),
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_fixture_ranks(
        fixture_name: String,
        m: i32,
        ef_search: i32,
        query_index: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(index_block_count, i32),
            name!(predicted_count, i32),
            name!(prefill_found, bool),
            name!(truth_top10_ids, Vec<i64>),
            name!(predicted_top10_ids, Vec<i64>),
            name!(exact_quantized_top10_ids, Vec<i64>),
            name!(truth_ranks_in_predicted, Vec<i32>),
            name!(exact_ranks_in_predicted, Vec<i32>),
        ),
    > {
        TableIterator::once(probe_graph_scan_recall_fixture_ranks(
            &fixture_name,
            m,
            ef_search,
            usize::try_from(query_index).expect("query index should be non-negative"),
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_fixture_score_audit(
        fixture_name: String,
        m: i32,
        ef_search: i32,
        query_index: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(exact_quantized_top10_ids, Vec<i64>),
            name!(exact_quantized_scores, Vec<f32>),
            name!(emitted_ranks, Vec<i32>),
            name!(emitted_scores, Vec<f32>),
        ),
    > {
        TableIterator::once(probe_graph_scan_recall_fixture_score_audit(
            &fixture_name,
            m,
            ef_search,
            usize::try_from(query_index).expect("query index should be non-negative"),
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_fixture_summary(
        fixture_name: String,
        m: i32,
        ef_search: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(query_count, i32),
            name!(graph_recall_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(build_code_recall_at_10, f32),
            name!(graph_below_exact_queries, i32),
            name!(graph_below_build_code_queries, i32),
            name!(build_code_below_exact_queries, i32),
            name!(worst_exact_gap, i32),
            name!(worst_build_code_gap, i32),
        ),
    > {
        TableIterator::once(probe_graph_scan_recall_fixture_summary(
            &fixture_name,
            m,
            ef_search,
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_top_level_oracle_summary_rel(
        table_name: String,
        index_name: String,
        m: i32,
        ef_search: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(query_count, i32),
            name!(graph_recall_at_10, f32),
            name!(oracle_top_level_recall_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(graph_below_oracle_queries, i32),
            name!(oracle_below_exact_queries, i32),
            name!(worst_oracle_gap, i32),
        ),
    > {
        TableIterator::once(
            probe_graph_scan_recall_top_level_oracle_summary_for_relation(
                &table_name,
                &index_name,
                m,
                ef_search,
                usize::try_from(query_count).expect("query count should be non-negative"),
            ),
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_top_level_oracle_k_summary_rel(
        table_name: String,
        index_name: String,
        m: i32,
        ef_search: i32,
        query_count: i32,
        seed_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(query_count, i32),
            name!(seed_count, i32),
            name!(graph_recall_at_10, f32),
            name!(oracle_top_level_k_recall_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(graph_below_oracle_queries, i32),
            name!(oracle_below_exact_queries, i32),
            name!(worst_oracle_gap, i32),
        ),
    > {
        TableIterator::once(
            probe_graph_scan_recall_top_level_oracle_k_summary_for_relation(
                &table_name,
                &index_name,
                m,
                ef_search,
                usize::try_from(query_count).expect("query count should be non-negative"),
                usize::try_from(seed_count).expect("seed count should be non-negative"),
            ),
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_layer_oracle_k_carrydown_summary_rel(
        table_name: String,
        index_name: String,
        m: i32,
        ef_search: i32,
        layer: i32,
        query_count: i32,
        seed_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(layer, i32),
            name!(query_count, i32),
            name!(seed_count, i32),
            name!(graph_recall_at_10, f32),
            name!(oracle_layer_k_carrydown_recall_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(graph_below_oracle_queries, i32),
            name!(oracle_below_exact_queries, i32),
            name!(worst_oracle_gap, i32),
        ),
    > {
        TableIterator::once(
            probe_graph_scan_recall_layer_oracle_k_carrydown_summary_for_relation(
                &table_name,
                &index_name,
                m,
                ef_search,
                usize::try_from(query_count).expect("query count should be non-negative"),
                u8::try_from(layer).expect("layer should fit in u8"),
                usize::try_from(seed_count).expect("seed count should be non-negative"),
            ),
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_layer_neighbor_coverage_summary_rel(
        table_name: String,
        index_name: String,
        m: i32,
        ef_search: i32,
        layer: i32,
        query_count: i32,
        seed_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(layer, i32),
            name!(query_count, i32),
            name!(seed_count, i32),
            name!(graph_recall_at_10, f32),
            name!(oracle_seed_layer0_neighbor_recall_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(graph_below_neighbor_queries, i32),
            name!(neighbor_below_exact_queries, i32),
            name!(worst_neighbor_gap, i32),
        ),
    > {
        TableIterator::once(
            probe_graph_scan_recall_layer_neighbor_coverage_summary_for_relation(
                &table_name,
                &index_name,
                m,
                ef_search,
                usize::try_from(query_count).expect("query count should be non-negative"),
                u8::try_from(layer).expect("layer should fit in u8"),
                usize::try_from(seed_count).expect("seed count should be non-negative"),
            ),
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_top_level_seed_coverage_rel(
        table_name: String,
        index_name: String,
        m: i32,
        ef_search: i32,
        query_count: i32,
        seed_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(query_count, i32),
            name!(seed_count, i32),
            name!(top_level_node_count, i32),
            name!(reachable_top_level_node_count, i32),
            name!(unique_oracle_seed_id_count, i32),
            name!(reachable_unique_oracle_seed_id_count, i32),
            name!(reachable_oracle_seed_slot_fraction, f32),
            name!(fully_reachable_queries, i32),
            name!(top_oracle_seed_ids, Vec<i64>),
            name!(top_oracle_seed_query_counts, Vec<i32>),
        ),
    > {
        TableIterator::once(
            probe_graph_scan_recall_top_level_seed_coverage_for_relation(
                &table_name,
                &index_name,
                m,
                ef_search,
                usize::try_from(query_count).expect("query count should be non-negative"),
                usize::try_from(seed_count).expect("seed count should be non-negative"),
            ),
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_exact_seed_summary_rel(
        table_name: String,
        index_name: String,
        m: i32,
        ef_search: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(query_count, i32),
            name!(graph_recall_at_10, f32),
            name!(exact_seed1_recall_at_10, f32),
            name!(exact_seed10_recall_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(graph_below_exact_seed10_queries, i32),
            name!(exact_seed10_below_exact_queries, i32),
            name!(worst_exact_seed10_gap, i32),
        ),
    > {
        TableIterator::once(probe_graph_scan_recall_exact_seed_summary_for_relation(
            &table_name,
            &index_name,
            m,
            ef_search,
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_fixture_query_overlaps(
        fixture_name: String,
        m: i32,
        ef_search: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(query_count, i32),
            name!(query_index, i32),
            name!(graph_overlap, i32),
            name!(exact_overlap, i32),
            name!(build_code_overlap, i32),
        ),
    > {
        TableIterator::new(collect_graph_scan_recall_fixture_query_overlaps(
            &fixture_name,
            m,
            ef_search,
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_test]
    fn test_tqhnsw_graph_scan_recall_gate() {
        if std::env::var_os("TQVECTOR_RUN_RECALL_GATE").is_none() {
            return;
        }

        let results = run_graph_scan_recall_gate();
        let gate_recall = results
            .iter()
            .find(|(m, ef_search, _, _, _)| *m == 8 && *ef_search == 128)
            .map(|(_, _, recall, _, _)| *recall)
            .expect("A4 gate config should have been measured");

        assert!(
            gate_recall >= 0.89,
            "A4 recall gate failed: Recall@10 at m=8 ef=128 was {:.2}% (required >= 89%)",
            gate_recall * 100.0
        );
    }

    #[pg_test]
    #[ignore]
    fn test_tqhnsw_graph_scan_recall_fixture_summary_1k_tiled_fwht() {
        let fixture_name = "tqhnsw_graph_scan_recall_tiled_1k";
        let index_blocks = reset_graph_scan_recall_fixture(fixture_name, 8, 1_000);
        let (
            _m,
            _ef_search,
            query_count,
            graph_recall_at_10,
            exact_quantized_recall_at_10,
            build_code_recall_at_10,
            graph_below_exact_queries,
            graph_below_build_code_queries,
            build_code_below_exact_queries,
            worst_exact_gap,
            worst_build_code_gap,
        ) = probe_graph_scan_recall_fixture_summary(fixture_name, 8, 128, 50);

        println!(
            "1k tiled fixture: blocks={index_blocks} queries={query_count} graph={graph_recall_at_10:.4} exact={exact_quantized_recall_at_10:.4} build_code={build_code_recall_at_10:.4} graph_below_exact={graph_below_exact_queries} graph_below_build_code={graph_below_build_code_queries} build_code_below_exact={build_code_below_exact_queries} worst_exact_gap={worst_exact_gap} worst_build_code_gap={worst_build_code_gap}"
        );

        assert!(
            exact_quantized_recall_at_10 >= 0.70,
            "expected tiled 1536 quantizer path to keep exact Recall@10 above 70% on the 1k fixture, got {:.2}%",
            exact_quantized_recall_at_10 * 100.0
        );
        assert!(
            graph_recall_at_10 >= 0.70,
            "expected live graph-first Recall@10 above 70% on the 1k tiled fixture, got {:.2}% (exact {:.2}%, build-code {:.2}%)",
            graph_recall_at_10 * 100.0,
            exact_quantized_recall_at_10 * 100.0,
            build_code_recall_at_10 * 100.0
        );
    }

    #[pg_test]
    #[ignore]
    fn test_tqhnsw_graph_scan_recall_fixture_gate_10k_tiled_fwht() {
        let fixture_prefix = "tqhnsw_graph_scan_recall_gate_tiled_10k";

        let reset_started = Instant::now();
        let reset_rows = reset_graph_scan_recall_gate_fixtures(fixture_prefix, 10_000);
        let reset_elapsed = reset_started.elapsed();

        let first_started = Instant::now();
        let first = run_graph_scan_recall_gate_from_fixtures(fixture_prefix, 100);
        let first_elapsed = first_started.elapsed();

        let second_started = Instant::now();
        let second = run_graph_scan_recall_gate_from_fixtures(fixture_prefix, 100);
        let second_elapsed = second_started.elapsed();

        println!(
            "10k fixture gate reuse: reset={reset_elapsed:?} fixtures={reset_rows:?} first={first_elapsed:?} second={second_elapsed:?} results={first:?}"
        );

        assert_eq!(
            first, second,
            "fixture-backed gate report should be stable across reruns"
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
