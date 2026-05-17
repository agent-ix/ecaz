use pgrx::pg_sys;

use super::{build, options, page, quantizer, training};
use crate::storage::page::ItemPointer;

const EMPTY_BOOTSTRAP_LOCK_MODE: pg_sys::LOCKMODE =
    pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE;

struct RelationLockGuard {
    relid: pg_sys::Oid,
    lockmode: pg_sys::LOCKMODE,
}

impl Drop for RelationLockGuard {
    fn drop(&mut self) {
        unsafe { pg_sys::UnlockRelationOid(self.relid, self.lockmode) };
    }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_aminsert(
    index_relation: pg_sys::Relation,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: pg_sys::ItemPointer,
    heap_relation: pg_sys::Relation,
    _check_unique: pg_sys::IndexUniqueCheck::Type,
    _index_unchanged: bool,
    index_info: *mut pg_sys::IndexInfo,
) -> bool {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let metadata = page::read_metadata_page(index_relation);
            validate_metadata_runtime_options(&metadata).unwrap_or_else(|e| pgrx::error!("{e}"));

            let indexed_vector_kind =
                build::resolve_indexed_vector_kind(heap_relation, index_info, "aminsert");
            let heap_tid = build::decode_heap_tid(heap_tid, "aminsert");
            let mut tuple = build::build_index_tuple(
                values,
                isnull,
                heap_tid,
                indexed_vector_kind,
                metadata.storage_format,
                "aminsert",
            );

            let result = if metadata.dimensions == 0 {
                insert_with_empty_bootstrap_lock(index_relation, tuple)
            } else {
                tuple = reencode_tuple_for_storage(index_relation, &metadata, tuple)
                    .unwrap_or_else(|e| pgrx::error!("ec_ivf aminsert failed: {e}"));
                insert_into_trained_index(index_relation, &metadata, tuple)
            };
            result.unwrap_or_else(|e| pgrx::error!("ec_ivf aminsert failed: {e}"));

            true
        })
    }
}

fn validate_metadata_runtime_options(metadata: &page::MetadataPage) -> Result<(), String> {
    metadata.storage_format.validate_v1_supported()?;
    metadata.rerank.validate_v1_supported()
}

unsafe fn lock_empty_bootstrap_relation(index_relation: pg_sys::Relation) -> RelationLockGuard {
    let relid = unsafe { (*index_relation).rd_id };
    unsafe { pg_sys::LockRelationOid(relid, EMPTY_BOOTSTRAP_LOCK_MODE) };
    RelationLockGuard {
        relid,
        lockmode: EMPTY_BOOTSTRAP_LOCK_MODE,
    }
}

unsafe fn insert_with_empty_bootstrap_lock(
    index_relation: pg_sys::Relation,
    tuple: build::BuildTuple,
) -> Result<(), String> {
    let guard = unsafe { lock_empty_bootstrap_relation(index_relation) };
    let metadata = unsafe { page::read_metadata_page(index_relation) };
    validate_metadata_runtime_options(&metadata)?;
    if metadata.dimensions == 0 {
        return unsafe { bootstrap_empty_index(index_relation, &metadata, tuple) };
    }
    drop(guard);
    unsafe { insert_into_trained_index(index_relation, &metadata, tuple) }
}

unsafe fn reencode_tuple_for_storage(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    mut tuple: build::BuildTuple,
) -> Result<build::BuildTuple, String> {
    if metadata.storage_format != options::StorageFormat::PqFastScan {
        return Ok(tuple);
    }
    let model = unsafe { quantizer::load_pq_fastscan_model(index_relation, metadata) }?;
    let ivf_quantizer = quantizer::IvfQuantizer::resolve_with_pq_group_size(
        metadata.storage_format,
        usize::from(metadata.dimensions),
        metadata_pq_group_size(metadata),
    )?;
    let (dimensions, gamma, payload) =
        ivf_quantizer.encode_source_with_pq_model(&tuple.source_vector, &model)?;
    tuple.dimensions = dimensions;
    tuple.gamma = gamma;
    tuple.payload = payload;
    Ok(tuple)
}

