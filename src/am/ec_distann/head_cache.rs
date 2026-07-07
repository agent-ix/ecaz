//! FR-080 coordinator head index: an in-memory Vamana graph over the
//! persisted entry-region sample, cached per backend and keyed on the
//! index oid, validated against a metadata fingerprint.
//!
//! The cache entry also carries the vec_id→TID directory and (for
//! GroupedPq) the flat codebooks — all query-independent state that would
//! otherwise be re-read every rescan. Until FR-082 epochs land, the
//! fingerprint is the metadata identity (node_count, seed, chain heads,
//! codec shape); M0's DML posture (aminsert errors, bulkdelete is a no-op)
//! means index content cannot change without a REINDEX, which moves the
//! chain heads and invalidates the entry.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::storage::{page::ItemPointer, relation::RelationHandle};

use super::{
    page::{DistannMetadataPage, DISTANN_NEIGHBOR_CODEC_GROUPED_PQ},
    quantizer::grouped_centroid_count,
    reader::{
        read_directory_from_relation, read_grouped_codebooks_from_relation,
        read_head_samples_from_relation,
    },
};

/// Deterministic head-graph seed domain ("distann_tg" analog of the SPIRE
/// top-graph wrap), keeping FR-080-AC-2 rebuild determinism explicit.
const DISTANN_HEAD_GRAPH_SEED_WRAP: u64 = 0x6469_7374_5f74_6721;

#[derive(Debug, Clone, PartialEq)]
struct DistannCacheFingerprint {
    node_count: u64,
    dimensions: u16,
    seed: u64,
    entry_point: ItemPointer,
    head_sample_head: ItemPointer,
    directory_head: ItemPointer,
    grouped_codebook_head: ItemPointer,
    neighbor_codec_kind: u8,
    codec_subvector_count: u16,
    codec_subvector_dim: u16,
}

impl DistannCacheFingerprint {
    fn of(metadata: &DistannMetadataPage) -> Self {
        Self {
            node_count: metadata.node_count,
            dimensions: metadata.dimensions,
            seed: metadata.seed,
            entry_point: metadata.entry_point,
            head_sample_head: metadata.head_sample_head,
            directory_head: metadata.directory_head,
            grouped_codebook_head: metadata.grouped_codebook_head,
            neighbor_codec_kind: metadata.neighbor_codec_kind,
            codec_subvector_count: metadata.codec_subvector_count,
            codec_subvector_dim: metadata.codec_subvector_dim,
        }
    }
}

pub(crate) struct DistannIndexCacheEntry {
    fingerprint: DistannCacheFingerprint,
    /// Ascending vec_id -> record TID (FR-078's per-node resolution map).
    pub(crate) directory: Vec<(u64, ItemPointer)>,
    /// In-memory Vamana over the sample; node ids index the sample arrays.
    pub(crate) head_graph: crate::am::VamanaGraph,
    pub(crate) head_entry: u32,
    pub(crate) head_vec_ids: Vec<u64>,
    pub(crate) head_vectors: Vec<Vec<f32>>,
    /// Flat GroupedPq centroids (None for the seeded codecs).
    pub(crate) flat_codebooks: Option<Vec<f32>>,
}

type CacheMap = HashMap<u32, Arc<DistannIndexCacheEntry>>;

fn cache() -> &'static Mutex<CacheMap> {
    static CACHE: OnceLock<Mutex<CacheMap>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Fetch (or build) the cached query-independent scan state for an index.
pub(crate) fn cached_index_entry(
    index_oid: u32,
    handle: RelationHandle,
    metadata: &DistannMetadataPage,
) -> Result<Arc<DistannIndexCacheEntry>, String> {
    let fingerprint = DistannCacheFingerprint::of(metadata);
    {
        let cache = cache()
            .lock()
            .map_err(|_| "ec_distann head cache lock poisoned".to_owned())?;
        if let Some(entry) = cache.get(&index_oid) {
            if entry.fingerprint == fingerprint {
                return Ok(Arc::clone(entry));
            }
        }
    }

    let entry = Arc::new(build_cache_entry(handle, metadata, fingerprint)?);
    cache()
        .lock()
        .map_err(|_| "ec_distann head cache lock poisoned".to_owned())?
        .insert(index_oid, Arc::clone(&entry));
    Ok(entry)
}

fn build_cache_entry(
    handle: RelationHandle,
    metadata: &DistannMetadataPage,
    fingerprint: DistannCacheFingerprint,
) -> Result<DistannIndexCacheEntry, String> {
    let node_count = usize::try_from(metadata.node_count)
        .map_err(|_| "ec_distann node_count exceeds usize".to_owned())?;
    let directory = read_directory_from_relation(handle, metadata.directory_head, node_count)?;

    let samples = read_head_samples_from_relation(
        handle,
        metadata.head_sample_head,
        usize::from(metadata.dimensions),
        metadata.head_index_cap as usize,
    )?;
    let head_vec_ids: Vec<u64> = samples.iter().map(|sample| sample.vec_id).collect();
    let head_vectors: Vec<Vec<f32>> = samples.into_iter().map(|sample| sample.vector).collect();

    // Deterministic in-memory Vamana over the sample (FR-080-AC-2). The
    // build distance is the same clamped 1-ip rule as the main graph.
    let head_dist = |left: u32, right: u32| -> f32 {
        crate::am::ec_diskann::source_inner_product_distance(
            &head_vectors[left as usize],
            &head_vectors[right as usize],
        )
    };
    let head_seed = metadata.seed ^ DISTANN_HEAD_GRAPH_SEED_WRAP;
    let head_entry = crate::am::approximate_medoid(
        head_vectors.len(),
        head_vectors.len(),
        head_seed,
        head_dist,
    );
    let (head_graph, _stats) = crate::am::build_vamana_graph_with_stats(
        head_vectors.len(),
        head_entry,
        usize::from(metadata.graph_degree_r),
        usize::from(metadata.build_list_size_l),
        metadata.alpha,
        head_seed,
        head_dist,
    );

    let flat_codebooks = if metadata.neighbor_codec_kind == DISTANN_NEIGHBOR_CODEC_GROUPED_PQ {
        Some(read_grouped_codebooks_from_relation(
            handle,
            metadata.grouped_codebook_head,
            usize::from(metadata.codec_subvector_count),
            grouped_centroid_count(metadata),
        )?)
    } else {
        None
    };

    Ok(DistannIndexCacheEntry {
        fingerprint,
        directory,
        head_graph,
        head_entry,
        head_vec_ids,
        head_vectors,
        flat_codebooks,
    })
}
