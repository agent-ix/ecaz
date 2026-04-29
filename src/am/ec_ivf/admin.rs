use pgrx::pg_sys;

use super::{options, page, quantizer::IvfQuantizer};
use crate::storage::page::ItemPointer;

pub(crate) const REINDEX_CHANGED_ROW_FRACTION_THRESHOLD: f64 = 0.20;
pub(crate) const REINDEX_LIST_IMBALANCE_THRESHOLD: f64 = 4.0;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IndexDriftSnapshot {
    pub block_count: u32,
    pub nlists: u32,
    pub total_live_tuples: u64,
    pub total_dead_tuples: u64,
    pub inserted_since_build: u64,
    pub changed_row_fraction: f64,
    pub average_list_live_count: f64,
    pub max_list_live_count: u64,
    pub list_imbalance_ratio: f64,
    pub empty_lists: u32,
    pub reindex_recommended: bool,
    pub reindex_reason: &'static str,
    pub changed_row_reindex_threshold: f64,
    pub list_imbalance_reindex_threshold: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IndexAdminSnapshot {
    pub block_count: u32,
    pub index_pages: f64,
    pub reltuples: f64,
    pub dimensions: u16,
    pub nlists: u32,
    pub relation_nprobe: u32,
    pub session_nprobe: Option<u32>,
    pub effective_nprobe: u32,
    pub effective_nprobe_source: &'static str,
    pub relation_rerank_width: i32,
    pub relation_posting_slack_percent: i32,
    pub session_rerank_width: Option<i32>,
    pub effective_rerank_width: i32,
    pub effective_rerank_width_source: &'static str,
    pub training_sample_rows: u32,
    pub training_version: u16,
    pub storage_format: &'static str,
    pub rerank: &'static str,
    pub total_live_tuples: u64,
    pub total_dead_tuples: u64,
    pub inserted_since_build: u64,
    pub changed_row_fraction: f64,
    pub average_list_live_count: f64,
    pub max_list_live_count: u64,
    pub list_imbalance_ratio: f64,
    pub empty_lists: u32,
    pub reindex_recommended: bool,
    pub reindex_reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndexPageOwnershipSnapshot {
    pub block_number: u32,
    pub line_pointer_count: u16,
    pub unused_line_pointers: u16,
    pub non_posting_tuples: u16,
    pub posting_tuples: u16,
    pub live_posting_tuples: u16,
    pub deleted_posting_tuples: u16,
    pub heap_tid_refs: u32,
    pub distinct_lists: u32,
    pub min_list_id: Option<u32>,
    pub max_list_id: Option<u32>,
    pub list_ids: String,
}

#[derive(Debug, Default)]
struct DirectoryDriftSummary {
    live_count_sum: u64,
    max_live_count: u64,
    empty_lists: u32,
}

pub(crate) unsafe fn index_drift_snapshot(index_relation: pg_sys::Relation) -> IndexDriftSnapshot {
    let metadata = unsafe { page::read_metadata_page(index_relation) };
    let directory = unsafe { directory_drift_summary(index_relation, &metadata) };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };

    let total_live_tuples = metadata.total_live_tuples;
    let changed_row_count = metadata
        .inserted_since_build
        .saturating_add(metadata.total_dead_tuples);
    let changed_row_denominator = total_live_tuples.saturating_add(metadata.total_dead_tuples);
    let changed_row_fraction = if changed_row_denominator == 0 {
        0.0
    } else {
        changed_row_count as f64 / changed_row_denominator as f64
    };
    let average_list_live_count = if metadata.nlists == 0 {
        0.0
    } else {
        directory.live_count_sum as f64 / metadata.nlists as f64
    };
    let list_imbalance_ratio = if average_list_live_count == 0.0 {
        0.0
    } else {
        directory.max_live_count as f64 / average_list_live_count
    };
    let changed_reindex = changed_row_fraction >= REINDEX_CHANGED_ROW_FRACTION_THRESHOLD;
    let imbalance_reindex = list_imbalance_ratio > REINDEX_LIST_IMBALANCE_THRESHOLD;
    let reindex_reason = match (changed_reindex, imbalance_reindex) {
        (true, true) => "changed_rows,list_imbalance",
        (true, false) => "changed_rows",
        (false, true) => "list_imbalance",
        (false, false) => "none",
    };

    IndexDriftSnapshot {
        block_count,
        nlists: metadata.nlists,
        total_live_tuples,
        total_dead_tuples: metadata.total_dead_tuples,
        inserted_since_build: metadata.inserted_since_build,
        changed_row_fraction,
        average_list_live_count,
        max_list_live_count: directory.max_live_count,
        list_imbalance_ratio,
        empty_lists: directory.empty_lists,
        reindex_recommended: changed_reindex || imbalance_reindex,
        reindex_reason,
        changed_row_reindex_threshold: REINDEX_CHANGED_ROW_FRACTION_THRESHOLD,
        list_imbalance_reindex_threshold: REINDEX_LIST_IMBALANCE_THRESHOLD,
    }
}

pub(crate) unsafe fn index_admin_snapshot(index_relation: pg_sys::Relation) -> IndexAdminSnapshot {
    let metadata = unsafe { page::read_metadata_page(index_relation) };
    let index_options = unsafe { options::relation_options(index_relation) };
    let nprobe = options::resolve_scan_nprobe(metadata.nlists, metadata.nprobe);
    let rerank_width = options::resolve_scan_rerank_width(index_options.rerank_width);
    let drift = unsafe { index_drift_snapshot(index_relation) };
    let reltuples = unsafe { (*(*index_relation).rd_rel).reltuples } as f64;

    IndexAdminSnapshot {
        block_count: drift.block_count,
        index_pages: f64::from(drift.block_count),
        reltuples,
        dimensions: metadata.dimensions,
        nlists: metadata.nlists,
        relation_nprobe: nprobe.relation_nprobe,
        session_nprobe: nprobe.session_nprobe,
        effective_nprobe: nprobe.effective_nprobe,
        effective_nprobe_source: nprobe.source,
        relation_rerank_width: rerank_width.relation_rerank_width,
        relation_posting_slack_percent: index_options.posting_slack_percent,
        session_rerank_width: rerank_width.session_rerank_width,
        effective_rerank_width: rerank_width.effective_rerank_width,
        effective_rerank_width_source: rerank_width.source,
        training_sample_rows: metadata.training_sample_rows,
        training_version: metadata.training_version,
        storage_format: metadata.storage_format.reloption_name(),
        rerank: metadata.rerank.reloption_name(),
        total_live_tuples: drift.total_live_tuples,
        total_dead_tuples: drift.total_dead_tuples,
        inserted_since_build: drift.inserted_since_build,
        changed_row_fraction: drift.changed_row_fraction,
        average_list_live_count: drift.average_list_live_count,
        max_list_live_count: drift.max_list_live_count,
        list_imbalance_ratio: drift.list_imbalance_ratio,
        empty_lists: drift.empty_lists,
        reindex_recommended: drift.reindex_recommended,
        reindex_reason: drift.reindex_reason,
    }
}

pub(crate) unsafe fn index_page_ownership(
    index_relation: pg_sys::Relation,
) -> Vec<IndexPageOwnershipSnapshot> {
    let metadata = unsafe { page::read_metadata_page(index_relation) };
    let quantizer = IvfQuantizer::resolve_with_pq_group_size(
        metadata.storage_format,
        usize::from(metadata.dimensions),
        metadata_pq_group_size(&metadata),
    )
    .unwrap_or_else(|err| pgrx::error!("{err}"));
    let summaries = unsafe {
        page::debug_ivf_posting_block_summaries(index_relation, quantizer.payload_len())
            .unwrap_or_else(|err| pgrx::error!("{err}"))
    };
    summaries
        .into_iter()
        .map(|summary| {
            let min_list_id = summary.list_ids.first().copied();
            let max_list_id = summary.list_ids.last().copied();
            let distinct_lists = u32::try_from(summary.list_ids.len())
                .unwrap_or_else(|_| pgrx::error!("ec_ivf block list-id count exceeds u32"));
            let list_ids = summary
                .list_ids
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",");
            IndexPageOwnershipSnapshot {
                block_number: summary.block_number,
                line_pointer_count: summary.line_pointer_count,
                unused_line_pointers: summary.unused_line_pointers,
                non_posting_tuples: summary.non_posting_tuples,
                posting_tuples: summary.posting_tuples,
                live_posting_tuples: summary.live_posting_tuples,
                deleted_posting_tuples: summary.deleted_posting_tuples,
                heap_tid_refs: summary.heap_tid_refs,
                distinct_lists,
                min_list_id,
                max_list_id,
                list_ids,
            }
        })
        .collect()
}

