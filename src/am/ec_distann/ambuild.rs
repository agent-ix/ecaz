//! pgrx-side ambuild wiring for `ec_distann` (Task 162 M0).
//!
//! Monolithic seed-deterministic build (the single-shard degenerate case of
//! FR-077): heap scan → D6 vec_id assignment with collision detection →
//! exact-distance Vamana build (shared `build_vamana_graph_with_stats`
//! core, ADR-085 reuse posture) → per-node codec encode → FR-076 lean
//! records (search code + neighbor vec_ids + embedded neighbor codes, NO
//! inline vector per D11) → sorted vec_id→TID directory → FR-080
//! entry-region BFS sample → metadata page.
//!
//! The build distance is `1 - ip(source, source)` over unit-normalized
//! vectors, identical to `ec_diskann`, so M0 recall parity compares graphs
//! built by the same rule.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr::{self, NonNull};

use pgrx::{pg_sys, PgBox};

use crate::am::common::callback::pg_am_callback;
use crate::am::ec_diskann::{
    decode_heap_tid, ecvector_datum_to_vec, source_inner_product_distance, write_data_pages,
};
use crate::storage::buffer_guard::LockedBufferGuard;
use crate::storage::page::{DataPageChain, ItemPointer, METADATA_BLOCK_NUMBER};
use crate::storage::relation::RelationHandle;
use crate::storage::wal;
use crate::DEFAULT_QUANT_SEED;

use super::identity::{vec_id_from_local_heap_tid, vec_id_from_source_identity};
use super::options::{self, DistannSourceIdentityProvider, EcDistannOptions};
use super::page::DistannMetadataPage;
use super::quantizer::DistannCodecBinding;
use super::tuple::{DistannDirectoryTuple, DistannHeadSampleTuple, DistannNodeTuple};

const P_NEW: pg_sys::BlockNumber = u32::MAX;

/// Same cap as the ec_diskann build (`build.rs::MEDOID_SAMPLE_CAP`).
const DISTANN_MEDOID_SAMPLE_CAP: usize = 1000;

/// Directory chunk size; 400 entries * 14 B fits one page with headroom.
const DISTANN_DIRECTORY_CHUNK_ENTRIES: usize = 400;

const DISTANN_UNIT_NORM_SAMPLE_CAP: usize = 1024;
const DISTANN_UNIT_NORM_EPSILON: f32 = 0.01;

/// ADR-063 canonical identity payload width (uuid / bytea16).
const DISTANN_SOURCE_IDENTITY_BYTES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DistannIdentityDatumKind {
    Uuid,
    Bytea16,
}

/// Resolved `source_identity = 'include'` attribute (ADR-063 v1 provider:
/// exactly one INCLUDE column of type uuid or bytea, canonicalized to the
/// 16-byte payload; NULL or wrong-width values reject the row).
#[derive(Debug, Clone, Copy)]
struct DistannIdentityAttribute {
    index_attr_offset: usize,
    datum_kind: DistannIdentityDatumKind,
}

struct CollectedRow {
    heap_tid: ItemPointer,
    source_vector: Vec<f32>,
    identity_payload: Option<[u8; DISTANN_SOURCE_IDENTITY_BYTES]>,
}

struct BuildState {
    options: EcDistannOptions,
    identity: Option<DistannIdentityAttribute>,
    dimensions: Option<u16>,
    rows: Vec<CollectedRow>,
    scanned_tuples: usize,
}

impl BuildState {
    fn new(index_relation: pg_sys::Relation) -> Self {
        let options = options::relation_options(index_relation);
        Self {
            options,
            identity: None,
            dimensions: None,
            rows: Vec::new(),
            scanned_tuples: 0,
        }
    }

    fn push(
        &mut self,
        heap_tid: ItemPointer,
        source_vector: Vec<f32>,
        identity_payload: Option<[u8; DISTANN_SOURCE_IDENTITY_BYTES]>,
    ) {
        self.scanned_tuples += 1;
        if source_vector.is_empty() {
            pgrx::error!("ec_distann ambuild received an empty indexed vector");
        }
        let dim = u16::try_from(source_vector.len()).unwrap_or_else(|_| {
            pgrx::error!(
                "ec_distann indexed vector dimension {} exceeds 65535",
                source_vector.len()
            )
        });
        match self.dimensions {
            None => self.dimensions = Some(dim),
            Some(existing) if existing == dim => {}
            Some(existing) => pgrx::error!(
                "ec_distann ambuild requires a single dimension; saw {dim} after {existing}"
            ),
        }
        self.rows.push(CollectedRow {
            heap_tid,
            source_vector,
            identity_payload,
        });
    }
}