unsafe fn insert_into_trained_index(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    tuple: build::BuildTuple,
) -> Result<(), String> {
    validate_metadata_runtime_options(metadata)?;
    validate_insert_tuple(metadata, &tuple)
        .map_err(|e| format!("ec_ivf aminsert found invalid tuple: {e}"))?;

    let model = unsafe { load_centroid_model(index_relation, metadata) }?;
    let list_id = training::assign_vector_to_centroid(&tuple.source_vector, &model)
        .map_err(|e| format!("ec_ivf aminsert centroid assignment failed: {e}"))?;
    let (directory_tid, directory) =
        unsafe { load_directory_entry(index_relation, metadata, list_id) }?;

    let posting = page::IvfPostingTuple {
        list_id: u32::try_from(list_id)
            .map_err(|_| "ec_ivf assigned list id exceeds u32".to_owned())?,
        deleted: false,
        heaptids: vec![tuple.heap_tid],
        gamma: tuple.gamma,
        rerank_tid: ItemPointer::INVALID,
        payload: tuple.payload,
    };
    let block_range = live_insert_block_range(&directory)
        .map_err(|e| format!("ec_ivf aminsert found invalid directory: {e}"))?;
    // Invariant: PostgreSQL calls aminsert with a fresh heap TID for INSERT
    // and non-HOT UPDATE paths; VACUUM removes any old index entries before
    // a heap line pointer can be reused. The debug validation helper below
    // keeps the corruption-check path available without scanning on inserts.
    let posting_tid =
        unsafe { page::append_ivf_posting_to_list_range(index_relation, block_range, &posting) }?;

    unsafe {
        page::update_ivf_list_directory(index_relation, directory_tid, |latest_directory| {
            if latest_directory.list_id != posting.list_id {
                return Err(format!(
                    "ec_ivf directory order mismatch during insert: got list {}, expected {}",
                    latest_directory.list_id, posting.list_id
                ));
            }
            apply_directory_insert_stats(latest_directory, posting_tid)
        })
    }
    .map_err(|e| format!("ec_ivf aminsert stats update failed: {e}"))?;
    unsafe { page::update_metadata_page(index_relation, apply_metadata_insert_stats) }
        .map_err(|e| format!("ec_ivf aminsert metadata update failed: {e}"))?;

    Ok(())
}

unsafe fn ensure_heap_tid_absent(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    heap_tid: ItemPointer,
) -> Result<(), String> {
    if metadata.directory_head == ItemPointer::INVALID {
        return Err("ec_ivf metadata has live dimensions but no directory head".to_owned());
    }

    let payload_len = super::quantizer::IvfQuantizer::resolve_with_pq_group_size(
        metadata.storage_format,
        metadata.dimensions as usize,
        metadata_pq_group_size(metadata),
    )?
    .payload_len();
    let mut next_tid = metadata.directory_head;
    for expected_list_id in 0..metadata.nlists {
        let (directory, following_tid) =
            unsafe { page::read_ivf_list_directory_and_next(index_relation, next_tid)? };
        if directory.list_id != expected_list_id {
            return Err(format!(
                "ec_ivf directory order mismatch: got list {}, expected {}",
                directory.list_id, expected_list_id
            ));
        }

        let postings = unsafe {
            page::read_ivf_postings_for_list_blocks(
                index_relation,
                directory.list_id,
                directory.head_block,
                directory.tail_block,
                payload_len,
            )?
        };
        if postings
            .iter()
            .filter(|posting| !posting.deleted)
            .any(|posting| posting.heaptids.contains(&heap_tid))
        {
            return Err(format!(
                "{}:{} is already present in the index",
                heap_tid.block_number, heap_tid.offset_number
            ));
        }

        next_tid = following_tid;
    }

    Ok(())
}

unsafe fn bootstrap_empty_index(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    tuple: build::BuildTuple,
) -> Result<(), String> {
    let options = options_from_metadata(metadata)?;
    let plan = build::stage_single_tuple_build_plan(options, tuple)?;
    unsafe { build::flush_build_plan(index_relation, &plan) };
    Ok(())
}

fn options_from_metadata(metadata: &page::MetadataPage) -> Result<options::EcIvfOptions, String> {
    Ok(options::EcIvfOptions {
        nlists: i32::try_from(metadata.nlists)
            .map_err(|_| "metadata nlists exceeds i32".to_owned())?,
        nprobe: i32::try_from(metadata.nprobe)
            .map_err(|_| "metadata nprobe exceeds i32".to_owned())?,
        rerank_width: 0,
        training_sample_rows: i32::try_from(metadata.training_sample_rows)
            .map_err(|_| "metadata training sample rows exceeds i32".to_owned())?,
        seed: i32::try_from(metadata.seed).map_err(|_| "metadata seed exceeds i32".to_owned())?,
        pq_group_size: i32::from(metadata.pq_group_size),
        posting_slack_percent: 0,
        storage_format: metadata.storage_format,
        rerank: metadata.rerank,
    })
}