unsafe fn directory_drift_summary(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
) -> DirectoryDriftSummary {
    if metadata.directory_head == ItemPointer::INVALID {
        if metadata.total_live_tuples != 0 {
            pgrx::error!("ec_ivf metadata has live tuples but no directory head");
        }
        return DirectoryDriftSummary {
            live_count_sum: 0,
            max_live_count: 0,
            empty_lists: metadata.nlists,
        };
    }

    let mut next_tid = metadata.directory_head;
    let mut summary = DirectoryDriftSummary::default();
    for expected_list_id in 0..metadata.nlists {
        let (directory, following_tid) =
            unsafe { page::read_ivf_list_directory_and_next(index_relation, next_tid) }
                .unwrap_or_else(|e| pgrx::error!("{e}"));
        if directory.list_id != expected_list_id {
            pgrx::error!(
                "ec_ivf directory order mismatch: got list {}, expected {}",
                directory.list_id,
                expected_list_id
            );
        }
        summary.live_count_sum = summary
            .live_count_sum
            .checked_add(directory.live_count)
            .unwrap_or_else(|| pgrx::error!("ec_ivf directory live count overflow"));
        summary.max_live_count = summary.max_live_count.max(directory.live_count);
        if directory.live_count == 0 {
            summary.empty_lists = summary
                .empty_lists
                .checked_add(1)
                .unwrap_or_else(|| pgrx::error!("ec_ivf empty-list count overflow"));
        }
        next_tid = following_tid;
    }

    summary
}

fn metadata_pq_group_size(metadata: &page::MetadataPage) -> Option<usize> {
    if metadata.storage_format == options::StorageFormat::PqFastScan && metadata.pq_group_size > 0 {
        Some(usize::from(metadata.pq_group_size))
    } else {
        None
    }
}