fn empty_metadata(state: &BuildState) -> DistannMetadataPage {
    DistannMetadataPage::empty(
        state.options.graph_degree as u16,
        state.options.build_list_size as u16,
        state.options.alpha,
        state.dimensions.unwrap_or(0),
        DEFAULT_QUANT_SEED,
        state.options.neighbor_code_format.metadata_kind(),
        state.options.head_index_cap as u32,
        state.options.closure_epsilon,
    )
}

pub(super) unsafe extern "C-unwind" fn ec_distann_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    pg_am_callback!({
        let mut state = BuildState::new(index_relation);
        state.identity =
            resolve_identity_attribute(heap_relation, index_info, &state.options);

        initialize_metadata_page(index_relation, empty_metadata(&state));

        let heap_tuples = pg_sys::table_index_build_scan(
            heap_relation,
            index_relation,
            index_info,
            false,
            false,
            Some(ec_distann_build_callback),
            (&mut state as *mut BuildState).cast(),
            ptr::null_mut(),
        );

        // `heap_tuples` counts live rows only; the callback may legitimately
        // also receive recently-dead tuples (they must be indexed for
        // concurrent snapshots), so no equality check between the two.

        let index_tuples = if state.rows.is_empty() {
            0.0
        } else {
            flush_build_state(index_relation, &state)
                .unwrap_or_else(|e| pgrx::error!("ec_distann ambuild failed: {e}"));
            state.rows.len() as f64
        };

        let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
        result.heap_tuples = heap_tuples;
        result.index_tuples = index_tuples;
        result.into_pg()
    })
}

pub(super) unsafe extern "C-unwind" fn ec_distann_ambuildempty(index_relation: pg_sys::Relation) {
    pg_am_callback!({
        let state = BuildState::new(index_relation);
        initialize_metadata_page(index_relation, empty_metadata(&state));
    })
}

unsafe extern "C-unwind" fn ec_distann_build_callback(
    _index: pg_sys::Relation,
    tid: pg_sys::ItemPointer,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    _tuple_is_alive: bool,
    state: *mut c_void,
) {
    pg_am_callback!({
        if values.is_null() || isnull.is_null() {
            pgrx::error!("ec_distann build callback received null datum arrays");
        }
        if *isnull {
            pgrx::error!("ec_distann does not support NULL indexed values");
        }
        let datum = *values;
        if datum.is_null() {
            pgrx::error!("ec_distann build callback received a null indexed datum");
        }
        let state = &mut *state.cast::<BuildState>();
        let source_vector = ecvector_datum_to_vec(datum);
        let heap_tid = decode_heap_tid(tid);
        let identity_payload = state
            .identity
            .map(|identity| extract_identity_payload(identity, values, isnull));
        state.push(heap_tid, source_vector, identity_payload);
    })
}

