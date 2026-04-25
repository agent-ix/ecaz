use pgrx::pg_sys;

use super::{build, options, page, training};
use crate::storage::page::ItemPointer;

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
            let mut metadata = page::read_metadata_page(index_relation);
            metadata
                .rerank
                .validate_v1_supported()
                .unwrap_or_else(|e| pgrx::error!("{e}"));

            let indexed_vector_kind =
                build::resolve_indexed_vector_kind(heap_relation, index_info, "aminsert");
            let heap_tid = build::decode_heap_tid(heap_tid, "aminsert");
            let tuple = build::build_index_tuple(
                values,
                isnull,
                heap_tid,
                indexed_vector_kind,
                "aminsert",
            );
            if metadata.dimensions == 0 {
                bootstrap_empty_index(index_relation, &metadata, tuple)
                    .unwrap_or_else(|e| pgrx::error!("ec_ivf empty-index insert failed: {e}"));
                return true;
            }
            validate_insert_tuple(&metadata, &tuple)
                .unwrap_or_else(|e| pgrx::error!("ec_ivf aminsert found invalid tuple: {e}"));

            let model = load_centroid_model(index_relation, &metadata)
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            let list_id = training::assign_vector_to_centroid(&tuple.source_vector, &model)
                .unwrap_or_else(|e| pgrx::error!("ec_ivf aminsert centroid assignment failed: {e}"));
            let (directory_tid, mut directory) =
                load_directory_entry(index_relation, &metadata, list_id)
                    .unwrap_or_else(|e| pgrx::error!("{e}"));
            ensure_heap_tid_absent(index_relation, &metadata, tuple.heap_tid)
                .unwrap_or_else(|e| pgrx::error!("ec_ivf aminsert found duplicate heap tid: {e}"));

            let posting = page::IvfPostingTuple {
                list_id: u32::try_from(list_id)
                    .unwrap_or_else(|_| pgrx::error!("ec_ivf assigned list id exceeds u32")),
                deleted: false,
                heaptids: vec![tuple.heap_tid],
                gamma: tuple.gamma,
                rerank_tid: ItemPointer::INVALID,
                payload: tuple.payload,
            };
            let tail_block = live_insert_tail_block(&directory)
                .unwrap_or_else(|e| pgrx::error!("ec_ivf aminsert found invalid directory: {e}"));
            let posting_tid = page::append_ivf_posting(index_relation, tail_block, &posting)
                .unwrap_or_else(|e| pgrx::error!("{e}"));

            apply_insert_stats(&mut metadata, &mut directory, posting_tid)
                .unwrap_or_else(|e| pgrx::error!("ec_ivf aminsert stats update failed: {e}"));
            page::rewrite_ivf_list_directory(index_relation, directory_tid, directory)
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            page::initialize_metadata_page(index_relation, metadata);

            true
        })
    }
}

unsafe fn ensure_heap_tid_absent(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    heap_tid: ItemPointer,
) -> Result<(), String> {
    if metadata.directory_head == ItemPointer::INVALID {
        return Err("ec_ivf metadata has live dimensions but no directory head".to_owned());
    }

    let payload_len = crate::code_len(metadata.dimensions as usize, crate::DEFAULT_QUANT_BITS);
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
        training_sample_rows: i32::try_from(metadata.training_sample_rows)
            .map_err(|_| "metadata training sample rows exceeds i32".to_owned())?,
        seed: i32::try_from(metadata.seed).map_err(|_| "metadata seed exceeds i32".to_owned())?,
        storage_format: metadata.storage_format,
        rerank: metadata.rerank,
    })
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
    training::normalize_vector(&tuple.source_vector, usize::from(metadata.dimensions))?;
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

fn live_insert_tail_block(
    directory: &page::IvfListDirectoryTuple,
) -> Result<Option<pg_sys::BlockNumber>, String> {
    match (
        directory.head_block == page::BlockRef::INVALID,
        directory.tail_block == page::BlockRef::INVALID,
    ) {
        (true, true) => Ok(None),
        (false, false) => Ok(Some(directory.tail_block.block_number)),
        _ => Err(format!(
            "list {} has partial posting block refs",
            directory.list_id
        )),
    }
}

fn apply_insert_stats(
    metadata: &mut page::MetadataPage,
    directory: &mut page::IvfListDirectoryTuple,
    posting_tid: ItemPointer,
) -> Result<(), String> {
    if directory.head_block == page::BlockRef::INVALID {
        directory.head_block = page::BlockRef {
            block_number: posting_tid.block_number,
        };
    }
    directory.tail_block = page::BlockRef {
        block_number: posting_tid.block_number,
    };
    directory.live_count = directory
        .live_count
        .checked_add(1)
        .ok_or_else(|| "list live count overflow".to_owned())?;
    directory.inserted_since_build = directory
        .inserted_since_build
        .checked_add(1)
        .ok_or_else(|| "list inserted-since-build count overflow".to_owned())?;
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
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let metadata = unsafe { page::read_metadata_page(index_relation) };
    let heap_tid = ItemPointer {
        block_number,
        offset_number,
    };
    let result = unsafe { ensure_heap_tid_absent(index_relation, &metadata, heap_tid) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result.unwrap_or_else(|e| pgrx::error!("ec_ivf duplicate heap tid validation failed: {e}"));
}
