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
        unpack_mse_indices, unpack_qjl_signs, EncodedTq, Int8ApproxNoQjl4BitQuery, PreparedQuery,
        ProdQuantizer,
    };

    // Hadamard
    pub use crate::quant::hadamard::{fwht_in_place, orthonormal_fwht_in_place};
    pub fn simd_backend() -> &'static str {
        crate::quant::simd_backend_name()
    }

    // Rotation
    pub use crate::quant::rotation::{inverse_srht, pad_input, sign_vector, srht, transform_dim};

    // Codebook
    pub use crate::quant::codebook::{beta_pdf, lloyd_max};
    pub use crate::quant::grouped_pq::{
        build_grouped_pq_lut_f32, grouped_pq_nibble, grouped_pq_score_f32, pack_grouped_pq_nibbles,
        GROUPED_PQ_CENTROIDS,
    };

    // MSE
    pub use crate::quant::mse::{decode_indices, nearest_centroid_index, quantize_to_indices};

    // QJL
    pub use crate::quant::qjl::{decode_mse_only, qjl_project};

    // Page codec
    pub use crate::am::page::{
        neighbor_slots, neighbor_tuple_encoded_len, CurrentFormatMetadata, DataPage, DataPageChain,
        ItemPointer, MetadataPage, TqElementTuple, TqNeighborTuple, HEAPTID_INLINE_CAPACITY,
        ITEM_POINTER_BYTES, PAGE_HEADER_BYTES,
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
fn tqhnsw_index_admin_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(block_count, i64),
        name!(total_live_nodes, i64),
        name!(inserted_since_rebuild, i64),
        name!(insert_drift_fraction, f64),
        name!(relation_ef_search, i32),
        name!(session_ef_search, Option<i32>),
        name!(effective_ef_search, i32),
        name!(effective_source, String),
        name!(planner_scan_enabled, bool),
    ),