/// Canonicalize the INCLUDE-column datum into the 16-byte ADR-063 payload.
/// NULL and wrong-width values reject the row: mixed identity namespaces
/// inside a global-writer index would break cross-node dedupe.
///
/// # Safety
/// `values`/`isnull` are the live PG build/insert callback arrays and
/// `identity` came from `resolve_identity_attribute` on the same index.
unsafe fn extract_identity_payload(
    identity: DistannIdentityAttribute,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
) -> [u8; DISTANN_SOURCE_IDENTITY_BYTES] {
    let values_view = crate::am::common::pg_ptr::DatumArrayView::new(
        NonNull::new(values).expect("ec_distann values should be non-null"),
        NonNull::new(isnull).expect("ec_distann isnull should be non-null"),
    );
    let datum = values_view.non_null_datum(
        identity.index_attr_offset,
        "ec_distann",
        "source_identity INCLUDE column",
    );
    match identity.datum_kind {
        DistannIdentityDatumKind::Uuid => {
            // PostgreSQL UUID datums are fixed 16-byte payloads.
            let bytes = std::slice::from_raw_parts(
                datum.cast_mut_ptr::<u8>(),
                DISTANN_SOURCE_IDENTITY_BYTES,
            );
            let mut payload = [0_u8; DISTANN_SOURCE_IDENTITY_BYTES];
            payload.copy_from_slice(bytes);
            payload
        }
        DistannIdentityDatumKind::Bytea16 => {
            let bytes = crate::am::common::detoast::DetoastedVarlena::packed_from_datum(datum)
                .unwrap_or_else(|| {
                    pgrx::error!("ec_distann could not detoast the source_identity INCLUDE column")
                })
                .to_vec();
            <[u8; DISTANN_SOURCE_IDENTITY_BYTES]>::try_from(bytes.as_slice()).unwrap_or_else(
                |_| {
                    pgrx::error!(
                        "ec_distann source_identity bytea payload length {} must be {} bytes",
                        bytes.len(),
                        DISTANN_SOURCE_IDENTITY_BYTES
                    )
                },
            )
        }
    }
}

/// Validate the ADR-063 DDL shape and resolve the identity attribute:
/// local mode = exactly one key column; include mode = one key column plus
/// exactly one INCLUDE column of type uuid or bytea.
unsafe fn resolve_identity_attribute(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    options: &EcDistannOptions,
) -> Option<DistannIdentityAttribute> {
    if index_info.is_null() {
        pgrx::error!("ec_distann ambuild received a null IndexInfo");
    }
    if heap_relation.is_null() {
        pgrx::error!("ec_distann ambuild received a null heap relation");
    }
    let info = crate::am::common::pg_ptr::index_info(
        NonNull::new(index_info).expect("ec_distann IndexInfo should be non-null"),
    );
    if info.ii_NumIndexKeyAttrs != 1 {
        pgrx::error!("ec_distann currently supports single-key-column indexes only");
    }
    match options.source_identity {
        DistannSourceIdentityProvider::None => {
            if info.ii_NumIndexAttrs != 1 {
                pgrx::error!(
                    "ec_distann without source_identity='include' takes no INCLUDE columns"
                );
            }
            None
        }
        DistannSourceIdentityProvider::Include => {
            if info.ii_NumIndexAttrs != 2 {
                pgrx::error!(
                    "ec_distann source_identity='include' requires exactly one INCLUDE column (uuid or 16-byte bytea)"
                );
            }
            let identity_attnum = i32::from(info.ii_IndexAttrNumbers[1]);
            if identity_attnum <= 0 {
                pgrx::error!(
                    "ec_distann source_identity INCLUDE column must be a base heap column"
                );
            }
            let tuple_desc = crate::storage::relation::relation_tuple_desc_copy_handle(
                NonNull::new(heap_relation).expect("heap relation should be non-null"),
            );
            let identity_att = tuple_desc
                .get(identity_attnum as usize - 1)
                .unwrap_or_else(|| {
                    pgrx::error!("ec_distann source_identity INCLUDE column not found in heap")
                });
            if identity_att.attisdropped {
                pgrx::error!(
                    "ec_distann source_identity INCLUDE column references a dropped column"
                );
            }
            let datum_kind = match crate::storage::type_info::base_type_oid(identity_att.atttypid)
            {
                pg_sys::UUIDOID => DistannIdentityDatumKind::Uuid,
                pg_sys::BYTEAOID => DistannIdentityDatumKind::Bytea16,
                _ => pgrx::error!(
                    "ec_distann source_identity INCLUDE column must be uuid or bytea"
                ),
            };
            Some(DistannIdentityAttribute {
                index_attr_offset: 1,
                datum_kind,
            })
        }
    }
}