fn metadata_pq_group_size(metadata: &page::MetadataPage) -> Option<usize> {
    if metadata.storage_format == options::StorageFormat::PqFastScan && metadata.pq_group_size > 0 {
        Some(usize::from(metadata.pq_group_size))
    } else {
        None
    }
}

fn validate_insert_tuple(
    metadata: &page::MetadataPage,
    tuple: &build::BuildTuple,
) -> Result<(), String> {
    if tuple.heap_tid == ItemPointer::INVALID {
        return Err("heap tid must be valid".to_owned());
    }
    if tuple.dimensions != metadata.dimensions {
        return Err(format!(
            "dimension mismatch: inserted {}, index {}",
            tuple.dimensions, metadata.dimensions
        ));
    }
    if !tuple.gamma.is_finite() {
        return Err("posting gamma must be finite".to_owned());
    }
    if tuple.source_vector.len() != usize::from(metadata.dimensions) {
        return Err(format!(
            "source dimensions mismatch: source dim {} vs index dim {}",
            tuple.source_vector.len(),
            metadata.dimensions
        ));
    }
    let expected_payload_len = super::quantizer::IvfQuantizer::resolve_with_pq_group_size(
        metadata.storage_format,
        usize::from(metadata.dimensions),
        metadata_pq_group_size(metadata),
    )?
    .payload_len();
    if tuple.payload.len() != expected_payload_len {
        return Err(format!(
            "posting payload length mismatch: got {}, expected {expected_payload_len} for index dim {}",
            tuple.payload.len(),
            metadata.dimensions
        ));
    }
    if !page::posting_tuple_fits(tuple.payload.len(), pg_sys::BLCKSZ as usize) {
        return Err(format!(
            "posting payload for dim {} does not fit on a page",
            tuple.dimensions
        ));
    }
    Ok(())
}

unsafe fn load_centroid_model(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
) -> Result<training::SphericalKMeansModel, String> {
    if metadata.centroid_head == ItemPointer::INVALID {
        return Err("ec_ivf metadata has live dimensions but no centroid head".to_owned());
    }

    let mut centroids = Vec::with_capacity(metadata.nlists as usize);
    let mut next_tid = metadata.centroid_head;
    for expected_list_id in 0..metadata.nlists {
        let (centroid, following_tid) = unsafe {
            page::read_ivf_centroid_and_next(
                index_relation,
                next_tid,
                usize::from(metadata.dimensions),
            )?
        };
        if centroid.list_id != expected_list_id {
            return Err(format!(
                "ec_ivf centroid order mismatch: got list {}, expected {}",
                centroid.list_id, expected_list_id
            ));
        }
        centroids.push(centroid.centroid);
        next_tid = following_tid;
    }

    Ok(training::SphericalKMeansModel {
        dimensions: usize::from(metadata.dimensions),
        centroids,
    })
}

unsafe fn load_directory_entry(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    list_id: usize,
) -> Result<(ItemPointer, page::IvfListDirectoryTuple), String> {
    if metadata.directory_head == ItemPointer::INVALID {
        return Err("ec_ivf metadata has live dimensions but no directory head".to_owned());
    }

    let mut next_tid = metadata.directory_head;
    for expected_list_id in 0..metadata.nlists {
        let current_tid = next_tid;
        let (directory, following_tid) =
            unsafe { page::read_ivf_list_directory_and_next(index_relation, current_tid)? };
        if directory.list_id != expected_list_id {
            return Err(format!(
                "ec_ivf directory order mismatch: got list {}, expected {}",
                directory.list_id, expected_list_id
            ));
        }
        if expected_list_id as usize == list_id {
            return Ok((current_tid, directory));
        }
        next_tid = following_tid;
    }

    Err(format!("ec_ivf assigned list {list_id} is out of range"))
}

fn live_insert_block_range(
    directory: &page::IvfListDirectoryTuple,
) -> Result<Option<(pg_sys::BlockNumber, pg_sys::BlockNumber)>, String> {
    match (
        directory.head_block == page::BlockRef::INVALID,
        directory.tail_block == page::BlockRef::INVALID,
    ) {
        (true, true) => Ok(None),
        (false, false) => Ok(Some((
            directory.head_block.block_number,
            directory.tail_block.block_number,
        ))),
        _ => Err(format!(
            "list {} has partial posting block refs",
            directory.list_id
        )),
    }
}

