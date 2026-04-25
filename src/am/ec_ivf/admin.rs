use pgrx::pg_sys;

use super::page;
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

#[derive(Debug, Default)]
struct DirectoryDriftSummary {
    live_count_sum: u64,
    max_live_count: u64,
    empty_lists: u32,
}

pub(crate) unsafe fn index_drift_snapshot(
    index_relation: pg_sys::Relation,
) -> IndexDriftSnapshot {
    let metadata = unsafe { page::read_metadata_page(index_relation) };
    let directory = unsafe { directory_drift_summary(index_relation, &metadata) };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
        )
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
    let changed_reindex =
        changed_row_fraction >= REINDEX_CHANGED_ROW_FRACTION_THRESHOLD;
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