unsafe fn flush_build_state(
    index_relation: pg_sys::Relation,
    state: &BuildState,
) -> Result<(), String> {
    let dimensions = state
        .dimensions
        .expect("non-empty build should record dimensions");
    let node_count = state.rows.len();
    let graph_degree = u16::try_from(state.options.graph_degree)
        .map_err(|_| "ec_distann graph_degree exceeds u16".to_owned())?;
    let seed = DEFAULT_QUANT_SEED;

    let source_refs: Vec<&[f32]> = state
        .rows
        .iter()
        .map(|row| row.source_vector.as_slice())
        .collect();
    warn_on_non_unit_source_sample(&source_refs);

    // D6 vec_id assignment + build-time collision detection. Include mode
    // hashes the ADR-063 canonical payload (stable across rebuilds, table
    // rewrites, and nodes); local mode hashes the heap TID (single-node
    // posture, stable across index rebuilds only).
    let vec_ids: Vec<u64> = state
        .rows
        .iter()
        .map(|row| match &row.identity_payload {
            Some(payload) => vec_id_from_source_identity(payload),
            None => vec_id_from_local_heap_tid(row.heap_tid),
        })
        .collect();
    let mut seen_vec_ids: HashMap<u64, usize> = HashMap::with_capacity(node_count);
    for (node, vec_id) in vec_ids.iter().enumerate() {
        match seen_vec_ids.entry(*vec_id) {
            Entry::Vacant(slot) => {
                slot.insert(node);
            }
            Entry::Occupied(existing) => {
                let prior = &state.rows[*existing.get()];
                let current = &state.rows[node];
                return Err(format!(
                    "ec_distann vec_id collision (ADR-085 D6 build error): vec_id {vec_id:#018x} \
                     for heap tids ({},{}) and ({},{})",
                    prior.heap_tid.block_number,
                    prior.heap_tid.offset_number,
                    current.heap_tid.block_number,
                    current.heap_tid.offset_number,
                ));
            }
        }
    }

    // Codec binding + per-node codes (search code == neighbor-code format).
    let binding = DistannCodecBinding::prepare(
        state.options.neighbor_code_format,
        &source_refs,
        usize::from(dimensions),
        seed,
    )?;
    let code_len = binding.code_len(usize::from(dimensions))?;
    let codes: Vec<Vec<u8>> = source_refs
        .iter()
        .map(|source| {
            let code = binding.encode(source);
            if code.len() != code_len {
                return Err(format!(
                    "ec_distann codec produced a {}-byte code, expected {code_len}",
                    code.len()
                ));
            }
            Ok(code)
        })
        .collect::<Result<_, String>>()?;

    // Seed-deterministic Vamana build over exact distances (FR-077
    // determinism contract starts here). The monolithic path is the
    // single-shard degenerate case; `build_shards >= 2` selects the sharded
    // closure-overlap build + stitch (FR-077), which returns a graph in the
    // same global node-id space so all downstream staging is unchanged.
    let dist = |left: u32, right: u32| -> f32 {
        source_inner_product_distance(source_refs[left as usize], source_refs[right as usize])
    };
    let shard_count = super::shard_build::resolve_shard_count(
        node_count,
        usize::try_from(state.options.build_shards.max(0)).unwrap_or(0),
    );
    let (graph, medoid) = if shard_count >= 2 {
        let (graph, medoid, shard_stats) = super::shard_build::build_sharded_graph(
            &source_refs,
            usize::from(dimensions),
            state.options.graph_degree as usize,
            state.options.build_list_size as usize,
            state.options.alpha,
            shard_count,
            state.options.closure_epsilon,
            seed,
        )?;
        // FR-077-AC-3 / ADR-085 D8: surface the closure/stitch manifest rows.
        // Single-node M1 has no persisted epoch manifest yet (FR-082, M2/M3),
        // so these land as a build NOTICE captured in the packet artifacts.
        pgrx::notice!(
            "ec_distann sharded build: shards={} duplication_factor={:.4} max_shard_size={} \
             stitch_edges_before_prune={} stitch_edges_after_prune={} stitch_peak_union_len={} \
             shard_output_retained_node_ids={} reachability_repairs={}",
            shard_stats.shard_count,
            shard_stats.duplication_factor,
            shard_stats.max_shard_size,
            shard_stats.stitch_edges_before_prune,
            shard_stats.stitch_edges_after_prune,
            shard_stats.stitch_peak_union_len,
            shard_stats.shard_output_retained_node_ids,
            shard_stats.reachability_repairs,
        );
        (graph, medoid)
    } else {
        let medoid =
            crate::am::approximate_medoid(node_count, DISTANN_MEDOID_SAMPLE_CAP, seed, dist);
        let (graph, _stats) = crate::am::build_vamana_graph_with_stats(
            node_count,
            medoid,
            state.options.graph_degree as usize,
            state.options.build_list_size as usize,
            state.options.alpha,
            seed,
            dist,
        );
        (graph, medoid)
    };

    // Stage FR-076 node records; adjacency references neighbors by vec_id
    // and embeds their codes, so records never carry TIDs of other records.
    let page_size = pg_sys::BLCKSZ as usize;
    let mut chain = DataPageChain::new(page_size);
    let mut node_tids = Vec::with_capacity(node_count);
    let r = graph_degree as usize;
    for node in 0..node_count {
        let neighbors = &graph.neighbors[node];
        if neighbors.len() > r {
            return Err(format!(
                "ec_distann build produced {} neighbors for node {node}, exceeding graph_degree {r}",
                neighbors.len()
            ));
        }
        let mut neighbor_vec_ids = vec![0_u64; r];
        let mut neighbor_codes = vec![0_u8; r * code_len];
        for (slot, neighbor) in neighbors.iter().enumerate() {
            neighbor_vec_ids[slot] = vec_ids[*neighbor as usize];
            neighbor_codes[slot * code_len..(slot + 1) * code_len]
                .copy_from_slice(&codes[*neighbor as usize]);
        }
        let tuple = DistannNodeTuple {
            tombstoned: false,
            vec_id: vec_ids[node],
            heap_tid: state.rows[node].heap_tid,
            neighbor_count: u16::try_from(neighbors.len())
                .expect("neighbor count bounded by graph_degree u16"),
            search_code: codes[node].clone(),
            neighbor_vec_ids,
            neighbor_codes,
        };
        let payload = tuple.encode(graph_degree, code_len)?;
        node_tids.push(chain.insert_raw_tuple(payload)?);
    }

    // GroupedPq codebooks persist in the same chain (seeded codecs skip).
    let grouped_codebook_head = match binding.grouped_model() {
        Some(model) => {
            crate::am::ec_diskann::persist::stage_grouped_codebook_chain(&mut chain, model)?
        }
        None => ItemPointer::INVALID,
    };

    let directory_head = stage_directory_chain(&mut chain, &vec_ids, &node_tids)?;
    let head_sample_head = stage_head_sample_chain(
        &mut chain,
        &graph,
        medoid,
        state.options.head_index_cap as usize,
        &vec_ids,
        &source_refs,
    )?;

    let handle = NonNull::new(index_relation)
        .ok_or_else(|| "ec_distann flush needs a valid index relation".to_owned())?;
    write_data_pages(handle, &chain);

    let mut metadata = empty_metadata(state);
    metadata.dimensions = dimensions;
    metadata.entry_point = node_tids[medoid as usize];
    metadata.node_count = node_count as u64;
    metadata.codec_subvector_count = binding.metadata_subvector_count();
    metadata.codec_subvector_dim = binding.metadata_subvector_dim();
    metadata.grouped_codebook_head = grouped_codebook_head;
    metadata.directory_head = directory_head;
    metadata.head_sample_head = head_sample_head;
    // FR-082 build-time content digest (Task 164, reviewer 2026-07-08-01 P1):
    // binds the sorted vec_id set + co-placed source vectors + stitched
    // adjacency so the epoch fingerprint distinguishes two graphs that share
    // shape metadata but differ in content (a stale remote node cannot pass
    // validation with a different vec_id set / vectors / edges).
    metadata.content_digest = compute_content_digest(&vec_ids, &source_refs, &graph);
    overwrite_metadata_page_handle(handle, &metadata);
    Ok(())
}