> {
    let index_relation =
        unsafe { open_valid_tqhnsw_index(index_oid, "tqhnsw_index_admin_snapshot") };
    let snapshot = unsafe { am::index_admin_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::from(snapshot.block_count),
        i64::try_from(snapshot.total_live_nodes).expect("total live nodes should fit in i64"),
        i64::try_from(snapshot.inserted_since_rebuild)
            .expect("inserted-since-rebuild should fit in i64"),
        snapshot.insert_drift_fraction,
        snapshot.relation_ef_search,
        snapshot.session_ef_search,
        snapshot.effective_ef_search,
        snapshot.effective_source.to_owned(),
        snapshot.planner_scan_enabled,
    ))
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
    use std::sync::{Mutex, OnceLock};

    struct ScopedEnvVar {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl ScopedEnvVar {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for ScopedEnvVar {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.as_ref() {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn env_var_test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env-var test lock should not be poisoned")
    }
    #[cfg(test)]
    use hnsw_rs::prelude::{AnnT, Distance, Hnsw};
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

    // External oracle from the Qdrant `vector-db-benchmark` published results,
    // setup `qdrant-m-16-ef-128` (m=16, ef_construct=128, hnsw_ef=128) on
    // `dbpedia-openai-1M-1536-angular`. See `docs/RECALL_ANN_BENCHMARKS_ANCHOR.md`.
    // Source: https://qdrant.tech/benchmarks/results-1-100-thread-2024-06-15.json
    const ANN_BENCHMARKS_ANCHOR_PUBLISHED_RECALL_AT_10: f32 = 0.96082_f32;
    const ANN_BENCHMARKS_ANCHOR_TOLERANCE: f32 = 0.02_f32;

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

    #[cfg(test)]
    fn random_clustered_vectors(
        n: usize,
        dim: usize,
        n_clusters: usize,
        spread: f32,
        seed: u64,
    ) -> Vec<Vec<f32>> {
        let centers = random_unit_vectors(n_clusters, dim, seed + 100_000);
        let mut rng = ChaCha8Rng::seed_from_u64(seed + 200_000);
        let mut corpus = Vec::with_capacity(n);

        for i in 0..n {
            let center = &centers[i % n_clusters];
            let mut values = center
                .iter()
                .map(|&coordinate| {
                    let u1 = rng.gen_range(0.0001_f32..1.0_f32);
                    let u2 = rng.gen_range(0.0_f32..std::f32::consts::TAU);
                    let noise = (-2.0 * u1.ln()).sqrt() * u2.cos() * spread;
                    coordinate + noise
                })
                .collect::<Vec<_>>();
            let norm = values.iter().map(|value| value * value).sum::<f32>().sqrt();
            for value in &mut values {
                *value /= norm.max(f32::EPSILON);
            }
            corpus.push(values);
        }

        corpus
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

    fn parse_ctid(ctid: &str) -> am::page::ItemPointer {
        let trimmed = ctid.trim();
        let inner = trimmed
            .strip_prefix('(')
            .and_then(|value| value.strip_suffix(')'))
            .expect("ctid should use (block,offset) formatting");
        let (block_number, offset_number) = inner
            .split_once(',')
            .expect("ctid should contain block and offset");
        am::page::ItemPointer {
            block_number: block_number
                .trim()
                .parse()
                .expect("ctid block number should parse"),
            offset_number: offset_number
                .trim()
                .parse()
                .expect("ctid offset number should parse"),
        }
    }

    fn heap_tid_for_row(table_name: &str, id: i64) -> am::page::ItemPointer {
        let ctid = Spi::get_one::<String>(&format!(
            "SELECT ctid::text FROM {table_name} WHERE id = {id}"
        ))
        .expect("SPI query should succeed")
        .expect("table row should exist");
        parse_ctid(&ctid)
    }

    fn decode_index_elements_and_neighbors(
        index_oid: pg_sys::Oid,
        code_len: usize,
    ) -> (
        am::page::MetadataPage,
        Vec<(am::page::ItemPointer, am::page::TqElementTuple)>,
        HashMap<am::page::ItemPointer, am::page::TqNeighborTuple>,
    ) {
        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let mut elements = Vec::new();
        let mut neighbors = HashMap::new();

        for page in data_pages {
            for (idx, tuple) in page.tuples.iter().enumerate() {
                let tid = am::page::ItemPointer {
                    block_number: page.block_number,
                    offset_number: u16::try_from(idx + 1)
                        .expect("page tuple offset should fit in u16"),
                };
                match tuple.first().copied() {
                    Some(am::page::TQ_ELEMENT_TAG) => {
                        elements.push((
                            tid,
                            am::page::TqElementTuple::decode(tuple, code_len)
                                .expect("element tuple should decode"),
                        ));
                    }
                    Some(am::page::TQ_NEIGHBOR_TAG) => {
                        neighbors.insert(
                            tid,
                            am::page::TqNeighborTuple::decode(tuple)
                                .expect("neighbor tuple should decode"),
                        );
                    }
                    _ => {}
                }
            }
        }

        (metadata, elements, neighbors)
    }

    fn find_element_for_heap_tid(
        elements: &[(am::page::ItemPointer, am::page::TqElementTuple)],
        heap_tid: am::page::ItemPointer,
    ) -> (am::page::ItemPointer, &am::page::TqElementTuple) {
        let (element_tid, element) = elements
            .iter()
            .find(|(_, element)| element.heaptids.contains(&heap_tid))
            .expect("element should be discoverable by heap tid");
        (*element_tid, element)
    }

    fn layer_neighbor_slice(
        neighbor_tids: &[am::page::ItemPointer],
        m: usize,
        layer: u8,
    ) -> &[am::page::ItemPointer] {
        let (start, end) = if layer == 0 {
            (0, (m * 2).min(neighbor_tids.len()))
        } else {
            let start = (m * 2) + (usize::from(layer) - 1) * m;
            if start >= neighbor_tids.len() {
                return &neighbor_tids[0..0];
            }
            (start, (start + m).min(neighbor_tids.len()))
        };
        &neighbor_tids[start..end]
    }

    fn count_neighbor_refs(
        neighbors: &HashMap<am::page::ItemPointer, am::page::TqNeighborTuple>,
        target_tid: am::page::ItemPointer,
    ) -> usize {
        neighbors
            .values()
            .map(|neighbor| {
                neighbor
                    .tids
                    .iter()
                    .filter(|tid| **tid == target_tid)
                    .count()
            })
            .sum()
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

    #[cfg(test)]
    fn encode_recall_corpus_payloads(corpus: &[Vec<f32>]) -> Vec<Vec<u8>> {
        let quantizer = ProdQuantizer::cached(
            RECALL_DIM,
            u8::try_from(RECALL_BITS).expect("recall bits should fit into u8"),
            RECALL_SEED as u64,
        );
        corpus
            .iter()
            .map(|vector| quantizer.pack_payload(&quantizer.encode(vector)))
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

    #[cfg(test)]
    fn brute_force_top_k_exact_quantized(
        corpus_payloads: &[Vec<u8>],
        query: &[f32],
        k: usize,
    ) -> Vec<usize> {
        let quantizer = ProdQuantizer::cached(
            RECALL_DIM,
            u8::try_from(RECALL_BITS).expect("recall bits should fit into u8"),
            RECALL_SEED as u64,
        );
        let prepared = quantizer.prepare_ip_query(query);
        let mut scores = corpus_payloads
            .iter()
            .enumerate()
            .map(|(i, payload)| (i, quantizer.score_ip_encoded(&prepared, payload)))
            .collect::<Vec<_>>();
        scores.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        scores.truncate(k);
        scores.into_iter().map(|(i, _)| i).collect()
    }

    #[cfg(test)]
    #[derive(Debug, Clone, Copy)]
    struct RecallBuildCodeDistance {
        score_offset: f32,
    }

    #[cfg(test)]
    impl RecallBuildCodeDistance {
        fn new() -> Self {
            let quantizer = ProdQuantizer::cached(
                RECALL_DIM,
                u8::try_from(RECALL_BITS).expect("recall bits should fit into u8"),
                RECALL_SEED as u64,
            );
            let max_abs_centroid = quantizer
                .codebook
                .iter()
                .map(|value| value.abs())
                .fold(0.0_f32, f32::max);
            Self {
                score_offset: RECALL_DIM as f32 * max_abs_centroid * max_abs_centroid,
            }
        }
    }

    #[cfg(test)]
    impl Distance<u8> for RecallBuildCodeDistance {
        fn eval(&self, va: &[u8], vb: &[u8]) -> f32 {
            self.score_offset
                - score_code_inner_product(
                    RECALL_DIM,
                    u8::try_from(RECALL_BITS).expect("recall bits should fit into u8"),
                    RECALL_SEED as u64,
                    va,
                    vb,
                )
        }
    }

    #[cfg(test)]
    #[derive(Debug, Clone, Copy)]
    struct RecallSourceDistance {
        score_offset: f32,
    }

    #[cfg(test)]
    impl Distance<f32> for RecallSourceDistance {
        fn eval(&self, va: &[f32], vb: &[f32]) -> f32 {
            self.score_offset - dot_product(va, vb)
        }
    }

    #[cfg(test)]
    fn recall_neighbor_tuple_aligned_bytes(payload_len: usize) -> usize {
        let tuple_len = 4 + payload_len + 4;
        let remainder = tuple_len % 8;
        if remainder == 0 {
            tuple_len
        } else {
            tuple_len + (8 - remainder)
        }
    }

    #[cfg(test)]
    fn recall_max_level_that_fits(m: u16) -> u8 {
        let usable_page_bytes = 8192_usize.saturating_sub(24);
        let mut level = 0_u8;
        loop {
            let payload_len = crate::bench_api::neighbor_tuple_encoded_len(level, m);
            if recall_neighbor_tuple_aligned_bytes(payload_len) > usable_page_bytes {
                return level.saturating_sub(1);
            }
            if level == u8::MAX {
                return level;
            }
            level = level.saturating_add(1);
        }
    }

    #[cfg(test)]
    fn probe_hnsw_rs_code_graph_recall(
        corpus: &[Vec<f32>],
        queries: &[Vec<f32>],
        m: usize,
        ef_search: usize,
    ) -> (f32, f32, f32) {
        let corpus_codes = encode_recall_corpus_codes(corpus);
        let corpus_payloads = encode_recall_corpus_payloads(corpus);
        let max_layer = usize::from(recall_max_level_that_fits(
            u16::try_from(m).expect("m should fit into u16"),
        ))
        .saturating_add(1)
        .max(1);
        let hnsw = Hnsw::new(
            m,
            corpus_codes.len(),
            max_layer,
            usize::try_from(RECALL_EF_CONSTRUCTION).expect("ef_construction should fit usize"),
            RecallBuildCodeDistance::new(),
        );
        let build_started = Instant::now();
        let corpus_code_slices = corpus_codes
            .iter()
            .enumerate()
            .map(|(origin_id, code)| (code.as_slice(), origin_id))
            .collect::<Vec<_>>();
        hnsw.parallel_insert_slice(&corpus_code_slices);
        let build_elapsed = build_started.elapsed();

        let mut hnsw_hits = 0_i32;
        let mut build_code_hits = 0_i32;
        let mut exact_hits = 0_i32;
        let search_started = Instant::now();
        for query in queries {
            let truth_ids = brute_force_top_k(corpus, query, RECALL_K)
                .into_iter()
                .map(|idx| i64::try_from(idx).expect("origin id should fit into i64"))
                .collect::<Vec<_>>();
            let query_code = encode_recall_query_code(query);
            let hnsw_ids = hnsw
                .search_neighbours(query_code.as_slice(), RECALL_K, ef_search)
                .into_iter()
                .map(|neighbor| i64::try_from(neighbor.d_id).expect("origin id should fit i64"))
                .collect::<Vec<_>>();
            let build_code_ids =
                brute_force_top_k_code_inner_product(&corpus_codes, &query_code, RECALL_K)
                    .into_iter()
                    .map(|idx| i64::try_from(idx).expect("origin id should fit into i64"))
                    .collect::<Vec<_>>();
            let exact_ids = brute_force_top_k_exact_quantized(&corpus_payloads, query, RECALL_K)
                .into_iter()
                .map(|idx| i64::try_from(idx).expect("origin id should fit into i64"))
                .collect::<Vec<_>>();

            hnsw_hits += recall_top_k_overlap(&truth_ids, &hnsw_ids);
            build_code_hits += recall_top_k_overlap(&truth_ids, &build_code_ids);
            exact_hits += recall_top_k_overlap(&truth_ids, &exact_ids);
        }
        let search_elapsed = search_started.elapsed();
        println!(
            "hnsw-rs code graph timings: m={m} ef_search={ef_search} build={build_elapsed:?} search={search_elapsed:?}"
        );

        let denom = (queries.len() * RECALL_K) as f32;
        (
            hnsw_hits as f32 / denom,
            build_code_hits as f32 / denom,
            exact_hits as f32 / denom,
        )
    }

    #[cfg(test)]
    fn probe_hnsw_rs_source_graph_recall(
        corpus: &[Vec<f32>],
        queries: &[Vec<f32>],
        m: usize,
        ef_search: usize,
    ) -> f32 {
        let max_layer = usize::from(recall_max_level_that_fits(
            u16::try_from(m).expect("m should fit into u16"),
        ))
        .saturating_add(1)
        .max(1);
        let hnsw = Hnsw::new(
            m,
            corpus.len(),
            max_layer,
            usize::try_from(RECALL_EF_CONSTRUCTION).expect("ef_construction should fit usize"),
            RecallSourceDistance { score_offset: 1.0 },
        );
        let build_started = Instant::now();
        for (origin_id, vector) in corpus.iter().enumerate() {
            hnsw.insert((vector.as_slice(), origin_id));
        }
        let build_elapsed = build_started.elapsed();

        let mut hnsw_hits = 0_i32;
        let search_started = Instant::now();
        for query in queries {
            let truth_ids = brute_force_top_k(corpus, query, RECALL_K)
                .into_iter()
                .map(|idx| i64::try_from(idx).expect("origin id should fit into i64"))
                .collect::<Vec<_>>();
            let hnsw_ids = hnsw
                .search_neighbours(query.as_slice(), RECALL_K, ef_search)
                .into_iter()
                .map(|neighbor| i64::try_from(neighbor.d_id).expect("origin id should fit i64"))
                .collect::<Vec<_>>();
            hnsw_hits += recall_top_k_overlap(&truth_ids, &hnsw_ids);
        }
        let search_elapsed = search_started.elapsed();
        println!(
            "hnsw-rs source graph timings: m={m} ef_search={ef_search} build={build_elapsed:?} search={search_elapsed:?}"
        );

        hnsw_hits as f32 / (queries.len() * RECALL_K) as f32
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
    fn test_fr020_empty_index_remains_planner_gated() {
        Spi::run("CREATE TABLE tqhnsw_empty_cost (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_empty_cost_idx ON tqhnsw_empty_cost USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("empty-index creation should succeed");

        let modeled_startup = Spi::get_one::<f64>(
            "SELECT modeled_startup_cost FROM tqhnsw_index_cost_snapshot('tqhnsw_empty_cost_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("modeled startup should be non-null");
        let modeled_total = Spi::get_one::<f64>(
            "SELECT modeled_total_cost FROM tqhnsw_index_cost_snapshot('tqhnsw_empty_cost_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("modeled total should be non-null");
        assert_eq!(
            modeled_startup,
            f64::MAX,
            "empty tqhnsw index must keep the FR-020 gate active even after D2 activation"
        );
        assert_eq!(
            modeled_total,
            f64::MAX,
            "empty tqhnsw index must keep the FR-020 gate active even after D2 activation"
        );

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM tqhnsw_empty_cost \
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
            "planner must not pick an empty tqhnsw index even with D2 activation: {plan}"
        );
    }

    #[pg_test]
    fn test_fr020_ac2_planner_prefers_seqscan_for_small_tables() {
        Spi::run("CREATE TABLE tqhnsw_small_seqscan (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_small_seqscan \
             SELECT g, encode_to_tqvector(ARRAY[g::real, (g * 0.25)::real, (g * -0.5)::real, 1.0::real], 4, 42) \
             FROM generate_series(1, 50) AS g",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_small_seqscan_idx ON tqhnsw_small_seqscan USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("ANALYZE tqhnsw_small_seqscan").expect("analyze should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM tqhnsw_small_seqscan \
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
            "planner should prefer sequential scan on a 50-row table even with FR-020 activated (AC-2): {plan}"
        );
    }

    #[pg_test]
    fn test_tqhnsw_planner_chooses_index_scan_for_ordered_query() {
        Spi::run("CREATE TABLE tqhnsw_scan_plan (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_scan_plan VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_scan_plan_idx ON tqhnsw_scan_plan USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM tqhnsw_scan_plan \
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
            plan.contains("Index Scan") || plan.contains("Index Only Scan"),
            "planner should select the tqhnsw index scan once FR-020 cost activation is live: {plan}"
        );
    }

    #[pg_test]
    fn test_fr020_ac1_planner_chooses_index_scan_for_large_table() {
        Spi::run("CREATE TABLE tqhnsw_ac1_large (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        // Build 10K 64-dim vectors from four MD5 digests per row so each row
        // gets 64 distinct byte values without 10K × N hashtext calls. Keeping
        // this at 64 dimensions is deliberate: lowering the dimension count
        // flips the FR-020 crossover back toward seqscan at 10K rows.
        Spi::run(
            "INSERT INTO tqhnsw_ac1_large \
             SELECT g, encode_to_tqvector( \
                 ARRAY( \
                     SELECT ((get_byte( \
                              decode(md5(g::text) \
                                     || md5((g + 999983)::text) \
                                     || md5((g + 1999993)::text) \
                                     || md5((g + 2999999)::text), 'hex'), \
                              i)::real - 128.0) / 128.0)::real \
                     FROM generate_series(0, 63) AS i), \
                 4, 42) \
             FROM generate_series(1, 10000) AS g",
        )
        .expect("10k-row insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_ac1_large_idx ON tqhnsw_ac1_large USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("ANALYZE tqhnsw_ac1_large").expect("analyze should succeed");

        let query_array = {
            let mut s = String::from("ARRAY[");
            for i in 0..64 {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&format!("{:.6}", (i as f32 * 0.05 - 1.5)));
            }
            s.push_str("]::real[]");
            s
        };
        let explain_sql = format!(
            "EXPLAIN (COSTS OFF) SELECT id FROM tqhnsw_ac1_large \
             ORDER BY embedding <#> {query_array} LIMIT 10"
        );

        let plan = Spi::connect(|client| {
            let rows = client
                .select(&explain_sql, None, &[])
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
            plan.contains("Index Scan") && plan.contains("tqhnsw_ac1_large_idx"),
            "FR-020-AC-1 / TC-206: planner must naturally pick the tqhnsw index on a 10K-row table: {plan}"
        );
    }

    #[pg_test]
    fn test_tqhnsw_index_admin_snapshot_tracks_insert_drift() {
        Spi::run("CREATE TABLE tqhnsw_admin_snapshot (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_admin_snapshot VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.5, 0.5, -0.5, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_admin_snapshot_idx ON tqhnsw_admin_snapshot USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (ef_search = 77)",
        )
        .expect("index creation should succeed");
        Spi::run("SET tqhnsw.ef_search = 19").expect("set should succeed");

        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT total_live_nodes FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("live-node count should be non-null"),
            3
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT inserted_since_rebuild FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("inserted-since-rebuild should be non-null"),
            0
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT insert_drift_fraction FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("insert drift fraction should be non-null"),
            0.0
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT relation_ef_search FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("relation ef_search should be non-null"),
            77
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT session_ef_search FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed"),
            Some(19)
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT effective_source FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective source should be non-null"),
            "session"
        );

        Spi::run(
            "INSERT INTO tqhnsw_admin_snapshot VALUES
             (4, encode_to_tqvector(ARRAY[0.9, 0.1, 0.25, -0.9], 4, 42))",
        )
        .expect("live insert should succeed");

        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT total_live_nodes FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("live-node count should be non-null"),
            4
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT inserted_since_rebuild FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("inserted-since-rebuild should be non-null"),
            1
        );
        assert!(
            (Spi::get_one::<f64>(
                "SELECT insert_drift_fraction FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("insert drift fraction should be non-null")
                - 0.25)
                .abs()
                < 1e-9,
            "one live insert after a three-row build should report 25% drift",
        );

        Spi::run(
            "INSERT INTO tqhnsw_admin_snapshot VALUES
             (5, encode_to_tqvector(ARRAY[0.9, 0.1, 0.25, -0.9], 4, 42))",
        )
        .expect("duplicate insert should succeed");

        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT total_live_nodes FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("live-node count should be non-null"),
            4,
            "duplicate coalescing should not create a new live node",
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT inserted_since_rebuild FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("inserted-since-rebuild should be non-null"),
            1,
            "duplicate coalescing should not advance the insert-drift counter",
        );

        Spi::run("RESET tqhnsw.ef_search").expect("reset should succeed");
    }

    #[pg_test]
    fn test_tqhnsw_index_admin_snapshot_counts_empty_first_insert() {
        Spi::run(
            "CREATE TABLE tqhnsw_admin_snapshot_empty (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_admin_snapshot_empty_idx ON tqhnsw_admin_snapshot_empty USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT total_live_nodes FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_empty_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("live-node count should be non-null"),
            0
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT inserted_since_rebuild FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_empty_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("inserted-since-rebuild should be non-null"),
            0
        );

        Spi::run(
            "INSERT INTO tqhnsw_admin_snapshot_empty VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("first insert should succeed");

        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT total_live_nodes FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_empty_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("live-node count should be non-null"),
            1
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT inserted_since_rebuild FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_empty_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("inserted-since-rebuild should be non-null"),
            1,
            "the first successful live insert should start the post-build drift counter",
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT insert_drift_fraction FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_empty_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("insert drift fraction should be non-null"),
            1.0
        );
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw_index_admin_snapshot requires a tqhnsw index")]
    fn test_tqhnsw_index_admin_snapshot_rejects_non_tqhnsw_index() {
        Spi::run(
            "CREATE TABLE tqhnsw_admin_snapshot_wrong_am (id bigint primary key, value bigint)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_admin_snapshot_wrong_am_idx ON tqhnsw_admin_snapshot_wrong_am USING btree (value)",
        )
        .expect("index creation should succeed");

        let _ = Spi::get_one::<i64>(
            "SELECT total_live_nodes FROM tqhnsw_index_admin_snapshot('tqhnsw_admin_snapshot_wrong_am_idx'::regclass)",
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
            Spi::get_one::<bool>(
                "SELECT planner_scan_enabled FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("planner flag should be non-null"),
            "planner gate should be live after D2 cost-model activation"
        );
        assert!(
            Spi::get_one::<String>(
                "SELECT planner_gate_reason FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("gate reason should be non-null")
                .contains("FR-020"),
            "cost snapshot should reference FR-020 once the planner gate is retired"
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
        let max_level = Spi::get_one::<i32>(
            "SELECT max_level FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("max level should be non-null");
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT resolved_tree_height FROM tqhnsw_index_cost_snapshot('tqhnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("resolved tree height should be non-null"),
            f64::from(max_level)
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
            Spi::get_one::<bool>(
                "SELECT planner_scan_enabled FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("planner scan flag should be non-null")
        );
        assert!(
            Spi::get_one::<bool>(
                "SELECT ordered_scan_ready FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("ordered scan readiness should be non-null")
        );
        assert!(
            Spi::get_one::<bool>(
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
            Spi::get_one::<bool>(
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
            "planner scan selection is live: FR-020 cost model active (ADR-011 superseded)"
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT next_runtime_blocker FROM tqhnsw_planner_integration_snapshot('tqhnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("runtime blocker should be non-null"),
            "no merged runtime blocker remains on main; post-vacuum benchmark/reporting is next"
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
        let persisted_binary_quantizer = crate::quant::prod::ProdQuantizer::cached(
            metadata.dimensions as usize,
            metadata.bits,
            metadata.seed,
        );
        let expected_binary_word_count =
            if persisted_binary_quantizer.binary_sign_no_qjl_4bit_supported() {
                (metadata.dimensions as usize).div_ceil(64)
            } else {
                0
            };
        for (element_tid, element) in &elements {
            assert!(element.level <= metadata.max_level);
            assert!(!element.deleted);
            assert_eq!(element.heaptids.len(), 1);
            assert_ne!(element.heaptids[0], am::page::ItemPointer::INVALID);
            assert_eq!(
                element.binary_words.len(),
                expected_binary_word_count,
                "builds should persist ADR-031 sidecars only on the supported no-QJL 4-bit lane",
            );
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
    fn test_experimental_grouped_v2_source_build_writes_grouped_pages() {
        let _lock = env_var_test_lock();
        let _guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_source_build (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 19 + dim) as f32) * 0.05).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 11 + dim) as f32) * 0.04).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_source_build VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_source_build_idx ON tqhnsw_grouped_v2_source_build USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_grouped_v2_source_build_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };

        assert_eq!(metadata.format_version, am::page::INDEX_FORMAT_V2_GROUPED);
        assert_eq!(metadata.transform_kind, am::page::TransformKind::Srht);
        assert_eq!(
            metadata.search_codec_kind,
            am::page::SearchCodecKind::GroupedPq
        );
        assert_eq!(
            metadata.rerank_codec_kind,
            am::page::RerankCodecKind::ScalarQuantized
        );
        assert_eq!(
            metadata.payload_flags & am::page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE,
            am::page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE
        );
        assert_eq!(
            metadata.payload_flags & am::page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD,
            am::page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD
        );
        assert_eq!(metadata.dimensions, 16);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);
        assert_eq!(metadata.search_bits, 4);
        assert_eq!(metadata.search_subvector_count, 1);
        assert_eq!(metadata.search_subvector_dim, 16);

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

        let grouped_hot_tids = page_tuples
            .iter()
            .filter_map(|(tid, tuple)| {
                (tuple.first().copied() == Some(am::page::TQ_GROUPED_HOT_TAG)).then_some(*tid)
            })
            .collect::<Vec<_>>();
        let rerank_count = page_tuples
            .iter()
            .filter(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_RERANK_TAG))
            .count();
        let neighbor_count = page_tuples
            .iter()
            .filter(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_NEIGHBOR_TAG))
            .count();
        let grouped_codebook_count = page_tuples
            .iter()
            .filter(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_GROUPED_CODEBOOK_TAG))
            .count();

        assert_eq!(grouped_hot_tids.len(), 16);
        assert_eq!(rerank_count, 16);
        assert_eq!(neighbor_count, 16);
        assert_eq!(
            grouped_codebook_count,
            metadata.search_subvector_count as usize
        );
        assert_ne!(
            metadata.grouped_codebook_head,
            am::page::ItemPointer::INVALID
        );
        assert!(!page_tuples
            .iter()
            .any(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG)));
        assert!(
            grouped_hot_tids.contains(&metadata.entry_point),
            "entry point should identify a grouped hot tuple under the experimental ADR-030 v2 build gate"
        );
    }

    #[pg_test]
    fn test_grouped_v2_graph_reads_load_entry_and_neighbors() {
        let _lock = env_var_test_lock();
        let _guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_graph_reads (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 23 + dim) as f32) * 0.03).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 13 + dim) as f32) * 0.06).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_graph_reads VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_graph_reads_idx ON tqhnsw_grouped_v2_graph_reads USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_grouped_v2_graph_reads_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let (_block_count, metadata, _data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let layout = match am::graph::GraphStorageDescriptor::from_metadata(&metadata).unwrap() {
            am::graph::GraphStorageDescriptor::GroupedV2(layout) => layout,
            am::graph::GraphStorageDescriptor::ScalarV1 { .. } => {
                panic!("experimental grouped-v2 build should not decode as scalar storage")
            }
        };

        let index_relation = unsafe {
            open_valid_tqhnsw_index(index_oid, "test_experimental_grouped_v2_graph_reads")
        };

        unsafe {
            am::graph::with_graph_storage_tuple(
                index_relation,
                metadata.entry_point,
                am::graph::GraphStorageDescriptor::GroupedV2(layout),
                |entry| match entry {
                    am::graph::GraphTupleRef::GroupedHot(tuple) => {
                        assert_eq!(tuple.search_code.len(), layout.search_code_len);
                        assert_eq!(tuple.collect_binary_words().len(), layout.binary_word_count);
                        assert!(tuple.heaptid_count() > 0);
                    }
                    am::graph::GraphTupleRef::Scalar(_) => {
                        panic!("grouped-v2 entry should decode as grouped-hot tuple")
                    }
                },
            );
        }

        let (entry, neighbors) = unsafe {
            am::graph::load_grouped_graph_adjacency(index_relation, metadata.entry_point, layout)
        };

        assert_eq!(entry.tid, metadata.entry_point);
        assert!(!entry.deleted);
        assert_eq!(entry.search_code.len(), layout.search_code_len);
        assert_eq!(entry.binary_words.len(), layout.binary_word_count);
        assert!(!entry.heaptids.is_empty());
        assert_ne!(entry.reranktid, am::page::ItemPointer::INVALID);
        assert_eq!(neighbors.tid, entry.neighbortid);
        assert!(neighbors.count > 0);
        assert!(
            neighbors
                .tids
                .iter()
                .any(|tid| *tid != am::page::ItemPointer::INVALID),
            "entry adjacency should include at least one real grouped-hot neighbor",
        );

        let first_neighbor_tid = neighbors
            .tids
            .iter()
            .copied()
            .find(|tid| *tid != am::page::ItemPointer::INVALID)
            .expect("grouped entry should expose a readable neighbor");
        let neighbor = unsafe {
            am::graph::load_grouped_graph_element(index_relation, first_neighbor_tid, layout)
        };

        assert_eq!(neighbor.search_code.len(), layout.search_code_len);
        assert_eq!(neighbor.binary_words.len(), layout.binary_word_count);
        assert!(!neighbor.heaptids.is_empty());
        assert_ne!(neighbor.reranktid, am::page::ItemPointer::INVALID);

        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    }

    #[pg_test]
    fn test_grouped_v2_graph_reads_load_cold_rerank_payload() {
        let _lock = env_var_test_lock();
        let _guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_rerank_reads (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 41 + dim) as f32) * 0.03).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 17 + dim) as f32) * 0.05).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_rerank_reads VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_rerank_reads_idx ON tqhnsw_grouped_v2_rerank_reads USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_grouped_v2_rerank_reads_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let (_block_count, metadata, _data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let layout = match am::graph::GraphStorageDescriptor::from_metadata(&metadata).unwrap() {
            am::graph::GraphStorageDescriptor::GroupedV2(layout) => layout,
            am::graph::GraphStorageDescriptor::ScalarV1 { .. } => {
                panic!("experimental grouped-v2 build should not decode as scalar storage")
            }
        };

        let index_relation = unsafe {
            open_valid_tqhnsw_index(
                index_oid,
                "test_grouped_v2_graph_reads_load_cold_rerank_payload",
            )
        };
        let entry = unsafe {
            am::graph::load_grouped_graph_element(index_relation, metadata.entry_point, layout)
        };
        let rerank = unsafe {
            am::graph::load_grouped_rerank_payload(index_relation, entry.reranktid, layout)
        };

        assert_eq!(rerank.tid, entry.reranktid);
        assert_eq!(rerank.code.len(), layout.rerank_code_len);
        assert!(rerank.gamma.is_finite());
        assert!(
            rerank.code.iter().any(|byte| *byte != 0),
            "cold rerank payload should contain a non-empty scalar code",
        );

        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    }

    #[pg_test]
    fn test_grouped_v2_graph_reads_load_persisted_codebooks() {
        let _lock = env_var_test_lock();
        let _guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_codebook_reads (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 43 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 19 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_codebook_reads VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_codebook_reads_idx ON tqhnsw_grouped_v2_codebook_reads USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_grouped_v2_codebook_reads_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let (_block_count, metadata, _data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_ne!(
            metadata.grouped_codebook_head,
            am::page::ItemPointer::INVALID
        );

        let index_relation = unsafe {
            open_valid_tqhnsw_index(
                index_oid,
                "test_grouped_v2_graph_reads_load_persisted_codebooks",
            )
        };
        let model = unsafe { am::graph::load_grouped_codebook_model(index_relation, &metadata) };

        assert_eq!(model.head_tid, metadata.grouped_codebook_head);
        assert_eq!(model.group_count, metadata.search_subvector_count as usize);
        assert_eq!(model.group_size, metadata.search_subvector_dim as usize);
        assert_eq!(
            model.flat_codebooks.len(),
            model.group_count * model.group_size * crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS
        );
        assert!(
            model.flat_codebooks.iter().all(|value| value.is_finite()),
            "persisted grouped codebooks should decode as finite f32 values",
        );

        let head = unsafe {
            am::graph::with_grouped_codebook_tuple(
                index_relation,
                model.head_tid,
                model.group_size * crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS,
                |tuple| (tuple.group_index, tuple.nexttid),
            )
        };
        assert_eq!(head.0, 0);
        if model.group_count == 1 {
            assert_eq!(head.1, am::page::ItemPointer::INVALID);
        } else {
            assert_ne!(head.1, am::page::ItemPointer::INVALID);
        }

        let query = vec![0.5_f32; model.group_count * model.group_size];
        let lut = crate::quant::grouped_pq::build_grouped_pq_lut_f32(
            &query,
            &model.flat_codebooks,
            model.group_size,
        );
        assert_eq!(
            lut.len(),
            model.group_count * crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS
        );

        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    }

    #[pg_test]
    #[should_panic(
        expected = "tqhnsw scan runtime does not support ADR-030 grouped-v2 indexes yet"
    )]
    fn test_experimental_grouped_v2_ordered_scan_rejects_runtime() {
        let _lock = env_var_test_lock();
        let _guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_runtime_reject (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 23 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 13 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_runtime_reject VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_runtime_reject_idx ON tqhnsw_grouped_v2_runtime_reject USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let _ = Spi::get_one::<i64>(
            "SELECT id FROM tqhnsw_grouped_v2_runtime_reject \
             ORDER BY embedding <#> ARRAY[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, \
                                      0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6]::real[] \
             LIMIT 1",
        );
    }

    #[pg_test]
    fn test_grouped_v2_ordered_scan_runtime_gate_smoke() {
        let _lock = env_var_test_lock();
        let _build_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");
        let _scan_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1");

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_runtime_enabled (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 29 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 17 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_runtime_enabled VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_runtime_enabled_idx ON tqhnsw_grouped_v2_runtime_enabled USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM tqhnsw_grouped_v2_runtime_enabled \
                     ORDER BY embedding <#> ARRAY[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, \
                                              0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6]::real[] \
                     LIMIT 3",
                    None,
                    &[],
                )
                .expect("EXPLAIN should succeed");
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
            plan.contains("Index Scan") || plan.contains("Index Only Scan"),
            "grouped-v2 runtime smoke test should route through tqhnsw when the scan gate is enabled: {plan}"
        );

        let ordered_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id FROM tqhnsw_grouped_v2_runtime_enabled \
                     ORDER BY embedding <#> ARRAY[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, \
                                              0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6]::real[] \
                     LIMIT 3",
                    None,
                    &[],
                )
                .expect("ordered SELECT should succeed when the grouped runtime gate is enabled")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(ordered_ids.len(), 3);
        assert!(
            ordered_ids.windows(2).all(|pair| pair[0] != pair[1]),
            "grouped-v2 runtime smoke test should emit distinct ids"
        );
    }

    #[pg_test]
    #[should_panic(
        expected = "tqhnsw grouped-v2 live rerank window must be between 1 and 16, got 0"
    )]
    fn test_grouped_v2_runtime_rejects_invalid_live_window_env() {
        let _lock = env_var_test_lock();
        let _build_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");
        let _scan_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1");
        let _window_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_WINDOW", "0");

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_runtime_invalid_window (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 29 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 17 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_runtime_invalid_window VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_runtime_invalid_window_idx ON tqhnsw_grouped_v2_runtime_invalid_window USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let _ = Spi::get_one::<i64>(
            "SELECT id FROM tqhnsw_grouped_v2_runtime_invalid_window \
             ORDER BY embedding <#> ARRAY[0.5, 0.1, 0.4, -0.8, 0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, -0.1, -0.2, -0.3, -0.4]::real[] \
             LIMIT 1",
        )
        .expect("ordered scan should reach amrescan before rejecting invalid grouped window env");
    }

    #[pg_test]
    fn test_grouped_v2_runtime_captures_exact_rerank_comparison_scores() {
        let _lock = env_var_test_lock();
        let _build_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");
        let _scan_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1");

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_runtime_compare (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 31 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 19 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_runtime_compare VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_runtime_compare_idx ON tqhnsw_grouped_v2_runtime_compare USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_grouped_v2_runtime_compare_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![
            0.1_f32, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6,
        ];
        let observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_score_comparisons(index_oid, query.clone())
        };
        let query_literal = format_recall_vector_sql_literal(&query);
        let exact_scores = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT
                            split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                            split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number,
                            embedding <#> {query_literal} AS exact_score
                         FROM tqhnsw_grouped_v2_runtime_compare"
                    ),
                    None,
                    &[],
                )
                .expect("exact grouped rerank comparison query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    let exact_score = row["exact_score"]
                        .value::<f32>()
                        .expect("exact score should decode")
                        .expect("exact score should be non-null");
                    (
                        (
                            u32::try_from(block_number)
                                .expect("block number should be non-negative"),
                            u16::try_from(offset_number)
                                .expect("offset number should be positive"),
                        ),
                        exact_score,
                    )
                })
                .collect::<HashMap<_, _>>()
        });

        assert!(
            !observed.is_empty(),
            "grouped-v2 runtime comparison path should emit at least one ordered result"
        );
        for (heap_tid, _approx_score, comparison_score, _approx_rank) in observed {
            let comparison_score = comparison_score
                .expect("grouped-v2 emitted results should carry an exact rerank comparison score");
            let expected = exact_scores
                .get(&heap_tid)
                .copied()
                .expect("every emitted heap tid should map back to an exact SQL score");
            assert_eq!(
                comparison_score, expected,
                "grouped-v2 comparison score should match the operator-facing exact <#> score for the emitted tuple"
            );
        }
    }

    type DebugScanComparisonRow = ((u32, u16), f32, Option<f32>, Option<i32>);

    fn create_grouped_v2_runtime_fixture(table_name: &str, index_name: &str) -> pg_sys::Oid {
        Spi::run(&format!(
            "CREATE TABLE {table_name} (
                id bigint primary key,
                source real[],
                embedding tqvector
            )"
        ))
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 41 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 29 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO {table_name} VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')"
        ))
        .expect("index creation should succeed");

        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
            .expect("SPI query should succeed")
            .expect("index oid should exist")
    }

    fn grouped_v2_runtime_query() -> Vec<f32> {
        vec![
            0.12_f32, 0.22, 0.32, 0.42, 0.52, 0.62, 0.72, 0.82, 0.92, 1.02, 1.12, 1.22, 1.32, 1.42,
            1.52, 1.62,
        ]
    }

    fn grouped_v2_exact_traversal_runtime_observed_scores(
        table_name: &str,
        index_name: &str,
        scope: Option<&str>,
    ) -> Vec<DebugScanComparisonRow> {
        let _lock = env_var_test_lock();
        let _build_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");
        let _scan_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1");
        let _exact_guard =
            ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL", "1");
        let _scope_guard = scope.map(|value| {
            ScopedEnvVar::set(
                "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_SCOPE",
                value,
            )
        });
        let index_oid = create_grouped_v2_runtime_fixture(table_name, index_name);
        unsafe { am::debug_gettuple_scan_heap_tids_with_score_comparisons(index_oid, grouped_v2_runtime_query()) }
    }

    #[pg_test]
    fn test_grouped_v2_exact_traversal_emits_exact_scores() {
        let observed = grouped_v2_exact_traversal_runtime_observed_scores(
            "tqhnsw_grouped_v2_runtime_exact_traversal",
            "tqhnsw_grouped_v2_runtime_exact_traversal_idx",
            None,
        );

        assert!(
            !observed.is_empty(),
            "exact grouped traversal runtime should still emit ordered results"
        );
        for (_heap_tid, emitted_score, comparison_score, _approx_rank) in observed {
            let comparison_score = comparison_score
                .expect("exact grouped traversal runtime should still attach comparison scores");
            assert_eq!(
                emitted_score, comparison_score,
                "exact grouped traversal should emit the same exact rerank score it records as the comparison sidecar"
            );
        }
    }

    #[pg_test]
    fn test_tqhnsw_debug_adr030_runtime_settings_reflect_env() {
        let _lock = env_var_test_lock();
        let _build_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");
        let _scan_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1");
        let _window_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_WINDOW", "8");
        let _exact_guard =
            ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL", "1");
        let _scope_guard =
            ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_SCOPE", "all");
        let _limit_guard =
            ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_LIMIT", "1");

        assert_eq!(
            Spi::get_one::<bool>(
                "SELECT grouped_build_enabled
                 FROM tests.tqhnsw_debug_adr030_runtime_settings()"
            )
            .expect("runtime settings probe should succeed"),
            Some(true),
            "the runtime settings probe should surface the grouped build gate",
        );
        assert_eq!(
            Spi::get_one::<bool>(
                "SELECT grouped_scan_enabled
                 FROM tests.tqhnsw_debug_adr030_runtime_settings()"
            )
            .expect("runtime settings probe should succeed"),
            Some(true),
            "the runtime settings probe should surface the grouped scan gate",
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT grouped_scan_window
                 FROM tests.tqhnsw_debug_adr030_runtime_settings()"
            )
            .expect("runtime settings probe should succeed")
            .as_deref(),
            Some("8"),
            "the runtime settings probe should surface the configured grouped scan window",
        );
        assert_eq!(
            Spi::get_one::<bool>(
                "SELECT grouped_exact_traversal_enabled
                 FROM tests.tqhnsw_debug_adr030_runtime_settings()"
            )
            .expect("runtime settings probe should succeed"),
            Some(true),
            "the runtime settings probe should surface the grouped exact traversal gate",
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT grouped_exact_traversal_scope
                 FROM tests.tqhnsw_debug_adr030_runtime_settings()"
            )
            .expect("runtime settings probe should succeed")
            .as_deref(),
            Some("all"),
            "the runtime settings probe should surface the grouped exact traversal scope",
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT grouped_exact_traversal_limit
                 FROM tests.tqhnsw_debug_adr030_runtime_settings()"
            )
            .expect("runtime settings probe should succeed")
            .as_deref(),
            Some("1"),
            "the runtime settings probe should surface the grouped exact traversal limit",
        );
    }

    #[pg_test]
    fn test_grouped_v2_profile_exact_counters_zero_without_gate() {
        let _lock = env_var_test_lock();
        let _build_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");
        let _scan_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1");
        let index_oid = create_grouped_v2_runtime_fixture(
            "tqhnsw_grouped_v2_runtime_profile_approx",
            "tqhnsw_grouped_v2_runtime_profile_approx_idx",
        );
        let (
            _rescan_elapsed_us,
            _emit_elapsed_us,
            _total_elapsed_us,
            _rescan_phase,
            _rescan_current_result,
            _rescan_ordered_slots,
            _rescan_pending_heap_tids,
            _rescan_visited_elements,
            _rescan_expanded_sources,
            _rescan_emitted_elements,
            _rescan_bootstrap_expansions,
            _rescan_bootstrap_pages_read,
            _rescan_quantizer_cache_hit,
            _result_count,
            _final_phase,
            _final_ordered_slots,
            _total_bootstrap_expansions,
            _total_bootstrap_pages_read,
            _total_linear_pages_read,
            _total_elements_scored,
            _total_elements_skipped,
            _total_heap_tids_returned,
            _total_quantizer_cache_hit,
            _total_emitted_elements,
            _rescan_amrescan_total_elapsed_us,
            _rescan_query_decode_elapsed_us,
            _rescan_scan_setup_elapsed_us,
            _rescan_store_query_elapsed_us,
            _rescan_prepare_query_elapsed_us,
            _rescan_reset_state_elapsed_us,
            _rescan_initialize_entry_elapsed_us,
            _rescan_upper_layer_seed_elapsed_us,
            _rescan_layer0_seed_elapsed_us,
            _rescan_stage_ordered_results_elapsed_us,
            _rescan_initial_prefetch_elapsed_us,
            _rescan_frontier_consume_elapsed_us,
            _rescan_graph_result_materialize_elapsed_us,
            _graph_element_cache_hits,
            _graph_element_cache_misses,
            _graph_element_load_elapsed_us,
            _graph_neighbor_cache_hits,
            _graph_neighbor_cache_misses,
            _graph_neighbor_load_elapsed_us,
            _candidate_score_calls,
            _candidate_score_elapsed_us,
            _score_cache_hits,
            _score_cache_misses,
            grouped_traversal_approx_score_calls,
            grouped_traversal_approx_score_elapsed_us,
            grouped_traversal_exact_score_calls,
            grouped_traversal_exact_score_elapsed_us,
            grouped_traversal_budgeted_expansions,
            grouped_traversal_budgeted_candidates,
            grouped_traversal_budgeted_exact_candidates,
        ) = unsafe { am::debug_profile_ordered_scan(index_oid, grouped_v2_runtime_query()) };

        assert!(
            grouped_traversal_approx_score_calls > 0
                && grouped_traversal_approx_score_elapsed_us >= 0,
            "grouped approximate scans should surface grouped approximate traversal scoring work",
        );
        assert_eq!(
            (
                grouped_traversal_exact_score_calls,
                grouped_traversal_exact_score_elapsed_us,
                grouped_traversal_budgeted_expansions,
                grouped_traversal_budgeted_candidates,
                grouped_traversal_budgeted_exact_candidates,
            ),
            (0, 0, 0, 0, 0),
            "grouped approximate scans should leave grouped exact traversal counters inert",
        );
    }

    #[pg_test]
    fn test_grouped_v2_runtime_profile_budgeted_exact_counters() {
        let _lock = env_var_test_lock();
        let _build_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");
        let _scan_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1");
        let _exact_guard =
            ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL", "1");
        let _limit_guard =
            ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_LIMIT", "1");
        let index_oid = create_grouped_v2_runtime_fixture(
            "tqhnsw_grouped_v2_runtime_profile_budgeted_exact",
            "tqhnsw_grouped_v2_runtime_profile_budgeted_exact_idx",
        );
        let (
            _rescan_elapsed_us,
            _emit_elapsed_us,
            _total_elapsed_us,
            _rescan_phase,
            _rescan_current_result,
            _rescan_ordered_slots,
            _rescan_pending_heap_tids,
            _rescan_visited_elements,
            _rescan_expanded_sources,
            _rescan_emitted_elements,
            _rescan_bootstrap_expansions,
            _rescan_bootstrap_pages_read,
            _rescan_quantizer_cache_hit,
            _result_count,
            _final_phase,
            _final_ordered_slots,
            _total_bootstrap_expansions,
            _total_bootstrap_pages_read,
            _total_linear_pages_read,
            _total_elements_scored,
            _total_elements_skipped,
            _total_heap_tids_returned,
            _total_quantizer_cache_hit,
            _total_emitted_elements,
            _rescan_amrescan_total_elapsed_us,
            _rescan_query_decode_elapsed_us,
            _rescan_scan_setup_elapsed_us,
            _rescan_store_query_elapsed_us,
            _rescan_prepare_query_elapsed_us,
            _rescan_reset_state_elapsed_us,
            _rescan_initialize_entry_elapsed_us,
            _rescan_upper_layer_seed_elapsed_us,
            _rescan_layer0_seed_elapsed_us,
            _rescan_stage_ordered_results_elapsed_us,
            _rescan_initial_prefetch_elapsed_us,
            _rescan_frontier_consume_elapsed_us,
            _rescan_graph_result_materialize_elapsed_us,
            _graph_element_cache_hits,
            _graph_element_cache_misses,
            _graph_element_load_elapsed_us,
            _graph_neighbor_cache_hits,
            _graph_neighbor_cache_misses,
            _graph_neighbor_load_elapsed_us,
            _candidate_score_calls,
            _candidate_score_elapsed_us,
            score_cache_hits,
            score_cache_misses,
            grouped_traversal_approx_score_calls,
            grouped_traversal_approx_score_elapsed_us,
            grouped_traversal_exact_score_calls,
            grouped_traversal_exact_score_elapsed_us,
            grouped_traversal_budgeted_expansions,
            grouped_traversal_budgeted_candidates,
            grouped_traversal_budgeted_exact_candidates,
        ) = unsafe { am::debug_profile_ordered_scan(index_oid, grouped_v2_runtime_query()) };

        assert!(
            grouped_traversal_approx_score_calls > 0
                && grouped_traversal_approx_score_elapsed_us >= 0,
            "budgeted grouped exact traversal should still score grouped approximate candidates first",
        );
        assert!(
            grouped_traversal_exact_score_calls > 0
                && grouped_traversal_exact_score_elapsed_us >= 0,
            "budgeted grouped exact traversal should surface exact rescoring work",
        );
        assert!(
            score_cache_hits > 0 && score_cache_misses > 0,
            "budgeted grouped exact traversal should reuse cached exact scores after the first miss path",
        );
        assert!(
            grouped_traversal_budgeted_expansions > 0
                && grouped_traversal_budgeted_candidates
                    >= grouped_traversal_budgeted_exact_candidates,
            "budgeted grouped exact traversal should report the candidate sets it exact-rescored",
        );
        assert!(
            grouped_traversal_exact_score_calls >= grouped_traversal_budgeted_exact_candidates,
            "grouped exact traversal should include at least the budgeted exact rescoring calls, even if entry or seed scoring adds more",
        );
        assert_eq!(
            grouped_traversal_budgeted_expansions,
            grouped_traversal_budgeted_exact_candidates,
            "limit=1 should exact-rescore one grouped candidate per budgeted expansion",
        );
    }

    #[pg_test]
    #[should_panic(
        expected = "tqhnsw grouped-v2 exact traversal scope must be one of [all, layer0], got \"bogus\""
    )]
    fn test_grouped_v2_exact_traversal_rejects_invalid_scope_env() {
        let _lock = env_var_test_lock();
        let _build_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");
        let _scan_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1");
        let _exact_guard =
            ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL", "1");
        let _scope_guard = ScopedEnvVar::set(
            "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_SCOPE",
            "bogus",
        );

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_runtime_invalid_exact_scope (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 41 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 29 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_runtime_invalid_exact_scope VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_runtime_invalid_exact_scope_idx ON tqhnsw_grouped_v2_runtime_invalid_exact_scope USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let _ = Spi::get_one::<i64>(
            "SELECT id FROM tqhnsw_grouped_v2_runtime_invalid_exact_scope \
             ORDER BY embedding <#> ARRAY[0.5, 0.1, 0.4, -0.8, 0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, -0.1, -0.2, -0.3, -0.4]::real[] \
             LIMIT 1",
        )
        .expect("ordered scan should reach amrescan before rejecting invalid grouped exact traversal scope");
    }

    #[pg_test]
    #[should_panic(
        expected = "tqhnsw grouped-v2 exact traversal limit must be a positive integer, got bogus"
    )]
    fn test_grouped_v2_exact_traversal_rejects_invalid_limit_env() {
        let _lock = env_var_test_lock();
        let _build_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");
        let _scan_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1");
        let _exact_guard =
            ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL", "1");
        let _limit_guard = ScopedEnvVar::set(
            "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_LIMIT",
            "bogus",
        );

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_runtime_invalid_exact_limit (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 43 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 31 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_runtime_invalid_exact_limit VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_runtime_invalid_exact_limit_idx ON tqhnsw_grouped_v2_runtime_invalid_exact_limit USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let _ = Spi::get_one::<i64>(
            "SELECT id FROM tqhnsw_grouped_v2_runtime_invalid_exact_limit \
             ORDER BY embedding <#> ARRAY[0.5, 0.1, 0.4, -0.8, 0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, -0.1, -0.2, -0.3, -0.4]::real[] \
             LIMIT 1",
        )
        .expect("ordered scan should reach amrescan before rejecting invalid grouped exact traversal limit");
    }

    #[pg_test]
    fn test_grouped_v2_runtime_comparison_summary_matches_emitted_rows() {
        let _lock = env_var_test_lock();
        let _build_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");
        let _scan_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1");

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_runtime_summary (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 37 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 23 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_runtime_summary VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_runtime_summary_idx ON tqhnsw_grouped_v2_runtime_summary USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_grouped_v2_runtime_summary_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![
            0.15_f32, 0.25, 0.35, 0.45, 0.55, 0.65, 0.75, 0.85, 0.95, 1.05, 1.15, 1.25, 1.35, 1.45,
            1.55, 1.65,
        ];
        let observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_score_comparisons(index_oid, query.clone())
        };

        let compared_rows = observed
            .iter()
            .filter_map(
                |(_heap_tid, approx_score, comparison_score, _approx_rank)| {
                    comparison_score.map(|exact_score| (*approx_score, exact_score))
                },
            )
            .collect::<Vec<_>>();
        let expected_emitted_result_count =
            i32::try_from(observed.len()).expect("emitted result count should fit in i32");
        let expected_grouped_result_count = expected_emitted_result_count;
        let expected_compared_result_count =
            i32::try_from(compared_rows.len()).expect("compared result count should fit in i32");
        let expected_missing_comparison_count =
            expected_grouped_result_count - expected_compared_result_count;
        let expected_mean_abs_score_delta = if compared_rows.is_empty() {
            0.0
        } else {
            compared_rows
                .iter()
                .map(|(approx_score, exact_score)| f64::from((approx_score - exact_score).abs()))
                .sum::<f64>()
                / f64::from(expected_compared_result_count)
        };
        let expected_max_abs_score_delta = compared_rows
            .iter()
            .map(|(approx_score, exact_score)| (approx_score - exact_score).abs())
            .fold(0.0_f32, f32::max);
        let expected_mean_signed_score_delta = if compared_rows.is_empty() {
            0.0
        } else {
            compared_rows
                .iter()
                .map(|(approx_score, exact_score)| f64::from(approx_score - exact_score))
                .sum::<f64>()
                / f64::from(expected_compared_result_count)
        };

        let query_literal = format_recall_vector_sql_literal(&query);
        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            missing_comparison_count,
            mean_abs_score_delta,
            max_abs_score_delta,
            mean_signed_score_delta,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    &format!(
                        "SELECT
                            emitted_result_count,
                            grouped_result_count,
                            compared_result_count,
                            missing_comparison_count,
                            mean_abs_score_delta,
                            max_abs_score_delta,
                            mean_signed_score_delta
                         FROM tests.tqhnsw_debug_grouped_scan_comparison_summary(
                            'tqhnsw_grouped_v2_runtime_summary_idx'::regclass::oid,
                            {query_literal}
                         )"
                    ),
                    None,
                    &[],
                )
                .expect("grouped comparison summary query should succeed")
                .next()
                .expect("grouped comparison summary should return one row");
            (
                row["emitted_result_count"]
                    .value::<i32>()
                    .expect("emitted result count should decode")
                    .expect("emitted result count should be non-null"),
                row["grouped_result_count"]
                    .value::<i32>()
                    .expect("grouped result count should decode")
                    .expect("grouped result count should be non-null"),
                row["compared_result_count"]
                    .value::<i32>()
                    .expect("compared result count should decode")
                    .expect("compared result count should be non-null"),
                row["missing_comparison_count"]
                    .value::<i32>()
                    .expect("missing comparison count should decode")
                    .expect("missing comparison count should be non-null"),
                row["mean_abs_score_delta"]
                    .value::<f64>()
                    .expect("mean abs score delta should decode")
                    .expect("mean abs score delta should be non-null"),
                row["max_abs_score_delta"]
                    .value::<f32>()
                    .expect("max abs score delta should decode")
                    .expect("max abs score delta should be non-null"),
                row["mean_signed_score_delta"]
                    .value::<f64>()
                    .expect("mean signed score delta should decode")
                    .expect("mean signed score delta should be non-null"),
            )
        });

        assert_eq!(emitted_result_count, expected_emitted_result_count);
        assert_eq!(grouped_result_count, expected_grouped_result_count);
        assert_eq!(compared_result_count, expected_compared_result_count);
        assert_eq!(missing_comparison_count, expected_missing_comparison_count);
        assert!(
            (mean_abs_score_delta - expected_mean_abs_score_delta).abs() <= 1e-6,
            "mean abs grouped score delta should match the emitted-row summary"
        );
        assert!(
            (max_abs_score_delta - expected_max_abs_score_delta).abs() <= f32::EPSILON,
            "max abs grouped score delta should match the emitted-row summary"
        );
        assert!(
            (mean_signed_score_delta - expected_mean_signed_score_delta).abs() <= 1e-6,
            "mean signed grouped score delta should match the emitted-row summary"
        );
    }

    #[pg_test]
    fn test_scalar_runtime_summary_reports_no_grouped_comparisons() {
        Spi::run(
            "CREATE TABLE tqhnsw_scalar_runtime_summary (
                id bigint primary key,
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        Spi::run(
            "INSERT INTO tqhnsw_scalar_runtime_summary VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.5, 0.5, 0.25, -0.75], 4, 42)),
             (4, encode_to_tqvector(ARRAY[-0.25, 0.9, 0.1, -0.4], 4, 42))",
        )
        .expect("insert should succeed");

        Spi::run(
            "CREATE INDEX tqhnsw_scalar_runtime_summary_idx ON tqhnsw_scalar_runtime_summary USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            missing_comparison_count,
            mean_abs_score_delta,
            max_abs_score_delta,
            mean_signed_score_delta,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    "SELECT
                        emitted_result_count,
                        grouped_result_count,
                        compared_result_count,
                        missing_comparison_count,
                        mean_abs_score_delta,
                        max_abs_score_delta,
                        mean_signed_score_delta
                     FROM tests.tqhnsw_debug_grouped_scan_comparison_summary(
                        'tqhnsw_scalar_runtime_summary_idx'::regclass::oid,
                        ARRAY[1.0, 0.0, 0.5, -1.0]::real[]
                     )",
                    None,
                    &[],
                )
                .expect("scalar comparison summary query should succeed")
                .next()
                .expect("scalar comparison summary should return one row");
            (
                row["emitted_result_count"]
                    .value::<i32>()
                    .expect("emitted result count should decode")
                    .expect("emitted result count should be non-null"),
                row["grouped_result_count"]
                    .value::<i32>()
                    .expect("grouped result count should decode")
                    .expect("grouped result count should be non-null"),
                row["compared_result_count"]
                    .value::<i32>()
                    .expect("compared result count should decode")
                    .expect("compared result count should be non-null"),
                row["missing_comparison_count"]
                    .value::<i32>()
                    .expect("missing comparison count should decode")
                    .expect("missing comparison count should be non-null"),
                row["mean_abs_score_delta"]
                    .value::<f64>()
                    .expect("mean abs score delta should decode")
                    .expect("mean abs score delta should be non-null"),
                row["max_abs_score_delta"]
                    .value::<f32>()
                    .expect("max abs score delta should decode")
                    .expect("max abs score delta should be non-null"),
                row["mean_signed_score_delta"]
                    .value::<f64>()
                    .expect("mean signed score delta should decode")
                    .expect("mean signed score delta should be non-null"),
            )
        });

        assert!(emitted_result_count > 0);
        assert_eq!(grouped_result_count, 0);
        assert_eq!(compared_result_count, 0);
        assert_eq!(missing_comparison_count, 0);
        assert_eq!(mean_abs_score_delta, 0.0);
        assert_eq!(max_abs_score_delta, 0.0);
        assert_eq!(mean_signed_score_delta, 0.0);
    }

    #[pg_test]
    fn test_grouped_v2_runtime_comparison_rows_report_exact_ranks() {
        let _lock = env_var_test_lock();
        let _build_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");
        let _scan_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1");

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_runtime_comparison_rows (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 19 + dim) as f32) * 0.04).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 29 + dim) as f32) * 0.03).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_runtime_comparison_rows VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_runtime_comparison_rows_idx ON tqhnsw_grouped_v2_runtime_comparison_rows USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_grouped_v2_runtime_comparison_rows_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![
            0.05_f32, 0.15, 0.25, 0.35, 0.45, 0.55, 0.65, 0.75, 0.85, 0.95, 1.05, 1.15, 1.25, 1.35,
            1.45, 1.55,
        ];
        let observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_score_comparisons(index_oid, query.clone())
        };
        let mut expected_exact_ranks = vec![None; observed.len()];
        let mut ordered_observed = observed
            .iter()
            .enumerate()
            .map(
                |(
                    idx,
                    ((block_number, offset_number), approx_score, comparison_score, approx_rank),
                )| {
                    (
                        idx,
                        *block_number,
                        *offset_number,
                        *approx_score,
                        *comparison_score,
                        approx_rank.unwrap_or_else(|| {
                            i32::try_from(idx + 1).expect("approx rank should fit in i32")
                        }),
                    )
                },
            )
            .collect::<Vec<_>>();
        ordered_observed.sort_by_key(|row| row.5);
        let mut compared_rows = ordered_observed
            .iter()
            .enumerate()
            .filter_map(
                |(
                    idx,
                    (
                        _live_idx,
                        _block_number,
                        _offset_number,
                        _approx_score,
                        comparison_score,
                        _approx_rank,
                    ),
                )| { comparison_score.map(|exact_score| (idx, exact_score)) },
            )
            .collect::<Vec<_>>();
        compared_rows.sort_by(|(left_idx, left_score), (right_idx, right_score)| {
            let left_approx_rank = ordered_observed[*left_idx].5;
            let right_approx_rank = ordered_observed[*right_idx].5;
            left_score
                .total_cmp(right_score)
                .then_with(|| left_approx_rank.cmp(&right_approx_rank))
        });
        for (rank, (idx, _exact_score)) in compared_rows.into_iter().enumerate() {
            expected_exact_ranks[idx] =
                Some(i32::try_from(rank + 1).expect("exact rank should fit in i32"));
        }
        let expected_rows = ordered_observed
            .iter()
            .enumerate()
            .map(
                |(
                    idx,
                    (
                        _live_idx,
                        block_number,
                        offset_number,
                        approx_score,
                        comparison_score,
                        approx_rank,
                    ),
                )| {
                    let exact_rank = expected_exact_ranks[idx];
                    let exact_rank_shift = exact_rank.map(|rank| approx_rank - rank);
                    (
                        i64::from(*block_number),
                        i32::from(*offset_number),
                        *approx_rank,
                        *approx_score,
                        *comparison_score,
                        exact_rank,
                        exact_rank_shift,
                    )
                },
            )
            .collect::<Vec<_>>();

        let query_literal = format_recall_vector_sql_literal(&query);
        let actual_rows = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT
                            block_number,
                            offset_number,
                            approx_rank,
                            approx_score,
                            comparison_score,
                            exact_rank,
                            exact_rank_shift
                         FROM tests.tqhnsw_debug_grouped_scan_comparison_rows(
                            'tqhnsw_grouped_v2_runtime_comparison_rows_idx'::regclass::oid,
                            {query_literal}
                         )
                         ORDER BY approx_rank"
                    ),
                    None,
                    &[],
                )
                .expect("grouped comparison rows query should succeed")
                .map(|row| {
                    (
                        row["block_number"]
                            .value::<i64>()
                            .expect("block number should decode")
                            .expect("block number should be non-null"),
                        row["offset_number"]
                            .value::<i32>()
                            .expect("offset number should decode")
                            .expect("offset number should be non-null"),
                        row["approx_rank"]
                            .value::<i32>()
                            .expect("approx rank should decode")
                            .expect("approx rank should be non-null"),
                        row["approx_score"]
                            .value::<f32>()
                            .expect("approx score should decode")
                            .expect("approx score should be non-null"),
                        row["comparison_score"]
                            .value::<f32>()
                            .expect("comparison score should decode"),
                        row["exact_rank"]
                            .value::<i32>()
                            .expect("exact rank should decode"),
                        row["exact_rank_shift"]
                            .value::<i32>()
                            .expect("exact rank shift should decode"),
                    )
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(actual_rows.len(), expected_rows.len());
        for (actual, expected) in actual_rows.iter().zip(expected_rows.iter()) {
            assert_eq!(actual.0, expected.0);
            assert_eq!(actual.1, expected.1);
            assert_eq!(actual.2, expected.2);
            assert_eq!(actual.3.to_bits(), expected.3.to_bits());
            assert_eq!(
                actual.4.map(f32::to_bits),
                expected.4.map(f32::to_bits),
                "comparison score should preserve the emitted exact rerank score"
            );
            assert_eq!(actual.5, expected.5);
            assert_eq!(actual.6, expected.6);
        }
    }

    #[pg_test]
    fn test_scalar_runtime_comparison_rows_leave_exact_order_null() {
        Spi::run(
            "CREATE TABLE tqhnsw_scalar_runtime_comparison_rows (
                id bigint primary key,
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        Spi::run(
            "INSERT INTO tqhnsw_scalar_runtime_comparison_rows VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.5, 0.5, 0.25, -0.75], 4, 42)),
             (4, encode_to_tqvector(ARRAY[-0.25, 0.9, 0.1, -0.4], 4, 42))",
        )
        .expect("insert should succeed");

        Spi::run(
            "CREATE INDEX tqhnsw_scalar_runtime_comparison_rows_idx ON tqhnsw_scalar_runtime_comparison_rows USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let rows = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        approx_rank,
                        comparison_score,
                        exact_rank,
                        exact_rank_shift
                     FROM tests.tqhnsw_debug_grouped_scan_comparison_rows(
                        'tqhnsw_scalar_runtime_comparison_rows_idx'::regclass::oid,
                        ARRAY[1.0, 0.0, 0.5, -1.0]::real[]
                     )
                     ORDER BY approx_rank",
                    None,
                    &[],
                )
                .expect("scalar comparison rows query should succeed")
                .map(|row| {
                    (
                        row["approx_rank"]
                            .value::<i32>()
                            .expect("approx rank should decode")
                            .expect("approx rank should be non-null"),
                        row["comparison_score"]
                            .value::<f32>()
                            .expect("comparison score should decode"),
                        row["exact_rank"]
                            .value::<i32>()
                            .expect("exact rank should decode"),
                        row["exact_rank_shift"]
                            .value::<i32>()
                            .expect("exact rank shift should decode"),
                    )
                })
                .collect::<Vec<_>>()
        });

        assert!(!rows.is_empty());
        for (idx, (approx_rank, comparison_score, exact_rank, exact_rank_shift)) in
            rows.iter().enumerate()
        {
            assert_eq!(
                *approx_rank,
                i32::try_from(idx + 1).expect("approx rank should fit in i32")
            );
            assert_eq!(*comparison_score, None);
            assert_eq!(*exact_rank, None);
            assert_eq!(*exact_rank_shift, None);
        }
    }

    #[pg_test]
    fn test_grouped_v2_order_drift_summary_matches_rows() {
        let _lock = env_var_test_lock();
        let _build_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");
        let _scan_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1");

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_runtime_order_drift_summary (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 31 + dim) as f32) * 0.025).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 17 + dim) as f32) * 0.035).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_runtime_order_drift_summary VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_runtime_order_drift_summary_idx ON tqhnsw_grouped_v2_runtime_order_drift_summary USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_grouped_v2_runtime_order_drift_summary_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![
            0.12_f32, 0.22, 0.32, 0.42, 0.52, 0.62, 0.72, 0.82, 0.92, 1.02, 1.12, 1.22, 1.32, 1.42,
            1.52, 1.62,
        ];
        let observed = unsafe { am::debug_grouped_scan_comparison_rows(index_oid, query.clone()) };
        let expected_emitted_result_count =
            i32::try_from(observed.len()).expect("emitted result count should fit in i32");
        let expected_grouped_result_count = expected_emitted_result_count;
        let mut expected_compared_result_count = 0_i32;
        let mut abs_rank_shift_sum = 0.0_f64;
        let mut expected_max_abs_rank_shift = 0_i32;
        let mut d_squared_sum = 0.0_f64;
        let mut expected_exact_best_approx_rank = None;
        let mut expected_exact_top4_max_approx_rank = None;

        for (
            _heap_tid,
            approx_rank,
            _approx_score,
            _comparison_score,
            exact_rank,
            exact_rank_shift,
        ) in &observed
        {
            let Some(exact_rank) = exact_rank else {
                continue;
            };
            expected_compared_result_count += 1;
            let rank_shift =
                exact_rank_shift.expect("grouped comparison rows should populate exact rank shift");
            let abs_rank_shift = rank_shift.abs();
            abs_rank_shift_sum += f64::from(abs_rank_shift);
            expected_max_abs_rank_shift = expected_max_abs_rank_shift.max(abs_rank_shift);
            let d = f64::from(*approx_rank - *exact_rank);
            d_squared_sum += d * d;
            if *exact_rank == 1 {
                expected_exact_best_approx_rank = Some(*approx_rank);
            }
            if *exact_rank <= 4 {
                expected_exact_top4_max_approx_rank = Some(
                    expected_exact_top4_max_approx_rank
                        .map_or(*approx_rank, |max_rank: i32| max_rank.max(*approx_rank)),
                );
            }
        }

        let expected_mean_abs_rank_shift = if expected_compared_result_count == 0 {
            0.0
        } else {
            abs_rank_shift_sum / f64::from(expected_compared_result_count)
        };
        let expected_spearman_rank_correlation = if expected_compared_result_count < 2 {
            0.0
        } else {
            let n = f64::from(expected_compared_result_count);
            1.0 - (6.0 * d_squared_sum / (n * (n * n - 1.0)))
        };
        let expected_window_1_contains_exact_best =
            expected_exact_best_approx_rank.is_some_and(|rank| rank <= 1);
        let expected_window_2_contains_exact_best =
            expected_exact_best_approx_rank.is_some_and(|rank| rank <= 2);
        let expected_window_4_contains_exact_best =
            expected_exact_best_approx_rank.is_some_and(|rank| rank <= 4);
        let expected_window_8_contains_exact_best =
            expected_exact_best_approx_rank.is_some_and(|rank| rank <= 8);

        let query_literal = format_recall_vector_sql_literal(&query);
        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            mean_abs_rank_shift,
            max_abs_rank_shift,
            spearman_rank_correlation,
            exact_best_approx_rank,
            exact_top4_max_approx_rank,
            window_1_contains_exact_best,
            window_2_contains_exact_best,
            window_4_contains_exact_best,
            window_8_contains_exact_best,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    &format!(
                        "SELECT
                            emitted_result_count,
                            grouped_result_count,
                            compared_result_count,
                            mean_abs_rank_shift,
                            max_abs_rank_shift,
                            spearman_rank_correlation,
                            exact_best_approx_rank,
                            exact_top4_max_approx_rank,
                            window_1_contains_exact_best,
                            window_2_contains_exact_best,
                            window_4_contains_exact_best,
                            window_8_contains_exact_best
                         FROM tests.tqhnsw_debug_grouped_scan_order_drift_summary(
                            'tqhnsw_grouped_v2_runtime_order_drift_summary_idx'::regclass::oid,
                            {query_literal}
                         )"
                    ),
                    None,
                    &[],
                )
                .expect("grouped order drift summary query should succeed")
                .next()
                .expect("grouped order drift summary should return one row");
            (
                row["emitted_result_count"]
                    .value::<i32>()
                    .expect("emitted result count should decode")
                    .expect("emitted result count should be non-null"),
                row["grouped_result_count"]
                    .value::<i32>()
                    .expect("grouped result count should decode")
                    .expect("grouped result count should be non-null"),
                row["compared_result_count"]
                    .value::<i32>()
                    .expect("compared result count should decode")
                    .expect("compared result count should be non-null"),
                row["mean_abs_rank_shift"]
                    .value::<f64>()
                    .expect("mean abs rank shift should decode")
                    .expect("mean abs rank shift should be non-null"),
                row["max_abs_rank_shift"]
                    .value::<i32>()
                    .expect("max abs rank shift should decode")
                    .expect("max abs rank shift should be non-null"),
                row["spearman_rank_correlation"]
                    .value::<f64>()
                    .expect("spearman rank correlation should decode")
                    .expect("spearman rank correlation should be non-null"),
                row["exact_best_approx_rank"]
                    .value::<i32>()
                    .expect("exact best approx rank should decode"),
                row["exact_top4_max_approx_rank"]
                    .value::<i32>()
                    .expect("exact top4 max approx rank should decode"),
                row["window_1_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 1 flag should decode")
                    .expect("window 1 flag should be non-null"),
                row["window_2_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 2 flag should decode")
                    .expect("window 2 flag should be non-null"),
                row["window_4_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 4 flag should decode")
                    .expect("window 4 flag should be non-null"),
                row["window_8_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 8 flag should decode")
                    .expect("window 8 flag should be non-null"),
            )
        });

        assert_eq!(emitted_result_count, expected_emitted_result_count);
        assert_eq!(grouped_result_count, expected_grouped_result_count);
        assert_eq!(compared_result_count, expected_compared_result_count);
        assert!(
            (mean_abs_rank_shift - expected_mean_abs_rank_shift).abs() <= 1e-6,
            "mean abs rank shift should match the emitted-row order summary"
        );
        assert_eq!(max_abs_rank_shift, expected_max_abs_rank_shift);
        assert!(
            (spearman_rank_correlation - expected_spearman_rank_correlation).abs() <= 1e-6,
            "spearman rank correlation should match the emitted-row order summary"
        );
        assert_eq!(exact_best_approx_rank, expected_exact_best_approx_rank);
        assert_eq!(
            exact_top4_max_approx_rank,
            expected_exact_top4_max_approx_rank
        );
        assert_eq!(
            window_1_contains_exact_best,
            expected_window_1_contains_exact_best
        );
        assert_eq!(
            window_2_contains_exact_best,
            expected_window_2_contains_exact_best
        );
        assert_eq!(
            window_4_contains_exact_best,
            expected_window_4_contains_exact_best
        );
        assert_eq!(
            window_8_contains_exact_best,
            expected_window_8_contains_exact_best
        );
    }

    #[pg_test]
    fn test_scalar_order_drift_summary_is_inert() {
        Spi::run(
            "CREATE TABLE tqhnsw_scalar_runtime_order_drift_summary (
                id bigint primary key,
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        Spi::run(
            "INSERT INTO tqhnsw_scalar_runtime_order_drift_summary VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.5, 0.5, 0.25, -0.75], 4, 42)),
             (4, encode_to_tqvector(ARRAY[-0.25, 0.9, 0.1, -0.4], 4, 42))",
        )
        .expect("insert should succeed");

        Spi::run(
            "CREATE INDEX tqhnsw_scalar_runtime_order_drift_summary_idx ON tqhnsw_scalar_runtime_order_drift_summary USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            mean_abs_rank_shift,
            max_abs_rank_shift,
            spearman_rank_correlation,
            exact_best_approx_rank,
            exact_top4_max_approx_rank,
            window_1_contains_exact_best,
            window_2_contains_exact_best,
            window_4_contains_exact_best,
            window_8_contains_exact_best,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    "SELECT
                        emitted_result_count,
                        grouped_result_count,
                        compared_result_count,
                        mean_abs_rank_shift,
                        max_abs_rank_shift,
                        spearman_rank_correlation,
                        exact_best_approx_rank,
                        exact_top4_max_approx_rank,
                        window_1_contains_exact_best,
                        window_2_contains_exact_best,
                        window_4_contains_exact_best,
                        window_8_contains_exact_best
                     FROM tests.tqhnsw_debug_grouped_scan_order_drift_summary(
                        'tqhnsw_scalar_runtime_order_drift_summary_idx'::regclass::oid,
                        ARRAY[1.0, 0.0, 0.5, -1.0]::real[]
                     )",
                    None,
                    &[],
                )
                .expect("scalar order drift summary query should succeed")
                .next()
                .expect("scalar order drift summary should return one row");
            (
                row["emitted_result_count"]
                    .value::<i32>()
                    .expect("emitted result count should decode")
                    .expect("emitted result count should be non-null"),
                row["grouped_result_count"]
                    .value::<i32>()
                    .expect("grouped result count should decode")
                    .expect("grouped result count should be non-null"),
                row["compared_result_count"]
                    .value::<i32>()
                    .expect("compared result count should decode")
                    .expect("compared result count should be non-null"),
                row["mean_abs_rank_shift"]
                    .value::<f64>()
                    .expect("mean abs rank shift should decode")
                    .expect("mean abs rank shift should be non-null"),
                row["max_abs_rank_shift"]
                    .value::<i32>()
                    .expect("max abs rank shift should decode")
                    .expect("max abs rank shift should be non-null"),
                row["spearman_rank_correlation"]
                    .value::<f64>()
                    .expect("spearman rank correlation should decode")
                    .expect("spearman rank correlation should be non-null"),
                row["exact_best_approx_rank"]
                    .value::<i32>()
                    .expect("exact best approx rank should decode"),
                row["exact_top4_max_approx_rank"]
                    .value::<i32>()
                    .expect("exact top4 max approx rank should decode"),
                row["window_1_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 1 flag should decode")
                    .expect("window 1 flag should be non-null"),
                row["window_2_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 2 flag should decode")
                    .expect("window 2 flag should be non-null"),
                row["window_4_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 4 flag should decode")
                    .expect("window 4 flag should be non-null"),
                row["window_8_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 8 flag should decode")
                    .expect("window 8 flag should be non-null"),
            )
        });

        assert!(emitted_result_count > 0);
        assert_eq!(grouped_result_count, 0);
        assert_eq!(compared_result_count, 0);
        assert_eq!(mean_abs_rank_shift, 0.0);
        assert_eq!(max_abs_rank_shift, 0);
        assert_eq!(spearman_rank_correlation, 0.0);
        assert_eq!(exact_best_approx_rank, None);
        assert_eq!(exact_top4_max_approx_rank, None);
        assert!(!window_1_contains_exact_best);
        assert!(!window_2_contains_exact_best);
        assert!(!window_4_contains_exact_best);
        assert!(!window_8_contains_exact_best);
    }

    #[pg_test]
    fn test_grouped_v2_windowed_rows_match_simulation() {
        let _lock = env_var_test_lock();
        let _build_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");
        let _scan_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1");

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_runtime_windowed_rows (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 41 + dim) as f32) * 0.02).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 13 + dim) as f32) * 0.04).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_runtime_windowed_rows VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_runtime_windowed_rows_idx ON tqhnsw_grouped_v2_runtime_windowed_rows USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_grouped_v2_runtime_windowed_rows_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![
            0.14_f32, 0.24, 0.34, 0.44, 0.54, 0.64, 0.74, 0.84, 0.94, 1.04, 1.14, 1.24, 1.34, 1.44,
            1.54, 1.64,
        ];
        let baseline_rows =
            unsafe { am::debug_grouped_scan_comparison_rows(index_oid, query.clone()) };
        let window_size = 4_usize;
        let mut buffered_rows = Vec::with_capacity(window_size);
        let mut next_idx = 0usize;
        let mut expected_rows = Vec::with_capacity(baseline_rows.len());
        while expected_rows.len() < baseline_rows.len() {
            while buffered_rows.len() < window_size && next_idx < baseline_rows.len() {
                buffered_rows.push(baseline_rows[next_idx]);
                next_idx += 1;
            }
            let (selected_idx, _) = buffered_rows
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    left.3
                        .unwrap_or(left.2)
                        .total_cmp(&right.3.unwrap_or(right.2))
                        .then_with(|| left.1.cmp(&right.1))
                })
                .expect("windowed grouped simulation should always have a buffered row");
            let (
                (block_number, offset_number),
                approx_rank,
                approx_score,
                comparison_score,
                exact_rank,
                exact_rank_shift,
            ) = buffered_rows.remove(selected_idx);
            let windowed_rank =
                i32::try_from(expected_rows.len() + 1).expect("windowed rank should fit in i32");
            let windowed_rank_shift = exact_rank.map(|rank| windowed_rank - rank);
            expected_rows.push((
                i64::from(block_number),
                i32::from(offset_number),
                approx_rank,
                windowed_rank,
                approx_score,
                comparison_score,
                exact_rank,
                exact_rank_shift,
                windowed_rank_shift,
            ));
        }

        let query_literal = format_recall_vector_sql_literal(&query);
        let actual_rows = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT
                            block_number,
                            offset_number,
                            approx_rank,
                            windowed_rank,
                            approx_score,
                            comparison_score,
                            exact_rank,
                            exact_rank_shift,
                            windowed_rank_shift
                         FROM tests.tqhnsw_debug_grouped_scan_windowed_rows(
                            'tqhnsw_grouped_v2_runtime_windowed_rows_idx'::regclass::oid,
                            {query_literal},
                            4
                         )
                         ORDER BY windowed_rank"
                    ),
                    None,
                    &[],
                )
                .expect("grouped windowed rows query should succeed")
                .map(|row| {
                    (
                        row["block_number"]
                            .value::<i64>()
                            .expect("block number should decode")
                            .expect("block number should be non-null"),
                        row["offset_number"]
                            .value::<i32>()
                            .expect("offset number should decode")
                            .expect("offset number should be non-null"),
                        row["approx_rank"]
                            .value::<i32>()
                            .expect("approx rank should decode")
                            .expect("approx rank should be non-null"),
                        row["windowed_rank"]
                            .value::<i32>()
                            .expect("windowed rank should decode")
                            .expect("windowed rank should be non-null"),
                        row["approx_score"]
                            .value::<f32>()
                            .expect("approx score should decode")
                            .expect("approx score should be non-null"),
                        row["comparison_score"]
                            .value::<f32>()
                            .expect("comparison score should decode"),
                        row["exact_rank"]
                            .value::<i32>()
                            .expect("exact rank should decode"),
                        row["exact_rank_shift"]
                            .value::<i32>()
                            .expect("exact rank shift should decode"),
                        row["windowed_rank_shift"]
                            .value::<i32>()
                            .expect("windowed rank shift should decode"),
                    )
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(actual_rows.len(), expected_rows.len());
        for (actual, expected) in actual_rows.iter().zip(expected_rows.iter()) {
            assert_eq!(actual.0, expected.0);
            assert_eq!(actual.1, expected.1);
            assert_eq!(actual.2, expected.2);
            assert_eq!(actual.3, expected.3);
            assert_eq!(actual.4.to_bits(), expected.4.to_bits());
            assert_eq!(actual.5.map(f32::to_bits), expected.5.map(f32::to_bits));
            assert_eq!(actual.6, expected.6);
            assert_eq!(actual.7, expected.7);
            assert_eq!(actual.8, expected.8);
        }
    }

    fn assert_grouped_v2_runtime_live_window_matches_windowed_simulation(
        window_size: i32,
        configure_window_env: bool,
    ) {
        let _lock = env_var_test_lock();
        let _build_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");
        let _scan_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1");
        let window_value = window_size.to_string();
        let _window_guard = configure_window_env.then(|| {
            ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_WINDOW", &window_value)
        });

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_runtime_live_window (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=32 {
            let source = (0..16)
                .map(|dim| {
                    format!(
                        "{:.6}",
                        (((id * 43 + dim * 7) as f32) * 0.019).cos()
                            + (((id * 17 + dim * 5) as f32) * 0.011).sin()
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| {
                    format!(
                        "{:.6}",
                        (((id * 29 + dim * 11) as f32) * 0.023).sin()
                            + (((id * 13 + dim * 3) as f32) * 0.017).cos()
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_runtime_live_window VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_runtime_live_window_idx ON tqhnsw_grouped_v2_runtime_live_window USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_grouped_v2_runtime_live_window_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let candidate_queries = (0..24)
            .map(|seed| {
                (0..16)
                    .map(|dim| {
                        (((seed * 31 + dim * 7) as f32) * 0.021).sin()
                            + (((seed * 19 + dim * 5) as f32) * 0.014).cos()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let (query, expected_live_order, live_rows, baseline_rows) = candidate_queries
            .into_iter()
            .find_map(|query| {
                let windowed_rows = unsafe {
                    am::debug_grouped_scan_windowed_rows(index_oid, query.clone(), window_size)
                };
                if !windowed_rows.iter().any(
                    |(_heap_tid, approx_rank, windowed_rank, _, _, _, _, _)| {
                        approx_rank != windowed_rank
                    },
                ) {
                    return None;
                }

                let expected_live_order = windowed_rows
                    .iter()
                    .map(
                        |(
                            heap_tid,
                            _approx_rank,
                            _windowed_rank,
                            _approx_score,
                            _comparison_score,
                            _exact_rank,
                            _exact_rank_shift,
                            _windowed_rank_shift,
                        )| *heap_tid,
                    )
                    .collect::<Vec<_>>();
                let live_rows = unsafe {
                    am::debug_gettuple_scan_heap_tids_with_score_comparisons(
                        index_oid,
                        query.clone(),
                    )
                };
                let baseline_rows =
                    unsafe { am::debug_grouped_scan_comparison_rows(index_oid, query.clone()) };
                Some((query, expected_live_order, live_rows, baseline_rows))
            })
            .expect("at least one deterministic grouped query should exhibit live window movement");

        let actual_live_order = live_rows
            .iter()
            .map(|(heap_tid, _approx_score, _comparison_score, _approx_rank)| *heap_tid)
            .collect::<Vec<_>>();
        assert_eq!(
            actual_live_order, expected_live_order,
            "grouped live runtime order should match the window-size-{window_size} simulation"
        );

        let baseline_approx_order = baseline_rows
            .iter()
            .map(
                |(
                    heap_tid,
                    _approx_rank,
                    _approx_score,
                    _comparison_score,
                    _exact_rank,
                    _exact_rank_shift,
                )| *heap_tid,
            )
            .collect::<Vec<_>>();
        assert_ne!(
            actual_live_order, baseline_approx_order,
            "the selected query should prove the live grouped rerank window changes output order"
        );

        let mut live_rows_sorted_by_approx_rank = live_rows.clone();
        live_rows_sorted_by_approx_rank.sort_by_key(
            |(_heap_tid, _approx_score, _comparison_score, approx_rank)| {
                approx_rank.expect(
                    "grouped live results should preserve baseline approximate rank sidecars",
                )
            },
        );
        let preserved_approx_order = live_rows_sorted_by_approx_rank
            .iter()
            .map(|(heap_tid, _approx_score, _comparison_score, _approx_rank)| *heap_tid)
            .collect::<Vec<_>>();
        assert_eq!(
            preserved_approx_order, baseline_approx_order,
            "grouped comparison rows should still expose baseline approximate order after live rerank cutover"
        );

        let _query_literal = format_recall_vector_sql_literal(&query);
    }

    #[pg_test]
    fn test_grouped_v2_runtime_live_window_matches_windowed_simulation() {
        assert_grouped_v2_runtime_live_window_matches_windowed_simulation(4, false);
    }

    #[pg_test]
    fn test_grouped_v2_runtime_live_window_respects_window_env() {
        assert_grouped_v2_runtime_live_window_matches_windowed_simulation(8, true);
    }

    #[pg_test]
    fn test_scalar_windowed_rows_are_inert() {
        Spi::run(
            "CREATE TABLE tqhnsw_scalar_runtime_windowed_rows (
                id bigint primary key,
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        Spi::run(
            "INSERT INTO tqhnsw_scalar_runtime_windowed_rows VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.5, 0.5, 0.25, -0.75], 4, 42)),
             (4, encode_to_tqvector(ARRAY[-0.25, 0.9, 0.1, -0.4], 4, 42))",
        )
        .expect("insert should succeed");

        Spi::run(
            "CREATE INDEX tqhnsw_scalar_runtime_windowed_rows_idx ON tqhnsw_scalar_runtime_windowed_rows USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let rows = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        approx_rank,
                        windowed_rank,
                        comparison_score,
                        exact_rank,
                        exact_rank_shift,
                        windowed_rank_shift
                     FROM tests.tqhnsw_debug_grouped_scan_windowed_rows(
                        'tqhnsw_scalar_runtime_windowed_rows_idx'::regclass::oid,
                        ARRAY[1.0, 0.0, 0.5, -1.0]::real[],
                        4
                     )
                     ORDER BY windowed_rank",
                    None,
                    &[],
                )
                .expect("scalar windowed rows query should succeed")
                .map(|row| {
                    (
                        row["approx_rank"]
                            .value::<i32>()
                            .expect("approx rank should decode")
                            .expect("approx rank should be non-null"),
                        row["windowed_rank"]
                            .value::<i32>()
                            .expect("windowed rank should decode")
                            .expect("windowed rank should be non-null"),
                        row["comparison_score"]
                            .value::<f32>()
                            .expect("comparison score should decode"),
                        row["exact_rank"]
                            .value::<i32>()
                            .expect("exact rank should decode"),
                        row["exact_rank_shift"]
                            .value::<i32>()
                            .expect("exact rank shift should decode"),
                        row["windowed_rank_shift"]
                            .value::<i32>()
                            .expect("windowed rank shift should decode"),
                    )
                })
                .collect::<Vec<_>>()
        });

        assert!(!rows.is_empty());
        for (
            idx,
            (
                approx_rank,
                windowed_rank,
                comparison_score,
                exact_rank,
                exact_rank_shift,
                windowed_rank_shift,
            ),
        ) in rows.iter().enumerate()
        {
            let expected_rank = i32::try_from(idx + 1).expect("rank should fit in i32");
            assert_eq!(*approx_rank, expected_rank);
            assert_eq!(*windowed_rank, expected_rank);
            assert_eq!(*comparison_score, None);
            assert_eq!(*exact_rank, None);
            assert_eq!(*exact_rank_shift, None);
            assert_eq!(*windowed_rank_shift, None);
        }
    }

    #[pg_test]
    fn test_grouped_v2_windowed_summary_matches_rows() {
        let _lock = env_var_test_lock();
        let _build_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");
        let _scan_guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1");

        Spi::run(
            "CREATE TABLE tqhnsw_grouped_v2_runtime_windowed_summary (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 23 + dim) as f32) * 0.035).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 31 + dim) as f32) * 0.025).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_grouped_v2_runtime_windowed_summary VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_grouped_v2_runtime_windowed_summary_idx ON tqhnsw_grouped_v2_runtime_windowed_summary USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_grouped_v2_runtime_windowed_summary_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![
            0.16_f32, 0.26, 0.36, 0.46, 0.56, 0.66, 0.76, 0.86, 0.96, 1.06, 1.16, 1.26, 1.36, 1.46,
            1.56, 1.66,
        ];
        let baseline_rows =
            unsafe { am::debug_grouped_scan_comparison_rows(index_oid, query.clone()) };
        let windowed_rows =
            unsafe { am::debug_grouped_scan_windowed_rows(index_oid, query.clone(), 4) };

        let rank_metrics = |rows: &[(i32, Option<i32>, Option<i32>)]| {
            let mut compared_result_count = 0_i32;
            let mut abs_rank_shift_sum = 0.0_f64;
            let mut max_abs_rank_shift = 0_i32;
            let mut d_squared_sum = 0.0_f64;
            let mut exact_best_rank = None;
            let mut exact_top4_max_rank = None;

            for (observed_rank, exact_rank, explicit_rank_shift) in rows {
                let Some(exact_rank) = exact_rank else {
                    continue;
                };
                compared_result_count += 1;
                let abs_rank_shift = explicit_rank_shift
                    .unwrap_or(observed_rank - exact_rank)
                    .abs();
                abs_rank_shift_sum += f64::from(abs_rank_shift);
                max_abs_rank_shift = max_abs_rank_shift.max(abs_rank_shift);
                let d = f64::from(observed_rank - exact_rank);
                d_squared_sum += d * d;
                if *exact_rank == 1 {
                    exact_best_rank = Some(*observed_rank);
                }
                if *exact_rank <= 4 {
                    exact_top4_max_rank = Some(
                        exact_top4_max_rank
                            .map_or(*observed_rank, |max_rank: i32| max_rank.max(*observed_rank)),
                    );
                }
            }

            let mean_abs_rank_shift = if compared_result_count == 0 {
                0.0
            } else {
                abs_rank_shift_sum / f64::from(compared_result_count)
            };
            let spearman_rank_correlation = if compared_result_count < 2 {
                0.0
            } else {
                let n = f64::from(compared_result_count);
                1.0 - (6.0 * d_squared_sum / (n * (n * n - 1.0)))
            };

            (
                compared_result_count,
                mean_abs_rank_shift,
                max_abs_rank_shift,
                spearman_rank_correlation,
                exact_best_rank,
                exact_top4_max_rank,
            )
        };

        let expected_emitted_result_count =
            i32::try_from(baseline_rows.len()).expect("emitted result count should fit in i32");
        let expected_grouped_result_count = expected_emitted_result_count;
        let baseline_metric_rows = baseline_rows
            .iter()
            .map(
                |(
                    _heap_tid,
                    approx_rank,
                    _approx_score,
                    _comparison_score,
                    exact_rank,
                    exact_rank_shift,
                )| { (*approx_rank, *exact_rank, *exact_rank_shift) },
            )
            .collect::<Vec<_>>();
        let windowed_metric_rows = windowed_rows
            .iter()
            .map(
                |(
                    _heap_tid,
                    _approx_rank,
                    windowed_rank,
                    _approx_score,
                    _comparison_score,
                    exact_rank,
                    _exact_rank_shift,
                    windowed_rank_shift,
                )| (*windowed_rank, *exact_rank, *windowed_rank_shift),
            )
            .collect::<Vec<_>>();
        let (
            expected_compared_result_count,
            expected_mean_abs_rank_shift_before,
            expected_max_abs_rank_shift_before,
            expected_spearman_before,
            expected_exact_best_approx_rank,
            expected_exact_top4_max_approx_rank,
        ) = rank_metrics(&baseline_metric_rows);
        let (
            _windowed_compared_result_count,
            expected_mean_abs_rank_shift_after,
            expected_max_abs_rank_shift_after,
            expected_spearman_after,
            expected_exact_best_windowed_rank,
            expected_exact_top4_max_windowed_rank,
        ) = rank_metrics(&windowed_metric_rows);

        let query_literal = format_recall_vector_sql_literal(&query);
        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            window_size,
            exact_best_approx_rank,
            exact_best_windowed_rank,
            exact_top4_max_approx_rank,
            exact_top4_max_windowed_rank,
            mean_abs_rank_shift_before,
            mean_abs_rank_shift_after,
            max_abs_rank_shift_before,
            max_abs_rank_shift_after,
            spearman_before,
            spearman_after,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    &format!(
                        "SELECT
                            emitted_result_count,
                            grouped_result_count,
                            compared_result_count,
                            window_size,
                            exact_best_approx_rank,
                            exact_best_windowed_rank,
                            exact_top4_max_approx_rank,
                            exact_top4_max_windowed_rank,
                            mean_abs_rank_shift_before,
                            mean_abs_rank_shift_after,
                            max_abs_rank_shift_before,
                            max_abs_rank_shift_after,
                            spearman_rank_correlation_before,
                            spearman_rank_correlation_after
                         FROM tests.tqhnsw_debug_grouped_scan_windowed_summary(
                            'tqhnsw_grouped_v2_runtime_windowed_summary_idx'::regclass::oid,
                            {query_literal},
                            4
                         )"
                    ),
                    None,
                    &[],
                )
                .expect("grouped windowed summary query should succeed")
                .next()
                .expect("grouped windowed summary should return one row");
            (
                row["emitted_result_count"]
                    .value::<i32>()
                    .expect("emitted result count should decode")
                    .expect("emitted result count should be non-null"),
                row["grouped_result_count"]
                    .value::<i32>()
                    .expect("grouped result count should decode")
                    .expect("grouped result count should be non-null"),
                row["compared_result_count"]
                    .value::<i32>()
                    .expect("compared result count should decode")
                    .expect("compared result count should be non-null"),
                row["window_size"]
                    .value::<i32>()
                    .expect("window size should decode")
                    .expect("window size should be non-null"),
                row["exact_best_approx_rank"]
                    .value::<i32>()
                    .expect("exact best approx rank should decode"),
                row["exact_best_windowed_rank"]
                    .value::<i32>()
                    .expect("exact best windowed rank should decode"),
                row["exact_top4_max_approx_rank"]
                    .value::<i32>()
                    .expect("exact top4 max approx rank should decode"),
                row["exact_top4_max_windowed_rank"]
                    .value::<i32>()
                    .expect("exact top4 max windowed rank should decode"),
                row["mean_abs_rank_shift_before"]
                    .value::<f64>()
                    .expect("mean abs rank shift before should decode")
                    .expect("mean abs rank shift before should be non-null"),
                row["mean_abs_rank_shift_after"]
                    .value::<f64>()
                    .expect("mean abs rank shift after should decode")
                    .expect("mean abs rank shift after should be non-null"),
                row["max_abs_rank_shift_before"]
                    .value::<i32>()
                    .expect("max abs rank shift before should decode")
                    .expect("max abs rank shift before should be non-null"),
                row["max_abs_rank_shift_after"]
                    .value::<i32>()
                    .expect("max abs rank shift after should decode")
                    .expect("max abs rank shift after should be non-null"),
                row["spearman_rank_correlation_before"]
                    .value::<f64>()
                    .expect("spearman rank correlation before should decode")
                    .expect("spearman rank correlation before should be non-null"),
                row["spearman_rank_correlation_after"]
                    .value::<f64>()
                    .expect("spearman rank correlation after should decode")
                    .expect("spearman rank correlation after should be non-null"),
            )
        });

        assert_eq!(emitted_result_count, expected_emitted_result_count);
        assert_eq!(grouped_result_count, expected_grouped_result_count);
        assert_eq!(compared_result_count, expected_compared_result_count);
        assert_eq!(window_size, 4);
        assert_eq!(exact_best_approx_rank, expected_exact_best_approx_rank);
        assert_eq!(exact_best_windowed_rank, expected_exact_best_windowed_rank);
        assert_eq!(
            exact_top4_max_approx_rank,
            expected_exact_top4_max_approx_rank
        );
        assert_eq!(
            exact_top4_max_windowed_rank,
            expected_exact_top4_max_windowed_rank
        );
        assert!(
            (mean_abs_rank_shift_before - expected_mean_abs_rank_shift_before).abs() <= 1e-6,
            "baseline mean abs rank shift should match the row aggregation"
        );
        assert!(
            (mean_abs_rank_shift_after - expected_mean_abs_rank_shift_after).abs() <= 1e-6,
            "windowed mean abs rank shift should match the row aggregation"
        );
        assert_eq!(
            max_abs_rank_shift_before,
            expected_max_abs_rank_shift_before
        );
        assert_eq!(max_abs_rank_shift_after, expected_max_abs_rank_shift_after);
        assert!(
            (spearman_before - expected_spearman_before).abs() <= 1e-6,
            "baseline spearman should match the row aggregation"
        );
        assert!(
            (spearman_after - expected_spearman_after).abs() <= 1e-6,
            "windowed spearman should match the row aggregation"
        );
        if let (Some(approx_rank), Some(windowed_rank)) =
            (exact_best_approx_rank, exact_best_windowed_rank)
        {
            assert!(
                windowed_rank <= approx_rank,
                "a sliding rerank window should not push the exact-best emitted row later than its baseline approximate rank"
            );
        }
    }

    #[pg_test]
    fn test_scalar_windowed_summary_is_inert() {
        Spi::run(
            "CREATE TABLE tqhnsw_scalar_runtime_windowed_summary (
                id bigint primary key,
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");

        Spi::run(
            "INSERT INTO tqhnsw_scalar_runtime_windowed_summary VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.5, 0.5, 0.25, -0.75], 4, 42)),
             (4, encode_to_tqvector(ARRAY[-0.25, 0.9, 0.1, -0.4], 4, 42))",
        )
        .expect("insert should succeed");

        Spi::run(
            "CREATE INDEX tqhnsw_scalar_runtime_windowed_summary_idx ON tqhnsw_scalar_runtime_windowed_summary USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            window_size,
            exact_best_approx_rank,
            exact_best_windowed_rank,
            exact_top4_max_approx_rank,
            exact_top4_max_windowed_rank,
            mean_abs_rank_shift_before,
            mean_abs_rank_shift_after,
            max_abs_rank_shift_before,
            max_abs_rank_shift_after,
            spearman_before,
            spearman_after,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    "SELECT
                        emitted_result_count,
                        grouped_result_count,
                        compared_result_count,
                        window_size,
                        exact_best_approx_rank,
                        exact_best_windowed_rank,
                        exact_top4_max_approx_rank,
                        exact_top4_max_windowed_rank,
                        mean_abs_rank_shift_before,
                        mean_abs_rank_shift_after,
                        max_abs_rank_shift_before,
                        max_abs_rank_shift_after,
                        spearman_rank_correlation_before,
                        spearman_rank_correlation_after
                     FROM tests.tqhnsw_debug_grouped_scan_windowed_summary(
                        'tqhnsw_scalar_runtime_windowed_summary_idx'::regclass::oid,
                        ARRAY[1.0, 0.0, 0.5, -1.0]::real[],
                        4
                     )",
                    None,
                    &[],
                )
                .expect("scalar windowed summary query should succeed")
                .next()
                .expect("scalar windowed summary should return one row");
            (
                row["emitted_result_count"]
                    .value::<i32>()
                    .expect("emitted result count should decode")
                    .expect("emitted result count should be non-null"),
                row["grouped_result_count"]
                    .value::<i32>()
                    .expect("grouped result count should decode")
                    .expect("grouped result count should be non-null"),
                row["compared_result_count"]
                    .value::<i32>()
                    .expect("compared result count should decode")
                    .expect("compared result count should be non-null"),
                row["window_size"]
                    .value::<i32>()
                    .expect("window size should decode")
                    .expect("window size should be non-null"),
                row["exact_best_approx_rank"]
                    .value::<i32>()
                    .expect("exact best approx rank should decode"),
                row["exact_best_windowed_rank"]
                    .value::<i32>()
                    .expect("exact best windowed rank should decode"),
                row["exact_top4_max_approx_rank"]
                    .value::<i32>()
                    .expect("exact top4 max approx rank should decode"),
                row["exact_top4_max_windowed_rank"]
                    .value::<i32>()
                    .expect("exact top4 max windowed rank should decode"),
                row["mean_abs_rank_shift_before"]
                    .value::<f64>()
                    .expect("mean abs rank shift before should decode")
                    .expect("mean abs rank shift before should be non-null"),
                row["mean_abs_rank_shift_after"]
                    .value::<f64>()
                    .expect("mean abs rank shift after should decode")
                    .expect("mean abs rank shift after should be non-null"),
                row["max_abs_rank_shift_before"]
                    .value::<i32>()
                    .expect("max abs rank shift before should decode")
                    .expect("max abs rank shift before should be non-null"),
                row["max_abs_rank_shift_after"]
                    .value::<i32>()
                    .expect("max abs rank shift after should decode")
                    .expect("max abs rank shift after should be non-null"),
                row["spearman_rank_correlation_before"]
                    .value::<f64>()
                    .expect("spearman rank correlation before should decode")
                    .expect("spearman rank correlation before should be non-null"),
                row["spearman_rank_correlation_after"]
                    .value::<f64>()
                    .expect("spearman rank correlation after should decode")
                    .expect("spearman rank correlation after should be non-null"),
            )
        });

        assert!(emitted_result_count > 0);
        assert_eq!(grouped_result_count, 0);
        assert_eq!(compared_result_count, 0);
        assert_eq!(window_size, 4);
        assert_eq!(exact_best_approx_rank, None);
        assert_eq!(exact_best_windowed_rank, None);
        assert_eq!(exact_top4_max_approx_rank, None);
        assert_eq!(exact_top4_max_windowed_rank, None);
        assert_eq!(mean_abs_rank_shift_before, 0.0);
        assert_eq!(mean_abs_rank_shift_after, 0.0);
        assert_eq!(max_abs_rank_shift_before, 0);
        assert_eq!(max_abs_rank_shift_after, 0);
        assert_eq!(spearman_before, 0.0);
        assert_eq!(spearman_after, 0.0);
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
    fn test_build_keeps_element_neighbor_local() {
        Spi::run("CREATE TABLE tqhnsw_build_locality (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");

        let payload_len = code_len(4, 8);
        for id in 1..=128 {
            let code = (0..payload_len)
                .map(|offset| ((id * 29 + offset as i32) & 0xff) as u8)
                .collect::<Vec<_>>();
            Spi::run(&format!(
                "INSERT INTO tqhnsw_build_locality VALUES \
                 ({id}, '[dim=4,bits=8,seed=42,gamma=0.5]:{payload}'::tqvector)",
                payload = hex::encode(code),
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX tqhnsw_build_locality_idx ON tqhnsw_build_locality USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 4, ef_construction = 64)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_build_locality_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        let (_block_count, _metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
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
                                    offset_number: u16::try_from(idx + 1)
                                        .expect("page tuple offset should fit in u16"),
                                },
                                am::page::TqElementTuple::decode(tuple, code_len(4, 8))
                                    .expect("element tuple should decode"),
                            ))
                        } else {
                            None
                        }
                    })
            })
            .collect::<Vec<_>>();

        assert!(
            !elements.is_empty(),
            "build should persist at least one element tuple"
        );
        assert!(elements
            .iter()
            .any(|(tid, element)| { element.neighbortid.block_number == tid.block_number }));
        assert!(elements.iter().all(|(tid, element)| {
            element.neighbortid.block_number <= tid.block_number
                && tid.block_number - element.neighbortid.block_number <= 1
        }),
        "build should keep each element tuple on the same page as its neighbor tuple or on the immediately following page");
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
                    binary_words: Vec::new(),
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

        let m = 2_u16;
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
                let insert_pair_fits = |page: &mut am::page::DataPage, offset_number: u16| {
                    let heap_tid = am::page::ItemPointer {
                        block_number: 0,
                        offset_number,
                    };
                    let level = am::debug_insert_level_for_heap_tid(m, 42, heap_tid, code_len);
                    let neighbor_slots = am::page::neighbor_slots(level, m);
                    let neighbor = am::page::TqNeighborTuple {
                        count: u16::try_from(neighbor_slots)
                            .expect("neighbor slot count should fit in u16"),
                        tids: vec![am::page::ItemPointer::INVALID; neighbor_slots],
                    };
                    let element = am::page::TqElementTuple {
                        level,
                        deleted: false,
                        heaptids: vec![heap_tid],
                        gamma: 0.5,
                        neighbortid: am::page::ItemPointer::INVALID,
                        code: vec![0x11_u8; code_len],
                        binary_words: Vec::new(),
                    };
                    page.insert_neighbor(&neighbor).is_ok() && page.insert_element(&element).is_ok()
                };
                let mut pairs = 0_usize;
                while insert_pair_fits(
                    &mut staged_page,
                    u16::try_from(pairs + 1).expect("offset should fit in u16"),
                ) {
                    pairs += 1;
                }

                let mut next_page = am::page::DataPage::new(
                    am::page::FIRST_DATA_BLOCK_NUMBER + 1,
                    pg_sys::BLCKSZ as usize,
                );
                let next_offset = u16::try_from(pairs + 1).expect("offset should fit in u16");
                let reuse_offset = u16::try_from(pairs + 2).expect("offset should fit in u16");
                if pairs >= 2
                    && insert_pair_fits(&mut next_page, next_offset)
                    && insert_pair_fits(&mut next_page, reuse_offset)
                {
                    Some((dim, pairs))
                } else {
                    None
                }
            })
            .expect("should find a dimension where one page fits multiple pairs");
        let code_len = code_len(dim as usize, 4);

        Spi::run(&format!(
            "CREATE INDEX tqhnsw_insert_rollover_reuse_idx ON tqhnsw_insert_rollover_reuse USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = {m})"
        ))
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
    #[should_panic(expected = "tqhnsw does not support non-finite gamma values")]
    fn test_non_empty_index_build_rejects_non_finite_gamma() {
        Spi::run("CREATE TABLE tqhnsw_build_nan_gamma (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_build_nan_gamma VALUES
             (1, '[dim=4,bits=4,seed=42,gamma=NaN]:112233'::tqvector)",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_build_nan_gamma_idx ON tqhnsw_build_nan_gamma USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw does not support non-finite gamma values")]
    fn test_tqhnsw_insert_rejects_non_finite_gamma() {
        Spi::run(
            "CREATE TABLE tqhnsw_insert_nan_gamma (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_nan_gamma_idx ON tqhnsw_insert_nan_gamma USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_nan_gamma VALUES
             (1, '[dim=4,bits=4,seed=42,gamma=NaN]:112233'::tqvector)",
        )
        .expect("insert should fail");
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
    #[should_panic(expected = "tqhnsw aminsert does not support ADR-030 grouped-v2 indexes yet")]
    fn test_tqhnsw_insert_rejects_grouped_v2_index() {
        let _lock = env_var_test_lock();
        let _guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");

        Spi::run(
            "CREATE TABLE tqhnsw_insert_grouped_v2_reject (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");
        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 29 + dim) as f32) * 0.05).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 17 + dim) as f32) * 0.04).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_insert_grouped_v2_reject VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("seed insert should succeed");
        }
        Spi::run(
            "CREATE INDEX tqhnsw_insert_grouped_v2_reject_idx ON tqhnsw_insert_grouped_v2_reject USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (build_source_column = 'source')",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_grouped_v2_reject VALUES
             (17,
              ARRAY[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8,
                    0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6]::real[],
              encode_to_tqvector(
                  ARRAY[0.2, 0.1, 0.0, -0.1, -0.2, -0.3, -0.4, -0.5,
                        0.5, 0.4, 0.3, 0.2, 0.1, 0.0, -0.1, -0.2]::real[],
                  4,
                  42
              ))",
        )
        .expect("insert should fail");
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw vacuum does not support ADR-030 grouped-v2 indexes yet")]
    fn test_tqhnsw_vacuum_rejects_grouped_v2_index() {
        let _lock = env_var_test_lock();
        let _guard = ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD", "1");

        Spi::run(
            "CREATE TABLE tqhnsw_vacuum_grouped_v2_reject (
                id bigint primary key,
                source real[],
                embedding tqvector
            )",
        )
        .expect("table creation should succeed");
        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 31 + dim) as f32) * 0.05).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 37 + dim) as f32) * 0.04).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO tqhnsw_vacuum_grouped_v2_reject VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_tqvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("seed insert should succeed");
        }
        Spi::run(
            "CREATE INDEX tqhnsw_vacuum_grouped_v2_reject_idx ON tqhnsw_vacuum_grouped_v2_reject USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_vacuum_grouped_v2_reject_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let _ = unsafe { am::debug_vacuum_stats(index_oid) };
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
        let (metadata, elements, neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);
        assert_eq!(elements.len(), 1);

        let (entry_tid, entry_element) = elements
            .iter()
            .find(|(tid, _)| *tid == metadata.entry_point)
            .expect("entry point should identify the inserted element");
        assert_eq!(metadata.max_level, entry_element.level);
        assert_eq!(entry_element.heaptids.len(), 1);

        let neighbor = neighbors
            .get(&entry_element.neighbortid)
            .expect("entry element neighbor tuple should exist");
        assert_eq!(neighbor.count as usize, neighbor.tids.len());
        assert_eq!(
            neighbor.tids.len(),
            am::page::neighbor_slots(entry_element.level, metadata.m)
        );

        let tuple_count = elements.len() + neighbors.len();
        assert_eq!(
            tuple_count, 2,
            "aminsert should append one neighbor and one element tuple"
        );
        assert_eq!(metadata.entry_point, *entry_tid);
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
        let (metadata, elements, _neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);
        let entry_element = elements
            .iter()
            .find(|(tid, _)| *tid == metadata.entry_point)
            .expect("entry point should identify a live element tuple");
        assert_eq!(entry_element.1.level, metadata.max_level);

        let element_count = elements.len();
        assert_eq!(
            element_count, 2,
            "second insert into an initially empty index should validate against persisted shape metadata"
        );
    }

    #[pg_test]
    fn test_tqhnsw_insert_repairs_invalid_entry_point_after_shape_init() {
        Spi::run(
            "CREATE TABLE tqhnsw_insert_entry_point_repair (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_entry_point_repair_idx ON tqhnsw_insert_entry_point_repair USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_entry_point_repair VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_insert_entry_point_repair_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (_block_count, _m, _ef_construction, mut metadata) =
            unsafe { am::debug_index_metadata(index_oid) };
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);

        metadata.entry_point = am::page::ItemPointer::INVALID;
        unsafe {
            am::debug_update_index_metadata(index_oid, metadata);
        }

        let (_block_count, _m, _ef_construction, metadata) =
            unsafe { am::debug_index_metadata(index_oid) };
        assert_eq!(metadata.entry_point, am::page::ItemPointer::INVALID);

        Spi::run(
            "INSERT INTO tqhnsw_insert_entry_point_repair VALUES
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42))",
        )
        .expect("repairing insert should succeed");

        let (metadata, elements, _neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);
        assert_eq!(elements.len(), 2);
        let (entry_tid, entry_element) = elements
            .iter()
            .find(|(tid, _)| *tid == metadata.entry_point)
            .expect("repairing insert should repoint metadata at a live element tuple");
        assert_eq!(entry_element.level, metadata.max_level);
        assert_eq!(metadata.entry_point, *entry_tid);
    }

    #[pg_test]
    fn test_tqhnsw_insert_neighbor_tuple_sizing_matches_levels() {
        Spi::run(
            "CREATE TABLE tqhnsw_insert_level_shape (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_level_shape_idx ON tqhnsw_insert_level_shape USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_insert_level_shape_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        let mut inserted_rows = 0_i64;
        let mut found_upper_level = false;
        while inserted_rows < 128 && !found_upper_level {
            inserted_rows += 1;
            Spi::run(&format!(
                "INSERT INTO tqhnsw_insert_level_shape VALUES (
                    {id},
                    encode_to_tqvector(ARRAY[
                        {id}.0,
                        {two}.0,
                        {three}.0,
                        {four}.0
                    ], 4, 42)
                )",
                id = inserted_rows,
                two = inserted_rows * 2,
                three = inserted_rows * 3,
                four = inserted_rows * 4,
            ))
            .expect("insert should succeed");

            let heap_tid = heap_tid_for_row("tqhnsw_insert_level_shape", inserted_rows);
            let expected_level =
                am::debug_insert_level_for_heap_tid(2, 42, heap_tid, code_len(4, 4));
            found_upper_level |= expected_level > 0;
        }
        assert!(
            found_upper_level,
            "deterministic insert level assignment should produce an upper-layer node within 128 inserts"
        );

        let (metadata, elements, neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        assert_eq!(elements.len(), inserted_rows as usize);
        assert!(
            elements.iter().any(|(_, element)| element.level > 0),
            "test fixture should contain at least one inserted upper-layer node"
        );

        for (_, element) in &elements {
            let neighbor = neighbors
                .get(&element.neighbortid)
                .expect("neighbor tuple should exist for each inserted element");
            assert_eq!(neighbor.count as usize, neighbor.tids.len());
            assert_eq!(
                neighbor.tids.len(),
                am::page::neighbor_slots(element.level, metadata.m),
                "neighbor tuple sizing should match the inserted element level",
            );
        }
    }

    #[pg_test]
    fn test_tqhnsw_insert_promotes_entry_point_on_level_up() {
        Spi::run(
            "CREATE TABLE tqhnsw_insert_level_promotion (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_level_promotion_idx ON tqhnsw_insert_level_promotion USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");

        Spi::run(
            "INSERT INTO tqhnsw_insert_level_promotion VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42))",
        )
        .expect("seed insert should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_insert_level_promotion_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let mut previous_metadata = unsafe { am::debug_index_metadata(index_oid) }.3;
        for id in 2_i64..=128_i64 {
            Spi::run(&format!(
                "INSERT INTO tqhnsw_insert_level_promotion VALUES (
                    {id},
                    encode_to_tqvector(ARRAY[
                        {id}.0,
                        {two}.0,
                        {three}.0,
                        {four}.0
                    ], 4, 42)
                )",
                id = id,
                two = id * 2,
                three = id * 3,
                four = id * 4,
            ))
            .expect("insert should succeed");

            let heap_tid = heap_tid_for_row("tqhnsw_insert_level_promotion", id);
            let expected_level =
                am::debug_insert_level_for_heap_tid(2, 42, heap_tid, code_len(4, 4));
            let (metadata, elements, _neighbors) =
                decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
            if expected_level > previous_metadata.max_level {
                let (promoted_tid, promoted_element) = elements
                    .iter()
                    .find(|(_, element)| element.heaptids.contains(&heap_tid))
                    .expect("promoted element should be discoverable by heap tid");
                assert_eq!(promoted_element.level, expected_level);
                assert_eq!(metadata.max_level, expected_level);
                assert_eq!(metadata.entry_point, *promoted_tid);
                return;
            }
            previous_metadata = metadata;
        }

        panic!("expected a higher-level insert to promote metadata within 128 inserts");
    }

    #[pg_test]
    fn test_tqhnsw_insert_populates_forward_links_from_live_entry_seed() {
        Spi::run(
            "CREATE TABLE tqhnsw_insert_live_forward_links (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_live_forward_links_idx ON tqhnsw_insert_live_forward_links USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_live_forward_links VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_insert_live_forward_links_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let row1_heap_tid = heap_tid_for_row("tqhnsw_insert_live_forward_links", 1);
        let (_metadata, elements, _neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (row1_element_tid, _) = find_element_for_heap_tid(&elements, row1_heap_tid);

        Spi::run(
            "INSERT INTO tqhnsw_insert_live_forward_links VALUES
             (2, encode_to_tqvector(ARRAY[0.9, 0.1, 0.25, -0.9], 4, 42))",
        )
        .expect("second insert should succeed");

        let row2_heap_tid = heap_tid_for_row("tqhnsw_insert_live_forward_links", 2);
        let (metadata, elements, neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (row1_element_tid_after, row1_element) =
            find_element_for_heap_tid(&elements, row1_heap_tid);
        let (row2_element_tid, row2_element) = find_element_for_heap_tid(&elements, row2_heap_tid);
        let row1_neighbors = neighbors
            .get(&row1_element.neighbortid)
            .expect("seed element neighbor tuple should exist");
        let row2_neighbors = neighbors
            .get(&row2_element.neighbortid)
            .expect("second insert neighbor tuple should exist");
        let row1_layer0 = layer_neighbor_slice(&row1_neighbors.tids, usize::from(metadata.m), 0);
        let row2_layer0 = layer_neighbor_slice(&row2_neighbors.tids, usize::from(metadata.m), 0);
        let populated_layer0_slots = row2_layer0
            .iter()
            .take(usize::from(metadata.m))
            .copied()
            .filter(|tid| *tid != am::page::ItemPointer::INVALID)
            .collect::<Vec<_>>();

        assert_eq!(
            populated_layer0_slots,
            vec![row1_element_tid],
            "the second live insert should seed its forward links from the existing entry element",
        );
        assert_eq!(
            row1_element_tid_after, row1_element_tid,
            "the seed element tid should remain stable after the live insert",
        );
        assert!(
            row1_layer0.contains(&row2_element_tid),
            "the seeded element should receive a layer-0 backlink to the newly inserted node",
        );
        assert!(
            row2_layer0
                .iter()
                .skip(usize::from(metadata.m))
                .all(|tid| *tid == am::page::ItemPointer::INVALID),
            "the upper-layer checkpoint still leaves the second half of the layer-0 forward window invalid on the new node",
        );
    }

    #[pg_test]
    fn test_tqhnsw_insert_populates_forward_links_against_built_graph() {
        Spi::run(
            "CREATE TABLE tqhnsw_insert_built_forward_links (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_built_forward_links VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.0, 0.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.0, 0.0, 1.0, 0.0], 4, 42)),
             (4, encode_to_tqvector(ARRAY[0.0, 0.0, 0.0, 1.0], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_built_forward_links_idx ON tqhnsw_insert_built_forward_links USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_insert_built_forward_links_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (_metadata, before_elements, _before_neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let existing_element_tids = before_elements
            .iter()
            .map(|(tid, _)| *tid)
            .collect::<HashSet<_>>();

        Spi::run(
            "INSERT INTO tqhnsw_insert_built_forward_links VALUES
             (5, encode_to_tqvector(ARRAY[1.0, 0.2, 0.1, 0.0], 4, 42))",
        )
        .expect("live insert should succeed");

        let row5_heap_tid = heap_tid_for_row("tqhnsw_insert_built_forward_links", 5);
        let (metadata, elements, neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (row5_element_tid, row5_element) = find_element_for_heap_tid(&elements, row5_heap_tid);
        let row5_neighbors = neighbors
            .get(&row5_element.neighbortid)
            .expect("inserted element neighbor tuple should exist");
        let row5_layer0 = layer_neighbor_slice(&row5_neighbors.tids, usize::from(metadata.m), 0);
        let populated_layer0_slots = row5_layer0
            .iter()
            .take(usize::from(metadata.m))
            .copied()
            .filter(|tid| *tid != am::page::ItemPointer::INVALID)
            .collect::<Vec<_>>();

        assert!(
            !populated_layer0_slots.is_empty(),
            "live insert into a built graph should materialize at least one forward link",
        );
        assert!(
            populated_layer0_slots
                .iter()
                .all(|tid| existing_element_tids.contains(tid)),
            "forward links should target pre-existing graph elements in this one-way slice",
        );
        assert!(
            populated_layer0_slots
                .iter()
                .all(|tid| *tid != row5_element_tid),
            "forward links must not self-reference the newly inserted element",
        );
        assert!(
            row5_layer0
                .iter()
                .skip(usize::from(metadata.m))
                .all(|tid| *tid == am::page::ItemPointer::INVALID),
            "the second half of the layer-0 forward window stays invalid even after upper-layer links land",
        );
    }

    #[pg_test]
    fn test_tqhnsw_insert_populates_upper_layer_links_when_available() {
        Spi::run(
            "CREATE TABLE tqhnsw_insert_upper_layer_links (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_upper_layer_links_idx ON tqhnsw_insert_upper_layer_links USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_insert_upper_layer_links_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let mut chosen_insert = None;
        for id in 1_i64..=192_i64 {
            let previous_metadata = unsafe { am::debug_index_metadata(index_oid) }.3;
            Spi::run(&format!(
                "INSERT INTO tqhnsw_insert_upper_layer_links VALUES (
                    {id},
                    encode_to_tqvector(ARRAY[
                        {id}.0,
                        {two}.0,
                        {three}.0,
                        {four}.0
                    ], 4, 42)
                )",
                id = id,
                two = id * 2,
                three = id * 3,
                four = id * 4,
            ))
            .expect("live insert should succeed");

            let heap_tid = heap_tid_for_row("tqhnsw_insert_upper_layer_links", id);
            let level = am::debug_insert_level_for_heap_tid(2, 42, heap_tid, code_len(4, 4));
            if previous_metadata.max_level > 0 && level > 0 {
                chosen_insert = Some((id, level));
                break;
            }
        }

        let (inserted_id, expected_level) = chosen_insert
            .expect("deterministic insert levels should produce an upper-layer live insert");
        let inserted_heap_tid = heap_tid_for_row("tqhnsw_insert_upper_layer_links", inserted_id);
        let (metadata, elements, neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (inserted_element_tid, inserted_element) =
            find_element_for_heap_tid(&elements, inserted_heap_tid);
        let inserted_neighbors = neighbors
            .get(&inserted_element.neighbortid)
            .expect("inserted upper-layer neighbor tuple should exist");
        assert_eq!(inserted_element.level, expected_level);
        assert!(
            inserted_element.level > 0,
            "the chosen live insert must participate in at least one upper layer",
        );

        let layer1_forward_tids =
            layer_neighbor_slice(&inserted_neighbors.tids, usize::from(metadata.m), 1)
                .iter()
                .copied()
                .filter(|tid| *tid != am::page::ItemPointer::INVALID)
                .collect::<Vec<_>>();
        assert!(
            !layer1_forward_tids.is_empty(),
            "an upper-layer live insert should populate at least one layer-1 forward link",
        );

        let mut layer1_backlink_targets = 0_usize;
        for forward_tid in layer1_forward_tids {
            let (_, forward_element) = elements
                .iter()
                .find(|(tid, _)| *tid == forward_tid)
                .expect("upper-layer forward link should target an existing element");
            assert!(
                forward_element.level >= 1,
                "upper-layer forward links must target elements that participate in layer 1",
            );

            let forward_neighbors = neighbors
                .get(&forward_element.neighbortid)
                .expect("upper-layer forward target neighbor tuple should exist");
            if layer_neighbor_slice(&forward_neighbors.tids, usize::from(metadata.m), 1)
                .contains(&inserted_element_tid)
            {
                layer1_backlink_targets += 1;
            }
        }
        assert!(
            layer1_backlink_targets > 0,
            "at least one sparse layer-1 forward target should receive a matching layer-1 backlink to the new element",
        );
        assert!(
            layer_neighbor_slice(&inserted_neighbors.tids, usize::from(metadata.m), 0)
                .iter()
                .skip(usize::from(metadata.m))
                .all(|tid| *tid == am::page::ItemPointer::INVALID),
            "upper-layer link coverage should not change the second half of the layer-0 forward window",
        );
    }

    #[pg_test]
    fn test_tqhnsw_insert_rewrites_full_layer0_backlink_slice() {
        Spi::run(
            "CREATE TABLE tqhnsw_insert_full_layer0_backlink (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        for id in 1_i64..=12_i64 {
            let delta = id as f32 * 0.08;
            let z = if id % 2 == 0 { 0.25 } else { -0.25 };
            Spi::run(&format!(
                "INSERT INTO tqhnsw_insert_full_layer0_backlink VALUES
                 ({id}, encode_to_tqvector(ARRAY[1.0, {delta}, {z}, 0.0], 4, 42))",
            ))
            .expect("seed insert should succeed");
        }
        Spi::run(
            "CREATE INDEX tqhnsw_insert_full_layer0_backlink_idx ON tqhnsw_insert_full_layer0_backlink USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_insert_full_layer0_backlink_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        for id in 13_i64..=40_i64 {
            let (_metadata_before, elements_before, neighbors_before) =
                decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
            let delta = (id - 12) as f32 * 0.04;
            let w = (id - 12) as f32 * 0.02;
            let z = if id % 2 == 0 { 0.25 } else { -0.25 };
            Spi::run(&format!(
                "INSERT INTO tqhnsw_insert_full_layer0_backlink VALUES
                 ({id}, encode_to_tqvector(ARRAY[1.0, {delta}, {z}, {w}], 4, 42))",
            ))
            .expect("live insert should succeed");

            let inserted_heap_tid = heap_tid_for_row("tqhnsw_insert_full_layer0_backlink", id);
            let (metadata, elements_after, neighbors_after) =
                decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
            let (inserted_element_tid, inserted_element) =
                find_element_for_heap_tid(&elements_after, inserted_heap_tid);
            let inserted_neighbors = neighbors_after
                .get(&inserted_element.neighbortid)
                .expect("inserted element neighbor tuple should exist");
            let inserted_layer0_forward_tids =
                layer_neighbor_slice(&inserted_neighbors.tids, usize::from(metadata.m), 0)
                    .iter()
                    .take(usize::from(metadata.m))
                    .copied()
                    .filter(|tid| *tid != am::page::ItemPointer::INVALID)
                    .collect::<Vec<_>>();

            for forward_tid in inserted_layer0_forward_tids {
                let (_, before_element) = elements_before
                    .iter()
                    .find(|(tid, _)| *tid == forward_tid)
                    .expect("forward target should exist before the live insert");
                let before_neighbors = neighbors_before
                    .get(&before_element.neighbortid)
                    .expect("forward target neighbor tuple should exist before the live insert");
                let before_layer0 =
                    layer_neighbor_slice(&before_neighbors.tids, usize::from(metadata.m), 0);
                if before_layer0.contains(&am::page::ItemPointer::INVALID) {
                    continue;
                }

                let (_, after_element) = elements_after
                    .iter()
                    .find(|(tid, _)| *tid == forward_tid)
                    .expect("forward target should still exist after the live insert");
                let after_neighbors = neighbors_after
                    .get(&after_element.neighbortid)
                    .expect("forward target neighbor tuple should exist after the live insert");
                let after_layer0 =
                    layer_neighbor_slice(&after_neighbors.tids, usize::from(metadata.m), 0);
                if !after_layer0.contains(&inserted_element_tid) {
                    continue;
                }

                assert!(
                    after_layer0
                        .iter()
                        .all(|tid| *tid != am::page::ItemPointer::INVALID),
                    "overflow rewrite should preserve the full 2M layer-0 capacity on selected targets",
                );
                assert_ne!(
                    after_layer0, before_layer0,
                    "admitting the new element into a full layer-0 target should evict at least one prior neighbor",
                );
                return;
            }
        }

        panic!("expected a bounded live-insert search to rewrite at least one full layer-0 target");
    }

    #[pg_test]
    fn test_tqhnsw_live_insert_is_graph_reachable_via_backlinks() {
        Spi::run(
            "CREATE TABLE tqhnsw_insert_graph_reachable (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_insert_graph_reachable VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.0, 0.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.0, 0.0, 1.0, 0.0], 4, 42)),
             (4, encode_to_tqvector(ARRAY[0.0, 0.0, 0.0, 1.0], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_insert_graph_reachable_idx ON tqhnsw_insert_graph_reachable USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 8, ef_search = 8)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_insert_graph_reachable_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let mut chosen_insert = None;
        for id in 5_i64..=32_i64 {
            let delta = (id - 4) as f32 * 0.02;
            Spi::run(&format!(
                "INSERT INTO tqhnsw_insert_graph_reachable VALUES
                 ({id}, encode_to_tqvector(ARRAY[1.0, {delta}, 0.1, 0.0], 4, 42))",
            ))
            .expect("live insert should succeed");

            let heap_tid = heap_tid_for_row("tqhnsw_insert_graph_reachable", id);
            let level = am::debug_insert_level_for_heap_tid(8, 42, heap_tid, code_len(4, 4));
            if level == 0 {
                chosen_insert = Some((id, vec![1.0, delta, 0.1, 0.0]));
                break;
            }
        }

        let (inserted_id, query) = chosen_insert
            .expect("deterministic insert levels should produce a level-0 live insert");
        let inserted_heap_tid = heap_tid_for_row("tqhnsw_insert_graph_reachable", inserted_id);
        let (metadata, elements, _neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (inserted_element_tid, _inserted_element) =
            find_element_for_heap_tid(&elements, inserted_heap_tid);
        assert_ne!(
            metadata.entry_point, inserted_element_tid,
            "the reachability check should exercise backlinks rather than an entry-point promotion",
        );

        let (_head, frontier, frontier_slots, _frontier_provenance, _expanded_sources) =
            unsafe { am::debug_rescan_candidate_frontier(index_oid, query) };
        let frontier_tids = frontier_slots
            .iter()
            .filter_map(|(valid, tid, _)| valid.then_some(*tid))
            .collect::<Vec<_>>();

        assert!(
            !frontier.is_empty(),
            "reachable live inserts should contribute to a non-empty graph frontier",
        );
        assert!(
            frontier_tids.contains(&(inserted_element_tid.block_number, inserted_element_tid.offset_number)),
            "the graph-seeded runtime frontier should reach the live-inserted element before any linear fallback",
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
    fn test_tqhnsw_vacuum_pass1_compacts_duplicate_heaptids() {
        Spi::run(
            "CREATE TABLE tqhnsw_vacuum_pass1_duplicates (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_vacuum_pass1_duplicates VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.25, 0.75], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_vacuum_pass1_duplicates_idx ON tqhnsw_vacuum_pass1_duplicates USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let survivor_heap_tid = heap_tid_for_row("tqhnsw_vacuum_pass1_duplicates", 1);
        let deleted_heap_tid = heap_tid_for_row("tqhnsw_vacuum_pass1_duplicates", 2);
        Spi::run("DELETE FROM tqhnsw_vacuum_pass1_duplicates WHERE id = 2")
            .expect("delete should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_vacuum_pass1_duplicates_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let stats = unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };
        let (_metadata, elements, _neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (_element_tid, duplicate_element) =
            find_element_for_heap_tid(&elements, survivor_heap_tid);

        assert_eq!(
            duplicate_element.heaptids,
            vec![survivor_heap_tid],
            "pass 1 should compact the duplicate element to the surviving heap tid",
        );
        assert!(
            elements
                .iter()
                .all(|(_, element)| !element.heaptids.contains(&deleted_heap_tid)),
            "pass 1 should remove the deleted heap tid from every element payload",
        );
        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(
            stats.num_index_tuples, 2.0,
            "amvacuumcleanup should report the remaining live element count",
        );
    }

    #[pg_test]
    fn test_tqhnsw_vacuum_pass1_makes_deleted_row_unreachable() {
        Spi::run(
            "CREATE TABLE tqhnsw_vacuum_pass1_scan (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_vacuum_pass1_scan VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.25, 0.75], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_vacuum_pass1_scan_idx ON tqhnsw_vacuum_pass1_scan USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let deleted_heap_tid = heap_tid_for_row("tqhnsw_vacuum_pass1_scan", 2);
        Spi::run("DELETE FROM tqhnsw_vacuum_pass1_scan WHERE id = 2")
            .expect("delete should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_vacuum_pass1_scan_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let stats = unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };
        let (_metadata, elements, _neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let deleted_element = elements
            .iter()
            .find(|(_, element)| {
                element.code
                    == encoded_code_bytes(
                        ProdQuantizer::new(4, 4, 42).encode(&[0.5, 1.0, -0.5, 0.25]),
                    )
            })
            .expect("deleted element should still be present after vacuum finalization");

        assert!(
            deleted_element.1.heaptids.is_empty(),
            "pass 1 should clear the last heap tid from a fully dead element",
        );
        assert!(
            deleted_element.1.deleted,
            "vacuum should finalize a fully dead element once pass 1 strips its last heap tid",
        );

        let returned =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, vec![0.5, 1.0, -0.5, 0.25]) };
        assert!(
            !returned.contains(&(
                deleted_heap_tid.block_number,
                deleted_heap_tid.offset_number
            )),
            "graph/runtime scans should skip elements whose heap tid array is empty after pass 1",
        );
        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(stats.num_index_tuples, 2.0);
    }

    #[pg_test]
    fn test_tqhnsw_vacuum_pass2_unlinks_deleted_neighbor_refs() {
        Spi::run(
            "CREATE TABLE tqhnsw_vacuum_pass2_unlink (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_vacuum_pass2_unlink VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.25, 0.75], 4, 42)),
             (4, encode_to_tqvector(ARRAY[0.25, -0.75, 1.0, 0.5], 4, 42)),
             (5, encode_to_tqvector(ARRAY[-0.5, -1.0, 0.75, 0.25], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_vacuum_pass2_unlink_idx ON tqhnsw_vacuum_pass2_unlink USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let deleted_heap_tid = heap_tid_for_row("tqhnsw_vacuum_pass2_unlink", 2);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_vacuum_pass2_unlink_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (_metadata_before, elements_before, neighbors_before) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (deleted_element_tid, _deleted_element) =
            find_element_for_heap_tid(&elements_before, deleted_heap_tid);
        assert!(
            count_neighbor_refs(&neighbors_before, deleted_element_tid) > 0,
            "fixture should start with at least one persisted neighbor ref to the soon-to-be-deleted node",
        );

        Spi::run("DELETE FROM tqhnsw_vacuum_pass2_unlink WHERE id = 2")
            .expect("delete should succeed");

        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };

        let (_metadata_after, elements_after, neighbors_after) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (_, deleted_element_after) = elements_after
            .iter()
            .find(|(tid, _)| *tid == deleted_element_tid)
            .expect("deleted element tuple should remain on disk after vacuum");

        assert!(
            deleted_element_after.deleted,
            "vacuum should still finalize the fully dead element after pass-2 repair",
        );
        assert_eq!(
            count_neighbor_refs(&neighbors_after, deleted_element_tid),
            0,
            "pass 2 should remove every persisted neighbor ref to the deleted element tid",
        );
    }

    #[pg_test]
    fn test_tqhnsw_vacuum_pass2_layer0_replacement_fills_broken_edges() {
        Spi::run(
            "CREATE TABLE tqhnsw_vacuum_pass2_replace (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_vacuum_pass2_replace VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.9, 0.1, 0.45, -0.9], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.8, 0.2, 0.4, -0.8], 4, 42)),
             (4, encode_to_tqvector(ARRAY[0.7, 0.3, 0.35, -0.7], 4, 42)),
             (5, encode_to_tqvector(ARRAY[0.6, 0.4, 0.3, -0.6], 4, 42)),
             (6, encode_to_tqvector(ARRAY[0.5, 0.5, 0.25, -0.5], 4, 42)),
             (7, encode_to_tqvector(ARRAY[0.4, 0.6, 0.2, -0.4], 4, 42)),
             (8, encode_to_tqvector(ARRAY[0.3, 0.7, 0.15, -0.3], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_vacuum_pass2_replace_idx ON tqhnsw_vacuum_pass2_replace USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_vacuum_pass2_replace_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (metadata_before, elements_before, neighbors_before) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (deleted_row_id, deleted_heap_tid, deleted_element_tid, affected_before) =
            (1_i64..=8)
                .find_map(|id| {
                    let deleted_heap_tid = heap_tid_for_row("tqhnsw_vacuum_pass2_replace", id);
                    let (deleted_element_tid, _) =
                        find_element_for_heap_tid(&elements_before, deleted_heap_tid);
                    let affected_before = elements_before
                        .iter()
                        .filter_map(|(element_tid, element)| {
                            if *element_tid == deleted_element_tid
                                || element.deleted
                                || element.heaptids.is_empty()
                            {
                                return None;
                            }

                            let neighbor = neighbors_before
                                .get(&element.neighbortid)
                                .expect("live element should have a persisted neighbor tuple");
                            let layer0 = layer_neighbor_slice(
                                &neighbor.tids,
                                usize::from(metadata_before.m),
                                0,
                            );
                            layer0.contains(&deleted_element_tid).then(|| {
                                (
                                    *element_tid,
                                    layer0
                                        .iter()
                                        .copied()
                                        .filter(|tid| {
                                            *tid != am::page::ItemPointer::INVALID
                                                && *tid != deleted_element_tid
                                        })
                                        .collect::<Vec<_>>(),
                                )
                            })
                        })
                        .collect::<Vec<_>>();

                    (!affected_before.is_empty())
                        .then_some((id, deleted_heap_tid, deleted_element_tid, affected_before))
                })
                .expect(
                    "fixture should provide at least one deletable row with a live inbound layer-0 edge",
                );

        Spi::run(&format!(
            "DELETE FROM tqhnsw_vacuum_pass2_replace WHERE id = {deleted_row_id}"
        ))
        .expect("delete should succeed");
        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };

        let (metadata_after, elements_after, neighbors_after) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let mut replacement_filled = false;

        for (affected_tid, surviving_before) in affected_before {
            let (_, element_after) = elements_after
                .iter()
                .find(|(tid, _)| *tid == affected_tid)
                .expect("affected live element should remain on disk after vacuum");
            let neighbor_after = neighbors_after
                .get(&element_after.neighbortid)
                .expect("affected live element should keep a persisted neighbor tuple");
            let layer0_after =
                layer_neighbor_slice(&neighbor_after.tids, usize::from(metadata_after.m), 0);
            let surviving_after = layer0_after
                .iter()
                .copied()
                .filter(|tid| *tid != am::page::ItemPointer::INVALID)
                .collect::<Vec<_>>();

            if surviving_after
                .iter()
                .any(|tid| *tid != deleted_element_tid && !surviving_before.contains(tid))
            {
                replacement_filled = true;
                break;
            }
        }

        assert_eq!(
            count_neighbor_refs(&neighbors_after, deleted_element_tid),
            0,
            "vacuum replacement should still leave no persisted refs to the deleted element tid",
        );
        assert!(
            replacement_filled,
            "vacuum replacement search should fill at least one broken layer-0 edge with a new live candidate",
        );
    }

    #[pg_test]
    fn test_tqhnsw_vacuum_pass2_upper_replacement_fills_broken_edges() {
        Spi::run(
            "CREATE TABLE tqhnsw_vacuum_pass2_upper_replace (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        for id in 1_i64..=192_i64 {
            Spi::run(&format!(
                "INSERT INTO tqhnsw_vacuum_pass2_upper_replace VALUES (
                    {id},
                    encode_to_tqvector(ARRAY[
                        {id}.0,
                        {two}.0,
                        {three}.0,
                        {four}.0
                    ], 4, 42)
                )",
                id = id,
                two = id * 2,
                three = id * 3,
                four = id * 4,
            ))
            .expect("seed insert should succeed");
        }
        Spi::run(
            "CREATE INDEX tqhnsw_vacuum_pass2_upper_replace_idx ON tqhnsw_vacuum_pass2_upper_replace USING tqhnsw \
             (embedding tqvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_vacuum_pass2_upper_replace_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (metadata_before, elements_before, neighbors_before) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        assert!(
            metadata_before.max_level > 0,
            "fixture should build at least one upper layer before vacuum repair runs",
        );

        let (deleted_row_id, deleted_heap_tid, deleted_element_tid, affected_before) =
            (1_i64..=192_i64)
                .find_map(|id| {
                    let deleted_heap_tid =
                        heap_tid_for_row("tqhnsw_vacuum_pass2_upper_replace", id);
                    let (deleted_element_tid, _) =
                        find_element_for_heap_tid(&elements_before, deleted_heap_tid);
                    let affected_before = elements_before
                        .iter()
                        .filter_map(|(element_tid, element)| {
                            if *element_tid == deleted_element_tid
                                || element.deleted
                                || element.heaptids.is_empty()
                                || element.level < 1
                            {
                                return None;
                            }

                            let neighbor = neighbors_before
                                .get(&element.neighbortid)
                                .expect("live upper-layer element should have a persisted neighbor tuple");
                            let layer1 = layer_neighbor_slice(
                                &neighbor.tids,
                                usize::from(metadata_before.m),
                                1,
                            );
                            layer1.contains(&deleted_element_tid).then(|| {
                                (
                                    *element_tid,
                                    layer1
                                        .iter()
                                        .copied()
                                        .filter(|tid| {
                                            *tid != am::page::ItemPointer::INVALID
                                                && *tid != deleted_element_tid
                                        })
                                        .collect::<Vec<_>>(),
                                )
                            })
                        })
                        .collect::<Vec<_>>();

                    (!affected_before.is_empty())
                        .then_some((id, deleted_heap_tid, deleted_element_tid, affected_before))
                })
                .expect(
                    "fixture should provide at least one deletable row with a live inbound layer-1 edge",
                );

        Spi::run(&format!(
            "DELETE FROM tqhnsw_vacuum_pass2_upper_replace WHERE id = {deleted_row_id}"
        ))
        .expect("delete should succeed");
        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };

        let (metadata_after, elements_after, neighbors_after) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let mut replacement_filled = false;

        for (affected_tid, surviving_before) in affected_before {
            let (_, element_after) = elements_after
                .iter()
                .find(|(tid, _)| *tid == affected_tid)
                .expect("affected live upper-layer element should remain on disk after vacuum");
            let neighbor_after = neighbors_after
                .get(&element_after.neighbortid)
                .expect("affected live upper-layer element should keep a persisted neighbor tuple");
            let layer1_after =
                layer_neighbor_slice(&neighbor_after.tids, usize::from(metadata_after.m), 1);
            let surviving_after = layer1_after
                .iter()
                .copied()
                .filter(|tid| *tid != am::page::ItemPointer::INVALID)
                .collect::<Vec<_>>();

            if surviving_after
                .iter()
                .any(|tid| *tid != deleted_element_tid && !surviving_before.contains(tid))
            {
                replacement_filled = true;
                break;
            }
        }

        assert_eq!(
            count_neighbor_refs(&neighbors_after, deleted_element_tid),
            0,
            "upper-layer vacuum replacement should still leave no persisted refs to the deleted element tid",
        );
        assert!(
            replacement_filled,
            "vacuum replacement search should fill at least one broken upper-layer edge with a new live candidate",
        );
    }

    #[pg_test]
    fn test_tqhnsw_vacuum_pass1_is_stable_across_repeated_replays() {
        Spi::run(
            "CREATE TABLE tqhnsw_vacuum_pass1_repeat (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_vacuum_pass1_repeat VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.25, 0.75], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_vacuum_pass1_repeat_idx ON tqhnsw_vacuum_pass1_repeat USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let deleted_heap_tid = heap_tid_for_row("tqhnsw_vacuum_pass1_repeat", 2);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_vacuum_pass1_repeat_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (_metadata_before, elements_before, neighbors_before) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (deleted_element_tid, _deleted_element) =
            find_element_for_heap_tid(&elements_before, deleted_heap_tid);
        assert!(
            count_neighbor_refs(&neighbors_before, deleted_element_tid) > 0,
            "fixture should start with at least one persisted neighbor ref to the deleted element",
        );

        Spi::run("DELETE FROM tqhnsw_vacuum_pass1_repeat WHERE id = 2")
            .expect("delete should succeed");

        let first_stats =
            unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };
        let second_stats =
            unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };
        let (_metadata, elements, neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));

        assert_eq!(first_stats.tuples_removed, 1.0);
        assert_eq!(second_stats.tuples_removed, 0.0);
        assert_eq!(second_stats.num_index_tuples, first_stats.num_index_tuples);
        assert_eq!(
            elements
                .iter()
                .filter(|(_, element)| element.heaptids.is_empty() && element.deleted)
                .count(),
            1,
            "the second pass should observe the already-finalized fully dead element without rewriting it again",
        );
        assert_eq!(
            count_neighbor_refs(&neighbors, deleted_element_tid),
            0,
            "replaying the same vacuum delete-set should keep the deleted element tid fully unlinked from persisted neighbor tuples",
        );
    }

    #[pg_test]
    fn test_tqhnsw_vacuum_finalized_nodes_skip_duplicate_coalesce() {
        Spi::run("CREATE TABLE tqhnsw_vacuum_reinsert (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_vacuum_reinsert VALUES
             (1, encode_to_tqvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_vacuum_reinsert_idx ON tqhnsw_vacuum_reinsert USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let deleted_heap_tid = heap_tid_for_row("tqhnsw_vacuum_reinsert", 1);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'tqhnsw_vacuum_reinsert_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        Spi::run("DELETE FROM tqhnsw_vacuum_reinsert WHERE id = 1").expect("delete should succeed");
        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };

        Spi::run(
            "INSERT INTO tqhnsw_vacuum_reinsert VALUES
             (2, encode_to_tqvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42))",
        )
        .expect("replacement insert should succeed");

        let replacement_heap_tid = heap_tid_for_row("tqhnsw_vacuum_reinsert", 2);
        let (_metadata, elements, _neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (_replacement_tid, replacement_element) =
            find_element_for_heap_tid(&elements, replacement_heap_tid);

        assert!(
            !replacement_element.deleted,
            "duplicate insert should append or coalesce into a live element, not a finalized tombstone",
        );
        assert_eq!(replacement_element.heaptids, vec![replacement_heap_tid]);
        assert_eq!(
            elements
                .iter()
                .filter(|(_, element)| element.deleted)
                .count(),
            1,
            "the finalized vacuum tombstone should remain on disk until page compaction lands",
        );

        let returned =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, vec![0.5, 1.0, -0.5, 0.25]) };
        assert!(
            returned.contains(&(
                replacement_heap_tid.block_number,
                replacement_heap_tid.offset_number
            )),
            "the replacement row should stay reachable after reinserting the same encoded vector",
        );
    }

    #[pg_test]
    fn test_tqhnsw_debug_scan_result_count_matches_scan_helper() {
        Spi::run(
            "CREATE TABLE tqhnsw_debug_scan_result_count_fixture \
             (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_debug_scan_result_count_fixture VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.95, 0.05, 0.45, -0.95], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.0, -0.5, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_debug_scan_result_count_fixture_idx \
             ON tqhnsw_debug_scan_result_count_fixture USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_debug_scan_result_count_fixture_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![1.0, 0.0, 0.5, -1.0];
        let rust_count = i32::try_from(unsafe {
            am::debug_gettuple_scan_heap_tids(index_oid, query.clone()).len()
        })
        .expect("scan result count should fit in i32");
        let sql_count = Spi::get_one::<i32>(
            "SELECT tests.tqhnsw_debug_scan_result_count(
                 'tqhnsw_debug_scan_result_count_fixture_idx'::regclass::oid,
                 ARRAY[1.0, 0.0, 0.5, -1.0]::real[]
             )",
        )
        .expect("debug scan SQL wrapper should succeed")
        .expect("debug scan SQL wrapper should return a row");

        assert_eq!(
            sql_count, rust_count,
            "the SQL-visible debug scan wrapper should exercise the same live tqhnsw scan path",
        );
    }

    #[pg_test]
    fn test_tqhnsw_debug_scan_profile_reports_graph_first_counters() {
        Spi::run(
            "CREATE TABLE tqhnsw_debug_scan_profile_fixture \
             (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_debug_scan_profile_fixture VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.95, 0.05, 0.45, -0.95], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.0, -0.5, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_debug_scan_profile_fixture_idx \
             ON tqhnsw_debug_scan_profile_fixture USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_debug_scan_profile_fixture_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (
            _rescan_elapsed_us,
            _emit_elapsed_us,
            _total_elapsed_us,
            rescan_phase,
            rescan_current_result,
            _rescan_ordered_slots,
            _rescan_pending_heap_tids,
            _rescan_visited_elements,
            _rescan_expanded_sources,
            _rescan_emitted_elements,
            _rescan_bootstrap_expansions,
            rescan_bootstrap_pages_read,
            _rescan_quantizer_cache_hit,
            result_count,
            final_phase,
            final_ordered_slots,
            _total_bootstrap_expansions,
            _total_bootstrap_pages_read,
            total_linear_pages_read,
            total_elements_scored,
            _total_elements_skipped,
            total_heap_tids_returned,
            _total_quantizer_cache_hit,
            total_emitted_elements,
            rescan_amrescan_total_elapsed_us,
            rescan_query_decode_elapsed_us,
            rescan_scan_setup_elapsed_us,
            rescan_store_query_elapsed_us,
            rescan_prepare_query_elapsed_us,
            rescan_reset_state_elapsed_us,
            rescan_initialize_entry_elapsed_us,
            rescan_upper_layer_seed_elapsed_us,
            rescan_layer0_seed_elapsed_us,
            rescan_stage_ordered_results_elapsed_us,
            rescan_initial_prefetch_elapsed_us,
            rescan_frontier_consume_elapsed_us,
            rescan_graph_result_materialize_elapsed_us,
            graph_element_cache_hits,
            graph_element_cache_misses,
            graph_element_load_elapsed_us,
            graph_neighbor_cache_hits,
            graph_neighbor_cache_misses,
            graph_neighbor_load_elapsed_us,
            candidate_score_calls,
            candidate_score_elapsed_us,
            score_cache_hits,
            score_cache_misses,
            grouped_traversal_approx_score_calls,
            grouped_traversal_approx_score_elapsed_us,
            grouped_traversal_exact_score_calls,
            grouped_traversal_exact_score_elapsed_us,
            grouped_traversal_budgeted_expansions,
            grouped_traversal_budgeted_candidates,
            grouped_traversal_budgeted_exact_candidates,
        ) = unsafe { am::debug_profile_ordered_scan(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert_eq!(
            rescan_phase, "graph_traversal",
            "the profile helper should report that ordered scans start in the graph-traversal phase",
        );
        assert!(
            rescan_current_result,
            "amrescan should prefetch the first ordered result into current-result state on a non-empty index",
        );
        assert!(
            rescan_bootstrap_pages_read > 0,
            "prefetching the first ordered result should read at least one graph page",
        );
        assert!(
            rescan_amrescan_total_elapsed_us >= 0
                && rescan_query_decode_elapsed_us >= 0
                && rescan_scan_setup_elapsed_us >= 0
                && rescan_store_query_elapsed_us >= 0
                && rescan_prepare_query_elapsed_us >= 0
                && rescan_reset_state_elapsed_us >= 0
                && rescan_initialize_entry_elapsed_us >= 0
                && rescan_upper_layer_seed_elapsed_us >= 0
                && rescan_layer0_seed_elapsed_us >= 0
                && rescan_stage_ordered_results_elapsed_us >= 0
                && rescan_initial_prefetch_elapsed_us >= 0
                && rescan_frontier_consume_elapsed_us >= 0
                && rescan_graph_result_materialize_elapsed_us >= 0,
            "the profile helper should surface non-negative rescan timing buckets",
        );
        assert!(
            graph_element_cache_misses > 0 && graph_neighbor_cache_misses > 0,
            "profiling should record graph cache misses on a non-empty fixture",
        );
        assert!(
            graph_element_load_elapsed_us >= 0 && graph_neighbor_load_elapsed_us >= 0,
            "profiling should surface non-negative graph load timing buckets",
        );
        assert!(
            graph_element_cache_hits >= 0 && graph_neighbor_cache_hits >= 0,
            "profiling should surface graph cache hit counters even when the fixture is tiny",
        );
        assert!(
            candidate_score_calls > 0 && candidate_score_elapsed_us >= 0,
            "profiling should record candidate scoring work during scan seeding",
        );
        assert!(
            score_cache_hits >= 0 && score_cache_misses > 0,
            "profiling should surface score-cache counters and at least one first-score miss on a non-empty fixture",
        );
        assert_eq!(
            (
                grouped_traversal_approx_score_calls,
                grouped_traversal_approx_score_elapsed_us,
                grouped_traversal_exact_score_calls,
                grouped_traversal_exact_score_elapsed_us,
                grouped_traversal_budgeted_expansions,
                grouped_traversal_budgeted_candidates,
                grouped_traversal_budgeted_exact_candidates,
            ),
            (0, 0, 0, 0, 0, 0, 0),
            "scalar fixtures should leave grouped traversal counters inert",
        );
        assert!(
            result_count > 0,
            "the profiled scan should return at least one heap TID on a non-empty fixture",
        );
        assert_eq!(
            final_phase, "exhausted",
            "a full profiled scan should end in the exhausted phase",
        );
        assert_eq!(
            final_ordered_slots, 0,
            "full scan exhaustion should leave no current result or frontier slots staged",
        );
        assert_eq!(
            total_linear_pages_read, 0,
            "the graph-first ordered runtime should not fall back to linear scanning on this fixture",
        );
        assert!(
            total_elements_scored >= total_emitted_elements,
            "scored elements should cover every emitted ordered element",
        );
        assert_eq!(
            total_heap_tids_returned, result_count,
            "heap-TID return count should match the helper's emitted row count",
        );
    }

    #[pg_test]
    fn test_tqhnsw_debug_reachable_live_count_matches_admin_snapshot() {
        Spi::run(
            "CREATE TABLE tqhnsw_debug_reachable_live_fixture \
             (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_debug_reachable_live_fixture VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.95, 0.05, 0.45, -0.95], 4, 42)),
             (3, encode_to_tqvector(ARRAY[0.9, 0.1, 0.4, -0.9], 4, 42)),
             (4, encode_to_tqvector(ARRAY[-1.0, 0.0, -0.5, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_debug_reachable_live_fixture_idx \
             ON tqhnsw_debug_reachable_live_fixture USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_debug_reachable_live_fixture_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let rust_count =
            i32::try_from(unsafe { am::debug_layer0_reachable_live_element_tids(index_oid).len() })
                .expect("reachable live element count should fit in i32");
        let sql_count = Spi::get_one::<i32>(
            "SELECT tests.tqhnsw_debug_reachable_live_element_count(
                 'tqhnsw_debug_reachable_live_fixture_idx'::regclass::oid
             )",
        )
        .expect("debug reachability SQL wrapper should succeed")
        .expect("debug reachability SQL wrapper should return a row");
        let live_count = Spi::get_one::<i64>(
            "SELECT total_live_nodes
             FROM tqhnsw_index_admin_snapshot('tqhnsw_debug_reachable_live_fixture_idx'::regclass)",
        )
        .expect("admin snapshot query should succeed")
        .expect("admin snapshot should return a row");

        assert_eq!(
            sql_count, rust_count,
            "the SQL-visible reachability wrapper should match the Rust helper",
        );
        assert_eq!(
            i64::from(sql_count),
            live_count,
            "the reachable live element count should match the admin snapshot on a connected fixture",
        );
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
    fn test_tqhnsw_rescan_scaffold_accepts_unused_zero_key_buffer() {
        let query = vec![1.0, 0.0, 0.5, -1.0];
        let index_oid = setup_rescan_scaffold_index("tqhnsw_rescan_scaffold_zero_qual_buffer");
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
        ) = unsafe { am::debug_rescan_with_unused_key_buffer(index_oid, query.clone()) };

        assert!(rescan_called, "amrescan should still initialize scan state");
        assert_eq!(query_dimensions, query.len() as u16);
        assert_eq!(stored_query, query);
        assert_eq!(scan_dimensions, 4);
        assert_eq!(scan_bits, 4);
        assert_eq!(scan_code_len, code_len(4, 4));
        assert!(
            scan_block_count >= 2,
            "rescan should cache the current index block count"
        );
        assert!(
            has_prepared_query,
            "non-empty rescans should prepare the query"
        );
        assert_eq!(prepared_lut_len, 32);
        assert_eq!(prepared_sq_len, 4);
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
    fn test_tqhnsw_sql_ordered_index_scan_executes() {
        Spi::run(
            "CREATE TABLE tqhnsw_sql_ordered_exec (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO tqhnsw_sql_ordered_exec VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_tqvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX tqhnsw_sql_ordered_exec_idx ON tqhnsw_sql_ordered_exec USING tqhnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("ANALYZE tqhnsw_sql_ordered_exec").expect("analyze should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM tqhnsw_sql_ordered_exec \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
                     LIMIT 2",
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
            plan.contains("Index Scan") || plan.contains("Index Only Scan"),
            "ordered execution test should route through tqhnsw at runtime: {plan}"
        );

        let ordered_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id FROM tqhnsw_sql_ordered_exec \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
                     LIMIT 2",
                    None,
                    &[],
                )
                .expect("ordered SELECT should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(
            ordered_ids.len(),
            2,
            "query should return the requested LIMIT"
        );
        assert_eq!(
            ordered_ids[0], 1,
            "runtime ordered index scan should return the nearest vector first"
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
        Spi::run("RESET tqhnsw.ef_search").expect("reset should succeed");
        Spi::run("RESET tqhnsw.disable_binary_prefilter").expect("reset should succeed");
        Spi::run("RESET tqhnsw.force_binary_derivation").expect("reset should succeed");
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
            snapshot.planner_scan_enabled,
            "planner-facing scan selection should be live after D2 cost-model activation"
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
            neighbor_count <= am::page::neighbor_slots(metadata.max_level, metadata.m),
            "current-result neighbor count should decode within persisted neighbor capacity"
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
    type GraphScanRecallExternalSummaryRow = (
        i32, // m
        i32, // ef_search
        i32, // corpus_rows
        i32, // query_count
        f32, // graph_recall_at_10
        f32, // graph_recall_at_100
        f32, // ndcg_at_10
        f32, // mean_abs_score_error
        f32, // spearman_rho_at_10
        f32, // exact_quantized_recall_at_10
        i32, // graph_below_exact_queries
        i32, // worst_exact_gap
    );
    type GraphScanRecallAnnBenchmarksReferenceRow = (
        i32,  // m
        i32,  // ef_search
        f32,  // recall_at_10
        f32,  // published_recall_at_10
        f32,  // absolute_delta
        bool, // within_two_percent
    );
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
    type GraphScanRecallHistogramRow = (
        i32, // recall_bucket (0..=10)
        i32, // query_count
        f32, // query_fraction
    );
    type GraphScanRecallEfSweepRow = (
        i32, // m
        i32, // ef_search
        f32, // recall_at_10
        f32, // exact_quantized_recall_at_10
        f32, // mean_abs_score_error
        f32, // mean_query_latency_ms
    );
    type GraphScanRecallFailureBreakdownRow = (
        i32,      // query_index
        i32,      // graph_recall_at_10
        i32,      // exact_quantized_recall_at_10
        Vec<i64>, // missed_ids
    );

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

    /// Read `(id, source)` rows from a corpus / query table loaded by
    /// `scripts/load_real_corpus.py`. The returned vectors preserve the row
    /// order returned by Postgres so that ground-truth indices stay stable
    /// across reruns.
    fn load_external_recall_relation(table_name: &str) -> (Vec<i64>, Vec<Vec<f32>>) {
        let table_name = recall_fixture_ident(table_name);
        Spi::connect(|client| {
            let mut ids: Vec<i64> = Vec::new();
            let mut vectors: Vec<Vec<f32>> = Vec::new();
            let rows = client
                .select(
                    &format!("SELECT id, source FROM {table_name} ORDER BY id"),
                    None,
                    &[],
                )
                .expect("external recall relation query should succeed");
            for row in rows {
                let id = row["id"]
                    .value::<i64>()
                    .expect("id should decode")
                    .expect("id should be non-null");
                let source = row["source"]
                    .value::<Vec<f32>>()
                    .expect("source real[] should decode")
                    .expect("source real[] should be non-null");
                ids.push(id);
                vectors.push(source);
            }
            (ids, vectors)
        })
    }

    struct ExternalRecallContext {
        corpus_ids: Vec<i64>,
        corpus: Vec<Vec<f32>>,
        queries: Vec<Vec<f32>>,
        ground_truth_top_k: Vec<Vec<(usize, f32)>>,
        exact_quantized_row_indices_top10: Option<Vec<Vec<i64>>>,
        ctid_to_row_index: HashMap<(u32, u16), usize>,
    }

    fn build_external_recall_context(
        corpus_table: &str,
        query_table: &str,
        include_exact_quantized_top10: bool,
    ) -> ExternalRecallContext {
        let corpus_table_ident = recall_fixture_ident(corpus_table);
        let (corpus_ids, corpus) = load_external_recall_relation(corpus_table);
        let (_query_ids, queries) = load_external_recall_relation(query_table);

        assert!(
            !corpus.is_empty(),
            "external recall corpus {corpus_table_ident} must contain at least one row"
        );
        assert!(
            !queries.is_empty(),
            "external recall query table must contain at least one row"
        );

        let recall_k_wide = RECALL_K * 10;
        let ground_truth_top_k: Vec<Vec<(usize, f32)>> = queries
            .iter()
            .map(|query| {
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
                scores.truncate(recall_k_wide);
                scores
            })
            .collect();

        let id_to_row_index: HashMap<i64, usize> = corpus_ids
            .iter()
            .enumerate()
            .map(|(idx, id)| (*id, idx))
            .collect();
        let ctid_to_row_index: HashMap<(u32, u16), usize> = ctid_id_map(&corpus_table_ident)
            .into_iter()
            .map(|(ctid, id)| {
                let id_i64 = i64::try_from(id).expect("ctid id should fit into bigint");
                let row_index = *id_to_row_index
                    .get(&id_i64)
                    .expect("ctid id should map back to a corpus row index");
                (ctid, row_index)
            })
            .collect();
        let exact_quantized_row_indices_top10 = include_exact_quantized_top10.then(|| {
            Spi::connect(|client| {
                queries
                    .iter()
                    .map(|query| {
                        client
                            .select(
                                &format!(
                                    "SELECT id
                                     FROM {corpus_table_ident}
                                     ORDER BY embedding <#> $1
                                     LIMIT 10"
                                ),
                                None,
                                &[query.clone().into()],
                            )
                            .expect("exact quantized external recall query should succeed")
                            .map(|row| {
                                let id = row["id"]
                                    .value::<i64>()
                                    .expect("id should decode")
                                    .expect("id should be non-null");
                                i64::try_from(*id_to_row_index.get(&id).expect(
                                    "exact quantized id should map back to a corpus row index",
                                ))
                                .expect("row index should fit into bigint")
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect()
            })
        });

        ExternalRecallContext {
            corpus_ids,
            corpus,
            queries,
            ground_truth_top_k,
            exact_quantized_row_indices_top10,
            ctid_to_row_index,
        }
    }

    fn ndcg_at_k_external(true_top_k: &[(usize, f32)], pred_ids: &[i64], k: usize) -> f32 {
        let relevance: HashMap<usize, f32> =
            true_top_k.iter().take(k).map(|(i, s)| (*i, *s)).collect();

        let dcg: f32 = pred_ids
            .iter()
            .take(k)
            .enumerate()
            .map(|(rank, idx)| {
                let rel = *idx as usize;
                let score = relevance.get(&rel).copied().unwrap_or(0.0).max(0.0);
                score / ((rank as f32 + 2.0).ln() / 2.0_f32.ln())
            })
            .sum();

        let idcg: f32 = true_top_k
            .iter()
            .take(k)
            .enumerate()
            .map(|(rank, (_, score))| {
                let rel = score.max(0.0);
                rel / ((rank as f32 + 2.0).ln() / 2.0_f32.ln())
            })
            .sum();

        if idcg == 0.0 {
            0.0
        } else {
            dcg / idcg
        }
    }

    fn spearman_rank_correlation_external(true_top_k: &[(usize, f32)], pred_ids: &[i64]) -> f32 {
        let n = true_top_k.len().min(pred_ids.len());
        if n < 2 {
            return 0.0;
        }

        let pred_rank: HashMap<usize, usize> = pred_ids
            .iter()
            .enumerate()
            .take(n)
            .map(|(rank, idx)| (*idx as usize, rank))
            .collect();

        let mut d_squared_sum = 0.0_f64;
        for (true_rank, (idx, _)) in true_top_k.iter().enumerate().take(n) {
            let pred_r = pred_rank.get(idx).copied().unwrap_or(n);
            let d = true_rank as f64 - pred_r as f64;
            d_squared_sum += d * d;
        }

        let n = n as f64;
        1.0 - (6.0 * d_squared_sum / (n * (n * n - 1.0))) as f32
    }

    fn probe_graph_scan_recall_external_summary_for_context(
        context: &ExternalRecallContext,
        index_name: &str,
        m: i32,
        ef_search: i32,
    ) -> GraphScanRecallExternalSummaryRow {
        let index_name_ident = recall_fixture_ident(index_name);
        let recall_k_wide = RECALL_K * 10;

        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name_ident}'::regclass::oid"))
                .expect("external recall index oid query should succeed")
                .expect("external recall index oid should exist");

        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let query_count = context.queries.len();
        let mut graph_top_10_hits = 0_i32;
        let mut graph_top_100_hits = 0_i32;
        let mut exact_top_10_hits = 0_i32;
        let mut graph_below_exact_queries = 0_i32;
        let mut worst_exact_gap = 0_i32;
        let mut ndcg_sum = 0.0_f32;
        let mut mae_sum = 0.0_f32;
        let mut spearman_sum = 0.0_f32;
        let exact_quantized_row_indices_top10 = context
            .exact_quantized_row_indices_top10
            .as_ref()
            .expect("summary context should include exact quantized top-10 rows");

        for ((query, truth), exact_quantized_row_indices) in context
            .queries
            .iter()
            .zip(context.ground_truth_top_k.iter())
            .zip(exact_quantized_row_indices_top10.iter())
        {
            // Graph scan: returns heap tids plus operator-facing `<#>` scores.
            let predicted_row_indices_with_scores: Vec<(usize, f32)> =
                unsafe { am::debug_gettuple_scan_heap_tids_with_scores(index_oid, query.clone()) }
                    .into_iter()
                    .map(|(heap_tid, operator_score)| {
                        let row_index = *context
                            .ctid_to_row_index
                            .get(&heap_tid)
                            .expect("graph heap tid should map back to a corpus row index");
                        (row_index, operator_score)
                    })
                    .collect();
            let predicted_row_indices: Vec<i64> = predicted_row_indices_with_scores
                .iter()
                .map(|(row_index, _)| {
                    i64::try_from(*row_index).expect("row index should fit into bigint")
                })
                .collect();

            // Top-10 graph recall vs fp32 truth (row-index space).
            let truth_top_10_ids: Vec<i64> = truth
                .iter()
                .take(RECALL_K)
                .map(|(idx, _)| *idx as i64)
                .collect();
            let predicted_top_10_ids: Vec<i64> = predicted_row_indices
                .iter()
                .take(RECALL_K)
                .copied()
                .collect();
            let graph_overlap_10 = recall_top_k_overlap(&truth_top_10_ids, &predicted_top_10_ids);
            graph_top_10_hits += graph_overlap_10;

            // Top-100 graph recall vs the wider truth band.
            let truth_top_100_ids: Vec<i64> = truth
                .iter()
                .take(recall_k_wide)
                .map(|(idx, _)| *idx as i64)
                .collect();
            let predicted_top_100_ids: Vec<i64> = predicted_row_indices
                .iter()
                .take(recall_k_wide)
                .copied()
                .collect();
            graph_top_100_hits += recall_top_k_overlap(&truth_top_100_ids, &predicted_top_100_ids);

            let exact_overlap_10 =
                recall_top_k_overlap(&truth_top_10_ids, exact_quantized_row_indices);
            exact_top_10_hits += exact_overlap_10;

            if graph_overlap_10 < exact_overlap_10 {
                graph_below_exact_queries += 1;
                worst_exact_gap = worst_exact_gap.max(exact_overlap_10 - graph_overlap_10);
            }

            ndcg_sum += ndcg_at_k_external(truth, &predicted_top_10_ids, RECALL_K);
            spearman_sum += spearman_rank_correlation_external(
                &truth.iter().take(RECALL_K).copied().collect::<Vec<_>>(),
                &predicted_top_10_ids,
            );

            // NFR-003 MAE: per predicted item, compare the graph's approximate
            // inner product estimate against the true fp32 inner product for
            // that same item. The operator-facing `<#>` score is ascending
            // negative inner product, so negate it back into similarity space.
            let predicted_top_10_score_errors: Vec<f32> = predicted_row_indices_with_scores
                .iter()
                .take(RECALL_K)
                .map(|(row_index, operator_score)| {
                    let approx_inner_product = -*operator_score;
                    let true_inner_product = dot_product(query, &context.corpus[*row_index]);
                    (approx_inner_product - true_inner_product).abs()
                })
                .collect();
            if !predicted_top_10_score_errors.is_empty() {
                mae_sum += predicted_top_10_score_errors.iter().sum::<f32>()
                    / predicted_top_10_score_errors.len() as f32;
            }
        }

        let recall_10_denom = (query_count as f32) * (RECALL_K as f32);
        let recall_100_denom = (query_count as f32) * (recall_k_wide as f32);
        (
            m,
            ef_search,
            i32::try_from(context.corpus_ids.len()).expect("corpus row count should fit into int"),
            i32::try_from(query_count).expect("query count should fit into int"),
            graph_top_10_hits as f32 / recall_10_denom,
            graph_top_100_hits as f32 / recall_100_denom,
            ndcg_sum / query_count as f32,
            mae_sum / query_count as f32,
            spearman_sum / query_count as f32,
            exact_top_10_hits as f32 / recall_10_denom,
            graph_below_exact_queries,
            worst_exact_gap,
        )
    }

    fn probe_graph_scan_recall_external_summary_for_relation(
        corpus_table: &str,
        query_table: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
    ) -> GraphScanRecallExternalSummaryRow {
        let context = build_external_recall_context(corpus_table, query_table, true);
        probe_graph_scan_recall_external_summary_for_context(&context, index_name, m, ef_search)
    }

    fn probe_graph_scan_recall_external_gate_row_for_context(
        context: &ExternalRecallContext,
        index_name: &str,
        m: i32,
        ef_search: i32,
        target: Option<f32>,
    ) -> (i32, i32, f32, Option<f32>, bool) {
        let index_name_ident = recall_fixture_ident(index_name);

        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name_ident}'::regclass::oid"))
                .expect("external recall index oid query should succeed")
                .expect("external recall index oid should exist");

        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut graph_top_10_hits = 0_i32;
        for (query, truth) in context
            .queries
            .iter()
            .zip(context.ground_truth_top_k.iter())
        {
            let predicted_row_indices: Vec<i64> =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .map(|heap_tid| {
                        let row_index = *context
                            .ctid_to_row_index
                            .get(&heap_tid)
                            .expect("graph heap tid should map back to a corpus row index");
                        i64::try_from(row_index).expect("row index should fit into bigint")
                    })
                    .collect();

            let truth_top_10_ids: Vec<i64> = truth
                .iter()
                .take(RECALL_K)
                .map(|(idx, _)| *idx as i64)
                .collect();
            let predicted_top_10_ids: Vec<i64> = predicted_row_indices
                .iter()
                .take(RECALL_K)
                .copied()
                .collect();
            graph_top_10_hits += recall_top_k_overlap(&truth_top_10_ids, &predicted_top_10_ids);
        }

        let recall_at_10 =
            graph_top_10_hits as f32 / ((context.queries.len() as f32) * (RECALL_K as f32));
        let passed = target.map(|gate| recall_at_10 >= gate).unwrap_or(true);
        (m, ef_search, recall_at_10, target, passed)
    }

    // One-shot oracle: re-uses the external recall context machinery and
    // compares the measured `recall@10` against the published anchor recorded
    // in `docs/RECALL_ANN_BENCHMARKS_ANCHOR.md`. This is intentionally not a
    // sweep — anchor diagnostics live in the histogram / ef_sweep surfaces.
    fn probe_graph_scan_recall_ann_benchmarks_reference_for_relation(
        corpus_table: &str,
        query_table: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
    ) -> GraphScanRecallAnnBenchmarksReferenceRow {
        let summary = probe_graph_scan_recall_external_summary_for_relation(
            corpus_table,
            query_table,
            index_name,
            m,
            ef_search,
        );
        let measured_recall_at_10 = summary.4;
        let absolute_delta = measured_recall_at_10 - ANN_BENCHMARKS_ANCHOR_PUBLISHED_RECALL_AT_10;
        let within_two_percent = absolute_delta.abs() <= ANN_BENCHMARKS_ANCHOR_TOLERANCE;
        (
            m,
            ef_search,
            measured_recall_at_10,
            ANN_BENCHMARKS_ANCHOR_PUBLISHED_RECALL_AT_10,
            absolute_delta,
            within_two_percent,
        )
    }

    fn run_graph_scan_recall_gate_from_external(
        corpus_table: &str,
        query_table: &str,
        fixture_prefix: &str,
    ) -> Vec<(i32, i32, f32, Option<f32>, bool)> {
        let fixture_prefix = recall_fixture_ident(fixture_prefix);
        let context = build_external_recall_context(corpus_table, query_table, false);
        RECALL_GATE_CONFIGS
            .iter()
            .copied()
            .map(|(m, ef_search, target)| {
                let index_name = format!("{fixture_prefix}_m{m}_idx");
                probe_graph_scan_recall_external_gate_row_for_context(
                    &context,
                    &index_name,
                    m,
                    ef_search,
                    target,
                )
            })
            .collect()
    }

    /// Returns one row per top-10 recall bucket (`0..=10`). Buckets with no
    /// queries are still emitted so the output is always 11 rows. Builds the
    /// graph scan top-10 for each query in the supplied context and bins by
    /// the count of correct items vs the precomputed fp32 ground truth.
    fn build_graph_scan_recall_histogram_for_context(
        context: &ExternalRecallContext,
        index_name: &str,
        ef_search: i32,
    ) -> Vec<GraphScanRecallHistogramRow> {
        let index_name_ident = recall_fixture_ident(index_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name_ident}'::regclass::oid"))
                .expect("histogram index oid query should succeed")
                .expect("histogram index oid should exist");

        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut buckets = [0_i32; RECALL_K + 1];
        for (query, truth) in context
            .queries
            .iter()
            .zip(context.ground_truth_top_k.iter())
        {
            let truth_top_10_ids: Vec<i64> = truth
                .iter()
                .take(RECALL_K)
                .map(|(idx, _)| *idx as i64)
                .collect();
            let predicted_top_10_ids: Vec<i64> =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        let row_index = *context
                            .ctid_to_row_index
                            .get(&heap_tid)
                            .expect("graph heap tid should map back to a corpus row index");
                        i64::try_from(row_index).expect("row index should fit into bigint")
                    })
                    .collect();
            let overlap = recall_top_k_overlap(&truth_top_10_ids, &predicted_top_10_ids);
            let bucket = usize::try_from(overlap)
                .expect("overlap should be non-negative")
                .min(RECALL_K);
            buckets[bucket] += 1;
        }

        let total_queries = context.queries.len() as f32;
        (0..=RECALL_K)
            .map(|bucket| {
                let count = buckets[bucket];
                let fraction = if total_queries > 0.0 {
                    count as f32 / total_queries
                } else {
                    0.0
                };
                (
                    i32::try_from(bucket).expect("bucket index should fit into int"),
                    count,
                    fraction,
                )
            })
            .collect()
    }

    /// Sweeps a list of `ef_search` values against a single fixture, building
    /// the external recall context exactly once and reusing it for every probe.
    /// Per-row latency is the wall clock spent inside
    /// `probe_graph_scan_recall_external_summary_for_context` for that
    /// `ef_search`, divided by the query count — it includes the small per-row
    /// overhead of NDCG/MAE/Spearman bookkeeping but is dominated by the graph
    /// scan itself.
    fn run_graph_scan_recall_ef_sweep_for_context(
        context: &ExternalRecallContext,
        index_name: &str,
        m: i32,
        ef_values: &[i32],
    ) -> Vec<GraphScanRecallEfSweepRow> {
        let query_count = context.queries.len();
        ef_values
            .iter()
            .copied()
            .map(|ef_search| {
                let started = Instant::now();
                let summary = probe_graph_scan_recall_external_summary_for_context(
                    context, index_name, m, ef_search,
                );
                let elapsed = started.elapsed();
                let mean_query_latency_ms = if query_count > 0 {
                    (elapsed.as_secs_f64() * 1000.0 / query_count as f64) as f32
                } else {
                    0.0
                };
                (
                    m,
                    ef_search,
                    summary.4, // graph_recall_at_10
                    summary.9, // exact_quantized_recall_at_10
                    summary.7, // mean_abs_score_error
                    mean_query_latency_ms,
                )
            })
            .collect()
    }

    /// Lists every query whose top-10 graph recall is strictly less than
    /// `recall_threshold`, alongside the exact-quantized recall on the same
    /// query and the corpus ids that neither retrieval surface managed to
    /// find. Rows are emitted in `query_index` order so the output is
    /// deterministic for diffing.
    fn run_graph_scan_recall_failure_breakdown_for_context(
        context: &ExternalRecallContext,
        index_name: &str,
        ef_search: i32,
        recall_threshold: i32,
    ) -> Vec<GraphScanRecallFailureBreakdownRow> {
        let index_name_ident = recall_fixture_ident(index_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name_ident}'::regclass::oid"))
                .expect("failure breakdown index oid query should succeed")
                .expect("failure breakdown index oid should exist");
        let exact_quantized_row_indices_top10 = context
            .exact_quantized_row_indices_top10
            .as_ref()
            .expect("failure breakdown context should include exact quantized top-10 rows");

        Spi::run(&format!("SET LOCAL tqhnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut rows = Vec::new();
        for (query_index, ((query, truth), exact_quantized_row_indices)) in context
            .queries
            .iter()
            .zip(context.ground_truth_top_k.iter())
            .zip(exact_quantized_row_indices_top10.iter())
            .enumerate()
        {
            let truth_top_10_row_indices: Vec<i64> = truth
                .iter()
                .take(RECALL_K)
                .map(|(idx, _)| *idx as i64)
                .collect();
            let predicted_top_10_row_indices: Vec<i64> =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        let row_index = *context
                            .ctid_to_row_index
                            .get(&heap_tid)
                            .expect("graph heap tid should map back to a corpus row index");
                        i64::try_from(row_index).expect("row index should fit into bigint")
                    })
                    .collect();
            let graph_recall =
                recall_top_k_overlap(&truth_top_10_row_indices, &predicted_top_10_row_indices);
            if graph_recall >= recall_threshold {
                continue;
            }
            let exact_recall =
                recall_top_k_overlap(&truth_top_10_row_indices, exact_quantized_row_indices);

            // Missed = truth_top_10 \ (graph_top_10 ∪ exact_quantized_top_10),
            // mapped from row indices back to corpus ids so the output is
            // human-actionable.
            let missed_ids: Vec<i64> = truth_top_10_row_indices
                .iter()
                .filter(|row_index| {
                    !predicted_top_10_row_indices.contains(row_index)
                        && !exact_quantized_row_indices.contains(row_index)
                })
                .map(|row_index| {
                    let idx = usize::try_from(*row_index)
                        .expect("missed row index should be non-negative");
                    context.corpus_ids[idx]
                })
                .collect();

            rows.push((
                i32::try_from(query_index).expect("query index should fit into int"),
                graph_recall,
                exact_recall,
                missed_ids,
            ));
        }
        rows
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
        frequent_oracle_seeds
            .sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
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
    fn tqhnsw_graph_scan_recall_external_summary(
        corpus_table: String,
        query_table: String,
        index_name: String,
        m: i32,
        ef_search: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(corpus_rows, i32),
            name!(query_count, i32),
            name!(graph_recall_at_10, f32),
            name!(graph_recall_at_100, f32),
            name!(ndcg_at_10, f32),
            name!(mean_abs_score_error, f32),
            name!(spearman_rho_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(graph_below_exact_queries, i32),
            name!(worst_exact_gap, i32),
        ),
    > {
        TableIterator::once(probe_graph_scan_recall_external_summary_for_relation(
            &corpus_table,
            &query_table,
            &index_name,
            m,
            ef_search,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_external_gate_report(
        corpus_table: String,
        query_table: String,
        fixture_prefix: String,
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
        TableIterator::new(run_graph_scan_recall_gate_from_external(
            &corpus_table,
            &query_table,
            &fixture_prefix,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_ann_benchmarks_reference(
        corpus_table: String,
        query_table: String,
        index_name: String,
        m: i32,
        ef_search: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(recall_at_10, f32),
            name!(published_recall_at_10, f32),
            name!(absolute_delta, f32),
            name!(within_two_percent, bool),
        ),
    > {
        TableIterator::once(
            probe_graph_scan_recall_ann_benchmarks_reference_for_relation(
                &corpus_table,
                &query_table,
                &index_name,
                m,
                ef_search,
            ),
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_histogram(
        corpus_table: String,
        query_table: String,
        index_name: String,
        m: i32,
        ef_search: i32,
    ) -> TableIterator<
        'static,
        (
            name!(recall_bucket, i32),
            name!(query_count, i32),
            name!(query_fraction, f32),
        ),
    > {
        // `m` is part of the SQL signature for parity with the other recall
        // diagnostics; the histogram itself is fully determined by `index_name`
        // and `ef_search`.
        let _ = m;
        let context = build_external_recall_context(&corpus_table, &query_table, false);
        TableIterator::new(build_graph_scan_recall_histogram_for_context(
            &context,
            &index_name,
            ef_search,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_ef_sweep(
        corpus_table: String,
        query_table: String,
        index_name: String,
        m: i32,
        ef_values: Vec<i32>,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(recall_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(mean_abs_score_error, f32),
            name!(mean_query_latency_ms, f32),
        ),
    > {
        let context = build_external_recall_context(&corpus_table, &query_table, true);
        TableIterator::new(run_graph_scan_recall_ef_sweep_for_context(
            &context,
            &index_name,
            m,
            &ef_values,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_graph_scan_recall_failure_breakdown(
        corpus_table: String,
        query_table: String,
        index_name: String,
        m: i32,
        ef_search: i32,
        recall_threshold: i32,
    ) -> TableIterator<
        'static,
        (
            name!(query_index, i32),
            name!(graph_recall_at_10, i32),
            name!(exact_quantized_recall_at_10, i32),
            name!(missed_ids, Vec<i64>),
        ),
    > {
        // `m` is part of the SQL signature for parity with the other recall
        // diagnostics; the breakdown itself is fully determined by `index_name`
        // and `ef_search`.
        let _ = m;
        let context = build_external_recall_context(&corpus_table, &query_table, true);
        TableIterator::new(run_graph_scan_recall_failure_breakdown_for_context(
            &context,
            &index_name,
            ef_search,
            recall_threshold,
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

        // Build a map of neighbor TID -> decoded neighbor tuple
        let neighbor_map: HashMap<am::page::ItemPointer, am::page::TqNeighborTuple> = data_pages
            .iter()
            .flat_map(|page| {
                page.tuples
                    .iter()
                    .enumerate()
                    .filter_map(move |(idx, tuple)| {
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

                // For each layer this element participates in, count valid neighbors
                for layer in 0..=element.level {
                    let (start, end) = if layer == 0 {
                        (0, m * 2)
                    } else {
                        let s = m * 2 + (usize::from(layer) - 1) * m;
                        (s, s + m)
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

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_debug_scan_profile(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<
        'static,
        (
            name!(rescan_elapsed_us, i64),
            name!(emit_elapsed_us, i64),
            name!(total_elapsed_us, i64),
            name!(rescan_phase, String),
            name!(rescan_current_result, bool),
            name!(rescan_ordered_slots, i32),
            name!(rescan_pending_heap_tids, i32),
            name!(rescan_visited_elements, i32),
            name!(rescan_expanded_sources, i32),
            name!(rescan_emitted_elements, i32),
            name!(rescan_bootstrap_expansions, i32),
            name!(rescan_bootstrap_pages_read, i32),
            name!(rescan_quantizer_cache_hit, bool),
            name!(result_count, i32),
            name!(final_phase, String),
            name!(final_ordered_slots, i32),
            name!(total_bootstrap_expansions, i32),
            name!(total_bootstrap_pages_read, i32),
            name!(total_linear_pages_read, i32),
            name!(total_elements_scored, i32),
            name!(total_elements_skipped, i32),
            name!(total_heap_tids_returned, i32),
            name!(total_quantizer_cache_hit, bool),
            name!(total_emitted_elements, i32),
        ),
    > {
        let index_relation =
            unsafe { open_valid_tqhnsw_index(index_oid, "tests.tqhnsw_debug_scan_profile") };
        unsafe {
            pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        }

        let (
            rescan_elapsed_us,
            emit_elapsed_us,
            total_elapsed_us,
            rescan_phase,
            rescan_current_result,
            rescan_ordered_slots,
            rescan_pending_heap_tids,
            rescan_visited_elements,
            rescan_expanded_sources,
            rescan_emitted_elements,
            rescan_bootstrap_expansions,
            rescan_bootstrap_pages_read,
            rescan_quantizer_cache_hit,
            result_count,
            final_phase,
            final_ordered_slots,
            total_bootstrap_expansions,
            total_bootstrap_pages_read,
            total_linear_pages_read,
            total_elements_scored,
            total_elements_skipped,
            total_heap_tids_returned,
            total_quantizer_cache_hit,
            total_emitted_elements,
            _rescan_amrescan_total_elapsed_us,
            _rescan_query_decode_elapsed_us,
            _rescan_scan_setup_elapsed_us,
            _rescan_store_query_elapsed_us,
            _rescan_prepare_query_elapsed_us,
            _rescan_reset_state_elapsed_us,
            _rescan_initialize_entry_elapsed_us,
            _rescan_upper_layer_seed_elapsed_us,
            _rescan_layer0_seed_elapsed_us,
            _rescan_stage_ordered_results_elapsed_us,
            _rescan_initial_prefetch_elapsed_us,
            _rescan_frontier_consume_elapsed_us,
            _rescan_graph_result_materialize_elapsed_us,
            _graph_element_cache_hits,
            _graph_element_cache_misses,
            _graph_element_load_elapsed_us,
            _graph_neighbor_cache_hits,
            _graph_neighbor_cache_misses,
            _graph_neighbor_load_elapsed_us,
            _candidate_score_calls,
            _candidate_score_elapsed_us,
            _score_cache_hits,
            _score_cache_misses,
            _grouped_traversal_approx_score_calls,
            _grouped_traversal_approx_score_elapsed_us,
            _grouped_traversal_exact_score_calls,
            _grouped_traversal_exact_score_elapsed_us,
            _grouped_traversal_budgeted_expansions,
            _grouped_traversal_budgeted_candidates,
            _grouped_traversal_budgeted_exact_candidates,
        ) = unsafe { am::debug_profile_ordered_scan(index_oid, query) };

        TableIterator::once((
            rescan_elapsed_us,
            emit_elapsed_us,
            total_elapsed_us,
            rescan_phase,
            rescan_current_result,
            rescan_ordered_slots,
            rescan_pending_heap_tids,
            rescan_visited_elements,
            rescan_expanded_sources,
            rescan_emitted_elements,
            rescan_bootstrap_expansions,
            rescan_bootstrap_pages_read,
            rescan_quantizer_cache_hit,
            result_count,
            final_phase,
            final_ordered_slots,
            total_bootstrap_expansions,
            total_bootstrap_pages_read,
            total_linear_pages_read,
            total_elements_scored,
            total_elements_skipped,
            total_heap_tids_returned,
            total_quantizer_cache_hit,
            total_emitted_elements,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_debug_adr030_runtime_settings() -> TableIterator<
        'static,
        (
            name!(grouped_build_enabled, bool),
            name!(grouped_scan_enabled, bool),
            name!(grouped_scan_window, Option<String>),
            name!(grouped_exact_traversal_enabled, bool),
            name!(grouped_exact_traversal_scope, Option<String>),
            name!(grouped_exact_traversal_limit, Option<String>),
        ),
    > {
        TableIterator::once((
            std::env::var_os("TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD").is_some(),
            std::env::var_os("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN").is_some(),
            std::env::var_os("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_WINDOW")
                .map(|value| value.to_string_lossy().into_owned()),
            std::env::var_os("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL").is_some(),
            std::env::var_os("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_SCOPE")
                .map(|value| value.to_string_lossy().into_owned()),
            std::env::var_os("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_LIMIT")
                .map(|value| value.to_string_lossy().into_owned()),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_debug_scan_hot_path_profile(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<
        'static,
        (
            name!(rescan_amrescan_total_elapsed_us, i64),
            name!(rescan_query_decode_elapsed_us, i64),
            name!(rescan_scan_setup_elapsed_us, i64),
            name!(rescan_store_query_elapsed_us, i64),
            name!(rescan_prepare_query_elapsed_us, i64),
            name!(rescan_reset_state_elapsed_us, i64),
            name!(rescan_initialize_entry_elapsed_us, i64),
            name!(rescan_upper_layer_seed_elapsed_us, i64),
            name!(rescan_layer0_seed_elapsed_us, i64),
            name!(rescan_stage_ordered_results_elapsed_us, i64),
            name!(rescan_initial_prefetch_elapsed_us, i64),
            name!(rescan_frontier_consume_elapsed_us, i64),
            name!(rescan_graph_result_materialize_elapsed_us, i64),
            name!(graph_element_cache_hits, i32),
            name!(graph_element_cache_misses, i32),
            name!(graph_element_load_elapsed_us, i64),
            name!(graph_neighbor_cache_hits, i32),
            name!(graph_neighbor_cache_misses, i32),
            name!(graph_neighbor_load_elapsed_us, i64),
            name!(candidate_score_calls, i32),
            name!(candidate_score_elapsed_us, i64),
            name!(score_cache_hits, i32),
            name!(score_cache_misses, i32),
            name!(grouped_traversal_approx_score_calls, i32),
            name!(grouped_traversal_approx_score_elapsed_us, i64),
            name!(grouped_traversal_exact_score_calls, i32),
            name!(grouped_traversal_exact_score_elapsed_us, i64),
            name!(grouped_traversal_budgeted_expansions, i32),
            name!(grouped_traversal_budgeted_candidates, i32),
            name!(grouped_traversal_budgeted_exact_candidates, i32),
        ),
    > {
        let index_relation = unsafe {
            open_valid_tqhnsw_index(index_oid, "tests.tqhnsw_debug_scan_hot_path_profile")
        };
        unsafe {
            pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        }

        let (
            _rescan_elapsed_us,
            _emit_elapsed_us,
            _total_elapsed_us,
            _rescan_phase,
            _rescan_current_result,
            _rescan_ordered_slots,
            _rescan_pending_heap_tids,
            _rescan_visited_elements,
            _rescan_expanded_sources,
            _rescan_emitted_elements,
            _rescan_bootstrap_expansions,
            _rescan_bootstrap_pages_read,
            _rescan_quantizer_cache_hit,
            _result_count,
            _final_phase,
            _final_ordered_slots,
            _total_bootstrap_expansions,
            _total_bootstrap_pages_read,
            _total_linear_pages_read,
            _total_elements_scored,
            _total_elements_skipped,
            _total_heap_tids_returned,
            _total_quantizer_cache_hit,
            _total_emitted_elements,
            rescan_amrescan_total_elapsed_us,
            rescan_query_decode_elapsed_us,
            rescan_scan_setup_elapsed_us,
            rescan_store_query_elapsed_us,
            rescan_prepare_query_elapsed_us,
            rescan_reset_state_elapsed_us,
            rescan_initialize_entry_elapsed_us,
            rescan_upper_layer_seed_elapsed_us,
            rescan_layer0_seed_elapsed_us,
            rescan_stage_ordered_results_elapsed_us,
            rescan_initial_prefetch_elapsed_us,
            rescan_frontier_consume_elapsed_us,
            rescan_graph_result_materialize_elapsed_us,
            graph_element_cache_hits,
            graph_element_cache_misses,
            graph_element_load_elapsed_us,
            graph_neighbor_cache_hits,
            graph_neighbor_cache_misses,
            graph_neighbor_load_elapsed_us,
            candidate_score_calls,
            candidate_score_elapsed_us,
            score_cache_hits,
            score_cache_misses,
            grouped_traversal_approx_score_calls,
            grouped_traversal_approx_score_elapsed_us,
            grouped_traversal_exact_score_calls,
            grouped_traversal_exact_score_elapsed_us,
            grouped_traversal_budgeted_expansions,
            grouped_traversal_budgeted_candidates,
            grouped_traversal_budgeted_exact_candidates,
        ) = unsafe { am::debug_profile_ordered_scan(index_oid, query) };

        TableIterator::once((
            rescan_amrescan_total_elapsed_us,
            rescan_query_decode_elapsed_us,
            rescan_scan_setup_elapsed_us,
            rescan_store_query_elapsed_us,
            rescan_prepare_query_elapsed_us,
            rescan_reset_state_elapsed_us,
            rescan_initialize_entry_elapsed_us,
            rescan_upper_layer_seed_elapsed_us,
            rescan_layer0_seed_elapsed_us,
            rescan_stage_ordered_results_elapsed_us,
            rescan_initial_prefetch_elapsed_us,
            rescan_frontier_consume_elapsed_us,
            rescan_graph_result_materialize_elapsed_us,
            graph_element_cache_hits,
            graph_element_cache_misses,
            graph_element_load_elapsed_us,
            graph_neighbor_cache_hits,
            graph_neighbor_cache_misses,
            graph_neighbor_load_elapsed_us,
            candidate_score_calls,
            candidate_score_elapsed_us,
            score_cache_hits,
            score_cache_misses,
            grouped_traversal_approx_score_calls,
            grouped_traversal_approx_score_elapsed_us,
            grouped_traversal_exact_score_calls,
            grouped_traversal_exact_score_elapsed_us,
            grouped_traversal_budgeted_expansions,
            grouped_traversal_budgeted_candidates,
            grouped_traversal_budgeted_exact_candidates,
        ))
    }

    #[pg_extern]
    fn tqhnsw_debug_scan_result_count(index_oid: pg_sys::Oid, query: Vec<f32>) -> i32 {
        let index_relation =
            unsafe { open_valid_tqhnsw_index(index_oid, "tests.tqhnsw_debug_scan_result_count") };
        unsafe {
            pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        }

        i32::try_from(unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query) }.len())
            .expect("debug scan result count should fit in i32")
    }

    #[pg_extern]
    fn tqhnsw_debug_scan_heap_tids(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<'static, (name!(block_number, i64), name!(offset_number, i32))> {
        let index_relation =
            unsafe { open_valid_tqhnsw_index(index_oid, "tests.tqhnsw_debug_scan_heap_tids") };
        unsafe {
            pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        }

        let rows = unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query) }
            .into_iter()
            .map(|(block_number, offset_number)| {
                (i64::from(block_number), i32::from(offset_number))
            })
            .collect::<Vec<_>>();
        TableIterator::new(rows)
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_debug_grouped_scan_order_drift_summary(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<
        'static,
        (
            name!(emitted_result_count, i32),
            name!(grouped_result_count, i32),
            name!(compared_result_count, i32),
            name!(mean_abs_rank_shift, f64),
            name!(max_abs_rank_shift, i32),
            name!(spearman_rank_correlation, f64),
            name!(exact_best_approx_rank, Option<i32>),
            name!(exact_top4_max_approx_rank, Option<i32>),
            name!(window_1_contains_exact_best, bool),
            name!(window_2_contains_exact_best, bool),
            name!(window_4_contains_exact_best, bool),
            name!(window_8_contains_exact_best, bool),
        ),
    > {
        let index_relation = unsafe {
            open_valid_tqhnsw_index(
                index_oid,
                "tests.tqhnsw_debug_grouped_scan_order_drift_summary",
            )
        };
        unsafe {
            pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        }

        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            mean_abs_rank_shift,
            max_abs_rank_shift,
            spearman_rank_correlation,
            exact_best_approx_rank,
            exact_top4_max_approx_rank,
            window_1_contains_exact_best,
            window_2_contains_exact_best,
            window_4_contains_exact_best,
            window_8_contains_exact_best,
        ) = unsafe { am::debug_grouped_scan_order_drift_summary(index_oid, query) };

        TableIterator::once((
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            mean_abs_rank_shift,
            max_abs_rank_shift,
            spearman_rank_correlation,
            exact_best_approx_rank,
            exact_top4_max_approx_rank,
            window_1_contains_exact_best,
            window_2_contains_exact_best,
            window_4_contains_exact_best,
            window_8_contains_exact_best,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_debug_grouped_scan_windowed_rows(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
        window_size: i32,
    ) -> TableIterator<
        'static,
        (
            name!(block_number, i64),
            name!(offset_number, i32),
            name!(approx_rank, i32),
            name!(windowed_rank, i32),
            name!(approx_score, f32),
            name!(comparison_score, Option<f32>),
            name!(exact_rank, Option<i32>),
            name!(exact_rank_shift, Option<i32>),
            name!(windowed_rank_shift, Option<i32>),
        ),
    > {
        let index_relation = unsafe {
            open_valid_tqhnsw_index(index_oid, "tests.tqhnsw_debug_grouped_scan_windowed_rows")
        };
        unsafe {
            pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        }

        let rows = unsafe { am::debug_grouped_scan_windowed_rows(index_oid, query, window_size) }
            .into_iter()
            .map(
                |(
                    (block_number, offset_number),
                    approx_rank,
                    windowed_rank,
                    approx_score,
                    comparison_score,
                    exact_rank,
                    exact_rank_shift,
                    windowed_rank_shift,
                )| {
                    (
                        i64::from(block_number),
                        i32::from(offset_number),
                        approx_rank,
                        windowed_rank,
                        approx_score,
                        comparison_score,
                        exact_rank,
                        exact_rank_shift,
                        windowed_rank_shift,
                    )
                },
            )
            .collect::<Vec<_>>();
        TableIterator::new(rows)
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_debug_grouped_scan_windowed_summary(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
        window_size: i32,
    ) -> TableIterator<
        'static,
        (
            name!(emitted_result_count, i32),
            name!(grouped_result_count, i32),
            name!(compared_result_count, i32),
            name!(window_size, i32),
            name!(exact_best_approx_rank, Option<i32>),
            name!(exact_best_windowed_rank, Option<i32>),
            name!(exact_top4_max_approx_rank, Option<i32>),
            name!(exact_top4_max_windowed_rank, Option<i32>),
            name!(mean_abs_rank_shift_before, f64),
            name!(mean_abs_rank_shift_after, f64),
            name!(max_abs_rank_shift_before, i32),
            name!(max_abs_rank_shift_after, i32),
            name!(spearman_rank_correlation_before, f64),
            name!(spearman_rank_correlation_after, f64),
        ),
    > {
        let index_relation = unsafe {
            open_valid_tqhnsw_index(
                index_oid,
                "tests.tqhnsw_debug_grouped_scan_windowed_summary",
            )
        };
        unsafe {
            pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        }

        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            window_size,
            exact_best_approx_rank,
            exact_best_windowed_rank,
            exact_top4_max_approx_rank,
            exact_top4_max_windowed_rank,
            mean_abs_rank_shift_before,
            mean_abs_rank_shift_after,
            max_abs_rank_shift_before,
            max_abs_rank_shift_after,
            spearman_rank_correlation_before,
            spearman_rank_correlation_after,
        ) = unsafe { am::debug_grouped_scan_windowed_summary(index_oid, query, window_size) };

        TableIterator::once((
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            window_size,
            exact_best_approx_rank,
            exact_best_windowed_rank,
            exact_top4_max_approx_rank,
            exact_top4_max_windowed_rank,
            mean_abs_rank_shift_before,
            mean_abs_rank_shift_after,
            max_abs_rank_shift_before,
            max_abs_rank_shift_after,
            spearman_rank_correlation_before,
            spearman_rank_correlation_after,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_debug_grouped_scan_comparison_rows(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<
        'static,
        (
            name!(block_number, i64),
            name!(offset_number, i32),
            name!(approx_rank, i32),
            name!(approx_score, f32),
            name!(comparison_score, Option<f32>),
            name!(exact_rank, Option<i32>),
            name!(exact_rank_shift, Option<i32>),
        ),
    > {
        let index_relation = unsafe {
            open_valid_tqhnsw_index(index_oid, "tests.tqhnsw_debug_grouped_scan_comparison_rows")
        };
        unsafe {
            pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        }

        let rows = unsafe { am::debug_grouped_scan_comparison_rows(index_oid, query) }
            .into_iter()
            .map(
                |(
                    (block_number, offset_number),
                    approx_rank,
                    approx_score,
                    comparison_score,
                    exact_rank,
                    exact_rank_shift,
                )| {
                    (
                        i64::from(block_number),
                        i32::from(offset_number),
                        approx_rank,
                        approx_score,
                        comparison_score,
                        exact_rank,
                        exact_rank_shift,
                    )
                },
            )
            .collect::<Vec<_>>();
        TableIterator::new(rows)
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn tqhnsw_debug_grouped_scan_comparison_summary(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<
        'static,
        (
            name!(emitted_result_count, i32),
            name!(grouped_result_count, i32),
            name!(compared_result_count, i32),
            name!(missing_comparison_count, i32),
            name!(mean_abs_score_delta, f64),
            name!(max_abs_score_delta, f32),
            name!(mean_signed_score_delta, f64),
        ),
    > {
        let index_relation = unsafe {
            open_valid_tqhnsw_index(
                index_oid,
                "tests.tqhnsw_debug_grouped_scan_comparison_summary",
            )
        };
        unsafe {
            pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        }

        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            missing_comparison_count,
            mean_abs_score_delta,
            max_abs_score_delta,
            mean_signed_score_delta,
        ) = unsafe { am::debug_grouped_scan_comparison_summary(index_oid, query) };

        TableIterator::once((
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            missing_comparison_count,
            mean_abs_score_delta,
            max_abs_score_delta,
            mean_signed_score_delta,
        ))
    }

    #[pg_extern]
    fn tqhnsw_debug_reachable_live_element_count(index_oid: pg_sys::Oid) -> i32 {
        let index_relation = unsafe {
            open_valid_tqhnsw_index(index_oid, "tests.tqhnsw_debug_reachable_live_element_count")
        };
        unsafe {
            pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        }

        i32::try_from(unsafe { am::debug_layer0_reachable_live_element_tids(index_oid) }.len())
            .expect("debug reachable live element count should fit in i32")
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

    /// Helper that materializes the external corpus / query / index layout
    /// described in `docs/RECALL_REAL_CORPUS.md` for a small synthetic dataset.
    /// Used by the external-probe smoke test below.
    fn create_external_recall_smoke_fixture(prefix: &str, corpus_size: usize, query_count: usize) {
        let corpus_table = format!("{prefix}_corpus");
        let queries_table = format!("{prefix}_queries");

        Spi::run(&format!("DROP TABLE IF EXISTS {corpus_table} CASCADE"))
            .expect("smoke fixture corpus drop should succeed");
        Spi::run(&format!("DROP TABLE IF EXISTS {queries_table} CASCADE"))
            .expect("smoke fixture queries drop should succeed");

        Spi::run(&format!(
            "CREATE TABLE {corpus_table} (
                id bigint primary key,
                source real[] NOT NULL,
                embedding tqvector
            )"
        ))
        .expect("smoke fixture corpus create should succeed");
        Spi::run(&format!(
            "CREATE TABLE {queries_table} (
                id bigint primary key,
                source real[] NOT NULL
            )"
        ))
        .expect("smoke fixture queries create should succeed");

        let corpus = random_unit_vectors(corpus_size, RECALL_DIM, RECALL_SEED as u64);
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);

        // Seed both tables with a single multi-row INSERT each instead of
        // one statement per row. pgrx 0.17 does not expose a stable
        // SPI-level COPY FROM STDIN API, so a batched INSERT is the fastest
        // transport available without adding a Postgres client crate. Row
        // order, ids, vector floats, and the encode call are preserved
        // byte-for-byte from the previous per-row path so the recall summary
        // remains deterministic.
        let corpus_values = corpus
            .iter()
            .enumerate()
            .map(|(id, vector)| {
                let source = format_recall_vector_sql_literal(vector);
                format!(
                    "({id}, {source}, encode_to_tqvector({source}, {RECALL_BITS}, {RECALL_SEED}))"
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        Spi::run(&format!(
            "INSERT INTO {corpus_table} (id, source, embedding) VALUES {corpus_values}"
        ))
        .expect("smoke fixture corpus batch insert should succeed");

        let query_values = queries
            .iter()
            .enumerate()
            .map(|(id, vector)| {
                let source = format_recall_vector_sql_literal(vector);
                format!("({id}, {source})")
            })
            .collect::<Vec<_>>()
            .join(", ");
        Spi::run(&format!(
            "INSERT INTO {queries_table} (id, source) VALUES {query_values}"
        ))
        .expect("smoke fixture query batch insert should succeed");

        for m in [8_i32, 16_i32] {
            let index_name = format!("{prefix}_m{m}_idx");
            Spi::run(&format!(
                "CREATE INDEX {index_name} ON {corpus_table} \
                 USING tqhnsw (embedding tqvector_ip_ops) \
                 WITH (m = {m}, ef_construction = {RECALL_EF_CONSTRUCTION}, \
                       build_source_column = 'source')"
            ))
            .expect("smoke fixture index create should succeed");
        }
    }

    #[pg_test]
    // Ignored because it requires the `pg_test` cargo feature and a scratch
    // pgrx test cluster to run, not because of long seeding. Seeding is
    // batched in `create_external_recall_smoke_fixture`; the remaining
    // wall-clock cost lives in the probe / gate phases below, which are
    // out of scope for the seeding fix.
    #[ignore]
    fn test_tqhnsw_graph_scan_recall_external_smoke_500() {
        // Smoke test for the external corpus / query / index probe path. The
        // real DBpedia corpus is staged out-of-band by
        // `scripts/load_real_corpus.py`; here we substitute a tiny synthetic
        // dataset that the loader's schema accepts so we can exercise the
        // Rust probe surface end-to-end.
        let prefix = "tqhnsw_recall_external_smoke";
        create_external_recall_smoke_fixture(prefix, 500, 25);

        let corpus_table = format!("{prefix}_corpus");
        let queries_table = format!("{prefix}_queries");
        let m8_index = format!("{prefix}_m8_idx");

        let summary = probe_graph_scan_recall_external_summary_for_relation(
            &corpus_table,
            &queries_table,
            &m8_index,
            8,
            128,
        );
        let (
            m,
            ef_search,
            corpus_rows,
            query_count,
            graph_recall_at_10,
            graph_recall_at_100,
            ndcg_at_10,
            mean_abs_score_error,
            spearman_rho_at_10,
            exact_quantized_recall_at_10,
            graph_below_exact_queries,
            worst_exact_gap,
        ) = summary;

        println!(
            "external smoke 500: m={m} ef={ef_search} corpus={corpus_rows} queries={query_count} \
             graph@10={graph_recall_at_10:.4} graph@100={graph_recall_at_100:.4} \
             ndcg@10={ndcg_at_10:.4} mae={mean_abs_score_error:.6} \
             spearman={spearman_rho_at_10:.4} exact@10={exact_quantized_recall_at_10:.4} \
             graph_below_exact={graph_below_exact_queries} worst_gap={worst_exact_gap}"
        );

        assert_eq!(m, 8);
        assert_eq!(ef_search, 128);
        assert_eq!(corpus_rows, 500);
        assert_eq!(query_count, 25);
        // The smoke fixture is uniformly random in 1536 dimensions; tqvector
        // recall is dominated by the quantizer noise floor in this regime.
        // We don't assert a specific recall — just that the path returns a
        // sane fraction in [0, 1] and doesn't blow up. The real recall gate
        // is `tqhnsw_graph_scan_recall_external_gate_report` against the
        // staged DBpedia corpus.
        assert!((0.0..=1.0).contains(&graph_recall_at_10));
        assert!((0.0..=1.0).contains(&graph_recall_at_100));
        assert!((0.0..=1.0).contains(&exact_quantized_recall_at_10));
        assert!((-1.0..=1.0).contains(&spearman_rho_at_10));
        assert!(ndcg_at_10 >= 0.0);
        assert!(mean_abs_score_error >= 0.0);

        // Reusability: rerunning against the same loaded tables and index
        // produces an identical row.
        let summary_two = probe_graph_scan_recall_external_summary_for_relation(
            &corpus_table,
            &queries_table,
            &m8_index,
            8,
            128,
        );
        assert_eq!(
            summary, summary_two,
            "external recall summary should be deterministic across reruns"
        );

        // Gate report wrapper: covers all four NFR-003 A4 configurations
        // against the m=8 and m=16 indexes built by the smoke fixture.
        let gate = run_graph_scan_recall_gate_from_external(&corpus_table, &queries_table, prefix);
        assert_eq!(
            gate.len(),
            RECALL_GATE_CONFIGS.len(),
            "gate report should emit one row per A4 config"
        );
        for ((m, ef_search, recall, target, passed), expected) in
            gate.iter().zip(RECALL_GATE_CONFIGS.iter())
        {
            assert_eq!(*m, expected.0);
            assert_eq!(*ef_search, expected.1);
            assert_eq!(*target, expected.2);
            assert!((0.0..=1.0).contains(recall));
            // Targetless rows always pass; gated rows are only asserted to
            // be deterministic, not to clear the gate on synthetic data.
            if expected.2.is_none() {
                assert!(*passed);
            }
        }
    }

    #[cfg(test)]
    #[test]
    #[ignore]
    fn test_hnsw_rs_code_graph_recall_uniform_10k() {
        let corpus = random_unit_vectors(RECALL_CORPUS_SIZE, RECALL_DIM, RECALL_SEED as u64);
        let queries = random_unit_vectors(20, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ef_search = 128_usize;

        let (hnsw_recall_at_10, build_code_recall_at_10, exact_quantized_recall_at_10) =
            probe_hnsw_rs_code_graph_recall(&corpus, &queries, 8, ef_search);
        println!(
            "hnsw-rs code graph probe: queries={} m=8 ef_search={ef_search} hnsw={hnsw_recall_at_10:.4} build_code={build_code_recall_at_10:.4} exact={exact_quantized_recall_at_10:.4}",
            queries.len()
        );
    }

    #[cfg(test)]
    #[test]
    #[ignore]
    fn test_hnsw_rs_source_graph_recall_uniform_10k() {
        let corpus = random_unit_vectors(RECALL_CORPUS_SIZE, RECALL_DIM, RECALL_SEED as u64);
        let queries = random_unit_vectors(20, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ef_search = 128_usize;

        let hnsw_recall_at_10 = probe_hnsw_rs_source_graph_recall(&corpus, &queries, 8, ef_search);
        println!(
            "hnsw-rs source graph probe: queries={} m=8 ef_search={ef_search} hnsw={hnsw_recall_at_10:.4}",
            queries.len()
        );
    }

    #[cfg(test)]
    #[test]
    #[ignore]
    fn test_hnsw_rs_source_graph_recall_clustered_10k() {
        let corpus =
            random_clustered_vectors(RECALL_CORPUS_SIZE, RECALL_DIM, 50, 0.3, RECALL_SEED as u64);
        let queries =
            random_clustered_vectors(20, RECALL_DIM, 50, 0.3, (RECALL_SEED as u64) + 500_000);
        let ef_search = 128_usize;

        let hnsw_recall_at_10 = probe_hnsw_rs_source_graph_recall(&corpus, &queries, 8, ef_search);
        println!(
            "hnsw-rs source graph clustered probe: queries={} m=8 ef_search={ef_search} hnsw={hnsw_recall_at_10:.4}",
            queries.len()
        );
    }

    #[cfg(test)]
    #[test]
    #[ignore]
    fn test_hnsw_rs_source_graph_recall_uniform_10k_m16_ef200() {
        let corpus = random_unit_vectors(RECALL_CORPUS_SIZE, RECALL_DIM, RECALL_SEED as u64);
        let queries = random_unit_vectors(20, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ef_search = 200_usize;

        let hnsw_recall_at_10 = probe_hnsw_rs_source_graph_recall(&corpus, &queries, 16, ef_search);
        println!(
            "hnsw-rs source graph probe: queries={} m=16 ef_search={ef_search} hnsw={hnsw_recall_at_10:.4}",
            queries.len()
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
