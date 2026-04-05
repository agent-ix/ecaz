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

pub(crate) fn score_code_inner_product(
    dim: usize,
    bits: u8,
    seed: u64,
    code_a: &[u8],
    code_b: &[u8],
) -> f32 {
    let quantizer = ProdQuantizer::cached(dim, bits, seed);
    let mut payload_a = Vec::with_capacity(MIN_BINARY_BYTES + code_a.len());
    payload_a.extend_from_slice(&0.0_f32.to_le_bytes());
    payload_a.extend_from_slice(code_a);

    let mut payload_b = Vec::with_capacity(MIN_BINARY_BYTES + code_b.len());
    payload_b.extend_from_slice(&0.0_f32.to_le_bytes());
    payload_b.extend_from_slice(code_b);

    quantizer.score_ip_encoded_lite(&payload_a, &payload_b)
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

    score_code_inner_product(dim_a as usize, bits_a, seed_a, codes_a, codes_b)
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
        assert_eq!(metadata.seed, 0);
    }

    #[pg_test]
    fn test_non_empty_index_build_writes_minimal_data_pages() {
        Spi::run(
            "CREATE TABLE tqhnsw_nonempty_build (id bigint primary key, embedding tqvector)",
        )
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

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_nonempty_build_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let (block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };

        assert!(block_count >= 2, "non-empty build should allocate a data page");
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

        assert_eq!(page_tuples.len(), 6, "each heap row should emit one neighbor and one element tuple");

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
                    Some(
                        (
                            *tid,
                            am::page::TqElementTuple::decode(tuple, code_len(4, 4))
                                .expect("element tuple should decode"),
                        ),
                    )
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
            assert!(
                neighbor.tids.len()
                    <= am::page::neighbor_slots(element.level, metadata.m)
            );
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

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_source_build_idx'::regclass::oid",
        )
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
                page.tuples.iter().enumerate().filter_map(move |(idx, tuple)| {
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
        assert!(elements.iter().all(|(_, element)| element.heaptids.len() == 1));
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

        assert_eq!(elements.len(), 2, "duplicate encoded vectors should share one element tuple");
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
        Spi::run(
            "CREATE TABLE tqhnsw_multipage_build (id bigint primary key, embedding tqvector)",
        )
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

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_multipage_build_idx'::regclass::oid",
        )
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
            data_pages.iter().filter(|page| !page.tuples.is_empty()).count() > 1,
            "more than one populated data page should exist"
        );

        let neighbor_map = neighbors.into_iter().collect::<std::collections::HashMap<_, _>>();
        for (_, element) in &elements {
            let neighbor = neighbor_map
                .get(&element.neighbortid)
                .expect("neighbor tuple should exist");
            assert_eq!(neighbor.count as usize, neighbor.tids.len());
            assert!(
                neighbor.tids.len()
                    <= am::page::neighbor_slots(element.level, metadata.m)
            );
            assert!(neighbor.tids.iter().all(|tid| element_tids.contains(tid)));
        }
    }

    #[pg_test]
    fn test_non_empty_index_build_coalesces_duplicate_vectors() {
        Spi::run(
            "CREATE TABLE tqhnsw_duplicate_build (id bigint primary key, embedding tqvector)",
        )
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

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_duplicate_build_idx'::regclass::oid",
        )
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

        assert_eq!(elements.len(), 2, "duplicate encoded vectors should share one element tuple");
        let mut heaptid_counts = elements
            .iter()
            .map(|element| element.heaptids.len())
            .collect::<Vec<_>>();
        heaptid_counts.sort_unstable();
        assert_eq!(heaptid_counts, vec![1, 2]);
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

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_insert_append_idx'::regclass::oid",
        )
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
        assert!(elements.iter().all(|element| element.level == 0 || element.level <= metadata.max_level));
        assert!(elements.iter().any(|element| element.heaptids.len() == 1));
    }

    #[pg_test]
    fn test_tqhnsw_insert_reuses_tail_page_when_space_remains() {
        Spi::run("CREATE TABLE tqhnsw_insert_tail_reuse (id bigint primary key, embedding tqvector)")
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

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_insert_tail_reuse_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (before_block_count, _metadata, _data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(before_block_count, 2, "seed build should fit on one data page");

        Spi::run(
            "INSERT INTO tqhnsw_insert_tail_reuse VALUES
             (4, encode_to_tqvector(ARRAY[0.5, -0.5, 0.1, 0.2], 4, 42))",
        )
        .expect("insert should succeed");

        let (after_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(after_block_count, before_block_count, "insert should reuse existing tail page");
        assert_eq!(metadata.seed, 42);

        let tuple_count = data_pages.iter().map(|page| page.tuples.len()).sum::<usize>();
        assert_eq!(tuple_count, 8, "three build tuples plus one inserted tuple should store four pairs");
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

                let mut staged_page =
                    am::page::DataPage::new(am::page::FIRST_DATA_BLOCK_NUMBER, pg_sys::BLCKSZ as usize);
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
                    neighbortid: am::page::ItemPointer::INVALID,
                    code: vec![0x11_u8; code_len],
                };
                let required_bytes =
                    am::page::raw_tuple_storage_bytes(neighbor.encode().expect("neighbor tuple should encode").len())
                        + am::page::raw_tuple_storage_bytes(
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

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_insert_new_page_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (before_block_count, metadata, _data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(before_block_count, 2, "seed build should occupy one data page");
        assert_eq!(metadata.dimensions, large_dim);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);

        Spi::run(&format!(
            "INSERT INTO tqhnsw_insert_new_page VALUES
             (2, '[dim={large_dim},bits=4,seed=42,gamma=0.5]:{}'::tqvector)",
            hex::encode(second_code),
        ))
        .expect("insert should succeed");

        let (after_block_count, _metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert!(
            after_block_count > before_block_count,
            "insert should allocate a new data page when the tail page is full"
        );
        assert_eq!(data_pages.len(), 2, "index should now have two data pages");
    }

    #[pg_test]
    fn test_tqhnsw_insert_coalesces_duplicate_vectors() {
        Spi::run("CREATE TABLE tqhnsw_insert_duplicate (id bigint primary key, embedding tqvector)")
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

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_insert_duplicate_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (before_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.seed, 42);
        let before_tuple_count = data_pages.iter().map(|page| page.tuples.len()).sum::<usize>();

        Spi::run(
            "INSERT INTO tqhnsw_insert_duplicate VALUES
             (3, encode_to_tqvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42))",
        )
        .expect("duplicate insert should succeed");

        let (after_block_count, _metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(after_block_count, before_block_count, "duplicate insert should not allocate a new block");
        let after_tuple_count = data_pages.iter().map(|page| page.tuples.len()).sum::<usize>();
        assert_eq!(after_tuple_count, before_tuple_count, "duplicate insert should not add a new tuple pair");

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

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'tqhnsw_empty_insert_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);
        assert_eq!(metadata.max_level, 0);
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);

        let tuple_count = data_pages.iter().map(|page| page.tuples.len()).sum::<usize>();
        assert_eq!(tuple_count, 2, "aminsert should append one neighbor and one element tuple");
    }

    #[pg_test]
    #[should_panic(expected = "tqhnsw aminsert requires matching tqvector shape")]
    fn test_tqhnsw_insert_rejects_mismatched_seed() {
        Spi::run("CREATE TABLE tqhnsw_insert_seed_mismatch (id bigint primary key, embedding tqvector)")
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