/// Deterministic digest over the build-time content: for each node in vec_id
/// order, fold its vec_id, its full-precision source vector (the co-placed
/// rerank tier, ADR-085 D11), and its adjacency (neighbor vec_ids, sorted for
/// per-node order-independence). Two builds of the same corpus+seed+options
/// yield the same digest (FR-077 determinism); any change to the vec_id set,
/// a vector, or an edge changes it. FNV-1a lanes with a final avalanche.
fn compute_content_digest(
    vec_ids: &[u64],
    source_refs: &[&[f32]],
    graph: &crate::am::VamanaGraph,
) -> u64 {
    let mut order: Vec<usize> = (0..vec_ids.len()).collect();
    order.sort_unstable_by_key(|&node| vec_ids[node]);

    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let fold = |value: u64, hash: &mut u64| {
        *hash ^= value;
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        *hash ^= *hash >> 29;
    };
    for &node in &order {
        fold(vec_ids[node], &mut hash);
        for component in source_refs[node] {
            fold(u64::from(component.to_bits()), &mut hash);
        }
        let mut neighbor_vec_ids: Vec<u64> = graph.neighbors[node]
            .iter()
            .map(|&neighbor| vec_ids[neighbor as usize])
            .collect();
        neighbor_vec_ids.sort_unstable();
        fold(neighbor_vec_ids.len() as u64, &mut hash);
        for neighbor in neighbor_vec_ids {
            fold(neighbor, &mut hash);
        }
    }
    // Final avalanche (murmur3 fmix64 constants).
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xff51_afd7_ed55_8ccd);
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    hash ^= hash >> 33;
    hash
}