fn apply_directory_insert_stats(
    directory: &mut page::IvfListDirectoryTuple,
    posting_tid: ItemPointer,
) -> Result<(), String> {
    match (
        directory.head_block == page::BlockRef::INVALID,
        directory.tail_block == page::BlockRef::INVALID,
    ) {
        (true, true) => {
            directory.head_block = page::BlockRef {
                block_number: posting_tid.block_number,
            };
            directory.tail_block = page::BlockRef {
                block_number: posting_tid.block_number,
            };
        }
        (false, false) => {
            if posting_tid.block_number < directory.head_block.block_number {
                directory.head_block = page::BlockRef {
                    block_number: posting_tid.block_number,
                };
            }
            if posting_tid.block_number > directory.tail_block.block_number {
                directory.tail_block = page::BlockRef {
                    block_number: posting_tid.block_number,
                };
            }
        }
        _ => {
            return Err(format!(
                "list {} has partial posting block refs",
                directory.list_id
            ));
        }
    }
    directory.live_count = directory
        .live_count
        .checked_add(1)
        .ok_or_else(|| "list live count overflow".to_owned())?;
    directory.inserted_since_build = directory
        .inserted_since_build
        .checked_add(1)
        .ok_or_else(|| "list inserted-since-build count overflow".to_owned())?;
    Ok(())
}

fn apply_metadata_insert_stats(metadata: &mut page::MetadataPage) -> Result<(), String> {
    metadata.total_live_tuples = metadata
        .total_live_tuples
        .checked_add(1)
        .ok_or_else(|| "metadata live tuple count overflow".to_owned())?;
    metadata.inserted_since_build = metadata
        .inserted_since_build
        .checked_add(1)
        .ok_or_else(|| "metadata inserted-since-build count overflow".to_owned())?;
    Ok(())
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_validate_no_duplicate_heap_tid(
    index_oid: pg_sys::Oid,
    block_number: u32,
    offset_number: u16,
) {
    let index_relation = crate::storage::relation_guard::IndexRelationGuard::access_share(
        index_oid,
        "debug_ec_ivf_validate_no_duplicate_heap_tid",
    );
    let metadata = unsafe { page::read_metadata_page(index_relation.as_ptr()) };
    let heap_tid = ItemPointer {
        block_number,
        offset_number,
    };
    let result = unsafe { ensure_heap_tid_absent(index_relation.as_ptr(), &metadata, heap_tid) };
    result.unwrap_or_else(|e| pgrx::error!("ec_ivf duplicate heap tid validation failed: {e}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(block_number: u32) -> page::BlockRef {
        page::BlockRef { block_number }
    }

    fn posting_tid(block_number: u32) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number: 1,
        }
    }

    #[test]
    fn directory_insert_stats_initializes_empty_block_range() {
        let mut directory = page::IvfListDirectoryTuple::empty(7);

        apply_directory_insert_stats(&mut directory, posting_tid(42)).unwrap();

        assert_eq!(directory.head_block, block(42));
        assert_eq!(directory.tail_block, block(42));
        assert_eq!(directory.live_count, 1);
        assert_eq!(directory.inserted_since_build, 1);
    }

    #[test]
    fn directory_insert_stats_extends_tail_forward() {
        let mut directory = page::IvfListDirectoryTuple {
            list_id: 7,
            head_block: block(10),
            tail_block: block(12),
            live_count: 2,
            dead_count: 0,
            inserted_since_build: 1,
        };

        apply_directory_insert_stats(&mut directory, posting_tid(13)).unwrap();

        assert_eq!(directory.head_block, block(10));
        assert_eq!(directory.tail_block, block(13));
        assert_eq!(directory.live_count, 3);
        assert_eq!(directory.inserted_since_build, 2);
    }

    #[test]
    fn directory_insert_stats_preserves_newer_tail_after_stale_tail_append() {
        let mut directory = page::IvfListDirectoryTuple {
            list_id: 7,
            head_block: block(10),
            tail_block: block(14),
            live_count: 5,
            dead_count: 0,
            inserted_since_build: 4,
        };

        apply_directory_insert_stats(&mut directory, posting_tid(12)).unwrap();

        assert_eq!(directory.head_block, block(10));
        assert_eq!(directory.tail_block, block(14));
        assert_eq!(directory.live_count, 6);
        assert_eq!(directory.inserted_since_build, 5);
    }

    #[test]
    fn directory_insert_stats_extends_head_backward() {
        let mut directory = page::IvfListDirectoryTuple {
            list_id: 7,
            head_block: block(10),
            tail_block: block(14),
            live_count: 5,
            dead_count: 0,
            inserted_since_build: 4,
        };

        apply_directory_insert_stats(&mut directory, posting_tid(9)).unwrap();

        assert_eq!(directory.head_block, block(9));
        assert_eq!(directory.tail_block, block(14));
        assert_eq!(directory.live_count, 6);
        assert_eq!(directory.inserted_since_build, 5);
    }
}
