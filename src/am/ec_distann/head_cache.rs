//! FR-080 coordinator head index: an in-memory Vamana graph over the
//! persisted entry-region sample, cached per backend and keyed on the
//! bounded two-entry-per-index LRU keyed on logical index/build/epoch identity and
//! validated against a metadata fingerprint.
//!
//! The cache entry also carries the vec_id→TID directory and (for
//! GroupedPq) the flat codebooks — all query-independent state that would
//! otherwise be re-read every rescan. Until FR-082 epochs land, the
//! fingerprint is the metadata identity (node_count, seed, chain heads,
//! codec shape); M0's DML posture (aminsert errors, bulkdelete is a no-op)
//! means index content cannot change without a REINDEX, which moves the
//! chain heads and invalidates the entry.

use std::collections::VecDeque;
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct DistannCacheKey {
    index_oid: u32,
    logical_index_uuid: [u8; 16],
    build_id: [u8; 16],
    epoch_fingerprint: [u8; 32],
}

impl DistannCacheKey {
    /// Legacy local indexes predate the physical generation catalog and do not
    /// have separate build/epoch UUIDs. Their persisted active epoch and
    /// content digest are the stable equivalents; the metadata fingerprint is
    /// still checked on every hit below.
    fn from_metadata(index_oid: u32, metadata: &DistannMetadataPage) -> Self {
        let mut build_id = [0_u8; 16];
        build_id[..8].copy_from_slice(&metadata.active_epoch.to_le_bytes());
        let mut epoch_fingerprint = [0_u8; 32];
        epoch_fingerprint[..8].copy_from_slice(&metadata.content_digest.to_le_bytes());
        Self {
            index_oid,
            logical_index_uuid: metadata.logical_index_uuid,
            build_id,
            epoch_fingerprint,
        }
    }
}

type CacheMap = VecDeque<(DistannCacheKey, Arc<DistannIndexCacheEntry>)>;

fn evict_oldest_for_index<T>(cache: &mut VecDeque<(DistannCacheKey, T)>, index_oid: u32) {
    let matching = cache
        .iter()
        .filter(|(key, _)| key.index_oid == index_oid)
        .count();
    if matching <= 2 {
        return;
    }
    let position = cache
        .iter()
        .rposition(|(key, _)| key.index_oid == index_oid)
        .expect("matching cache entry exists");
    cache.remove(position);
}

fn cache() -> &'static Mutex<CacheMap> {
    static CACHE: OnceLock<Mutex<CacheMap>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(VecDeque::new()))
}

/// Fetch (or build) the cached query-independent scan state for an index.
pub(crate) fn cached_index_entry(
    index_oid: u32,
    handle: RelationHandle,
    metadata: &DistannMetadataPage,
) -> Result<Arc<DistannIndexCacheEntry>, String> {
    let fingerprint = DistannCacheFingerprint::of(metadata);
    if !super::options::physical_epoch_cache_enabled() {
        return Ok(Arc::new(build_cache_entry(handle, metadata, fingerprint)?));
    }
    {
        let mut cache = cache()
            .lock()
            .map_err(|_| "ec_distann head cache lock poisoned".to_owned())?;
        let key = DistannCacheKey::from_metadata(index_oid, metadata);
        if let Some(position) = cache.iter().position(|(cached, _)| *cached == key) {
            let (cached_key, entry) = cache.remove(position).expect("cache position exists");
            if entry.fingerprint == fingerprint {
                let result = Arc::clone(&entry);
                cache.push_front((cached_key, entry));
                return Ok(result);
            }
        }
    }

    let entry = Arc::new(build_cache_entry(handle, metadata, fingerprint)?);
    let key = DistannCacheKey::from_metadata(index_oid, metadata);
    let mut cache = cache()
        .lock()
        .map_err(|_| "ec_distann head cache lock poisoned".to_owned())?;
    if let Some(position) = cache.iter().position(|(cached, _)| *cached == key) {
        cache.remove(position);
    }
    cache.push_front((key, Arc::clone(&entry)));
    // FR-080 bounds two epoch entries per logical index. A global truncate(2)
    // makes three indexes thrash one another in a backend that alternates
    // scans; retain unrelated indexes and evict only the oldest entry for the
    // index being inserted.
    evict_oldest_for_index(&mut cache, index_oid);
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
    let head_entry =
        crate::am::approximate_medoid(head_vectors.len(), head_vectors.len(), head_seed, head_dist);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn key(index_oid: u32, epoch: u8) -> DistannCacheKey {
        DistannCacheKey {
            index_oid,
            logical_index_uuid: [index_oid as u8; 16],
            build_id: [epoch; 16],
            epoch_fingerprint: [epoch; 32],
        }
    }

    #[test]
    fn cache_eviction_removes_oldest_matching_index() {
        let mut cache = VecDeque::from([
            (key(7, 3), ()),
            (key(7, 2), ()),
            (key(7, 1), ()),
            (key(9, 1), ()),
        ]);

        evict_oldest_for_index(&mut cache, 7);

        let epochs = cache
            .iter()
            .filter(|(entry, _)| entry.index_oid == 7)
            .map(|(entry, _)| entry.build_id[0])
            .collect::<Vec<_>>();
        assert_eq!(epochs, vec![3, 2]);
        assert!(cache.iter().any(|(entry, _)| entry.index_oid == 9));
    }
}