/// Stage the sorted vec_id→TID directory. Chunks are staged in reverse so
/// each can point at the next chunk's already-known TID; the returned head
/// covers the ascending-sorted entry stream.
fn stage_directory_chain(
    chain: &mut DataPageChain,
    vec_ids: &[u64],
    node_tids: &[ItemPointer],
) -> Result<ItemPointer, String> {
    let mut entries: Vec<(u64, ItemPointer)> = vec_ids
        .iter()
        .copied()
        .zip(node_tids.iter().copied())
        .collect();
    entries.sort_unstable_by_key(|(vec_id, _)| *vec_id);

    let mut next_tid = ItemPointer::INVALID;
    let chunks: Vec<&[(u64, ItemPointer)]> =
        entries.chunks(DISTANN_DIRECTORY_CHUNK_ENTRIES).collect();
    for chunk in chunks.iter().rev() {
        let tuple = DistannDirectoryTuple {
            next_tid,
            entries: chunk.to_vec(),
        };
        next_tid = chain.insert_raw_tuple(tuple.encode()?)?;
    }
    Ok(next_tid)
}

/// FR-080 entry-region sample: deterministic BFS from the medoid over the
/// built graph, capped at `head_index_cap` (C). The monolithic build is the
/// single-shard degenerate case: one medoid, one BFS sample.
fn stage_head_sample_chain(
    chain: &mut DataPageChain,
    graph: &crate::am::VamanaGraph,
    medoid: u32,
    head_index_cap: usize,
    vec_ids: &[u64],
    source_refs: &[&[f32]],
) -> Result<ItemPointer, String> {
    let node_count = vec_ids.len();
    let cap = head_index_cap.min(node_count).max(1);
    let mut sampled = Vec::with_capacity(cap);
    let mut visited = vec![false; node_count];
    let mut queue = std::collections::VecDeque::new();
    visited[medoid as usize] = true;
    queue.push_back(medoid);
    while let Some(node) = queue.pop_front() {
        sampled.push(node);
        if sampled.len() >= cap {
            break;
        }
        for neighbor in &graph.neighbors[node as usize] {
            let index = *neighbor as usize;
            if !visited[index] {
                visited[index] = true;
                queue.push_back(*neighbor);
            }
        }
    }

    // Stage in reverse for next_tid linking; the head yields BFS order.
    let mut next_tid = ItemPointer::INVALID;
    for node in sampled.iter().rev() {
        let index = *node as usize;
        let tuple = DistannHeadSampleTuple {
            next_tid,
            vec_id: vec_ids[index],
            vector: source_refs[index].to_vec(),
        };
        next_tid = chain.insert_raw_tuple(tuple.encode())?;
    }
    Ok(next_tid)
}

fn warn_on_non_unit_source_sample(source_refs: &[&[f32]]) {
    let sample_len = source_refs.len().min(DISTANN_UNIT_NORM_SAMPLE_CAP);
    for (index, source) in source_refs.iter().take(sample_len).enumerate() {
        let norm = source.iter().map(|v| v * v).sum::<f32>().sqrt();
        if !norm.is_finite() || (norm - 1.0).abs() > DISTANN_UNIT_NORM_EPSILON {
            pgrx::warning!(
                "ec_distann ambuild expects unit-normalized source vectors for the exact \
                 1-ip build distance; sampled ||v|| = {norm:.4} at position {index}"
            );
            return;
        }
    }
}

