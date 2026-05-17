use pgrx::extension_sql_file;
use pgrx::ffi::CString;
use pgrx::prelude::*;
use pgrx::{pg_sys, Internal};

const MODULE_VERSION_CSTR: &core::ffi::CStr = {
    const RAW: &str = env!("CARGO_PKG_VERSION");
    const BUFFER: [u8; RAW.len() + 1] = {
        let mut buffer = [0u8; RAW.len() + 1];
        let mut i = 0;
        while i < RAW.len() {
            buffer[i] = RAW.as_bytes()[i];
            i += 1;
        }
        buffer
    };
    if let Ok(value) = core::ffi::CStr::from_bytes_with_nul(&BUFFER) {
        value
    } else {
        panic!("CARGO_PKG_VERSION contains an interior NUL byte")
    }
};

// Use explicit fields so PG18 module metadata reports the ecaz name/version correctly.
pgrx::pg_module_magic!(name = c"ecaz", version = MODULE_VERSION_CSTR);

#[allow(dead_code)]
mod am;
#[cfg(feature = "pg18")]
mod pg18_pgstat_shim;
mod quant;
#[cfg(all(test, target_arch = "x86_64", target_os = "linux"))]
mod standalone_pg_backend_stubs;
pub(crate) mod storage;

use quant::prod::{payload_len, ProdQuantizer};

#[pg_guard]
pub unsafe extern "C-unwind" fn _PG_init() {
    am::register_gucs();
    unsafe {
        am::register_custom_scan();
        am::register_dml_frontdoor_planner_hook();
    }
    #[cfg(feature = "pg18")]
    unsafe {
        am::explain::register_pg18_explain_hooks();
        am::stats::register_pg18_stats();
    }
}

/// Public API surface for benchmarks and integration tests.
/// This stays narrow and explicit so benchmark and integration code can reuse
/// storage/quantizer helpers without reaching through internal modules directly.
pub mod bench_api {
    // Quantizer core
    pub use crate::quant::prod::{
        mse_code_len, pack_mse_indices, pack_qjl_signs, payload_len, qjl_code_len,
        unpack_mse_indices, unpack_qjl_signs, EncodedTq, Int8ApproxNoQjl4BitQuery,
        PreparedLutNoQjl4BitQuery, PreparedQuery, PreparedTiledLutNoQjl4BitQuery, ProdQuantizer,
    };

    // Hadamard
    #[cfg(feature = "bench")]
    pub use crate::quant::hadamard::fwht_in_place_scalar_reference;
    pub use crate::quant::hadamard::{fwht_in_place, orthonormal_fwht_in_place};
    pub fn simd_backend() -> &'static str {
        crate::quant::simd_backend_name()
    }

    // Rotation
    pub use crate::quant::rotation::{
        effective_transform_dim, inverse_srht, pad_input, sign_vector, srht, transform_dim,
    };

    // Codebook
    pub use crate::am::common::training::{
        derive_grouped_pq4_code, train_grouped_pq4_model, GroupedPq4Model,
    };
    pub use crate::quant::codebook::{beta_pdf, lloyd_max};
    pub use crate::quant::grouped_pq::{
        build_grouped_pq_lut_f32, encode_grouped_pq, grouped_pq_nibble, grouped_pq_score_f32,
        nearest_centroid_l2, pack_grouped_pq_nibbles, GROUPED_PQ_CENTROIDS,
    };

    // MSE
    pub use crate::quant::mse::{decode_indices, nearest_centroid_index, quantize_to_indices};

    // QJL
    pub use crate::quant::qjl::{decode_mse_only, qjl_project};

    // RaBitQ (ADR-045 Stage 1)
    pub use crate::quant::rabitq::{
        derive_persisted_sidecar_words, persisted_sidecar_word_count, CenterContext,
        CenteredScorer, DistanceEstimate, PreparedEstimator, RaBitQQuantizer, RaBitQScorer,
        Rotation, SrhtRotation, RABITQ_BOUND_CONFIDENCE, RABITQ_NORM_LEN, RABITQ_SCALAR_LEN,
        RABITQ_SUPPORTED_BITS, RABITQ_UNIT_DOT_LEN, RABITQ_XNORM_LEN,
    };
    pub use crate::quant::{Quantizer, QueryScorer};

    // Page codec
    pub use crate::am::page::{
        neighbor_slots, neighbor_tuple_encoded_len, CurrentFormatMetadata, MetadataPage,
        TqElementTuple, TqNeighborTuple,
    };
    pub use crate::am::{
        approximate_medoid, bfs_reachable, build_vamana_graph_with_pass1_extra_candidates,
        build_vamana_graph_with_stats, greedy_search, MetricSummary, VamanaBuildPassStats,
        VamanaBuildStats, VamanaGraph, VamanaMetadataPage, INDEX_FORMAT_V3_DISKANN,
        VAMANA_METADATA_BYTES,
    };
    pub use crate::storage::page::{
        DataPage, DataPageChain, ItemPointer, HEAPTID_INLINE_CAPACITY, ITEM_POINTER_BYTES,
        PAGE_HEADER_BYTES,
    };

    // Text I/O
    pub use crate::{format_text, parse_text, HEADER_BYTES, MIN_BINARY_BYTES};
}

extension_sql_file!("../sql/bootstrap.sql", name = "bootstrap", bootstrap);

/// Number of per-datum descriptor bytes: dim(2).
pub const HEADER_BYTES: usize = 2;
/// Minimum valid wire payload: descriptor plus gamma.
pub const MIN_BINARY_BYTES: usize = HEADER_BYTES + 4;
const ECVECTOR_MAX_DIM: usize = u16::MAX as usize;
pub(crate) const DEFAULT_QUANT_BITS: u8 = 4;
pub(crate) const DEFAULT_QUANT_SEED: u64 = 42;

fn validate_bits(bits: u8) -> Result<(), String> {
    if !(2..=8).contains(&bits) {
        return Err(format!("bits must be between 2 and 8, got {bits}"));
    }
    Ok(())
}

fn validate_tqvector_seed(seed: u64) -> Result<(), String> {
    if seed != DEFAULT_QUANT_SEED {
        return Err(format!(
            "tqvector seed must use the canonical default ({DEFAULT_QUANT_SEED}), got {seed}"
        ));
    }
    Ok(())
}

fn validate_tqvector_bits(bits: u8) -> Result<(), String> {
    if bits != DEFAULT_QUANT_BITS {
        return Err(format!(
            "tqvector bits must use the canonical default ({DEFAULT_QUANT_BITS}), got {bits}"
        ));
    }
    Ok(())
}

fn validate_ecvector_dim(dim: usize, typmod: i32, label: &str) -> Result<(), String> {
    if dim > ECVECTOR_MAX_DIM {
        return Err(format!(
            "{label} cannot have more than {ECVECTOR_MAX_DIM} dimensions"
        ));
    }
    if typmod >= 0 && dim != typmod as usize {
        return Err(format!(
            "{label} dimension mismatch: expected {typmod}, got {dim}"
        ));
    }
    Ok(())
}

fn code_len(dim: usize, bits: u8) -> usize {
    payload_len(dim, bits) - 4
}

pub(crate) fn quantize_embedding_to_code(
    embedding: &[f32],
    bits: u8,
    seed: u64,
) -> Result<(u16, f32, Vec<u8>), String> {
    if embedding.is_empty() {
        return Err("embedding must not be empty".into());
    }
    validate_bits(bits)?;
    let dim = u16::try_from(embedding.len()).map_err(|_| {
        format!(
            "embedding dimension {} exceeds maximum 65535",
            embedding.len()
        )
    })?;

    let quantizer = ProdQuantizer::cached(embedding.len(), bits, seed);
    let encoded = quantizer.encode(embedding);

    let mut code_bytes = encoded.mse_packed;
    code_bytes.extend_from_slice(&encoded.qjl_packed);

    Ok((dim, encoded.gamma, code_bytes))
}

fn ec_hnsw_access_method_oid() -> pg_sys::Oid {
    Spi::get_one::<pg_sys::Oid>("SELECT oid FROM pg_am WHERE amname = 'ec_hnsw'")
        .expect("SPI query should succeed")
        .expect("ec_hnsw access method should exist")
}

fn ec_ivf_access_method_oid() -> pg_sys::Oid {
    Spi::get_one::<pg_sys::Oid>("SELECT oid FROM pg_am WHERE amname = 'ec_ivf'")
        .expect("SPI query should succeed")
        .expect("ec_ivf access method should exist")
}

fn ec_spire_access_method_oid() -> pg_sys::Oid {
    Spi::get_one::<pg_sys::Oid>("SELECT oid FROM pg_am WHERE amname = 'ec_spire'")
        .expect("SPI query should succeed")
        .expect("ec_spire access method should exist")
}

fn ec_diskann_access_method_oid() -> pg_sys::Oid {
    Spi::get_one::<pg_sys::Oid>("SELECT oid FROM pg_am WHERE amname = 'ec_diskann'")
        .expect("SPI query should succeed")
        .expect("ec_diskann access method should exist")
}

unsafe fn open_valid_ec_hnsw_index(
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
    if rd_rel.relam != ec_hnsw_access_method_oid() {
        let relation_name = unsafe { std::ffi::CStr::from_ptr(rd_rel.relname.data.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        pgrx::error!("{caller_name} requires a ec_hnsw index, got relation \"{relation_name}\"");
    }
    index_relation
}

unsafe fn open_valid_ec_ivf_index(
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
    if rd_rel.relam != ec_ivf_access_method_oid() {
        let relation_name = unsafe { std::ffi::CStr::from_ptr(rd_rel.relname.data.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        pgrx::error!("{caller_name} requires a ec_ivf index, got relation \"{relation_name}\"");
    }
    index_relation
}

unsafe fn open_valid_ec_spire_index(
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
    if rd_rel.relam != ec_spire_access_method_oid() {
        let relation_name = unsafe { std::ffi::CStr::from_ptr(rd_rel.relname.data.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        pgrx::error!("{caller_name} requires a ec_spire index, got relation \"{relation_name}\"");
    }
    index_relation
}

unsafe fn relation_oid_exists(relation_oid: pg_sys::Oid) -> bool {
    relation_oid != pg_sys::InvalidOid && unsafe { pg_sys::get_rel_relkind(relation_oid) } != 0
}

unsafe fn open_valid_ec_diskann_index(
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
    if rd_rel.relam != ec_diskann_access_method_oid() {
        let relation_name = unsafe { std::ffi::CStr::from_ptr(rd_rel.relname.data.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        pgrx::error!("{caller_name} requires a ec_diskann index, got relation \"{relation_name}\"");
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
    validate_tqvector_bits(bits)?;
    validate_tqvector_seed(seed)?;

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
    validate_bits(bits).expect("tqvector pack must only be called with valid bits");
    validate_tqvector_bits(bits).expect("tqvector pack must only be called with canonical bits");
    validate_tqvector_seed(seed).expect("tqvector pack must only be called with canonical seed");
    let mut buf = Vec::with_capacity(MIN_BINARY_BYTES + codes.len());
    buf.extend_from_slice(&dim.to_le_bytes());
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
    let bits = DEFAULT_QUANT_BITS;
    let seed = DEFAULT_QUANT_SEED;
    let gamma = f32::from_le_bytes(data[2..6].try_into().expect("gamma bytes"));
    let codes = &data[6..];
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

fn pack_raw_f32(values: &[f32], label: &str) -> Result<Vec<u8>, String> {
    validate_ecvector_dim(values.len(), -1, label)?;
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(values));
    for (index, value) in values.iter().copied().enumerate() {
        if !value.is_finite() {
            return Err(format!(
                "{label} element {} must be finite, got {value}",
                index + 1
            ));
        }
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    Ok(bytes)
}

fn unpack_raw_f32(bytes: &[u8], label: &str) -> Result<Vec<f32>, String> {
    if bytes.len() % std::mem::size_of::<f32>() != 0 {
        return Err(format!("{label} length must be a multiple of 4 bytes"));
    }
    validate_ecvector_dim(bytes.len() / std::mem::size_of::<f32>(), -1, label)?;

    let mut values = Vec::with_capacity(bytes.len() / std::mem::size_of::<f32>());
    for (index, chunk) in bytes.chunks_exact(std::mem::size_of::<f32>()).enumerate() {
        let value = f32::from_le_bytes(chunk.try_into().expect("validated f32 chunk"));
        if !value.is_finite() {
            return Err(format!(
                "{label} element {} must be finite, got {value}",
                index + 1
            ));
        }
        values.push(value);
    }
    Ok(values)
}

fn parse_raw_f32_text(input: &str, label: &str) -> Result<Vec<f32>, String> {
    let input = input.trim();
    if !input.starts_with('[') {
        return Err(format!("{label} is missing '['"));
    }
    if !input.ends_with(']') {
        return Err(format!("{label} is missing ']'"));
    }

    let body = input[1..input.len() - 1].trim();
    if body.is_empty() {
        return Ok(Vec::new());
    }

    body.split(',')
        .enumerate()
        .map(|(index, raw)| {
            let value = raw
                .trim()
                .parse::<f32>()
                .map_err(|e| format!("{label} element {} is invalid: {e}", index + 1))?;
            if !value.is_finite() {
                return Err(format!(
                    "{label} element {} must be finite, got {value}",
                    index + 1
                ));
            }
            Ok(value)
        })
        .collect()
}

fn format_raw_f32_text(values: &[f32]) -> String {
    let body = values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("[{body}]")
}

fn format_u64_array_text(values: &[u64]) -> String {
    let body = values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("[{body}]")
}

fn raw_inner_product(left: &[f32], right: &[f32], label: &str) -> Result<f32, String> {
    if left.len() != right.len() {
        return Err(format!(
            "{label} dimension mismatch: left dim {}, right dim {}",
            left.len(),
            right.len()
        ));
    }
    Ok(left.iter().zip(right).map(|(l, r)| l * r).sum())
}

unsafe fn recv_raw_f32_message(msg: pg_sys::StringInfo, label: &str) -> Result<Vec<u8>, String> {
    if msg.is_null() {
        return Err(format!("{label}: missing input buffer"));
    }

    let total_len = usize::try_from(unsafe { (*msg).len })
        .map_err(|_| format!("{label}: invalid binary length"))?;
    let cursor = usize::try_from(unsafe { (*msg).cursor })
        .map_err(|_| format!("{label}: invalid binary cursor"))?;
    if cursor > total_len {
        return Err(format!("{label}: invalid binary cursor state"));
    }

    let remaining = total_len - cursor;
    let mut bytes = Vec::with_capacity(remaining);
    if remaining > 0 {
        let payload = unsafe { pg_sys::pq_getmsgbytes(msg, remaining as i32) as *const u8 };
        let payload = unsafe { std::slice::from_raw_parts(payload, remaining) };
        bytes.extend_from_slice(payload);
    }

    unsafe { pg_sys::pq_getmsgend(msg) };
    unpack_raw_f32(&bytes, label)?;
    Ok(bytes)
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn ecvector_in(input: &core::ffi::CStr, _type_oid: pg_sys::Oid, typmod: i32) -> Vec<u8> {
    let input = input
        .to_str()
        .unwrap_or_else(|_| pgrx::error!("invalid UTF-8 in ecvector input"));
    let values = parse_raw_f32_text(input, "ecvector")
        .unwrap_or_else(|e| pgrx::error!("invalid ecvector: {e}"));
    validate_ecvector_dim(values.len(), typmod, "ecvector")
        .unwrap_or_else(|e| pgrx::error!("invalid ecvector: {e}"));
    pack_raw_f32(&values, "ecvector").unwrap_or_else(|e| pgrx::error!("invalid ecvector: {e}"))
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn ecvector_out(vec: Vec<u8>) -> CString {
    let values =
        unpack_raw_f32(&vec, "ecvector").unwrap_or_else(|e| pgrx::error!("corrupt ecvector: {e}"));
    CString::new(format_raw_f32_text(&values)).expect("cstring without NUL")
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn ecvector_send(vec: Vec<u8>) -> Vec<u8> {
    unpack_raw_f32(&vec, "ecvector")
        .unwrap_or_else(|e| pgrx::error!("invalid ecvector binary: {e}"));
    vec
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn ecvector_recv(input: Internal, _type_oid: pg_sys::Oid, typmod: i32) -> Vec<u8> {
    let msg = unsafe {
        input
            .get::<pg_sys::StringInfoData>()
            .unwrap_or_else(|| pgrx::error!("invalid ecvector binary: missing input buffer"))
            as *const pg_sys::StringInfoData as pg_sys::StringInfo
    };

    let bytes = unsafe { recv_raw_f32_message(msg, "ecvector") }
        .unwrap_or_else(|e| pgrx::error!("invalid ecvector binary: {e}"));
    validate_ecvector_dim(bytes.len() / std::mem::size_of::<f32>(), typmod, "ecvector")
        .unwrap_or_else(|e| pgrx::error!("invalid ecvector binary: {e}"));
    bytes
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn ecvector_from_real_array(values: Vec<f32>, typmod: i32, _explicit: bool) -> Vec<u8> {
    validate_ecvector_dim(values.len(), typmod, "ecvector")
        .unwrap_or_else(|e| pgrx::error!("invalid ecvector: {e}"));
    pack_raw_f32(&values, "ecvector").unwrap_or_else(|e| pgrx::error!("invalid ecvector: {e}"))
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn ecvector_to_real_array(vec: Vec<u8>, _typmod: i32, _explicit: bool) -> Vec<f32> {
    unpack_raw_f32(&vec, "ecvector").unwrap_or_else(|e| pgrx::error!("corrupt ecvector: {e}"))
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn ecvector_from_bytea(bytes: Vec<u8>, typmod: i32, _explicit: bool) -> Vec<u8> {
    unpack_raw_f32(&bytes, "ecvector").unwrap_or_else(|e| pgrx::error!("invalid ecvector: {e}"));
    validate_ecvector_dim(bytes.len() / std::mem::size_of::<f32>(), typmod, "ecvector")
        .unwrap_or_else(|e| pgrx::error!("invalid ecvector: {e}"));
    bytes
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn ecvector_to_bytea(vec: Vec<u8>, _typmod: i32, _explicit: bool) -> Vec<u8> {
    unpack_raw_f32(&vec, "ecvector").unwrap_or_else(|e| pgrx::error!("corrupt ecvector: {e}"));
    vec
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn ecvector_coerce(vec: Vec<u8>, typmod: i32, _explicit: bool) -> Vec<u8> {
    unpack_raw_f32(&vec, "ecvector").unwrap_or_else(|e| pgrx::error!("corrupt ecvector: {e}"));
    validate_ecvector_dim(vec.len() / std::mem::size_of::<f32>(), typmod, "ecvector")
        .unwrap_or_else(|e| pgrx::error!("invalid ecvector: {e}"));
    vec
}

#[pg_guard]
#[no_mangle]
pub unsafe extern "C-unwind" fn ecvector_typmod_in(
    fcinfo: pg_sys::FunctionCallInfo,
) -> pg_sys::Datum {
    pgrx::pgrx_extern_c_guard(|| unsafe {
        let datum = pgrx::fcinfo::pg_getarg_datum_raw(fcinfo, 0);
        let original = datum
            .cast_mut_ptr::<std::ffi::c_void>()
            .cast::<pg_sys::ArrayType>();
        let array = pg_sys::pg_detoast_datum_packed(original.cast()).cast::<pg_sys::ArrayType>();
        let is_copy = !std::ptr::eq(array, original);
        let mut count = 0;
        let raw_typmods = pg_sys::ArrayGetIntegerTypmods(array, &mut count);
        let dim = if count == 1 {
            *raw_typmods
        } else {
            if is_copy {
                pg_sys::pfree(array.cast());
            }
            pgrx::error!("invalid type modifier");
        };
        if is_copy {
            pg_sys::pfree(array.cast());
        }
        if dim < 1 {
            pgrx::error!("dimensions for type ecvector must be at least 1");
        }
        if dim as usize > ECVECTOR_MAX_DIM {
            pgrx::error!(
                "dimensions for type ecvector cannot exceed {}",
                ECVECTOR_MAX_DIM
            );
        }
        dim.into_datum()
            .expect("typmod integer should convert to datum")
    })
}

#[no_mangle]
pub extern "C-unwind" fn pg_finfo_ecvector_typmod_in() -> *const pg_sys::Pg_finfo_record {
    static API_V1: pg_sys::Pg_finfo_record = pg_sys::Pg_finfo_record { api_version: 1 };
    &API_V1
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
fn ec_hnsw_index_admin_snapshot(
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
        unsafe { open_valid_ec_hnsw_index(index_oid, "ec_hnsw_index_admin_snapshot") };
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
fn ec_diskann_index_graph_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<'static, (name!(metric, String), name!(value, String))> {
    let index_relation =
        unsafe { open_valid_ec_diskann_index(index_oid, "ec_diskann_index_graph_summary") };
    let summary = match unsafe { am::diskann_graph_summary(index_relation) } {
        Ok(summary) => summary,
        Err(e) => {
            unsafe {
                pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
            }
            pgrx::error!("ec_diskann_index_graph_summary failed: {e}");
        }
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let rows = vec![
        ("block_count".to_owned(), summary.block_count.to_string()),
        (
            "graph_degree_r".to_owned(),
            summary.graph_degree_r.to_string(),
        ),
        (
            "build_list_size_l".to_owned(),
            summary.build_list_size_l.to_string(),
        ),
        ("alpha".to_owned(), format!("{:.6}", summary.alpha)),
        ("dimensions".to_owned(), summary.dimensions.to_string()),
        (
            "inserted_since_rebuild".to_owned(),
            summary.inserted_since_rebuild.to_string(),
        ),
        (
            "needs_medoid_refresh".to_owned(),
            summary.needs_medoid_refresh.to_string(),
        ),
        ("node_count".to_owned(), summary.node_count.to_string()),
        (
            "live_node_count".to_owned(),
            summary.live_node_count.to_string(),
        ),
        (
            "non_live_node_count".to_owned(),
            summary.non_live_node_count.to_string(),
        ),
        (
            "entry_point_live".to_owned(),
            summary.entry_point_live.to_string(),
        ),
        (
            "reachable_live_node_count".to_owned(),
            summary.reachable_live_node_count.to_string(),
        ),
        (
            "unreachable_live_node_count".to_owned(),
            summary.unreachable_live_node_count.to_string(),
        ),
        (
            "reachable_live_fraction".to_owned(),
            format!("{:.6}", summary.reachable_live_fraction),
        ),
        (
            "neighbor_ref_count".to_owned(),
            summary.neighbor_ref_count.to_string(),
        ),
        (
            "live_neighbor_ref_count".to_owned(),
            summary.live_neighbor_ref_count.to_string(),
        ),
        (
            "dead_neighbor_ref_count".to_owned(),
            summary.dead_neighbor_ref_count.to_string(),
        ),
        (
            "invalid_neighbor_ref_count".to_owned(),
            summary.invalid_neighbor_ref_count.to_string(),
        ),
        (
            "self_neighbor_ref_count".to_owned(),
            summary.self_neighbor_ref_count.to_string(),
        ),
        (
            "duplicate_neighbor_ref_count".to_owned(),
            summary.duplicate_neighbor_ref_count.to_string(),
        ),
        (
            "unresolvable_neighbor_ref_count".to_owned(),
            summary.unresolvable_neighbor_ref_count.to_string(),
        ),
        (
            "zero_out_degree_count".to_owned(),
            summary.zero_out_degree_count.to_string(),
        ),
        (
            "min_out_degree".to_owned(),
            summary.min_out_degree.to_string(),
        ),
        (
            "avg_out_degree".to_owned(),
            format!("{:.6}", summary.avg_out_degree),
        ),
        (
            "p50_out_degree".to_owned(),
            summary.p50_out_degree.to_string(),
        ),
        (
            "p95_out_degree".to_owned(),
            summary.p95_out_degree.to_string(),
        ),
        (
            "p99_out_degree".to_owned(),
            summary.p99_out_degree.to_string(),
        ),
        (
            "max_out_degree".to_owned(),
            summary.max_out_degree.to_string(),
        ),
        (
            "zero_in_degree_count".to_owned(),
            summary.zero_in_degree_count.to_string(),
        ),
        (
            "min_in_degree".to_owned(),
            summary.min_in_degree.to_string(),
        ),
        (
            "avg_in_degree".to_owned(),
            format!("{:.6}", summary.avg_in_degree),
        ),
        (
            "p50_in_degree".to_owned(),
            summary.p50_in_degree.to_string(),
        ),
        (
            "p95_in_degree".to_owned(),
            summary.p95_in_degree.to_string(),
        ),
        (
            "p99_in_degree".to_owned(),
            summary.p99_in_degree.to_string(),
        ),
        (
            "max_in_degree".to_owned(),
            summary.max_in_degree.to_string(),
        ),
    ];
    TableIterator::new(rows)
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_custom_scan_status() -> TableIterator<
    'static,
    (
        name!(provider_name, &'static str),
        name!(registered, bool),
        name!(rel_pathlist_hook_installed, bool),
        name!(path_generation_enabled, bool),
        name!(exec_wiring_enabled, bool),
        name!(status, &'static str),
        name!(next_step, &'static str),
    ),
> {
    let row = am::spire_custom_scan_status_row();
    TableIterator::once((
        row.provider_name,
        row.registered,
        row.rel_pathlist_hook_installed,
        row.path_generation_enabled,
        row.exec_wiring_enabled,
        row.status,
        row.next_step,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_dml_frontdoor_hook_status() -> TableIterator<
    'static,
    (
        name!(hook_name, &'static str),
        name!(planner_hook_installed, bool),
        name!(query_shape_classifier_enabled, bool),
        name!(query_shape_classifier_invoked_by_hook, bool),
        name!(unsupported_shape_fail_closed_enabled, bool),
        name!(plan_rewrite_enabled, bool),
        name!(last_classification_supported, Option<bool>),
        name!(last_classification_kind, Option<&'static str>),
        name!(last_classification_status, Option<&'static str>),
        name!(last_hook_action, Option<&'static str>),
        name!(status, &'static str),
        name!(next_step, &'static str),
    ),
> {
    let row = am::spire_dml_frontdoor_hook_status_row();
    TableIterator::once((
        row.hook_name,
        row.planner_hook_installed,
        row.query_shape_classifier_enabled,
        row.query_shape_classifier_invoked_by_hook,
        row.unsupported_shape_fail_closed_enabled,
        row.plan_rewrite_enabled,
        row.last_classification_supported,
        row.last_classification_kind,
        row.last_classification_status,
        row.last_hook_action,
        row.status,
        row.next_step,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_dml_frontdoor_relation_context(
    heap_relation_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(heap_relation_oid, pg_sys::Oid),
        name!(index_oid, pg_sys::Oid),
        name!(ec_spire_distributed_table, bool),
        name!(pk_column, Option<String>),
        name!(pk_type, Option<String>),
        name!(ordinary_column_count, i64),
        name!(embedding_columns, Vec<String>),
        name!(status, &'static str),
        name!(next_step, &'static str),
    ),
> {
    let row = am::spire_dml_frontdoor_relation_context_row(heap_relation_oid)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    TableIterator::once((
        row.heap_relation_oid,
        row.index_oid,
        row.ec_spire_distributed_table,
        row.pk_column,
        row.pk_type,
        i64::try_from(row.column_names.len()).expect("column count should fit i64"),
        row.embedding_columns,
        row.status,
        row.next_step,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_dml_frontdoor_relation_context_catalog(
    heap_relation_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(heap_relation_oid, pg_sys::Oid),
        name!(index_oid, pg_sys::Oid),
        name!(ec_spire_distributed_table, bool),
        name!(pk_column, Option<String>),
        name!(pk_type, Option<String>),
        name!(ordinary_column_count, i64),
        name!(embedding_columns, Vec<String>),
        name!(status, &'static str),
        name!(next_step, &'static str),
    ),
> {
    let row = unsafe { am::spire_dml_frontdoor_relation_context_catalog_row(heap_relation_oid) }
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    TableIterator::once((
        row.heap_relation_oid,
        row.index_oid,
        row.ec_spire_distributed_table,
        row.pk_column,
        row.pk_type,
        i64::try_from(row.column_names.len()).expect("column count should fit i64"),
        row.embedding_columns,
        row.status,
        row.next_step,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_dml_frontdoor_relation_context_cache() -> TableIterator<
    'static,
    (
        name!(relcache_callback_registered, bool),
        name!(entry_count, i64),
        name!(hit_count, i64),
        name!(miss_count, i64),
        name!(invalidation_count, i64),
        name!(status, &'static str),
    ),
> {
    let row = am::spire_dml_frontdoor_relation_context_cache_row();
    TableIterator::once((
        row.relcache_callback_registered,
        row.entry_count,
        row.hit_count,
        row.miss_count,
        row.invalidation_count,
        row.status,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_dml_frontdoor_classify_sql(
    sql: &str,
) -> TableIterator<
    'static,
    (
        name!(target_relation_oid, Option<pg_sys::Oid>),
        name!(relation_status, Option<&'static str>),
        name!(supported, bool),
        name!(operation, &'static str),
        name!(kind, &'static str),
        name!(status, &'static str),
        name!(error, Option<&'static str>),
        name!(hint, Option<&'static str>),
        name!(next_step, &'static str),
    ),
> {
    let query = unsafe {
        analyze_single_dml_frontdoor_query(sql)
            .unwrap_or_else(|e| pgrx::error!("ec_spire DML frontdoor SQL analysis failed: {e}"))
    };
    let Some(target_relation_oid) = (unsafe { am::spire_dml_frontdoor_target_relation_oid(query) })
    else {
        return TableIterator::once((
            None,
            None,
            false,
            "unsupported",
            "unsupported_target_relation",
            "unsupported_shape",
            Some("ec_spire_distributed: DML front door requires one target heap relation"),
            Some("See ADR-069 for the v1 SPIRE distributed DML shape."),
            "rewrite query as single-table UPDATE, DELETE, or PK SELECT",
        ));
    };
    let relation = am::spire_dml_frontdoor_relation_context_row(target_relation_oid)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    let pk_column = relation.pk_column.as_deref().unwrap_or("");
    let column_names = relation
        .column_names
        .iter()
        .map(|(attnum, name)| (*attnum, name.as_str()))
        .collect::<Vec<_>>();
    let embedding_columns = relation
        .embedding_columns
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let query_context = am::SpireDmlFrontdoorQueryContext {
        ec_spire_distributed_table: relation.ec_spire_distributed_table,
        pk_column,
        column_names: &column_names,
        embedding_columns: &embedding_columns,
    };
    let Some(shape) = (unsafe { am::spire_classify_dml_frontdoor_query(query, query_context) })
    else {
        return TableIterator::once((
            Some(target_relation_oid),
            Some(relation.status),
            false,
            "unsupported",
            "unsupported_operation",
            "unsupported_shape",
            Some("ec_spire_distributed: only UPDATE, DELETE, and PK SELECT are supported in v1"),
            Some("See ADR-069 for the v1 SPIRE distributed DML shape."),
            "rewrite query as single-table UPDATE, DELETE, or PK SELECT",
        ));
    };

    TableIterator::once((
        Some(target_relation_oid),
        Some(relation.status),
        shape.supported,
        shape.operation,
        shape.kind,
        shape.status,
        shape.error,
        shape.hint,
        if shape.supported {
            "wire planner hook to CustomScan executor replacement"
        } else {
            "rewrite query to fit the ADR-069 v1 DML front-door shape"
        },
    ))
}

#[pg_extern(stable)]
#[allow(clippy::type_complexity)]
fn ec_spire_dml_frontdoor_replacement_sql(
    sql: &str,
) -> TableIterator<
    'static,
    (
        name!(target_relation_oid, Option<pg_sys::Oid>),
        name!(index_oid, Option<pg_sys::Oid>),
        name!(supported, bool),
        name!(operation, &'static str),
        name!(kind, &'static str),
        name!(status, &'static str),
        name!(custom_scan_mode, &'static str),
        name!(primitive, &'static str),
        name!(pk_column, Option<String>),
        name!(pk_value_kind, &'static str),
        name!(pk_value_const, Option<i64>),
        name!(pk_value_param_id, Option<i32>),
        name!(updated_columns, Vec<String>),
        name!(projected_columns, Vec<String>),
        name!(error, Option<&'static str>),
        name!(hint, Option<&'static str>),
        name!(next_step, &'static str),
    ),
> {
    let query = unsafe {
        analyze_single_dml_frontdoor_query(sql)
            .unwrap_or_else(|e| pgrx::error!("ec_spire DML frontdoor SQL analysis failed: {e}"))
    };
    let Some(decision) =
        (unsafe { am::spire_dml_frontdoor_replacement_decision_catalog_row(query) })
    else {
        return TableIterator::once((
            None,
            None,
            false,
            "unsupported",
            "unsupported_target_relation",
            "unsupported_shape",
            "none",
            "none",
            None,
            "other",
            None,
            None,
            Vec::new(),
            Vec::new(),
            Some("ec_spire_distributed: DML front door requires one target heap relation"),
            Some("See ADR-069 for the v1 SPIRE distributed DML shape."),
            "raise ADR-069 planner error instead of using coordinator heap path",
        ));
    };

    TableIterator::once((
        Some(decision.target_relation_oid),
        if decision.index_oid == pg_sys::InvalidOid {
            None
        } else {
            Some(decision.index_oid)
        },
        decision.supported,
        decision.operation,
        decision.kind,
        decision.status,
        decision.custom_scan_mode,
        decision.primitive,
        decision.pk_column,
        decision.pk_value_kind,
        decision.pk_value_const,
        decision.pk_value_param_id,
        decision.updated_columns,
        decision.projected_columns,
        decision.error,
        decision.hint,
        decision.next_step,
    ))
}

#[pg_extern(stable)]
#[allow(clippy::type_complexity)]
fn ec_spire_dml_frontdoor_primitive_plan_sql(
    sql: &str,
) -> TableIterator<
    'static,
    (
        name!(target_relation_oid, Option<pg_sys::Oid>),
        name!(index_oid, Option<pg_sys::Oid>),
        name!(supported, bool),
        name!(custom_scan_mode, &'static str),
        name!(primitive, &'static str),
        name!(pk_column, Option<String>),
        name!(pk_value_kind, &'static str),
        name!(pk_value_const, Option<i64>),
        name!(pk_value_param_id, Option<i32>),
        name!(pk_value_bytes, Option<Vec<u8>>),
        name!(updated_columns, Vec<String>),
        name!(projected_columns, Vec<String>),
        name!(status, &'static str),
        name!(error, Option<String>),
        name!(next_step, &'static str),
    ),
> {
    let query = unsafe {
        analyze_single_dml_frontdoor_query(sql)
            .unwrap_or_else(|e| pgrx::error!("ec_spire DML frontdoor SQL analysis failed: {e}"))
    };
    let Some(decision) =
        (unsafe { am::spire_dml_frontdoor_replacement_decision_catalog_row(query) })
    else {
        return TableIterator::once((
            None,
            None,
            false,
            "none",
            "none",
            None,
            "other",
            None,
            None,
            None,
            Vec::new(),
            Vec::new(),
            "unsupported_shape",
            Some(
                "ec_spire_distributed: DML front door requires one target heap relation".to_owned(),
            ),
            "raise ADR-069 planner error instead of using coordinator heap path",
        ));
    };
    let target_relation_oid = Some(decision.target_relation_oid);
    let index_oid = if decision.index_oid == pg_sys::InvalidOid {
        None
    } else {
        Some(decision.index_oid)
    };
    let custom_scan_mode = decision.custom_scan_mode;
    let primitive = decision.primitive;
    let pk_column = decision.pk_column.clone();
    let pk_value_kind = decision.pk_value_kind;
    let pk_value_const = decision.pk_value_const;
    let pk_value_param_id = decision.pk_value_param_id;
    let updated_columns = decision.updated_columns.clone();
    let projected_columns = decision.projected_columns.clone();

    let primitive_plan =
        match am::spire_dml_frontdoor_primitive_plan_from_replacement_decision(&decision) {
            Ok(plan) => plan,
            Err(error) => {
                return TableIterator::once((
                    target_relation_oid,
                    index_oid,
                    false,
                    custom_scan_mode,
                    primitive,
                    pk_column,
                    pk_value_kind,
                    pk_value_const,
                    pk_value_param_id,
                    None,
                    updated_columns,
                    projected_columns,
                    "primitive_plan_not_ready",
                    Some(error),
                    "fix the DML replacement decision before building a CustomScan executor node",
                ));
            }
        };
    let (pk_value_bytes, status, error, next_step) =
        match am::spire_dml_frontdoor_primitive_plan_const_pk_value_bytes(&primitive_plan) {
            Ok(bytes) => (
                Some(bytes.to_vec()),
                "primitive_plan_ready",
                None,
                "wire planner hook to DML CustomScan executor replacement",
            ),
            Err(error) => (
                None,
                "primitive_plan_requires_runtime_params",
                Some(error),
                "evaluate bound parameters in the DML CustomScan executor before invoking the coordinator primitive",
            ),
        };

    TableIterator::once((
        target_relation_oid,
        index_oid,
        true,
        custom_scan_mode,
        primitive,
        pk_column,
        pk_value_kind,
        pk_value_const,
        pk_value_param_id,
        pk_value_bytes,
        updated_columns,
        projected_columns,
        status,
        error,
        next_step,
    ))
}

unsafe fn analyze_single_dml_frontdoor_query(sql: &str) -> Result<*mut pg_sys::Query, String> {
    let sql = CString::new(sql).map_err(|_| "SQL text contains an interior NUL byte".to_owned())?;
    let raw_parses = unsafe { pg_sys::pg_parse_query(sql.as_ptr()) };
    if raw_parses.is_null() {
        return Err("parser returned no statements".to_owned());
    }
    if unsafe { pg_sys::list_length(raw_parses) } != 1 {
        return Err("expected exactly one SQL statement".to_owned());
    }
    let raw_stmt = unsafe { pg_sys::list_nth(raw_parses, 0) }.cast::<pg_sys::RawStmt>();
    let queries = unsafe {
        pg_sys::pg_analyze_and_rewrite_fixedparams(
            raw_stmt,
            sql.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
        )
    };
    if queries.is_null() {
        return Err("analyzer returned no query".to_owned());
    }
    if unsafe { pg_sys::list_length(queries) } != 1 {
        return Err("expected exactly one analyzed query".to_owned());
    }
    Ok(unsafe { pg_sys::list_nth(queries, 0) }.cast::<pg_sys::Query>())
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_custom_scan_index_eligibility(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(local_placement_count, i64),
        name!(remote_node_count, i64),
        name!(remote_available_node_count, i64),
        name!(remote_placement_count, i64),
        name!(remote_available_placement_count, i64),
        name!(remote_unavailable_placement_count, i64),
        name!(all_remote_placements_available, bool),
        name!(eligible_for_custom_scan, bool),
        name!(status, &'static str),
        name!(next_step, &'static str),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_custom_scan_index_eligibility") };
    let row = unsafe { am::spire_custom_scan_index_eligibility_row(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
        i64::try_from(row.local_placement_count).expect("local placement count should fit in i64"),
        i64::try_from(row.remote_node_count).expect("remote node count should fit in i64"),
        i64::try_from(row.remote_available_node_count)
            .expect("remote available node count should fit in i64"),
        i64::try_from(row.remote_placement_count)
            .expect("remote placement count should fit in i64"),
        i64::try_from(row.remote_available_placement_count)
            .expect("remote available placement count should fit in i64"),
        i64::try_from(row.remote_unavailable_placement_count)
            .expect("remote unavailable placement count should fit in i64"),
        row.all_remote_placements_available,
        row.eligible_for_custom_scan,
        row.status,
        row.next_step,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_ivf_index_drift_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(block_count, i64),
        name!(nlists, i64),
        name!(total_live_tuples, i64),
        name!(total_dead_tuples, i64),
        name!(inserted_since_build, i64),
        name!(changed_row_fraction, f64),
        name!(average_list_live_count, f64),
        name!(max_list_live_count, i64),
        name!(list_imbalance_ratio, f64),
        name!(empty_lists, i64),
        name!(reindex_recommended, bool),
        name!(reindex_reason, String),
        name!(changed_row_reindex_threshold, f64),
        name!(list_imbalance_reindex_threshold, f64),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_ivf_index(index_oid, "ec_ivf_index_drift_snapshot") };
    let snapshot = unsafe { am::ivf_index_drift_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::from(snapshot.block_count),
        i64::from(snapshot.nlists),
        i64::try_from(snapshot.total_live_tuples).expect("total live tuples should fit in i64"),
        i64::try_from(snapshot.total_dead_tuples).expect("total dead tuples should fit in i64"),
        i64::try_from(snapshot.inserted_since_build)
            .expect("inserted-since-build should fit in i64"),
        snapshot.changed_row_fraction,
        snapshot.average_list_live_count,
        i64::try_from(snapshot.max_list_live_count).expect("max list count should fit in i64"),
        snapshot.list_imbalance_ratio,
        i64::from(snapshot.empty_lists),
        snapshot.reindex_recommended,
        snapshot.reindex_reason.to_owned(),
        snapshot.changed_row_reindex_threshold,
        snapshot.list_imbalance_reindex_threshold,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_ivf_index_admin_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(block_count, i64),
        name!(index_pages, f64),
        name!(reltuples, f64),
        name!(dimensions, i32),
        name!(nlists, i64),
        name!(relation_nprobe, i64),
        name!(session_nprobe, Option<i32>),
        name!(effective_nprobe, i64),
        name!(effective_nprobe_source, String),
        name!(relation_rerank_width, i32),
        name!(relation_posting_slack_percent, i32),
        name!(session_rerank_width, Option<i32>),
        name!(effective_rerank_width, i32),
        name!(effective_rerank_width_source, String),
        name!(training_sample_rows, i64),
        name!(training_version, i32),
        name!(storage_format, String),
        name!(rerank, String),
        name!(total_live_tuples, i64),
        name!(total_dead_tuples, i64),
        name!(inserted_since_build, i64),
        name!(changed_row_fraction, f64),
        name!(average_list_live_count, f64),
        name!(max_list_live_count, i64),
        name!(list_imbalance_ratio, f64),
        name!(empty_lists, i64),
        name!(reindex_recommended, bool),
        name!(reindex_reason, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_ivf_index(index_oid, "ec_ivf_index_admin_snapshot") };
    let snapshot = unsafe { am::ivf_index_admin_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::from(snapshot.block_count),
        snapshot.index_pages,
        snapshot.reltuples,
        i32::from(snapshot.dimensions),
        i64::from(snapshot.nlists),
        i64::from(snapshot.relation_nprobe),
        snapshot
            .session_nprobe
            .map(|value| i32::try_from(value).expect("session nprobe should fit in i32")),
        i64::from(snapshot.effective_nprobe),
        snapshot.effective_nprobe_source.to_owned(),
        snapshot.relation_rerank_width,
        snapshot.relation_posting_slack_percent,
        snapshot.session_rerank_width,
        snapshot.effective_rerank_width,
        snapshot.effective_rerank_width_source.to_owned(),
        i64::from(snapshot.training_sample_rows),
        i32::from(snapshot.training_version),
        snapshot.storage_format.to_owned(),
        snapshot.rerank.to_owned(),
        i64::try_from(snapshot.total_live_tuples).expect("total live tuples should fit in i64"),
        i64::try_from(snapshot.total_dead_tuples).expect("total dead tuples should fit in i64"),
        i64::try_from(snapshot.inserted_since_build)
            .expect("inserted-since-build should fit in i64"),
        snapshot.changed_row_fraction,
        snapshot.average_list_live_count,
        i64::try_from(snapshot.max_list_live_count).expect("max list count should fit in i64"),
        snapshot.list_imbalance_ratio,
        i64::from(snapshot.empty_lists),
        snapshot.reindex_recommended,
        snapshot.reindex_reason.to_owned(),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_active_snapshot_diagnostics(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(next_pid, i64),
        name!(next_local_vec_seq, i64),
        name!(consistency_mode, String),
        name!(object_count, i64),
        name!(placement_count, i64),
        name!(local_store_count, i64),
        name!(available_placement_count, i64),
        name!(stale_placement_count, i64),
        name!(unavailable_placement_count, i64),
        name!(skipped_placement_count, i64),
        name!(root_object_count, i64),
        name!(internal_object_count, i64),
        name!(leaf_object_count, i64),
        name!(delta_object_count, i64),
        name!(routing_child_count, i64),
        name!(leaf_assignment_count, i64),
        name!(delta_assignment_count, i64),
        name!(available_object_bytes, i64),
        name!(routing_object_bytes, i64),
        name!(leaf_object_bytes, i64),
        name!(delta_object_bytes, i64),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_index_active_snapshot_diagnostics")
    };
    let diagnostics = unsafe { am::spire_active_snapshot_diagnostics(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(diagnostics.active_epoch).expect("active epoch should fit in i64"),
        i64::try_from(diagnostics.next_pid).expect("next pid should fit in i64"),
        i64::try_from(diagnostics.next_local_vec_seq)
            .expect("next local vec sequence should fit in i64"),
        diagnostics.consistency_mode.to_owned(),
        i64::try_from(diagnostics.object_count).expect("object count should fit in i64"),
        i64::try_from(diagnostics.placement_count).expect("placement count should fit in i64"),
        i64::try_from(diagnostics.local_store_count).expect("local store count should fit in i64"),
        i64::try_from(diagnostics.available_placement_count)
            .expect("available placement count should fit in i64"),
        i64::try_from(diagnostics.stale_placement_count)
            .expect("stale placement count should fit in i64"),
        i64::try_from(diagnostics.unavailable_placement_count)
            .expect("unavailable placement count should fit in i64"),
        i64::try_from(diagnostics.skipped_placement_count)
            .expect("skipped placement count should fit in i64"),
        i64::try_from(diagnostics.root_object_count).expect("root object count should fit in i64"),
        i64::try_from(diagnostics.internal_object_count)
            .expect("internal object count should fit in i64"),
        i64::try_from(diagnostics.leaf_object_count).expect("leaf object count should fit in i64"),
        i64::try_from(diagnostics.delta_object_count)
            .expect("delta object count should fit in i64"),
        i64::try_from(diagnostics.routing_child_count)
            .expect("routing child count should fit in i64"),
        i64::try_from(diagnostics.leaf_assignment_count)
            .expect("leaf assignment count should fit in i64"),
        i64::try_from(diagnostics.delta_assignment_count)
            .expect("delta assignment count should fit in i64"),
        i64::try_from(diagnostics.available_object_bytes)
            .expect("available object bytes should fit in i64"),
        i64::try_from(diagnostics.routing_object_bytes)
            .expect("routing object bytes should fit in i64"),
        i64::try_from(diagnostics.leaf_object_bytes).expect("leaf object bytes should fit in i64"),
        i64::try_from(diagnostics.delta_object_bytes)
            .expect("delta object bytes should fit in i64"),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_allocator_snapshot(
    index_oid: pg_sys::Oid,
    warn_within: i64,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(warn_within, i64),
        name!(next_pid, i64),
        name!(remaining_pid_allocations, String),
        name!(pid_near_exhaustion, bool),
        name!(next_local_vec_seq, i64),
        name!(remaining_local_vec_id_allocations, String),
        name!(local_vec_id_near_exhaustion, bool),
    ),
> {
    if warn_within < 0 {
        pgrx::error!("ec_spire allocator warning threshold must be non-negative");
    }
    if unsafe { !relation_oid_exists(index_oid) } {
        return TableIterator::new(Vec::new().into_iter());
    }
    let warn_within =
        u64::try_from(warn_within).expect("non-negative warning threshold should fit in u64");
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_allocator_snapshot") };
    let snapshot = unsafe { am::spire_index_allocator_snapshot(index_relation, warn_within) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(snapshot.active_epoch).expect("active epoch should fit in i64"),
        i64::try_from(snapshot.warn_within).expect("warning threshold should fit in i64"),
        i64::try_from(snapshot.next_pid).expect("next pid should fit in i64"),
        snapshot.remaining_pid_allocations.to_string(),
        snapshot.pid_near_exhaustion,
        i64::try_from(snapshot.next_local_vec_seq)
            .expect("next local vec sequence should fit in i64"),
        snapshot.remaining_local_vec_id_allocations.to_string(),
        snapshot.local_vec_id_near_exhaustion,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_placement_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(local_store_id, i64),
        name!(store_relid, pg_sys::Oid),
        name!(placement_count, i64),
        name!(available_placement_count, i64),
        name!(stale_placement_count, i64),
        name!(unavailable_placement_count, i64),
        name!(skipped_placement_count, i64),
        name!(object_count, i64),
        name!(root_object_count, i64),
        name!(internal_object_count, i64),
        name!(leaf_object_count, i64),
        name!(delta_object_count, i64),
        name!(routing_child_count, i64),
        name!(assignment_count, i64),
        name!(placement_object_bytes, i64),
        name!(available_object_bytes, i64),
        name!(routing_object_bytes, i64),
        name!(leaf_object_bytes, i64),
        name!(delta_object_bytes, i64),
    ),
> {
    if unsafe { !relation_oid_exists(index_oid) } {
        return TableIterator::new(Vec::new().into_iter());
    }
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_placement_snapshot") };
    let rows = unsafe { am::spire_index_placement_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::from(row.node_id),
            i64::from(row.local_store_id),
            pg_sys::Oid::from(row.store_relid),
            i64::try_from(row.placement_count).expect("placement count should fit in i64"),
            i64::try_from(row.available_placement_count)
                .expect("available placement count should fit in i64"),
            i64::try_from(row.stale_placement_count)
                .expect("stale placement count should fit in i64"),
            i64::try_from(row.unavailable_placement_count)
                .expect("unavailable placement count should fit in i64"),
            i64::try_from(row.skipped_placement_count)
                .expect("skipped placement count should fit in i64"),
            i64::try_from(row.object_count).expect("object count should fit in i64"),
            i64::try_from(row.root_object_count).expect("root object count should fit in i64"),
            i64::try_from(row.internal_object_count)
                .expect("internal object count should fit in i64"),
            i64::try_from(row.leaf_object_count).expect("leaf object count should fit in i64"),
            i64::try_from(row.delta_object_count).expect("delta object count should fit in i64"),
            i64::try_from(row.routing_child_count).expect("routing child count should fit in i64"),
            i64::try_from(row.assignment_count).expect("assignment count should fit in i64"),
            i64::try_from(row.placement_object_bytes)
                .expect("placement object bytes should fit in i64"),
            i64::try_from(row.available_object_bytes)
                .expect("available object bytes should fit in i64"),
            i64::try_from(row.routing_object_bytes)
                .expect("routing object bytes should fit in i64"),
            i64::try_from(row.leaf_object_bytes).expect("leaf object bytes should fit in i64"),
            i64::try_from(row.delta_object_bytes).expect("delta object bytes should fit in i64"),
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_node_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(node_kind, &'static str),
        name!(descriptor_generation, i64),
        name!(descriptor_state, &'static str),
        name!(placement_count, i64),
        name!(available_placement_count, i64),
        name!(stale_placement_count, i64),
        name!(unavailable_placement_count, i64),
        name!(skipped_placement_count, i64),
        name!(local_store_count, i64),
        name!(last_seen_at_micros, i64),
        name!(last_served_epoch, i64),
        name!(min_retained_epoch, i64),
        name!(extension_version, String),
        name!(last_error, String),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_node_snapshot") };
    let rows = unsafe { am::spire_remote_node_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::from(row.node_id),
            row.node_kind,
            i64::try_from(row.descriptor_generation)
                .expect("descriptor generation should fit in i64"),
            row.descriptor_state,
            i64::try_from(row.placement_count).expect("placement count should fit in i64"),
            i64::try_from(row.available_placement_count)
                .expect("available placement count should fit in i64"),
            i64::try_from(row.stale_placement_count)
                .expect("stale placement count should fit in i64"),
            i64::try_from(row.unavailable_placement_count)
                .expect("unavailable placement count should fit in i64"),
            i64::try_from(row.skipped_placement_count)
                .expect("skipped placement count should fit in i64"),
            i64::try_from(row.local_store_count).expect("local store count should fit in i64"),
            row.last_seen_at_micros,
            i64::try_from(row.last_served_epoch).expect("last served epoch should fit in i64"),
            i64::try_from(row.min_retained_epoch).expect("min retained epoch should fit in i64"),
            row.extension_version,
            row.last_error,
            row.status,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_node_descriptor_contract() -> TableIterator<
    'static,
    (
        name!(field_ordinal, i64),
        name!(field_name, &'static str),
        name!(pg_type, &'static str),
        name!(semantic_role, &'static str),
        name!(required, bool),
        name!(validator, &'static str),
    ),
> {
    let rows = am::spire_remote_node_descriptor_contract_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.field_ordinal).expect("field ordinal should fit in i64"),
            row.field_name,
            row.pg_type,
            row.semantic_role,
            row.required,
            row.validator,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_node_descriptor_state_contract() -> TableIterator<
    'static,
    (
        name!(state_ordinal, i64),
        name!(descriptor_state, &'static str),
        name!(state_source, &'static str),
        name!(read_eligible, bool),
        name!(snapshot_status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let rows = am::spire_remote_node_descriptor_state_contract_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.state_ordinal).expect("state ordinal should fit in i64"),
            row.descriptor_state,
            row.state_source,
            row.read_eligible,
            row.snapshot_status,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_node_descriptor_registration_contract() -> TableIterator<
    'static,
    (
        name!(step_ordinal, i64),
        name!(step_name, &'static str),
        name!(input_field, &'static str),
        name!(semantic_role, &'static str),
        name!(validator, &'static str),
        name!(persistence_action, &'static str),
        name!(failure_status, &'static str),
    ),
> {
    let rows = am::spire_remote_node_descriptor_registration_contract_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.step_ordinal).expect("step ordinal should fit in i64"),
            row.step_name,
            row.input_field,
            row.semantic_role,
            row.validator,
            row.persistence_action,
            row.failure_status,
        )
    }))
}

#[pg_extern(strict)]
fn ec_spire_register_remote_node_descriptor(
    index_oid: pg_sys::Oid,
    node_id: i32,
    descriptor_generation: i64,
    conninfo_secret_name: String,
    remote_index_identity: Vec<u8>,
    remote_index_regclass: String,
    descriptor_state: String,
    last_served_epoch: i64,
    min_retained_epoch: i64,
    extension_version: String,
    last_error: String,
) -> bool {
    if node_id <= 0 {
        pgrx::error!("ec_spire_register_remote_node_descriptor node_id must be greater than 0");
    }
    if descriptor_generation < 0 {
        pgrx::error!(
            "ec_spire_register_remote_node_descriptor descriptor_generation must be non-negative"
        );
    }
    if conninfo_secret_name.is_empty() {
        pgrx::error!(
            "ec_spire_register_remote_node_descriptor conninfo_secret_name must be nonempty"
        );
    }
    if remote_index_identity.is_empty() {
        pgrx::error!(
            "ec_spire_register_remote_node_descriptor remote_index_identity must be nonempty"
        );
    }
    if remote_index_regclass.is_empty() {
        pgrx::error!(
            "ec_spire_register_remote_node_descriptor remote_index_regclass must be nonempty"
        );
    }
    if !am::spire_remote_node_descriptor_catalog_state_is_supported(descriptor_state.as_str()) {
        pgrx::error!(
            "ec_spire_register_remote_node_descriptor descriptor_state must be active, draining, disabled, or failed"
        );
    }
    if last_served_epoch < 0 {
        pgrx::error!(
            "ec_spire_register_remote_node_descriptor last_served_epoch must be non-negative"
        );
    }
    if min_retained_epoch < 0 {
        pgrx::error!(
            "ec_spire_register_remote_node_descriptor min_retained_epoch must be non-negative"
        );
    }
    if extension_version.is_empty() {
        pgrx::error!("ec_spire_register_remote_node_descriptor extension_version must be nonempty");
    }
    let conninfo_provider_lookup_key =
        am::spire_remote_conninfo_secret_provider_lookup_key(&conninfo_secret_name)
            .unwrap_or_else(|e| pgrx::error!("ec_spire_register_remote_node_descriptor {e}"));
    let node_id_u32 = u32::try_from(node_id).unwrap_or_else(|_| {
        pgrx::error!("ec_spire_register_remote_node_descriptor node_id is out of range")
    });
    let remote_insert_shape_fingerprint = match am::spire_remote_write_shape_fingerprint_from_secret(
        &conninfo_secret_name,
        node_id_u32,
        &remote_index_regclass,
    ) {
        Ok(fingerprint) => fingerprint,
        Err(error) => {
            pgrx::notice!(
                    "ec_spire_register_remote_node_descriptor skipped remote insert shape fingerprint for node_id {node_id}: {error}; descriptor will fail closed for coordinator-routed writes until refreshed with reachable remote"
                );
            "unset".to_owned()
        }
    };
    if let Some(warning) =
        am::spire_remote_prepared_transaction_registration_warning(&conninfo_secret_name, node_id)
    {
        pgrx::notice!("{warning}");
    }

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_register_remote_node_descriptor") };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let result = Spi::connect_mut(|client| {
        client
            .update(
                "LOCK TABLE ec_spire_remote_node_descriptor IN SHARE ROW EXCLUSIVE MODE",
                None,
                &[],
            )
            .map_err(|e| format!("ec_spire remote node descriptor lock failed: {e}"))?;
        let existing_secret_names = client
            .select(
                "SELECT conninfo_secret_name \
                   FROM ec_spire_remote_node_descriptor \
                  WHERE coordinator_index_oid = $1::oid \
                    AND node_id <> $2::integer",
                None,
                &[index_oid.into(), node_id.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote node descriptor secret collision scan failed: {e}")
            })?
            .map(|row| {
                row["conninfo_secret_name"]
                    .value::<String>()
                    .map_err(|e| {
                        format!(
                            "ec_spire remote node descriptor conninfo secret decode failed: {e}"
                        )
                    })?
                    .ok_or_else(|| {
                        "ec_spire remote node descriptor conninfo secret is null".to_owned()
                    })
            })
            .collect::<Result<Vec<_>, String>>()?;
        for existing_secret_name in existing_secret_names {
            if existing_secret_name == conninfo_secret_name {
                continue;
            }
            let existing_lookup_key =
                am::spire_remote_conninfo_secret_provider_lookup_key(&existing_secret_name)?;
            if existing_lookup_key == conninfo_provider_lookup_key {
                return Err(format!(
                    "ec_spire_register_remote_node_descriptor conninfo_secret_name maps to provider_lookup_key {conninfo_provider_lookup_key}, which collides with existing conninfo_secret_name {existing_secret_name}"
                ));
            }
        }

        client
            .select(
                "INSERT INTO ec_spire_remote_node_descriptor \
             (coordinator_index_oid, node_id, descriptor_generation, \
              conninfo_secret_name, remote_index_identity, remote_index_regclass, \
              coordinator_insert_shape_fingerprint, remote_insert_shape_fingerprint, \
              descriptor_state, last_seen_at, \
              last_served_epoch, min_retained_epoch, \
              extension_version, last_error) \
             VALUES ($1::oid, $2::integer, $3::bigint, $4::text, $5::bytea, $6::text, \
                     ec_spire_coordinator_index_shape_fingerprint($1::oid::regclass), \
                     $12::text, $7::text, clock_timestamp(), $8::bigint, $9::bigint, $10::text, $11::text) \
             ON CONFLICT (coordinator_index_oid, node_id) DO UPDATE SET \
                 descriptor_generation = EXCLUDED.descriptor_generation, \
                 conninfo_secret_name = EXCLUDED.conninfo_secret_name, \
                 remote_index_identity = EXCLUDED.remote_index_identity, \
                 remote_index_regclass = EXCLUDED.remote_index_regclass, \
                 coordinator_insert_shape_fingerprint = EXCLUDED.coordinator_insert_shape_fingerprint, \
                 remote_insert_shape_fingerprint = EXCLUDED.remote_insert_shape_fingerprint, \
                 descriptor_state = EXCLUDED.descriptor_state, \
                 last_seen_at = EXCLUDED.last_seen_at, \
                 last_served_epoch = EXCLUDED.last_served_epoch, \
                 min_retained_epoch = EXCLUDED.min_retained_epoch, \
                 extension_version = EXCLUDED.extension_version, \
                 last_error = EXCLUDED.last_error \
              WHERE EXCLUDED.descriptor_generation > \
                    ec_spire_remote_node_descriptor.descriptor_generation \
             RETURNING true AS registered",
                None,
                &[
                    index_oid.into(),
                    node_id.into(),
                    descriptor_generation.into(),
                    conninfo_secret_name.as_str().into(),
                    remote_index_identity.into(),
                    remote_index_regclass.as_str().into(),
                    descriptor_state.as_str().into(),
                    last_served_epoch.into(),
                    min_retained_epoch.into(),
                    extension_version.as_str().into(),
                    last_error.as_str().into(),
                    remote_insert_shape_fingerprint.as_str().into(),
                ],
            )
            .map_err(|e| format!("ec_spire remote node descriptor upsert failed: {e}"))?
            .map(|row| {
                row["registered"]
                    .value::<bool>()
                    .map_err(|e| {
                        format!("ec_spire remote node descriptor registration decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire remote node descriptor registration result is null".to_owned()
                    })
            })
            .next()
            .transpose()
            .map(|value| value.unwrap_or(false))
    });
    let registered = result.unwrap_or_else(|e| pgrx::error!("{e}"));
    if !registered {
        pgrx::ereport!(
            ERROR,
            pgrx::PgSqlErrorCode::ERRCODE_T_R_SERIALIZATION_FAILURE,
            "ec_spire_register_remote_node_descriptor descriptor_generation must advance existing descriptor_generation",
            "Retry the whole coordinator write after the winning descriptor refresh commits."
        );
    }
    true
}

#[pg_extern(strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_reap_orphaned_remote_prepared_xacts(
    node_id: i32,
) -> TableIterator<
    'static,
    (
        name!(node_id, i64),
        name!(index_oid, i64),
        name!(served_epoch, i64),
        name!(xid, i64),
        name!(gid, String),
        name!(intent_state, String),
        name!(coordinator_xid_live, bool),
        name!(action, String),
        name!(detail, String),
    ),
> {
    if node_id <= 0 {
        pgrx::error!("ec_spire_reap_orphaned_remote_prepared_xacts node_id must be greater than 0");
    }
    let node_id = u32::try_from(node_id).unwrap_or_else(|_| {
        pgrx::error!("ec_spire_reap_orphaned_remote_prepared_xacts node_id is out of range")
    });
    let rows = am::spire_reap_orphaned_remote_prepared_xacts(node_id)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::from(row.node_id),
            i64::from(row.index_oid),
            i64::try_from(row.served_epoch).expect("served epoch should fit in i64"),
            i64::try_from(row.xid).expect("xid should fit in i64"),
            row.gid,
            row.intent_state,
            row.coordinator_xid_live,
            row.action,
            row.detail,
        )
    }))
}

#[pg_extern]
#[allow(clippy::type_complexity)]
fn ec_spire_reap_all_orphaned_remote_prepared_xacts() -> TableIterator<
    'static,
    (
        name!(node_id, i64),
        name!(index_oid, i64),
        name!(served_epoch, i64),
        name!(xid, i64),
        name!(gid, String),
        name!(intent_state, String),
        name!(coordinator_xid_live, bool),
        name!(action, String),
        name!(detail, String),
    ),
> {
    let rows =
        am::spire_reap_orphaned_remote_prepared_xacts_all().unwrap_or_else(|e| pgrx::error!("{e}"));
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::from(row.node_id),
            i64::from(row.index_oid),
            i64::try_from(row.served_epoch).expect("served epoch should fit in i64"),
            i64::try_from(row.xid).expect("xid should fit in i64"),
            row.gid,
            row.intent_state,
            row.coordinator_xid_live,
            row.action,
            row.detail,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_node_descriptor_readiness(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(field_ordinal, i64),
        name!(field_name, &'static str),
        name!(semantic_role, &'static str),
        name!(required, bool),
        name!(validator, &'static str),
        name!(descriptor_state, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_node_descriptor_readiness")
    };
    let rows = unsafe { am::spire_remote_node_descriptor_readiness(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::from(row.node_id),
            i64::try_from(row.field_ordinal).expect("field ordinal should fit in i64"),
            row.field_name,
            row.semantic_role,
            row.required,
            row.validator,
            row.descriptor_state,
            row.status,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_node_descriptor_readiness_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(remote_node_count, i64),
        name!(descriptor_field_count, i64),
        name!(required_field_count, i64),
        name!(ready_field_count, i64),
        name!(blocked_field_count, i64),
        name!(missing_required_field_count, i64),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_node_descriptor_readiness_summary",
        )
    };
    let row = unsafe { am::spire_remote_node_descriptor_readiness_summary(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
        i64::try_from(row.remote_node_count).expect("remote node count should fit in i64"),
        i64::try_from(row.descriptor_field_count)
            .expect("descriptor field count should fit in i64"),
        i64::try_from(row.required_field_count).expect("required field count should fit in i64"),
        i64::try_from(row.ready_field_count).expect("ready field count should fit in i64"),
        i64::try_from(row.blocked_field_count).expect("blocked field count should fit in i64"),
        i64::try_from(row.missing_required_field_count)
            .expect("missing required field count should fit in i64"),
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_node_capability_plan(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(node_kind, &'static str),
        name!(descriptor_generation, i64),
        name!(descriptor_state, &'static str),
        name!(required_last_served_epoch, i64),
        name!(required_min_retained_epoch, i64),
        name!(required_candidate_format, &'static str),
        name!(required_extension_version, &'static str),
        name!(conninfo_source, &'static str),
        name!(remote_index_identity_status, &'static str),
        name!(epoch_window_status, &'static str),
        name!(candidate_format_status, &'static str),
        name!(extension_version_status, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_node_capability_plan") };
    let rows = unsafe { am::spire_remote_node_capability_plan(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::from(row.node_id),
            row.node_kind,
            i64::try_from(row.descriptor_generation)
                .expect("descriptor generation should fit in i64"),
            row.descriptor_state,
            i64::try_from(row.required_last_served_epoch)
                .expect("required last served epoch should fit in i64"),
            i64::try_from(row.required_min_retained_epoch)
                .expect("required min retained epoch should fit in i64"),
            row.required_candidate_format,
            row.required_extension_version,
            row.conninfo_source,
            row.remote_index_identity_status,
            row.epoch_window_status,
            row.candidate_format_status,
            row.extension_version_status,
            row.status,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_node_capability_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_count, i64),
        name!(local_node_count, i64),
        name!(remote_node_count, i64),
        name!(ready_node_count, i64),
        name!(blocked_node_count, i64),
        name!(missing_descriptor_node_count, i64),
        name!(required_candidate_format, &'static str),
        name!(required_extension_version, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_node_capability_summary") };
    let row = unsafe { am::spire_remote_node_capability_summary(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
        i64::try_from(row.node_count).expect("node count should fit in i64"),
        i64::try_from(row.local_node_count).expect("local node count should fit in i64"),
        i64::try_from(row.remote_node_count).expect("remote node count should fit in i64"),
        i64::try_from(row.ready_node_count).expect("ready node count should fit in i64"),
        i64::try_from(row.blocked_node_count).expect("blocked node count should fit in i64"),
        i64::try_from(row.missing_descriptor_node_count)
            .expect("missing descriptor node count should fit in i64"),
        row.required_candidate_format,
        row.required_extension_version,
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_publish_plan(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(descriptor_state, &'static str),
        name!(placement_count, i64),
        name!(available_placement_count, i64),
        name!(stale_placement_count, i64),
        name!(unavailable_placement_count, i64),
        name!(skipped_placement_count, i64),
        name!(required_last_served_epoch, i64),
        name!(required_min_retained_epoch, i64),
        name!(last_served_epoch, i64),
        name!(min_retained_epoch, i64),
        name!(epoch_window_status, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_epoch_publish_plan") };
    let rows = unsafe { am::spire_remote_epoch_publish_plan(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::from(row.node_id),
            row.descriptor_state,
            i64::try_from(row.placement_count).expect("placement count should fit in i64"),
            i64::try_from(row.available_placement_count)
                .expect("available placement count should fit in i64"),
            i64::try_from(row.stale_placement_count)
                .expect("stale placement count should fit in i64"),
            i64::try_from(row.unavailable_placement_count)
                .expect("unavailable placement count should fit in i64"),
            i64::try_from(row.skipped_placement_count)
                .expect("skipped placement count should fit in i64"),
            i64::try_from(row.required_last_served_epoch)
                .expect("required last served epoch should fit in i64"),
            i64::try_from(row.required_min_retained_epoch)
                .expect("required min retained epoch should fit in i64"),
            i64::try_from(row.last_served_epoch).expect("last served epoch should fit in i64"),
            i64::try_from(row.min_retained_epoch).expect("min retained epoch should fit in i64"),
            row.epoch_window_status,
            row.status,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_publish_readiness(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(remote_node_count, i64),
        name!(remote_placement_count, i64),
        name!(remote_available_placement_count, i64),
        name!(remote_unavailable_placement_count, i64),
        name!(remote_skipped_placement_count, i64),
        name!(ready_remote_node_count, i64),
        name!(blocked_remote_node_count, i64),
        name!(missing_descriptor_node_count, i64),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_epoch_publish_readiness") };
    let row = unsafe { am::spire_remote_epoch_publish_readiness(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
        i64::try_from(row.remote_node_count).expect("remote node count should fit in i64"),
        i64::try_from(row.remote_placement_count)
            .expect("remote placement count should fit in i64"),
        i64::try_from(row.remote_available_placement_count)
            .expect("remote available placement count should fit in i64"),
        i64::try_from(row.remote_unavailable_placement_count)
            .expect("remote unavailable placement count should fit in i64"),
        i64::try_from(row.remote_skipped_placement_count)
            .expect("remote skipped placement count should fit in i64"),
        i64::try_from(row.ready_remote_node_count)
            .expect("ready remote node count should fit in i64"),
        i64::try_from(row.blocked_remote_node_count)
            .expect("blocked remote node count should fit in i64"),
        i64::try_from(row.missing_descriptor_node_count)
            .expect("missing descriptor node count should fit in i64"),
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_publish_gate_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(publish_scope, &'static str),
        name!(publish_decision, &'static str),
        name!(remote_node_count, i64),
        name!(remote_placement_count, i64),
        name!(ready_remote_node_count, i64),
        name!(blocked_remote_node_count, i64),
        name!(missing_descriptor_node_count, i64),
        name!(policy_contract, &'static str),
        name!(status, &'static str),
        name!(next_blocker, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_epoch_publish_gate_summary")
    };
    let row = unsafe { am::spire_remote_epoch_publish_gate_summary(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
        row.publish_scope,
        row.publish_decision,
        i64::try_from(row.remote_node_count).expect("remote node count should fit in i64"),
        i64::try_from(row.remote_placement_count)
            .expect("remote placement count should fit in i64"),
        i64::try_from(row.ready_remote_node_count)
            .expect("ready remote node count should fit in i64"),
        i64::try_from(row.blocked_remote_node_count)
            .expect("blocked remote node count should fit in i64"),
        i64::try_from(row.missing_descriptor_node_count)
            .expect("missing descriptor node count should fit in i64"),
        row.policy_contract,
        row.status,
        row.next_blocker,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_plan(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(descriptor_state, &'static str),
        name!(placement_count, i64),
        name!(required_last_served_epoch, i64),
        name!(required_min_retained_epoch, i64),
        name!(last_served_epoch, i64),
        name!(min_retained_epoch, i64),
        name!(epoch_window_status, &'static str),
        name!(manifest_action, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_epoch_manifest_plan") };
    let rows = unsafe { am::spire_remote_epoch_manifest_plan(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::from(row.node_id),
            row.descriptor_state,
            i64::try_from(row.placement_count).expect("placement count should fit in i64"),
            i64::try_from(row.required_last_served_epoch)
                .expect("required last served epoch should fit in i64"),
            i64::try_from(row.required_min_retained_epoch)
                .expect("required min retained epoch should fit in i64"),
            i64::try_from(row.last_served_epoch).expect("last served epoch should fit in i64"),
            i64::try_from(row.min_retained_epoch).expect("min retained epoch should fit in i64"),
            row.epoch_window_status,
            row.manifest_action,
            row.status,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(manifest_scope, &'static str),
        name!(manifest_decision, &'static str),
        name!(manifest_entry_count, i64),
        name!(included_remote_node_count, i64),
        name!(blocked_remote_node_count, i64),
        name!(remote_placement_count, i64),
        name!(publish_decision, &'static str),
        name!(next_blocker, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_epoch_manifest_summary") };
    let row = unsafe { am::spire_remote_epoch_manifest_summary(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
        row.manifest_scope,
        row.manifest_decision,
        i64::try_from(row.manifest_entry_count).expect("manifest entry count should fit in i64"),
        i64::try_from(row.included_remote_node_count)
            .expect("included remote node count should fit in i64"),
        i64::try_from(row.blocked_remote_node_count)
            .expect("blocked remote node count should fit in i64"),
        i64::try_from(row.remote_placement_count)
            .expect("remote placement count should fit in i64"),
        row.publish_decision,
        row.next_blocker,
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(strict)]
fn ec_spire_persist_remote_epoch_manifest(index_oid: pg_sys::Oid) -> bool {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_persist_remote_epoch_manifest") };
    let summary = unsafe { am::spire_remote_epoch_manifest_summary(index_relation) };
    let manifest_rows = unsafe { am::spire_remote_epoch_manifest_plan(index_relation) };

    if summary.manifest_decision != "emit_distributed_epoch_manifest" {
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        pgrx::error!(
            "ec_spire_persist_remote_epoch_manifest cannot persist remote epoch manifest when decision is '{}' with next_blocker '{}'",
            summary.manifest_decision,
            summary.next_blocker
        );
    }
    let included_rows = manifest_rows
        .into_iter()
        .filter(|row| row.manifest_action == "include_remote_node")
        .collect::<Vec<_>>();
    if included_rows.is_empty() {
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        pgrx::error!(
            "ec_spire_persist_remote_epoch_manifest requires at least one included remote manifest entry"
        );
    }

    let active_epoch = i64::try_from(summary.active_epoch).expect("active epoch should fit in i64");
    let manifest_entry_count = i64::try_from(summary.manifest_entry_count)
        .expect("manifest entry count should fit in i64");
    let included_remote_node_count = i64::try_from(summary.included_remote_node_count)
        .expect("included remote node count should fit in i64");
    let remote_placement_count = i64::try_from(summary.remote_placement_count)
        .expect("remote placement count should fit in i64");

    let result = Spi::connect_mut(|client| {
        let current_active_epoch = i64::try_from(unsafe { am::spire_active_epoch(index_relation) })
            .map_err(|_| "ec_spire remote epoch manifest active epoch exceeds i64")?;
        if current_active_epoch != active_epoch {
            return Err(format!(
                "ec_spire_persist_remote_epoch_manifest active epoch changed from {active_epoch} to {current_active_epoch}; retry persistence"
            ));
        }

        client
            .update(
                "INSERT INTO ec_spire_remote_epoch_manifest \
                 (coordinator_index_oid, active_epoch, manifest_scope, manifest_decision, \
                  manifest_entry_count, included_remote_node_count, remote_placement_count, \
                  publish_decision, status, persisted_at_micros) \
                 VALUES ($1::oid, $2::bigint, $3::text, $4::text, $5::bigint, $6::bigint, \
                         $7::bigint, $8::text, $9::text, \
                         (extract(epoch from clock_timestamp()) * 1000000)::bigint) \
                 ON CONFLICT (coordinator_index_oid, active_epoch) DO UPDATE SET \
                     manifest_scope = EXCLUDED.manifest_scope, \
                     manifest_decision = EXCLUDED.manifest_decision, \
                     manifest_entry_count = EXCLUDED.manifest_entry_count, \
                     included_remote_node_count = EXCLUDED.included_remote_node_count, \
                     remote_placement_count = EXCLUDED.remote_placement_count, \
                     publish_decision = EXCLUDED.publish_decision, \
                     status = EXCLUDED.status, \
                     persisted_at_micros = EXCLUDED.persisted_at_micros",
                None,
                &[
                    index_oid.into(),
                    active_epoch.into(),
                    summary.manifest_scope.into(),
                    summary.manifest_decision.into(),
                    manifest_entry_count.into(),
                    included_remote_node_count.into(),
                    remote_placement_count.into(),
                    summary.publish_decision.into(),
                    summary.status.into(),
                ],
            )
            .map_err(|e| format!("ec_spire remote epoch manifest header persist failed: {e}"))?;
        client
            .update(
                "DELETE FROM ec_spire_remote_epoch_manifest_entry \
                  WHERE coordinator_index_oid = $1::oid AND active_epoch = $2::bigint",
                None,
                &[index_oid.into(), active_epoch.into()],
            )
            .map_err(|e| format!("ec_spire remote epoch manifest entry replace failed: {e}"))?;

        for row in included_rows {
            let node_id = i32::try_from(row.node_id).map_err(|_| {
                "ec_spire remote epoch manifest node_id should fit in i32".to_owned()
            })?;
            let placement_count = i64::try_from(row.placement_count)
                .map_err(|_| "ec_spire remote epoch manifest placement_count exceeds i64")?;
            let required_last_served_epoch = i64::try_from(row.required_last_served_epoch)
                .map_err(|_| {
                    "ec_spire remote epoch manifest required_last_served_epoch exceeds i64"
                        .to_owned()
                })?;
            let required_min_retained_epoch = i64::try_from(row.required_min_retained_epoch)
                .map_err(|_| {
                    "ec_spire remote epoch manifest required_min_retained_epoch exceeds i64"
                        .to_owned()
                })?;
            let last_served_epoch = i64::try_from(row.last_served_epoch)
                .map_err(|_| "ec_spire remote epoch manifest last_served_epoch exceeds i64")?;
            let min_retained_epoch = i64::try_from(row.min_retained_epoch)
                .map_err(|_| "ec_spire remote epoch manifest min_retained_epoch exceeds i64")?;

            client
                .update(
                    "INSERT INTO ec_spire_remote_epoch_manifest_entry \
                     (coordinator_index_oid, active_epoch, node_id, descriptor_state, \
                      placement_count, required_last_served_epoch, required_min_retained_epoch, \
                      last_served_epoch, min_retained_epoch, epoch_window_status, \
                      manifest_action, status) \
                     VALUES ($1::oid, $2::bigint, $3::integer, $4::text, $5::bigint, \
                             $6::bigint, $7::bigint, $8::bigint, $9::bigint, $10::text, \
                             $11::text, $12::text)",
                    None,
                    &[
                        index_oid.into(),
                        active_epoch.into(),
                        node_id.into(),
                        row.descriptor_state.into(),
                        placement_count.into(),
                        required_last_served_epoch.into(),
                        required_min_retained_epoch.into(),
                        last_served_epoch.into(),
                        min_retained_epoch.into(),
                        row.epoch_window_status.into(),
                        row.manifest_action.into(),
                        row.status.into(),
                    ],
                )
                .map_err(|e| {
                    format!(
                        "ec_spire remote epoch manifest entry persist failed for node_id {}: {e}",
                        row.node_id
                    )
                })?;
        }
        Ok::<(), String>(())
    });
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result.unwrap_or_else(|e| pgrx::error!("{e}"));
    true
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_catalog(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(manifest_scope, String),
        name!(manifest_decision, String),
        name!(manifest_entry_count, i64),
        name!(included_remote_node_count, i64),
        name!(remote_placement_count, i64),
        name!(publish_decision, String),
        name!(status, String),
        name!(persisted_at_micros, i64),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_epoch_manifest_catalog") };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let sql = format!(
        "SELECT active_epoch, manifest_scope, manifest_decision, manifest_entry_count, \
                included_remote_node_count, remote_placement_count, publish_decision, status, \
                persisted_at_micros \
           FROM ec_spire_remote_epoch_manifest \
          WHERE coordinator_index_oid = '{}'::oid \
          ORDER BY active_epoch",
        u32::from(index_oid)
    );
    let rows = Spi::connect(|client| {
        client
            .select(sql.as_str(), None, &[])
            .map_err(|e| format!("ec_spire remote epoch manifest catalog read failed: {e}"))?
            .map(|row| {
                Ok((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest active_epoch decode failed: {e}"))?
                        .ok_or_else(|| "manifest active_epoch is null".to_owned())?,
                    row["manifest_scope"]
                        .value::<String>()
                        .map_err(|e| format!("manifest_scope decode failed: {e}"))?
                        .ok_or_else(|| "manifest_scope is null".to_owned())?,
                    row["manifest_decision"]
                        .value::<String>()
                        .map_err(|e| format!("manifest_decision decode failed: {e}"))?
                        .ok_or_else(|| "manifest_decision is null".to_owned())?,
                    row["manifest_entry_count"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest_entry_count decode failed: {e}"))?
                        .ok_or_else(|| "manifest_entry_count is null".to_owned())?,
                    row["included_remote_node_count"]
                        .value::<i64>()
                        .map_err(|e| format!("included_remote_node_count decode failed: {e}"))?
                        .ok_or_else(|| "included_remote_node_count is null".to_owned())?,
                    row["remote_placement_count"]
                        .value::<i64>()
                        .map_err(|e| format!("remote_placement_count decode failed: {e}"))?
                        .ok_or_else(|| "remote_placement_count is null".to_owned())?,
                    row["publish_decision"]
                        .value::<String>()
                        .map_err(|e| format!("publish_decision decode failed: {e}"))?
                        .ok_or_else(|| "publish_decision is null".to_owned())?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| format!("manifest status decode failed: {e}"))?
                        .ok_or_else(|| "manifest status is null".to_owned())?,
                    row["persisted_at_micros"]
                        .value::<i64>()
                        .map_err(|e| format!("persisted_at_micros decode failed: {e}"))?
                        .ok_or_else(|| "persisted_at_micros is null".to_owned())?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::new(rows.into_iter())
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_entry_catalog(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(descriptor_state, String),
        name!(placement_count, i64),
        name!(required_last_served_epoch, i64),
        name!(required_min_retained_epoch, i64),
        name!(last_served_epoch, i64),
        name!(min_retained_epoch, i64),
        name!(epoch_window_status, String),
        name!(manifest_action, String),
        name!(status, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_epoch_manifest_entry_catalog")
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let sql = format!(
        "SELECT active_epoch, node_id, descriptor_state, placement_count, \
                required_last_served_epoch, required_min_retained_epoch, last_served_epoch, \
                min_retained_epoch, epoch_window_status, manifest_action, status \
           FROM ec_spire_remote_epoch_manifest_entry \
          WHERE coordinator_index_oid = '{}'::oid \
          ORDER BY active_epoch, node_id",
        u32::from(index_oid)
    );
    let rows = Spi::connect(|client| {
        client
            .select(sql.as_str(), None, &[])
            .map_err(|e| format!("ec_spire remote epoch manifest entry read failed: {e}"))?
            .map(|row| {
                Ok((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest entry active_epoch decode failed: {e}"))?
                        .ok_or_else(|| "manifest entry active_epoch is null".to_owned())?,
                    i64::from(
                        row["node_id"]
                            .value::<i32>()
                            .map_err(|e| format!("manifest entry node_id decode failed: {e}"))?
                            .ok_or_else(|| "manifest entry node_id is null".to_owned())?,
                    ),
                    row["descriptor_state"]
                        .value::<String>()
                        .map_err(|e| format!("manifest entry descriptor_state decode failed: {e}"))?
                        .ok_or_else(|| "manifest entry descriptor_state is null".to_owned())?,
                    row["placement_count"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest entry placement_count decode failed: {e}"))?
                        .ok_or_else(|| "manifest entry placement_count is null".to_owned())?,
                    row["required_last_served_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest entry required_last_served_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest entry required_last_served_epoch is null".to_owned()
                        })?,
                    row["required_min_retained_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest entry required_min_retained_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest entry required_min_retained_epoch is null".to_owned()
                        })?,
                    row["last_served_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest entry last_served_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest entry last_served_epoch is null".to_owned())?,
                    row["min_retained_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest entry min_retained_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest entry min_retained_epoch is null".to_owned())?,
                    row["epoch_window_status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest entry epoch_window_status decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest entry epoch_window_status is null".to_owned())?,
                    row["manifest_action"]
                        .value::<String>()
                        .map_err(|e| format!("manifest entry manifest_action decode failed: {e}"))?
                        .ok_or_else(|| "manifest entry manifest_action is null".to_owned())?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| format!("manifest entry status decode failed: {e}"))?
                        .ok_or_else(|| "manifest entry status is null".to_owned())?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::new(rows.into_iter())
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_catalog_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(current_manifest_decision, &'static str),
        name!(current_included_remote_node_count, i64),
        name!(current_remote_placement_count, i64),
        name!(persisted_manifest_count, i64),
        name!(persisted_entry_count, i64),
        name!(persisted_entry_mismatch_count, i64),
        name!(persisted_remote_placement_count, i64),
        name!(catalog_status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_epoch_manifest_catalog_summary")
    };
    let summary = unsafe { am::spire_remote_epoch_manifest_summary(index_relation) };
    let current_entries = unsafe { am::spire_remote_epoch_manifest_plan(index_relation) }
        .into_iter()
        .filter(|row| row.manifest_action == "include_remote_node")
        .collect::<Vec<_>>();
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let active_epoch = i64::try_from(summary.active_epoch).expect("active epoch should fit in i64");
    let catalog = Spi::connect(|client| {
        client
            .select(
                "SELECT count(m.*)::bigint AS manifest_count, \
                        coalesce(max(m.remote_placement_count), 0)::bigint AS remote_placement_count, \
                        count(e.*)::bigint AS entry_count \
                   FROM ec_spire_remote_epoch_manifest m \
                   LEFT JOIN ec_spire_remote_epoch_manifest_entry e \
                     ON e.coordinator_index_oid = m.coordinator_index_oid \
                    AND e.active_epoch = m.active_epoch \
                  WHERE m.coordinator_index_oid = $1::oid \
                    AND m.active_epoch = $2::bigint",
                None,
                &[index_oid.into(), active_epoch.into()],
            )
            .map_err(|e| format!("ec_spire remote epoch manifest catalog summary read failed: {e}"))?
            .map(|row| {
                Ok::<(i64, i64, i64), String>((
                    row["manifest_count"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest_count decode failed: {e}"))?
                        .ok_or_else(|| "manifest_count is null".to_owned())?,
                    row["entry_count"]
                        .value::<i64>()
                        .map_err(|e| format!("entry_count decode failed: {e}"))?
                        .ok_or_else(|| "entry_count is null".to_owned())?,
                    row["remote_placement_count"]
                        .value::<i64>()
                        .map_err(|e| format!("remote_placement_count decode failed: {e}"))?
                        .ok_or_else(|| "remote_placement_count is null".to_owned())?,
                ))
            })
            .next()
            .transpose()
            .map(|value| value.unwrap_or((0, 0, 0)))
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    let (persisted_manifest_count, persisted_entry_count, persisted_remote_placement_count) =
        catalog;
    let persisted_entries = Spi::connect(|client| {
        client
            .select(
                "SELECT node_id, placement_count, required_last_served_epoch, \
                        required_min_retained_epoch, last_served_epoch, min_retained_epoch, \
                        epoch_window_status, manifest_action, status \
                   FROM ec_spire_remote_epoch_manifest_entry \
                  WHERE coordinator_index_oid = $1::oid \
                    AND active_epoch = $2::bigint",
                None,
                &[index_oid.into(), active_epoch.into()],
            )
            .map_err(|e| format!("ec_spire remote epoch manifest entry summary read failed: {e}"))?
            .map(|row| {
                Ok((
                    row["node_id"]
                        .value::<i32>()
                        .map_err(|e| format!("entry node_id decode failed: {e}"))?
                        .ok_or_else(|| "entry node_id is null".to_owned())?,
                    row["placement_count"]
                        .value::<i64>()
                        .map_err(|e| format!("entry placement_count decode failed: {e}"))?
                        .ok_or_else(|| "entry placement_count is null".to_owned())?,
                    row["required_last_served_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("entry required_last_served_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| "entry required_last_served_epoch is null".to_owned())?,
                    row["required_min_retained_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("entry required_min_retained_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| "entry required_min_retained_epoch is null".to_owned())?,
                    row["last_served_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("entry last_served_epoch decode failed: {e}"))?
                        .ok_or_else(|| "entry last_served_epoch is null".to_owned())?,
                    row["min_retained_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("entry min_retained_epoch decode failed: {e}"))?
                        .ok_or_else(|| "entry min_retained_epoch is null".to_owned())?,
                    row["epoch_window_status"]
                        .value::<String>()
                        .map_err(|e| format!("entry epoch_window_status decode failed: {e}"))?
                        .ok_or_else(|| "entry epoch_window_status is null".to_owned())?,
                    row["manifest_action"]
                        .value::<String>()
                        .map_err(|e| format!("entry manifest_action decode failed: {e}"))?
                        .ok_or_else(|| "entry manifest_action is null".to_owned())?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| format!("entry status decode failed: {e}"))?
                        .ok_or_else(|| "entry status is null".to_owned())?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    let mut persisted_entry_mismatch_count = 0_i64;
    for current in &current_entries {
        let Some(persisted) = persisted_entries
            .iter()
            .find(|row| u32::try_from(row.0).ok() == Some(current.node_id))
        else {
            persisted_entry_mismatch_count += 1;
            continue;
        };
        let current_placement_count =
            i64::try_from(current.placement_count).expect("placement count should fit in i64");
        let current_required_last_served_epoch = i64::try_from(current.required_last_served_epoch)
            .expect("required last served epoch should fit in i64");
        let current_required_min_retained_epoch =
            i64::try_from(current.required_min_retained_epoch)
                .expect("required min retained epoch should fit in i64");
        let current_last_served_epoch =
            i64::try_from(current.last_served_epoch).expect("last served epoch should fit in i64");
        let current_min_retained_epoch = i64::try_from(current.min_retained_epoch)
            .expect("min retained epoch should fit in i64");
        if persisted.1 != current_placement_count
            || persisted.2 != current_required_last_served_epoch
            || persisted.3 != current_required_min_retained_epoch
            || persisted.4 != current_last_served_epoch
            || persisted.5 != current_min_retained_epoch
            || persisted.6 != current.epoch_window_status
            || persisted.7 != current.manifest_action
            || persisted.8 != current.status
        {
            persisted_entry_mismatch_count += 1;
        }
    }
    let current_included_remote_node_count = i64::try_from(summary.included_remote_node_count)
        .expect("included remote node count should fit in i64");
    let current_remote_placement_count = i64::try_from(summary.remote_placement_count)
        .expect("remote placement count should fit in i64");

    let (catalog_status, recommendation) =
        if summary.manifest_decision == "emit_local_epoch_manifest" {
            ("not_required", "none")
        } else if summary.manifest_decision != "emit_distributed_epoch_manifest" {
            (summary.status, summary.recommendation)
        } else if persisted_manifest_count == 0 {
            (
                "requires_remote_epoch_manifest_persistence",
                "persist distributed remote epoch manifest before publishing",
            )
        } else if persisted_entry_count != current_included_remote_node_count
            || persisted_remote_placement_count != current_remote_placement_count
            || persisted_entry_mismatch_count > 0
        {
            (
                "stale_remote_epoch_manifest",
                "refresh persisted remote epoch manifest before publishing",
            )
        } else {
            ("ready", "none")
        };

    TableIterator::once((
        active_epoch,
        summary.manifest_decision,
        current_included_remote_node_count,
        current_remote_placement_count,
        persisted_manifest_count,
        persisted_entry_count,
        persisted_entry_mismatch_count,
        persisted_remote_placement_count,
        catalog_status,
        recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_freshness(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(descriptor_state, String),
        name!(placement_count, i64),
        name!(required_last_served_epoch, i64),
        name!(last_served_epoch, i64),
        name!(required_min_retained_epoch, i64),
        name!(min_retained_epoch, i64),
        name!(epoch_window_status, String),
        name!(manifest_action, String),
        name!(current_status, String),
        name!(persisted_entry_present, bool),
        name!(persisted_entry_matches, bool),
        name!(catalog_status, String),
        name!(publication_action, String),
        name!(publication_status, String),
        name!(freshness_status, String),
        name!(next_action, String),
        name!(recommendation, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_epoch_manifest_freshness") };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let rows = Spi::connect(|client| {
        client
            .select(
                "WITH current_plan AS ( \
                     SELECT active_epoch, node_id, descriptor_state, placement_count, \
                            required_last_served_epoch, required_min_retained_epoch, \
                            last_served_epoch, min_retained_epoch, epoch_window_status, \
                            manifest_action, status AS current_status, recommendation \
                       FROM ec_spire_remote_epoch_manifest_plan($1::oid) \
                 ), publication AS ( \
                     SELECT node_id, persisted_entry_present, persisted_entry_matches, \
                            publication_action, status AS publication_status, recommendation \
                       FROM ec_spire_remote_epoch_manifest_publication_plan($1::oid) \
                 ), catalog AS ( \
                     SELECT current_manifest_decision, catalog_status, recommendation \
                       FROM ec_spire_remote_epoch_manifest_catalog_summary($1::oid) \
                 ) \
                 SELECT p.active_epoch, p.node_id, p.descriptor_state, p.placement_count, \
                        p.required_last_served_epoch, p.last_served_epoch, \
                        p.required_min_retained_epoch, p.min_retained_epoch, \
                        p.epoch_window_status, p.manifest_action, p.current_status, \
                        coalesce(pub.persisted_entry_present, false) AS persisted_entry_present, \
                        coalesce(pub.persisted_entry_matches, false) AS persisted_entry_matches, \
                        c.catalog_status, \
                        coalesce(pub.publication_action, 'block_manifest_publication') \
                            AS publication_action, \
                        coalesce(pub.publication_status, c.catalog_status) AS publication_status, \
                        CASE \
                            WHEN c.current_manifest_decision = 'emit_local_epoch_manifest' \
                                THEN 'not_required' \
                            WHEN p.current_status <> 'ready' THEN p.current_status \
                            WHEN NOT coalesce(pub.persisted_entry_present, false) \
                                THEN 'requires_remote_epoch_manifest_persistence' \
                            WHEN NOT coalesce(pub.persisted_entry_matches, false) \
                                THEN 'stale_remote_epoch_manifest' \
                            WHEN c.catalog_status <> 'ready' THEN c.catalog_status \
                            WHEN coalesce(pub.publication_status, c.catalog_status) <> 'ready' \
                                THEN coalesce(pub.publication_status, c.catalog_status) \
                            ELSE 'ready' \
                        END AS freshness_status, \
                        CASE \
                            WHEN c.current_manifest_decision = 'emit_local_epoch_manifest' \
                                THEN 'none' \
                            WHEN p.current_status <> 'ready' \
                                THEN 'refresh_remote_node_descriptor_or_epoch_window' \
                            WHEN NOT coalesce(pub.persisted_entry_present, false) \
                                THEN 'persist_remote_epoch_manifest' \
                            WHEN NOT coalesce(pub.persisted_entry_matches, false) \
                                THEN 'refresh_remote_epoch_manifest' \
                            WHEN c.catalog_status <> 'ready' \
                                THEN 'resolve_remote_epoch_manifest_catalog_blocker' \
                            WHEN coalesce(pub.publication_status, c.catalog_status) <> 'ready' \
                                THEN 'resolve_remote_epoch_manifest_publication_blocker' \
                            ELSE 'none' \
                        END AS next_action, \
                        CASE \
                            WHEN c.current_manifest_decision = 'emit_local_epoch_manifest' \
                                THEN 'none' \
                            WHEN p.current_status <> 'ready' THEN p.recommendation \
                            WHEN NOT coalesce(pub.persisted_entry_present, false) \
                                THEN 'persist distributed remote epoch manifest before Stage E fixture execution' \
                            WHEN NOT coalesce(pub.persisted_entry_matches, false) \
                                THEN 'refresh persisted remote epoch manifest before Stage E fixture execution' \
                            WHEN c.catalog_status <> 'ready' THEN c.recommendation \
                            WHEN coalesce(pub.publication_status, c.catalog_status) <> 'ready' \
                                THEN coalesce(pub.recommendation, c.recommendation) \
                            ELSE 'none' \
                        END AS recommendation \
                   FROM current_plan p \
                   CROSS JOIN catalog c \
                   LEFT JOIN publication pub ON pub.node_id = p.node_id \
                  ORDER BY p.node_id",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| format!("ec_spire remote epoch manifest freshness read failed: {e}"))?
            .map(|row| {
                Ok::<_, String>((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("freshness active_epoch decode failed: {e}"))?
                        .ok_or_else(|| "freshness active_epoch is null".to_owned())?,
                    row["node_id"]
                        .value::<i64>()
                        .map_err(|e| format!("freshness node_id decode failed: {e}"))?
                        .ok_or_else(|| "freshness node_id is null".to_owned())?,
                    row["descriptor_state"]
                        .value::<String>()
                        .map_err(|e| format!("freshness descriptor_state decode failed: {e}"))?
                        .ok_or_else(|| "freshness descriptor_state is null".to_owned())?,
                    row["placement_count"]
                        .value::<i64>()
                        .map_err(|e| format!("freshness placement_count decode failed: {e}"))?
                        .ok_or_else(|| "freshness placement_count is null".to_owned())?,
                    row["required_last_served_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("freshness required_last_served_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "freshness required_last_served_epoch is null".to_owned()
                        })?,
                    row["last_served_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("freshness last_served_epoch decode failed: {e}"))?
                        .ok_or_else(|| "freshness last_served_epoch is null".to_owned())?,
                    row["required_min_retained_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("freshness required_min_retained_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "freshness required_min_retained_epoch is null".to_owned()
                        })?,
                    row["min_retained_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("freshness min_retained_epoch decode failed: {e}"))?
                        .ok_or_else(|| "freshness min_retained_epoch is null".to_owned())?,
                    row["epoch_window_status"]
                        .value::<String>()
                        .map_err(|e| format!("freshness epoch_window_status decode failed: {e}"))?
                        .ok_or_else(|| "freshness epoch_window_status is null".to_owned())?,
                    row["manifest_action"]
                        .value::<String>()
                        .map_err(|e| format!("freshness manifest_action decode failed: {e}"))?
                        .ok_or_else(|| "freshness manifest_action is null".to_owned())?,
                    row["current_status"]
                        .value::<String>()
                        .map_err(|e| format!("freshness current_status decode failed: {e}"))?
                        .ok_or_else(|| "freshness current_status is null".to_owned())?,
                    row["persisted_entry_present"]
                        .value::<bool>()
                        .map_err(|e| {
                            format!("freshness persisted_entry_present decode failed: {e}")
                        })?
                        .ok_or_else(|| "freshness persisted_entry_present is null".to_owned())?,
                    row["persisted_entry_matches"]
                        .value::<bool>()
                        .map_err(|e| {
                            format!("freshness persisted_entry_matches decode failed: {e}")
                        })?
                        .ok_or_else(|| "freshness persisted_entry_matches is null".to_owned())?,
                    row["catalog_status"]
                        .value::<String>()
                        .map_err(|e| format!("freshness catalog_status decode failed: {e}"))?
                        .ok_or_else(|| "freshness catalog_status is null".to_owned())?,
                    row["publication_action"]
                        .value::<String>()
                        .map_err(|e| format!("freshness publication_action decode failed: {e}"))?
                        .ok_or_else(|| "freshness publication_action is null".to_owned())?,
                    row["publication_status"]
                        .value::<String>()
                        .map_err(|e| format!("freshness publication_status decode failed: {e}"))?
                        .ok_or_else(|| "freshness publication_status is null".to_owned())?,
                    row["freshness_status"]
                        .value::<String>()
                        .map_err(|e| format!("freshness freshness_status decode failed: {e}"))?
                        .ok_or_else(|| "freshness freshness_status is null".to_owned())?,
                    row["next_action"]
                        .value::<String>()
                        .map_err(|e| format!("freshness next_action decode failed: {e}"))?
                        .ok_or_else(|| "freshness next_action is null".to_owned())?,
                    row["recommendation"]
                        .value::<String>()
                        .map_err(|e| format!("freshness recommendation decode failed: {e}"))?
                        .ok_or_else(|| "freshness recommendation is null".to_owned())?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::new(rows.into_iter())
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_publication_plan(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(placement_count, i64),
        name!(persisted_entry_present, bool),
        name!(persisted_entry_matches, bool),
        name!(manifest_action, String),
        name!(publication_action, String),
        name!(publication_transport, String),
        name!(status, String),
        name!(recommendation, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_epoch_manifest_publication_plan")
    };
    let current_rows = unsafe { am::spire_remote_epoch_manifest_plan(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let Some(active_epoch) = current_rows.first().map(|row| row.active_epoch) else {
        return TableIterator::new(Vec::new().into_iter());
    };
    let active_epoch_i64 = i64::try_from(active_epoch).expect("active epoch should fit in i64");

    let (catalog_status, catalog_recommendation) = Spi::connect(|client| {
        client
            .select(
                "SELECT catalog_status, recommendation \
                   FROM ec_spire_remote_epoch_manifest_catalog_summary($1::oid)",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest publication summary read failed: {e}")
            })?
            .map(|row| {
                Ok::<(String, String), String>((
                    row["catalog_status"]
                        .value::<String>()
                        .map_err(|e| format!("publication catalog_status decode failed: {e}"))?
                        .ok_or_else(|| "publication catalog_status is null".to_owned())?,
                    row["recommendation"]
                        .value::<String>()
                        .map_err(|e| format!("publication recommendation decode failed: {e}"))?
                        .ok_or_else(|| "publication recommendation is null".to_owned())?,
                ))
            })
            .next()
            .transpose()
            .map(|value| value.unwrap_or_else(|| ("empty".to_owned(), "none".to_owned())))
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    let persisted_entries = Spi::connect(|client| {
        client
            .select(
                "SELECT node_id, placement_count, required_last_served_epoch, \
                        required_min_retained_epoch, last_served_epoch, min_retained_epoch, \
                        epoch_window_status, manifest_action, status \
                   FROM ec_spire_remote_epoch_manifest_entry \
                  WHERE coordinator_index_oid = $1::oid \
                    AND active_epoch = $2::bigint",
                None,
                &[index_oid.into(), active_epoch_i64.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest publication entry read failed: {e}")
            })?
            .map(|row| {
                Ok::<(i32, i64, i64, i64, i64, i64, String, String, String), String>((
                    row["node_id"]
                        .value::<i32>()
                        .map_err(|e| format!("publication node_id decode failed: {e}"))?
                        .ok_or_else(|| "publication node_id is null".to_owned())?,
                    row["placement_count"]
                        .value::<i64>()
                        .map_err(|e| format!("publication placement_count decode failed: {e}"))?
                        .ok_or_else(|| "publication placement_count is null".to_owned())?,
                    row["required_last_served_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("publication required_last_served_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "publication required_last_served_epoch is null".to_owned()
                        })?,
                    row["required_min_retained_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("publication required_min_retained_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "publication required_min_retained_epoch is null".to_owned()
                        })?,
                    row["last_served_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("publication last_served_epoch decode failed: {e}"))?
                        .ok_or_else(|| "publication last_served_epoch is null".to_owned())?,
                    row["min_retained_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("publication min_retained_epoch decode failed: {e}"))?
                        .ok_or_else(|| "publication min_retained_epoch is null".to_owned())?,
                    row["epoch_window_status"]
                        .value::<String>()
                        .map_err(|e| format!("publication epoch_window_status decode failed: {e}"))?
                        .ok_or_else(|| "publication epoch_window_status is null".to_owned())?,
                    row["manifest_action"]
                        .value::<String>()
                        .map_err(|e| format!("publication manifest_action decode failed: {e}"))?
                        .ok_or_else(|| "publication manifest_action is null".to_owned())?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| format!("publication status decode failed: {e}"))?
                        .ok_or_else(|| "publication status is null".to_owned())?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    let rows = current_rows
        .into_iter()
        .map(|current| {
            let persisted = persisted_entries
                .iter()
                .find(|entry| u32::try_from(entry.0).ok() == Some(current.node_id));
            let persisted_entry_present = persisted.is_some();
            let persisted_entry_matches = persisted.is_some_and(|entry| {
                let placement_count = i64::try_from(current.placement_count)
                    .expect("placement count should fit in i64");
                let required_last_served_epoch = i64::try_from(current.required_last_served_epoch)
                    .expect("required last served epoch should fit in i64");
                let required_min_retained_epoch =
                    i64::try_from(current.required_min_retained_epoch)
                        .expect("required min retained epoch should fit in i64");
                let last_served_epoch = i64::try_from(current.last_served_epoch)
                    .expect("last served epoch should fit in i64");
                let min_retained_epoch = i64::try_from(current.min_retained_epoch)
                    .expect("min retained epoch should fit in i64");
                entry.1 == placement_count
                    && entry.2 == required_last_served_epoch
                    && entry.3 == required_min_retained_epoch
                    && entry.4 == last_served_epoch
                    && entry.5 == min_retained_epoch
                    && entry.6 == current.epoch_window_status
                    && entry.7 == current.manifest_action
                    && entry.8 == current.status
            });
            let (publication_action, publication_transport, status, recommendation) =
                if current.status != "ready" {
                    (
                        "block_manifest_publication".to_owned(),
                        "none".to_owned(),
                        current.status.to_owned(),
                        current.recommendation.to_owned(),
                    )
                } else if catalog_status == "ready" && persisted_entry_matches {
                    (
                        "publish_remote_epoch_manifest".to_owned(),
                        "libpq_pipeline".to_owned(),
                        "ready".to_owned(),
                        "none".to_owned(),
                    )
                } else if catalog_status == "requires_remote_epoch_manifest_persistence" {
                    (
                        "persist_remote_epoch_manifest".to_owned(),
                        "none".to_owned(),
                        catalog_status.clone(),
                        catalog_recommendation.clone(),
                    )
                } else if catalog_status == "stale_remote_epoch_manifest" {
                    (
                        "refresh_remote_epoch_manifest".to_owned(),
                        "none".to_owned(),
                        catalog_status.clone(),
                        catalog_recommendation.clone(),
                    )
                } else {
                    (
                        "block_manifest_publication".to_owned(),
                        "none".to_owned(),
                        catalog_status.clone(),
                        catalog_recommendation.clone(),
                    )
                };

            (
                active_epoch_i64,
                i64::from(current.node_id),
                i64::try_from(current.placement_count).expect("placement count should fit in i64"),
                persisted_entry_present,
                persisted_entry_matches,
                current.manifest_action.to_owned(),
                publication_action,
                publication_transport,
                status,
                recommendation,
            )
        })
        .collect::<Vec<_>>();

    TableIterator::new(rows.into_iter())
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_publication_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(publication_scope, String),
        name!(publication_decision, String),
        name!(publication_entry_count, i64),
        name!(ready_publication_count, i64),
        name!(persistence_required_count, i64),
        name!(refresh_required_count, i64),
        name!(blocked_publication_count, i64),
        name!(remote_placement_count, i64),
        name!(publication_executor_status, String),
        name!(publication_executor_next_step, String),
        name!(next_blocker, String),
        name!(status, String),
        name!(recommendation, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_epoch_manifest_publication_summary",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let (active_epoch, current_manifest_decision, catalog_status, catalog_recommendation) =
        Spi::connect(|client| {
            client
                .select(
                    "SELECT active_epoch, current_manifest_decision, catalog_status, recommendation \
                       FROM ec_spire_remote_epoch_manifest_catalog_summary($1::oid)",
                    None,
                    &[index_oid.into()],
                )
                .map_err(|e| {
                    format!(
                        "ec_spire remote epoch manifest publication summary catalog read failed: {e}"
                    )
                })?
                .map(|row| {
                    Ok::<(i64, String, String, String), String>((
                        row["active_epoch"]
                            .value::<i64>()
                            .map_err(|e| format!("publication summary active_epoch decode failed: {e}"))?
                            .ok_or_else(|| "publication summary active_epoch is null".to_owned())?,
                        row["current_manifest_decision"]
                            .value::<String>()
                            .map_err(|e| {
                                format!(
                                    "publication summary current_manifest_decision decode failed: {e}"
                                )
                            })?
                            .ok_or_else(|| {
                                "publication summary current_manifest_decision is null".to_owned()
                            })?,
                        row["catalog_status"]
                            .value::<String>()
                            .map_err(|e| {
                                format!("publication summary catalog_status decode failed: {e}")
                            })?
                            .ok_or_else(|| {
                                "publication summary catalog_status is null".to_owned()
                            })?,
                        row["recommendation"]
                            .value::<String>()
                            .map_err(|e| {
                                format!("publication summary recommendation decode failed: {e}")
                            })?
                            .ok_or_else(|| {
                                "publication summary recommendation is null".to_owned()
                            })?,
                    ))
                })
                .next()
                .transpose()
                .map(|value| {
                    value.unwrap_or_else(|| {
                        (0, "build_required".to_owned(), "empty".to_owned(), "build index before remote epoch publication".to_owned())
                    })
                })
        })
        .unwrap_or_else(|e| pgrx::error!("{e}"));

    let (
        publication_entry_count,
        ready_publication_count,
        persistence_required_count,
        refresh_required_count,
        blocked_publication_count,
        remote_placement_count,
    ) = Spi::connect(|client| {
        client
            .select(
                "SELECT count(*)::bigint AS publication_entry_count, \
                        count(*) FILTER (WHERE status = 'ready')::bigint AS ready_publication_count, \
                        count(*) FILTER (WHERE status = 'requires_remote_epoch_manifest_persistence')::bigint \
                            AS persistence_required_count, \
                        count(*) FILTER (WHERE status = 'stale_remote_epoch_manifest')::bigint \
                            AS refresh_required_count, \
                        count(*) FILTER (WHERE status NOT IN ('ready', \
                            'requires_remote_epoch_manifest_persistence', \
                            'stale_remote_epoch_manifest'))::bigint AS blocked_publication_count, \
                        coalesce(sum(placement_count), 0)::bigint AS remote_placement_count \
                   FROM ec_spire_remote_epoch_manifest_publication_plan($1::oid)",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!(
                    "ec_spire remote epoch manifest publication summary plan read failed: {e}"
                )
            })?
            .map(|row| {
                Ok::<(i64, i64, i64, i64, i64, i64), String>((
                    row["publication_entry_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("publication_entry_count decode failed: {e}")
                        })?
                        .ok_or_else(|| "publication_entry_count is null".to_owned())?,
                    row["ready_publication_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("ready_publication_count decode failed: {e}")
                        })?
                        .ok_or_else(|| "ready_publication_count is null".to_owned())?,
                    row["persistence_required_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("persistence_required_count decode failed: {e}")
                        })?
                        .ok_or_else(|| "persistence_required_count is null".to_owned())?,
                    row["refresh_required_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("refresh_required_count decode failed: {e}")
                        })?
                        .ok_or_else(|| "refresh_required_count is null".to_owned())?,
                    row["blocked_publication_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("blocked_publication_count decode failed: {e}")
                        })?
                        .ok_or_else(|| "blocked_publication_count is null".to_owned())?,
                    row["remote_placement_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("remote_placement_count decode failed: {e}")
                        })?
                        .ok_or_else(|| "remote_placement_count is null".to_owned())?,
                ))
            })
            .next()
            .transpose()
            .map(|value| value.unwrap_or((0, 0, 0, 0, 0, 0)))
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    let (publication_scope, publication_decision, next_blocker, status, recommendation) =
        if active_epoch == 0 {
            (
                "empty".to_owned(),
                "build_required".to_owned(),
                "build_index".to_owned(),
                catalog_status,
                catalog_recommendation,
            )
        } else if current_manifest_decision == "emit_local_epoch_manifest" {
            (
                "local_only".to_owned(),
                "not_required".to_owned(),
                "none".to_owned(),
                "not_required".to_owned(),
                "none".to_owned(),
            )
        } else if ready_publication_count == publication_entry_count
            && publication_entry_count > 0
            && catalog_status == "ready"
        {
            (
                "distributed".to_owned(),
                "publish_remote_epoch_manifest".to_owned(),
                "none".to_owned(),
                "ready".to_owned(),
                "none".to_owned(),
            )
        } else if persistence_required_count > 0
            || catalog_status == "requires_remote_epoch_manifest_persistence"
        {
            (
                "distributed".to_owned(),
                "persist_remote_epoch_manifest".to_owned(),
                "remote_epoch_manifest_persistence".to_owned(),
                "requires_remote_epoch_manifest_persistence".to_owned(),
                "persist distributed remote epoch manifest before publishing".to_owned(),
            )
        } else if refresh_required_count > 0 || catalog_status == "stale_remote_epoch_manifest" {
            (
                "distributed".to_owned(),
                "refresh_remote_epoch_manifest".to_owned(),
                "remote_epoch_manifest_refresh".to_owned(),
                "stale_remote_epoch_manifest".to_owned(),
                "refresh persisted remote epoch manifest before publishing".to_owned(),
            )
        } else {
            (
                "distributed".to_owned(),
                "block_manifest_publication".to_owned(),
                "remote_epoch_publish_gate".to_owned(),
                catalog_status,
                catalog_recommendation,
            )
        };
    let (publication_executor_status, publication_executor_next_step) =
        if publication_decision == "publish_remote_epoch_manifest" {
            (
                "requires_libpq_executor".to_owned(),
                "conninfo_secret_resolution".to_owned(),
            )
        } else {
            ("none".to_owned(), "none".to_owned())
        };

    TableIterator::once((
        active_epoch,
        publication_scope,
        publication_decision,
        publication_entry_count,
        ready_publication_count,
        persistence_required_count,
        refresh_required_count,
        blocked_publication_count,
        remote_placement_count,
        publication_executor_status,
        publication_executor_next_step,
        next_blocker,
        status,
        recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_libpq_request_plan(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(conninfo_secret_name, String),
        name!(remote_index_regclass, String),
        name!(remote_index_source, String),
        name!(manifest_payload_source, String),
        name!(manifest_payload_format, String),
        name!(sql_template, String),
        name!(parameter_count, i64),
        name!(expected_result_column_count, i64),
        name!(publication_transport, String),
        name!(request_action, String),
        name!(executor_status, String),
        name!(executor_next_step, String),
        name!(status, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_epoch_manifest_libpq_request_plan",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let rows = Spi::connect(|client| {
        client
            .select(
                "SELECT p.active_epoch, p.node_id, \
                        coalesce(d.conninfo_secret_name, 'none') AS conninfo_secret_name, \
                        coalesce(d.remote_index_regclass, 'none') AS remote_index_regclass, \
                        p.publication_transport, p.status \
                   FROM ec_spire_remote_epoch_manifest_publication_plan($1::oid) p \
                   LEFT JOIN ec_spire_remote_node_descriptor d \
                     ON d.coordinator_index_oid = $1::oid \
                    AND d.node_id = p.node_id::integer \
                  ORDER BY p.node_id",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest libpq request plan read failed: {e}")
            })?
            .map(|row| {
                let publication_transport = row["publication_transport"]
                    .value::<String>()
                    .map_err(|e| format!("publication_transport decode failed: {e}"))?
                    .ok_or_else(|| "publication_transport is null".to_owned())?;
                let status = row["status"]
                    .value::<String>()
                    .map_err(|e| format!("publication status decode failed: {e}"))?
                    .ok_or_else(|| "publication status is null".to_owned())?;
                let (request_action, executor_status, executor_next_step) =
                    if status == "ready" && publication_transport == "libpq_pipeline" {
                        (
                            "send_remote_epoch_manifest".to_owned(),
                            "requires_libpq_executor".to_owned(),
                            "conninfo_secret_resolution".to_owned(),
                        )
                    } else {
                        (
                            "blocked_before_manifest_publication".to_owned(),
                            "none".to_owned(),
                            "none".to_owned(),
                        )
                    };
                Ok::<_, String>((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("request active_epoch decode failed: {e}"))?
                        .ok_or_else(|| "request active_epoch is null".to_owned())?,
                    row["node_id"]
                        .value::<i64>()
                        .map_err(|e| format!("request node_id decode failed: {e}"))?
                        .ok_or_else(|| "request node_id is null".to_owned())?,
                    row["conninfo_secret_name"]
                        .value::<String>()
                        .map_err(|e| format!("request conninfo_secret_name decode failed: {e}"))?
                        .ok_or_else(|| "request conninfo_secret_name is null".to_owned())?,
                    row["remote_index_regclass"]
                        .value::<String>()
                        .map_err(|e| format!("request remote_index_regclass decode failed: {e}"))?
                        .ok_or_else(|| "request remote_index_regclass is null".to_owned())?,
                    "remote_node_descriptor".to_owned(),
                    "ec_spire_remote_epoch_manifest_catalog".to_owned(),
                    "ec_spire_remote_epoch_manifest_v1".to_owned(),
                    "SELECT * FROM ec_spire_apply_remote_epoch_manifest_payload($1::oid, $2::bigint, $3::jsonb)"
                        .to_owned(),
                    3_i64,
                    3_i64,
                    publication_transport,
                    request_action,
                    executor_status,
                    executor_next_step,
                    status,
                ))
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::new(rows.into_iter())
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_libpq_request_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(request_count, i64),
        name!(ready_request_count, i64),
        name!(blocked_request_count, i64),
        name!(parameter_count_per_request, i64),
        name!(expected_result_column_count, i64),
        name!(publication_executor_status, String),
        name!(publication_executor_next_step, String),
        name!(status, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_epoch_manifest_libpq_request_summary",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let row = Spi::connect(|client| {
        client
            .select(
                "WITH request_rows AS ( \
                        SELECT * \
                          FROM ec_spire_remote_epoch_manifest_libpq_request_plan($1::oid) \
                    ), publication_summary AS ( \
                        SELECT * \
                          FROM ec_spire_remote_epoch_manifest_publication_summary($1::oid) \
                    ) \
                  SELECT s.active_epoch, \
                         count(r.active_epoch)::bigint AS request_count, \
                         count(*) FILTER ( \
                             WHERE r.request_action = 'send_remote_epoch_manifest' \
                               AND r.status = 'ready')::bigint AS ready_request_count, \
                         count(*) FILTER ( \
                             WHERE r.active_epoch IS NOT NULL \
                               AND (r.request_action <> 'send_remote_epoch_manifest' \
                                    OR r.status <> 'ready'))::bigint AS blocked_request_count, \
                         coalesce(max(r.parameter_count), 0)::bigint \
                             AS parameter_count_per_request, \
                         coalesce(max(r.expected_result_column_count), 0)::bigint \
                             AS expected_result_column_count, \
                         s.publication_executor_status, \
                         s.publication_executor_next_step, \
                         CASE \
                             WHEN count(r.active_epoch) = 0 THEN s.status \
                             WHEN bool_and(r.request_action = 'send_remote_epoch_manifest' \
                                           AND r.status = 'ready') THEN 'ready' \
                             ELSE 'blocked' \
                         END AS status \
                    FROM publication_summary s \
                    LEFT JOIN request_rows r ON true \
                   GROUP BY s.active_epoch, s.status, \
                            s.publication_executor_status, \
                            s.publication_executor_next_step",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest libpq request summary read failed: {e}")
            })?
            .map(|row| {
                Ok::<_, String>((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("request summary active_epoch decode failed: {e}"))?
                        .ok_or_else(|| "request summary active_epoch is null".to_owned())?,
                    row["request_count"]
                        .value::<i64>()
                        .map_err(|e| format!("request summary request_count decode failed: {e}"))?
                        .ok_or_else(|| "request summary request_count is null".to_owned())?,
                    row["ready_request_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("request summary ready_request_count decode failed: {e}")
                        })?
                        .ok_or_else(|| "request summary ready_request_count is null".to_owned())?,
                    row["blocked_request_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("request summary blocked_request_count decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "request summary blocked_request_count is null".to_owned()
                        })?,
                    row["parameter_count_per_request"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "request summary parameter_count_per_request decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "request summary parameter_count_per_request is null".to_owned()
                        })?,
                    row["expected_result_column_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "request summary expected_result_column_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "request summary expected_result_column_count is null".to_owned()
                        })?,
                    row["publication_executor_status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!(
                                "request summary publication_executor_status decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "request summary publication_executor_status is null".to_owned()
                        })?,
                    row["publication_executor_next_step"]
                        .value::<String>()
                        .map_err(|e| {
                            format!(
                                "request summary publication_executor_next_step decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "request summary publication_executor_next_step is null".to_owned()
                        })?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| format!("request summary status decode failed: {e}"))?
                        .ok_or_else(|| "request summary status is null".to_owned())?,
                ))
            })
            .next()
            .transpose()?
            .ok_or_else(|| {
                "ec_spire remote epoch manifest libpq request summary returned no rows".to_owned()
            })
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::once(row)
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_payload_plan(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(manifest_payload_format, String),
        name!(manifest_payload, pgrx::JsonB),
        name!(status, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_epoch_manifest_payload_plan")
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let rows = Spi::connect(|client| {
        client
            .select(
                "SELECT e.active_epoch, e.node_id, \
                        'ec_spire_remote_epoch_manifest_v1'::text AS manifest_payload_format, \
                        jsonb_build_object( \
                            'manifest_payload_format', 'ec_spire_remote_epoch_manifest_v1', \
                            'active_epoch', e.active_epoch, \
                            'manifest_scope', m.manifest_scope, \
                            'manifest_decision', m.manifest_decision, \
                            'manifest_entry_count', m.manifest_entry_count, \
                            'included_remote_node_count', m.included_remote_node_count, \
                            'remote_placement_count', m.remote_placement_count, \
                            'publish_decision', m.publish_decision, \
                            'entries', jsonb_build_array(jsonb_build_object( \
                                'node_id', e.node_id, \
                                'descriptor_state', e.descriptor_state, \
                                'placement_count', e.placement_count, \
                                'required_last_served_epoch', e.required_last_served_epoch, \
                                'required_min_retained_epoch', e.required_min_retained_epoch, \
                                'last_served_epoch', e.last_served_epoch, \
                                'min_retained_epoch', e.min_retained_epoch, \
                                'epoch_window_status', e.epoch_window_status, \
                                'manifest_action', e.manifest_action, \
                                'status', e.status))) AS manifest_payload, \
                        CASE \
                            WHEN r.request_action = 'send_remote_epoch_manifest' \
                             AND r.status = 'ready' THEN 'ready' \
                            ELSE r.status \
                        END AS status \
                   FROM ec_spire_remote_epoch_manifest_libpq_request_plan($1::oid) r \
                   JOIN ec_spire_remote_epoch_manifest m \
                     ON m.coordinator_index_oid = $1::oid \
                    AND m.active_epoch = r.active_epoch \
                   JOIN ec_spire_remote_epoch_manifest_entry e \
                     ON e.coordinator_index_oid = m.coordinator_index_oid \
                    AND e.active_epoch = m.active_epoch \
                    AND e.node_id = r.node_id::integer \
                  ORDER BY e.node_id",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| format!("ec_spire remote epoch manifest payload plan read failed: {e}"))?
            .map(|row| {
                Ok::<_, String>((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("payload active_epoch decode failed: {e}"))?
                        .ok_or_else(|| "payload active_epoch is null".to_owned())?,
                    row["node_id"]
                        .value::<i64>()
                        .map_err(|e| format!("payload node_id decode failed: {e}"))?
                        .ok_or_else(|| "payload node_id is null".to_owned())?,
                    row["manifest_payload_format"]
                        .value::<String>()
                        .map_err(|e| format!("payload manifest_payload_format decode failed: {e}"))?
                        .ok_or_else(|| "payload manifest_payload_format is null".to_owned())?,
                    row["manifest_payload"]
                        .value::<pgrx::JsonB>()
                        .map_err(|e| format!("payload manifest_payload decode failed: {e}"))?
                        .ok_or_else(|| "payload manifest_payload is null".to_owned())?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| format!("payload status decode failed: {e}"))?
                        .ok_or_else(|| "payload status is null".to_owned())?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::new(rows.into_iter())
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_payload_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(payload_count, i64),
        name!(ready_payload_count, i64),
        name!(blocked_payload_count, i64),
        name!(manifest_payload_format, String),
        name!(status, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_epoch_manifest_payload_summary")
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let row = Spi::connect(|client| {
        client
            .select(
                "WITH payload_rows AS ( \
                        SELECT * \
                          FROM ec_spire_remote_epoch_manifest_payload_plan($1::oid) \
                    ), publication_summary AS ( \
                        SELECT * \
                          FROM ec_spire_remote_epoch_manifest_publication_summary($1::oid) \
                    ) \
                  SELECT s.active_epoch, \
                         count(p.active_epoch)::bigint AS payload_count, \
                         count(*) FILTER (WHERE p.status = 'ready')::bigint AS ready_payload_count, \
                         count(*) FILTER (WHERE p.active_epoch IS NOT NULL \
                                           AND p.status <> 'ready')::bigint AS blocked_payload_count, \
                         coalesce(max(p.manifest_payload_format), 'none') AS manifest_payload_format, \
                         CASE \
                             WHEN count(p.active_epoch) = 0 THEN s.status \
                             WHEN bool_and(p.status = 'ready') THEN 'ready' \
                             ELSE 'blocked' \
                         END AS status \
                    FROM publication_summary s \
                    LEFT JOIN payload_rows p ON true \
                   GROUP BY s.active_epoch, s.status",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest payload summary read failed: {e}")
            })?
            .map(|row| {
                Ok::<_, String>((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("payload summary active_epoch decode failed: {e}"))?
                        .ok_or_else(|| "payload summary active_epoch is null".to_owned())?,
                    row["payload_count"]
                        .value::<i64>()
                        .map_err(|e| format!("payload summary payload_count decode failed: {e}"))?
                        .ok_or_else(|| "payload summary payload_count is null".to_owned())?,
                    row["ready_payload_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("payload summary ready_payload_count decode failed: {e}")
                        })?
                        .ok_or_else(|| "payload summary ready_payload_count is null".to_owned())?,
                    row["blocked_payload_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("payload summary blocked_payload_count decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "payload summary blocked_payload_count is null".to_owned()
                        })?,
                    row["manifest_payload_format"]
                        .value::<String>()
                        .map_err(|e| {
                            format!(
                                "payload summary manifest_payload_format decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "payload summary manifest_payload_format is null".to_owned()
                        })?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| format!("payload summary status decode failed: {e}"))?
                        .ok_or_else(|| "payload summary status is null".to_owned())?,
                ))
            })
            .next()
            .transpose()?
            .ok_or_else(|| {
                "ec_spire remote epoch manifest payload summary returned no rows".to_owned()
            })
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::once(row)
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_validate_remote_epoch_manifest_payload(
    remote_index_oid: pg_sys::Oid,
    active_epoch: i64,
    manifest_payload: pgrx::JsonB,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(validated_entry_count, i64),
        name!(status, String),
    ),
> {
    if active_epoch <= 0 {
        pgrx::error!(
            "ec_spire_validate_remote_epoch_manifest_payload active_epoch must be greater than 0"
        );
    }
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            remote_index_oid,
            "ec_spire_validate_remote_epoch_manifest_payload",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let payload = manifest_payload.0;
    let Some(object) = payload.as_object() else {
        return TableIterator::once((active_epoch, 0, "invalid_manifest_payload".to_owned()));
    };
    let payload_format = object
        .get("manifest_payload_format")
        .and_then(|value| value.as_str())
        .unwrap_or("none");
    if payload_format != "ec_spire_remote_epoch_manifest_v1" {
        return TableIterator::once((
            active_epoch,
            0,
            "invalid_manifest_payload_format".to_owned(),
        ));
    }
    let Some(payload_epoch) = object.get("active_epoch").and_then(|value| value.as_i64()) else {
        return TableIterator::once((active_epoch, 0, "missing_manifest_epoch".to_owned()));
    };
    if payload_epoch != active_epoch {
        return TableIterator::once((active_epoch, 0, "manifest_epoch_mismatch".to_owned()));
    }
    let Some(entries) = object.get("entries").and_then(|value| value.as_array()) else {
        return TableIterator::once((active_epoch, 0, "missing_manifest_entries".to_owned()));
    };
    if entries.is_empty() {
        return TableIterator::once((active_epoch, 0, "empty_manifest_entries".to_owned()));
    }

    let mut validated_entry_count = 0_i64;
    for entry in entries {
        let Some(entry_object) = entry.as_object() else {
            return TableIterator::once((active_epoch, 0, "invalid_manifest_entry".to_owned()));
        };
        let Some(node_id) = entry_object.get("node_id").and_then(|value| value.as_i64()) else {
            return TableIterator::once((
                active_epoch,
                0,
                "missing_manifest_entry_node".to_owned(),
            ));
        };
        if node_id <= 0 {
            return TableIterator::once((
                active_epoch,
                0,
                "invalid_manifest_entry_node".to_owned(),
            ));
        }
        if entry_object
            .get("manifest_action")
            .and_then(|value| value.as_str())
            != Some("include_remote_node")
        {
            return TableIterator::once((
                active_epoch,
                0,
                "invalid_manifest_entry_action".to_owned(),
            ));
        }
        if entry_object.get("status").and_then(|value| value.as_str()) != Some("ready") {
            return TableIterator::once((active_epoch, 0, "manifest_entry_not_ready".to_owned()));
        }
        validated_entry_count += 1;
    }

    TableIterator::once((active_epoch, validated_entry_count, "ready".to_owned()))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_apply_remote_epoch_manifest_payload(
    remote_index_oid: pg_sys::Oid,
    active_epoch: i64,
    manifest_payload: pgrx::JsonB,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(validated_entry_count, i64),
        name!(status, String),
    ),
> {
    if active_epoch <= 0 {
        pgrx::error!(
            "ec_spire_apply_remote_epoch_manifest_payload active_epoch must be greater than 0"
        );
    }
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            remote_index_oid,
            "ec_spire_apply_remote_epoch_manifest_payload",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let payload = manifest_payload.0;
    let Some(object) = payload.as_object() else {
        return TableIterator::once((active_epoch, 0, "invalid_manifest_payload".to_owned()));
    };
    let payload_format = object
        .get("manifest_payload_format")
        .and_then(|value| value.as_str())
        .unwrap_or("none");
    if payload_format != "ec_spire_remote_epoch_manifest_v1" {
        return TableIterator::once((
            active_epoch,
            0,
            "invalid_manifest_payload_format".to_owned(),
        ));
    }
    let Some(payload_epoch) = object.get("active_epoch").and_then(|value| value.as_i64()) else {
        return TableIterator::once((active_epoch, 0, "missing_manifest_epoch".to_owned()));
    };
    if payload_epoch != active_epoch {
        return TableIterator::once((active_epoch, 0, "manifest_epoch_mismatch".to_owned()));
    }
    let Some(entries) = object.get("entries").and_then(|value| value.as_array()) else {
        return TableIterator::once((active_epoch, 0, "missing_manifest_entries".to_owned()));
    };
    if entries.is_empty() {
        return TableIterator::once((active_epoch, 0, "empty_manifest_entries".to_owned()));
    }

    let manifest_scope = object
        .get("manifest_scope")
        .and_then(|value| value.as_str())
        .unwrap_or("none")
        .to_owned();
    let manifest_decision = object
        .get("manifest_decision")
        .and_then(|value| value.as_str())
        .unwrap_or("none")
        .to_owned();
    let manifest_entry_count = object
        .get("manifest_entry_count")
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    let included_remote_node_count = object
        .get("included_remote_node_count")
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    let remote_placement_count = object
        .get("remote_placement_count")
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    let publish_decision = object
        .get("publish_decision")
        .and_then(|value| value.as_str())
        .unwrap_or("none")
        .to_owned();

    let mut parsed_entries = Vec::new();
    for entry in entries {
        let Some(entry_object) = entry.as_object() else {
            return TableIterator::once((active_epoch, 0, "invalid_manifest_entry".to_owned()));
        };
        let Some(node_id) = entry_object.get("node_id").and_then(|value| value.as_i64()) else {
            return TableIterator::once((
                active_epoch,
                0,
                "missing_manifest_entry_node".to_owned(),
            ));
        };
        if node_id <= 0 {
            return TableIterator::once((
                active_epoch,
                0,
                "invalid_manifest_entry_node".to_owned(),
            ));
        }
        let descriptor_state = entry_object
            .get("descriptor_state")
            .and_then(|value| value.as_str())
            .unwrap_or("none")
            .to_owned();
        let placement_count = entry_object
            .get("placement_count")
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        let required_last_served_epoch = entry_object
            .get("required_last_served_epoch")
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        let required_min_retained_epoch = entry_object
            .get("required_min_retained_epoch")
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        let last_served_epoch = entry_object
            .get("last_served_epoch")
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        let min_retained_epoch = entry_object
            .get("min_retained_epoch")
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        let epoch_window_status = entry_object
            .get("epoch_window_status")
            .and_then(|value| value.as_str())
            .unwrap_or("none")
            .to_owned();
        let manifest_action = entry_object
            .get("manifest_action")
            .and_then(|value| value.as_str())
            .unwrap_or("none")
            .to_owned();
        if manifest_action != "include_remote_node" {
            return TableIterator::once((
                active_epoch,
                0,
                "invalid_manifest_entry_action".to_owned(),
            ));
        }
        let status = entry_object
            .get("status")
            .and_then(|value| value.as_str())
            .unwrap_or("none")
            .to_owned();
        if status != "ready" {
            return TableIterator::once((active_epoch, 0, "manifest_entry_not_ready".to_owned()));
        }
        parsed_entries.push((
            i32::try_from(node_id).unwrap_or_else(|_| {
                pgrx::error!("ec_spire apply remote epoch manifest node_id exceeds integer")
            }),
            descriptor_state,
            placement_count,
            required_last_served_epoch,
            required_min_retained_epoch,
            last_served_epoch,
            min_retained_epoch,
            epoch_window_status,
            manifest_action,
            status,
        ));
    }

    let applied_entry_count =
        i64::try_from(parsed_entries.len()).expect("applied entry count should fit in i64");
    let result = Spi::connect_mut(|client| {
        client
            .update(
                "INSERT INTO ec_spire_remote_epoch_manifest_applied \
                 (remote_index_oid, active_epoch, manifest_payload_format, manifest_scope, \
                  manifest_decision, manifest_entry_count, included_remote_node_count, \
                  remote_placement_count, publish_decision, status, applied_at_micros) \
                 VALUES ($1::oid, $2::bigint, $3::text, $4::text, $5::text, $6::bigint, \
                         $7::bigint, $8::bigint, $9::text, $10::text, \
                         (extract(epoch from clock_timestamp()) * 1000000)::bigint) \
                 ON CONFLICT (remote_index_oid, active_epoch) DO UPDATE SET \
                     manifest_payload_format = EXCLUDED.manifest_payload_format, \
                     manifest_scope = EXCLUDED.manifest_scope, \
                     manifest_decision = EXCLUDED.manifest_decision, \
                     manifest_entry_count = EXCLUDED.manifest_entry_count, \
                     included_remote_node_count = EXCLUDED.included_remote_node_count, \
                     remote_placement_count = EXCLUDED.remote_placement_count, \
                     publish_decision = EXCLUDED.publish_decision, \
                     status = EXCLUDED.status, \
                     applied_at_micros = EXCLUDED.applied_at_micros",
                None,
                &[
                    remote_index_oid.into(),
                    active_epoch.into(),
                    payload_format.into(),
                    manifest_scope.into(),
                    manifest_decision.into(),
                    manifest_entry_count.into(),
                    included_remote_node_count.into(),
                    remote_placement_count.into(),
                    publish_decision.into(),
                    "ready".into(),
                ],
            )
            .map_err(|e| format!("ec_spire remote epoch manifest applied header failed: {e}"))?;
        client
            .update(
                "DELETE FROM ec_spire_remote_epoch_manifest_applied_entry \
                  WHERE remote_index_oid = $1::oid AND active_epoch = $2::bigint",
                None,
                &[remote_index_oid.into(), active_epoch.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest applied entry replace failed: {e}")
            })?;

        for entry in parsed_entries {
            client
                .update(
                    "INSERT INTO ec_spire_remote_epoch_manifest_applied_entry \
                     (remote_index_oid, active_epoch, node_id, descriptor_state, \
                      placement_count, required_last_served_epoch, required_min_retained_epoch, \
                      last_served_epoch, min_retained_epoch, epoch_window_status, \
                      manifest_action, status) \
                     VALUES ($1::oid, $2::bigint, $3::integer, $4::text, $5::bigint, \
                             $6::bigint, $7::bigint, $8::bigint, $9::bigint, $10::text, \
                             $11::text, $12::text)",
                    None,
                    &[
                        remote_index_oid.into(),
                        active_epoch.into(),
                        entry.0.into(),
                        entry.1.into(),
                        entry.2.into(),
                        entry.3.into(),
                        entry.4.into(),
                        entry.5.into(),
                        entry.6.into(),
                        entry.7.into(),
                        entry.8.into(),
                        entry.9.into(),
                    ],
                )
                .map_err(|e| {
                    format!(
                        "ec_spire remote epoch manifest applied entry failed for node_id {}: {e}",
                        entry.0
                    )
                })?;
        }
        Ok::<(), String>(())
    });
    result.unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::once((active_epoch, applied_entry_count, "ready".to_owned()))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_libpq_dispatch_plan(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(conninfo_secret_name, String),
        name!(remote_index_regclass, String),
        name!(sql_template, String),
        name!(parameter_count, i64),
        name!(expected_result_column_count, i64),
        name!(manifest_payload_format, String),
        name!(manifest_payload, pgrx::JsonB),
        name!(dispatch_action, String),
        name!(receive_validator, String),
        name!(executor_status, String),
        name!(status, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_epoch_manifest_libpq_dispatch_plan",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let rows = Spi::connect(|client| {
        client
            .select(
                "SELECT r.active_epoch, r.node_id, r.conninfo_secret_name, \
                        r.remote_index_regclass, r.sql_template, r.parameter_count, \
                        r.expected_result_column_count, p.manifest_payload_format, \
                        p.manifest_payload, \
                        CASE \
                            WHEN r.request_action = 'send_remote_epoch_manifest' \
                             AND r.status = 'ready' \
                             AND p.status = 'ready' \
                            THEN 'open_pipeline_and_send_remote_epoch_manifest' \
                            ELSE 'blocked_before_manifest_dispatch' \
                        END AS dispatch_action, \
                        'ec_spire_remote_epoch_manifest_libpq_result_contract'::text \
                            AS receive_validator, \
                        CASE \
                            WHEN r.request_action = 'send_remote_epoch_manifest' \
                             AND r.status = 'ready' \
                             AND p.status = 'ready' \
                            THEN 'requires_libpq_executor' \
                            ELSE 'none' \
                        END AS executor_status, \
                        CASE \
                            WHEN r.status <> 'ready' THEN r.status \
                            ELSE p.status \
                        END AS status \
                   FROM ec_spire_remote_epoch_manifest_libpq_request_plan($1::oid) r \
                   JOIN ec_spire_remote_epoch_manifest_payload_plan($1::oid) p \
                     ON p.active_epoch = r.active_epoch \
                    AND p.node_id = r.node_id \
                  ORDER BY r.node_id",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest libpq dispatch plan read failed: {e}")
            })?
            .map(|row| {
                Ok::<_, String>((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest dispatch active_epoch decode failed: {e}"))?
                        .ok_or_else(|| "manifest dispatch active_epoch is null".to_owned())?,
                    row["node_id"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest dispatch node_id decode failed: {e}"))?
                        .ok_or_else(|| "manifest dispatch node_id is null".to_owned())?,
                    row["conninfo_secret_name"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest dispatch conninfo_secret_name decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest dispatch conninfo_secret_name is null".to_owned()
                        })?,
                    row["remote_index_regclass"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest dispatch remote_index_regclass decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest dispatch remote_index_regclass is null".to_owned()
                        })?,
                    row["sql_template"]
                        .value::<String>()
                        .map_err(|e| format!("manifest dispatch sql_template decode failed: {e}"))?
                        .ok_or_else(|| "manifest dispatch sql_template is null".to_owned())?,
                    row["parameter_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest dispatch parameter_count decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest dispatch parameter_count is null".to_owned())?,
                    row["expected_result_column_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest dispatch expected_result_column_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest dispatch expected_result_column_count is null".to_owned()
                        })?,
                    row["manifest_payload_format"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest dispatch manifest_payload_format decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest dispatch manifest_payload_format is null".to_owned()
                        })?,
                    row["manifest_payload"]
                        .value::<pgrx::JsonB>()
                        .map_err(|e| {
                            format!("manifest dispatch manifest_payload decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest dispatch manifest_payload is null".to_owned())?,
                    row["dispatch_action"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest dispatch dispatch_action decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest dispatch dispatch_action is null".to_owned())?,
                    row["receive_validator"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest dispatch receive_validator decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest dispatch receive_validator is null".to_owned())?,
                    row["executor_status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest dispatch executor_status decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest dispatch executor_status is null".to_owned())?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| format!("manifest dispatch status decode failed: {e}"))?
                        .ok_or_else(|| "manifest dispatch status is null".to_owned())?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::new(rows.into_iter())
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_libpq_bind_plan(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(parameter_ordinal, i64),
        name!(parameter_name, &'static str),
        name!(pg_type, &'static str),
        name!(value_source, &'static str),
        name!(value_status, String),
        name!(value_preview, String),
        name!(element_count, i64),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_epoch_manifest_libpq_bind_plan")
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let rows = Spi::connect(|client| {
        client
            .select(
                "SELECT active_epoch, node_id, remote_index_regclass, \
                        manifest_payload_format, manifest_payload, dispatch_action, status \
                   FROM ec_spire_remote_epoch_manifest_libpq_dispatch_plan($1::oid) \
                  ORDER BY node_id",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest libpq bind plan read failed: {e}")
            })?
            .map(|row| {
                let active_epoch = row["active_epoch"]
                    .value::<i64>()
                    .map_err(|e| format!("manifest bind active_epoch decode failed: {e}"))?
                    .ok_or_else(|| "manifest bind active_epoch is null".to_owned())?;
                let node_id = row["node_id"]
                    .value::<i64>()
                    .map_err(|e| format!("manifest bind node_id decode failed: {e}"))?
                    .ok_or_else(|| "manifest bind node_id is null".to_owned())?;
                let remote_index_regclass = row["remote_index_regclass"]
                    .value::<String>()
                    .map_err(|e| format!("manifest bind remote_index_regclass decode failed: {e}"))?
                    .ok_or_else(|| "manifest bind remote_index_regclass is null".to_owned())?;
                let manifest_payload_format = row["manifest_payload_format"]
                    .value::<String>()
                    .map_err(|e| {
                        format!("manifest bind manifest_payload_format decode failed: {e}")
                    })?
                    .ok_or_else(|| "manifest bind manifest_payload_format is null".to_owned())?;
                let manifest_payload = row["manifest_payload"]
                    .value::<pgrx::JsonB>()
                    .map_err(|e| format!("manifest bind manifest_payload decode failed: {e}"))?
                    .ok_or_else(|| "manifest bind manifest_payload is null".to_owned())?;
                let dispatch_action = row["dispatch_action"]
                    .value::<String>()
                    .map_err(|e| format!("manifest bind dispatch_action decode failed: {e}"))?
                    .ok_or_else(|| "manifest bind dispatch_action is null".to_owned())?;
                let status = row["status"]
                    .value::<String>()
                    .map_err(|e| format!("manifest bind status decode failed: {e}"))?
                    .ok_or_else(|| "manifest bind status is null".to_owned())?;

                let value_status =
                    if dispatch_action == "open_pipeline_and_send_remote_epoch_manifest" {
                        "ready".to_owned()
                    } else {
                        status
                    };
                let entry_count = manifest_payload
                    .0
                    .as_object()
                    .and_then(|object| object.get("entries"))
                    .and_then(|entries| entries.as_array())
                    .map(|entries| {
                        i64::try_from(entries.len())
                            .expect("manifest payload entry count should fit in i64")
                    })
                    .unwrap_or(0);

                Ok::<_, String>(vec![
                    (
                        active_epoch,
                        node_id,
                        1_i64,
                        "remote_index_oid",
                        "oid",
                        "remote_node_descriptor.remote_index_regclass",
                        value_status.clone(),
                        remote_index_regclass,
                        1_i64,
                    ),
                    (
                        active_epoch,
                        node_id,
                        2_i64,
                        "active_epoch",
                        "bigint",
                        "manifest_catalog.active_epoch",
                        value_status.clone(),
                        active_epoch.to_string(),
                        1_i64,
                    ),
                    (
                        active_epoch,
                        node_id,
                        3_i64,
                        "manifest_payload",
                        "jsonb",
                        "ec_spire_remote_epoch_manifest_payload_plan.manifest_payload",
                        value_status,
                        format!("{manifest_payload_format}:entries={entry_count}"),
                        entry_count,
                    ),
                ])
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::new(rows.into_iter().flatten())
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_libpq_bind_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(request_count, i64),
        name!(bind_count, i64),
        name!(ready_bind_count, i64),
        name!(blocked_bind_count, i64),
        name!(parameter_count_per_request, i64),
        name!(manifest_entry_count, i64),
        name!(executor_status, String),
        name!(status, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_epoch_manifest_libpq_bind_summary",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let row = Spi::connect(|client| {
        client
            .select(
                "WITH bind AS ( \
                     SELECT * FROM ec_spire_remote_epoch_manifest_libpq_bind_plan($1::oid) \
                 ), dispatch AS ( \
                     SELECT * FROM ec_spire_remote_epoch_manifest_libpq_dispatch_summary($1::oid) \
                 ) \
                 SELECT d.active_epoch, d.dispatch_count AS request_count, \
                        count(b.parameter_ordinal)::bigint AS bind_count, \
                        count(*) FILTER (WHERE b.value_status = 'ready')::bigint \
                            AS ready_bind_count, \
                        count(*) FILTER (WHERE b.parameter_ordinal IS NOT NULL \
                                           AND b.value_status <> 'ready')::bigint \
                            AS blocked_bind_count, \
                        3::bigint AS parameter_count_per_request, \
                        coalesce(sum(b.element_count) FILTER \
                            (WHERE b.parameter_name = 'manifest_payload'), 0)::bigint \
                            AS manifest_entry_count, \
                        d.executor_status, \
                        CASE \
                            WHEN d.dispatch_count = 0 THEN d.status \
                            WHEN count(*) FILTER \
                                (WHERE b.parameter_ordinal IS NOT NULL \
                                   AND b.value_status <> 'ready') = 0 THEN 'ready' \
                            ELSE d.status \
                        END AS status \
                   FROM dispatch d \
                   LEFT JOIN bind b ON b.active_epoch = d.active_epoch \
                  GROUP BY d.active_epoch, d.dispatch_count, d.executor_status, d.status",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest libpq bind summary read failed: {e}")
            })?
            .map(|row| {
                Ok::<_, String>((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest bind summary active_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest bind summary active_epoch is null".to_owned()
                        })?,
                    row["request_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest bind summary request_count decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest bind summary request_count is null".to_owned()
                        })?,
                    row["bind_count"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest bind summary bind_count decode failed: {e}"))?
                        .ok_or_else(|| "manifest bind summary bind_count is null".to_owned())?,
                    row["ready_bind_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest bind summary ready_bind_count decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest bind summary ready_bind_count is null".to_owned()
                        })?,
                    row["blocked_bind_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest bind summary blocked_bind_count decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest bind summary blocked_bind_count is null".to_owned()
                        })?,
                    row["parameter_count_per_request"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest bind summary parameter_count_per_request decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest bind summary parameter_count_per_request is null".to_owned()
                        })?,
                    row["manifest_entry_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest bind summary manifest_entry_count decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest bind summary manifest_entry_count is null".to_owned()
                        })?,
                    row["executor_status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest bind summary executor_status decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest bind summary executor_status is null".to_owned()
                        })?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| format!("manifest bind summary status decode failed: {e}"))?
                        .ok_or_else(|| "manifest bind summary status is null".to_owned())?,
                ))
            })
            .next()
            .transpose()?
            .ok_or_else(|| {
                "ec_spire remote epoch manifest libpq bind summary returned no rows".to_owned()
            })
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::once(row)
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_libpq_executor_work_plan(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(bind_count, i64),
        name!(bind_status, String),
        name!(dispatch_action, String),
        name!(next_executor_step, String),
        name!(executor_status, String),
        name!(work_action, String),
        name!(status, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_epoch_manifest_libpq_executor_work_plan",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let rows = Spi::connect(|client| {
        client
            .select(
                "SELECT d.active_epoch, d.node_id, 3::bigint AS bind_count, \
                        CASE \
                            WHEN d.dispatch_action = \
                                'open_pipeline_and_send_remote_epoch_manifest' \
                            THEN 'ready' \
                            ELSE d.status \
                        END AS bind_status, \
                        d.dispatch_action, e.next_executor_step, e.status AS executor_status, \
                        CASE \
                            WHEN d.dispatch_action = \
                                'open_pipeline_and_send_remote_epoch_manifest' \
                            THEN e.secret_resolution_action \
                            ELSE 'blocked_before_executor' \
                        END AS work_action, \
                        CASE \
                            WHEN d.dispatch_action = \
                                'open_pipeline_and_send_remote_epoch_manifest' \
                            THEN e.status \
                            ELSE d.status \
                        END AS status \
                   FROM ec_spire_remote_epoch_manifest_libpq_dispatch_plan($1::oid) d \
                  CROSS JOIN ec_spire_remote_epoch_manifest_libpq_executor_readiness($1::oid) e \
                  ORDER BY d.node_id",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest libpq executor work plan read failed: {e}")
            })?
            .map(|row| {
                Ok::<_, String>((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest executor work active_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest executor work active_epoch is null".to_owned())?,
                    row["node_id"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest executor work node_id decode failed: {e}"))?
                        .ok_or_else(|| "manifest executor work node_id is null".to_owned())?,
                    row["bind_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest executor work bind_count decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest executor work bind_count is null".to_owned())?,
                    row["bind_status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest executor work bind_status decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest executor work bind_status is null".to_owned())?,
                    row["dispatch_action"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest executor work dispatch_action decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest executor work dispatch_action is null".to_owned()
                        })?,
                    row["next_executor_step"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest executor work next_executor_step decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest executor work next_executor_step is null".to_owned()
                        })?,
                    row["executor_status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest executor work executor_status decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest executor work executor_status is null".to_owned()
                        })?,
                    row["work_action"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest executor work work_action decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest executor work work_action is null".to_owned())?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| format!("manifest executor work status decode failed: {e}"))?
                        .ok_or_else(|| "manifest executor work status is null".to_owned())?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::new(rows.into_iter())
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_libpq_executor_work_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(work_count, i64),
        name!(ready_work_count, i64),
        name!(blocked_work_count, i64),
        name!(bind_ready_work_count, i64),
        name!(next_executor_step, String),
        name!(executor_status, String),
        name!(status, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_epoch_manifest_libpq_executor_work_summary",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let row = Spi::connect(|client| {
        client
            .select(
                "WITH work AS ( \
                     SELECT * FROM \
                         ec_spire_remote_epoch_manifest_libpq_executor_work_plan($1::oid) \
                 ), readiness AS ( \
                     SELECT * FROM \
                         ec_spire_remote_epoch_manifest_libpq_executor_readiness($1::oid) \
                 ) \
                 SELECT r.active_epoch, \
                        count(w.node_id)::bigint AS work_count, \
                        count(*) FILTER (WHERE w.bind_status = 'ready')::bigint \
                            AS ready_work_count, \
                        count(*) FILTER (WHERE w.node_id IS NOT NULL \
                                           AND w.bind_status <> 'ready')::bigint \
                            AS blocked_work_count, \
                        count(*) FILTER (WHERE w.bind_status = 'ready')::bigint \
                            AS bind_ready_work_count, \
                        r.next_executor_step, r.status AS executor_status, \
                        CASE \
                            WHEN count(w.node_id) = 0 THEN r.status \
                            WHEN count(*) FILTER \
                                (WHERE w.node_id IS NOT NULL \
                                   AND w.bind_status <> 'ready') = 0 THEN r.status \
                            ELSE min(w.status) FILTER (WHERE w.bind_status <> 'ready') \
                        END AS status \
                   FROM readiness r \
                   LEFT JOIN work w ON w.active_epoch = r.active_epoch \
                  GROUP BY r.active_epoch, r.next_executor_step, r.status",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest libpq executor work summary read failed: {e}")
            })?
            .map(|row| {
                Ok::<_, String>((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest executor work summary active_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest executor work summary active_epoch is null".to_owned()
                        })?,
                    row["work_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest executor work summary work_count decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest executor work summary work_count is null".to_owned()
                        })?,
                    row["ready_work_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest executor work summary ready_work_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest executor work summary ready_work_count is null".to_owned()
                        })?,
                    row["blocked_work_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest executor work summary blocked_work_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest executor work summary blocked_work_count is null".to_owned()
                        })?,
                    row["bind_ready_work_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest executor work summary bind_ready_work_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest executor work summary bind_ready_work_count is null"
                                .to_owned()
                        })?,
                    row["next_executor_step"]
                        .value::<String>()
                        .map_err(|e| {
                            format!(
                                "manifest executor work summary next_executor_step decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest executor work summary next_executor_step is null".to_owned()
                        })?,
                    row["executor_status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!(
                                "manifest executor work summary executor_status decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest executor work summary executor_status is null".to_owned()
                        })?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest executor work summary status decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest executor work summary status is null".to_owned()
                        })?,
                ))
            })
            .next()
            .transpose()?
            .ok_or_else(|| {
                "ec_spire remote epoch manifest libpq executor work summary returned no rows"
                    .to_owned()
            })
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::once(row)
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_libpq_dispatch_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(dispatch_count, i64),
        name!(pipeline_dispatch_count, i64),
        name!(blocked_dispatch_count, i64),
        name!(ready_payload_count, i64),
        name!(executor_status, String),
        name!(status, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_epoch_manifest_libpq_dispatch_summary",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let row = Spi::connect(|client| {
        client
            .select(
                "WITH dispatch_rows AS ( \
                        SELECT * \
                          FROM ec_spire_remote_epoch_manifest_libpq_dispatch_plan($1::oid) \
                    ), publication_summary AS ( \
                        SELECT * \
                          FROM ec_spire_remote_epoch_manifest_publication_summary($1::oid) \
                    ) \
                  SELECT s.active_epoch, \
                         count(d.active_epoch)::bigint AS dispatch_count, \
                         count(*) FILTER ( \
                             WHERE d.dispatch_action = 'open_pipeline_and_send_remote_epoch_manifest')::bigint \
                             AS pipeline_dispatch_count, \
                         count(*) FILTER ( \
                             WHERE d.active_epoch IS NOT NULL \
                               AND d.dispatch_action <> 'open_pipeline_and_send_remote_epoch_manifest')::bigint \
                             AS blocked_dispatch_count, \
                         count(*) FILTER (WHERE d.status = 'ready')::bigint \
                             AS ready_payload_count, \
                         CASE \
                             WHEN count(*) FILTER ( \
                                 WHERE d.dispatch_action = 'open_pipeline_and_send_remote_epoch_manifest') > 0 \
                             THEN 'requires_libpq_executor' \
                             ELSE s.publication_executor_status \
                         END AS executor_status, \
                         CASE \
                             WHEN count(d.active_epoch) = 0 THEN s.status \
                             WHEN bool_and(d.status = 'ready') THEN 'ready' \
                             ELSE 'blocked' \
                         END AS status \
                    FROM publication_summary s \
                    LEFT JOIN dispatch_rows d ON true \
                   GROUP BY s.active_epoch, s.status, s.publication_executor_status",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest libpq dispatch summary read failed: {e}")
            })?
            .map(|row| {
                Ok::<_, String>((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("dispatch summary active_epoch decode failed: {e}"))?
                        .ok_or_else(|| "dispatch summary active_epoch is null".to_owned())?,
                    row["dispatch_count"]
                        .value::<i64>()
                        .map_err(|e| format!("dispatch summary dispatch_count decode failed: {e}"))?
                        .ok_or_else(|| "dispatch summary dispatch_count is null".to_owned())?,
                    row["pipeline_dispatch_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("dispatch summary pipeline_dispatch_count decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "dispatch summary pipeline_dispatch_count is null".to_owned()
                        })?,
                    row["blocked_dispatch_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("dispatch summary blocked_dispatch_count decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "dispatch summary blocked_dispatch_count is null".to_owned()
                        })?,
                    row["ready_payload_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("dispatch summary ready_payload_count decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "dispatch summary ready_payload_count is null".to_owned()
                        })?,
                    row["executor_status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("dispatch summary executor_status decode failed: {e}")
                        })?
                        .ok_or_else(|| "dispatch summary executor_status is null".to_owned())?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| format!("dispatch summary status decode failed: {e}"))?
                        .ok_or_else(|| "dispatch summary status is null".to_owned())?,
                ))
            })
            .next()
            .transpose()?
            .ok_or_else(|| {
                "ec_spire remote epoch manifest libpq dispatch summary returned no rows".to_owned()
            })
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::once(row)
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_libpq_executor_readiness(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(dispatch_count, i64),
        name!(pipeline_dispatch_count, i64),
        name!(blocked_dispatch_count, i64),
        name!(secret_resolution_action, String),
        name!(connection_action, String),
        name!(pipeline_action, String),
        name!(send_action, String),
        name!(receive_action, String),
        name!(result_action, String),
        name!(next_executor_step, String),
        name!(status, String),
        name!(recommendation, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_epoch_manifest_libpq_executor_readiness",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let (
        active_epoch,
        dispatch_count,
        pipeline_dispatch_count,
        blocked_dispatch_count,
        dispatch_status,
    ) = Spi::connect(|client| {
        client
            .select(
                "SELECT active_epoch, dispatch_count, pipeline_dispatch_count, \
                        blocked_dispatch_count, status \
                   FROM ec_spire_remote_epoch_manifest_libpq_dispatch_summary($1::oid)",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest libpq executor readiness read failed: {e}")
            })?
            .map(|row| {
                Ok::<_, String>((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest executor active_epoch decode failed: {e}"))?
                        .ok_or_else(|| "manifest executor active_epoch is null".to_owned())?,
                    row["dispatch_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest executor dispatch_count decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest executor dispatch_count is null".to_owned())?,
                    row["pipeline_dispatch_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest executor pipeline_dispatch_count decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest executor pipeline_dispatch_count is null".to_owned()
                        })?,
                    row["blocked_dispatch_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest executor blocked_dispatch_count decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest executor blocked_dispatch_count is null".to_owned()
                        })?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| format!("manifest executor status decode failed: {e}"))?
                        .ok_or_else(|| "manifest executor status is null".to_owned())?,
                ))
            })
            .next()
            .transpose()?
            .ok_or_else(|| {
                "ec_spire remote epoch manifest libpq executor readiness returned no rows"
                    .to_owned()
            })
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    let (
        secret_resolution_action,
        connection_action,
        pipeline_action,
        send_action,
        receive_action,
        result_action,
        next_executor_step,
        status,
        recommendation,
    ) = if pipeline_dispatch_count > 0 && blocked_dispatch_count == 0 {
        (
            "resolve_conninfo_secret",
            "open_remote_connection",
            "enter_pipeline_mode",
            "send_remote_epoch_manifest",
            "validate_remote_manifest_payload_result",
            "validated_remote_epoch_manifest_payload",
            "conninfo_secret_resolution",
            "requires_libpq_executor",
            "libpq manifest publication executor is required",
        )
    } else if dispatch_count == 0 {
        (
            "none",
            "none",
            "none",
            "none",
            "none",
            "none",
            "none",
            dispatch_status.as_str(),
            "no remote manifest dispatch is required",
        )
    } else {
        (
            "blocked",
            "blocked",
            "blocked",
            "blocked",
            "blocked",
            "blocked",
            "manifest_dispatch",
            dispatch_status.as_str(),
            "resolve manifest publication blockers before libpq execution",
        )
    };

    TableIterator::once((
        active_epoch,
        dispatch_count,
        pipeline_dispatch_count,
        blocked_dispatch_count,
        secret_resolution_action.to_owned(),
        connection_action.to_owned(),
        pipeline_action.to_owned(),
        send_action.to_owned(),
        receive_action.to_owned(),
        result_action.to_owned(),
        next_executor_step.to_owned(),
        status.to_owned(),
        recommendation.to_owned(),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_libpq_receive_plan(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(expected_result_column_count, i64),
        name!(validator_function, String),
        name!(result_action, String),
        name!(result_contract, String),
        name!(status, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_epoch_manifest_libpq_receive_plan",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let rows = Spi::connect(|client| {
        client
            .select(
                "SELECT d.active_epoch, d.node_id, \
                        d.expected_result_column_count, \
                        d.receive_validator AS validator_function, \
                        CASE \
                            WHEN d.dispatch_action = 'open_pipeline_and_send_remote_epoch_manifest' \
                             AND d.status = 'ready' \
                            THEN 'validate_remote_manifest_payload_result' \
                            ELSE 'blocked_before_manifest_receive' \
                        END AS result_action, \
                        'remote_manifest_payload_validation_result'::text \
                            AS result_contract, \
                        CASE \
                            WHEN d.dispatch_action = 'open_pipeline_and_send_remote_epoch_manifest' \
                             AND d.status = 'ready' \
                            THEN e.status \
                            ELSE d.status \
                        END AS status \
                   FROM ec_spire_remote_epoch_manifest_libpq_dispatch_plan($1::oid) d \
                  CROSS JOIN ec_spire_remote_epoch_manifest_libpq_executor_readiness($1::oid) e \
                  ORDER BY d.node_id",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest libpq receive plan read failed: {e}")
            })?
            .map(|row| {
                Ok::<_, String>((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest receive active_epoch decode failed: {e}"))?
                        .ok_or_else(|| "manifest receive active_epoch is null".to_owned())?,
                    row["node_id"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest receive node_id decode failed: {e}"))?
                        .ok_or_else(|| "manifest receive node_id is null".to_owned())?,
                    row["expected_result_column_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest receive expected_result_column_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest receive expected_result_column_count is null".to_owned()
                        })?,
                    row["validator_function"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest receive validator_function decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest receive validator_function is null".to_owned())?,
                    row["result_action"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest receive result_action decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest receive result_action is null".to_owned())?,
                    row["result_contract"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest receive result_contract decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest receive result_contract is null".to_owned())?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| format!("manifest receive status decode failed: {e}"))?
                        .ok_or_else(|| "manifest receive status is null".to_owned())?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::new(rows.into_iter())
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_libpq_receive_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(receive_count, i64),
        name!(ready_receive_count, i64),
        name!(blocked_receive_count, i64),
        name!(expected_result_column_count, i64),
        name!(validator_function, String),
        name!(result_contract, String),
        name!(next_executor_step, String),
        name!(executor_status, String),
        name!(status, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_epoch_manifest_libpq_receive_summary",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let row = Spi::connect(|client| {
        client
            .select(
                "WITH receive AS ( \
                     SELECT * FROM \
                         ec_spire_remote_epoch_manifest_libpq_receive_plan($1::oid) \
                 ), readiness AS ( \
                     SELECT * FROM \
                         ec_spire_remote_epoch_manifest_libpq_executor_readiness($1::oid) \
                 ) \
                 SELECT r.active_epoch, \
                        count(p.node_id)::bigint AS receive_count, \
                        count(*) FILTER \
                            (WHERE p.result_action = 'validate_remote_manifest_payload_result')::bigint \
                            AS ready_receive_count, \
                        count(*) FILTER \
                            (WHERE p.node_id IS NOT NULL \
                               AND p.result_action <> 'validate_remote_manifest_payload_result')::bigint \
                            AS blocked_receive_count, \
                        coalesce(max(p.expected_result_column_count), 0)::bigint \
                            AS expected_result_column_count, \
                        coalesce(max(p.validator_function), 'none') AS validator_function, \
                        coalesce(max(p.result_contract), 'none') AS result_contract, \
                        r.next_executor_step, r.status AS executor_status, \
                        CASE \
                            WHEN count(p.node_id) = 0 THEN r.status \
                            WHEN count(*) FILTER \
                                (WHERE p.node_id IS NOT NULL \
                                   AND p.result_action <> 'validate_remote_manifest_payload_result') = 0 \
                            THEN r.status \
                            ELSE min(p.status) FILTER \
                                (WHERE p.result_action <> 'validate_remote_manifest_payload_result') \
                        END AS status \
                   FROM readiness r \
                   LEFT JOIN receive p ON p.active_epoch = r.active_epoch \
                  GROUP BY r.active_epoch, r.next_executor_step, r.status",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest libpq receive summary read failed: {e}")
            })?
            .map(|row| {
                Ok::<_, String>((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest receive summary active_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest receive summary active_epoch is null".to_owned()
                        })?,
                    row["receive_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest receive summary receive_count decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest receive summary receive_count is null".to_owned()
                        })?,
                    row["ready_receive_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest receive summary ready_receive_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest receive summary ready_receive_count is null".to_owned()
                        })?,
                    row["blocked_receive_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest receive summary blocked_receive_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest receive summary blocked_receive_count is null".to_owned()
                        })?,
                    row["expected_result_column_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest receive summary expected_result_column_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest receive summary expected_result_column_count is null"
                                .to_owned()
                        })?,
                    row["validator_function"]
                        .value::<String>()
                        .map_err(|e| {
                            format!(
                                "manifest receive summary validator_function decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest receive summary validator_function is null".to_owned()
                        })?,
                    row["result_contract"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest receive summary result_contract decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest receive summary result_contract is null".to_owned()
                        })?,
                    row["next_executor_step"]
                        .value::<String>()
                        .map_err(|e| {
                            format!(
                                "manifest receive summary next_executor_step decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest receive summary next_executor_step is null".to_owned()
                        })?,
                    row["executor_status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest receive summary executor_status decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest receive summary executor_status is null".to_owned()
                        })?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest receive summary status decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest receive summary status is null".to_owned())?,
                ))
            })
            .next()
            .transpose()?
            .ok_or_else(|| {
                "ec_spire remote epoch manifest libpq receive summary returned no rows".to_owned()
            })
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::once(row)
}

#[derive(Debug)]
struct SpireManifestExecutorDispatchRow {
    active_epoch: i64,
    node_id: i64,
    conninfo_secret_name: String,
    remote_index_regclass: String,
    manifest_payload: pgrx::JsonB,
    dispatch_action: String,
    status: String,
}

#[derive(Debug)]
struct SpireManifestExecutorResultRow {
    active_epoch: i64,
    node_id: i64,
    connection_attempted: bool,
    connection_status: &'static str,
    validated_entry_count: i64,
    validation_result_status: String,
    conninfo_lookup_kind: &'static str,
    next_executor_step: &'static str,
    status: &'static str,
    recommendation: &'static str,
}

impl SpireManifestExecutorResultRow {
    fn into_tuple(
        self,
    ) -> (
        i64,
        i64,
        bool,
        &'static str,
        i64,
        String,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    ) {
        (
            self.active_epoch,
            self.node_id,
            self.connection_attempted,
            self.connection_status,
            self.validated_entry_count,
            self.validation_result_status,
            self.conninfo_lookup_kind,
            self.next_executor_step,
            self.status,
            self.recommendation,
        )
    }
}

fn load_spire_manifest_executor_dispatch_rows(
    index_oid: pg_sys::Oid,
) -> Result<Vec<SpireManifestExecutorDispatchRow>, String> {
    Spi::connect(|client| {
        client
            .select(
                "SELECT active_epoch, node_id, conninfo_secret_name, remote_index_regclass, \
                        manifest_payload, dispatch_action, status \
                   FROM ec_spire_remote_epoch_manifest_libpq_dispatch_plan($1::oid) \
                  ORDER BY node_id",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| format!("ec_spire manifest executor dispatch read failed: {e}"))?
            .map(|row| {
                Ok::<_, String>(SpireManifestExecutorDispatchRow {
                    active_epoch: row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest executor active_epoch decode failed: {e}"))?
                        .ok_or_else(|| "manifest executor active_epoch is null".to_owned())?,
                    node_id: row["node_id"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest executor node_id decode failed: {e}"))?
                        .ok_or_else(|| "manifest executor node_id is null".to_owned())?,
                    conninfo_secret_name: row["conninfo_secret_name"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest executor conninfo_secret_name decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest executor conninfo_secret_name is null".to_owned()
                        })?,
                    remote_index_regclass: row["remote_index_regclass"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest executor remote_index_regclass decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest executor remote_index_regclass is null".to_owned()
                        })?,
                    manifest_payload: row["manifest_payload"]
                        .value::<pgrx::JsonB>()
                        .map_err(|e| {
                            format!("manifest executor manifest_payload decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest executor manifest_payload is null".to_owned())?,
                    dispatch_action: row["dispatch_action"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest executor dispatch_action decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest executor dispatch_action is null".to_owned())?,
                    status: row["status"]
                        .value::<String>()
                        .map_err(|e| format!("manifest executor status decode failed: {e}"))?
                        .ok_or_else(|| "manifest executor status is null".to_owned())?,
                })
            })
            .collect::<Result<Vec<_>, String>>()
    })
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_libpq_executor_results(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(node_id, i64),
        name!(connection_attempted, bool),
        name!(connection_status, &'static str),
        name!(validated_entry_count, i64),
        name!(validation_result_status, String),
        name!(conninfo_lookup_kind, &'static str),
        name!(next_executor_step, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_epoch_manifest_libpq_executor_results",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let dispatch_rows = load_spire_manifest_executor_dispatch_rows(index_oid)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    let rows = dispatch_rows
        .into_iter()
        .map(|row| {
            if row.dispatch_action != "open_pipeline_and_send_remote_epoch_manifest" {
                return SpireManifestExecutorResultRow {
                    active_epoch: row.active_epoch,
                    node_id: row.node_id,
                    connection_attempted: false,
                    connection_status: "blocked_before_connection",
                    validated_entry_count: 0,
                    validation_result_status: row.status,
                    conninfo_lookup_kind: "not_attempted",
                    next_executor_step: "manifest_dispatch",
                    status: "blocked",
                    recommendation: "resolve manifest publication blockers before executor send",
                };
            }

            let provider_lookup_key =
                am::spire_remote_conninfo_secret_provider_lookup_key(&row.conninfo_secret_name)
                    .unwrap_or_else(|e| {
                        pgrx::error!(
                            "ec_spire manifest executor conninfo secret reference invalid: {e}"
                        )
                    });
            let conninfo = match std::env::var(&provider_lookup_key) {
                Ok(conninfo) if !conninfo.is_empty() => conninfo,
                Ok(_) => {
                    return SpireManifestExecutorResultRow {
                        active_epoch: row.active_epoch,
                        node_id: row.node_id,
                        connection_attempted: false,
                        connection_status: "conninfo_secret_empty",
                        validated_entry_count: 0,
                        validation_result_status: "requires_conninfo_secret_resolution".to_owned(),
                        conninfo_lookup_kind: "secret_provider",
                        next_executor_step: "conninfo_secret_resolution",
                        status: "requires_conninfo_secret_resolution",
                        recommendation: "configure a nonempty conninfo value in the external secret provider",
                    };
                }
                Err(_) => {
                    return SpireManifestExecutorResultRow {
                        active_epoch: row.active_epoch,
                        node_id: row.node_id,
                        connection_attempted: false,
                        connection_status: "conninfo_secret_missing",
                        validated_entry_count: 0,
                        validation_result_status: "requires_conninfo_secret_resolution".to_owned(),
                        conninfo_lookup_kind: "secret_provider",
                        next_executor_step: "conninfo_secret_resolution",
                        status: "requires_conninfo_secret_resolution",
                        recommendation: "configure the external secret provider entry for conninfo_secret_name",
                    };
                }
            };

            let mut client = match am::spire_remote_search_libpq_connect_with_session_timeouts(
                &conninfo,
                u32::try_from(row.node_id).expect("node_id should fit u32"),
                "manifest executor remote validation",
            ) {
                Ok(client) => client,
                Err(_) => {
                    return SpireManifestExecutorResultRow {
                        active_epoch: row.active_epoch,
                        node_id: row.node_id,
                        connection_attempted: true,
                        connection_status: "libpq_connection_open_failed",
                        validated_entry_count: 0,
                        validation_result_status: "libpq_connection_failed".to_owned(),
                        conninfo_lookup_kind: "secret_provider",
                        next_executor_step: "libpq_connection_open",
                        status: "libpq_connection_failed",
                        recommendation: "verify conninfo secret target and remote node availability",
                    };
                }
            };
            let remote_index_oid = match client.query_one(
                "SELECT to_regclass($1)::oid",
                &[&row.remote_index_regclass.as_str()],
            ) {
                Ok(result) => match result.try_get::<_, Option<u32>>(0) {
                    Ok(Some(oid)) => oid,
                    _ => {
                        return SpireManifestExecutorResultRow {
                            active_epoch: row.active_epoch,
                            node_id: row.node_id,
                            connection_attempted: true,
                            connection_status: "libpq_connection_opened",
                            validated_entry_count: 0,
                            validation_result_status: "remote_index_resolution_failed".to_owned(),
                            conninfo_lookup_kind: "secret_provider",
                            next_executor_step: "send_manifest_request",
                            status: "remote_index_resolution_failed",
                            recommendation: "verify remote_index_regclass on the target node",
                        };
                    }
                },
                Err(_) => {
                    return SpireManifestExecutorResultRow {
                        active_epoch: row.active_epoch,
                        node_id: row.node_id,
                        connection_attempted: true,
                        connection_status: "libpq_connection_opened",
                        validated_entry_count: 0,
                        validation_result_status: "remote_index_resolution_failed".to_owned(),
                        conninfo_lookup_kind: "secret_provider",
                        next_executor_step: "send_manifest_request",
                        status: "remote_index_resolution_failed",
                        recommendation: "verify remote_index_regclass on the target node",
                    };
                }
            };
            let payload = row.manifest_payload.0.to_string();
            let result = client.query_one(
                "SELECT active_epoch, validated_entry_count, status \
                   FROM ec_spire_apply_remote_epoch_manifest_payload($1::oid, $2::bigint, $3::text::jsonb)",
                &[&remote_index_oid, &row.active_epoch, &payload],
            );
            let result = match result {
                Ok(result) => result,
                Err(_) => {
                    return SpireManifestExecutorResultRow {
                        active_epoch: row.active_epoch,
                        node_id: row.node_id,
                        connection_attempted: true,
                        connection_status: "libpq_connection_opened",
                        validated_entry_count: 0,
                        validation_result_status: "manifest_validation_request_failed".to_owned(),
                        conninfo_lookup_kind: "secret_provider",
                        next_executor_step: "send_manifest_request",
                        status: "manifest_validation_request_failed",
                        recommendation: "verify remote manifest validation endpoint and payload contract",
                    };
                }
            };
            let validated_entry_count = result
                .try_get::<_, i64>("validated_entry_count")
                .unwrap_or(0);
            let validation_status = result
                .try_get::<_, String>("status")
                .unwrap_or_else(|_| "manifest_validation_result_decode_failed".to_owned());
            let (next_executor_step, status, recommendation) = if validation_status == "ready" {
                (
                    "none",
                    "ready",
                    "remote manifest payload validation succeeded",
                )
            } else {
                (
                    "receive_payload_validation_result",
                    "remote_manifest_validation_failed",
                    "inspect remote manifest payload validation status",
                )
            };

            SpireManifestExecutorResultRow {
                active_epoch: row.active_epoch,
                node_id: row.node_id,
                connection_attempted: true,
                connection_status: "libpq_connection_opened",
                validated_entry_count,
                validation_result_status: validation_status,
                conninfo_lookup_kind: "secret_provider",
                next_executor_step,
                status,
                recommendation,
            }
        })
        .map(SpireManifestExecutorResultRow::into_tuple)
        .collect::<Vec<_>>();

    TableIterator::new(rows.into_iter())
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_publication_gate_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(publication_decision, String),
        name!(publication_entry_count, i64),
        name!(libpq_request_count, i64),
        name!(libpq_dispatch_count, i64),
        name!(libpq_receive_count, i64),
        name!(ready_receive_count, i64),
        name!(publication_status, String),
        name!(libpq_executor_status, String),
        name!(libpq_executor_next_step, String),
        name!(receive_status, String),
        name!(next_blocker, String),
        name!(status, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_epoch_manifest_publication_gate_summary",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let row = Spi::connect(|client| {
        client
            .select(
                "WITH publication AS ( \
                     SELECT * FROM ec_spire_remote_epoch_manifest_publication_summary($1::oid) \
                 ), request AS ( \
                     SELECT * FROM ec_spire_remote_epoch_manifest_libpq_request_summary($1::oid) \
                 ), dispatch AS ( \
                     SELECT * FROM ec_spire_remote_epoch_manifest_libpq_dispatch_summary($1::oid) \
                 ), receive AS ( \
                     SELECT * FROM ec_spire_remote_epoch_manifest_libpq_receive_summary($1::oid) \
                 ) \
                 SELECT p.active_epoch, p.publication_decision, \
                        p.publication_entry_count, \
                        r.request_count AS libpq_request_count, \
                        d.dispatch_count AS libpq_dispatch_count, \
                        v.receive_count AS libpq_receive_count, \
                        v.ready_receive_count, \
                        p.status AS publication_status, \
                        v.executor_status AS libpq_executor_status, \
                        v.next_executor_step AS libpq_executor_next_step, \
                        v.status AS receive_status, \
                        CASE \
                            WHEN p.status <> 'ready' THEN p.next_blocker \
                            WHEN p.publication_decision = 'publish_remote_epoch_manifest' \
                             AND v.status = 'requires_libpq_executor' \
                            THEN v.next_executor_step \
                            WHEN p.publication_decision = 'publish_remote_epoch_manifest' \
                             AND v.status <> 'ready' \
                            THEN 'manifest_receive' \
                            ELSE 'none' \
                        END AS next_blocker, \
                        CASE \
                            WHEN p.status <> 'ready' THEN p.status \
                            WHEN p.publication_decision = 'publish_remote_epoch_manifest' \
                            THEN v.status \
                            ELSE p.status \
                        END AS status \
                   FROM publication p \
                   JOIN request r ON r.active_epoch = p.active_epoch \
                   JOIN dispatch d ON d.active_epoch = p.active_epoch \
                   JOIN receive v ON v.active_epoch = p.active_epoch",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!("ec_spire remote epoch manifest publication gate summary read failed: {e}")
            })?
            .map(|row| {
                Ok::<_, String>((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest publication gate active_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest publication gate active_epoch is null".to_owned()
                        })?,
                    row["publication_decision"]
                        .value::<String>()
                        .map_err(|e| {
                            format!(
                                "manifest publication gate publication_decision decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest publication gate publication_decision is null".to_owned()
                        })?,
                    row["publication_entry_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest publication gate publication_entry_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest publication gate publication_entry_count is null".to_owned()
                        })?,
                    row["libpq_request_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest publication gate libpq_request_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest publication gate libpq_request_count is null".to_owned()
                        })?,
                    row["libpq_dispatch_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest publication gate libpq_dispatch_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest publication gate libpq_dispatch_count is null".to_owned()
                        })?,
                    row["libpq_receive_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest publication gate libpq_receive_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest publication gate libpq_receive_count is null".to_owned()
                        })?,
                    row["ready_receive_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest publication gate ready_receive_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest publication gate ready_receive_count is null".to_owned()
                        })?,
                    row["publication_status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!(
                                "manifest publication gate publication_status decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest publication gate publication_status is null".to_owned()
                        })?,
                    row["libpq_executor_status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!(
                                "manifest publication gate libpq_executor_status decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest publication gate libpq_executor_status is null".to_owned()
                        })?,
                    row["libpq_executor_next_step"]
                        .value::<String>()
                        .map_err(|e| {
                            format!(
                                "manifest publication gate libpq_executor_next_step decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest publication gate libpq_executor_next_step is null"
                                .to_owned()
                        })?,
                    row["receive_status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest publication gate receive_status decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest publication gate receive_status is null".to_owned()
                        })?,
                    row["next_blocker"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest publication gate next_blocker decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest publication gate next_blocker is null".to_owned()
                        })?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest publication gate status decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest publication gate status is null".to_owned())?,
                ))
            })
            .next()
            .transpose()?
            .ok_or_else(|| {
                "ec_spire remote epoch manifest publication gate summary returned no rows"
                    .to_owned()
            })
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::once(row)
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_publication_result_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(publication_decision, String),
        name!(result_source, String),
        name!(publication_entry_count, i64),
        name!(libpq_receive_count, i64),
        name!(ready_receive_count, i64),
        name!(validation_result_status, String),
        name!(next_blocker, String),
        name!(status, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_epoch_manifest_publication_result_summary",
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let row = Spi::connect(|client| {
        client
            .select(
                "SELECT active_epoch, publication_decision, \
                        CASE \
                            WHEN publication_decision = 'not_required' THEN 'not_required' \
                            WHEN publication_decision = 'publish_remote_epoch_manifest' \
                             AND status = 'requires_libpq_executor' \
                            THEN 'pending_libpq_executor' \
                            WHEN publication_decision = 'publish_remote_epoch_manifest' \
                             AND status = 'ready' \
                            THEN 'remote_manifest_validation_result' \
                            ELSE 'blocked' \
                        END AS result_source, \
                        publication_entry_count, libpq_receive_count, \
                        ready_receive_count, receive_status AS validation_result_status, \
                        next_blocker, status \
                   FROM ec_spire_remote_epoch_manifest_publication_gate_summary($1::oid)",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| {
                format!(
                    "ec_spire remote epoch manifest publication result summary read failed: {e}"
                )
            })?
            .map(|row| {
                Ok::<_, String>((
                    row["active_epoch"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("manifest publication result active_epoch decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest publication result active_epoch is null".to_owned()
                        })?,
                    row["publication_decision"]
                        .value::<String>()
                        .map_err(|e| {
                            format!(
                                "manifest publication result publication_decision decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest publication result publication_decision is null".to_owned()
                        })?,
                    row["result_source"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest publication result result_source decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest publication result result_source is null".to_owned()
                        })?,
                    row["publication_entry_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest publication result publication_entry_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest publication result publication_entry_count is null".to_owned()
                        })?,
                    row["libpq_receive_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest publication result libpq_receive_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest publication result libpq_receive_count is null".to_owned()
                        })?,
                    row["ready_receive_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "manifest publication result ready_receive_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest publication result ready_receive_count is null".to_owned()
                        })?,
                    row["validation_result_status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!(
                                "manifest publication result validation_result_status decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "manifest publication result validation_result_status is null"
                                .to_owned()
                        })?,
                    row["next_blocker"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest publication result next_blocker decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "manifest publication result next_blocker is null".to_owned()
                        })?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("manifest publication result status decode failed: {e}")
                        })?
                        .ok_or_else(|| "manifest publication result status is null".to_owned())?,
                ))
            })
            .next()
            .transpose()?
            .ok_or_else(|| {
                "ec_spire remote epoch manifest publication result summary returned no rows"
                    .to_owned()
            })
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::once(row)
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_libpq_parameter_contract() -> TableIterator<
    'static,
    (
        name!(parameter_ordinal, i64),
        name!(parameter_name, &'static str),
        name!(pg_type, &'static str),
        name!(semantic_role, &'static str),
        name!(validator, &'static str),
    ),
> {
    TableIterator::new(
        vec![
            (
                1_i64,
                "remote_index_oid",
                "oid",
                "remote_spire_index_identity",
                "must_resolve_to_remote_spire_index",
            ),
            (
                2_i64,
                "active_epoch",
                "bigint",
                "published_manifest_epoch",
                "must_match_manifest_header_epoch",
            ),
            (
                3_i64,
                "manifest_payload",
                "jsonb",
                "ec_spire_remote_epoch_manifest_v1_payload",
                "must_include_manifest_header_and_entries",
            ),
        ]
        .into_iter(),
    )
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_libpq_result_contract() -> TableIterator<
    'static,
    (
        name!(column_ordinal, i64),
        name!(column_name, &'static str),
        name!(pg_type, &'static str),
        name!(semantic_role, &'static str),
        name!(nullable, bool),
        name!(validator, &'static str),
    ),
> {
    TableIterator::new(
        vec![
            (
                1_i64,
                "active_epoch",
                "bigint",
                "validated_manifest_epoch",
                false,
                "must_match_request_active_epoch",
            ),
            (
                2_i64,
                "validated_entry_count",
                "bigint",
                "validated_remote_node_manifest_entries",
                false,
                "must_be_non_negative",
            ),
            (
                3_i64,
                "status",
                "text",
                "remote_manifest_payload_validation_status",
                false,
                "must_report_ready_or_blocker",
            ),
        ]
        .into_iter(),
    )
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_libpq_executor_step_contract() -> TableIterator<
    'static,
    (
        name!(step_ordinal, i64),
        name!(step_name, &'static str),
        name!(executor_action, &'static str),
        name!(input_contract, &'static str),
        name!(output_contract, &'static str),
        name!(blocking_status, &'static str),
        name!(validator, &'static str),
    ),
> {
    TableIterator::new(
        vec![
            (
                1_i64,
                "conninfo_secret_resolution",
                "resolve_conninfo_secret",
                "ec_spire_remote_epoch_manifest_libpq_request_plan",
                "resolved_conninfo",
                "requires_libpq_executor",
                "secret_reference_must_resolve_without_exposure",
            ),
            (
                2_i64,
                "libpq_connection_open",
                "open_remote_connection",
                "resolved_conninfo",
                "open_libpq_connection",
                "requires_libpq_executor",
                "connection_must_target_registered_remote_node",
            ),
            (
                3_i64,
                "pipeline_mode_start",
                "enter_pipeline_mode",
                "open_libpq_connection",
                "pipeline_ready_connection",
                "requires_libpq_executor",
                "connection_must_enter_libpq_pipeline_mode",
            ),
            (
                4_i64,
                "send_manifest_request",
                "send_remote_epoch_manifest",
                "ec_spire_remote_epoch_manifest_libpq_parameter_contract",
                "pending_manifest_payload_validation_result",
                "requires_libpq_executor",
                "binds_must_match_manifest_parameter_contract",
            ),
            (
                5_i64,
                "receive_payload_validation_result",
                "validate_remote_manifest_payload_result",
                "pending_manifest_payload_validation_result",
                "ec_spire_remote_epoch_manifest_libpq_result_contract",
                "requires_libpq_executor",
                "result_must_match_manifest_result_contract",
            ),
        ]
        .into_iter(),
    )
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_operator_entrypoint_contract() -> TableIterator<
    'static,
    (
        name!(entrypoint_ordinal, i64),
        name!(entrypoint_name, &'static str),
        name!(area, &'static str),
        name!(operator_use, &'static str),
        name!(status_source, &'static str),
        name!(next_action, &'static str),
    ),
> {
    let rows = am::spire_remote_operator_entrypoint_contract_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.entrypoint_ordinal).expect("entrypoint ordinal should fit in i64"),
            row.entrypoint_name,
            row.area,
            row.operator_use,
            row.status_source,
            row.next_action,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_libpq_connection_lifecycle_contract() -> TableIterator<
    'static,
    (
        name!(surface, &'static str),
        name!(connection_lifecycle_policy, &'static str),
        name!(pooling_policy, &'static str),
        name!(secret_resolution_policy, &'static str),
        name!(conninfo_exposure_policy, &'static str),
        name!(failure_policy, &'static str),
        name!(resource_limit_policy, &'static str),
        name!(validator, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let rows = am::spire_remote_libpq_connection_lifecycle_contract_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            row.surface,
            row.connection_lifecycle_policy,
            row.pooling_policy,
            row.secret_resolution_policy,
            row.conninfo_exposure_policy,
            row.failure_policy,
            row.resource_limit_policy,
            row.validator,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_conninfo_secret_resolution_contract() -> TableIterator<
    'static,
    (
        name!(provider_ordinal, i64),
        name!(provider_policy, &'static str),
        name!(provider_status, &'static str),
        name!(secret_reference_field, &'static str),
        name!(sql_storage_policy, &'static str),
        name!(raw_conninfo_allowed, bool),
        name!(executor_action, &'static str),
        name!(failure_status, &'static str),
        name!(validator, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let rows = am::spire_remote_conninfo_secret_resolution_contract_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.provider_ordinal).expect("provider ordinal should fit in i64"),
            row.provider_policy,
            row.provider_status,
            row.secret_reference_field,
            row.sql_storage_policy,
            row.raw_conninfo_allowed,
            row.executor_action,
            row.failure_status,
            row.validator,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_conninfo_secret_resolution_status(
    conninfo_secret_name: String,
) -> TableIterator<
    'static,
    (
        name!(provider_policy, &'static str),
        name!(conninfo_secret_name, String),
        name!(provider_lookup_key, String),
        name!(resolved_conninfo_bytes, i64),
        name!(raw_conninfo_exposed, bool),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let row = am::spire_remote_conninfo_secret_resolution_status_row(&conninfo_secret_name);
    TableIterator::once((
        row.provider_policy,
        row.conninfo_secret_name,
        row.provider_lookup_key,
        i64::try_from(row.resolved_conninfo_bytes)
            .expect("resolved conninfo byte count should fit in i64"),
        row.raw_conninfo_exposed,
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_catalog_lifecycle_contract() -> TableIterator<
    'static,
    (
        name!(lifecycle_ordinal, i64),
        name!(lifecycle_event, &'static str),
        name!(oid_stability, &'static str),
        name!(catalog_risk, &'static str),
        name!(operator_action, &'static str),
        name!(cleanup_surface, &'static str),
        name!(migration_surface, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let rows = am::spire_remote_catalog_lifecycle_contract_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.lifecycle_ordinal).expect("lifecycle ordinal should fit in i64"),
            row.lifecycle_event,
            row.oid_stability,
            row.catalog_risk,
            row.operator_action,
            row.cleanup_surface,
            row.migration_surface,
            row.status,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_catalog_orphan_summary() -> TableIterator<
    'static,
    (
        name!(descriptor_orphan_count, i64),
        name!(manifest_orphan_count, i64),
        name!(manifest_entry_orphan_count, i64),
        name!(row_materialization_orphan_count, i64),
        name!(placement_orphan_count, i64),
        name!(cleanup_recommended, bool),
        name!(status, String),
    ),
> {
    let result = Spi::connect(|client| {
        client
            .select(
                "WITH live_spire_index AS ( \
                     SELECT c.oid \
                       FROM pg_class c \
                       JOIN pg_am am ON am.oid = c.relam \
                      WHERE c.relkind = 'i' AND am.amname = 'ec_spire' \
                 ) \
                 SELECT \
                     (SELECT count(*)::bigint \
                        FROM ec_spire_remote_node_descriptor d \
                       WHERE NOT EXISTS (SELECT 1 FROM live_spire_index l \
                                          WHERE l.oid = d.coordinator_index_oid)) \
                        AS descriptor_orphan_count, \
                     (SELECT count(*)::bigint \
                        FROM ec_spire_remote_epoch_manifest m \
                       WHERE NOT EXISTS (SELECT 1 FROM live_spire_index l \
                                          WHERE l.oid = m.coordinator_index_oid)) \
                        AS manifest_orphan_count, \
                     (SELECT count(*)::bigint \
                        FROM ec_spire_remote_epoch_manifest_entry e \
                       WHERE NOT EXISTS (SELECT 1 FROM live_spire_index l \
                                          WHERE l.oid = e.coordinator_index_oid)) \
                        AS manifest_entry_orphan_count, \
                     0::bigint AS row_materialization_orphan_count, \
                     (SELECT count(*)::bigint \
                        FROM ec_spire_placement p \
                       WHERE NOT EXISTS (SELECT 1 FROM live_spire_index l \
                                          WHERE l.oid = p.index_oid)) \
                        AS placement_orphan_count",
                None,
                &[],
            )
            .map_err(|e| format!("ec_spire remote catalog orphan summary failed: {e}"))?
            .map(|row| {
                Ok::<(i64, i64, i64, i64, i64), String>((
                    row["descriptor_orphan_count"]
                        .value::<i64>()
                        .map_err(|e| format!("descriptor orphan count decode failed: {e}"))?
                        .ok_or_else(|| "descriptor orphan count is null".to_owned())?,
                    row["manifest_orphan_count"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest orphan count decode failed: {e}"))?
                        .ok_or_else(|| "manifest orphan count is null".to_owned())?,
                    row["manifest_entry_orphan_count"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest entry orphan count decode failed: {e}"))?
                        .ok_or_else(|| "manifest entry orphan count is null".to_owned())?,
                    row["row_materialization_orphan_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("row materialization orphan count decode failed: {e}")
                        })?
                        .ok_or_else(|| "row materialization orphan count is null".to_owned())?,
                    row["placement_orphan_count"]
                        .value::<i64>()
                        .map_err(|e| format!("placement orphan count decode failed: {e}"))?
                        .ok_or_else(|| "placement orphan count is null".to_owned())?,
                ))
            })
            .next()
            .transpose()
            .map(|value| value.unwrap_or((0, 0, 0, 0, 0)))
    });
    let (
        descriptor_orphan_count,
        manifest_orphan_count,
        manifest_entry_orphan_count,
        row_materialization_orphan_count,
        placement_orphan_count,
    ) = result.unwrap_or_else(|e| pgrx::error!("{e}"));
    let cleanup_recommended = descriptor_orphan_count > 0
        || manifest_orphan_count > 0
        || manifest_entry_orphan_count > 0
        || row_materialization_orphan_count > 0
        || placement_orphan_count > 0;
    let status = if cleanup_recommended {
        "orphaned_remote_catalog_rows"
    } else {
        "ready"
    };

    TableIterator::once((
        descriptor_orphan_count,
        manifest_orphan_count,
        manifest_entry_orphan_count,
        row_materialization_orphan_count,
        placement_orphan_count,
        cleanup_recommended,
        status.to_owned(),
    ))
}

#[pg_extern]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_catalog_orphan_cleanup() -> TableIterator<
    'static,
    (
        name!(descriptor_removed_count, i64),
        name!(manifest_removed_count, i64),
        name!(manifest_entry_removed_count, i64),
        name!(row_materialization_removed_count, i64),
        name!(placement_removed_count, i64),
        name!(status, String),
    ),
> {
    let result = Spi::connect_mut(|client| {
        let counts = client
            .select(
                "WITH live_spire_index AS ( \
                     SELECT c.oid \
                       FROM pg_class c \
                       JOIN pg_am am ON am.oid = c.relam \
                      WHERE c.relkind = 'i' AND am.amname = 'ec_spire' \
                 ) \
                 SELECT \
                     (SELECT count(*)::bigint \
                        FROM ec_spire_remote_node_descriptor d \
                       WHERE NOT EXISTS (SELECT 1 FROM live_spire_index l \
                                          WHERE l.oid = d.coordinator_index_oid)) \
                        AS descriptor_removed_count, \
                     (SELECT count(*)::bigint \
                        FROM ec_spire_remote_epoch_manifest m \
                       WHERE NOT EXISTS (SELECT 1 FROM live_spire_index l \
                                          WHERE l.oid = m.coordinator_index_oid)) \
                        AS manifest_removed_count, \
                     (SELECT count(*)::bigint \
                        FROM ec_spire_remote_epoch_manifest_entry e \
                       WHERE NOT EXISTS (SELECT 1 FROM live_spire_index l \
                                          WHERE l.oid = e.coordinator_index_oid)) \
                        AS manifest_entry_removed_count, \
                     0::bigint AS row_materialization_removed_count, \
                     (SELECT count(*)::bigint \
                        FROM ec_spire_placement p \
                       WHERE NOT EXISTS (SELECT 1 FROM live_spire_index l \
                                          WHERE l.oid = p.index_oid)) \
                        AS placement_removed_count",
                None,
                &[],
            )
            .map_err(|e| format!("ec_spire remote catalog orphan cleanup count failed: {e}"))?
            .map(|row| {
                Ok::<(i64, i64, i64, i64, i64), String>((
                    row["descriptor_removed_count"]
                        .value::<i64>()
                        .map_err(|e| format!("descriptor removed count decode failed: {e}"))?
                        .ok_or_else(|| "descriptor removed count is null".to_owned())?,
                    row["manifest_removed_count"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest removed count decode failed: {e}"))?
                        .ok_or_else(|| "manifest removed count is null".to_owned())?,
                    row["manifest_entry_removed_count"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest entry removed count decode failed: {e}"))?
                        .ok_or_else(|| "manifest entry removed count is null".to_owned())?,
                    row["row_materialization_removed_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("row materialization removed count decode failed: {e}")
                        })?
                        .ok_or_else(|| "row materialization removed count is null".to_owned())?,
                    row["placement_removed_count"]
                        .value::<i64>()
                        .map_err(|e| format!("placement removed count decode failed: {e}"))?
                        .ok_or_else(|| "placement removed count is null".to_owned())?,
                ))
            })
            .next()
            .transpose()?
            .unwrap_or((0, 0, 0, 0, 0));
        client
            .update(
                "WITH live_spire_index AS ( \
                     SELECT c.oid \
                       FROM pg_class c \
                       JOIN pg_am am ON am.oid = c.relam \
                      WHERE c.relkind = 'i' AND am.amname = 'ec_spire' \
                 ) \
                 DELETE FROM ec_spire_placement p \
                  WHERE NOT EXISTS (SELECT 1 FROM live_spire_index l \
                                     WHERE l.oid = p.index_oid)",
                None,
                &[],
            )
            .map_err(|e| format!("ec_spire placement orphan cleanup failed: {e}"))?;
        client
            .update(
                "WITH live_spire_index AS ( \
                     SELECT c.oid \
                       FROM pg_class c \
                       JOIN pg_am am ON am.oid = c.relam \
                      WHERE c.relkind = 'i' AND am.amname = 'ec_spire' \
                 ) \
                 DELETE FROM ec_spire_remote_epoch_manifest m \
                  WHERE NOT EXISTS (SELECT 1 FROM live_spire_index l \
                                     WHERE l.oid = m.coordinator_index_oid)",
                None,
                &[],
            )
            .map_err(|e| format!("ec_spire remote manifest orphan cleanup failed: {e}"))?;
        client
            .update(
                "WITH live_spire_index AS ( \
                     SELECT c.oid \
                       FROM pg_class c \
                       JOIN pg_am am ON am.oid = c.relam \
                      WHERE c.relkind = 'i' AND am.amname = 'ec_spire' \
                 ) \
                 DELETE FROM ec_spire_remote_node_descriptor d \
                  WHERE NOT EXISTS (SELECT 1 FROM live_spire_index l \
                                     WHERE l.oid = d.coordinator_index_oid)",
                None,
                &[],
            )
            .map_err(|e| format!("ec_spire remote descriptor orphan cleanup failed: {e}"))?;
        Ok::<(i64, i64, i64, i64, i64), String>(counts)
    });
    let (
        descriptor_removed_count,
        manifest_removed_count,
        manifest_entry_removed_count,
        row_materialization_removed_count,
        placement_removed_count,
    ) = result.unwrap_or_else(|e| pgrx::error!("{e}"));
    let removed_any = descriptor_removed_count > 0
        || manifest_removed_count > 0
        || manifest_entry_removed_count > 0
        || row_materialization_removed_count > 0
        || placement_removed_count > 0;
    let status = if removed_any {
        "removed_orphaned_remote_catalog_rows"
    } else {
        "ready"
    };

    TableIterator::once((
        descriptor_removed_count,
        manifest_removed_count,
        manifest_entry_removed_count,
        row_materialization_removed_count,
        placement_removed_count,
        status.to_owned(),
    ))
}

#[pg_extern]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_catalog_index_cleanup(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(descriptor_removed_count, i64),
        name!(manifest_removed_count, i64),
        name!(manifest_entry_removed_count, i64),
        name!(row_materialization_removed_count, i64),
        name!(placement_removed_count, i64),
        name!(applied_manifest_removed_count, i64),
        name!(applied_manifest_entry_removed_count, i64),
        name!(status, String),
    ),
> {
    let result = Spi::connect_mut(|client| {
        let counts = client
            .select(
                "SELECT \
                    (SELECT count(*)::bigint \
                       FROM ec_spire_remote_node_descriptor \
                      WHERE coordinator_index_oid = $1::oid) \
                        AS descriptor_removed_count, \
                    (SELECT count(*)::bigint \
                       FROM ec_spire_remote_epoch_manifest \
                      WHERE coordinator_index_oid = $1::oid) \
                        AS manifest_removed_count, \
                    (SELECT count(*)::bigint \
                       FROM ec_spire_remote_epoch_manifest_entry \
                      WHERE coordinator_index_oid = $1::oid) \
                        AS manifest_entry_removed_count, \
                    0::bigint AS row_materialization_removed_count, \
                    (SELECT count(*)::bigint \
                       FROM ec_spire_placement \
                      WHERE index_oid = $1::oid) \
                        AS placement_removed_count, \
                    (SELECT count(*)::bigint \
                       FROM ec_spire_remote_epoch_manifest_applied \
                      WHERE remote_index_oid = $1::oid) \
                        AS applied_manifest_removed_count, \
                    (SELECT count(*)::bigint \
                       FROM ec_spire_remote_epoch_manifest_applied_entry \
                      WHERE remote_index_oid = $1::oid) \
                        AS applied_manifest_entry_removed_count",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| format!("ec_spire remote catalog index cleanup count failed: {e}"))?
            .map(|row| {
                Ok::<(i64, i64, i64, i64, i64, i64, i64), String>((
                    row["descriptor_removed_count"]
                        .value::<i64>()
                        .map_err(|e| format!("descriptor removed count decode failed: {e}"))?
                        .ok_or_else(|| "descriptor removed count is null".to_owned())?,
                    row["manifest_removed_count"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest removed count decode failed: {e}"))?
                        .ok_or_else(|| "manifest removed count is null".to_owned())?,
                    row["manifest_entry_removed_count"]
                        .value::<i64>()
                        .map_err(|e| format!("manifest entry removed count decode failed: {e}"))?
                        .ok_or_else(|| "manifest entry removed count is null".to_owned())?,
                    row["row_materialization_removed_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("row materialization removed count decode failed: {e}")
                        })?
                        .ok_or_else(|| "row materialization removed count is null".to_owned())?,
                    row["placement_removed_count"]
                        .value::<i64>()
                        .map_err(|e| format!("placement removed count decode failed: {e}"))?
                        .ok_or_else(|| "placement removed count is null".to_owned())?,
                    row["applied_manifest_removed_count"]
                        .value::<i64>()
                        .map_err(|e| format!("applied manifest removed count decode failed: {e}"))?
                        .ok_or_else(|| "applied manifest removed count is null".to_owned())?,
                    row["applied_manifest_entry_removed_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("applied manifest entry removed count decode failed: {e}")
                        })?
                        .ok_or_else(|| "applied manifest entry removed count is null".to_owned())?,
                ))
            })
            .next()
            .transpose()?
            .unwrap_or((0, 0, 0, 0, 0, 0, 0));
        client
            .update(
                "DELETE FROM ec_spire_placement \
                  WHERE index_oid = $1::oid",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| format!("ec_spire placement index cleanup failed: {e}"))?;
        client
            .update(
                "DELETE FROM ec_spire_remote_epoch_manifest_applied \
                  WHERE remote_index_oid = $1::oid",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| format!("ec_spire remote catalog applied manifest cleanup failed: {e}"))?;
        client
            .update(
                "DELETE FROM ec_spire_remote_epoch_manifest \
                  WHERE coordinator_index_oid = $1::oid",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| format!("ec_spire remote catalog index manifest cleanup failed: {e}"))?;
        client
            .update(
                "DELETE FROM ec_spire_remote_node_descriptor \
                  WHERE coordinator_index_oid = $1::oid",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| format!("ec_spire remote catalog index descriptor cleanup failed: {e}"))?;
        Ok::<(i64, i64, i64, i64, i64, i64, i64), String>(counts)
    });
    let (
        descriptor_removed_count,
        manifest_removed_count,
        manifest_entry_removed_count,
        row_materialization_removed_count,
        placement_removed_count,
        applied_manifest_removed_count,
        applied_manifest_entry_removed_count,
    ) = result.unwrap_or_else(|e| pgrx::error!("{e}"));
    let removed_any = descriptor_removed_count > 0
        || manifest_removed_count > 0
        || manifest_entry_removed_count > 0
        || row_materialization_removed_count > 0
        || placement_removed_count > 0
        || applied_manifest_removed_count > 0
        || applied_manifest_entry_removed_count > 0;
    let status = if removed_any {
        "removed_index_remote_catalog_rows"
    } else {
        "ready"
    };

    TableIterator::once((
        descriptor_removed_count,
        manifest_removed_count,
        manifest_entry_removed_count,
        row_materialization_removed_count,
        placement_removed_count,
        applied_manifest_removed_count,
        applied_manifest_entry_removed_count,
        status.to_owned(),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_degradation_policy_contract() -> TableIterator<
    'static,
    (
        name!(consistency_mode, &'static str),
        name!(placement_state, &'static str),
        name!(search_action, &'static str),
        name!(publish_action, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let rows = am::spire_remote_degradation_policy_contract_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            row.consistency_mode,
            row.placement_state,
            row.search_action,
            row.publish_action,
            row.status,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_publication_contract() -> TableIterator<
    'static,
    (
        name!(step_ordinal, i64),
        name!(prerequisite, &'static str),
        name!(publication_action, &'static str),
        name!(required_status, &'static str),
        name!(validator, &'static str),
        name!(failure_status, &'static str),
    ),
> {
    let rows = am::spire_remote_epoch_manifest_publication_contract_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.step_ordinal).expect("step ordinal should fit in i64"),
            row.prerequisite,
            row.publication_action,
            row.required_status,
            row.validator,
            row.failure_status,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_epoch_manifest_publication_result_contract() -> TableIterator<
    'static,
    (
        name!(result_ordinal, i64),
        name!(result_source, &'static str),
        name!(publication_decision, &'static str),
        name!(status_family, &'static str),
        name!(semantic_role, &'static str),
        name!(validator, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    TableIterator::new(
        vec![
            (
                1_i64,
                "not_required",
                "not_required",
                "not_required",
                "local_only_manifest_publication_result",
                "must_have_zero_libpq_receive_count",
                "remote publication is not required for local-only manifests",
            ),
            (
                2_i64,
                "pending_libpq_executor",
                "publish_remote_epoch_manifest",
                "requires_libpq_executor",
                "distributed_manifest_waiting_for_transport_executor",
                "must_name_next_executor_step",
                "run the future libpq executor against the manifest work plan",
            ),
            (
                3_i64,
                "remote_manifest_validation_result",
                "publish_remote_epoch_manifest",
                "ready",
                "distributed_manifest_payload_validation_result",
                "must_match_manifest_result_contract",
                "v1: synthesize after the remote apply executor lands",
            ),
            (
                4_i64,
                "blocked",
                "any_blocked_publication_decision",
                "blocked",
                "pre_publication_gate_blocked_result",
                "must_preserve_original_blocker_status",
                "resolve the upstream publication blocker before remote apply",
            ),
        ]
        .into_iter(),
    )
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_scan_placement_snapshot(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(effective_nprobe, i64),
        name!(effective_nprobe_source, String),
        name!(effective_rerank_width, i64),
        name!(effective_rerank_width_source, String),
        name!(node_id, i64),
        name!(local_store_id, i64),
        name!(route_count, i64),
        name!(leaf_route_count, i64),
        name!(delta_route_count, i64),
        name!(prefetched_object_count, i64),
        name!(scanned_pid_count, i64),
        name!(leaf_pid_count, i64),
        name!(delta_pid_count, i64),
        name!(candidate_row_count, i64),
        name!(leaf_candidate_row_count, i64),
        name!(delta_candidate_row_count, i64),
        name!(primary_candidate_row_count, i64),
        name!(boundary_replica_candidate_row_count, i64),
        name!(deduped_candidate_row_count, i64),
        name!(deduped_primary_candidate_row_count, i64),
        name!(deduped_boundary_replica_candidate_row_count, i64),
        name!(truncated_candidate_row_count, i64),
        name!(truncated_primary_candidate_row_count, i64),
        name!(truncated_boundary_replica_candidate_row_count, i64),
        name!(candidate_winner_count, i64),
        name!(primary_candidate_winner_count, i64),
        name!(boundary_replica_candidate_winner_count, i64),
        name!(delete_delta_row_count, i64),
        name!(dropped_unselected_delta_route_count, i64),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_scan_placement_snapshot") };
    let rows = unsafe { am::spire_index_scan_placement_snapshot(index_relation, query) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::from(row.effective_nprobe),
            row.effective_nprobe_source.to_owned(),
            i64::try_from(row.effective_rerank_width)
                .expect("effective rerank width should fit in i64"),
            row.effective_rerank_width_source.to_owned(),
            i64::from(row.node_id),
            i64::from(row.local_store_id),
            i64::try_from(row.route_count).expect("route count should fit in i64"),
            i64::try_from(row.leaf_route_count).expect("leaf route count should fit in i64"),
            i64::try_from(row.delta_route_count).expect("delta route count should fit in i64"),
            i64::try_from(row.prefetched_object_count)
                .expect("prefetched object count should fit in i64"),
            i64::try_from(row.scanned_pid_count).expect("scanned pid count should fit in i64"),
            i64::try_from(row.leaf_pid_count).expect("leaf pid count should fit in i64"),
            i64::try_from(row.delta_pid_count).expect("delta pid count should fit in i64"),
            i64::try_from(row.candidate_row_count).expect("candidate row count should fit in i64"),
            i64::try_from(row.leaf_candidate_row_count)
                .expect("leaf candidate row count should fit in i64"),
            i64::try_from(row.delta_candidate_row_count)
                .expect("delta candidate row count should fit in i64"),
            i64::try_from(row.primary_candidate_row_count)
                .expect("primary candidate row count should fit in i64"),
            i64::try_from(row.boundary_replica_candidate_row_count)
                .expect("boundary replica candidate row count should fit in i64"),
            i64::try_from(row.deduped_candidate_row_count)
                .expect("deduped candidate row count should fit in i64"),
            i64::try_from(row.deduped_primary_candidate_row_count)
                .expect("deduped primary candidate row count should fit in i64"),
            i64::try_from(row.deduped_boundary_replica_candidate_row_count)
                .expect("deduped boundary replica candidate row count should fit in i64"),
            i64::try_from(row.truncated_candidate_row_count)
                .expect("truncated candidate row count should fit in i64"),
            i64::try_from(row.truncated_primary_candidate_row_count)
                .expect("truncated primary candidate row count should fit in i64"),
            i64::try_from(row.truncated_boundary_replica_candidate_row_count)
                .expect("truncated boundary replica candidate row count should fit in i64"),
            i64::try_from(row.candidate_winner_count)
                .expect("candidate winner count should fit in i64"),
            i64::try_from(row.primary_candidate_winner_count)
                .expect("primary candidate winner count should fit in i64"),
            i64::try_from(row.boundary_replica_candidate_winner_count)
                .expect("boundary replica candidate winner count should fit in i64"),
            i64::try_from(row.delete_delta_row_count)
                .expect("delete delta row count should fit in i64"),
            i64::try_from(row.dropped_unselected_delta_route_count)
                .expect("dropped unselected delta route count should fit in i64"),
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_selected_pid_placement_snapshot(
    index_oid: pg_sys::Oid,
    selected_pids: Vec<i64>,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(selection_ordinal, i64),
        name!(pid, i64),
        name!(node_id, i64),
        name!(local_store_id, i64),
        name!(store_relid, i64),
        name!(placement_state, &'static str),
        name!(object_version, i64),
        name!(object_bytes, i64),
    ),
> {
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_index_selected_pid_placement_snapshot selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_index_selected_pid_placement_snapshot")
    };
    let rows =
        unsafe { am::spire_index_selected_pid_placement_snapshot(index_relation, selected_pids) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::try_from(row.selection_ordinal).expect("selection ordinal should fit in i64"),
            i64::try_from(row.pid).expect("PID should fit in i64"),
            i64::from(row.node_id),
            i64::from(row.local_store_id),
            i64::from(row.store_relid),
            row.placement_state,
            i64::try_from(row.object_version).expect("object version should fit in i64"),
            i64::try_from(row.object_bytes).expect("object bytes should fit in i64"),
        )
    }))
}

#[pg_extern(stable, strict)]
fn ec_spire_index_scan_local_store_execution_snapshot(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(effective_nprobe, i64),
        name!(node_id, i64),
        name!(local_store_id, i64),
        name!(local_store_execution_mode, String),
        name!(local_store_read_ahead_primitive, String),
        name!(local_store_parallelism_next_step, String),
        name!(route_count, i64),
        name!(prefetched_object_count, i64),
        name!(scanned_pid_count, i64),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_index_scan_local_store_execution_snapshot",
        )
    };
    let rows = unsafe { am::spire_index_scan_placement_snapshot(index_relation, query) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::from(row.effective_nprobe),
            i64::from(row.node_id),
            i64::from(row.local_store_id),
            "sequential_backend".to_owned(),
            ec_spire_local_store_read_ahead_primitive_label().to_owned(),
            "async_or_parallel_store_group_executor".to_owned(),
            i64::try_from(row.route_count).expect("route count should fit in i64"),
            i64::try_from(row.prefetched_object_count)
                .expect("prefetched object count should fit in i64"),
            i64::try_from(row.scanned_pid_count).expect("scanned pid count should fit in i64"),
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_scan_local_store_read_overlap_harness(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(effective_nprobe, i64),
        name!(node_id, i64),
        name!(local_store_id, i64),
        name!(route_count, i64),
        name!(leaf_route_count, i64),
        name!(delta_route_count, i64),
        name!(candidate_row_count, i64),
        name!(prefetched_object_bytes, i64),
        name!(read_batch_count, i64),
        name!(delta_decode_count, i64),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_index_scan_local_store_read_overlap_harness",
        )
    };
    let rows = unsafe { am::spire_index_scan_placement_snapshot(index_relation, query) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::from(row.effective_nprobe),
            i64::from(row.node_id),
            i64::from(row.local_store_id),
            i64::try_from(row.route_count).expect("route count should fit in i64"),
            i64::try_from(row.leaf_route_count).expect("leaf route count should fit in i64"),
            i64::try_from(row.delta_route_count).expect("delta route count should fit in i64"),
            i64::try_from(row.candidate_row_count).expect("candidate row count should fit in i64"),
            i64::try_from(row.prefetched_object_bytes)
                .expect("prefetched object bytes should fit in i64"),
            i64::try_from(row.read_batch_count).expect("read batch count should fit in i64"),
            i64::try_from(row.delta_decode_count).expect("delta decode count should fit in i64"),
        )
    }))
}

#[cfg(feature = "pg18")]
fn ec_spire_local_store_read_ahead_primitive_label() -> &'static str {
    "pg18_read_stream"
}

#[cfg(not(feature = "pg18"))]
fn ec_spire_local_store_read_ahead_primitive_label() -> &'static str {
    "prefetch_buffer"
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_scan_routing_snapshot(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(effective_nprobe, i64),
        name!(effective_nprobe_source, String),
        name!(adaptive_nprobe_decision, String),
        name!(recursive_beam_width, i64),
        name!(max_leaf_routes, i64),
        name!(max_routing_expansions, i64),
        name!(routing_level, i64),
        name!(input_frontier_width, i64),
        name!(expanded_parent_count, i64),
        name!(selected_child_count, i64),
        name!(deduped_route_count, i64),
        name!(truncation_reason, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_scan_routing_snapshot") };
    let rows = unsafe { am::spire_index_scan_routing_snapshot(index_relation, query) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::from(row.effective_nprobe),
            row.effective_nprobe_source.to_owned(),
            row.adaptive_nprobe_decision.to_owned(),
            i64::try_from(row.recursive_beam_width)
                .expect("recursive beam width should fit in i64"),
            i64::try_from(row.max_leaf_routes).expect("max leaf routes should fit in i64"),
            i64::try_from(row.max_routing_expansions)
                .expect("max routing expansions should fit in i64"),
            i64::from(row.routing_level),
            i64::try_from(row.input_frontier_width)
                .expect("input frontier width should fit in i64"),
            i64::try_from(row.expanded_parent_count)
                .expect("expanded parent count should fit in i64"),
            i64::try_from(row.selected_child_count)
                .expect("selected child count should fit in i64"),
            i64::try_from(row.deduped_route_count).expect("deduped route count should fit in i64"),
            row.truncation_reason.to_owned(),
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_scan_pipeline_snapshot(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> TableIterator<
    'static,
    (
        name!(step_ordinal, i64),
        name!(step_name, &'static str),
        name!(active_epoch, i64),
        name!(status, &'static str),
        name!(item_count, i64),
        name!(ready_count, i64),
        name!(blocked_count, i64),
        name!(route_count, i64),
        name!(candidate_count, i64),
        name!(heap_rerank_row_count, i64),
        name!(remote_fanout_count, i64),
        name!(next_blocker, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if unsafe { !relation_oid_exists(index_oid) } {
        return TableIterator::new(Vec::new().into_iter());
    }
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_scan_pipeline_snapshot") };
    let routing_rows =
        unsafe { am::spire_index_scan_routing_snapshot(index_relation, query.clone()) };
    let placement_rows = unsafe { am::spire_index_scan_placement_snapshot(index_relation, query) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let active_epoch = placement_rows
        .first()
        .map(|row| row.active_epoch)
        .or_else(|| routing_rows.first().map(|row| row.active_epoch))
        .unwrap_or(0);
    let routing_route_count = routing_rows
        .last()
        .map(|row| row.deduped_route_count)
        .unwrap_or(0);
    let routing_truncated = routing_rows
        .iter()
        .any(|row| row.truncation_reason != "none");
    let store_count = u64::try_from(placement_rows.len()).expect("store count should fit in u64");
    let route_count = placement_rows
        .iter()
        .fold(0_u64, |count, row| count.saturating_add(row.route_count));
    let prefetched_object_count = placement_rows.iter().fold(0_u64, |count, row| {
        count.saturating_add(row.prefetched_object_count)
    });
    let candidate_count = placement_rows.iter().fold(0_u64, |count, row| {
        count.saturating_add(row.candidate_row_count)
    });
    let candidate_winner_count = placement_rows.iter().fold(0_u64, |count, row| {
        count.saturating_add(row.candidate_winner_count)
    });
    let truncated_candidate_count = placement_rows.iter().fold(0_u64, |count, row| {
        count.saturating_add(row.truncated_candidate_row_count)
    });
    let effective_rerank_width = placement_rows
        .iter()
        .map(|row| row.effective_rerank_width)
        .max()
        .unwrap_or(0);
    let heap_rerank_row_count = if effective_rerank_width == 0 {
        candidate_winner_count
    } else {
        candidate_winner_count.min(effective_rerank_width)
    };

    let active_epoch = i64::try_from(active_epoch).expect("active epoch should fit in i64");
    let routing_level_count =
        i64::try_from(routing_rows.len()).expect("routing level count should fit in i64");
    let store_count = i64::try_from(store_count).expect("store count should fit in i64");
    let route_count = i64::try_from(route_count).expect("route count should fit in i64");
    let routing_route_count =
        i64::try_from(routing_route_count).expect("routing route count should fit in i64");
    let prefetched_object_count =
        i64::try_from(prefetched_object_count).expect("prefetched object count should fit in i64");
    let candidate_count =
        i64::try_from(candidate_count).expect("candidate count should fit in i64");
    let candidate_winner_count =
        i64::try_from(candidate_winner_count).expect("candidate winner count should fit in i64");
    let truncated_candidate_count = i64::try_from(truncated_candidate_count)
        .expect("truncated candidate count should fit in i64");
    let heap_rerank_row_count =
        i64::try_from(heap_rerank_row_count).expect("heap rerank row count should fit in i64");

    let rows = vec![
        (
            1,
            "routing",
            active_epoch,
            if routing_truncated {
                "truncated"
            } else {
                "ready"
            },
            routing_level_count,
            routing_level_count,
            0,
            routing_route_count,
            0,
            0,
            0,
            if routing_truncated {
                "routing_budget"
            } else {
                "none"
            },
            if routing_truncated {
                "increase recursive route budget or inspect routing diagnostics"
            } else {
                "none"
            },
        ),
        (
            2,
            "placement",
            active_epoch,
            "ready",
            store_count,
            store_count,
            0,
            route_count,
            0,
            0,
            0,
            "none",
            "none",
        ),
        (
            3,
            "prefetch",
            active_epoch,
            "ready",
            prefetched_object_count,
            prefetched_object_count,
            0,
            route_count,
            0,
            0,
            0,
            "none",
            "none",
        ),
        (
            4,
            "candidates",
            active_epoch,
            if truncated_candidate_count > 0 {
                "truncated"
            } else {
                "ready"
            },
            candidate_count,
            candidate_winner_count,
            truncated_candidate_count,
            route_count,
            candidate_count,
            0,
            0,
            if truncated_candidate_count > 0 {
                "candidate_budget"
            } else {
                "none"
            },
            if truncated_candidate_count > 0 {
                "increase max_candidate_rows or inspect candidate diagnostics"
            } else {
                "none"
            },
        ),
        (
            5,
            "heap_rerank",
            active_epoch,
            "ready",
            heap_rerank_row_count,
            heap_rerank_row_count,
            0,
            0,
            candidate_winner_count,
            heap_rerank_row_count,
            0,
            "none",
            "none",
        ),
        (
            6,
            "remote_fanout",
            active_epoch,
            "not_applicable_local_scan",
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            "none",
            "use ec_spire_remote_pipeline_steps for remote fanout diagnostics",
        ),
    ];

    TableIterator::new(rows.into_iter())
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_root_routing_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(root_pid, i64),
        name!(root_object_version, i64),
        name!(root_level, i32),
        name!(root_child_count, i64),
        name!(centroid_dimensions, i32),
        name!(centroid_index, i64),
        name!(child_pid, i64),
        name!(child_kind, String),
        name!(child_object_version, i64),
        name!(child_level, i32),
        name!(child_parent_pid, i64),
        name!(child_assignment_count, i64),
        name!(child_node_id, i64),
        name!(child_local_store_id, i64),
        name!(child_store_relid, i64),
        name!(child_placement_state, String),
        name!(child_object_bytes, i64),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_root_routing_snapshot") };
    let rows = unsafe { am::spire_index_root_routing_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::try_from(row.root_pid).expect("root pid should fit in i64"),
            i64::try_from(row.root_object_version).expect("root object version should fit in i64"),
            i32::from(row.root_level),
            i64::try_from(row.root_child_count).expect("root child count should fit in i64"),
            i32::from(row.centroid_dimensions),
            i64::from(row.centroid_index),
            i64::try_from(row.child_pid).expect("child pid should fit in i64"),
            row.child_kind.to_owned(),
            i64::try_from(row.child_object_version)
                .expect("child object version should fit in i64"),
            i32::from(row.child_level),
            i64::try_from(row.child_parent_pid).expect("child parent pid should fit in i64"),
            i64::try_from(row.child_assignment_count)
                .expect("child assignment count should fit in i64"),
            i64::from(row.child_node_id),
            i64::from(row.child_local_store_id),
            i64::from(row.child_store_relid),
            row.child_placement_state.to_owned(),
            i64::try_from(row.child_object_bytes).expect("child object bytes should fit in i64"),
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_routing_centroid_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(parent_pid, i64),
        name!(parent_kind, String),
        name!(parent_object_version, i64),
        name!(parent_level, i32),
        name!(parent_child_count, i64),
        name!(centroid_dimensions, i32),
        name!(centroid_index, i64),
        name!(child_pid, i64),
        name!(child_kind, String),
        name!(child_object_version, i64),
        name!(child_level, i32),
        name!(child_parent_pid, i64),
        name!(child_assignment_count, i64),
        name!(child_node_id, i64),
        name!(child_local_store_id, i64),
        name!(child_store_relid, i64),
        name!(child_placement_state, String),
        name!(child_object_bytes, i64),
        name!(centroid, Vec<f32>),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_routing_centroid_snapshot") };
    let rows = unsafe { am::spire_index_routing_centroid_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::try_from(row.parent_pid).expect("parent pid should fit in i64"),
            row.parent_kind.to_owned(),
            i64::try_from(row.parent_object_version)
                .expect("parent object version should fit in i64"),
            i32::from(row.parent_level),
            i64::try_from(row.parent_child_count).expect("parent child count should fit in i64"),
            i32::from(row.centroid_dimensions),
            i64::from(row.centroid_index),
            i64::try_from(row.child_pid).expect("child pid should fit in i64"),
            row.child_kind.to_owned(),
            i64::try_from(row.child_object_version)
                .expect("child object version should fit in i64"),
            i32::from(row.child_level),
            i64::try_from(row.child_parent_pid).expect("child parent pid should fit in i64"),
            i64::try_from(row.child_assignment_count)
                .expect("child assignment count should fit in i64"),
            i64::from(row.child_node_id),
            i64::from(row.child_local_store_id),
            i64::from(row.child_store_relid),
            row.child_placement_state.to_owned(),
            i64::try_from(row.child_object_bytes).expect("child object bytes should fit in i64"),
            row.centroid,
        )
    }))
}

#[pg_extern(stable, strict)]
fn ec_spire_classify_centroid(
    embedding: Vec<f32>,
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(node_id, i64),
        name!(centroid_id, i64),
        name!(epoch, i64),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_classify_centroid") };
    let classification = unsafe { am::spire_classify_centroid(index_relation, &embedding) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let (node_id, centroid_id, epoch) = classification.unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::once((
        i64::from(node_id),
        i64::try_from(centroid_id).expect("centroid id should fit in i64"),
        i64::try_from(epoch).expect("epoch should fit in i64"),
    ))
}

#[pg_extern(stable, strict)]
fn ec_spire_plan_coordinator_insert(
    index_oid: pg_sys::Oid,
    pk_value: Vec<u8>,
    embedding: Vec<f32>,
    source_identity: Vec<u8>,
) -> TableIterator<
    'static,
    (
        name!(index_oid, pg_sys::Oid),
        name!(pk_value, Vec<u8>),
        name!(node_id, i64),
        name!(centroid_id, i64),
        name!(served_epoch, i64),
        name!(source_identity, Vec<u8>),
    ),
> {
    if pk_value.is_empty() {
        pgrx::error!("ec_spire_plan_coordinator_insert pk_value must not be empty");
    }
    if source_identity.len() != 16 {
        pgrx::error!("ec_spire_plan_coordinator_insert source_identity must be exactly 16 bytes");
    }

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_plan_coordinator_insert") };
    let classification = unsafe { am::spire_classify_centroid(index_relation, &embedding) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let (node_id, centroid_id, epoch) = classification.unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::once((
        index_oid,
        pk_value,
        i64::from(node_id),
        i64::try_from(centroid_id).expect("centroid id should fit in i64"),
        i64::try_from(epoch).expect("served epoch should fit in i64"),
        source_identity,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_plan_coordinator_insert_dispatch(
    index_oid: pg_sys::Oid,
    node_id: i32,
    served_epoch: i64,
) -> TableIterator<
    'static,
    (
        name!(index_oid, pg_sys::Oid),
        name!(node_id, i64),
        name!(served_epoch, i64),
        name!(dispatch_transport, &'static str),
        name!(transaction_protocol, &'static str),
        name!(conninfo_secret_name, String),
        name!(conninfo_provider_lookup_key, String),
        name!(remote_index_regclass, String),
        name!(descriptor_generation, i64),
        name!(remote_index_identity_bytes, i64),
        name!(dispatch_action, &'static str),
        name!(status, &'static str),
        name!(next_step, &'static str),
    ),
> {
    if node_id <= 0 {
        pgrx::error!("ec_spire_plan_coordinator_insert_dispatch node_id must be greater than 0");
    }
    if served_epoch <= 0 {
        pgrx::error!(
            "ec_spire_plan_coordinator_insert_dispatch served_epoch must be greater than 0"
        );
    }

    let node_id = u32::try_from(node_id).expect("positive node_id should fit in u32");
    let served_epoch =
        u64::try_from(served_epoch).expect("positive served_epoch should fit in u64");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_plan_coordinator_insert_dispatch")
    };
    let row = unsafe {
        am::spire_coordinator_insert_dispatch_plan_row(index_relation, node_id, served_epoch)
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        row.index_oid,
        i64::from(row.node_id),
        i64::try_from(row.served_epoch).expect("served epoch should fit in i64"),
        row.dispatch_transport,
        row.transaction_protocol,
        row.conninfo_secret_name,
        row.conninfo_provider_lookup_key,
        row.remote_index_regclass,
        i64::try_from(row.descriptor_generation).expect("descriptor generation should fit in i64"),
        i64::try_from(row.remote_index_identity_bytes)
            .expect("remote index identity byte count should fit in i64"),
        row.dispatch_action,
        row.status,
        row.next_step,
    ))
}

#[pg_extern(strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_prepare_coordinator_insert_tuple_payload(
    index_oid: pg_sys::Oid,
    pk_value: Vec<u8>,
    embedding: Vec<f32>,
    source_identity: Vec<u8>,
    row_payload: pgrx::JsonB,
    requested_columns: Vec<String>,
) -> TableIterator<
    'static,
    (
        name!(index_oid, pg_sys::Oid),
        name!(pk_value, Vec<u8>),
        name!(node_id, i64),
        name!(centroid_id, i64),
        name!(served_epoch, i64),
        name!(source_identity, Vec<u8>),
        name!(prepared_gid, String),
        name!(remote_insert_sent, bool),
        name!(remote_prepared, bool),
        name!(placement_staged, bool),
        name!(status, &'static str),
        name!(next_step, &'static str),
    ),
> {
    const POST_STAGING_STATUS: &str = "remote_insert_prepared_pending_local_commit";
    const POST_STAGING_NEXT_STEP: &str = "await_local_commit";

    if pk_value.is_empty() {
        pgrx::error!(
            "ec_spire_prepare_coordinator_insert_tuple_payload pk_value must not be empty"
        );
    }
    if source_identity.len() != 16 {
        pgrx::error!(
            "ec_spire_prepare_coordinator_insert_tuple_payload source_identity must be exactly 16 bytes"
        );
    }

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_prepare_coordinator_insert_tuple_payload",
        )
    };
    let (node_id, centroid_id, served_epoch) =
        unsafe { am::spire_classify_centroid(index_relation, &embedding) }
            .unwrap_or_else(|e| pgrx::error!("{e}"));
    let row_payload_json = row_payload.0.to_string();
    let prepare_row = unsafe {
        am::spire_coordinator_insert_prepare_remote_tuple_payload(
            index_relation,
            node_id,
            served_epoch,
            &row_payload_json,
            &requested_columns,
        )
    }
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    Spi::connect_mut(|client| {
        let descriptor_refreshed = client
            .select(
                "UPDATE ec_spire_remote_node_descriptor \
                    SET descriptor_generation = $3::bigint, \
                        remote_index_identity = $4::bytea, \
                        last_seen_at = clock_timestamp(), \
                        last_served_epoch = $5::bigint, \
                        min_retained_epoch = $6::bigint, \
                        extension_version = $7::text, \
                        last_error = 'none' \
                  WHERE coordinator_index_oid = $1::oid \
                    AND node_id = $2::integer \
                    AND descriptor_generation < $3::bigint \
              RETURNING true AS refreshed",
                None,
                &[
                    index_oid.into(),
                    i32::try_from(node_id)
                        .expect("coordinator insert node_id should fit i32")
                        .into(),
                    i64::try_from(prepare_row.descriptor_generation)
                        .expect("descriptor generation should fit i64")
                        .into(),
                    prepare_row.remote_index_identity.clone().into(),
                    i64::try_from(prepare_row.remote_last_served_epoch)
                        .expect("remote last served epoch should fit i64")
                        .into(),
                    i64::try_from(prepare_row.remote_min_retained_epoch)
                        .expect("remote min retained epoch should fit i64")
                        .into(),
                    prepare_row.remote_extension_version.as_str().into(),
                ],
            )
            .map_err(|e| format!("ec_spire coordinator insert descriptor refresh failed: {e}"))?
            .map(|row| {
                row["refreshed"]
                    .value::<bool>()
                    .map_err(|e| {
                        format!("ec_spire coordinator insert descriptor refresh decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire coordinator insert descriptor refresh result is null".to_owned()
                    })
            })
            .next()
            .transpose()?
            .unwrap_or(false);
        if !descriptor_refreshed {
            pgrx::ereport!(
                ERROR,
                pgrx::PgSqlErrorCode::ERRCODE_T_R_SERIALIZATION_FAILURE,
                "ec_spire_register_remote_node_descriptor descriptor_generation must advance existing descriptor_generation",
                "Retry the whole coordinator write after the winning descriptor refresh commits."
            );
        }
        client
            .update(
                "INSERT INTO ec_spire_placement \
                     (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
                 VALUES ($1::oid, $2::bytea, $3::integer, $4::bigint, $5::bigint, $6::bytea)",
                None,
                &[
                    index_oid.into(),
                    pk_value.clone().into(),
                    i32::try_from(node_id)
                        .expect("coordinator insert node_id should fit i32")
                        .into(),
                    i64::try_from(centroid_id)
                        .expect("coordinator insert centroid_id should fit i64")
                        .into(),
                    i64::try_from(served_epoch)
                        .expect("coordinator insert served_epoch should fit i64")
                        .into(),
                    source_identity.clone().into(),
                ],
            )
            .map_err(|e| format!("ec_spire coordinator insert placement staging failed: {e}"))?;
        Ok::<(), String>(())
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::once((
        index_oid,
        pk_value,
        i64::from(node_id),
        i64::try_from(centroid_id).expect("centroid id should fit in i64"),
        i64::try_from(served_epoch).expect("served epoch should fit in i64"),
        source_identity,
        prepare_row.prepared_gid,
        prepare_row.remote_insert_sent,
        prepare_row.remote_prepared,
        true,
        POST_STAGING_STATUS,
        POST_STAGING_NEXT_STEP,
    ))
}

#[pg_extern(strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_prepare_coordinator_insert_tuple_payload_batch(
    index_oid: pg_sys::Oid,
    pk_value_hex_values: Vec<String>,
    node_ids: Vec<i32>,
    centroid_ids: Vec<i64>,
    served_epochs: Vec<i64>,
    source_identity_hex_values: Vec<String>,
    row_payload_json_values: Vec<String>,
    requested_columns: Vec<String>,
) -> TableIterator<
    'static,
    (
        name!(index_oid, pg_sys::Oid),
        name!(pk_value, Vec<u8>),
        name!(node_id, i64),
        name!(centroid_id, i64),
        name!(served_epoch, i64),
        name!(source_identity, Vec<u8>),
        name!(prepared_gid, String),
        name!(remote_insert_sent, bool),
        name!(remote_prepared, bool),
        name!(placement_staged, bool),
        name!(status, &'static str),
        name!(next_step, &'static str),
    ),
> {
    const POST_STAGING_STATUS: &str = "remote_insert_prepared_pending_local_commit";
    const POST_STAGING_NEXT_STEP: &str = "await_local_commit";

    let row_count = pk_value_hex_values.len();
    if row_count == 0 {
        pgrx::error!("ec_spire_prepare_coordinator_insert_tuple_payload_batch row list is empty");
    }
    for (label, len) in [
        ("node_ids", node_ids.len()),
        ("centroid_ids", centroid_ids.len()),
        ("served_epochs", served_epochs.len()),
        (
            "source_identity_hex_values",
            source_identity_hex_values.len(),
        ),
        ("row_payload_json_values", row_payload_json_values.len()),
    ] {
        if len != row_count {
            pgrx::error!(
                "ec_spire_prepare_coordinator_insert_tuple_payload_batch {label} length {len} does not match pk_value_hex_values length {row_count}"
            );
        }
    }
    if requested_columns.is_empty() {
        pgrx::error!(
            "ec_spire_prepare_coordinator_insert_tuple_payload_batch requested_columns must not be empty"
        );
    }
    let pk_values = pk_value_hex_values
        .iter()
        .map(|value| {
            hex::decode(value).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_prepare_coordinator_insert_tuple_payload_batch pk_value hex is invalid"
                )
            })
        })
        .collect::<Vec<_>>();
    let source_identities = source_identity_hex_values
        .iter()
        .map(|value| {
            hex::decode(value).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_prepare_coordinator_insert_tuple_payload_batch source_identity hex is invalid"
                )
            })
        })
        .collect::<Vec<_>>();
    if pk_values.iter().any(Vec::is_empty) {
        pgrx::error!(
            "ec_spire_prepare_coordinator_insert_tuple_payload_batch pk_values must not contain empty bytea values"
        );
    }
    if source_identities
        .iter()
        .any(|source_identity| source_identity.len() != 16)
    {
        pgrx::error!(
            "ec_spire_prepare_coordinator_insert_tuple_payload_batch source identities must be exactly 16 bytes"
        );
    }

    let batch_rows = node_ids
        .iter()
        .zip(served_epochs.iter())
        .zip(row_payload_json_values.iter())
        .map(|((node_id, served_epoch), row_payload_json)| {
            if *node_id <= 0 {
                pgrx::error!(
                    "ec_spire_prepare_coordinator_insert_tuple_payload_batch node_id must be greater than 0"
                );
            }
            if *served_epoch <= 0 {
                pgrx::error!(
                    "ec_spire_prepare_coordinator_insert_tuple_payload_batch served_epoch must be greater than 0"
                );
            }
            (
                u32::try_from(*node_id).expect("positive node_id should fit u32"),
                u64::try_from(*served_epoch).expect("positive served_epoch should fit u64"),
                row_payload_json.clone(),
            )
        })
        .collect::<Vec<_>>();

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_prepare_coordinator_insert_tuple_payload_batch",
        )
    };
    let prepare_rows = unsafe {
        am::spire_coordinator_insert_prepare_remote_tuple_payload_batch(
            index_relation,
            batch_rows,
            &requested_columns,
        )
    }
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    if prepare_rows.len() != row_count {
        pgrx::error!(
            "ec_spire_prepare_coordinator_insert_tuple_payload_batch prepared {} rows for {row_count} inputs",
            prepare_rows.len()
        );
    }

    Spi::connect_mut(|client| {
        let mut refreshed_descriptors = std::collections::HashSet::<(i32, i64)>::new();
        for (((((pk_value, node_id), centroid_id), served_epoch), source_identity), prepare_row) in
            pk_values
                .iter()
                .zip(node_ids.iter())
                .zip(centroid_ids.iter())
                .zip(served_epochs.iter())
                .zip(source_identities.iter())
                .zip(prepare_rows.iter())
        {
            let prepared_node_id =
                i32::try_from(prepare_row.node_id).expect("prepared node_id should fit i32");
            if prepared_node_id != *node_id {
                pgrx::error!(
                    "ec_spire_prepare_coordinator_insert_tuple_payload_batch prepared node_id {prepared_node_id} does not match planned node_id {node_id}"
                );
            }
            let descriptor_generation = i64::try_from(prepare_row.descriptor_generation)
                .expect("descriptor generation should fit i64");
            if refreshed_descriptors.insert((prepared_node_id, descriptor_generation)) {
                let descriptor_refreshed = client
                    .select(
                        "UPDATE ec_spire_remote_node_descriptor \
                            SET descriptor_generation = $3::bigint, \
                                remote_index_identity = $4::bytea, \
                                last_seen_at = clock_timestamp(), \
                                last_served_epoch = $5::bigint, \
                                min_retained_epoch = $6::bigint, \
                                extension_version = $7::text, \
                                last_error = 'none' \
                          WHERE coordinator_index_oid = $1::oid \
                            AND node_id = $2::integer \
                            AND descriptor_generation < $3::bigint \
                      RETURNING true AS refreshed",
                        None,
                        &[
                            index_oid.into(),
                            prepared_node_id.into(),
                            descriptor_generation.into(),
                            prepare_row.remote_index_identity.clone().into(),
                            i64::try_from(prepare_row.remote_last_served_epoch)
                                .expect("remote last served epoch should fit i64")
                                .into(),
                            i64::try_from(prepare_row.remote_min_retained_epoch)
                                .expect("remote min retained epoch should fit i64")
                                .into(),
                            prepare_row.remote_extension_version.as_str().into(),
                        ],
                    )
                    .map_err(|e| {
                        format!("ec_spire coordinator insert descriptor refresh failed: {e}")
                    })?
                    .map(|row| {
                        row["refreshed"]
                            .value::<bool>()
                            .map_err(|e| {
                                format!(
                                    "ec_spire coordinator insert descriptor refresh decode failed: {e}"
                                )
                            })?
                            .ok_or_else(|| {
                                "ec_spire coordinator insert descriptor refresh result is null"
                                    .to_owned()
                            })
                    })
                    .next()
                    .transpose()?
                    .unwrap_or(false);
                if !descriptor_refreshed {
                    pgrx::ereport!(
                        ERROR,
                        pgrx::PgSqlErrorCode::ERRCODE_T_R_SERIALIZATION_FAILURE,
                        "ec_spire_register_remote_node_descriptor descriptor_generation must advance existing descriptor_generation",
                        "Retry the whole coordinator write after the winning descriptor refresh commits."
                    );
                }
            }
            client
                .update(
                    "INSERT INTO ec_spire_placement \
                         (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
                     VALUES ($1::oid, $2::bytea, $3::integer, $4::bigint, $5::bigint, $6::bytea)",
                    None,
                    &[
                        index_oid.into(),
                        pk_value.clone().into(),
                        (*node_id).into(),
                        (*centroid_id).into(),
                        (*served_epoch).into(),
                        source_identity.clone().into(),
                    ],
                )
                .map_err(|e| format!("ec_spire coordinator insert placement staging failed: {e}"))?;
        }
        Ok::<(), String>(())
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    let output_rows = pk_values
        .into_iter()
        .zip(node_ids)
        .zip(centroid_ids)
        .zip(served_epochs)
        .zip(source_identities)
        .zip(prepare_rows)
        .map(
            |(
                ((((pk_value, node_id), centroid_id), served_epoch), source_identity),
                prepare_row,
            )| {
                (
                    index_oid,
                    pk_value,
                    i64::from(node_id),
                    centroid_id,
                    served_epoch,
                    source_identity,
                    prepare_row.prepared_gid,
                    prepare_row.remote_insert_sent,
                    prepare_row.remote_prepared,
                    true,
                    POST_STAGING_STATUS,
                    POST_STAGING_NEXT_STEP,
                )
            },
        )
        .collect::<Vec<_>>();
    TableIterator::new(output_rows.into_iter())
}

#[pg_extern(strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_forward_coordinator_update_tuple_payload(
    index_oid: pg_sys::Oid,
    pk_column: String,
    pk_value: Vec<u8>,
    row_payload: pgrx::JsonB,
    updated_columns: Vec<String>,
) -> TableIterator<
    'static,
    (
        name!(index_oid, pg_sys::Oid),
        name!(pk_value, Vec<u8>),
        name!(node_id, i64),
        name!(served_epoch, i64),
        name!(remote_update_sent, bool),
        name!(remote_updated_count, i64),
        name!(status, &'static str),
        name!(next_step, &'static str),
    ),
> {
    if pk_column.is_empty() {
        pgrx::error!(
            "ec_spire_forward_coordinator_update_tuple_payload pk_column must be nonempty"
        );
    }
    if pk_value.is_empty() {
        pgrx::error!(
            "ec_spire_forward_coordinator_update_tuple_payload pk_value must not be empty"
        );
    }
    if updated_columns.is_empty() {
        pgrx::error!(
            "ec_spire_forward_coordinator_update_tuple_payload updated column list must be nonempty"
        );
    }
    if updated_columns.iter().any(|column| column == &pk_column) {
        pgrx::error!(
            "ec_spire_forward_coordinator_update_tuple_payload must not update the primary-key column"
        );
    }
    let index_key_columns = ec_spire_index_key_column_names(
        index_oid,
        "ec_spire_forward_coordinator_update_tuple_payload",
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    if updated_columns.iter().any(|column| {
        index_key_columns
            .iter()
            .any(|key_column| key_column == column)
    }) {
        ec_spire_reject_distributed_embedding_update();
    }

    let (node_id, served_epoch) = Spi::connect(|client| {
        client
            .select(
                "SELECT node_id, served_epoch \
                   FROM ec_spire_placement \
                  WHERE index_oid = $1::oid \
                    AND pk_value = $2::bytea",
                None,
                &[index_oid.into(), pk_value.clone().into()],
            )
            .map_err(|e| format!("ec_spire coordinator update placement lookup failed: {e}"))?
            .map(|row| {
                let node_id = row["node_id"]
                    .value::<i32>()
                    .map_err(|e| {
                        format!("ec_spire coordinator update placement node_id decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire coordinator update placement node_id is null".to_owned()
                    })?;
                let served_epoch = row["served_epoch"]
                    .value::<i64>()
                    .map_err(|e| {
                        format!(
                            "ec_spire coordinator update placement served_epoch decode failed: {e}"
                        )
                    })?
                    .ok_or_else(|| {
                        "ec_spire coordinator update placement served_epoch is null".to_owned()
                    })?;
                Ok::<(i32, i64), String>((node_id, served_epoch))
            })
            .next()
            .transpose()
            .map(|value| {
                value.ok_or_else(|| {
                    "ec_spire coordinator update placement row is missing".to_owned()
                })
            })?
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    if served_epoch <= 0 {
        pgrx::error!("ec_spire coordinator update placement served_epoch must be greater than 0");
    }

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_forward_coordinator_update_tuple_payload",
        )
    };
    let row_payload_json = row_payload.0.to_string();
    if node_id == 0 {
        let heap_relation_oid = unsafe {
            (*index_relation)
                .rd_index
                .as_ref()
                .expect("opened index relation should expose pg_index metadata")
                .indrelid
        };
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        let updated_count = ec_spire_update_tuple_payload_on_heap(
            heap_relation_oid,
            &pk_column,
            &pk_value,
            &row_payload_json,
            &updated_columns,
            "ec_spire_forward_coordinator_update_tuple_payload",
        )
        .unwrap_or_else(|e| pgrx::error!("{e}"));
        return TableIterator::once((
            index_oid,
            pk_value,
            0,
            served_epoch,
            false,
            i64::try_from(updated_count).expect("updated count should fit i64"),
            "local_update_applied",
            "done",
        ));
    }
    if node_id < 0 {
        pgrx::error!("ec_spire coordinator update placement node_id must not be negative");
    }
    let update_row = unsafe {
        am::spire_coordinator_update_remote_tuple_payload(
            index_relation,
            u32::try_from(node_id).expect("positive node_id should fit u32"),
            u64::try_from(served_epoch).expect("positive served_epoch should fit u64"),
            &pk_column,
            &pk_value,
            &row_payload_json,
            &updated_columns,
        )
    }
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    TableIterator::once((
        index_oid,
        pk_value,
        i64::from(update_row.node_id),
        served_epoch,
        update_row.remote_update_sent,
        i64::try_from(update_row.remote_updated_count).expect("updated count should fit i64"),
        update_row.status,
        update_row.next_step,
    ))
}

#[pg_extern(strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_prepare_coordinator_delete_tuple_payload(
    index_oid: pg_sys::Oid,
    pk_column: String,
    pk_value: Vec<u8>,
) -> TableIterator<
    'static,
    (
        name!(index_oid, pg_sys::Oid),
        name!(pk_value, Vec<u8>),
        name!(node_id, i64),
        name!(served_epoch, i64),
        name!(prepared_gid, String),
        name!(remote_delete_sent, bool),
        name!(remote_prepared, bool),
        name!(remote_deleted_count, i64),
        name!(placement_deleted, bool),
        name!(status, &'static str),
        name!(next_step, &'static str),
    ),
> {
    const POST_DELETE_STATUS: &str = "remote_delete_prepared_pending_local_commit";
    const POST_DELETE_NEXT_STEP: &str = "await_local_commit";

    if pk_column.is_empty() {
        pgrx::error!(
            "ec_spire_prepare_coordinator_delete_tuple_payload pk_column must be nonempty"
        );
    }
    if pk_value.is_empty() {
        pgrx::error!(
            "ec_spire_prepare_coordinator_delete_tuple_payload pk_value must not be empty"
        );
    }

    let placement = Spi::connect(|client| {
        client
            .select(
                "SELECT node_id, served_epoch \
                   FROM ec_spire_placement \
                  WHERE index_oid = $1::oid \
                    AND pk_value = $2::bytea",
                None,
                &[index_oid.into(), pk_value.clone().into()],
            )
            .map_err(|e| format!("ec_spire coordinator delete placement lookup failed: {e}"))?
            .map(|row| {
                let node_id = row["node_id"]
                    .value::<i32>()
                    .map_err(|e| {
                        format!("ec_spire coordinator delete placement node_id decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire coordinator delete placement node_id is null".to_owned()
                    })?;
                let served_epoch = row["served_epoch"]
                    .value::<i64>()
                    .map_err(|e| {
                        format!(
                            "ec_spire coordinator delete placement served_epoch decode failed: {e}"
                        )
                    })?
                    .ok_or_else(|| {
                        "ec_spire coordinator delete placement served_epoch is null".to_owned()
                    })?;
                Ok::<(i32, i64), String>((node_id, served_epoch))
            })
            .next()
            .transpose()
            .map_err(|e| format!("ec_spire coordinator delete placement lookup failed: {e}"))
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    let Some((node_id, served_epoch)) = placement else {
        // Negative node_id is a result-row sentinel for "no routing happened";
        // it is never written to ec_spire_placement.
        return TableIterator::once((
            index_oid,
            pk_value,
            -1,
            0,
            "none".to_owned(),
            false,
            false,
            0,
            false,
            "delete_not_found_noop",
            "done",
        ));
    };
    if node_id < 0 {
        pgrx::error!("ec_spire coordinator delete placement node_id must not be negative");
    }
    if served_epoch <= 0 {
        pgrx::error!("ec_spire coordinator delete placement served_epoch must be greater than 0");
    }

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_prepare_coordinator_delete_tuple_payload",
        )
    };
    if node_id == 0 {
        let heap_relation_oid = unsafe {
            (*index_relation)
                .rd_index
                .as_ref()
                .expect("opened index relation should expose pg_index metadata")
                .indrelid
        };
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        let deleted_count = ec_spire_delete_tuple_payload_on_heap(
            heap_relation_oid,
            &pk_column,
            &pk_value,
            "ec_spire_prepare_coordinator_delete_tuple_payload",
        )
        .unwrap_or_else(|e| pgrx::error!("{e}"));
        if deleted_count > 1 {
            pgrx::error!(
                "ec_spire coordinator local delete expected at most one row, got {}",
                deleted_count
            );
        }
        let placement_deleted = ec_spire_delete_placement_row(index_oid, &pk_value)
            .unwrap_or_else(|e| pgrx::error!("{e}"));
        return TableIterator::once((
            index_oid,
            pk_value,
            0,
            served_epoch,
            "none".to_owned(),
            false,
            false,
            i64::try_from(deleted_count).expect("deleted count should fit i64"),
            placement_deleted,
            if deleted_count == 0 {
                "local_delete_not_found_noop"
            } else {
                "local_delete_applied"
            },
            "done",
        ));
    }
    let delete_row = unsafe {
        am::spire_coordinator_delete_prepare_remote_tuple_payload(
            index_relation,
            u32::try_from(node_id).expect("positive node_id should fit u32"),
            u64::try_from(served_epoch).expect("positive served_epoch should fit u64"),
            &pk_column,
            &pk_value,
        )
    }
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    if delete_row.remote_deleted_count > 1 {
        pgrx::error!(
            "ec_spire coordinator delete expected at most one remote row, got {}",
            delete_row.remote_deleted_count
        );
    }

    let placement_deleted =
        ec_spire_delete_placement_row(index_oid, &pk_value).unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::once((
        index_oid,
        pk_value,
        i64::from(delete_row.node_id),
        served_epoch,
        delete_row.prepared_gid,
        delete_row.remote_delete_sent,
        delete_row.remote_prepared,
        i64::try_from(delete_row.remote_deleted_count).expect("deleted count should fit i64"),
        placement_deleted,
        if delete_row.remote_deleted_count == 0 {
            "remote_delete_not_found_prepared_pending_local_commit"
        } else {
            POST_DELETE_STATUS
        },
        POST_DELETE_NEXT_STEP,
    ))
}

#[pg_extern(strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_forward_coordinator_select_tuple_payload(
    index_oid: pg_sys::Oid,
    pk_column: String,
    pk_value: Vec<u8>,
    requested_columns: Vec<String>,
) -> TableIterator<
    'static,
    (
        name!(index_oid, pg_sys::Oid),
        name!(pk_value, Vec<u8>),
        name!(node_id, i64),
        name!(served_epoch, i64),
        name!(remote_select_sent, bool),
        name!(selected_count, i64),
        name!(payload_column_count, i32),
        name!(tuple_payload_json, Option<String>),
        name!(status, &'static str),
        name!(next_step, &'static str),
    ),
> {
    if pk_column.is_empty() {
        pgrx::error!(
            "ec_spire_forward_coordinator_select_tuple_payload pk_column must be nonempty"
        );
    }
    if pk_value.is_empty() {
        pgrx::error!(
            "ec_spire_forward_coordinator_select_tuple_payload pk_value must not be empty"
        );
    }
    if requested_columns.is_empty() {
        pgrx::error!(
            "ec_spire_forward_coordinator_select_tuple_payload requested column list must be nonempty"
        );
    }
    let payload_column_count = i32::try_from(requested_columns.len()).unwrap_or_else(|_| {
        pgrx::error!("ec_spire_forward_coordinator_select_tuple_payload too many requested columns")
    });

    let (node_id, served_epoch) = Spi::connect(|client| {
        client
            .select(
                "SELECT node_id, served_epoch \
                   FROM ec_spire_placement \
                  WHERE index_oid = $1::oid \
                    AND pk_value = $2::bytea",
                None,
                &[index_oid.into(), pk_value.clone().into()],
            )
            .map_err(|e| format!("ec_spire coordinator select placement lookup failed: {e}"))?
            .map(|row| {
                let node_id = row["node_id"]
                    .value::<i32>()
                    .map_err(|e| {
                        format!("ec_spire coordinator select placement node_id decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire coordinator select placement node_id is null".to_owned()
                    })?;
                let served_epoch = row["served_epoch"]
                    .value::<i64>()
                    .map_err(|e| {
                        format!(
                            "ec_spire coordinator select placement served_epoch decode failed: {e}"
                        )
                    })?
                    .ok_or_else(|| {
                        "ec_spire coordinator select placement served_epoch is null".to_owned()
                    })?;
                Ok::<(i32, i64), String>((node_id, served_epoch))
            })
            .next()
            .transpose()
            .map(|value| {
                value.ok_or_else(|| {
                    "ec_spire coordinator select placement row is missing".to_owned()
                })
            })?
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    if served_epoch <= 0 {
        pgrx::error!("ec_spire coordinator select placement served_epoch must be greater than 0");
    }

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_forward_coordinator_select_tuple_payload",
        )
    };
    if node_id == 0 {
        let heap_relation_oid = unsafe {
            (*index_relation)
                .rd_index
                .as_ref()
                .expect("opened index relation should expose pg_index metadata")
                .indrelid
        };
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        let (selected_count, tuple_payload_json) = ec_spire_select_tuple_payload_on_heap(
            heap_relation_oid,
            &pk_column,
            &pk_value,
            &requested_columns,
            "ec_spire_forward_coordinator_select_tuple_payload",
        )
        .unwrap_or_else(|e| pgrx::error!("{e}"));
        if selected_count > 1 {
            pgrx::error!(
                "ec_spire coordinator select expected at most one row, got {}",
                selected_count
            );
        }
        return TableIterator::once((
            index_oid,
            pk_value,
            0,
            served_epoch,
            false,
            i64::try_from(selected_count).expect("selected count should fit i64"),
            payload_column_count,
            tuple_payload_json,
            "local_select_ready",
            "done",
        ));
    }
    if node_id < 0 {
        pgrx::error!("ec_spire coordinator select placement node_id must not be negative");
    }
    let select_row = unsafe {
        am::spire_coordinator_select_remote_tuple_payload(
            index_relation,
            u32::try_from(node_id).expect("positive node_id should fit u32"),
            u64::try_from(served_epoch).expect("positive served_epoch should fit u64"),
            &pk_column,
            &pk_value,
            &requested_columns,
        )
    }
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    if select_row.remote_selected_count > 1 {
        pgrx::error!(
            "ec_spire coordinator select expected at most one remote row, got {}",
            select_row.remote_selected_count
        );
    }

    TableIterator::once((
        index_oid,
        pk_value,
        i64::from(select_row.node_id),
        served_epoch,
        select_row.remote_select_sent,
        i64::try_from(select_row.remote_selected_count).expect("selected count should fit i64"),
        payload_column_count,
        select_row.tuple_payload_json,
        select_row.status,
        select_row.next_step,
    ))
}

#[pg_extern(stable)]
#[allow(clippy::type_complexity)]
fn ec_spire_coordinator_dml_frontdoor_plan() -> TableIterator<
    'static,
    (
        name!(operation, &'static str),
        name!(frontdoor_integration, &'static str),
        name!(supported_shape, &'static str),
        name!(primitive, &'static str),
        name!(status, &'static str),
        name!(next_step, &'static str),
    ),
> {
    TableIterator::new(
        vec![
            (
                "insert",
                "before_row_queue_after_statement_flush_triggers",
                "bigint_pk_ecvector_embedding_bytea_source_identity",
                "ec_spire_prepare_coordinator_insert_tuple_payload_batch",
                "ready",
                "batch queue and flush triggers installed by ec_spire_enable_coordinator_insert",
            ),
            (
                "update_non_embedding",
                "planner_customscan_hook",
                "single_table_bigint_pk_equality_no_returning_non_embedding_columns",
                "ec_spire_forward_coordinator_update_tuple_payload",
                "frontdoor_pending",
                "wire relation metadata and CustomScan executor replacement",
            ),
            (
                "delete",
                "planner_customscan_hook",
                "single_table_bigint_pk_equality_no_returning",
                "ec_spire_prepare_coordinator_delete_tuple_payload",
                "frontdoor_pending",
                "wire relation metadata and CustomScan executor replacement",
            ),
            (
                "pk_select",
                "planner_customscan_hook",
                "single_table_bigint_pk_equality_projection",
                "ec_spire_forward_coordinator_select_tuple_payload",
                "frontdoor_pending",
                "wire relation metadata and CustomScan executor replacement",
            ),
            (
                "update_embedding",
                "shared_update_primitive_guard",
                "any update touching the ec_spire indexed embedding column",
                "ec_spire_forward_coordinator_update_tuple_payload",
                "ready",
                "reject with ADR-069 error and hint",
            ),
        ]
        .into_iter(),
    )
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_hierarchy_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(root_pid, i64),
        name!(root_level, i32),
        name!(max_observed_level, i32),
        name!(hierarchy_depth, i32),
        name!(routing_object_count, i64),
        name!(root_routing_object_count, i64),
        name!(internal_routing_object_count, i64),
        name!(leaf_object_count, i64),
        name!(delta_object_count, i64),
        name!(centroid_dimensions, i32),
        name!(root_child_count, i64),
        name!(distinct_leaf_parent_count, i64),
        name!(recursive_routing_supported, bool),
        name!(per_level_nprobe_supported, bool),
        name!(status, String),
        name!(recommendation, String),
    ),
> {
    if unsafe { !relation_oid_exists(index_oid) } {
        return TableIterator::new(Vec::new().into_iter());
    }
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_hierarchy_snapshot") };
    let snapshot = unsafe { am::spire_index_hierarchy_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(snapshot.active_epoch).expect("active epoch should fit in i64"),
        i64::try_from(snapshot.root_pid).expect("root pid should fit in i64"),
        i32::from(snapshot.root_level),
        i32::from(snapshot.max_observed_level),
        i32::from(snapshot.hierarchy_depth),
        i64::try_from(snapshot.routing_object_count)
            .expect("routing object count should fit in i64"),
        i64::try_from(snapshot.root_routing_object_count)
            .expect("root routing object count should fit in i64"),
        i64::try_from(snapshot.internal_routing_object_count)
            .expect("internal routing object count should fit in i64"),
        i64::try_from(snapshot.leaf_object_count).expect("leaf object count should fit in i64"),
        i64::try_from(snapshot.delta_object_count).expect("delta object count should fit in i64"),
        i32::from(snapshot.centroid_dimensions),
        i64::try_from(snapshot.root_child_count).expect("root child count should fit in i64"),
        i64::try_from(snapshot.distinct_leaf_parent_count)
            .expect("distinct leaf parent count should fit in i64"),
        snapshot.recursive_routing_supported,
        snapshot.per_level_nprobe_supported,
        snapshot.status.to_owned(),
        snapshot.recommendation.to_owned(),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_top_graph_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(top_graph_enabled, bool),
        name!(top_graph_count, i64),
        name!(top_graph_pid, i64),
        name!(root_pid, i64),
        name!(frontier_kind, &'static str),
        name!(frontier_parent_level, i32),
        name!(frontier_child_level, i32),
        name!(frontier_node_count, i64),
        name!(root_child_count, i64),
        name!(active_leaf_count, i64),
        name!(object_version, i64),
        name!(published_epoch_backref, i64),
        name!(level, i32),
        name!(node_count, i64),
        name!(dimensions, i32),
        name!(graph_degree, i64),
        name!(build_list_size, i64),
        name!(alpha, f32),
        name!(entry_node, i64),
        name!(edge_count, i64),
        name!(max_node_degree, i64),
        name!(effective_route_count, i64),
        name!(effective_search_list_size, i64),
        name!(configured_search_list_size, Option<i64>),
        name!(object_bytes, i64),
        name!(object_tuple_count, i64),
        name!(object_meta_tuple_count, i64),
        name!(object_segment_count, i64),
        name!(object_segment_tuple_count, i64),
        name!(status, String),
        name!(recommendation, String),
    ),
> {
    if unsafe { !relation_oid_exists(index_oid) } {
        return TableIterator::new(Vec::new().into_iter());
    }
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_top_graph_snapshot") };
    let snapshot = unsafe { am::spire_index_top_graph_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(snapshot.active_epoch).expect("active epoch should fit in i64"),
        snapshot.top_graph_enabled,
        i64::try_from(snapshot.top_graph_count).expect("top graph count should fit in i64"),
        i64::try_from(snapshot.top_graph_pid).expect("top graph pid should fit in i64"),
        i64::try_from(snapshot.root_pid).expect("root pid should fit in i64"),
        snapshot.frontier_kind,
        i32::from(snapshot.frontier_parent_level),
        i32::from(snapshot.frontier_child_level),
        i64::try_from(snapshot.frontier_node_count).expect("frontier node count should fit in i64"),
        i64::try_from(snapshot.root_child_count).expect("root child count should fit in i64"),
        i64::try_from(snapshot.active_leaf_count).expect("active leaf count should fit in i64"),
        i64::try_from(snapshot.object_version).expect("object version should fit in i64"),
        i64::try_from(snapshot.published_epoch_backref)
            .expect("published epoch backref should fit in i64"),
        i32::from(snapshot.level),
        i64::try_from(snapshot.node_count).expect("node count should fit in i64"),
        i32::from(snapshot.dimensions),
        i64::from(snapshot.graph_degree),
        i64::from(snapshot.build_list_size),
        snapshot.alpha,
        i64::try_from(snapshot.entry_node).expect("entry node should fit in i64"),
        i64::try_from(snapshot.edge_count).expect("edge count should fit in i64"),
        i64::try_from(snapshot.max_node_degree).expect("max node degree should fit in i64"),
        i64::from(snapshot.effective_route_count),
        i64::from(snapshot.effective_search_list_size),
        snapshot.configured_search_list_size.map(i64::from),
        i64::try_from(snapshot.object_bytes).expect("object bytes should fit in i64"),
        i64::try_from(snapshot.object_tuple_count).expect("object tuple count should fit in i64"),
        i64::try_from(snapshot.object_meta_tuple_count)
            .expect("object meta tuple count should fit in i64"),
        i64::try_from(snapshot.object_segment_count)
            .expect("object segment count should fit in i64"),
        i64::try_from(snapshot.object_segment_tuple_count)
            .expect("object segment tuple count should fit in i64"),
        snapshot.status.to_owned(),
        snapshot.recommendation.to_owned(),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_fanout_plan(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    selected_pids: Vec<i64>,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(target_kind, &'static str),
        name!(node_id, i64),
        name!(pid, i64),
        name!(placement_state, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!("ec_spire_remote_search_fanout_plan requested_epoch must be greater than 0");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!("ec_spire_remote_search_fanout_plan selected PID {pid} is negative")
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_fanout_plan") };
    let rows = unsafe {
        am::spire_remote_search_fanout_plan_rows(
            index_relation,
            requested_epoch,
            selected_pids,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            row.target_kind,
            i64::from(row.node_id),
            i64::try_from(row.pid).expect("pid should fit in i64"),
            row.placement_state,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_target_plan(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    selected_pids: Vec<i64>,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(target_kind, &'static str),
        name!(node_id, i64),
        name!(selected_pids, Vec<i64>),
        name!(pid_count, i64),
        name!(placement_state, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!("ec_spire_remote_search_target_plan requested_epoch must be greater than 0");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!("ec_spire_remote_search_target_plan selected PID {pid} is negative")
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_target_plan") };
    let rows = unsafe {
        am::spire_remote_search_target_plan_rows(
            index_relation,
            requested_epoch,
            selected_pids,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            row.target_kind,
            i64::from(row.node_id),
            row.selected_pids
                .into_iter()
                .map(|pid| i64::try_from(pid).expect("pid should fit in i64"))
                .collect::<Vec<_>>(),
            i64::try_from(row.pid_count).expect("pid count should fit in i64"),
            row.placement_state,
            row.status,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_target_readiness(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    selected_pids: Vec<i64>,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(target_kind, &'static str),
        name!(node_id, i64),
        name!(selected_pids, Vec<i64>),
        name!(pid_count, i64),
        name!(placement_state, &'static str),
        name!(node_kind, &'static str),
        name!(descriptor_state, &'static str),
        name!(node_status, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_target_readiness requested_epoch must be greater than 0"
        );
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_target_readiness selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_target_readiness") };
    let rows = unsafe {
        am::spire_remote_search_target_readiness_rows(
            index_relation,
            requested_epoch,
            selected_pids,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            row.target_kind,
            i64::from(row.node_id),
            row.selected_pids
                .into_iter()
                .map(|pid| i64::try_from(pid).expect("pid should fit in i64"))
                .collect::<Vec<_>>(),
            i64::try_from(row.pid_count).expect("pid count should fit in i64"),
            row.placement_state,
            row.node_kind,
            row.descriptor_state,
            row.node_status,
            row.status,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_request_plan(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(target_kind, &'static str),
        name!(node_id, i64),
        name!(selected_pids, Vec<i64>),
        name!(pid_count, i64),
        name!(query_dimension, i64),
        name!(top_k, i64),
        name!(consistency_mode, &'static str),
        name!(endpoint_function, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!("ec_spire_remote_search_request_plan requested_epoch must be greater than 0");
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_request_plan top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!("ec_spire_remote_search_request_plan selected PID {pid} is negative")
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_request_plan") };
    let rows = unsafe {
        am::spire_remote_search_request_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            row.target_kind,
            i64::from(row.node_id),
            row.selected_pids
                .into_iter()
                .map(|pid| i64::try_from(pid).expect("pid should fit in i64"))
                .collect::<Vec<_>>(),
            i64::try_from(row.pid_count).expect("pid count should fit in i64"),
            i64::try_from(row.query_dimension).expect("query dimension should fit in i64"),
            i64::try_from(row.top_k).expect("top_k should fit in i64"),
            row.consistency_mode,
            row.endpoint_function,
            row.status,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_request_readiness(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(target_kind, &'static str),
        name!(node_id, i64),
        name!(selected_pids, Vec<i64>),
        name!(pid_count, i64),
        name!(query_dimension, i64),
        name!(top_k, i64),
        name!(consistency_mode, &'static str),
        name!(endpoint_function, &'static str),
        name!(node_kind, &'static str),
        name!(descriptor_state, &'static str),
        name!(node_status, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_request_readiness requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_request_readiness top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_request_readiness selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_request_readiness") };
    let rows = unsafe {
        am::spire_remote_search_request_readiness_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            row.target_kind,
            i64::from(row.node_id),
            row.selected_pids
                .into_iter()
                .map(|pid| i64::try_from(pid).expect("pid should fit in i64"))
                .collect::<Vec<_>>(),
            i64::try_from(row.pid_count).expect("pid count should fit in i64"),
            i64::try_from(row.query_dimension).expect("query dimension should fit in i64"),
            i64::try_from(row.top_k).expect("top_k should fit in i64"),
            row.consistency_mode,
            row.endpoint_function,
            row.node_kind,
            row.descriptor_state,
            row.node_status,
            row.status,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_request_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(request_count, i64),
        name!(local_request_count, i64),
        name!(remote_request_count, i64),
        name!(skipped_request_count, i64),
        name!(executable_pid_count, i64),
        name!(local_pid_count, i64),
        name!(remote_pid_count, i64),
        name!(skipped_pid_count, i64),
        name!(query_dimension, i64),
        name!(top_k, i64),
        name!(consistency_mode, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_request_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_request_summary top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_request_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_request_summary") };
    let row = unsafe {
        am::spire_remote_search_request_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.request_count).expect("request count should fit in i64"),
        i64::try_from(row.local_request_count).expect("local request count should fit in i64"),
        i64::try_from(row.remote_request_count).expect("remote request count should fit in i64"),
        i64::try_from(row.skipped_request_count).expect("skipped request count should fit in i64"),
        i64::try_from(row.executable_pid_count).expect("executable pid count should fit in i64"),
        i64::try_from(row.local_pid_count).expect("local pid count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.skipped_pid_count).expect("skipped pid count should fit in i64"),
        i64::try_from(row.query_dimension).expect("query dimension should fit in i64"),
        i64::try_from(row.top_k).expect("top_k should fit in i64"),
        row.consistency_mode,
        row.status,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_readiness_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(request_count, i64),
        name!(ready_request_count, i64),
        name!(blocked_request_count, i64),
        name!(local_request_count, i64),
        name!(remote_request_count, i64),
        name!(skipped_request_count, i64),
        name!(executable_pid_count, i64),
        name!(ready_pid_count, i64),
        name!(blocked_pid_count, i64),
        name!(skipped_pid_count, i64),
        name!(missing_descriptor_request_count, i64),
        name!(missing_descriptor_pid_count, i64),
        name!(transport_request_count, i64),
        name!(transport_pid_count, i64),
        name!(query_dimension, i64),
        name!(top_k, i64),
        name!(consistency_mode, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_readiness_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_readiness_summary top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_readiness_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_readiness_summary") };
    let row = unsafe {
        am::spire_remote_search_readiness_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.request_count).expect("request count should fit in i64"),
        i64::try_from(row.ready_request_count).expect("ready request count should fit in i64"),
        i64::try_from(row.blocked_request_count).expect("blocked request count should fit in i64"),
        i64::try_from(row.local_request_count).expect("local request count should fit in i64"),
        i64::try_from(row.remote_request_count).expect("remote request count should fit in i64"),
        i64::try_from(row.skipped_request_count).expect("skipped request count should fit in i64"),
        i64::try_from(row.executable_pid_count).expect("executable pid count should fit in i64"),
        i64::try_from(row.ready_pid_count).expect("ready pid count should fit in i64"),
        i64::try_from(row.blocked_pid_count).expect("blocked pid count should fit in i64"),
        i64::try_from(row.skipped_pid_count).expect("skipped pid count should fit in i64"),
        i64::try_from(row.missing_descriptor_request_count)
            .expect("missing descriptor request count should fit in i64"),
        i64::try_from(row.missing_descriptor_pid_count)
            .expect("missing descriptor pid count should fit in i64"),
        i64::try_from(row.transport_request_count)
            .expect("transport request count should fit in i64"),
        i64::try_from(row.transport_pid_count).expect("transport pid count should fit in i64"),
        i64::try_from(row.query_dimension).expect("query dimension should fit in i64"),
        i64::try_from(row.top_k).expect("top_k should fit in i64"),
        row.consistency_mode,
        row.status,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_execution_plan(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(target_kind, &'static str),
        name!(node_id, i64),
        name!(selected_pids, Vec<i64>),
        name!(pid_count, i64),
        name!(query_dimension, i64),
        name!(top_k, i64),
        name!(consistency_mode, &'static str),
        name!(execution_transport, &'static str),
        name!(endpoint_function, &'static str),
        name!(remote_index_source, &'static str),
        name!(conninfo_source, &'static str),
        name!(candidate_format, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_execution_plan requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_execution_plan top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!("ec_spire_remote_search_execution_plan selected PID {pid} is negative")
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_execution_plan") };
    let rows = unsafe {
        am::spire_remote_search_execution_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            row.target_kind,
            i64::from(row.node_id),
            row.selected_pids
                .into_iter()
                .map(|pid| i64::try_from(pid).expect("pid should fit in i64"))
                .collect::<Vec<_>>(),
            i64::try_from(row.pid_count).expect("pid count should fit in i64"),
            i64::try_from(row.query_dimension).expect("query dimension should fit in i64"),
            i64::try_from(row.top_k).expect("top_k should fit in i64"),
            row.consistency_mode,
            row.execution_transport,
            row.endpoint_function,
            row.remote_index_source,
            row.conninfo_source,
            row.candidate_format,
            row.status,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_execution_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(plan_count, i64),
        name!(local_plan_count, i64),
        name!(remote_plan_count, i64),
        name!(skipped_plan_count, i64),
        name!(ready_plan_count, i64),
        name!(blocked_plan_count, i64),
        name!(degraded_skipped_plan_count, i64),
        name!(local_pid_count, i64),
        name!(remote_pid_count, i64),
        name!(skipped_pid_count, i64),
        name!(blocked_pid_count, i64),
        name!(query_dimension, i64),
        name!(top_k, i64),
        name!(consistency_mode, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_execution_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_execution_summary top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_execution_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_execution_summary") };
    let row = unsafe {
        am::spire_remote_search_execution_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.plan_count).expect("plan count should fit in i64"),
        i64::try_from(row.local_plan_count).expect("local plan count should fit in i64"),
        i64::try_from(row.remote_plan_count).expect("remote plan count should fit in i64"),
        i64::try_from(row.skipped_plan_count).expect("skipped plan count should fit in i64"),
        i64::try_from(row.ready_plan_count).expect("ready plan count should fit in i64"),
        i64::try_from(row.blocked_plan_count).expect("blocked plan count should fit in i64"),
        i64::try_from(row.degraded_skipped_plan_count)
            .expect("degraded skipped plan count should fit in i64"),
        i64::try_from(row.local_pid_count).expect("local pid count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.skipped_pid_count).expect("skipped pid count should fit in i64"),
        i64::try_from(row.blocked_pid_count).expect("blocked pid count should fit in i64"),
        i64::try_from(row.query_dimension).expect("query dimension should fit in i64"),
        i64::try_from(row.top_k).expect("top_k should fit in i64"),
        row.consistency_mode,
        row.status,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_request_plan(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(node_id, i64),
        name!(selected_pids, Vec<i64>),
        name!(pid_count, i64),
        name!(query_dimension, i64),
        name!(top_k, i64),
        name!(consistency_mode, &'static str),
        name!(execution_transport, &'static str),
        name!(sql_template, &'static str),
        name!(parameter_count, i64),
        name!(result_column_count, i64),
        name!(remote_index_source, &'static str),
        name!(conninfo_source, &'static str),
        name!(candidate_format, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_request_plan requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_libpq_request_plan top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_request_plan selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_libpq_request_plan")
    };
    let rows = unsafe {
        am::spire_remote_search_libpq_request_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            i64::from(row.node_id),
            row.selected_pids
                .into_iter()
                .map(|pid| i64::try_from(pid).expect("pid should fit in i64"))
                .collect::<Vec<_>>(),
            i64::try_from(row.pid_count).expect("pid count should fit in i64"),
            i64::try_from(row.query_dimension).expect("query dimension should fit in i64"),
            i64::try_from(row.top_k).expect("top_k should fit in i64"),
            row.consistency_mode,
            row.execution_transport,
            row.sql_template,
            i64::try_from(row.parameter_count).expect("parameter count should fit in i64"),
            i64::try_from(row.result_column_count).expect("result column count should fit in i64"),
            row.remote_index_source,
            row.conninfo_source,
            row.candidate_format,
            row.status,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_request_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(request_count, i64),
        name!(ready_request_count, i64),
        name!(blocked_request_count, i64),
        name!(remote_pid_count, i64),
        name!(blocked_pid_count, i64),
        name!(parameter_count_per_request, i64),
        name!(result_column_count, i64),
        name!(query_dimension, i64),
        name!(top_k, i64),
        name!(consistency_mode, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_request_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_libpq_request_summary top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_request_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_libpq_request_summary")
    };
    let row = unsafe {
        am::spire_remote_search_libpq_request_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.request_count).expect("request count should fit in i64"),
        i64::try_from(row.ready_request_count).expect("ready request count should fit in i64"),
        i64::try_from(row.blocked_request_count).expect("blocked request count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.blocked_pid_count).expect("blocked pid count should fit in i64"),
        i64::try_from(row.parameter_count_per_request)
            .expect("parameter count per request should fit in i64"),
        i64::try_from(row.result_column_count).expect("result column count should fit in i64"),
        i64::try_from(row.query_dimension).expect("query dimension should fit in i64"),
        i64::try_from(row.top_k).expect("top_k should fit in i64"),
        row.consistency_mode,
        row.status,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_connection_plan(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(node_id, i64),
        name!(selected_pids, Vec<i64>),
        name!(pid_count, i64),
        name!(execution_transport, &'static str),
        name!(conninfo_secret_name, String),
        name!(remote_index_regclass, String),
        name!(remote_index_identity_bytes, i64),
        name!(conninfo_resolution, &'static str),
        name!(pipeline_mode, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_connection_plan requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_libpq_connection_plan top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_connection_plan selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_libpq_connection_plan")
    };
    let rows = unsafe {
        am::spire_remote_search_libpq_connection_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            i64::from(row.node_id),
            row.selected_pids
                .into_iter()
                .map(|pid| i64::try_from(pid).expect("pid should fit in i64"))
                .collect::<Vec<_>>(),
            i64::try_from(row.pid_count).expect("pid count should fit in i64"),
            row.execution_transport,
            row.conninfo_secret_name,
            row.remote_index_regclass,
            i64::try_from(row.remote_index_identity_bytes)
                .expect("remote index identity byte count should fit in i64"),
            row.conninfo_resolution,
            row.pipeline_mode,
            row.status,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_connection_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(connection_count, i64),
        name!(descriptor_resolved_connection_count, i64),
        name!(missing_descriptor_connection_count, i64),
        name!(pipeline_connection_count, i64),
        name!(remote_pid_count, i64),
        name!(blocked_pid_count, i64),
        name!(query_dimension, i64),
        name!(top_k, i64),
        name!(consistency_mode, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_connection_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_libpq_connection_summary top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_connection_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_libpq_connection_summary")
    };
    let row = unsafe {
        am::spire_remote_search_libpq_connection_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.connection_count).expect("connection count should fit in i64"),
        i64::try_from(row.descriptor_resolved_connection_count)
            .expect("descriptor resolved connection count should fit in i64"),
        i64::try_from(row.missing_descriptor_connection_count)
            .expect("missing descriptor connection count should fit in i64"),
        i64::try_from(row.pipeline_connection_count)
            .expect("pipeline connection count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.blocked_pid_count).expect("blocked pid count should fit in i64"),
        i64::try_from(row.query_dimension).expect("query dimension should fit in i64"),
        i64::try_from(row.top_k).expect("top_k should fit in i64"),
        row.consistency_mode,
        row.status,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_dispatch_plan(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(node_id, i64),
        name!(selected_pids, Vec<i64>),
        name!(pid_count, i64),
        name!(query_dimension, i64),
        name!(top_k, i64),
        name!(consistency_mode, &'static str),
        name!(sql_template, &'static str),
        name!(parameter_count, i64),
        name!(result_column_count, i64),
        name!(conninfo_secret_name, String),
        name!(remote_index_regclass, String),
        name!(pipeline_mode, &'static str),
        name!(dispatch_action, &'static str),
        name!(receive_validator, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_dispatch_plan requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_libpq_dispatch_plan top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_dispatch_plan selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_libpq_dispatch_plan")
    };
    let rows = unsafe {
        am::spire_remote_search_libpq_dispatch_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            i64::from(row.node_id),
            row.selected_pids
                .into_iter()
                .map(|pid| i64::try_from(pid).expect("pid should fit in i64"))
                .collect::<Vec<_>>(),
            i64::try_from(row.pid_count).expect("pid count should fit in i64"),
            i64::try_from(row.query_dimension).expect("query dimension should fit in i64"),
            i64::try_from(row.top_k).expect("top_k should fit in i64"),
            row.consistency_mode,
            row.sql_template,
            i64::try_from(row.parameter_count).expect("parameter count should fit in i64"),
            i64::try_from(row.result_column_count).expect("result column count should fit in i64"),
            row.conninfo_secret_name,
            row.remote_index_regclass,
            row.pipeline_mode,
            row.dispatch_action,
            row.receive_validator,
            row.status,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_bind_plan(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(node_id, i64),
        name!(parameter_ordinal, i64),
        name!(parameter_name, &'static str),
        name!(pg_type, &'static str),
        name!(value_source, &'static str),
        name!(value_status, String),
        name!(value_preview, String),
        name!(element_count, i64),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_bind_plan requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_libpq_bind_plan top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_bind_plan selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_libpq_bind_plan") };
    let rows = unsafe {
        am::spire_remote_search_libpq_dispatch_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let bind_rows = rows.into_iter().flat_map(|row| {
        let value_status = if row.dispatch_action == "open_pipeline_and_send_remote_search" {
            "ready".to_owned()
        } else {
            row.status.to_owned()
        };
        let requested_epoch =
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64");
        let node_id = i64::from(row.node_id);
        let query_dimension =
            i64::try_from(row.query_dimension).expect("query dimension should fit in i64");
        let pid_count = i64::try_from(row.pid_count).expect("pid count should fit in i64");
        let top_k = i64::try_from(row.top_k).expect("top_k should fit in i64");

        vec![
            (
                requested_epoch,
                node_id,
                1_i64,
                "remote_index_oid",
                "oid",
                "remote_node_descriptor.remote_index_regclass",
                value_status.clone(),
                row.remote_index_regclass,
                1_i64,
            ),
            (
                requested_epoch,
                node_id,
                2_i64,
                "requested_epoch",
                "bigint",
                "coordinator_request.requested_epoch",
                value_status.clone(),
                requested_epoch.to_string(),
                1_i64,
            ),
            (
                requested_epoch,
                node_id,
                3_i64,
                "query",
                "real[]",
                "coordinator_request.query",
                value_status.clone(),
                format!("query_dimension={query_dimension}"),
                query_dimension,
            ),
            (
                requested_epoch,
                node_id,
                4_i64,
                "selected_pids",
                "bigint[]",
                "target_plan.selected_pids",
                value_status.clone(),
                format!("pid_count={pid_count}"),
                pid_count,
            ),
            (
                requested_epoch,
                node_id,
                5_i64,
                "top_k",
                "integer",
                "coordinator_request.top_k",
                value_status.clone(),
                top_k.to_string(),
                1_i64,
            ),
            (
                requested_epoch,
                node_id,
                6_i64,
                "consistency_mode",
                "text",
                "coordinator_request.consistency_mode",
                value_status,
                row.consistency_mode.to_owned(),
                1_i64,
            ),
        ]
    });

    TableIterator::new(bind_rows)
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_bind_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(request_count, i64),
        name!(bind_count, i64),
        name!(ready_bind_count, i64),
        name!(blocked_bind_count, i64),
        name!(parameter_count_per_request, i64),
        name!(remote_pid_count, i64),
        name!(blocked_pid_count, i64),
        name!(status, String),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_bind_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_libpq_bind_summary top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_bind_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_libpq_bind_summary")
    };
    let rows = unsafe {
        am::spire_remote_search_libpq_dispatch_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let parameter_count_per_request = 6_u64;
    let request_count =
        u64::try_from(rows.len()).expect("remote search bind request count should fit in u64");
    let mut ready_bind_count = 0_u64;
    let mut blocked_bind_count = 0_u64;
    let mut remote_pid_count = 0_u64;
    let mut blocked_pid_count = 0_u64;
    let mut first_blocked_status = "ready";

    for row in &rows {
        remote_pid_count = remote_pid_count
            .checked_add(row.pid_count)
            .unwrap_or_else(|| pgrx::error!("remote search bind summary remote pid overflow"));
        if row.dispatch_action == "open_pipeline_and_send_remote_search" {
            ready_bind_count = ready_bind_count
                .checked_add(parameter_count_per_request)
                .unwrap_or_else(|| pgrx::error!("remote search bind summary ready bind overflow"));
        } else {
            blocked_bind_count = blocked_bind_count
                .checked_add(parameter_count_per_request)
                .unwrap_or_else(|| {
                    pgrx::error!("remote search bind summary blocked bind overflow")
                });
            blocked_pid_count = blocked_pid_count
                .checked_add(row.pid_count)
                .unwrap_or_else(|| pgrx::error!("remote search bind summary blocked pid overflow"));
            if first_blocked_status == "ready" {
                first_blocked_status = row.status;
            }
        }
    }

    let bind_count = request_count
        .checked_mul(parameter_count_per_request)
        .unwrap_or_else(|| pgrx::error!("remote search bind summary bind count overflow"));
    let status = if blocked_bind_count == 0 {
        "ready".to_owned()
    } else {
        first_blocked_status.to_owned()
    };

    TableIterator::once((
        i64::try_from(requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(request_count).expect("request count should fit in i64"),
        i64::try_from(bind_count).expect("bind count should fit in i64"),
        i64::try_from(ready_bind_count).expect("ready bind count should fit in i64"),
        i64::try_from(blocked_bind_count).expect("blocked bind count should fit in i64"),
        i64::try_from(parameter_count_per_request)
            .expect("parameter count per request should fit in i64"),
        i64::try_from(remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(blocked_pid_count).expect("blocked pid count should fit in i64"),
        status,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_secret_plan(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(node_id, i64),
        name!(selected_pids, Vec<i64>),
        name!(pid_count, i64),
        name!(conninfo_secret_name, String),
        name!(provider_lookup_key, String),
        name!(resolved_conninfo_bytes, i64),
        name!(raw_conninfo_exposed, bool),
        name!(secret_resolution_action, &'static str),
        name!(next_executor_step, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_secret_plan requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_libpq_secret_plan top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_secret_plan selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_libpq_secret_plan") };
    let rows = unsafe {
        am::spire_remote_search_libpq_secret_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            i64::from(row.node_id),
            row.selected_pids
                .into_iter()
                .map(|pid| i64::try_from(pid).expect("pid should fit in i64"))
                .collect::<Vec<_>>(),
            i64::try_from(row.pid_count).expect("pid count should fit in i64"),
            row.conninfo_secret_name,
            row.provider_lookup_key,
            i64::try_from(row.resolved_conninfo_bytes)
                .expect("resolved conninfo byte count should fit in i64"),
            row.raw_conninfo_exposed,
            row.secret_resolution_action,
            row.next_executor_step,
            row.status,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_secret_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(secret_count, i64),
        name!(resolved_secret_count, i64),
        name!(blocked_secret_count, i64),
        name!(remote_pid_count, i64),
        name!(blocked_pid_count, i64),
        name!(next_executor_step, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_secret_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_libpq_secret_summary top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_secret_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_libpq_secret_summary")
    };
    let row = unsafe {
        am::spire_remote_search_libpq_secret_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.secret_count).expect("secret count should fit in i64"),
        i64::try_from(row.resolved_secret_count).expect("resolved secret count should fit in i64"),
        i64::try_from(row.blocked_secret_count).expect("blocked secret count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.blocked_pid_count).expect("blocked pid count should fit in i64"),
        row.next_executor_step,
        row.status,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_connection_open_plan(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(node_id, i64),
        name!(selected_pids, Vec<i64>),
        name!(pid_count, i64),
        name!(conninfo_secret_name, String),
        name!(provider_lookup_key, String),
        name!(resolved_conninfo_bytes, i64),
        name!(connection_lifecycle_policy, &'static str),
        name!(pooling_policy, &'static str),
        name!(connection_action, &'static str),
        name!(next_executor_step, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_connection_open_plan requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_connection_open_plan top_k must be non-negative"
        );
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_connection_open_plan selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_libpq_connection_open_plan",
        )
    };
    let rows = unsafe {
        am::spire_remote_search_libpq_connection_open_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            i64::from(row.node_id),
            row.selected_pids
                .into_iter()
                .map(|pid| i64::try_from(pid).expect("pid should fit in i64"))
                .collect::<Vec<_>>(),
            i64::try_from(row.pid_count).expect("pid count should fit in i64"),
            row.conninfo_secret_name,
            row.provider_lookup_key,
            i64::try_from(row.resolved_conninfo_bytes)
                .expect("resolved conninfo byte count should fit in i64"),
            row.connection_lifecycle_policy,
            row.pooling_policy,
            row.connection_action,
            row.next_executor_step,
            row.status,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_connection_open_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(connection_count, i64),
        name!(ready_connection_count, i64),
        name!(blocked_connection_count, i64),
        name!(remote_pid_count, i64),
        name!(blocked_pid_count, i64),
        name!(next_executor_step, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_connection_open_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_connection_open_summary top_k must be non-negative"
        );
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_connection_open_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_libpq_connection_open_summary",
        )
    };
    let row = unsafe {
        am::spire_remote_search_libpq_connection_open_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.connection_count).expect("connection count should fit in i64"),
        i64::try_from(row.ready_connection_count)
            .expect("ready connection count should fit in i64"),
        i64::try_from(row.blocked_connection_count)
            .expect("blocked connection count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.blocked_pid_count).expect("blocked pid count should fit in i64"),
        row.next_executor_step,
        row.status,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_executor_connection_check(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(node_id, i64),
        name!(pid_count, i64),
        name!(conninfo_secret_name, String),
        name!(provider_lookup_key, String),
        name!(connection_attempted, bool),
        name!(connection_status, &'static str),
        name!(conninfo_lookup_kind, &'static str),
        name!(next_executor_step, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_executor_connection_check requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_executor_connection_check top_k must be non-negative"
        );
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_executor_connection_check selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_libpq_executor_connection_check",
        )
    };
    let rows = unsafe {
        am::spire_remote_search_libpq_connection_open_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        let (
            connection_attempted,
            connection_status,
            conninfo_lookup_kind,
            next_executor_step,
            status,
            recommendation,
        ) = if row.connection_action != "open_libpq_connection" {
            (
                false,
                "blocked_before_connection",
                "not_attempted",
                row.next_executor_step,
                row.status,
                row.recommendation,
            )
        } else {
            match std::env::var(&row.provider_lookup_key) {
                Ok(conninfo) if !conninfo.is_empty() => {
                    match am::spire_remote_search_libpq_connect_with_session_timeouts(
                        &conninfo,
                        row.node_id,
                        "remote search libpq executor connection check",
                    ) {
                        Ok(_) => (
                            true,
                            "libpq_connection_opened",
                            "secret_provider",
                            "enter_libpq_pipeline_mode",
                            "requires_libpq_executor",
                            "enter libpq pipeline mode and send remote search request",
                        ),
                        Err(_) => (
                            true,
                            "libpq_connection_open_failed",
                            "secret_provider",
                            "open_libpq_connection",
                            "libpq_connection_failed",
                            "verify conninfo secret target and remote node availability",
                        ),
                    }
                }
                Ok(_) => (
                    false,
                    "conninfo_secret_empty",
                    "secret_provider",
                    "conninfo_secret_resolution",
                    "requires_conninfo_secret_resolution",
                    "configure a nonempty conninfo value in the external secret provider",
                ),
                Err(_) => (
                    false,
                    "conninfo_secret_missing",
                    "secret_provider",
                    "conninfo_secret_resolution",
                    "requires_conninfo_secret_resolution",
                    "configure the external secret provider entry for conninfo_secret_name",
                ),
            }
        };

        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            i64::from(row.node_id),
            i64::try_from(row.pid_count).expect("pid count should fit in i64"),
            row.conninfo_secret_name,
            row.provider_lookup_key,
            connection_attempted,
            connection_status,
            conninfo_lookup_kind,
            next_executor_step,
            status,
            recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_executor_candidates(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(served_epoch, i64),
        name!(node_id, i64),
        name!(pid, i64),
        name!(object_version, i64),
        name!(row_index, i64),
        name!(assignment_flags, i16),
        name!(vec_id, Vec<u8>),
        name!(row_locator, Vec<u8>),
        name!(score, f32),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_executor_candidates requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_libpq_executor_candidates top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_executor_candidates selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_libpq_executor_candidates",
        )
    };
    let rows = unsafe {
        am::spire_remote_search_libpq_executor_candidate_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.served_epoch).expect("served epoch should fit in i64"),
            i64::from(row.node_id),
            i64::try_from(row.pid).expect("pid should fit in i64"),
            i64::try_from(row.object_version).expect("object version should fit in i64"),
            i64::from(row.row_index),
            i16::try_from(row.assignment_flags).expect("assignment flags should fit in i16"),
            row.vec_id,
            row.row_locator,
            row.score,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_executor_receive_attempts(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(node_id, i64),
        name!(selected_pids, Vec<i64>),
        name!(pid_count, i64),
        name!(candidate_count, i64),
        name!(status, String),
        name!(next_blocker, String),
        name!(failure_action, String),
        name!(failure_reason, String),
        name!(recommendation, String),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_executor_receive_attempts requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_executor_receive_attempts top_k must be non-negative"
        );
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_executor_receive_attempts selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_libpq_executor_receive_attempts",
        )
    };
    let rows = unsafe {
        am::spire_remote_search_libpq_executor_receive_attempt_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            i64::from(row.node_id),
            row.selected_pids
                .into_iter()
                .map(|pid| i64::try_from(pid).expect("pid should fit in i64"))
                .collect::<Vec<_>>(),
            i64::try_from(row.pid_count).expect("pid count should fit in i64"),
            i64::try_from(row.candidate_count).expect("candidate count should fit in i64"),
            row.status,
            row.next_blocker,
            row.failure_action,
            row.failure_reason,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_executor_heap_candidates(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(served_epoch, i64),
        name!(node_id, i64),
        name!(pid, i64),
        name!(object_version, i64),
        name!(row_index, i64),
        name!(assignment_flags, i16),
        name!(vec_id, Vec<u8>),
        name!(row_locator, Vec<u8>),
        name!(heap_block, i64),
        name!(heap_offset, i32),
        name!(score, f32),
        name!(heap_lookup_owner, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_executor_heap_candidates requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_executor_heap_candidates top_k must be non-negative"
        );
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_executor_heap_candidates selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_libpq_executor_heap_candidates",
        )
    };
    let rows = unsafe {
        am::spire_remote_search_libpq_executor_heap_candidate_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            i64::try_from(row.served_epoch).expect("served epoch should fit in i64"),
            i64::from(row.node_id),
            i64::try_from(row.pid).expect("pid should fit in i64"),
            i64::try_from(row.object_version).expect("object version should fit in i64"),
            i64::from(row.row_index),
            i16::try_from(row.assignment_flags).expect("assignment flags should fit in i16"),
            row.vec_id,
            row.row_locator,
            i64::from(row.heap_block),
            i32::from(row.heap_offset),
            row.score,
            row.heap_lookup_owner,
            row.status,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_executor_heap_candidate_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(returned_candidate_count, i64),
        name!(heap_lookup_owner, &'static str),
        name!(result_source, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_executor_heap_candidate_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_executor_heap_candidate_summary top_k must be non-negative"
        );
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_executor_heap_candidate_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_libpq_executor_heap_candidate_summary",
        )
    };
    let rows = unsafe {
        am::spire_remote_search_libpq_executor_heap_candidate_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let returned_candidate_count =
        i64::try_from(rows.len()).expect("returned candidate count should fit in i64");
    let (heap_lookup_owner, result_source, status, recommendation) = if top_k == 0 {
        ("none", "none", "empty_top_k", "none")
    } else if returned_candidate_count > 0 {
        (
            "origin_node_row_locator",
            "remote_heap_candidates",
            "ready",
            "none",
        )
    } else {
        (
            "origin_node_row_locator",
            "remote_heap_candidates",
            "no_candidate_batches",
            "inspect remote search candidate availability",
        )
    };

    TableIterator::once((
        i64::try_from(requested_epoch).expect("requested epoch should fit in i64"),
        returned_candidate_count,
        heap_lookup_owner,
        result_source,
        status,
        recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_identity_cache_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(dispatch_count, i64),
        name!(compact_candidate_count, i64),
        name!(heap_candidate_count, i64),
        name!(endpoint_identity_cache_entry_count, i64),
        name!(endpoint_identity_query_count, i64),
        name!(endpoint_identity_cache_hit_count, i64),
        name!(endpoint_identity_cache_miss_count, i64),
        name!(raw_conninfo_cached, bool),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_identity_cache_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_identity_cache_summary top_k must be non-negative"
        );
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_identity_cache_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_libpq_identity_cache_summary",
        )
    };
    let row = unsafe {
        am::spire_remote_search_libpq_identity_cache_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.dispatch_count).expect("dispatch count should fit in i64"),
        i64::try_from(row.compact_candidate_count)
            .expect("compact candidate count should fit in i64"),
        i64::try_from(row.heap_candidate_count).expect("heap candidate count should fit in i64"),
        i64::try_from(row.endpoint_identity_cache_entry_count)
            .expect("endpoint identity cache entry count should fit in i64"),
        i64::try_from(row.endpoint_identity_query_count)
            .expect("endpoint identity query count should fit in i64"),
        i64::try_from(row.endpoint_identity_cache_hit_count)
            .expect("endpoint identity cache hit count should fit in i64"),
        i64::try_from(row.endpoint_identity_cache_miss_count)
            .expect("endpoint identity cache miss count should fit in i64"),
        row.raw_conninfo_cached,
        row.status,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_executor_work_plan(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(node_id, i64),
        name!(selected_pids, Vec<i64>),
        name!(pid_count, i64),
        name!(bind_count, i64),
        name!(bind_status, String),
        name!(dispatch_action, &'static str),
        name!(next_executor_step, &'static str),
        name!(executor_status, &'static str),
        name!(work_action, &'static str),
        name!(status, String),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_executor_work_plan requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_libpq_executor_work_plan top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_executor_work_plan selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_libpq_executor_work_plan")
    };
    let dispatch_rows = unsafe {
        am::spire_remote_search_libpq_dispatch_plan_rows(
            index_relation,
            requested_epoch,
            query.clone(),
            selected_pids.clone(),
            top_k,
            &consistency_mode,
        )
    };
    let readiness_row = unsafe {
        am::spire_remote_search_libpq_executor_readiness_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(dispatch_rows.into_iter().map(move |row| {
        let pipeline_ready = row.dispatch_action == "open_pipeline_and_send_remote_search";
        let bind_status = if pipeline_ready {
            "ready".to_owned()
        } else {
            row.status.to_owned()
        };
        let work_action = if pipeline_ready {
            readiness_row.secret_resolution_action
        } else {
            "blocked_before_executor"
        };
        let status = if pipeline_ready {
            readiness_row.status.to_owned()
        } else {
            row.status.to_owned()
        };

        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            i64::from(row.node_id),
            row.selected_pids
                .into_iter()
                .map(|pid| i64::try_from(pid).expect("pid should fit in i64"))
                .collect::<Vec<_>>(),
            i64::try_from(row.pid_count).expect("pid count should fit in i64"),
            6_i64,
            bind_status,
            row.dispatch_action,
            readiness_row.next_executor_step,
            readiness_row.status,
            work_action,
            status,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_executor_work_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(work_count, i64),
        name!(ready_work_count, i64),
        name!(blocked_work_count, i64),
        name!(remote_pid_count, i64),
        name!(blocked_pid_count, i64),
        name!(next_executor_step, &'static str),
        name!(executor_status, &'static str),
        name!(status, String),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_executor_work_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_executor_work_summary top_k must be non-negative"
        );
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_executor_work_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_libpq_executor_work_summary",
        )
    };
    let rows = unsafe {
        am::spire_remote_search_libpq_dispatch_plan_rows(
            index_relation,
            requested_epoch,
            query.clone(),
            selected_pids.clone(),
            top_k,
            &consistency_mode,
        )
    };
    let readiness = unsafe {
        am::spire_remote_search_libpq_executor_readiness_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let mut ready_work_count = 0_u64;
    let mut blocked_work_count = 0_u64;
    let mut remote_pid_count = 0_u64;
    let mut blocked_pid_count = 0_u64;
    let mut first_blocked_status = "ready";

    for row in &rows {
        remote_pid_count = remote_pid_count
            .checked_add(row.pid_count)
            .unwrap_or_else(|| pgrx::error!("remote search executor work remote pid overflow"));
        if row.dispatch_action == "open_pipeline_and_send_remote_search" {
            ready_work_count = ready_work_count
                .checked_add(1)
                .unwrap_or_else(|| pgrx::error!("remote search executor ready work overflow"));
        } else {
            blocked_work_count = blocked_work_count
                .checked_add(1)
                .unwrap_or_else(|| pgrx::error!("remote search executor blocked work overflow"));
            blocked_pid_count = blocked_pid_count
                .checked_add(row.pid_count)
                .unwrap_or_else(|| pgrx::error!("remote search executor blocked pid overflow"));
            if first_blocked_status == "ready" {
                first_blocked_status = row.status;
            }
        }
    }

    let work_count =
        u64::try_from(rows.len()).expect("remote search executor work count should fit in u64");
    let status = if blocked_work_count == 0 {
        readiness.status.to_owned()
    } else {
        first_blocked_status.to_owned()
    };

    TableIterator::once((
        i64::try_from(requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(work_count).expect("work count should fit in i64"),
        i64::try_from(ready_work_count).expect("ready work count should fit in i64"),
        i64::try_from(blocked_work_count).expect("blocked work count should fit in i64"),
        i64::try_from(remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(blocked_pid_count).expect("blocked pid count should fit in i64"),
        readiness.next_executor_step,
        readiness.status,
        status,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_dispatch_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(dispatch_count, i64),
        name!(pipeline_dispatch_count, i64),
        name!(missing_descriptor_dispatch_count, i64),
        name!(remote_pid_count, i64),
        name!(blocked_pid_count, i64),
        name!(query_dimension, i64),
        name!(top_k, i64),
        name!(consistency_mode, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_dispatch_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_libpq_dispatch_summary top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_dispatch_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_libpq_dispatch_summary")
    };
    let row = unsafe {
        am::spire_remote_search_libpq_dispatch_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.dispatch_count).expect("dispatch count should fit in i64"),
        i64::try_from(row.pipeline_dispatch_count)
            .expect("pipeline dispatch count should fit in i64"),
        i64::try_from(row.missing_descriptor_dispatch_count)
            .expect("missing descriptor dispatch count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.blocked_pid_count).expect("blocked pid count should fit in i64"),
        i64::try_from(row.query_dimension).expect("query dimension should fit in i64"),
        i64::try_from(row.top_k).expect("top_k should fit in i64"),
        row.consistency_mode,
        row.status,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_executor_budget_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(dispatch_count, i64),
        name!(admitted_dispatch_count, i64),
        name!(budget_blocked_dispatch_count, i64),
        name!(remote_pid_count, i64),
        name!(admitted_pid_count, i64),
        name!(budget_blocked_pid_count, i64),
        name!(max_nodes, i64),
        name!(max_pids, i64),
        name!(max_pids_per_node, i64),
        name!(max_concurrent_dispatches, i64),
        name!(max_concurrent_dispatches_per_node, i64),
        name!(connect_timeout_ms, i64),
        name!(statement_timeout_ms, i64),
        name!(next_executor_step, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_executor_budget_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_executor_budget_summary top_k must be non-negative"
        );
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_executor_budget_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_libpq_executor_budget_summary",
        )
    };
    let row = unsafe {
        am::spire_remote_search_libpq_executor_budget_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.dispatch_count).expect("dispatch count should fit in i64"),
        i64::try_from(row.admitted_dispatch_count)
            .expect("admitted dispatch count should fit in i64"),
        i64::try_from(row.budget_blocked_dispatch_count)
            .expect("budget-blocked dispatch count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.admitted_pid_count).expect("admitted pid count should fit in i64"),
        i64::try_from(row.budget_blocked_pid_count)
            .expect("budget-blocked pid count should fit in i64"),
        i64::try_from(row.max_nodes).expect("max nodes should fit in i64"),
        i64::try_from(row.max_pids).expect("max pids should fit in i64"),
        i64::try_from(row.max_pids_per_node).expect("max pids per node should fit in i64"),
        i64::try_from(row.max_concurrent_dispatches)
            .expect("max concurrent dispatches should fit in i64"),
        i64::try_from(row.max_concurrent_dispatches_per_node)
            .expect("max concurrent dispatches per node should fit in i64"),
        i64::try_from(row.connect_timeout_ms).expect("connect timeout should fit in i64"),
        i64::try_from(row.statement_timeout_ms).expect("statement timeout should fit in i64"),
        row.next_executor_step,
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_production_policy_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(active_epoch, i64),
        name!(consistency_mode_source, &'static str),
        name!(requested_consistency_mode, &'static str),
        name!(active_consistency_mode, &'static str),
        name!(status, &'static str),
        name!(failure_category, &'static str),
        name!(failure_action, &'static str),
        name!(next_executor_step, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_production_policy_summary requested_epoch must be greater than 0"
        );
    }
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_production_policy_summary",
        )
    };
    let row = unsafe {
        am::spire_remote_search_production_consistency_policy_summary_row(
            index_relation,
            requested_epoch,
            "function_argument",
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
        row.consistency_mode_source,
        row.requested_consistency_mode,
        row.active_consistency_mode,
        row.status,
        row.failure_category,
        row.failure_action,
        row.next_executor_step,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_production_policy_session_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(active_epoch, i64),
        name!(consistency_mode_source, &'static str),
        name!(requested_consistency_mode, &'static str),
        name!(active_consistency_mode, &'static str),
        name!(status, &'static str),
        name!(failure_category, &'static str),
        name!(failure_action, &'static str),
        name!(next_executor_step, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_production_policy_session_summary requested_epoch must be greater than 0"
        );
    }
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_production_policy_session_summary",
        )
    };
    let row = unsafe {
        am::spire_remote_search_production_session_consistency_policy_summary_row(
            index_relation,
            requested_epoch,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
        row.consistency_mode_source,
        row.requested_consistency_mode,
        row.active_consistency_mode,
        row.status,
        row.failure_category,
        row.failure_action,
        row.next_executor_step,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_production_fault_matrix() -> TableIterator<
    'static,
    (
        name!(fault_ordinal, i64),
        name!(failure_category, &'static str),
        name!(fault_scope, &'static str),
        name!(next_executor_step, &'static str),
        name!(strict_action, &'static str),
        name!(strict_status, &'static str),
        name!(degraded_action, &'static str),
        name!(degraded_status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let rows = am::spire_remote_search_production_fault_matrix_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.fault_ordinal).expect("fault ordinal should fit in i64"),
            row.failure_category,
            row.fault_scope,
            row.next_executor_step,
            row.strict_action,
            row.strict_status,
            row.degraded_action,
            row.degraded_status,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_stage_e_fault_matrix() -> TableIterator<
    'static,
    (
        name!(fault_ordinal, i64),
        name!(fault_case, &'static str),
        name!(fixture_scope, &'static str),
        name!(failure_category, &'static str),
        name!(next_executor_step, &'static str),
        name!(strict_action, &'static str),
        name!(strict_status, &'static str),
        name!(degraded_action, &'static str),
        name!(degraded_status, &'static str),
        name!(counter_delta, &'static str),
        name!(required_evidence, &'static str),
    ),
> {
    let rows = am::spire_remote_search_stage_e_fault_matrix_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.fault_ordinal).expect("fault ordinal should fit in i64"),
            row.fault_case,
            row.fixture_scope,
            row.failure_category,
            row.next_executor_step,
            row.strict_action,
            row.strict_status,
            row.degraded_action,
            row.degraded_status,
            row.counter_delta,
            row.required_evidence,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_stage_e_lifecycle_matrix() -> TableIterator<
    'static,
    (
        name!(lifecycle_ordinal, i64),
        name!(lifecycle_case, &'static str),
        name!(ddl_event, &'static str),
        name!(fanout_timing, &'static str),
        name!(affected_surface, &'static str),
        name!(strict_action, &'static str),
        name!(strict_status, &'static str),
        name!(degraded_action, &'static str),
        name!(degraded_status, &'static str),
        name!(required_detection, &'static str),
        name!(next_executor_step, &'static str),
        name!(required_evidence, &'static str),
    ),
> {
    let rows = am::spire_remote_search_stage_e_lifecycle_matrix_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.lifecycle_ordinal).expect("lifecycle ordinal should fit in i64"),
            row.lifecycle_case,
            row.ddl_event,
            row.fanout_timing,
            row.affected_surface,
            row.strict_action,
            row.strict_status,
            row.degraded_action,
            row.degraded_status,
            row.required_detection,
            row.next_executor_step,
            row.required_evidence,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_production_executor_state_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(state_model, &'static str),
        name!(transport_mode, &'static str),
        name!(dispatch_count, i64),
        name!(planned_dispatch_count, i64),
        name!(blocked_before_dispatch_count, i64),
        name!(remote_pid_count, i64),
        name!(planned_pid_count, i64),
        name!(blocked_pid_count, i64),
        name!(conninfo_secret_lookup_count, i64),
        name!(socket_open_count, i64),
        name!(endpoint_identity_query_count, i64),
        name!(transport_pending_dispatch_count, i64),
        name!(transport_sent_dispatch_count, i64),
        name!(transport_ready_dispatch_count, i64),
        name!(transport_failed_dispatch_count, i64),
        name!(transport_row_count, i64),
        name!(first_transport_failure_category, &'static str),
        name!(candidate_receive_pending_dispatch_count, i64),
        name!(candidate_receive_sent_dispatch_count, i64),
        name!(candidate_receive_ready_dispatch_count, i64),
        name!(candidate_receive_failed_dispatch_count, i64),
        name!(candidate_row_count, i64),
        name!(first_candidate_receive_failure_category, &'static str),
        name!(degraded_skipped_dispatch_count, i64),
        name!(first_degraded_skip_category, &'static str),
        name!(cancelled_dispatch_count, i64),
        name!(first_cancellation_category, &'static str),
        name!(next_executor_step, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_production_executor_state_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_production_executor_state_summary top_k must be non-negative"
        );
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_production_executor_state_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_production_executor_state_summary",
        )
    };
    let row = unsafe {
        am::spire_remote_search_production_executor_state_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        row.state_model,
        row.transport_mode,
        i64::try_from(row.dispatch_count).expect("dispatch count should fit in i64"),
        i64::try_from(row.planned_dispatch_count)
            .expect("planned dispatch count should fit in i64"),
        i64::try_from(row.blocked_before_dispatch_count)
            .expect("blocked-before-dispatch count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.planned_pid_count).expect("planned pid count should fit in i64"),
        i64::try_from(row.blocked_pid_count).expect("blocked pid count should fit in i64"),
        i64::try_from(row.conninfo_secret_lookup_count)
            .expect("secret lookup count should fit in i64"),
        i64::try_from(row.socket_open_count).expect("socket open count should fit in i64"),
        i64::try_from(row.endpoint_identity_query_count)
            .expect("endpoint identity query count should fit in i64"),
        i64::try_from(row.transport_pending_dispatch_count)
            .expect("transport pending dispatch count should fit in i64"),
        i64::try_from(row.transport_sent_dispatch_count)
            .expect("transport sent dispatch count should fit in i64"),
        i64::try_from(row.transport_ready_dispatch_count)
            .expect("transport ready dispatch count should fit in i64"),
        i64::try_from(row.transport_failed_dispatch_count)
            .expect("transport failed dispatch count should fit in i64"),
        i64::try_from(row.transport_row_count).expect("transport row count should fit in i64"),
        row.first_transport_failure_category,
        i64::try_from(row.candidate_receive_pending_dispatch_count)
            .expect("candidate receive pending dispatch count should fit in i64"),
        i64::try_from(row.candidate_receive_sent_dispatch_count)
            .expect("candidate receive sent dispatch count should fit in i64"),
        i64::try_from(row.candidate_receive_ready_dispatch_count)
            .expect("candidate receive ready dispatch count should fit in i64"),
        i64::try_from(row.candidate_receive_failed_dispatch_count)
            .expect("candidate receive failed dispatch count should fit in i64"),
        i64::try_from(row.candidate_row_count).expect("candidate row count should fit in i64"),
        row.first_candidate_receive_failure_category,
        i64::try_from(row.degraded_skipped_dispatch_count)
            .expect("degraded skipped dispatch count should fit in i64"),
        row.first_degraded_skip_category,
        i64::try_from(row.cancelled_dispatch_count)
            .expect("cancelled dispatch count should fit in i64"),
        row.first_cancellation_category,
        row.next_executor_step,
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_degraded_skip_report(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(node_id, i64),
        name!(skipped_pid_count, i64),
        name!(first_skip_category, &'static str),
        name!(first_skip_hint, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_degraded_skip_report requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_degraded_skip_report top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_degraded_skip_report selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_degraded_skip_report")
    };
    let rows = unsafe {
        am::spire_remote_search_production_degraded_skip_report_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            i64::from(row.node_id),
            i64::try_from(row.skipped_pid_count).expect("skipped PID count should fit in i64"),
            row.first_skip_category,
            row.first_skip_hint,
            row.status,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_production_executor_session_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(consistency_mode_source, &'static str),
        name!(consistency_mode, &'static str),
        name!(dispatch_count, i64),
        name!(degraded_skipped_dispatch_count, i64),
        name!(first_degraded_skip_category, &'static str),
        name!(next_executor_step, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_production_executor_session_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_production_executor_session_summary top_k must be non-negative"
        );
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_production_executor_session_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_production_executor_session_summary",
        )
    };
    let row = unsafe {
        am::spire_remote_search_production_executor_session_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        row.consistency_mode_source,
        row.consistency_mode,
        i64::try_from(row.dispatch_count).expect("dispatch count should fit in i64"),
        i64::try_from(row.degraded_skipped_dispatch_count)
            .expect("degraded skipped dispatch count should fit in i64"),
        row.first_degraded_skip_category,
        row.next_executor_step,
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_production_scan_handoff_summary(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
    top_k: i32,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(consistency_mode_source, &'static str),
        name!(consistency_mode, &'static str),
        name!(effective_nprobe, i64),
        name!(selected_pid_count, i64),
        name!(local_pid_count, i64),
        name!(remote_pid_count, i64),
        name!(skipped_pid_count, i64),
        name!(dispatch_count, i64),
        name!(candidate_receive_ready_dispatch_count, i64),
        name!(candidate_receive_failed_dispatch_count, i64),
        name!(degraded_skipped_dispatch_count, i64),
        name!(first_degraded_skip_category, &'static str),
        name!(candidate_row_count, i64),
        name!(merged_candidate_count, i64),
        name!(duplicate_vec_id_count, i64),
        name!(final_heap_fetch_status, &'static str),
        name!(next_blocker, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_production_scan_handoff_summary top_k must be non-negative"
        );
    }
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_production_scan_handoff_summary",
        )
    };
    let row = unsafe {
        am::spire_remote_search_production_scan_handoff_summary_row(index_relation, query, top_k)
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        row.consistency_mode_source,
        row.consistency_mode,
        i64::try_from(row.effective_nprobe).expect("effective nprobe should fit in i64"),
        i64::try_from(row.selected_pid_count).expect("selected pid count should fit in i64"),
        i64::try_from(row.local_pid_count).expect("local pid count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.skipped_pid_count).expect("skipped pid count should fit in i64"),
        i64::try_from(row.dispatch_count).expect("dispatch count should fit in i64"),
        i64::try_from(row.candidate_receive_ready_dispatch_count)
            .expect("candidate receive ready dispatch count should fit in i64"),
        i64::try_from(row.candidate_receive_failed_dispatch_count)
            .expect("candidate receive failed dispatch count should fit in i64"),
        i64::try_from(row.degraded_skipped_dispatch_count)
            .expect("degraded skipped dispatch count should fit in i64"),
        row.first_degraded_skip_category,
        i64::try_from(row.candidate_row_count).expect("candidate row count should fit in i64"),
        i64::try_from(row.merged_candidate_count)
            .expect("merged candidate count should fit in i64"),
        i64::try_from(row.duplicate_vec_id_count)
            .expect("duplicate vec id count should fit in i64"),
        row.final_heap_fetch_status,
        row.next_blocker,
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_production_scan_heap_resolution_summary(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
    top_k: i32,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(consistency_mode_source, &'static str),
        name!(consistency_mode, &'static str),
        name!(effective_nprobe, i64),
        name!(selected_pid_count, i64),
        name!(local_pid_count, i64),
        name!(remote_pid_count, i64),
        name!(skipped_pid_count, i64),
        name!(dispatch_count, i64),
        name!(compact_candidate_count, i64),
        name!(remote_heap_ready_dispatch_count, i64),
        name!(remote_heap_failed_dispatch_count, i64),
        name!(remote_heap_candidate_count, i64),
        name!(local_heap_candidate_count, i64),
        name!(returned_candidate_count, i64),
        name!(result_source, &'static str),
        name!(final_heap_fetch_status, &'static str),
        name!(next_blocker, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_production_scan_heap_resolution_summary top_k must be non-negative"
        );
    }
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_production_scan_heap_resolution_summary",
        )
    };
    let row = unsafe {
        am::spire_remote_search_production_scan_heap_resolution_summary_row(
            index_relation,
            query,
            top_k,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        row.consistency_mode_source,
        row.consistency_mode,
        i64::try_from(row.effective_nprobe).expect("effective nprobe should fit in i64"),
        i64::try_from(row.selected_pid_count).expect("selected pid count should fit in i64"),
        i64::try_from(row.local_pid_count).expect("local pid count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.skipped_pid_count).expect("skipped pid count should fit in i64"),
        i64::try_from(row.dispatch_count).expect("dispatch count should fit in i64"),
        i64::try_from(row.compact_candidate_count)
            .expect("compact candidate count should fit in i64"),
        i64::try_from(row.remote_heap_ready_dispatch_count)
            .expect("remote heap ready dispatch count should fit in i64"),
        i64::try_from(row.remote_heap_failed_dispatch_count)
            .expect("remote heap failed dispatch count should fit in i64"),
        i64::try_from(row.remote_heap_candidate_count)
            .expect("remote heap candidate count should fit in i64"),
        i64::try_from(row.local_heap_candidate_count)
            .expect("local heap candidate count should fit in i64"),
        i64::try_from(row.returned_candidate_count)
            .expect("returned candidate count should fit in i64"),
        row.result_source,
        row.final_heap_fetch_status,
        row.next_blocker,
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_production_read_profile(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
    top_k: i32,
) -> TableIterator<'static, (name!(metric, String), name!(value, String))> {
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_production_read_profile top_k must be non-negative");
    }
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");
    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_production_read_profile")
    };
    let row = unsafe {
        am::spire_remote_search_production_read_profile_row(index_relation, query, top_k)
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    macro_rules! metric {
        ($rows:expr, $name:literal, $value:expr) => {
            $rows.push(($name.to_owned(), $value.to_string()));
        };
    }
    let mut rows = Vec::new();

    metric!(rows, "requested_epoch", row.requested_epoch);
    metric!(rows, "consistency_mode_source", row.consistency_mode_source);
    metric!(rows, "consistency_mode", row.consistency_mode);
    metric!(rows, "effective_nprobe", row.effective_nprobe);
    metric!(rows, "selected_pid_count", row.selected_pid_count);
    metric!(rows, "local_pid_count", row.local_pid_count);
    metric!(rows, "remote_pid_count", row.remote_pid_count);
    metric!(rows, "skipped_pid_count", row.skipped_pid_count);
    metric!(rows, "dispatch_count", row.dispatch_count);
    metric!(rows, "compact_candidate_count", row.compact_candidate_count);
    metric!(
        rows,
        "remote_heap_ready_dispatch_count",
        row.remote_heap_ready_dispatch_count
    );
    metric!(
        rows,
        "remote_heap_failed_dispatch_count",
        row.remote_heap_failed_dispatch_count
    );
    metric!(
        rows,
        "remote_heap_candidate_count",
        row.remote_heap_candidate_count
    );
    metric!(
        rows,
        "local_heap_candidate_count",
        row.local_heap_candidate_count
    );
    metric!(
        rows,
        "returned_candidate_count",
        row.returned_candidate_count
    );
    metric!(rows, "result_source", row.result_source);
    metric!(rows, "final_heap_fetch_status", row.final_heap_fetch_status);
    metric!(rows, "next_blocker", row.next_blocker);
    metric!(rows, "status", row.status);
    metric!(rows, "recommendation", row.recommendation);
    metric!(rows, "planning_elapsed_ms", row.planning_elapsed_ms);
    metric!(
        rows,
        "fingerprint_guard_elapsed_ms",
        row.fingerprint_guard_elapsed_ms
    );
    metric!(
        rows,
        "conninfo_secret_lookup_elapsed_ms",
        row.conninfo_secret_lookup_elapsed_ms
    );
    metric!(rows, "connect_elapsed_ms", row.connect_elapsed_ms);
    metric!(
        rows,
        "statement_timeout_setup_elapsed_ms",
        row.statement_timeout_setup_elapsed_ms
    );
    metric!(
        rows,
        "regclass_probe_elapsed_ms",
        row.regclass_probe_elapsed_ms
    );
    metric!(
        rows,
        "endpoint_identity_elapsed_ms",
        row.endpoint_identity_elapsed_ms
    );
    metric!(
        rows,
        "candidate_receive_elapsed_ms",
        row.candidate_receive_elapsed_ms
    );
    metric!(rows, "heap_receive_elapsed_ms", row.heap_receive_elapsed_ms);
    metric!(
        rows,
        "payload_decode_elapsed_ms",
        row.payload_decode_elapsed_ms
    );
    metric!(rows, "merge_elapsed_ms", row.merge_elapsed_ms);
    metric!(rows, "total_elapsed_ms", row.total_elapsed_ms);
    metric!(
        rows,
        "conninfo_secret_lookup_count",
        row.conninfo_secret_lookup_count
    );
    metric!(rows, "socket_open_count", row.socket_open_count);
    metric!(rows, "tls_disable_count", row.tls_disable_count);
    metric!(rows, "tls_require_count", row.tls_require_count);
    metric!(rows, "tls_verify_full_count", row.tls_verify_full_count);
    metric!(
        rows,
        "statement_timeout_setup_count",
        row.statement_timeout_setup_count
    );
    metric!(rows, "regclass_probe_count", row.regclass_probe_count);
    metric!(
        rows,
        "endpoint_identity_query_count",
        row.endpoint_identity_query_count
    );
    metric!(
        rows,
        "candidate_receive_query_count",
        row.candidate_receive_query_count
    );
    metric!(
        rows,
        "heap_receive_query_count",
        row.heap_receive_query_count
    );
    metric!(
        rows,
        "payload_decode_row_count",
        row.payload_decode_row_count
    );
    metric!(rows, "payload_decode_bytes", row.payload_decode_bytes);
    metric!(rows, "merge_input_count", row.merge_input_count);
    metric!(
        rows,
        "merge_duplicate_vec_id_count",
        row.merge_duplicate_vec_id_count
    );
    metric!(rows, "merge_output_count", row.merge_output_count);
    metric!(rows, "strict_fail_count", row.strict_fail_count);
    metric!(rows, "remote_timeout_count", row.remote_timeout_count);
    metric!(rows, "remote_cancel_count", row.remote_cancel_count);
    metric!(
        rows,
        "degraded_skipped_dispatch_count",
        row.degraded_skipped_dispatch_count
    );
    TableIterator::new(rows)
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_operator_diagnostics(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
    top_k: i32,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(consistency_mode, &'static str),
        name!(remote_node_count, i64),
        name!(ready_remote_node_count, i64),
        name!(blocked_remote_node_count, i64),
        name!(min_remote_last_served_epoch, i64),
        name!(max_remote_last_served_epoch, i64),
        name!(remote_readiness_status, &'static str),
        name!(effective_nprobe, i64),
        name!(selected_pid_count, i64),
        name!(local_pid_count, i64),
        name!(remote_pid_count, i64),
        name!(skipped_pid_count, i64),
        name!(remote_fanout_count, i64),
        name!(candidate_batch_count, i64),
        name!(candidate_row_count, i64),
        name!(remote_heap_ready_dispatch_count, i64),
        name!(remote_heap_failed_dispatch_count, i64),
        name!(remote_heap_candidate_count, i64),
        name!(local_heap_candidate_count, i64),
        name!(returned_candidate_count, i64),
        name!(result_source, &'static str),
        name!(final_heap_fetch_status, &'static str),
        name!(merge_status, &'static str),
        name!(am_delivery_status, &'static str),
        name!(am_deliverable_output_count, i64),
        name!(remote_origin_output_count, i64),
        name!(next_blocker, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_operator_diagnostics top_k must be non-negative");
    }
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_operator_diagnostics")
    };
    let row =
        unsafe { am::spire_remote_search_operator_diagnostics_row(index_relation, query, top_k) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
        row.consistency_mode,
        i64::try_from(row.remote_node_count).expect("remote node count should fit in i64"),
        i64::try_from(row.ready_remote_node_count)
            .expect("ready remote node count should fit in i64"),
        i64::try_from(row.blocked_remote_node_count)
            .expect("blocked remote node count should fit in i64"),
        i64::try_from(row.min_remote_last_served_epoch)
            .expect("minimum remote last served epoch should fit in i64"),
        i64::try_from(row.max_remote_last_served_epoch)
            .expect("maximum remote last served epoch should fit in i64"),
        row.remote_readiness_status,
        i64::try_from(row.effective_nprobe).expect("effective nprobe should fit in i64"),
        i64::try_from(row.selected_pid_count).expect("selected pid count should fit in i64"),
        i64::try_from(row.local_pid_count).expect("local pid count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.skipped_pid_count).expect("skipped pid count should fit in i64"),
        i64::try_from(row.remote_fanout_count).expect("remote fanout count should fit in i64"),
        i64::try_from(row.candidate_batch_count).expect("candidate batch count should fit in i64"),
        i64::try_from(row.candidate_row_count).expect("candidate row count should fit in i64"),
        i64::try_from(row.remote_heap_ready_dispatch_count)
            .expect("remote heap ready dispatch count should fit in i64"),
        i64::try_from(row.remote_heap_failed_dispatch_count)
            .expect("remote heap failed dispatch count should fit in i64"),
        i64::try_from(row.remote_heap_candidate_count)
            .expect("remote heap candidate count should fit in i64"),
        i64::try_from(row.local_heap_candidate_count)
            .expect("local heap candidate count should fit in i64"),
        i64::try_from(row.returned_candidate_count)
            .expect("returned candidate count should fit in i64"),
        row.result_source,
        row.final_heap_fetch_status,
        row.merge_status,
        row.am_delivery_status,
        i64::try_from(row.am_deliverable_output_count)
            .expect("AM deliverable output count should fit in i64"),
        i64::try_from(row.remote_origin_output_count)
            .expect("remote origin output count should fit in i64"),
        row.next_blocker,
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_executor_readiness(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(dispatch_count, i64),
        name!(pipeline_dispatch_count, i64),
        name!(blocked_dispatch_count, i64),
        name!(secret_resolution_action, &'static str),
        name!(connection_action, &'static str),
        name!(pipeline_action, &'static str),
        name!(send_action, &'static str),
        name!(receive_action, &'static str),
        name!(merge_action, &'static str),
        name!(next_executor_step, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_libpq_executor_readiness requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_libpq_executor_readiness top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_libpq_executor_readiness selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_libpq_executor_readiness")
    };
    let row = unsafe {
        am::spire_remote_search_libpq_executor_readiness_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.dispatch_count).expect("dispatch count should fit in i64"),
        i64::try_from(row.pipeline_dispatch_count)
            .expect("pipeline dispatch count should fit in i64"),
        i64::try_from(row.blocked_dispatch_count)
            .expect("blocked dispatch count should fit in i64"),
        row.secret_resolution_action,
        row.connection_action,
        row.pipeline_action,
        row.send_action,
        row.receive_action,
        row.merge_action,
        row.next_executor_step,
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_parameter_contract() -> TableIterator<
    'static,
    (
        name!(parameter_ordinal, i64),
        name!(parameter_name, &'static str),
        name!(pg_type, &'static str),
        name!(semantic_role, &'static str),
        name!(validator, &'static str),
    ),
> {
    let rows = am::spire_remote_search_libpq_parameter_contract_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.parameter_ordinal).expect("parameter ordinal should fit in i64"),
            row.parameter_name,
            row.pg_type,
            row.semantic_role,
            row.validator,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_executor_step_contract() -> TableIterator<
    'static,
    (
        name!(step_ordinal, i64),
        name!(step_name, &'static str),
        name!(executor_action, &'static str),
        name!(input_contract, &'static str),
        name!(output_contract, &'static str),
        name!(blocking_status, &'static str),
        name!(validator, &'static str),
    ),
> {
    let rows = am::spire_remote_search_libpq_executor_step_contract_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.step_ordinal).expect("step ordinal should fit in i64"),
            row.step_name,
            row.executor_action,
            row.input_contract,
            row.output_contract,
            row.blocking_status,
            row.validator,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_libpq_result_contract() -> TableIterator<
    'static,
    (
        name!(column_ordinal, i64),
        name!(column_name, &'static str),
        name!(pg_type, &'static str),
        name!(semantic_role, &'static str),
        name!(nullable, bool),
        name!(validator, &'static str),
    ),
> {
    let rows = am::spire_remote_search_libpq_result_contract_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.column_ordinal).expect("column ordinal should fit in i64"),
            row.column_name,
            row.pg_type,
            row.semantic_role,
            row.nullable,
            row.validator,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_endpoint_contract() -> TableIterator<
    'static,
    (
        name!(contract_ordinal, i64),
        name!(contract_item, &'static str),
        name!(contract_value, &'static str),
        name!(status, &'static str),
        name!(validator, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let rows = am::spire_remote_search_endpoint_contract_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.contract_ordinal).expect("contract ordinal should fit in i64"),
            row.contract_item,
            row.contract_value,
            row.status,
            row.validator,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_endpoint_identity(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(protocol_version, &'static str),
        name!(extension_version, &'static str),
        name!(opclass_identity, String),
        name!(storage_format, &'static str),
        name!(assignment_payload_format, &'static str),
        name!(quantizer_profile, &'static str),
        name!(scoring_profile, &'static str),
        name!(tuple_transport_capabilities, Vec<String>),
        name!(tuple_transport_default, &'static str),
        name!(tuple_transport_status, &'static str),
        name!(profile_fingerprint, String),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_endpoint_identity") };
    let row = unsafe { am::spire_remote_search_endpoint_identity_row(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        row.protocol_version,
        row.extension_version,
        row.opclass_identity,
        row.storage_format,
        row.assignment_payload_format,
        row.quantizer_profile,
        row.scoring_profile,
        row.tuple_transport_capabilities,
        row.tuple_transport_default,
        row.tuple_transport_status,
        row.profile_fingerprint,
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_coordinator_result_contract() -> TableIterator<
    'static,
    (
        name!(result_source, &'static str),
        name!(status_family, &'static str),
        name!(semantic_role, &'static str),
        name!(validator, &'static str),
    ),
> {
    TableIterator::new(
        vec![
            (
                "local_heap_candidates",
                "ready_or_degraded_ready",
                "coordinator_local_heap_result_rows",
                "must_have_positive_returned_candidate_count",
            ),
            (
                "remote_heap_candidates",
                "ready",
                "origin_node_heap_result_rows",
                "must_have_positive_returned_candidate_count_and_origin_node_heap_owner",
            ),
            (
                "blocked",
                "blocked",
                "pre_result_gate_blocked",
                "must_preserve_next_blocker",
            ),
            (
                "none",
                "empty_top_k",
                "empty_top_k_result",
                "must_have_zero_returned_candidate_count_and_no_blocker",
            ),
        ]
        .into_iter(),
    )
}

struct SpireRemotePipelineStepRow {
    step_ordinal: i64,
    step_name: &'static str,
    requested_epoch: i64,
    status: String,
    item_count: i64,
    ready_count: i64,
    blocked_count: i64,
    remote_pid_count: i64,
    next_blocker: String,
    recommendation: String,
}

fn remote_pipeline_manifest_apply_step(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
) -> SpireRemotePipelineStepRow {
    let (
        publication_entry_count,
        libpq_receive_count,
        ready_receive_count,
        next_blocker,
        status,
    ) = Spi::connect(|client| {
        client
            .select(
                "SELECT publication_entry_count, libpq_receive_count, \
                        ready_receive_count, next_blocker, status \
                   FROM ec_spire_remote_epoch_manifest_publication_result_summary($1::oid)",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| format!("ec_spire remote pipeline manifest apply read failed: {e}"))?
            .map(|row| {
                Ok::<_, String>((
                    row["publication_entry_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "remote pipeline manifest publication_entry_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "remote pipeline manifest publication_entry_count is null".to_owned()
                        })?,
                    row["libpq_receive_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("remote pipeline manifest libpq_receive_count decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "remote pipeline manifest libpq_receive_count is null".to_owned()
                        })?,
                    row["ready_receive_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!("remote pipeline manifest ready_receive_count decode failed: {e}")
                        })?
                        .ok_or_else(|| {
                            "remote pipeline manifest ready_receive_count is null".to_owned()
                        })?,
                    row["next_blocker"]
                        .value::<String>()
                        .map_err(|e| {
                            format!("remote pipeline manifest next_blocker decode failed: {e}")
                        })?
                        .ok_or_else(|| "remote pipeline manifest next_blocker is null".to_owned())?,
                    row["status"]
                        .value::<String>()
                        .map_err(|e| format!("remote pipeline manifest status decode failed: {e}"))?
                        .ok_or_else(|| "remote pipeline manifest status is null".to_owned())?,
                ))
            })
            .next()
            .transpose()?
            .ok_or_else(|| "ec_spire remote pipeline manifest apply returned no rows".to_owned())
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    let blocked_count = libpq_receive_count.saturating_sub(ready_receive_count);
    let recommendation = if status == "ready" || status == "not_required" {
        "none"
    } else {
        "run remote epoch manifest publication executor or resolve manifest blocker"
    };

    SpireRemotePipelineStepRow {
        step_ordinal: 5,
        step_name: "manifest_apply",
        requested_epoch,
        status,
        item_count: publication_entry_count,
        ready_count: ready_receive_count,
        blocked_count,
        remote_pid_count: publication_entry_count,
        next_blocker,
        recommendation: recommendation.to_owned(),
    }
}

#[allow(clippy::too_many_arguments)]
fn spire_remote_pipeline_step_rows(
    function_name: &'static str,
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
    probe_remote_executor: bool,
) -> Vec<SpireRemotePipelineStepRow> {
    if requested_epoch <= 0 {
        pgrx::error!("{function_name} requested_epoch must be greater than 0");
    }
    if top_k < 0 {
        pgrx::error!("{function_name} top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid)
                .unwrap_or_else(|_| pgrx::error!("{function_name} selected PID {pid} is negative"))
        })
        .collect::<Vec<_>>();
    let requested_epoch_u64 =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe { open_valid_ec_spire_index(index_oid, function_name) };
    let dispatch = unsafe {
        am::spire_remote_search_libpq_dispatch_summary_row(
            index_relation,
            requested_epoch_u64,
            query.clone(),
            selected_pids.clone(),
            top_k,
            &consistency_mode,
        )
    };
    let connection_open_rows = unsafe {
        am::spire_remote_search_libpq_connection_open_plan_rows(
            index_relation,
            requested_epoch_u64,
            query.clone(),
            selected_pids.clone(),
            top_k,
            &consistency_mode,
        )
    };

    let mut connection_ready_count = 0_i64;
    let mut connection_blocked_count = 0_i64;
    let mut connection_remote_pid_count = 0_i64;
    let mut connection_status = "ready".to_owned();
    let mut connection_next_blocker = "none".to_owned();
    let mut connection_recommendation = "none".to_owned();
    for row in &connection_open_rows {
        let pid_count = i64::try_from(row.pid_count).expect("pid count should fit in i64");
        connection_remote_pid_count = connection_remote_pid_count
            .checked_add(pid_count)
            .unwrap_or_else(|| pgrx::error!("remote pipeline connection PID count overflow"));
        let (ready, status, next_blocker, recommendation) = if row.connection_action
            != "open_libpq_connection"
        {
            (
                false,
                row.status.to_owned(),
                row.next_executor_step.to_owned(),
                row.recommendation.to_owned(),
            )
        } else if probe_remote_executor {
            match std::env::var(&row.provider_lookup_key) {
                Ok(conninfo) if !conninfo.is_empty() => {
                    match am::spire_remote_search_libpq_connect_with_session_timeouts(
                        &conninfo,
                        row.node_id,
                        "remote pipeline connection check",
                    ) {
                        Ok(_) => (
                            true,
                            "requires_libpq_executor".to_owned(),
                            "enter_libpq_pipeline_mode".to_owned(),
                            "enter libpq pipeline mode and send remote search request".to_owned(),
                        ),
                        Err(_) => (
                            false,
                            "libpq_connection_failed".to_owned(),
                            "open_libpq_connection".to_owned(),
                            "verify conninfo secret target and remote node availability".to_owned(),
                        ),
                    }
                }
                Ok(_) => (
                    false,
                    "requires_conninfo_secret_resolution".to_owned(),
                    "conninfo_secret_resolution".to_owned(),
                    "configure a nonempty conninfo value in the external secret provider"
                        .to_owned(),
                ),
                Err(_) => (
                    false,
                    "requires_conninfo_secret_resolution".to_owned(),
                    "conninfo_secret_resolution".to_owned(),
                    "configure the external secret provider entry for conninfo_secret_name"
                        .to_owned(),
                ),
            }
        } else {
            match std::env::var(&row.provider_lookup_key) {
                Ok(conninfo) if !conninfo.is_empty() => (
                    true,
                    "requires_libpq_executor".to_owned(),
                    "enter_libpq_pipeline_mode".to_owned(),
                    "run ec_spire_remote_pipeline_steps_live to open libpq connections and execute remote pipeline diagnostics".to_owned(),
                ),
                Ok(_) => (
                    false,
                    "requires_conninfo_secret_resolution".to_owned(),
                    "conninfo_secret_resolution".to_owned(),
                    "configure a nonempty conninfo value in the external secret provider"
                        .to_owned(),
                ),
                Err(_) => (
                    false,
                    "requires_conninfo_secret_resolution".to_owned(),
                    "conninfo_secret_resolution".to_owned(),
                    "configure the external secret provider entry for conninfo_secret_name"
                        .to_owned(),
                ),
            }
        };
        if ready {
            connection_ready_count = connection_ready_count
                .checked_add(1)
                .unwrap_or_else(|| pgrx::error!("remote pipeline connection ready overflow"));
        } else {
            connection_blocked_count = connection_blocked_count
                .checked_add(1)
                .unwrap_or_else(|| pgrx::error!("remote pipeline connection blocked overflow"));
            if connection_status == "ready" {
                connection_status = status;
                connection_next_blocker = next_blocker;
                connection_recommendation = recommendation;
            }
        }
    }
    if connection_blocked_count == 0 && connection_ready_count > 0 {
        connection_status = "requires_libpq_executor".to_owned();
        connection_next_blocker = "enter_libpq_pipeline_mode".to_owned();
        connection_recommendation = if probe_remote_executor {
            "enter libpq pipeline mode and send remote search request".to_owned()
        } else {
            "run ec_spire_remote_pipeline_steps_live to open libpq connections and execute remote pipeline diagnostics".to_owned()
        };
    }

    let identity_cache_summary = if probe_remote_executor
        && connection_blocked_count == 0
        && top_k > 0
        && connection_ready_count > 0
    {
        Some(unsafe {
            am::spire_remote_search_libpq_identity_cache_summary_row(
                index_relation,
                requested_epoch_u64,
                query.clone(),
                selected_pids.clone(),
                top_k,
                &consistency_mode,
            )
        })
    } else {
        None
    };

    let (candidate_count, candidate_status, candidate_recommendation) =
        if connection_blocked_count > 0 {
            (
                0_i64,
                connection_status.clone(),
                connection_recommendation.clone(),
            )
        } else if top_k == 0 {
            (0_i64, "empty_top_k".to_owned(), "none".to_owned())
        } else if connection_ready_count == 0 {
            (0_i64, "ready".to_owned(), "none".to_owned())
        } else if !probe_remote_executor {
            (
                0_i64,
                "requires_libpq_executor".to_owned(),
                "run ec_spire_remote_pipeline_steps_live to execute remote candidate diagnostics"
                    .to_owned(),
            )
        } else {
            let summary = identity_cache_summary
                .as_ref()
                .expect("identity cache summary should exist when libpq executor is required");
            let count = i64::try_from(summary.compact_candidate_count)
                .expect("candidate count should fit in i64");
            if count > 0 {
                (count, "ready".to_owned(), "none".to_owned())
            } else if summary.status != "ready" {
                (
                    0_i64,
                    summary.status.to_owned(),
                    "inspect remote libpq identity cache readiness".to_owned(),
                )
            } else {
                (
                    0_i64,
                    "no_candidate_batches".to_owned(),
                    "inspect remote search candidate availability".to_owned(),
                )
            }
        };

    let (heap_candidate_count, heap_status, heap_recommendation) = if connection_blocked_count > 0 {
        (
            0_i64,
            connection_status.clone(),
            connection_recommendation.clone(),
        )
    } else if top_k == 0 {
        (0_i64, "empty_top_k".to_owned(), "none".to_owned())
    } else if connection_ready_count == 0 {
        (0_i64, "ready".to_owned(), "none".to_owned())
    } else if !probe_remote_executor {
        (
            0_i64,
            "requires_libpq_executor".to_owned(),
            "run ec_spire_remote_pipeline_steps_live to execute remote heap-candidate diagnostics"
                .to_owned(),
        )
    } else {
        let summary = identity_cache_summary
            .as_ref()
            .expect("identity cache summary should exist when libpq executor is required");
        let count = i64::try_from(summary.heap_candidate_count)
            .expect("heap candidate count should fit in i64");
        if count > 0 {
            (count, "ready".to_owned(), "none".to_owned())
        } else if summary.status != "ready" {
            (
                0_i64,
                summary.status.to_owned(),
                "inspect remote libpq identity cache readiness".to_owned(),
            )
        } else {
            (
                0_i64,
                "no_candidate_batches".to_owned(),
                "inspect remote search candidate availability".to_owned(),
            )
        }
    };

    let coordinator_result = if probe_remote_executor {
        Some(unsafe {
            am::spire_remote_search_coordinator_result_summary_row(
                index_relation,
                requested_epoch_u64,
                query,
                selected_pids,
                top_k,
                &consistency_mode,
            )
        })
    } else {
        None
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let coordinator_step = if let Some(coordinator_result) = coordinator_result {
        SpireRemotePipelineStepRow {
            step_ordinal: 6,
            step_name: "coordinator_result",
            requested_epoch,
            status: coordinator_result.status.to_owned(),
            item_count: i64::try_from(coordinator_result.returned_candidate_count)
                .expect("coordinator result count should fit in i64"),
            ready_count: i64::try_from(coordinator_result.returned_candidate_count)
                .expect("coordinator result count should fit in i64"),
            blocked_count: if coordinator_result.next_blocker == "none" {
                0
            } else {
                1
            },
            remote_pid_count: i64::try_from(coordinator_result.remote_pid_count)
                .expect("coordinator remote pid count should fit in i64"),
            next_blocker: coordinator_result.next_blocker.to_owned(),
            recommendation: coordinator_result.recommendation.to_owned(),
        }
    } else if connection_blocked_count > 0 {
        SpireRemotePipelineStepRow {
            step_ordinal: 6,
            step_name: "coordinator_result",
            requested_epoch,
            status: connection_status.clone(),
            item_count: 0,
            ready_count: 0,
            blocked_count: 1,
            remote_pid_count: connection_remote_pid_count,
            next_blocker: connection_next_blocker.clone(),
            recommendation: connection_recommendation.clone(),
        }
    } else if top_k == 0 {
        SpireRemotePipelineStepRow {
            step_ordinal: 6,
            step_name: "coordinator_result",
            requested_epoch,
            status: "empty_top_k".to_owned(),
            item_count: 0,
            ready_count: 0,
            blocked_count: 0,
            remote_pid_count: connection_remote_pid_count,
            next_blocker: "none".to_owned(),
            recommendation: "none".to_owned(),
        }
    } else if connection_ready_count > 0 {
        SpireRemotePipelineStepRow {
            step_ordinal: 6,
            step_name: "coordinator_result",
            requested_epoch,
            status: "requires_libpq_executor".to_owned(),
            item_count: 0,
            ready_count: 0,
            blocked_count: 1,
            remote_pid_count: connection_remote_pid_count,
            next_blocker: "enter_libpq_pipeline_mode".to_owned(),
            recommendation:
                "run ec_spire_remote_pipeline_steps_live to execute remote coordinator diagnostics"
                    .to_owned(),
        }
    } else {
        SpireRemotePipelineStepRow {
            step_ordinal: 6,
            step_name: "coordinator_result",
            requested_epoch,
            status: "ready".to_owned(),
            item_count: 0,
            ready_count: 0,
            blocked_count: 0,
            remote_pid_count: 0,
            next_blocker: "none".to_owned(),
            recommendation: "none".to_owned(),
        }
    };

    let mut rows = Vec::with_capacity(6);
    rows.push(SpireRemotePipelineStepRow {
        step_ordinal: 1,
        step_name: "dispatch_plan",
        requested_epoch,
        status: dispatch.status.to_owned(),
        item_count: i64::try_from(dispatch.dispatch_count)
            .expect("dispatch count should fit in i64"),
        ready_count: i64::try_from(dispatch.pipeline_dispatch_count)
            .expect("pipeline dispatch count should fit in i64"),
        blocked_count: i64::try_from(dispatch.missing_descriptor_dispatch_count)
            .expect("missing descriptor dispatch count should fit in i64"),
        remote_pid_count: i64::try_from(dispatch.remote_pid_count)
            .expect("remote pid count should fit in i64"),
        next_blocker: if dispatch.status == "ready" {
            "none".to_owned()
        } else {
            dispatch.status.to_owned()
        },
        recommendation: if dispatch.status == "ready" {
            "none".to_owned()
        } else {
            "resolve remote search dispatch blockers".to_owned()
        },
    });
    rows.push(SpireRemotePipelineStepRow {
        step_ordinal: 2,
        step_name: "connection_check",
        requested_epoch,
        status: connection_status,
        item_count: i64::try_from(connection_open_rows.len())
            .expect("connection count should fit in i64"),
        ready_count: connection_ready_count,
        blocked_count: connection_blocked_count,
        remote_pid_count: connection_remote_pid_count,
        next_blocker: connection_next_blocker,
        recommendation: connection_recommendation,
    });
    rows.push(SpireRemotePipelineStepRow {
        step_ordinal: 3,
        step_name: "candidates",
        requested_epoch,
        status: candidate_status,
        item_count: candidate_count,
        ready_count: candidate_count,
        blocked_count: if candidate_count == 0 && candidate_recommendation != "none" {
            1
        } else {
            0
        },
        remote_pid_count: connection_remote_pid_count,
        next_blocker: if candidate_recommendation == "none" {
            "none".to_owned()
        } else {
            "remote_search_candidates".to_owned()
        },
        recommendation: candidate_recommendation,
    });
    rows.push(SpireRemotePipelineStepRow {
        step_ordinal: 4,
        step_name: "heap_candidates",
        requested_epoch,
        status: heap_status,
        item_count: heap_candidate_count,
        ready_count: heap_candidate_count,
        blocked_count: if heap_candidate_count == 0 && heap_recommendation != "none" {
            1
        } else {
            0
        },
        remote_pid_count: connection_remote_pid_count,
        next_blocker: if heap_recommendation == "none" {
            "none".to_owned()
        } else {
            "remote_heap_candidates".to_owned()
        },
        recommendation: heap_recommendation,
    });
    rows.push(remote_pipeline_manifest_apply_step(
        index_oid,
        requested_epoch,
    ));
    rows.push(coordinator_step);

    rows
}

#[allow(clippy::type_complexity)]
fn spire_remote_pipeline_step_table(
    rows: Vec<SpireRemotePipelineStepRow>,
) -> TableIterator<
    'static,
    (
        name!(step_ordinal, i64),
        name!(step_name, &'static str),
        name!(requested_epoch, i64),
        name!(status, String),
        name!(item_count, i64),
        name!(ready_count, i64),
        name!(blocked_count, i64),
        name!(remote_pid_count, i64),
        name!(next_blocker, String),
        name!(recommendation, String),
    ),
> {
    TableIterator::new(rows.into_iter().map(|row| {
        (
            row.step_ordinal,
            row.step_name,
            row.requested_epoch,
            row.status,
            row.item_count,
            row.ready_count,
            row.blocked_count,
            row.remote_pid_count,
            row.next_blocker,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_pipeline_steps(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(step_ordinal, i64),
        name!(step_name, &'static str),
        name!(requested_epoch, i64),
        name!(status, String),
        name!(item_count, i64),
        name!(ready_count, i64),
        name!(blocked_count, i64),
        name!(remote_pid_count, i64),
        name!(next_blocker, String),
        name!(recommendation, String),
    ),
> {
    spire_remote_pipeline_step_table(spire_remote_pipeline_step_rows(
        "ec_spire_remote_pipeline_steps",
        index_oid,
        requested_epoch,
        query,
        selected_pids,
        top_k,
        consistency_mode,
        false,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_pipeline_steps_live(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(step_ordinal, i64),
        name!(step_name, &'static str),
        name!(requested_epoch, i64),
        name!(status, String),
        name!(item_count, i64),
        name!(ready_count, i64),
        name!(blocked_count, i64),
        name!(remote_pid_count, i64),
        name!(next_blocker, String),
        name!(recommendation, String),
    ),
> {
    spire_remote_pipeline_step_table(spire_remote_pipeline_step_rows(
        "ec_spire_remote_pipeline_steps_live",
        index_oid,
        requested_epoch,
        query,
        selected_pids,
        top_k,
        consistency_mode,
        true,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_receive_plan(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(node_id, i64),
        name!(selected_pids, Vec<i64>),
        name!(pid_count, i64),
        name!(expected_candidate_format, &'static str),
        name!(expected_result_column_count, i64),
        name!(validator_function, &'static str),
        name!(row_locator_policy, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!("ec_spire_remote_search_receive_plan requested_epoch must be greater than 0");
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_receive_plan top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!("ec_spire_remote_search_receive_plan selected PID {pid} is negative")
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_receive_plan") };
    let rows = unsafe {
        am::spire_remote_search_receive_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            i64::from(row.node_id),
            row.selected_pids
                .into_iter()
                .map(|pid| i64::try_from(pid).expect("pid should fit in i64"))
                .collect::<Vec<_>>(),
            i64::try_from(row.pid_count).expect("pid count should fit in i64"),
            row.expected_candidate_format,
            i64::try_from(row.expected_result_column_count)
                .expect("expected result column count should fit in i64"),
            row.validator_function,
            row.row_locator_policy,
            row.status,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_receive_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(receive_count, i64),
        name!(ready_receive_count, i64),
        name!(blocked_receive_count, i64),
        name!(remote_pid_count, i64),
        name!(blocked_pid_count, i64),
        name!(expected_result_column_count, i64),
        name!(validator_function, &'static str),
        name!(row_locator_policy, &'static str),
        name!(status, String),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_receive_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_receive_summary top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_receive_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_receive_summary") };
    let rows = unsafe {
        am::spire_remote_search_receive_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let mut ready_receive_count = 0_u64;
    let mut blocked_receive_count = 0_u64;
    let mut remote_pid_count = 0_u64;
    let mut blocked_pid_count = 0_u64;
    let mut expected_result_column_count = 0_u64;
    let mut validator_function = "none";
    let mut row_locator_policy = "none";
    let mut first_blocked_status = "ready";

    for row in &rows {
        remote_pid_count = remote_pid_count
            .checked_add(row.pid_count)
            .unwrap_or_else(|| pgrx::error!("remote search receive remote pid overflow"));
        expected_result_column_count =
            expected_result_column_count.max(row.expected_result_column_count);
        validator_function = row.validator_function;
        row_locator_policy = row.row_locator_policy;
        if row.status == "ready" {
            ready_receive_count = ready_receive_count
                .checked_add(1)
                .unwrap_or_else(|| pgrx::error!("remote search receive ready count overflow"));
        } else {
            blocked_receive_count = blocked_receive_count
                .checked_add(1)
                .unwrap_or_else(|| pgrx::error!("remote search receive blocked count overflow"));
            blocked_pid_count = blocked_pid_count
                .checked_add(row.pid_count)
                .unwrap_or_else(|| pgrx::error!("remote search receive blocked pid overflow"));
            if first_blocked_status == "ready" {
                first_blocked_status = row.status;
            }
        }
    }

    let receive_count =
        u64::try_from(rows.len()).expect("remote search receive count should fit in u64");
    let status = if blocked_receive_count == 0 {
        "ready".to_owned()
    } else {
        first_blocked_status.to_owned()
    };

    TableIterator::once((
        i64::try_from(requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(receive_count).expect("receive count should fit in i64"),
        i64::try_from(ready_receive_count).expect("ready receive count should fit in i64"),
        i64::try_from(blocked_receive_count).expect("blocked receive count should fit in i64"),
        i64::try_from(remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(blocked_pid_count).expect("blocked pid count should fit in i64"),
        i64::try_from(expected_result_column_count)
            .expect("expected result column count should fit in i64"),
        validator_function,
        row_locator_policy,
        status,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_merge_input_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(remote_batch_count, i64),
        name!(local_batch_count, i64),
        name!(skipped_batch_count, i64),
        name!(ready_batch_count, i64),
        name!(blocked_batch_count, i64),
        name!(remote_pid_count, i64),
        name!(local_pid_count, i64),
        name!(skipped_pid_count, i64),
        name!(merge_function, &'static str),
        name!(dedupe_key, &'static str),
        name!(tie_breaker, &'static str),
        name!(top_k, i64),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_merge_input_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_merge_input_summary top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_merge_input_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_merge_input_summary")
    };
    let row = unsafe {
        am::spire_remote_search_merge_input_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.remote_batch_count).expect("remote batch count should fit in i64"),
        i64::try_from(row.local_batch_count).expect("local batch count should fit in i64"),
        i64::try_from(row.skipped_batch_count).expect("skipped batch count should fit in i64"),
        i64::try_from(row.ready_batch_count).expect("ready batch count should fit in i64"),
        i64::try_from(row.blocked_batch_count).expect("blocked batch count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.local_pid_count).expect("local pid count should fit in i64"),
        i64::try_from(row.skipped_pid_count).expect("skipped pid count should fit in i64"),
        row.merge_function,
        row.dedupe_key,
        row.tie_breaker,
        i64::try_from(row.top_k).expect("top_k should fit in i64"),
        row.status,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_merge_order_contract() -> TableIterator<
    'static,
    (
        name!(order_ordinal, i64),
        name!(order_key, &'static str),
        name!(direction, &'static str),
        name!(semantic_role, &'static str),
        name!(validator, &'static str),
    ),
> {
    let rows = am::spire_remote_search_merge_order_contract_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.order_ordinal).expect("order ordinal should fit in i64"),
            row.order_key,
            row.direction,
            row.semantic_role,
            row.validator,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_row_locator_contract() -> TableIterator<
    'static,
    (
        name!(contract_item, &'static str),
        name!(contract_value, &'static str),
        name!(status, &'static str),
    ),
> {
    let rows = am::spire_remote_search_row_locator_contract_rows();
    TableIterator::new(
        rows.into_iter()
            .map(|row| (row.contract_item, row.contract_value, row.status)),
    )
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_vector_identity_contract() -> TableIterator<
    'static,
    (
        name!(contract_item, &'static str),
        name!(contract_value, &'static str),
        name!(status, &'static str),
    ),
> {
    let rows = am::spire_remote_search_vector_identity_contract_rows();
    TableIterator::new(
        rows.into_iter()
            .map(|row| (row.contract_item, row.contract_value, row.status)),
    )
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_heap_resolution_contract() -> TableIterator<
    'static,
    (
        name!(resolution_scope, &'static str),
        name!(candidate_source, &'static str),
        name!(heap_lookup_owner, &'static str),
        name!(row_locator_policy, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    let rows = am::spire_remote_search_heap_resolution_contract_rows();
    TableIterator::new(rows.into_iter().map(|row| {
        (
            row.resolution_scope,
            row.candidate_source,
            row.heap_lookup_owner,
            row.row_locator_policy,
            row.status,
            row.recommendation,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_local_heap_resolution_plan(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(node_id, i64),
        name!(pid, i64),
        name!(row_index, i64),
        name!(vec_id, Vec<u8>),
        name!(row_locator, Vec<u8>),
        name!(heap_block, i64),
        name!(heap_offset, i32),
        name!(heap_lookup_owner, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_local_heap_resolution_plan requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_local_heap_resolution_plan top_k must be non-negative"
        );
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_local_heap_resolution_plan selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_local_heap_resolution_plan",
        )
    };
    let rows = unsafe {
        am::spire_remote_search_local_heap_resolution_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            i64::from(row.node_id),
            i64::try_from(row.pid).expect("pid should fit in i64"),
            i64::from(row.row_index),
            row.vec_id,
            row.row_locator,
            i64::from(row.heap_block),
            i32::from(row.heap_offset),
            row.heap_lookup_owner,
            row.status,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_heap_resolution_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(local_plan_count, i64),
        name!(remote_plan_count, i64),
        name!(skipped_plan_count, i64),
        name!(local_pid_count, i64),
        name!(remote_pid_count, i64),
        name!(decoded_local_locator_count, i64),
        name!(local_heap_resolution_status, &'static str),
        name!(remote_heap_resolution_status, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_heap_resolution_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_heap_resolution_summary top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_heap_resolution_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_heap_resolution_summary")
    };
    let row = unsafe {
        am::spire_remote_search_heap_resolution_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.local_plan_count).expect("local plan count should fit in i64"),
        i64::try_from(row.remote_plan_count).expect("remote plan count should fit in i64"),
        i64::try_from(row.skipped_plan_count).expect("skipped plan count should fit in i64"),
        i64::try_from(row.local_pid_count).expect("local pid count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.decoded_local_locator_count)
            .expect("decoded local locator count should fit in i64"),
        row.local_heap_resolution_status,
        row.remote_heap_resolution_status,
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_local_heap_candidates(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(served_epoch, i64),
        name!(node_id, i64),
        name!(pid, i64),
        name!(object_version, i64),
        name!(row_index, i64),
        name!(assignment_flags, i16),
        name!(vec_id, Vec<u8>),
        name!(row_locator, Vec<u8>),
        name!(heap_block, i64),
        name!(heap_offset, i32),
        name!(score, f32),
        name!(heap_lookup_owner, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_local_heap_candidates requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_local_heap_candidates top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_local_heap_candidates selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_local_heap_candidates")
    };
    let rows = unsafe {
        am::spire_remote_search_local_heap_candidate_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
            i64::try_from(row.served_epoch).expect("served epoch should fit in i64"),
            i64::from(row.node_id),
            i64::try_from(row.pid).expect("pid should fit in i64"),
            i64::try_from(row.object_version).expect("object version should fit in i64"),
            i64::from(row.row_index),
            i16::try_from(row.assignment_flags).expect("assignment flags should fit in i16"),
            row.vec_id,
            row.row_locator,
            i64::from(row.heap_block),
            i32::from(row.heap_offset),
            row.score,
            row.heap_lookup_owner,
            row.status,
        )
    }))
}

fn ec_spire_validate_tuple_payload_columns(
    heap_relation_oid: pg_sys::Oid,
    requested_columns: &[String],
) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    for column in requested_columns {
        if column.is_empty() {
            return Err("requested column name must be nonempty".to_owned());
        }
        if !seen.insert(column.as_str()) {
            return Err(format!(
                "requested column \"{column}\" appears more than once"
            ));
        }
    }

    let requested_column_refs = requested_columns
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let valid_count = Spi::connect(|client| {
        client
            .select(
                "SELECT count(*)::bigint AS valid_count \
                   FROM unnest($1::text[]) AS requested(column_name) \
                   JOIN pg_attribute AS attr \
                     ON attr.attrelid = $2::oid \
                    AND attr.attname = requested.column_name \
                    AND attr.attnum > 0 \
                    AND NOT attr.attisdropped",
                None,
                &[
                    requested_column_refs.as_slice().into(),
                    heap_relation_oid.into(),
                ],
            )
            .map_err(|e| format!("ec_spire tuple payload column validation failed: {e}"))?
            .first()
            .get_one::<i64>()
            .map_err(|e| format!("ec_spire tuple payload valid_count decode failed: {e}"))?
            .ok_or_else(|| "ec_spire tuple payload valid_count is null".to_owned())
    })?;

    let requested_count = i64::try_from(requested_columns.len())
        .map_err(|_| "requested column count exceeds i64".to_owned())?;
    if valid_count != requested_count {
        return Err(
            "requested columns must all be ordinary columns on the indexed heap relation"
                .to_owned(),
        );
    }

    Ok(())
}

struct EcSpireTypedTuplePayloadColumn {
    attnum: i16,
    name: String,
    type_oid: pg_sys::Oid,
    typmod: i32,
    collation: pg_sys::Oid,
    send_function_sql: String,
}

fn ec_spire_tuple_payload_column_metadata(
    heap_relation_oid: pg_sys::Oid,
    requested_columns: &[String],
    context: &str,
) -> Result<Vec<EcSpireTypedTuplePayloadColumn>, String> {
    ec_spire_validate_tuple_payload_columns(heap_relation_oid, requested_columns)
        .map_err(|e| format!("{context} {e}"))?;
    let requested_column_refs = requested_columns
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    Spi::connect(|client| {
        let rows = client
            .select(
                "SELECT attr.attnum::int2 AS attnum, \
                        attr.attname::text AS attname, \
                        attr.atttypid::oid AS atttypid, \
                        attr.atttypmod::int4 AS atttypmod, \
                        attr.attcollation::oid AS attcollation, \
                        format_type(attr.atttypid, attr.atttypmod)::text AS type_name, \
                        typ.typsend::oid AS typsend_oid, \
                        proc_namespace.nspname::text AS send_schema, \
                        send_proc.proname::text AS send_name \
                   FROM unnest($1::text[]) WITH ORDINALITY AS requested(column_name, ordinality) \
                   JOIN pg_attribute AS attr \
                     ON attr.attrelid = $2::oid \
                    AND attr.attname = requested.column_name \
                    AND attr.attnum > 0 \
                    AND NOT attr.attisdropped \
                   JOIN pg_type AS typ \
                     ON typ.oid = attr.atttypid \
              LEFT JOIN pg_proc AS send_proc \
                     ON send_proc.oid = typ.typsend \
              LEFT JOIN pg_namespace AS proc_namespace \
                     ON proc_namespace.oid = send_proc.pronamespace \
                  ORDER BY requested.ordinality",
                None,
                &[
                    requested_column_refs.as_slice().into(),
                    heap_relation_oid.into(),
                ],
            )
            .map_err(|e| format!("{context} typed metadata lookup failed: {e}"))?;
        let mut metadata = Vec::with_capacity(requested_columns.len());
        for row in rows {
            let attnum = row["attnum"]
                .value::<i16>()
                .map_err(|e| format!("{context} typed attnum decode failed: {e}"))?
                .ok_or_else(|| format!("{context} typed attnum is null"))?;
            let name = row["attname"]
                .value::<String>()
                .map_err(|e| format!("{context} typed attname decode failed: {e}"))?
                .ok_or_else(|| format!("{context} typed attname is null"))?;
            let type_oid = row["atttypid"]
                .value::<pg_sys::Oid>()
                .map_err(|e| format!("{context} typed atttypid decode failed: {e}"))?
                .ok_or_else(|| format!("{context} typed atttypid is null"))?;
            let typmod = row["atttypmod"]
                .value::<i32>()
                .map_err(|e| format!("{context} typed atttypmod decode failed: {e}"))?
                .ok_or_else(|| format!("{context} typed atttypmod is null"))?;
            let collation = row["attcollation"]
                .value::<pg_sys::Oid>()
                .map_err(|e| format!("{context} typed attcollation decode failed: {e}"))?
                .ok_or_else(|| format!("{context} typed attcollation is null"))?;
            let type_name = row["type_name"]
                .value::<String>()
                .map_err(|e| format!("{context} typed type name decode failed: {e}"))?
                .ok_or_else(|| format!("{context} typed type name is null"))?;
            let send_oid = row["typsend_oid"]
                .value::<pg_sys::Oid>()
                .map_err(|e| format!("{context} typed typsend decode failed: {e}"))?
                .ok_or_else(|| format!("{context} typed typsend is null"))?;
            if send_oid == pg_sys::InvalidOid {
                return Err(format!(
                    "{context} unsupported_type_binary_io for column \"{name}\" type {type_name} oid {type_oid}"
                ));
            }
            let send_schema = row["send_schema"]
                .value::<String>()
                .map_err(|e| format!("{context} typed send schema decode failed: {e}"))?
                .ok_or_else(|| format!("{context} typed send schema is null"))?;
            let send_name = row["send_name"]
                .value::<String>()
                .map_err(|e| format!("{context} typed send name decode failed: {e}"))?
                .ok_or_else(|| format!("{context} typed send name is null"))?;
            metadata.push(EcSpireTypedTuplePayloadColumn {
                attnum,
                name,
                type_oid,
                typmod,
                collation,
                send_function_sql: pgrx::spi::quote_qualified_identifier(&send_schema, &send_name),
            });
        }
        if metadata.len() != requested_columns.len() {
            return Err(format!(
                "{context} typed metadata returned {} rows for {} requested columns",
                metadata.len(),
                requested_columns.len()
            ));
        }
        Ok(metadata)
    })
}

fn ec_spire_quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn ec_spire_relation_regclass_text(relation_oid: pg_sys::Oid) -> Result<String, String> {
    Spi::connect(|client| {
        client
            .select(
                "SELECT $1::oid::regclass::text AS relation_name",
                None,
                &[relation_oid.into()],
            )
            .map_err(|e| format!("ec_spire tuple payload relation lookup failed: {e}"))?
            .first()
            .get_one::<String>()
            .map_err(|e| format!("ec_spire tuple payload relation name decode failed: {e}"))?
            .ok_or_else(|| "ec_spire tuple payload relation name is null".to_owned())
    })
}

fn ec_spire_index_key_column_names(
    index_oid: pg_sys::Oid,
    context: &str,
) -> Result<Vec<String>, String> {
    Spi::connect(|client| {
        client
            .select(
                "SELECT coalesce(array_agg(attr.attname::text ORDER BY key_column.ord), ARRAY[]::text[]) \
                   AS key_columns \
                   FROM pg_index AS idx \
                   JOIN unnest(idx.indkey) WITH ORDINALITY AS key_column(attnum, ord) \
                     ON key_column.attnum > 0 \
                   JOIN pg_attribute AS attr \
                     ON attr.attrelid = idx.indrelid \
                    AND attr.attnum = key_column.attnum \
                  WHERE idx.indexrelid = $1::oid",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| format!("{context} index key column lookup failed: {e}"))?
            .first()
            .get_one::<Vec<String>>()
            .map_err(|e| format!("{context} index key column decode failed: {e}"))?
            .ok_or_else(|| format!("{context} index key column list is null"))
    })
}

fn ec_spire_reject_distributed_embedding_update() -> ! {
    pgrx::pg_sys::panic::ErrorReport::new(
        pgrx::PgSqlErrorCode::ERRCODE_FEATURE_NOT_SUPPORTED,
        "ec_spire_distributed: UPDATE of indexed embedding column is not supported on a distributed ec_spire table. Use DELETE + INSERT.",
        pgrx::function_name!(),
    )
    .set_hint("Cross-shard atomic moves will be available in a future release.")
    .report(pgrx::PgLogLevel::ERROR);
    unreachable!();
}

fn ec_spire_delete_placement_row(index_oid: pg_sys::Oid, pk_value: &[u8]) -> Result<bool, String> {
    Spi::connect_mut(|client| {
        client
            .select(
                "DELETE FROM ec_spire_placement \
                  WHERE index_oid = $1::oid \
                    AND pk_value = $2::bytea \
              RETURNING true AS deleted",
                None,
                &[index_oid.into(), pk_value.to_vec().into()],
            )
            .map_err(|e| format!("ec_spire coordinator delete placement delete failed: {e}"))?
            .map(|row| {
                row["deleted"]
                    .value::<bool>()
                    .map_err(|e| {
                        format!("ec_spire coordinator delete placement delete decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire coordinator delete placement delete result is null".to_owned()
                    })
            })
            .next()
            .transpose()
            .map(|value| value.unwrap_or(false))
    })
}

#[pg_extern(strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_insert_tuple_payload(
    index_oid: pg_sys::Oid,
    row_payload: pgrx::JsonB,
    requested_columns: Vec<String>,
) -> TableIterator<
    'static,
    (
        name!(index_oid, pg_sys::Oid),
        name!(heap_relation_oid, pg_sys::Oid),
        name!(inserted_count, i64),
        name!(payload_column_count, i32),
        name!(status, &'static str),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_insert_tuple_payload") };
    let heap_relation_oid = unsafe {
        (*index_relation)
            .rd_index
            .as_ref()
            .expect("opened index relation should expose pg_index metadata")
            .indrelid
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    if requested_columns.is_empty() {
        pgrx::error!("ec_spire_remote_insert_tuple_payload requested column list must be nonempty");
    }
    ec_spire_validate_tuple_payload_columns(heap_relation_oid, &requested_columns)
        .unwrap_or_else(|e| pgrx::error!("ec_spire_remote_insert_tuple_payload {e}"));
    let heap_relation_regclass = ec_spire_relation_regclass_text(heap_relation_oid)
        .unwrap_or_else(|e| pgrx::error!("ec_spire_remote_insert_tuple_payload {e}"));
    let payload_column_count = i32::try_from(requested_columns.len())
        .unwrap_or_else(|_| pgrx::error!("ec_spire_remote_insert_tuple_payload too many columns"));
    let column_list = requested_columns
        .iter()
        .map(|column| ec_spire_quote_identifier(column))
        .collect::<Vec<_>>()
        .join(", ");
    let row_payload_text = row_payload.0.to_string();
    let insert_sql = format!(
        "WITH inserted AS ( \
             INSERT INTO {heap_relation_regclass} ({column_list}) \
             SELECT {column_list} \
               FROM jsonb_populate_record(NULL::{heap_relation_regclass}, $1::text::jsonb) \
             RETURNING 1 \
         ) \
         SELECT count(*)::bigint AS inserted_count FROM inserted"
    );
    let inserted_count = Spi::connect_mut(|client| {
        client
            .select(insert_sql.as_str(), None, &[row_payload_text.into()])
            .map_err(|e| format!("ec_spire remote tuple payload insert failed: {e}"))?
            .first()
            .get_one::<i64>()
            .map_err(|e| {
                format!("ec_spire remote tuple payload inserted_count decode failed: {e}")
            })?
            .ok_or_else(|| "ec_spire remote tuple payload inserted_count is null".to_owned())
    })
    .unwrap_or_else(|e| pgrx::error!("ec_spire_remote_insert_tuple_payload {e}"));

    TableIterator::once((
        index_oid,
        heap_relation_oid,
        inserted_count,
        payload_column_count,
        "ready",
    ))
}

fn ec_spire_update_tuple_payload_on_heap(
    heap_relation_oid: pg_sys::Oid,
    pk_column: &str,
    pk_value: &[u8],
    row_payload_json: &str,
    updated_columns: &[String],
    context: &str,
) -> Result<u64, String> {
    let mut validation_columns = Vec::with_capacity(updated_columns.len() + 1);
    validation_columns.push(pk_column.to_owned());
    validation_columns.extend(updated_columns.iter().cloned());
    ec_spire_validate_tuple_payload_columns(heap_relation_oid, &validation_columns)
        .map_err(|e| format!("{context} {e}"))?;
    let heap_relation_regclass =
        ec_spire_relation_regclass_text(heap_relation_oid).map_err(|e| format!("{context} {e}"))?;
    let pk_identifier = ec_spire_quote_identifier(pk_column);
    let set_list = updated_columns
        .iter()
        .map(|column| {
            let identifier = ec_spire_quote_identifier(column);
            format!("{identifier} = payload.{identifier}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    let update_sql = format!(
        "WITH payload AS ( \
             SELECT * \
               FROM jsonb_populate_record(NULL::{heap_relation_regclass}, $1::text::jsonb) \
         ), updated AS ( \
             UPDATE {heap_relation_regclass} AS target \
                SET {set_list} \
               FROM payload \
              WHERE int8send(target.{pk_identifier}::bigint)::bytea = $2::bytea \
          RETURNING 1 \
         ) \
         SELECT count(*)::bigint AS updated_count FROM updated"
    );
    Spi::connect_mut(|client| {
        client
            .select(
                update_sql.as_str(),
                None,
                &[row_payload_json.into(), pk_value.to_vec().into()],
            )
            .map_err(|e| format!("{context} remote tuple payload update failed: {e}"))?
            .first()
            .get_one::<i64>()
            .map_err(|e| {
                format!("{context} remote tuple payload updated_count decode failed: {e}")
            })?
            .ok_or_else(|| format!("{context} remote tuple payload updated_count is null"))
            .and_then(|value| {
                u64::try_from(value).map_err(|_| {
                    format!("{context} remote tuple payload updated_count is negative")
                })
            })
    })
}

fn ec_spire_delete_tuple_payload_on_heap(
    heap_relation_oid: pg_sys::Oid,
    pk_column: &str,
    pk_value: &[u8],
    context: &str,
) -> Result<u64, String> {
    ec_spire_validate_tuple_payload_columns(heap_relation_oid, &[pk_column.to_owned()])
        .map_err(|e| format!("{context} {e}"))?;
    let heap_relation_regclass =
        ec_spire_relation_regclass_text(heap_relation_oid).map_err(|e| format!("{context} {e}"))?;
    let pk_identifier = ec_spire_quote_identifier(pk_column);
    let delete_sql = format!(
        "WITH deleted AS ( \
             DELETE FROM {heap_relation_regclass} AS target \
              WHERE int8send(target.{pk_identifier}::bigint)::bytea = $1::bytea \
          RETURNING 1 \
         ) \
         SELECT count(*)::bigint AS deleted_count FROM deleted"
    );
    Spi::connect_mut(|client| {
        client
            .select(delete_sql.as_str(), None, &[pk_value.to_vec().into()])
            .map_err(|e| format!("{context} tuple payload delete failed: {e}"))?
            .first()
            .get_one::<i64>()
            .map_err(|e| format!("{context} tuple payload deleted_count decode failed: {e}"))?
            .ok_or_else(|| format!("{context} tuple payload deleted_count is null"))
            .and_then(|value| {
                u64::try_from(value)
                    .map_err(|_| format!("{context} tuple payload deleted_count is negative"))
            })
    })
}

fn ec_spire_select_tuple_payload_on_heap(
    heap_relation_oid: pg_sys::Oid,
    pk_column: &str,
    pk_value: &[u8],
    requested_columns: &[String],
    context: &str,
) -> Result<(u64, Option<String>), String> {
    let mut validation_columns = Vec::with_capacity(requested_columns.len() + 1);
    if !requested_columns.iter().any(|column| column == pk_column) {
        validation_columns.push(pk_column.to_owned());
    }
    validation_columns.extend(requested_columns.iter().cloned());
    ec_spire_validate_tuple_payload_columns(heap_relation_oid, &validation_columns)
        .map_err(|e| format!("{context} {e}"))?;
    let heap_relation_regclass =
        ec_spire_relation_regclass_text(heap_relation_oid).map_err(|e| format!("{context} {e}"))?;
    let pk_identifier = ec_spire_quote_identifier(pk_column);
    let column_list = requested_columns
        .iter()
        .map(|column| ec_spire_quote_identifier(column))
        .collect::<Vec<_>>()
        .join(", ");
    let select_sql = format!(
        "WITH selected AS ( \
             SELECT {column_list} \
               FROM {heap_relation_regclass} AS target \
              WHERE int8send(target.{pk_identifier}::bigint)::bytea = $1::bytea \
         ), summarized AS ( \
             SELECT count(*)::bigint AS selected_count, \
                    jsonb_agg(to_jsonb(selected)) AS payloads \
               FROM selected \
         ) \
         SELECT selected_count, \
                CASE WHEN selected_count = 1 THEN (payloads -> 0)::text ELSE NULL END \
                    AS tuple_payload_json \
           FROM summarized"
    );
    Spi::connect(|client| {
        client
            .select(select_sql.as_str(), None, &[pk_value.to_vec().into()])
            .map_err(|e| format!("{context} tuple payload select failed: {e}"))?
            .map(|row| {
                let selected_count = row["selected_count"]
                    .value::<i64>()
                    .map_err(|e| {
                        format!("{context} tuple payload selected_count decode failed: {e}")
                    })?
                    .ok_or_else(|| format!("{context} tuple payload selected_count is null"))
                    .and_then(|value| {
                        u64::try_from(value).map_err(|_| {
                            format!("{context} tuple payload selected_count is negative")
                        })
                    })?;
                let tuple_payload_json = row["tuple_payload_json"]
                    .value::<String>()
                    .map_err(|e| format!("{context} tuple payload JSON decode failed: {e}"))?;
                Ok::<(u64, Option<String>), String>((selected_count, tuple_payload_json))
            })
            .next()
            .transpose()
            .map(|value| {
                value.ok_or_else(|| format!("{context} tuple payload select returned no rows"))
            })?
    })
}

#[pg_extern(strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_update_tuple_payload(
    index_oid: pg_sys::Oid,
    pk_column: String,
    pk_value: Vec<u8>,
    row_payload: pgrx::JsonB,
    updated_columns: Vec<String>,
) -> TableIterator<
    'static,
    (
        name!(index_oid, pg_sys::Oid),
        name!(heap_relation_oid, pg_sys::Oid),
        name!(updated_count, i64),
        name!(payload_column_count, i32),
        name!(status, &'static str),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_update_tuple_payload") };
    let heap_relation_oid = unsafe {
        (*index_relation)
            .rd_index
            .as_ref()
            .expect("opened index relation should expose pg_index metadata")
            .indrelid
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    if pk_column.is_empty() {
        pgrx::error!("ec_spire_remote_update_tuple_payload pk_column must be nonempty");
    }
    if pk_value.is_empty() {
        pgrx::error!("ec_spire_remote_update_tuple_payload pk_value must not be empty");
    }
    if updated_columns.is_empty() {
        pgrx::error!("ec_spire_remote_update_tuple_payload updated column list must be nonempty");
    }
    if updated_columns.iter().any(|column| column == &pk_column) {
        pgrx::error!("ec_spire_remote_update_tuple_payload must not update the primary-key column");
    }
    let payload_column_count = i32::try_from(updated_columns.len()).unwrap_or_else(|_| {
        pgrx::error!("ec_spire_remote_update_tuple_payload too many updated columns")
    });
    let row_payload_text = row_payload.0.to_string();
    let updated_count = ec_spire_update_tuple_payload_on_heap(
        heap_relation_oid,
        &pk_column,
        &pk_value,
        &row_payload_text,
        &updated_columns,
        "ec_spire_remote_update_tuple_payload",
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    TableIterator::once((
        index_oid,
        heap_relation_oid,
        i64::try_from(updated_count).expect("updated count should fit i64"),
        payload_column_count,
        "ready",
    ))
}

#[pg_extern(strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_delete_tuple_payload(
    index_oid: pg_sys::Oid,
    pk_column: String,
    pk_value: Vec<u8>,
) -> TableIterator<
    'static,
    (
        name!(index_oid, pg_sys::Oid),
        name!(heap_relation_oid, pg_sys::Oid),
        name!(deleted_count, i64),
        name!(status, &'static str),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_delete_tuple_payload") };
    let heap_relation_oid = unsafe {
        (*index_relation)
            .rd_index
            .as_ref()
            .expect("opened index relation should expose pg_index metadata")
            .indrelid
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    if pk_column.is_empty() {
        pgrx::error!("ec_spire_remote_delete_tuple_payload pk_column must be nonempty");
    }
    if pk_value.is_empty() {
        pgrx::error!("ec_spire_remote_delete_tuple_payload pk_value must not be empty");
    }
    let deleted_count = ec_spire_delete_tuple_payload_on_heap(
        heap_relation_oid,
        &pk_column,
        &pk_value,
        "ec_spire_remote_delete_tuple_payload",
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"));

    TableIterator::once((
        index_oid,
        heap_relation_oid,
        i64::try_from(deleted_count).expect("deleted count should fit i64"),
        "ready",
    ))
}

#[pg_extern(strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_select_tuple_payload(
    index_oid: pg_sys::Oid,
    pk_column: String,
    pk_value: Vec<u8>,
    requested_columns: Vec<String>,
) -> TableIterator<
    'static,
    (
        name!(index_oid, pg_sys::Oid),
        name!(heap_relation_oid, pg_sys::Oid),
        name!(selected_count, i64),
        name!(payload_column_count, i32),
        name!(tuple_payload_json, Option<String>),
        name!(status, &'static str),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_select_tuple_payload") };
    let heap_relation_oid = unsafe {
        (*index_relation)
            .rd_index
            .as_ref()
            .expect("opened index relation should expose pg_index metadata")
            .indrelid
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    if pk_column.is_empty() {
        pgrx::error!("ec_spire_remote_select_tuple_payload pk_column must be nonempty");
    }
    if pk_value.is_empty() {
        pgrx::error!("ec_spire_remote_select_tuple_payload pk_value must not be empty");
    }
    if requested_columns.is_empty() {
        pgrx::error!("ec_spire_remote_select_tuple_payload requested column list must be nonempty");
    }
    let payload_column_count = i32::try_from(requested_columns.len()).unwrap_or_else(|_| {
        pgrx::error!("ec_spire_remote_select_tuple_payload too many requested columns")
    });
    let (selected_count, tuple_payload_json) = ec_spire_select_tuple_payload_on_heap(
        heap_relation_oid,
        &pk_column,
        &pk_value,
        &requested_columns,
        "ec_spire_remote_select_tuple_payload",
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    if selected_count > 1 {
        pgrx::error!(
            "ec_spire remote select expected at most one row, got {}",
            selected_count
        );
    }

    TableIterator::once((
        index_oid,
        heap_relation_oid,
        i64::try_from(selected_count).expect("selected count should fit i64"),
        payload_column_count,
        tuple_payload_json,
        "ready",
    ))
}

fn ec_spire_remote_search_tuple_payloads_for_ctids(
    heap_relation_regclass: &str,
    requested_columns: &[String],
    ctids: &[String],
) -> Result<Vec<(pgrx::JsonB, bool)>, String> {
    if ctids.is_empty() {
        return Ok(Vec::new());
    }
    let requested_column_refs = requested_columns
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let ctid_refs = ctids.iter().map(String::as_str).collect::<Vec<_>>();
    let sql = format!(
        "SELECT candidate.ctid_text, \
                heap.payload IS NULL AS tuple_payload_missing, \
                CASE WHEN heap.payload IS NULL THEN '{{}}'::jsonb \
                     ELSE ( \
                       SELECT coalesce( \
                                jsonb_object_agg(requested.column_name, \
                                                 heap.payload -> requested.column_name \
                                                 ORDER BY requested.ordinality), \
                                '{{}}'::jsonb) \
                         FROM unnest($2::text[]) WITH ORDINALITY \
                              AS requested(column_name, ordinality) \
                     ) \
                 END AS tuple_payload \
           FROM unnest($1::text[]) WITH ORDINALITY AS candidate(ctid_text, ordinality) \
           LEFT JOIN LATERAL ( \
             SELECT to_jsonb(heap_row) AS payload \
               FROM {heap_relation_regclass} AS heap_row \
              WHERE heap_row.ctid = candidate.ctid_text::tid \
           ) AS heap ON true \
          ORDER BY candidate.ordinality"
    );

    Spi::connect(|client| {
        let rows = client
            .select(
                sql.as_str(),
                None,
                &[
                    ctid_refs.as_slice().into(),
                    requested_column_refs.as_slice().into(),
                ],
            )
            .map_err(|e| format!("ec_spire tuple payload heap fetch failed: {e}"))?;
        let mut payloads = Vec::with_capacity(ctids.len());
        for row in rows {
            row.get::<String>(1)
                .map_err(|e| format!("ec_spire tuple payload ctid decode failed: {e}"))?
                .ok_or_else(|| "ec_spire tuple payload ctid is null".to_owned())?;
            let tuple_payload_missing = row
                .get::<bool>(2)
                .map_err(|e| format!("ec_spire tuple payload missing flag decode failed: {e}"))?
                .ok_or_else(|| "ec_spire tuple payload missing flag is null".to_owned())?;
            let tuple_payload = row
                .get::<pgrx::JsonB>(3)
                .map_err(|e| format!("ec_spire tuple payload decode failed: {e}"))?
                .ok_or_else(|| "ec_spire tuple payload is null".to_owned())?;
            payloads.push((tuple_payload, tuple_payload_missing));
        }
        if payloads.len() != ctids.len() {
            return Err(format!(
                "ec_spire tuple payload heap fetch returned {} rows for {} CTIDs",
                payloads.len(),
                ctids.len()
            ));
        }
        Ok(payloads)
    })
}

fn ec_spire_remote_search_typed_tuple_payloads_for_ctids(
    heap_relation_regclass: &str,
    columns: &[EcSpireTypedTuplePayloadColumn],
    ctids: &[String],
) -> Result<Vec<(Vec<bool>, Vec<Vec<u8>>, bool)>, String> {
    if ctids.is_empty() {
        return Ok(Vec::new());
    }
    let ctid_refs = ctids.iter().map(String::as_str).collect::<Vec<_>>();
    let projected_columns = columns
        .iter()
        .map(|column| {
            let identifier = ec_spire_quote_identifier(&column.name);
            format!("heap_row.{identifier} AS {identifier}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    let found_projection = if projected_columns.is_empty() {
        "true AS __ec_spire_found".to_owned()
    } else {
        format!("true AS __ec_spire_found, {projected_columns}")
    };
    let payload_null_exprs = columns
        .iter()
        .map(|column| {
            let identifier = ec_spire_quote_identifier(&column.name);
            format!("(heap.__ec_spire_found IS NULL OR heap.{identifier} IS NULL)")
        })
        .collect::<Vec<_>>()
        .join(", ");
    let payload_value_exprs = columns
        .iter()
        .map(|column| {
            let identifier = ec_spire_quote_identifier(&column.name);
            let send_function = &column.send_function_sql;
            format!(
                "CASE WHEN heap.__ec_spire_found IS NULL OR heap.{identifier} IS NULL \
                      THEN ''::bytea \
                      ELSE {send_function}(heap.{identifier}) \
                 END"
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let payload_null_array = if payload_null_exprs.is_empty() {
        "ARRAY[]::boolean[]".to_owned()
    } else {
        format!("ARRAY[{payload_null_exprs}]::boolean[]")
    };
    let payload_value_array = if payload_value_exprs.is_empty() {
        "ARRAY[]::bytea[]".to_owned()
    } else {
        format!("ARRAY[{payload_value_exprs}]::bytea[]")
    };
    let sql = format!(
        "SELECT candidate.ctid_text, \
                heap.__ec_spire_found IS NULL AS tuple_payload_missing, \
                {payload_null_array} AS payload_nulls, \
                {payload_value_array} AS payload_values \
           FROM unnest($1::text[]) WITH ORDINALITY AS candidate(ctid_text, ordinality) \
           LEFT JOIN LATERAL ( \
             SELECT {found_projection} \
               FROM {heap_relation_regclass} AS heap_row \
              WHERE heap_row.ctid = candidate.ctid_text::tid \
           ) AS heap ON true \
          ORDER BY candidate.ordinality"
    );

    Spi::connect(|client| {
        let rows = client
            .select(sql.as_str(), None, &[ctid_refs.as_slice().into()])
            .map_err(|e| format!("ec_spire typed tuple payload heap fetch failed: {e}"))?;
        let mut payloads = Vec::with_capacity(ctids.len());
        for row in rows {
            row["ctid_text"]
                .value::<String>()
                .map_err(|e| format!("ec_spire typed tuple payload ctid decode failed: {e}"))?
                .ok_or_else(|| "ec_spire typed tuple payload ctid is null".to_owned())?;
            let tuple_payload_missing = row["tuple_payload_missing"]
                .value::<bool>()
                .map_err(|e| {
                    format!("ec_spire typed tuple payload missing flag decode failed: {e}")
                })?
                .ok_or_else(|| "ec_spire typed tuple payload missing flag is null".to_owned())?;
            let payload_nulls = row["payload_nulls"]
                .value::<Vec<bool>>()
                .map_err(|e| format!("ec_spire typed tuple payload nulls decode failed: {e}"))?
                .ok_or_else(|| "ec_spire typed tuple payload nulls are null".to_owned())?;
            let payload_values = row["payload_values"]
                .value::<pgrx::datum::Array<&[u8]>>()
                .map_err(|e| format!("ec_spire typed tuple payload values decode failed: {e}"))?
                .ok_or_else(|| "ec_spire typed tuple payload values are null".to_owned())?
                .iter_deny_null()
                .into_iter()
                .map(<[u8]>::to_vec)
                .collect::<Vec<_>>();
            if payload_nulls.len() != columns.len() || payload_values.len() != columns.len() {
                return Err(format!(
                    "ec_spire typed tuple payload returned {} null flags and {} values for {} columns",
                    payload_nulls.len(),
                    payload_values.len(),
                    columns.len()
                ));
            }
            payloads.push((payload_nulls, payload_values, tuple_payload_missing));
        }
        if payloads.len() != ctids.len() {
            return Err(format!(
                "ec_spire typed tuple payload heap fetch returned {} rows for {} CTIDs",
                payloads.len(),
                ctids.len()
            ));
        }
        Ok(payloads)
    })
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_tuple_payload(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
    requested_columns: Vec<String>,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(served_epoch, i64),
        name!(node_id, i64),
        name!(pid, i64),
        name!(object_version, i64),
        name!(row_index, i64),
        name!(assignment_flags, i16),
        name!(vec_id, Vec<u8>),
        name!(row_locator, Vec<u8>),
        name!(heap_block, i64),
        name!(heap_offset, i32),
        name!(score, f32),
        name!(tuple_payload, pgrx::JsonB),
        name!(tuple_payload_missing, bool),
        name!(payload_key, &'static str),
        name!(payload_column_count, i32),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!("ec_spire_remote_search_tuple_payload requested_epoch must be greater than 0");
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_tuple_payload top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!("ec_spire_remote_search_tuple_payload selected PID {pid} is negative")
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_tuple_payload") };
    let heap_relation_oid = unsafe {
        (*index_relation)
            .rd_index
            .as_ref()
            .expect("opened index relation should expose pg_index metadata")
            .indrelid
    };
    let rows = unsafe {
        am::spire_remote_search_local_heap_candidate_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    ec_spire_validate_tuple_payload_columns(heap_relation_oid, &requested_columns)
        .unwrap_or_else(|e| pgrx::error!("ec_spire_remote_search_tuple_payload {e}"));
    let heap_relation_regclass = ec_spire_relation_regclass_text(heap_relation_oid)
        .unwrap_or_else(|e| pgrx::error!("ec_spire_remote_search_tuple_payload {e}"));
    let payload_column_count = i32::try_from(requested_columns.len())
        .unwrap_or_else(|_| pgrx::error!("ec_spire_remote_search_tuple_payload too many columns"));
    let ctids = rows
        .iter()
        .map(|row| format!("({},{})", row.heap_block, row.heap_offset))
        .collect::<Vec<_>>();
    let payloads = ec_spire_remote_search_tuple_payloads_for_ctids(
        &heap_relation_regclass,
        &requested_columns,
        &ctids,
    )
    .unwrap_or_else(|e| pgrx::error!("ec_spire_remote_search_tuple_payload {e}"));

    let payload_rows = rows
        .into_iter()
        .zip(payloads)
        .map(|(row, (tuple_payload, tuple_payload_missing))| {
            let status = if tuple_payload_missing {
                "remote_tuple_payload_missing"
            } else {
                row.status
            };
            (
                i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
                i64::try_from(row.served_epoch).expect("served epoch should fit in i64"),
                i64::from(row.node_id),
                i64::try_from(row.pid).expect("pid should fit in i64"),
                i64::try_from(row.object_version).expect("object version should fit in i64"),
                i64::from(row.row_index),
                i16::try_from(row.assignment_flags).expect("assignment flags should fit in i16"),
                row.vec_id,
                row.row_locator,
                i64::from(row.heap_block),
                i32::from(row.heap_offset),
                row.score,
                tuple_payload,
                tuple_payload_missing,
                "node_id_vec_id",
                payload_column_count,
                status,
            )
        })
        .collect::<Vec<_>>();

    TableIterator::new(payload_rows.into_iter())
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_tuple_payload_typed(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
    requested_columns: Vec<String>,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(served_epoch, i64),
        name!(node_id, i64),
        name!(pid, i64),
        name!(object_version, i64),
        name!(row_index, i64),
        name!(assignment_flags, i16),
        name!(vec_id, Vec<u8>),
        name!(row_locator, Vec<u8>),
        name!(heap_block, i64),
        name!(heap_offset, i32),
        name!(score, f32),
        name!(payload_attnums, Vec<i16>),
        name!(payload_names, Vec<String>),
        name!(payload_type_oids, Vec<pg_sys::Oid>),
        name!(payload_typmods, Vec<i32>),
        name!(payload_collations, Vec<pg_sys::Oid>),
        name!(payload_nulls, Vec<bool>),
        name!(payload_values, Vec<Vec<u8>>),
        name!(payload_formats, Vec<String>),
        name!(tuple_payload_missing, bool),
        name!(payload_key, &'static str),
        name!(payload_column_count, i32),
        name!(tuple_transport, &'static str),
        name!(tuple_transport_status, &'static str),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_tuple_payload_typed requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_tuple_payload_typed top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_tuple_payload_typed selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_tuple_payload_typed")
    };
    let heap_relation_oid = unsafe {
        (*index_relation)
            .rd_index
            .as_ref()
            .expect("opened index relation should expose pg_index metadata")
            .indrelid
    };
    let rows = unsafe {
        am::spire_remote_search_local_heap_candidate_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let columns = ec_spire_tuple_payload_column_metadata(
        heap_relation_oid,
        &requested_columns,
        "ec_spire_remote_search_tuple_payload_typed",
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    let heap_relation_regclass = ec_spire_relation_regclass_text(heap_relation_oid)
        .unwrap_or_else(|e| pgrx::error!("ec_spire_remote_search_tuple_payload_typed {e}"));
    let payload_column_count = i32::try_from(requested_columns.len()).unwrap_or_else(|_| {
        pgrx::error!("ec_spire_remote_search_tuple_payload_typed too many columns")
    });
    let ctids = rows
        .iter()
        .map(|row| format!("({},{})", row.heap_block, row.heap_offset))
        .collect::<Vec<_>>();
    let payloads = ec_spire_remote_search_typed_tuple_payloads_for_ctids(
        &heap_relation_regclass,
        &columns,
        &ctids,
    )
    .unwrap_or_else(|e| pgrx::error!("ec_spire_remote_search_tuple_payload_typed {e}"));
    let payload_attnums = columns
        .iter()
        .map(|column| column.attnum)
        .collect::<Vec<_>>();
    let payload_names = columns
        .iter()
        .map(|column| column.name.clone())
        .collect::<Vec<_>>();
    let payload_type_oids = columns
        .iter()
        .map(|column| column.type_oid)
        .collect::<Vec<_>>();
    let payload_typmods = columns
        .iter()
        .map(|column| column.typmod)
        .collect::<Vec<_>>();
    let payload_collations = columns
        .iter()
        .map(|column| column.collation)
        .collect::<Vec<_>>();
    let payload_formats = columns
        .iter()
        .map(|_| "pg_binary_attr_v1".to_owned())
        .collect::<Vec<_>>();

    let payload_rows = rows
        .into_iter()
        .zip(payloads)
        .map(
            |(row, (payload_nulls, payload_values, tuple_payload_missing))| {
                let status = if tuple_payload_missing {
                    "remote_tuple_payload_missing"
                } else {
                    row.status
                };
                (
                    i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
                    i64::try_from(row.served_epoch).expect("served epoch should fit in i64"),
                    i64::from(row.node_id),
                    i64::try_from(row.pid).expect("pid should fit in i64"),
                    i64::try_from(row.object_version).expect("object version should fit in i64"),
                    i64::from(row.row_index),
                    i16::try_from(row.assignment_flags)
                        .expect("assignment flags should fit in i16"),
                    row.vec_id,
                    row.row_locator,
                    i64::from(row.heap_block),
                    i32::from(row.heap_offset),
                    row.score,
                    payload_attnums.clone(),
                    payload_names.clone(),
                    payload_type_oids.clone(),
                    payload_typmods.clone(),
                    payload_collations.clone(),
                    payload_nulls,
                    payload_values,
                    payload_formats.clone(),
                    tuple_payload_missing,
                    "node_id_vec_id",
                    payload_column_count,
                    "pg_binary_attr_v1",
                    if tuple_payload_missing {
                        "remote_tuple_payload_missing"
                    } else {
                        "ready"
                    },
                    status,
                )
            },
        )
        .collect::<Vec<_>>();

    TableIterator::new(payload_rows.into_iter())
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_local_heap_candidate_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(local_plan_count, i64),
        name!(remote_plan_count, i64),
        name!(skipped_plan_count, i64),
        name!(local_pid_count, i64),
        name!(remote_pid_count, i64),
        name!(decoded_local_locator_count, i64),
        name!(returned_candidate_count, i64),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_local_heap_candidate_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_local_heap_candidate_summary top_k must be non-negative"
        );
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_local_heap_candidate_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_local_heap_candidate_summary",
        )
    };
    let row = unsafe {
        am::spire_remote_search_local_heap_candidate_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.local_plan_count).expect("local plan count should fit in i64"),
        i64::try_from(row.remote_plan_count).expect("remote plan count should fit in i64"),
        i64::try_from(row.skipped_plan_count).expect("skipped plan count should fit in i64"),
        i64::try_from(row.local_pid_count).expect("local pid count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.decoded_local_locator_count)
            .expect("decoded local locator count should fit in i64"),
        i64::try_from(row.returned_candidate_count)
            .expect("returned candidate count should fit in i64"),
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_coordinator_result_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(local_plan_count, i64),
        name!(remote_plan_count, i64),
        name!(skipped_plan_count, i64),
        name!(local_pid_count, i64),
        name!(remote_pid_count, i64),
        name!(skipped_pid_count, i64),
        name!(decoded_local_locator_count, i64),
        name!(returned_candidate_count, i64),
        name!(result_source, &'static str),
        name!(libpq_receive_count, i64),
        name!(libpq_receive_status, &'static str),
        name!(final_heap_fetch_status, &'static str),
        name!(next_blocker, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_coordinator_result_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!(
            "ec_spire_remote_search_coordinator_result_summary top_k must be non-negative"
        );
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_coordinator_result_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_coordinator_result_summary",
        )
    };
    let row = unsafe {
        am::spire_remote_search_coordinator_result_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.local_plan_count).expect("local plan count should fit in i64"),
        i64::try_from(row.remote_plan_count).expect("remote plan count should fit in i64"),
        i64::try_from(row.skipped_plan_count).expect("skipped plan count should fit in i64"),
        i64::try_from(row.local_pid_count).expect("local pid count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.skipped_pid_count).expect("skipped pid count should fit in i64"),
        i64::try_from(row.decoded_local_locator_count)
            .expect("decoded local locator count should fit in i64"),
        i64::try_from(row.returned_candidate_count)
            .expect("returned candidate count should fit in i64"),
        row.result_source,
        i64::try_from(row.libpq_receive_count).expect("libpq receive count should fit in i64"),
        row.libpq_receive_status,
        row.final_heap_fetch_status,
        row.next_blocker,
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_finalization_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(remote_batch_count, i64),
        name!(local_batch_count, i64),
        name!(skipped_batch_count, i64),
        name!(merge_status, &'static str),
        name!(row_locator_policy, &'static str),
        name!(local_heap_resolution, &'static str),
        name!(remote_heap_resolution, &'static str),
        name!(final_heap_fetch_status, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_finalization_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_finalization_summary top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_finalization_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_finalization_summary")
    };
    let row = unsafe {
        am::spire_remote_search_finalization_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.remote_batch_count).expect("remote batch count should fit in i64"),
        i64::try_from(row.local_batch_count).expect("local batch count should fit in i64"),
        i64::try_from(row.skipped_batch_count).expect("skipped batch count should fit in i64"),
        row.merge_status,
        row.row_locator_policy,
        row.local_heap_resolution,
        row.remote_heap_resolution,
        row.final_heap_fetch_status,
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_coordinator_gate_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(local_plan_count, i64),
        name!(remote_plan_count, i64),
        name!(skipped_plan_count, i64),
        name!(local_pid_count, i64),
        name!(remote_pid_count, i64),
        name!(skipped_pid_count, i64),
        name!(execution_status, &'static str),
        name!(libpq_dispatch_count, i64),
        name!(libpq_dispatch_status, &'static str),
        name!(libpq_executor_status, &'static str),
        name!(libpq_executor_next_step, &'static str),
        name!(libpq_receive_count, i64),
        name!(libpq_receive_status, &'static str),
        name!(merge_status, &'static str),
        name!(final_heap_fetch_status, &'static str),
        name!(next_blocker, &'static str),
        name!(status, &'static str),
        name!(recommendation, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_coordinator_gate_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_coordinator_gate_summary top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_coordinator_gate_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");

    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_coordinator_gate_summary")
    };
    let row = unsafe {
        am::spire_remote_search_coordinator_gate_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.local_plan_count).expect("local plan count should fit in i64"),
        i64::try_from(row.remote_plan_count).expect("remote plan count should fit in i64"),
        i64::try_from(row.skipped_plan_count).expect("skipped plan count should fit in i64"),
        i64::try_from(row.local_pid_count).expect("local pid count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.skipped_pid_count).expect("skipped pid count should fit in i64"),
        row.execution_status,
        i64::try_from(row.libpq_dispatch_count).expect("libpq dispatch count should fit in i64"),
        row.libpq_dispatch_status,
        row.libpq_executor_status,
        row.libpq_executor_next_step,
        i64::try_from(row.libpq_receive_count).expect("libpq receive count should fit in i64"),
        row.libpq_receive_status,
        row.merge_status,
        row.final_heap_fetch_status,
        row.next_blocker,
        row.status,
        row.recommendation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_coordinator_local(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(served_epoch, i64),
        name!(node_id, i64),
        name!(pid, i64),
        name!(object_version, i64),
        name!(row_index, i64),
        name!(assignment_flags, i16),
        name!(vec_id, Vec<u8>),
        name!(row_locator, Vec<u8>),
        name!(score, f32),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_coordinator_local requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_coordinator_local top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_coordinator_local selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");

    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search_coordinator_local") };
    let rows = unsafe {
        am::spire_remote_search_coordinator_local_candidates(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.served_epoch).expect("served epoch should fit in i64"),
            i64::from(row.node_id),
            i64::try_from(row.pid).expect("pid should fit in i64"),
            i64::try_from(row.object_version).expect("object version should fit in i64"),
            i64::from(row.row_index),
            i16::try_from(row.assignment_flags).expect("assignment flags should fit in i16"),
            row.vec_id,
            row.row_locator,
            row.score,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search_coordinator_local_summary(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(requested_epoch, i64),
        name!(local_pid_count, i64),
        name!(remote_target_count, i64),
        name!(remote_pid_count, i64),
        name!(skipped_placement_count, i64),
        name!(candidate_input_count, i64),
        name!(duplicate_vec_id_count, i64),
        name!(returned_candidate_count, i64),
        name!(status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!(
            "ec_spire_remote_search_coordinator_local_summary requested_epoch must be greater than 0"
        );
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search_coordinator_local_summary top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!(
                    "ec_spire_remote_search_coordinator_local_summary selected PID {pid} is negative"
                )
            })
        })
        .collect::<Vec<_>>();
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");

    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_remote_search_coordinator_local_summary",
        )
    };
    let row = unsafe {
        am::spire_remote_search_coordinator_local_summary(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(row.requested_epoch).expect("requested epoch should fit in i64"),
        i64::try_from(row.local_pid_count).expect("local pid count should fit in i64"),
        i64::try_from(row.remote_target_count).expect("remote target count should fit in i64"),
        i64::try_from(row.remote_pid_count).expect("remote pid count should fit in i64"),
        i64::try_from(row.skipped_placement_count)
            .expect("skipped placement count should fit in i64"),
        i64::try_from(row.candidate_input_count).expect("candidate input count should fit in i64"),
        i64::try_from(row.duplicate_vec_id_count)
            .expect("duplicate vec_id count should fit in i64"),
        i64::try_from(row.returned_candidate_count)
            .expect("returned candidate count should fit in i64"),
        row.status,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_remote_search(
    index_oid: pg_sys::Oid,
    requested_epoch: i64,
    query: Vec<f32>,
    selected_pids: Vec<i64>,
    top_k: i32,
    consistency_mode: String,
) -> TableIterator<
    'static,
    (
        name!(served_epoch, i64),
        name!(node_id, i64),
        name!(pid, i64),
        name!(object_version, i64),
        name!(row_index, i64),
        name!(assignment_flags, i16),
        name!(vec_id, Vec<u8>),
        name!(row_locator, Vec<u8>),
        name!(score, f32),
        name!(protocol_version, &'static str),
        name!(extension_version, &'static str),
        name!(opclass_identity, String),
        name!(storage_format, &'static str),
        name!(assignment_payload_format, &'static str),
        name!(quantizer_profile, &'static str),
        name!(scoring_profile, &'static str),
        name!(profile_fingerprint, String),
        name!(endpoint_status, &'static str),
    ),
> {
    if requested_epoch <= 0 {
        pgrx::error!("ec_spire_remote_search requested_epoch must be greater than 0");
    }
    if top_k < 0 {
        pgrx::error!("ec_spire_remote_search top_k must be non-negative");
    }
    let selected_pids = selected_pids
        .into_iter()
        .map(|pid| {
            u64::try_from(pid).unwrap_or_else(|_| {
                pgrx::error!("ec_spire_remote_search selected PID {pid} is negative")
            })
        })
        .collect::<Vec<_>>();
    let top_k = usize::try_from(top_k).expect("non-negative top_k should fit usize");
    let requested_epoch =
        u64::try_from(requested_epoch).expect("positive requested_epoch should fit u64");

    let index_relation = unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_remote_search") };
    let rows = unsafe {
        am::spire_remote_search_candidates(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            &consistency_mode,
        )
    };
    let endpoint_identity =
        unsafe { am::spire_remote_search_endpoint_identity_row(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(move |row| {
        (
            i64::try_from(row.served_epoch).expect("served epoch should fit in i64"),
            i64::from(row.node_id),
            i64::try_from(row.pid).expect("pid should fit in i64"),
            i64::try_from(row.object_version).expect("object version should fit in i64"),
            i64::from(row.row_index),
            i16::try_from(row.assignment_flags).expect("assignment flags should fit in i16"),
            row.vec_id,
            row.row_locator,
            row.score,
            endpoint_identity.protocol_version,
            endpoint_identity.extension_version,
            endpoint_identity.opclass_identity.clone(),
            endpoint_identity.storage_format,
            endpoint_identity.assignment_payload_format,
            endpoint_identity.quantizer_profile,
            endpoint_identity.scoring_profile,
            endpoint_identity.profile_fingerprint.clone(),
            endpoint_identity.status,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_object_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(pid, i64),
        name!(object_kind, String),
        name!(object_version, i64),
        name!(published_epoch_backref, i64),
        name!(level, i32),
        name!(parent_pid, i64),
        name!(child_count, i64),
        name!(assignment_count, i64),
        name!(node_id, i64),
        name!(local_store_id, i64),
        name!(store_relid, i64),
        name!(placement_state, String),
        name!(object_bytes, i64),
        name!(object_readable, bool),
    ),
> {
    if unsafe { !relation_oid_exists(index_oid) } {
        return TableIterator::new(Vec::new().into_iter());
    }
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_object_snapshot") };
    let rows = unsafe { am::spire_index_object_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::try_from(row.pid).expect("pid should fit in i64"),
            row.object_kind.to_owned(),
            i64::try_from(row.object_version).expect("object version should fit in i64"),
            i64::try_from(row.published_epoch_backref)
                .expect("published epoch backref should fit in i64"),
            i32::from(row.level),
            i64::try_from(row.parent_pid).expect("parent pid should fit in i64"),
            i64::try_from(row.child_count).expect("child count should fit in i64"),
            i64::try_from(row.assignment_count).expect("assignment count should fit in i64"),
            i64::from(row.node_id),
            i64::from(row.local_store_id),
            i64::from(row.store_relid),
            row.placement_state.to_owned(),
            i64::try_from(row.object_bytes).expect("object bytes should fit in i64"),
            row.object_readable,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_options_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(nlists, i32),
        name!(recursive_fanout, i32),
        name!(recursive_build_enabled, bool),
        name!(local_store_count, i32),
        name!(local_store_tablespaces, Option<String>),
        name!(boundary_replica_count, i32),
        name!(boundary_replication_enabled, bool),
        name!(scan_dedupe_mode, String),
        name!(active_leaf_count, i64),
        name!(relation_nprobe, i32),
        name!(session_nprobe, Option<i32>),
        name!(effective_nprobe, i64),
        name!(effective_nprobe_source, String),
        name!(effective_nprobe_per_level, Vec<i64>),
        name!(nprobe_policy_per_level, Vec<String>),
        name!(recursive_beam_width, i64),
        name!(max_leaf_routes, i64),
        name!(max_routing_expansions, i64),
        name!(relation_rerank_width, i32),
        name!(session_rerank_width, Option<i32>),
        name!(effective_rerank_width, i32),
        name!(effective_rerank_width_source, String),
        name!(training_sample_rows, i32),
        name!(seed, i32),
        name!(pq_group_size, i32),
        name!(storage_format, String),
        name!(assignment_payload_format, String),
        name!(assignment_payload_scannable, bool),
        name!(assignment_payload_status, String),
        name!(assignment_payload_recommendation, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_options_snapshot") };
    let snapshot = unsafe { am::spire_index_options_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        snapshot.nlists,
        snapshot.recursive_fanout,
        snapshot.recursive_build_enabled,
        snapshot.local_store_count,
        snapshot.local_store_tablespaces,
        snapshot.boundary_replica_count,
        snapshot.boundary_replication_enabled,
        snapshot.scan_dedupe_mode.to_owned(),
        i64::from(snapshot.active_leaf_count),
        snapshot.relation_nprobe,
        snapshot.session_nprobe,
        i64::from(snapshot.effective_nprobe),
        snapshot.effective_nprobe_source.to_owned(),
        snapshot
            .effective_nprobe_per_level
            .into_iter()
            .map(i64::from)
            .collect(),
        snapshot
            .nprobe_policy_per_level
            .into_iter()
            .map(str::to_owned)
            .collect(),
        i64::try_from(snapshot.recursive_beam_width)
            .expect("recursive beam width should fit in i64"),
        i64::try_from(snapshot.max_leaf_routes).expect("max leaf routes should fit in i64"),
        i64::try_from(snapshot.max_routing_expansions)
            .expect("max routing expansions should fit in i64"),
        snapshot.relation_rerank_width,
        snapshot.session_rerank_width,
        snapshot.effective_rerank_width,
        snapshot.effective_rerank_width_source.to_owned(),
        snapshot.training_sample_rows,
        snapshot.seed,
        snapshot.pq_group_size,
        snapshot.storage_format.to_owned(),
        snapshot.assignment_payload_format.to_owned(),
        snapshot.assignment_payload_scannable,
        snapshot.assignment_payload_status.to_owned(),
        snapshot.assignment_payload_recommendation.to_owned(),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_writer_identity_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(source_identity_provider, String),
        name!(writer_identity_status, String),
        name!(writer_identity_recommendation, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_writer_identity_snapshot") };
    let snapshot = unsafe { am::spire_index_writer_identity_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        snapshot.source_identity_provider.to_owned(),
        snapshot.writer_identity_status.to_owned(),
        snapshot.writer_identity_recommendation.to_owned(),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_boundary_replica_identity_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(vec_id, Vec<u8>),
        name!(vec_id_scope, String),
        name!(assignment_count, i64),
        name!(primary_assignment_count, i64),
        name!(boundary_replica_assignment_count, i64),
        name!(delta_insert_assignment_count, i64),
        name!(leaf_pid_count, i64),
        name!(node_count, i64),
        name!(local_store_count, i64),
        name!(min_node_id, i64),
        name!(max_node_id, i64),
        name!(status, String),
        name!(recommendation, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_index_boundary_replica_identity_snapshot",
        )
    };
    let rows = unsafe { am::spire_index_boundary_replica_identity_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            row.vec_id,
            row.vec_id_scope.to_owned(),
            i64::try_from(row.assignment_count).expect("assignment count should fit in i64"),
            i64::try_from(row.primary_assignment_count)
                .expect("primary assignment count should fit in i64"),
            i64::try_from(row.boundary_replica_assignment_count)
                .expect("boundary replica assignment count should fit in i64"),
            i64::try_from(row.delta_insert_assignment_count)
                .expect("delta insert assignment count should fit in i64"),
            i64::try_from(row.leaf_pid_count).expect("leaf pid count should fit in i64"),
            i64::try_from(row.node_count).expect("node count should fit in i64"),
            i64::try_from(row.local_store_count).expect("local store count should fit in i64"),
            i64::from(row.min_node_id),
            i64::from(row.max_node_id),
            row.status.to_owned(),
            row.recommendation.to_owned(),
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_boundary_replica_placement_diagnostics(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(vec_id, Vec<u8>),
        name!(vec_id_scope, String),
        name!(assignment_count, i64),
        name!(primary_assignment_count, i64),
        name!(boundary_replica_assignment_count, i64),
        name!(stale_boundary_replica_count, i64),
        name!(unavailable_boundary_replica_count, i64),
        name!(skipped_boundary_replica_count, i64),
        name!(node_count, i64),
        name!(min_node_id, i64),
        name!(max_node_id, i64),
        name!(status, String),
        name!(degraded_mode_action, String),
        name!(recommendation, String),
    ),
> {
    if unsafe { !relation_oid_exists(index_oid) } {
        return TableIterator::new(Vec::new().into_iter());
    }
    let index_relation = unsafe {
        open_valid_ec_spire_index(
            index_oid,
            "ec_spire_index_boundary_replica_placement_diagnostics",
        )
    };
    let rows = unsafe { am::spire_index_boundary_replica_placement_diagnostics(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            row.vec_id,
            row.vec_id_scope.to_owned(),
            i64::try_from(row.assignment_count).expect("assignment count should fit in i64"),
            i64::try_from(row.primary_assignment_count)
                .expect("primary assignment count should fit in i64"),
            i64::try_from(row.boundary_replica_assignment_count)
                .expect("boundary replica assignment count should fit in i64"),
            i64::try_from(row.stale_boundary_replica_count)
                .expect("stale boundary replica count should fit in i64"),
            i64::try_from(row.unavailable_boundary_replica_count)
                .expect("unavailable boundary replica count should fit in i64"),
            i64::try_from(row.skipped_boundary_replica_count)
                .expect("skipped boundary replica count should fit in i64"),
            i64::try_from(row.node_count).expect("node count should fit in i64"),
            i64::from(row.min_node_id),
            i64::from(row.max_node_id),
            row.status.to_owned(),
            row.degraded_mode_action.to_owned(),
            row.recommendation.to_owned(),
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_level_parameter_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(level, i32),
        name!(routing_object_count, i64),
        name!(routing_child_count, i64),
        name!(target_fanout, i64),
        name!(relation_nprobe, i32),
        name!(session_nprobe, Option<i32>),
        name!(effective_nprobe, i64),
        name!(effective_nprobe_source, String),
        name!(nprobe_policy, String),
        name!(training_sample_rows, i32),
        name!(training_iterations, i64),
        name!(centroid_dimensions, i32),
        name!(distance_operator, String),
        name!(assignment_payload_format, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_level_parameter_snapshot") };
    let rows = unsafe { am::spire_index_level_parameter_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i32::from(row.level),
            i64::try_from(row.routing_object_count)
                .expect("routing object count should fit in i64"),
            i64::try_from(row.routing_child_count).expect("routing child count should fit in i64"),
            i64::from(row.target_fanout),
            row.relation_nprobe,
            row.session_nprobe,
            i64::from(row.effective_nprobe),
            row.effective_nprobe_source.to_owned(),
            row.nprobe_policy.to_owned(),
            row.training_sample_rows,
            i64::try_from(row.training_iterations).expect("training iterations should fit in i64"),
            i32::from(row.centroid_dimensions),
            row.distance_operator.to_owned(),
            row.assignment_payload_format.to_owned(),
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_scan_sanity_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(active_leaf_count, i64),
        name!(effective_nprobe, i64),
        name!(effective_nprobe_source, String),
        name!(exact_leaf_coverage, bool),
        name!(effective_rerank_width, i32),
        name!(effective_rerank_width_source, String),
        name!(full_frontier_rerank, bool),
        name!(recall_sanity_status, String),
        name!(latency_risk_status, String),
        name!(recommendation, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_scan_sanity_snapshot") };
    let snapshot = unsafe { am::spire_index_scan_sanity_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(snapshot.active_epoch).expect("active epoch should fit in i64"),
        i64::from(snapshot.active_leaf_count),
        i64::from(snapshot.effective_nprobe),
        snapshot.effective_nprobe_source.to_owned(),
        snapshot.exact_leaf_coverage,
        snapshot.effective_rerank_width,
        snapshot.effective_rerank_width_source.to_owned(),
        snapshot.full_frontier_rerank,
        snapshot.recall_sanity_status.to_owned(),
        snapshot.latency_risk_status.to_owned(),
        snapshot.recommendation.to_owned(),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_health_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(consistency_mode, String),
        name!(status, String),
        name!(healthy, bool),
        name!(recommendation, String),
        name!(compaction_recommended, bool),
        name!(object_count, i64),
        name!(leaf_assignment_count, i64),
        name!(delta_assignment_count, i64),
        name!(delta_object_count, i64),
        name!(available_placement_count, i64),
        name!(stale_placement_count, i64),
        name!(unavailable_placement_count, i64),
        name!(skipped_placement_count, i64),
    ),
> {
    if unsafe { !relation_oid_exists(index_oid) } {
        return TableIterator::new(Vec::new().into_iter());
    }
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_health_snapshot") };
    let snapshot = unsafe { am::spire_index_health_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(snapshot.active_epoch).expect("active epoch should fit in i64"),
        snapshot.consistency_mode.to_owned(),
        snapshot.status.to_owned(),
        snapshot.healthy,
        snapshot.recommendation.to_owned(),
        snapshot.compaction_recommended,
        i64::try_from(snapshot.object_count).expect("object count should fit in i64"),
        i64::try_from(snapshot.leaf_assignment_count)
            .expect("leaf assignment count should fit in i64"),
        i64::try_from(snapshot.delta_assignment_count)
            .expect("delta assignment count should fit in i64"),
        i64::try_from(snapshot.delta_object_count).expect("delta object count should fit in i64"),
        i64::try_from(snapshot.available_placement_count)
            .expect("available placement count should fit in i64"),
        i64::try_from(snapshot.stale_placement_count)
            .expect("stale placement count should fit in i64"),
        i64::try_from(snapshot.unavailable_placement_count)
            .expect("unavailable placement count should fit in i64"),
        i64::try_from(snapshot.skipped_placement_count)
            .expect("skipped placement count should fit in i64"),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_relation_storage_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(relation_block_count, i64),
        name!(relation_object_tuple_count, i64),
        name!(relation_object_tuple_bytes, i64),
        name!(active_referenced_tuple_count, i64),
        name!(active_referenced_tuple_bytes, i64),
        name!(cleanup_candidate_tuple_count, i64),
        name!(cleanup_candidate_tuple_bytes, i64),
        name!(physical_cleanup_supported, bool),
        name!(recommendation, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_relation_storage_snapshot") };
    let snapshot = unsafe { am::spire_index_relation_storage_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(snapshot.active_epoch).expect("active epoch should fit in i64"),
        i64::try_from(snapshot.relation_block_count).expect("block count should fit in i64"),
        i64::try_from(snapshot.relation_object_tuple_count)
            .expect("object tuple count should fit in i64"),
        i64::try_from(snapshot.relation_object_tuple_bytes)
            .expect("object tuple bytes should fit in i64"),
        i64::try_from(snapshot.active_referenced_tuple_count)
            .expect("active referenced tuple count should fit in i64"),
        i64::try_from(snapshot.active_referenced_tuple_bytes)
            .expect("active referenced tuple bytes should fit in i64"),
        i64::try_from(snapshot.cleanup_candidate_tuple_count)
            .expect("cleanup candidate tuple count should fit in i64"),
        i64::try_from(snapshot.cleanup_candidate_tuple_bytes)
            .expect("cleanup candidate tuple bytes should fit in i64"),
        snapshot.physical_cleanup_supported,
        snapshot.recommendation.to_owned(),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_epoch_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(epoch, i64),
        name!(state, String),
        name!(consistency_mode, String),
        name!(published_at_micros, i64),
        name!(retain_until_micros, i64),
        name!(active_query_count, i64),
        name!(manifest_block, i64),
        name!(manifest_offset, i32),
        name!(is_active_root_manifest, bool),
        name!(cleanup_eligible_now, bool),
        name!(cleanup_blocked_reason, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_epoch_snapshot") };
    let rows = unsafe { am::spire_index_epoch_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::try_from(row.epoch).expect("epoch should fit in i64"),
            row.state.to_owned(),
            row.consistency_mode.to_owned(),
            row.published_at_micros,
            row.retain_until_micros,
            i64::try_from(row.active_query_count).expect("active query count should fit in i64"),
            i64::from(row.manifest_block),
            i32::from(row.manifest_offset),
            row.is_active_root_manifest,
            row.cleanup_eligible_now,
            row.cleanup_blocked_reason.to_owned(),
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_epoch_cleanup_summary(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(epoch_manifest_count, i64),
        name!(active_root_manifest_count, i64),
        name!(retired_epoch_count, i64),
        name!(failed_epoch_count, i64),
        name!(cleanup_eligible_epoch_count, i64),
        name!(retained_retired_epoch_count, i64),
        name!(active_query_blocked_epoch_count, i64),
        name!(retention_window_blocked_epoch_count, i64),
        name!(cleanup_candidate_tuple_count, i64),
        name!(cleanup_candidate_tuple_bytes, i64),
        name!(physical_cleanup_supported, bool),
        name!(physical_cleanup_status, String),
        name!(recommendation, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_epoch_cleanup_summary") };
    let epoch_rows = unsafe { am::spire_index_epoch_snapshot(index_relation) };
    let storage_snapshot = unsafe { am::spire_index_relation_storage_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let active_root_manifest_count = epoch_rows
        .iter()
        .filter(|row| row.is_active_root_manifest)
        .count();
    let retired_epoch_count = epoch_rows
        .iter()
        .filter(|row| row.state == "retired")
        .count();
    let failed_epoch_count = epoch_rows
        .iter()
        .filter(|row| row.state == "failed")
        .count();
    let cleanup_eligible_epoch_count = epoch_rows
        .iter()
        .filter(|row| row.cleanup_eligible_now)
        .count();
    let retained_retired_epoch_count = epoch_rows
        .iter()
        .filter(|row| row.cleanup_blocked_reason == "retained_retired_epoch")
        .count();
    let active_query_blocked_epoch_count = epoch_rows
        .iter()
        .filter(|row| row.cleanup_blocked_reason == "active_queries")
        .count();
    let retention_window_blocked_epoch_count = epoch_rows
        .iter()
        .filter(|row| row.cleanup_blocked_reason == "retention_window")
        .count();
    let physical_cleanup_status = if storage_snapshot.cleanup_candidate_tuple_count == 0 {
        "not_required"
    } else if cleanup_eligible_epoch_count > 0 && storage_snapshot.physical_cleanup_supported {
        "supported"
    } else if storage_snapshot.physical_cleanup_supported {
        "blocked_by_retention"
    } else {
        "blocked_not_implemented"
    };
    let recommendation = match physical_cleanup_status {
        "not_required" => "none",
        "supported" => "run the SPIRE physical cleanup worker when available",
        _ => {
            "cleanup debt is visible, but old-epoch tuple reclamation is not implemented in this build"
        }
    };

    TableIterator::once((
        i64::try_from(storage_snapshot.active_epoch).expect("active epoch should fit in i64"),
        i64::try_from(epoch_rows.len()).expect("epoch manifest count should fit in i64"),
        i64::try_from(active_root_manifest_count)
            .expect("active root manifest count should fit in i64"),
        i64::try_from(retired_epoch_count).expect("retired epoch count should fit in i64"),
        i64::try_from(failed_epoch_count).expect("failed epoch count should fit in i64"),
        i64::try_from(cleanup_eligible_epoch_count)
            .expect("cleanup eligible epoch count should fit in i64"),
        i64::try_from(retained_retired_epoch_count)
            .expect("retained retired epoch count should fit in i64"),
        i64::try_from(active_query_blocked_epoch_count)
            .expect("active query blocked epoch count should fit in i64"),
        i64::try_from(retention_window_blocked_epoch_count)
            .expect("retention window blocked epoch count should fit in i64"),
        i64::try_from(storage_snapshot.cleanup_candidate_tuple_count)
            .expect("cleanup candidate tuple count should fit in i64"),
        i64::try_from(storage_snapshot.cleanup_candidate_tuple_bytes)
            .expect("cleanup candidate tuple bytes should fit in i64"),
        storage_snapshot.physical_cleanup_supported,
        physical_cleanup_status.to_owned(),
        recommendation.to_owned(),
    ))
}

#[pg_extern(volatile, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_epoch_cleanup_run(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(cleanup_epoch_count, i64),
        name!(protected_tuple_count, i64),
        name!(removed_tuple_count, i64),
        name!(removed_tuple_bytes, i64),
        name!(physical_cleanup_supported, bool),
        name!(physical_cleanup_status, String),
        name!(cleanup_message, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_epoch_cleanup_run") };
    let result = unsafe { am::spire_index_epoch_cleanup_run(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(result.active_epoch).expect("active epoch should fit in i64"),
        i64::try_from(result.cleanup_epoch_count).expect("cleanup epoch count should fit in i64"),
        i64::try_from(result.protected_tuple_count)
            .expect("protected tuple count should fit in i64"),
        i64::try_from(result.removed_tuple_count).expect("removed tuple count should fit in i64"),
        i64::try_from(result.removed_tuple_bytes).expect("removed tuple bytes should fit in i64"),
        true,
        result.physical_cleanup_status.to_owned(),
        result.cleanup_message.to_owned(),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_leaf_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(leaf_pid, i64),
        name!(parent_pid, i64),
        name!(object_version, i64),
        name!(node_id, i64),
        name!(local_store_id, i64),
        name!(placement_state, String),
        name!(base_assignment_count, i64),
        name!(base_primary_assignment_count, i64),
        name!(base_boundary_replica_assignment_count, i64),
        name!(delta_object_count, i64),
        name!(delta_insert_assignment_count, i64),
        name!(delta_boundary_replica_insert_assignment_count, i64),
        name!(delta_delete_assignment_count, i64),
        name!(effective_assignment_count, i64),
        name!(effective_boundary_replica_assignment_count, i64),
        name!(split_assignment_threshold, i64),
        name!(merge_assignment_threshold, i64),
        name!(split_recommended, bool),
        name!(merge_recommended, bool),
        name!(maintenance_action, String),
        name!(maintenance_reason, String),
        name!(leaf_object_bytes, i64),
        name!(delta_object_bytes, i64),
    ),
> {
    if unsafe { !relation_oid_exists(index_oid) } {
        return TableIterator::new(Vec::new().into_iter());
    }
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_leaf_snapshot") };
    let rows = unsafe { am::spire_index_leaf_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::try_from(row.leaf_pid).expect("leaf pid should fit in i64"),
            i64::try_from(row.parent_pid).expect("parent pid should fit in i64"),
            i64::try_from(row.object_version).expect("object version should fit in i64"),
            i64::from(row.node_id),
            i64::from(row.local_store_id),
            row.placement_state.to_owned(),
            i64::try_from(row.base_assignment_count)
                .expect("base assignment count should fit in i64"),
            i64::try_from(row.base_primary_assignment_count)
                .expect("base primary assignment count should fit in i64"),
            i64::try_from(row.base_boundary_replica_assignment_count)
                .expect("base boundary replica assignment count should fit in i64"),
            i64::try_from(row.delta_object_count).expect("delta object count should fit in i64"),
            i64::try_from(row.delta_insert_assignment_count)
                .expect("delta insert assignment count should fit in i64"),
            i64::try_from(row.delta_boundary_replica_insert_assignment_count)
                .expect("delta boundary replica insert assignment count should fit in i64"),
            i64::try_from(row.delta_delete_assignment_count)
                .expect("delta delete assignment count should fit in i64"),
            i64::try_from(row.effective_assignment_count)
                .expect("effective assignment count should fit in i64"),
            i64::try_from(row.effective_boundary_replica_assignment_count)
                .expect("effective boundary replica assignment count should fit in i64"),
            i64::try_from(row.split_assignment_threshold)
                .expect("split assignment threshold should fit in i64"),
            i64::try_from(row.merge_assignment_threshold)
                .expect("merge assignment threshold should fit in i64"),
            row.split_recommended,
            row.merge_recommended,
            row.maintenance_action.to_owned(),
            row.maintenance_reason.to_owned(),
            i64::try_from(row.leaf_object_bytes).expect("leaf object bytes should fit in i64"),
            i64::try_from(row.delta_object_bytes).expect("delta object bytes should fit in i64"),
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_maintenance_plan_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(planner_status, String),
        name!(planned_action, String),
        name!(planned_reason, String),
        name!(replaced_parent_pid, i64),
        name!(affected_leaf_pids, String),
        name!(replacement_leaf_count, i64),
        name!(replacement_leaf_pids, String),
        name!(publish_epoch, i64),
        name!(next_pid, i64),
        name!(next_local_vec_seq, i64),
        name!(planner_message, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_maintenance_plan_snapshot") };
    let snapshot = unsafe { am::spire_index_maintenance_plan_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(snapshot.active_epoch).expect("active epoch should fit in i64"),
        snapshot.planner_status.to_owned(),
        snapshot.planned_action.to_owned(),
        snapshot.planned_reason.to_owned(),
        i64::try_from(snapshot.replaced_parent_pid).expect("parent pid should fit in i64"),
        format_u64_array_text(&snapshot.affected_leaf_pids),
        i64::try_from(snapshot.replacement_leaf_count)
            .expect("replacement leaf count should fit in i64"),
        format_u64_array_text(&snapshot.replacement_leaf_pids),
        i64::try_from(snapshot.publish_epoch).expect("publish epoch should fit in i64"),
        i64::try_from(snapshot.next_pid).expect("next pid should fit in i64"),
        i64::try_from(snapshot.next_local_vec_seq).expect("next local vec seq should fit in i64"),
        snapshot.planner_message.to_owned(),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_locked_maintenance_plan_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(planner_status, String),
        name!(planned_action, String),
        name!(planned_reason, String),
        name!(replaced_parent_pid, i64),
        name!(affected_leaf_pids, String),
        name!(replacement_leaf_count, i64),
        name!(replacement_leaf_pids, String),
        name!(publish_epoch, i64),
        name!(next_pid, i64),
        name!(next_local_vec_seq, i64),
        name!(planner_message, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_index_locked_maintenance_plan_snapshot")
    };
    let snapshot = unsafe { am::spire_index_locked_maintenance_plan_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(snapshot.active_epoch).expect("active epoch should fit in i64"),
        snapshot.planner_status.to_owned(),
        snapshot.planned_action.to_owned(),
        snapshot.planned_reason.to_owned(),
        i64::try_from(snapshot.replaced_parent_pid).expect("parent pid should fit in i64"),
        format_u64_array_text(&snapshot.affected_leaf_pids),
        i64::try_from(snapshot.replacement_leaf_count)
            .expect("replacement leaf count should fit in i64"),
        format_u64_array_text(&snapshot.replacement_leaf_pids),
        i64::try_from(snapshot.publish_epoch).expect("publish epoch should fit in i64"),
        i64::try_from(snapshot.next_pid).expect("next pid should fit in i64"),
        i64::try_from(snapshot.next_local_vec_seq).expect("next local vec seq should fit in i64"),
        snapshot.planner_message.to_owned(),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_locked_maintenance_run_plan(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch_before, i64),
        name!(active_epoch_after, i64),
        name!(maintenance_status, String),
        name!(planned_action, String),
        name!(planned_reason, String),
        name!(replaced_parent_pid, i64),
        name!(affected_leaf_pids, String),
        name!(replacement_leaf_count, i64),
        name!(replacement_leaf_pids, String),
        name!(publish_epoch, i64),
        name!(next_pid, i64),
        name!(next_local_vec_seq, i64),
        name!(published, bool),
        name!(maintenance_message, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_index_locked_maintenance_run_plan")
    };
    let result = unsafe { am::spire_index_locked_maintenance_run_plan(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(result.active_epoch_before).expect("active epoch should fit in i64"),
        i64::try_from(result.active_epoch_after).expect("active epoch should fit in i64"),
        result.maintenance_status.to_owned(),
        result.planned_action.to_owned(),
        result.planned_reason.to_owned(),
        i64::try_from(result.replaced_parent_pid).expect("parent pid should fit in i64"),
        format_u64_array_text(&result.affected_leaf_pids),
        i64::try_from(result.replacement_leaf_count)
            .expect("replacement leaf count should fit in i64"),
        format_u64_array_text(&result.replacement_leaf_pids),
        i64::try_from(result.publish_epoch).expect("publish epoch should fit in i64"),
        i64::try_from(result.next_pid).expect("next pid should fit in i64"),
        i64::try_from(result.next_local_vec_seq).expect("next local vec seq should fit in i64"),
        result.published,
        result.maintenance_message.to_owned(),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_maintenance_scheduler_plan(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(scheduler_policy, String),
        name!(scheduler_status, String),
        name!(plan_entrypoint, String),
        name!(run_entrypoint, String),
        name!(publish_lock_mode, String),
        name!(lock_time_recheck, bool),
        name!(planner_status, String),
        name!(planned_action, String),
        name!(planned_reason, String),
        name!(publish_epoch, i64),
        name!(next_pid, i64),
        name!(next_local_vec_seq, i64),
        name!(recommendation, String),
    ),
> {
    let index_relation = unsafe {
        open_valid_ec_spire_index(index_oid, "ec_spire_index_maintenance_scheduler_plan")
    };
    let snapshot = unsafe { am::spire_index_locked_maintenance_plan_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let (scheduler_status, recommendation) = if snapshot.planner_status == "planned" {
        (
            "due",
            "call ec_spire_index_maintenance_scheduler_run(index_oid) from an operator-controlled periodic job",
        )
    } else {
        ("idle", "none")
    };

    TableIterator::once((
        i64::try_from(snapshot.active_epoch).expect("active epoch should fit in i64"),
        "operator_periodic_job".to_owned(),
        scheduler_status.to_owned(),
        "ec_spire_index_locked_maintenance_run_plan".to_owned(),
        "ec_spire_index_maintenance_scheduler_run".to_owned(),
        "ShareUpdateExclusiveLock".to_owned(),
        true,
        snapshot.planner_status.to_owned(),
        snapshot.planned_action.to_owned(),
        snapshot.planned_reason.to_owned(),
        i64::try_from(snapshot.publish_epoch).expect("publish epoch should fit in i64"),
        i64::try_from(snapshot.next_pid).expect("next pid should fit in i64"),
        i64::try_from(snapshot.next_local_vec_seq).expect("next local vec seq should fit in i64"),
        recommendation.to_owned(),
    ))
}

#[pg_extern(volatile, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_maintenance_run(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch_before, i64),
        name!(active_epoch_after, i64),
        name!(maintenance_status, String),
        name!(planned_action, String),
        name!(planned_reason, String),
        name!(replaced_parent_pid, i64),
        name!(affected_leaf_pids, String),
        name!(replacement_leaf_count, i64),
        name!(replacement_leaf_pids, String),
        name!(publish_epoch, i64),
        name!(next_pid, i64),
        name!(next_local_vec_seq, i64),
        name!(published, bool),
        name!(maintenance_message, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_maintenance_run") };
    let result = unsafe { am::spire_index_maintenance_run(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(result.active_epoch_before).expect("active epoch should fit in i64"),
        i64::try_from(result.active_epoch_after).expect("active epoch should fit in i64"),
        result.maintenance_status.to_owned(),
        result.planned_action.to_owned(),
        result.planned_reason.to_owned(),
        i64::try_from(result.replaced_parent_pid).expect("parent pid should fit in i64"),
        format_u64_array_text(&result.affected_leaf_pids),
        i64::try_from(result.replacement_leaf_count)
            .expect("replacement leaf count should fit in i64"),
        format_u64_array_text(&result.replacement_leaf_pids),
        i64::try_from(result.publish_epoch).expect("publish epoch should fit in i64"),
        i64::try_from(result.next_pid).expect("next pid should fit in i64"),
        i64::try_from(result.next_local_vec_seq).expect("next local vec seq should fit in i64"),
        result.published,
        result.maintenance_message.to_owned(),
    ))
}

#[pg_extern(volatile, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_maintenance_scheduler_run(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(scheduler_policy, String),
        name!(scheduler_status, String),
        name!(run_entrypoint, String),
        name!(publish_lock_mode, String),
        name!(lock_time_recheck, bool),
        name!(active_epoch_before, i64),
        name!(active_epoch_after, i64),
        name!(maintenance_status, String),
        name!(planned_action, String),
        name!(planned_reason, String),
        name!(replaced_parent_pid, i64),
        name!(affected_leaf_pids, String),
        name!(replacement_leaf_count, i64),
        name!(replacement_leaf_pids, String),
        name!(publish_epoch, i64),
        name!(next_pid, i64),
        name!(next_local_vec_seq, i64),
        name!(published, bool),
        name!(maintenance_message, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_maintenance_scheduler_run") };
    let result = unsafe { am::spire_index_maintenance_run(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    let scheduler_status = if result.published { "ran" } else { "idle" };

    TableIterator::once((
        "operator_periodic_job".to_owned(),
        scheduler_status.to_owned(),
        "ec_spire_index_maintenance_run".to_owned(),
        "ShareUpdateExclusiveLock".to_owned(),
        true,
        i64::try_from(result.active_epoch_before).expect("active epoch should fit in i64"),
        i64::try_from(result.active_epoch_after).expect("active epoch should fit in i64"),
        result.maintenance_status.to_owned(),
        result.planned_action.to_owned(),
        result.planned_reason.to_owned(),
        i64::try_from(result.replaced_parent_pid).expect("parent pid should fit in i64"),
        format_u64_array_text(&result.affected_leaf_pids),
        i64::try_from(result.replacement_leaf_count)
            .expect("replacement leaf count should fit in i64"),
        format_u64_array_text(&result.replacement_leaf_pids),
        i64::try_from(result.publish_epoch).expect("publish epoch should fit in i64"),
        i64::try_from(result.next_pid).expect("next pid should fit in i64"),
        i64::try_from(result.next_local_vec_seq).expect("next local vec seq should fit in i64"),
        result.published,
        result.maintenance_message.to_owned(),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_delta_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(delta_pid, i64),
        name!(parent_leaf_pid, i64),
        name!(object_version, i64),
        name!(published_epoch_backref, i64),
        name!(node_id, i64),
        name!(local_store_id, i64),
        name!(store_relid, i64),
        name!(placement_state, String),
        name!(assignment_count, i64),
        name!(insert_assignment_count, i64),
        name!(delete_assignment_count, i64),
        name!(object_bytes, i64),
    ),
> {
    if unsafe { !relation_oid_exists(index_oid) } {
        return TableIterator::new(Vec::new().into_iter());
    }
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_delta_snapshot") };
    let rows = unsafe { am::spire_index_delta_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::try_from(row.active_epoch).expect("active epoch should fit in i64"),
            i64::try_from(row.delta_pid).expect("delta pid should fit in i64"),
            i64::try_from(row.parent_leaf_pid).expect("parent leaf pid should fit in i64"),
            i64::try_from(row.object_version).expect("object version should fit in i64"),
            i64::try_from(row.published_epoch_backref)
                .expect("published epoch backref should fit in i64"),
            i64::from(row.node_id),
            i64::from(row.local_store_id),
            i64::from(row.store_relid),
            row.placement_state.to_owned(),
            i64::try_from(row.assignment_count).expect("assignment count should fit in i64"),
            i64::try_from(row.insert_assignment_count)
                .expect("insert assignment count should fit in i64"),
            i64::try_from(row.delete_assignment_count)
                .expect("delete assignment count should fit in i64"),
            i64::try_from(row.object_bytes).expect("object bytes should fit in i64"),
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_insert_debt_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(active_epoch, i64),
        name!(active_leaf_count, i64),
        name!(leaf_count_with_deltas, i64),
        name!(delta_object_count, i64),
        name!(delta_insert_assignment_count, i64),
        name!(max_delta_objects_per_leaf, i64),
        name!(insert_batching_supported, bool),
        name!(batching_recommended, bool),
        name!(recommendation, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_insert_debt_snapshot") };
    let snapshot = unsafe { am::spire_index_insert_debt_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        i64::try_from(snapshot.active_epoch).expect("active epoch should fit in i64"),
        i64::try_from(snapshot.active_leaf_count).expect("active leaf count should fit in i64"),
        i64::try_from(snapshot.leaf_count_with_deltas)
            .expect("leaf count with deltas should fit in i64"),
        i64::try_from(snapshot.delta_object_count).expect("delta object count should fit in i64"),
        i64::try_from(snapshot.delta_insert_assignment_count)
            .expect("delta insert assignment count should fit in i64"),
        i64::try_from(snapshot.max_delta_objects_per_leaf)
            .expect("max delta objects per leaf should fit in i64"),
        snapshot.insert_batching_supported,
        snapshot.batching_recommended,
        snapshot.recommendation.to_owned(),
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_ivf_index_page_ownership(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(block_number, i64),
        name!(line_pointer_count, i32),
        name!(unused_line_pointers, i32),
        name!(non_posting_tuples, i32),
        name!(posting_tuples, i32),
        name!(live_posting_tuples, i32),
        name!(deleted_posting_tuples, i32),
        name!(heap_tid_refs, i64),
        name!(distinct_lists, i32),
        name!(min_list_id, Option<i32>),
        name!(max_list_id, Option<i32>),
        name!(list_ids, String),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_ivf_index(index_oid, "ec_ivf_index_page_ownership") };
    let rows = unsafe { am::ivf_index_page_ownership(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::new(rows.into_iter().map(|row| {
        (
            i64::from(row.block_number),
            i32::from(row.line_pointer_count),
            i32::from(row.unused_line_pointers),
            i32::from(row.non_posting_tuples),
            i32::from(row.posting_tuples),
            i32::from(row.live_posting_tuples),
            i32::from(row.deleted_posting_tuples),
            i64::from(row.heap_tid_refs),
            i32::try_from(row.distinct_lists).expect("distinct-list count should fit in i32"),
            row.min_list_id
                .map(|value| i32::try_from(value).expect("list id should fit in i32")),
            row.max_list_id
                .map(|value| i32::try_from(value).expect("list id should fit in i32")),
            row.list_ids,
        )
    }))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_cost_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(planner_scan_enabled, bool),
        name!(planner_gate_reason, String),
        name!(dimensions, i32),
        name!(nlists, i64),
        name!(active_leaf_count, i64),
        name!(relation_nprobe, i64),
        name!(session_nprobe, Option<i32>),
        name!(effective_nprobe, i64),
        name!(effective_nprobe_source, String),
        name!(local_store_count, i32),
        name!(recursive_fanout, Option<i32>),
        name!(resolved_tree_height, f64),
        name!(tree_height_source, String),
        name!(pg18_tree_height_callback_ready, bool),
        name!(average_leaf_live_count, f64),
        name!(estimated_routing_scores, i64),
        name!(estimated_selected_leaves, i64),
        name!(estimated_candidate_rows, f64),
        name!(estimated_routing_pages, f64),
        name!(estimated_leaf_pages, f64),
        name!(storage_format, String),
        name!(relation_rerank_width, i32),
        name!(session_rerank_width, Option<i32>),
        name!(effective_rerank_width, i32),
        name!(effective_rerank_width_source, String),
        name!(index_pages, f64),
        name!(reltuples, f64),
        name!(modeled_startup_cost, f64),
        name!(modeled_total_cost, f64),
        name!(modeled_selectivity, f64),
        name!(modeled_correlation, f64),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_cost_snapshot") };
    let snapshot = unsafe { am::spire_index_cost_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        snapshot.planner_scan_enabled,
        snapshot.planner_gate_reason.to_owned(),
        i32::from(snapshot.dimensions),
        i64::from(snapshot.nlists),
        i64::from(snapshot.active_leaf_count),
        i64::from(snapshot.relation_nprobe),
        snapshot
            .session_nprobe
            .map(|value| i32::try_from(value).expect("session nprobe should fit in i32")),
        i64::from(snapshot.effective_nprobe),
        snapshot.effective_nprobe_source.to_owned(),
        i32::try_from(snapshot.local_store_count).expect("local store count should fit in i32"),
        snapshot
            .recursive_fanout
            .map(|value| i32::try_from(value).expect("recursive fanout should fit in i32")),
        snapshot.resolved_tree_height,
        snapshot.tree_height_source.to_owned(),
        snapshot.pg18_tree_height_callback_ready,
        snapshot.average_leaf_live_count,
        i64::try_from(snapshot.estimated_routing_scores)
            .expect("routing score estimate should fit in i64"),
        i64::from(snapshot.estimated_selected_leaves),
        snapshot.estimated_candidate_rows,
        snapshot.estimated_routing_pages,
        snapshot.estimated_leaf_pages,
        snapshot.storage_format.to_owned(),
        snapshot.relation_rerank_width,
        snapshot.session_rerank_width,
        snapshot.effective_rerank_width,
        snapshot.effective_rerank_width_source.to_owned(),
        snapshot.index_pages,
        snapshot.reltuples,
        snapshot.modeled_startup_cost,
        snapshot.modeled_total_cost,
        snapshot.modeled_selectivity,
        snapshot.modeled_correlation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_spire_index_cost_tuning_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(storage_format, String),
        name!(effective_rerank_width, i32),
        name!(cost_routing_dimension_scale, f64),
        name!(cost_leaf_dimension_scale, f64),
        name!(cost_index_page_scale, f64),
        name!(cost_local_store_page_fanout_scale, f64),
        name!(cost_storage_scoring_multiplier, f64),
        name!(effective_storage_scoring_multiplier, f64),
        name!(cost_rerank_multiplier, f64),
        name!(effective_rerank_multiplier, f64),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_spire_index(index_oid, "ec_spire_index_cost_tuning_snapshot") };
    let snapshot = unsafe { am::spire_index_cost_tuning_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        snapshot.storage_format.to_owned(),
        snapshot.effective_rerank_width,
        snapshot.cost_routing_dimension_scale,
        snapshot.cost_leaf_dimension_scale,
        snapshot.cost_index_page_scale,
        snapshot.cost_local_store_page_fanout_scale,
        snapshot.cost_storage_scoring_multiplier,
        snapshot.effective_storage_scoring_multiplier,
        snapshot.cost_rerank_multiplier,
        snapshot.effective_rerank_multiplier,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_ivf_index_cost_snapshot(
    index_oid: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(planner_scan_enabled, bool),
        name!(planner_gate_reason, String),
        name!(dimensions, i32),
        name!(nlists, i64),
        name!(relation_nprobe, i64),
        name!(session_nprobe, Option<i32>),
        name!(effective_nprobe, i64),
        name!(effective_nprobe_source, String),
        name!(resolved_tree_height, f64),
        name!(tree_height_source, String),
        name!(pg18_tree_height_callback_ready, bool),
        name!(ordering_compare_type, String),
        name!(pg18_strategy_translation_ready, bool),
        name!(average_list_live_count, f64),
        name!(estimated_centroid_scores, i64),
        name!(estimated_selected_lists, i64),
        name!(estimated_candidate_rows, f64),
        name!(estimated_posting_pages, f64),
        name!(storage_format, String),
        name!(scoring_mode, String),
        name!(scoring_multiplier, f64),
        name!(rerank, String),
        name!(rerank_multiplier, f64),
        name!(index_pages, f64),
        name!(reltuples, f64),
        name!(random_page_cost, f64),
        name!(seq_page_cost, f64),
        name!(cpu_operator_cost, f64),
        name!(modeled_startup_cost, f64),
        name!(modeled_total_cost, f64),
        name!(modeled_selectivity, f64),
        name!(modeled_correlation, f64),
    ),
> {
    let index_relation =
        unsafe { open_valid_ec_ivf_index(index_oid, "ec_ivf_index_cost_snapshot") };
    let snapshot = unsafe { am::ivf_index_cost_snapshot(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    TableIterator::once((
        snapshot.planner_scan_enabled,
        snapshot.planner_gate_reason.to_owned(),
        i32::from(snapshot.dimensions),
        i64::from(snapshot.nlists),
        i64::from(snapshot.relation_nprobe),
        snapshot
            .session_nprobe
            .map(|value| i32::try_from(value).expect("session nprobe should fit in i32")),
        i64::from(snapshot.effective_nprobe),
        snapshot.effective_nprobe_source.to_owned(),
        snapshot.resolved_tree_height,
        snapshot.tree_height_source.to_owned(),
        snapshot.pg18_tree_height_callback_ready,
        snapshot.ordering_compare_type.to_owned(),
        snapshot.pg18_strategy_translation_ready,
        snapshot.average_list_live_count,
        i64::from(snapshot.estimated_centroid_scores),
        i64::from(snapshot.estimated_selected_lists),
        snapshot.estimated_candidate_rows,
        snapshot.estimated_posting_pages,
        snapshot.storage_format.to_owned(),
        snapshot.scoring_mode.to_owned(),
        snapshot.scoring_multiplier,
        snapshot.rerank.to_owned(),
        snapshot.rerank_multiplier,
        snapshot.index_pages,
        snapshot.reltuples,
        snapshot.random_page_cost,
        snapshot.seq_page_cost,
        snapshot.cpu_operator_cost,
        snapshot.modeled_startup_cost,
        snapshot.modeled_total_cost,
        snapshot.modeled_selectivity,
        snapshot.modeled_correlation,
    ))
}

#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_hnsw_index_cost_snapshot(
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
        unsafe { open_valid_ec_hnsw_index(index_oid, "ec_hnsw_index_cost_snapshot") };
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
fn ec_hnsw_planner_integration_snapshot(
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
        unsafe { open_valid_ec_hnsw_index(index_oid, "ec_hnsw_planner_integration_snapshot") };
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

#[cfg(feature = "pg18")]
#[pg_extern(stable)]
#[allow(clippy::type_complexity)]
fn ecaz_stats() -> TableIterator<
    'static,
    (
        name!(total_distance_calcs, i64),
        name!(total_graph_hops, i64),
        name!(total_linear_pages, i64),
        name!(total_scans_started, i64),
        name!(total_scans_bootstrap_only, i64),
        name!(quantizer_cache_hits, i64),
        name!(quantizer_cache_misses, i64),
        name!(bootstrap_hit_rate, f64),
        name!(quantizer_cache_rate, f64),
    ),
> {
    let summary = am::stats::current_stats_counters().summary();
    TableIterator::once((
        i64::try_from(summary.total_distance_calcs)
            .expect("distance calc counter should fit in i64"),
        i64::try_from(summary.total_graph_hops).expect("graph hop counter should fit in i64"),
        i64::try_from(summary.total_linear_pages).expect("linear page counter should fit in i64"),
        i64::try_from(summary.total_scans_started).expect("scan counter should fit in i64"),
        i64::try_from(summary.total_scans_bootstrap_only)
            .expect("bootstrap-only counter should fit in i64"),
        i64::try_from(summary.quantizer_cache_hits)
            .expect("quantizer cache-hit counter should fit in i64"),
        i64::try_from(summary.quantizer_cache_misses)
            .expect("quantizer cache-miss counter should fit in i64"),
        summary.bootstrap_hit_rate,
        summary.quantizer_cache_rate,
    ))
}

fn encode_embedding_to_tqvector(
    embedding: Vec<f32>,
    bits: i32,
    seed: i64,
) -> Result<Vec<u8>, String> {
    let bits = u8::try_from(bits).map_err(|_| "bits must fit into u8".to_string())?;
    let seed = u64::try_from(seed).map_err(|_| "seed must fit into u64".to_string())?;
    validate_tqvector_bits(bits)?;
    validate_tqvector_seed(seed)?;
    let (dim, gamma, code_bytes) = quantize_embedding_to_code(&embedding, bits, seed)?;
    Ok(pack(dim, bits, seed, gamma, &code_bytes))
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn encode_to_tqvector(embedding: Vec<f32>, bits: i32, seed: i64) -> Vec<u8> {
    encode_embedding_to_tqvector(embedding, bits, seed).unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn encode_to_ecvector(embedding: Vec<f32>, bits: i32, seed: i64) -> Vec<u8> {
    let bits = u8::try_from(bits).map_err(|_| "bits must fit into u8".to_string());
    let seed = u64::try_from(seed).map_err(|_| "seed must fit into u64".to_string());
    let bits = bits.unwrap_or_else(|e| pgrx::error!("{e}"));
    let seed = seed.unwrap_or_else(|e| pgrx::error!("{e}"));
    if bits != DEFAULT_QUANT_BITS || seed != DEFAULT_QUANT_SEED {
        pgrx::error!(
            "encode_to_ecvector expects the canonical quantizer defaults ({DEFAULT_QUANT_BITS},{DEFAULT_QUANT_SEED}), got ({bits},{seed})"
        );
    }
    pack_raw_f32(&embedding, "ecvector").unwrap_or_else(|e| pgrx::error!("invalid ecvector: {e}"))
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

fn score_ecvector_inner_product(left: &[u8], right: &[u8]) -> Result<f32, String> {
    let left = unpack_raw_f32(left, "ecvector(left)")?;
    let right = unpack_raw_f32(right, "ecvector(right)")?;
    raw_inner_product(&left, &right, "ecvector")
}

fn score_ecvector_negative_query_inner_product(
    candidate: &[u8],
    query: &[f32],
) -> Result<f32, String> {
    let candidate = unpack_raw_f32(candidate, "ecvector(candidate)")?;
    Ok(-raw_inner_product(&candidate, query, "ecvector/query")?)
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn ecvector_inner_product(left: Vec<u8>, right: Vec<u8>) -> f32 {
    score_ecvector_inner_product(&left, &right).unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn ecvector_negative_inner_product(left: Vec<u8>, right: Vec<u8>) -> f32 {
    -ecvector_inner_product(left, right)
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn ecvector_query_inner_product(candidate: Vec<u8>, query: Vec<f32>) -> f32 {
    -ecvector_negative_query_inner_product(candidate, query)
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn ecvector_negative_query_inner_product(candidate: Vec<u8>, query: Vec<f32>) -> f32 {
    score_ecvector_negative_query_inner_product(&candidate, &query)
        .unwrap_or_else(|e| pgrx::error!("{e}"))
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
        assert!(unpack(&[0_u8; 5]).is_err());
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
        vec!["max_prepared_transactions = 10"]
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    include!("tests/mod.rs");
}