/// Decode the block-0 metadata record. Shared by later scan/build slices
/// and the TC-037 metadata assertions.
pub(crate) unsafe fn read_metadata_from_index(
    index_relation: pg_sys::Relation,
) -> Result<DistannMetadataPage, String> {
    let handle = NonNull::new(index_relation)
        .ok_or_else(|| "ec_distann metadata read needs a valid index relation".to_owned())?;
    read_metadata_from_index_handle(handle)
}

pub(crate) fn read_metadata_from_index_handle(
    handle: RelationHandle,
) -> Result<DistannMetadataPage, String> {
    let metadata_buffer = LockedBufferGuard::read_main_handle(
        handle,
        METADATA_BLOCK_NUMBER,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_SHARE as i32,
    )
    .ok_or_else(|| "ec_distann could not open metadata page".to_owned())?;
    let page = metadata_buffer.page();
    // SAFETY: `page` comes from the locked metadata buffer and remains pinned
    // while the special area is inspected; the size check below guarantees the
    // special pointer covers the serialized metadata payload.
    let metadata_bytes = unsafe {
        let special_size = pg_sys::PageGetSpecialSize(page) as usize;
        if special_size < super::page::DISTANN_METADATA_BYTES {
            return Err(format!(
                "ec_distann metadata page special area too small: got {special_size}, expected at least {}",
                super::page::DISTANN_METADATA_BYTES
            ));
        }
        let metadata_ptr = pg_sys::PageGetSpecialPointer(page).cast::<u8>();
        std::slice::from_raw_parts(
            metadata_ptr.cast_const(),
            super::page::DISTANN_METADATA_BYTES,
        )
    };
    DistannMetadataPage::decode(metadata_bytes)
}

unsafe fn initialize_metadata_page(
    index_relation: pg_sys::Relation,
    metadata: DistannMetadataPage,
) {
    let handle = NonNull::new(index_relation).unwrap_or_else(|| {
        pgrx::error!("ec_distann metadata initialization needs a valid index relation")
    });
    initialize_metadata_page_handle(handle, metadata);
}

fn initialize_metadata_page_handle(handle: RelationHandle, metadata: DistannMetadataPage) {
    let existing_blocks = crate::storage::relation::main_fork_block_count_handle(handle);
    let target_block = if existing_blocks == 0 {
        P_NEW
    } else {
        METADATA_BLOCK_NUMBER
    };
    let buffer = if target_block == P_NEW {
        LockedBufferGuard::read_main_locked_handle(
            handle,
            target_block,
            pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
        )
    } else {
        LockedBufferGuard::read_main_handle(
            handle,
            target_block,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
    }
    .unwrap_or_else(|| pgrx::error!("ec_distann failed to allocate metadata buffer"));
    write_metadata_to_buffer(handle, &buffer, &metadata);
}

fn overwrite_metadata_page_handle(handle: RelationHandle, metadata: &DistannMetadataPage) {
    let buffer = LockedBufferGuard::read_main_handle(
        handle,
        METADATA_BLOCK_NUMBER,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
    )
    .unwrap_or_else(|| pgrx::error!("ec_distann failed to open metadata buffer"));
    write_metadata_to_buffer(handle, &buffer, metadata);
}

fn write_metadata_to_buffer(
    handle: RelationHandle,
    buffer: &LockedBufferGuard,
    metadata: &DistannMetadataPage,
) {
    let mut wal_txn = wal::WalTxnScope::start_handle(handle);
    {
        let mut page = wal_txn.register_page(buffer);
        let metadata_bytes = metadata.encode();
        let special_size = (metadata_bytes.len() + 7) & !7;
        page.init(special_size);
        // SAFETY: `page` is the WAL-registered metadata page; the special area
        // was just sized from the encoded metadata before this owned-memory
        // copy.
        unsafe {
            let dst = pg_sys::PageGetSpecialPointer(page.page_ptr()).cast::<u8>();
            ptr::copy_nonoverlapping(metadata_bytes.as_ptr(), dst, metadata_bytes.len());
        }
    }
    wal_txn.finish();
}
