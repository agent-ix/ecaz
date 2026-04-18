use std::ptr;

use pgrx::{itemptr::item_pointer_get_both, pg_sys, PgBox};

use super::{graph, options, page, wal, P_NEW, TQHNSW_PLANNER_SCAN_ENABLED};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LiveEntryCandidate {
    pub tid: page::ItemPointer,
    pub level: u8,
}

pub(super) unsafe fn initialize_metadata_page(
    index_relation: pg_sys::Relation,
    metadata: page::MetadataPage,
) {
    let existing_blocks = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let target_block = if existing_blocks == 0 {
        P_NEW
    } else {
        page::METADATA_BLOCK_NUMBER
    };
    let read_mode = if target_block == P_NEW {
        pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK
    } else {
        pg_sys::ReadBufferMode::RBM_NORMAL
    };
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            target_block,
            read_mode,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to allocate metadata buffer");
    }

    if target_block != P_NEW {
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let metadata_bytes = metadata.encode();
    let special_size = (metadata_bytes.len() + 7) & !7;
    unsafe { pg_sys::PageInit(page, page_size, special_size) };
    unsafe { write_metadata_bytes(page, &metadata_bytes) };

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

unsafe fn write_metadata_bytes(page: pg_sys::Page, metadata_bytes: &[u8]) {
    let page_contents = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    unsafe {
        ptr::copy_nonoverlapping(metadata_bytes.as_ptr(), page_contents, metadata_bytes.len());
    }
}

pub(super) unsafe fn update_metadata_page(
    index_relation: pg_sys::Relation,
    metadata: page::MetadataPage,
) {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            page::METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to open metadata buffer");
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let metadata_bytes = metadata.encode();
    let special_size = (metadata_bytes.len() + 7) & !7;
    unsafe { pg_sys::PageInit(page, page_size, special_size) };
    unsafe { write_metadata_bytes(page, &metadata_bytes) };
    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

pub(super) unsafe fn with_locked_metadata_page<T>(
    index_relation: pg_sys::Relation,
    f: impl FnOnce(&mut page::MetadataPage) -> T,
) -> T {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            page::METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to open metadata buffer");
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let raw_page = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let page_bytes = unsafe { std::slice::from_raw_parts(raw_page, page_size) };
    let mut metadata =
        page::MetadataPage::decode_page(page_bytes).expect("metadata page should decode");
    let result = f(&mut metadata);

    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let metadata_bytes = metadata.encode();
    let special_size = (metadata_bytes.len() + 7) & !7;
    unsafe { pg_sys::PageInit(page, page_size, special_size) };
    unsafe { write_metadata_bytes(page, &metadata_bytes) };
    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    result
}

pub(super) unsafe fn tqhnsw_noop_vacuum_stats(
    index_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    let stats = if stats.is_null() {
        unsafe { PgBox::<pg_sys::IndexBulkDeleteResult>::alloc0().into_pg() }
    } else {
        stats
    };

    unsafe {
        (*stats).num_pages = pg_sys::RelationGetNumberOfBlocksInFork(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
        );
        (*stats).estimated_count = false;
        (*stats).num_index_tuples = count_element_tuples(index_relation) as f64;
    }

    stats
}

pub(super) unsafe fn count_element_tuples(index_relation: pg_sys::Relation) -> usize {
    let metadata = unsafe { read_metadata_page(index_relation) };
    let storage =
        unsafe { graph::GraphStorageDescriptor::from_index_relation(index_relation, &metadata) }
            .unwrap_or_else(|e| pgrx::error!("{e}"));
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let mut count = 0_usize;

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
        let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let line_pointer_count = page_line_pointer_count(page_ptr);

        for offset in 1..=line_pointer_count {
            let item_id = unsafe { &*page_item_id(page_ptr, offset) };
            if item_id.lp_flags() == 0 {
                continue;
            }

            let tuple_offset = item_id.lp_off() as usize;
            let tuple_len = item_id.lp_len() as usize;
            if tuple_offset + tuple_len > page_size {
                pgrx::error!(
                    "tqhnsw found invalid tuple bounds while counting vacuum tuples on block {block_number}"
                );
            }

            let tuple_bytes =
                unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
            match storage {
                graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                    if tuple_bytes.first().copied() == Some(page::TQ_ELEMENT_TAG) {
                        let element = page::TqElementTuple::decode(tuple_bytes, code_len)
                            .unwrap_or_else(|e| {
                                pgrx::error!(
                                    "tqhnsw failed to decode element tuple while counting: {e}"
                                )
                            });
                        if !element.deleted && !element.heaptids.is_empty() {
                            count += 1;
                        }
                    }
                }
                graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
                    if tuple_bytes.first().copied() == Some(page::TQ_TURBO_HOT_TAG) {
                        let element =
                            page::TqTurboHotTuple::decode(tuple_bytes, layout.binary_word_count)
                                .unwrap_or_else(|e| {
                                    pgrx::error!(
                                        "tqhnsw failed to decode TurboQuant V3 tuple while counting: {e}"
                                    )
                                });
                        if !element.deleted && !element.heaptids.is_empty() {
                            count += 1;
                        }
                    }
                }
                graph::GraphStorageDescriptor::PqFastScan(layout) => {
                    if tuple_bytes.first().copied() == Some(page::TQ_GROUPED_HOT_TAG) {
                        let element = page::TqGroupedHotTuple::decode(
                            tuple_bytes,
                            layout.binary_word_count,
                            layout.search_code_len,
                        )
                        .unwrap_or_else(|e| {
                            pgrx::error!(
                                "tqhnsw failed to decode grouped hot tuple while counting: {e}"
                            )
                        });
                        if !element.deleted && !element.heaptids.is_empty() {
                            count += 1;
                        }
                    }
                }
            }
        }

        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }

    count
}

pub(super) unsafe fn highest_level_live_entry_candidate(
    index_relation: pg_sys::Relation,
    storage: graph::GraphStorageDescriptor,
) -> Option<LiveEntryCandidate> {
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let mut best_level = None;
    let mut best = None;

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
        let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let line_pointer_count = page_line_pointer_count(page_ptr);

        for offset in 1..=line_pointer_count {
            let item_id = unsafe { &*page_item_id(page_ptr, offset) };
            if item_id.lp_flags() == 0 {
                continue;
            }

            let tuple_offset = item_id.lp_off() as usize;
            let tuple_len = item_id.lp_len() as usize;
            if tuple_offset + tuple_len > page_size {
                pgrx::error!(
                    "tqhnsw found invalid tuple bounds while selecting a live entry candidate on block {block_number}"
                );
            }

            let tuple_bytes =
                unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
            let tid = page::ItemPointer {
                block_number,
                offset_number: offset,
            };
            let candidate = match storage {
                graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                    if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
                        None
                    } else {
                        let element = page::TqElementTuple::decode(tuple_bytes, code_len)
                            .unwrap_or_else(|e| {
                                pgrx::error!(
                                    "tqhnsw failed to decode element tuple while selecting a live entry candidate: {e}"
                                )
                            });
                        (!element.deleted && !element.heaptids.is_empty()).then_some(
                            LiveEntryCandidate {
                                tid,
                                level: element.level,
                            },
                        )
                    }
                }
                graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
                    if tuple_bytes.first().copied() != Some(page::TQ_TURBO_HOT_TAG) {
                        None
                    } else {
                        let element =
                            page::TqTurboHotTuple::decode(tuple_bytes, layout.binary_word_count)
                                .unwrap_or_else(|e| {
                                    pgrx::error!(
                                        "tqhnsw failed to decode TurboQuant V3 tuple while selecting a live entry candidate: {e}"
                                    )
                                });
                        (!element.deleted && !element.heaptids.is_empty()).then_some(
                            LiveEntryCandidate {
                                tid,
                                level: element.level,
                            },
                        )
                    }
                }
                graph::GraphStorageDescriptor::PqFastScan(layout) => {
                    if tuple_bytes.first().copied() != Some(page::TQ_GROUPED_HOT_TAG) {
                        None
                    } else {
                        let element = page::TqGroupedHotTuple::decode(
                            tuple_bytes,
                            layout.binary_word_count,
                            layout.search_code_len,
                        )
                        .unwrap_or_else(|e| {
                            pgrx::error!(
                                "tqhnsw failed to decode grouped hot tuple while selecting a live entry candidate: {e}"
                            )
                        });
                        (!element.deleted && !element.heaptids.is_empty()).then_some(
                            LiveEntryCandidate {
                                tid,
                                level: element.level,
                            },
                        )
                    }
                }
            };
            if let Some(candidate) = candidate {
                match best_level {
                    None => {
                        best_level = Some(candidate.level);
                        best = Some(candidate);
                    }
                    Some(level) if candidate.level > level => {
                        best_level = Some(candidate.level);
                        best = Some(candidate);
                    }
                    Some(level)
                        if candidate.level == level
                            && match best {
                                None => true,
                                Some(existing) => {
                                    (candidate.tid.block_number, candidate.tid.offset_number)
                                        < (existing.tid.block_number, existing.tid.offset_number)
                                }
                            } =>
                    {
                        best = Some(candidate);
                    }
                    Some(_) => {}
                }
            }
        }

        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }

    best
}

pub(super) unsafe fn page_item_id(page_ptr: *mut u8, offset: u16) -> *const pg_sys::ItemIdData {
    unsafe {
        page_ptr
            .add(
                page::PAGE_HEADER_BYTES + ((offset - 1) as usize * size_of::<pg_sys::ItemIdData>()),
            )
            .cast::<pg_sys::ItemIdData>()
    }
}

pub(super) fn page_line_pointer_count(page_ptr: *mut u8) -> u16 {
    let page_header = page_ptr.cast::<pg_sys::PageHeaderData>();
    ((unsafe { (*page_header).pd_lower } as usize - size_of::<pg_sys::PageHeaderData>())
        / size_of::<pg_sys::ItemIdData>()) as u16
}

pub(super) unsafe fn decode_heap_tid(tid: pg_sys::ItemPointer) -> page::ItemPointer {
    if tid.is_null() {
        pgrx::error!("tqhnsw ambuild received a null heap tid");
    }
    let (block_number, offset_number) = item_pointer_get_both(unsafe { *tid });
    page::ItemPointer {
        block_number,
        offset_number,
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DebugIndexDataPage {
    pub block_number: u32,
    pub tuples: Vec<Vec<u8>>,
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_index_pages(
    index_oid: pg_sys::Oid,
) -> (u32, page::MetadataPage, Vec<DebugIndexDataPage>) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };

    let metadata = unsafe { read_metadata_page(index_relation) };
    let mut data_pages = Vec::new();
    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        data_pages.push(unsafe { read_data_page(index_relation, block_number) });
    }

    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (block_count, metadata, data_pages)
}

pub(super) unsafe fn read_metadata_page(index_relation: pg_sys::Relation) -> page::MetadataPage {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            page::METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let raw_page = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let page_bytes = unsafe { std::slice::from_raw_parts(raw_page, page_size) };
    let metadata =
        page::MetadataPage::decode_page(page_bytes).expect("metadata page should decode");
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    metadata
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IndexAdminSnapshot {
    pub block_count: u32,
    pub total_live_nodes: usize,
    pub inserted_since_rebuild: usize,
    pub insert_drift_fraction: f64,
    pub relation_ef_search: i32,
    pub session_ef_search: Option<i32>,
    pub effective_ef_search: i32,
    pub effective_source: &'static str,
    pub planner_scan_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndexExplainSnapshot {
    pub planner_scan_enabled: bool,
    pub ordered_scan_ready: bool,
    pub planner_gate_reason: &'static str,
    pub ordering_strategy: i32,
    pub ordering_compare_type: &'static str,
    pub pg18_strategy_translation_ready: bool,
    pub explain_option_name: &'static str,
    pub pg18_custom_explain_option_ready: bool,
    pub pg18_explain_per_node_hook_ready: bool,
    pub effective_ef_search: i32,
    pub effective_source: &'static str,
    pub total_live_nodes: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IndexCostSnapshot {
    pub planner_scan_enabled: bool,
    pub planner_gate_reason: &'static str,
    pub relation_ef_search: i32,
    pub session_ef_search: Option<i32>,
    pub effective_ef_search: i32,
    pub effective_source: &'static str,
    pub m: i32,
    pub dimensions: u16,
    pub max_level: u8,
    pub resolved_tree_height: f64,
    pub tree_height_source: &'static str,
    pub pg18_tree_height_callback_ready: bool,
    pub index_pages: f64,
    pub reltuples: f64,
    pub random_page_cost: f64,
    pub seq_page_cost: f64,
    pub cpu_operator_cost: f64,
    pub modeled_startup_cost: f64,
    pub modeled_total_cost: f64,
    pub modeled_selectivity: f64,
    pub modeled_correlation: f64,
    pub gated_startup_cost: f64,
    pub gated_total_cost: f64,
    pub gated_selectivity: f64,
    pub gated_correlation: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Pg18DiagnosticsSnapshot {
    pub explain_option_name: &'static str,
    pub stats_function_name: &'static str,
    pub pg18_custom_explain_option_ready: bool,
    pub pg18_explain_per_node_hook_ready: bool,
    pub pg18_pgstat_kind_ready: bool,
    pub pg18_stats_sql_function_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReadStreamSnapshot {
    pub graph_stream_mode: &'static str,
    pub linear_stream_mode: &'static str,
    pub graph_stream_access_pattern: &'static str,
    pub linear_stream_access_pattern: &'static str,
    pub pg18_callback_surface_ready: bool,
    pub pg18_scan_wiring_ready: bool,
    pub pg18_vacuum_wiring_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlannerIntegrationSnapshot {
    pub planner_scan_enabled: bool,
    pub ordered_scan_ready: bool,
    pub runtime_ordered_scan_ready: bool,
    pub planner_cost_model_ready: bool,
    pub planner_cost_callback_live: bool,
    pub pg18_callback_surface_ready: bool,
    pub pg18_diagnostics_surface_ready: bool,
    pub pg18_read_stream_surface_ready: bool,
    pub effective_ef_search: i32,
    pub effective_source: &'static str,
    pub planner_gate_reason: &'static str,
    pub next_runtime_blocker: &'static str,
    pub next_pg18_blocker: &'static str,
}

pub(crate) unsafe fn index_admin_snapshot(index_relation: pg_sys::Relation) -> IndexAdminSnapshot {
    let relation_options = unsafe { options::relation_options(index_relation) };
    let tuning = options::resolve_scan_tuning(&relation_options);
    let metadata = unsafe { read_metadata_page(index_relation) };
    let total_live_nodes = unsafe { count_element_tuples(index_relation) };
    let inserted_since_rebuild =
        usize::try_from(metadata.inserted_since_rebuild).unwrap_or_else(|_| {
            pgrx::error!(
                "tqhnsw metadata inserted_since_rebuild {} exceeds usize",
                metadata.inserted_since_rebuild
            )
        });
    IndexAdminSnapshot {
        block_count: unsafe {
            pg_sys::RelationGetNumberOfBlocksInFork(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
            )
        },
        total_live_nodes,
        inserted_since_rebuild,
        insert_drift_fraction: insert_drift_fraction(total_live_nodes, inserted_since_rebuild),
        relation_ef_search: tuning.relation_ef_search,
        session_ef_search: tuning.session_ef_search,
        effective_ef_search: tuning.effective_ef_search,
        effective_source: match tuning.source {
            options::EfSearchSource::Relation => "relation",
            options::EfSearchSource::Session => "session",
        },
        planner_scan_enabled: TQHNSW_PLANNER_SCAN_ENABLED,
    }
}

fn insert_drift_fraction(total_live_nodes: usize, inserted_since_rebuild: usize) -> f64 {
    if total_live_nodes == 0 {
        return 0.0;
    }
    inserted_since_rebuild as f64 / total_live_nodes as f64
}

pub(crate) unsafe fn index_explain_snapshot(
    index_relation: pg_sys::Relation,
) -> IndexExplainSnapshot {
    let admin = unsafe { index_admin_snapshot(index_relation) };
    let translation = super::cost::strategy_translation_snapshot();
    let explain = super::explain::explain_option_snapshot();
    IndexExplainSnapshot {
        planner_scan_enabled: admin.planner_scan_enabled,
        ordered_scan_ready: true,
        planner_gate_reason:
            "planner scan selection is live: FR-020 cost model active (ADR-011 superseded)",
        ordering_strategy: translation.ordering_strategy,
        ordering_compare_type: translation.ordering_compare_type.as_str(),
        pg18_strategy_translation_ready: translation.pg18_callback_ready,
        explain_option_name: explain.option_name,
        pg18_custom_explain_option_ready: explain.pg18_custom_explain_option_ready,
        pg18_explain_per_node_hook_ready: explain.pg18_explain_per_node_hook_ready,
        effective_ef_search: admin.effective_ef_search,
        effective_source: admin.effective_source,
        total_live_nodes: admin.total_live_nodes,
    }
}

pub(crate) unsafe fn index_cost_snapshot(index_relation: pg_sys::Relation) -> IndexCostSnapshot {
    let relation_options = unsafe { options::relation_options(index_relation) };
    let tuning = options::resolve_scan_tuning(&relation_options);
    let metadata = unsafe { read_metadata_page(index_relation) };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let index_pages = f64::from(block_count);
    let reltuples = unsafe { (*(*index_relation).rd_rel).reltuples } as f64;
    let tree_height = super::cost::metadata_fallback_tree_height(metadata.max_level);
    let constants = unsafe { super::cost::current_planner_cost_constants() };
    // Block 0 is always the metadata page; an empty index has block_count == 1.
    // FR-020's "Empty index (0 data pages)" gate must trip on
    // `block_count <= FIRST_DATA_BLOCK_NUMBER`, not on `index_pages <= 0`.
    let modeled = if block_count <= super::page::FIRST_DATA_BLOCK_NUMBER {
        super::cost::gated_planner_cost_estimate(index_pages)
    } else {
        super::cost::estimate_planner_cost(
            super::cost::PlannerCostInputs {
                index_pages,
                reltuples,
                m: relation_options.m,
                ef_search: tuning.effective_ef_search,
                dimensions: metadata.dimensions,
                tree_height: tree_height.tree_height,
            },
            constants,
        )
    };
    let gated = super::cost::gated_planner_cost_estimate(index_pages);

    IndexCostSnapshot {
        planner_scan_enabled: TQHNSW_PLANNER_SCAN_ENABLED,
        planner_gate_reason:
            "planner cost activation is live: FR-020 cost model replaces ADR-011 override",
        relation_ef_search: tuning.relation_ef_search,
        session_ef_search: tuning.session_ef_search,
        effective_ef_search: tuning.effective_ef_search,
        effective_source: match tuning.source {
            options::EfSearchSource::Relation => "relation",
            options::EfSearchSource::Session => "session",
        },
        m: relation_options.m,
        dimensions: metadata.dimensions,
        max_level: metadata.max_level,
        resolved_tree_height: tree_height.tree_height,
        tree_height_source: tree_height.source,
        pg18_tree_height_callback_ready: tree_height.pg18_callback_ready,
        index_pages,
        reltuples,
        random_page_cost: constants.random_page_cost,
        seq_page_cost: constants.seq_page_cost,
        cpu_operator_cost: constants.cpu_operator_cost,
        modeled_startup_cost: modeled.startup_cost,
        modeled_total_cost: modeled.total_cost,
        modeled_selectivity: modeled.selectivity,
        modeled_correlation: modeled.correlation,
        gated_startup_cost: gated.startup_cost,
        gated_total_cost: gated.total_cost,
        gated_selectivity: gated.selectivity,
        gated_correlation: gated.correlation,
    }
}

pub(crate) fn pg18_diagnostics_snapshot() -> Pg18DiagnosticsSnapshot {
    let explain = super::explain::explain_option_snapshot();
    let stats = super::stats::stats_snapshot();
    Pg18DiagnosticsSnapshot {
        explain_option_name: explain.option_name,
        stats_function_name: stats.function_name,
        pg18_custom_explain_option_ready: explain.pg18_custom_explain_option_ready,
        pg18_explain_per_node_hook_ready: explain.pg18_explain_per_node_hook_ready,
        pg18_pgstat_kind_ready: stats.pg18_pgstat_kind_ready,
        pg18_stats_sql_function_ready: stats.pg18_sql_function_ready,
    }
}

pub(crate) fn read_stream_snapshot() -> ReadStreamSnapshot {
    let stream = super::stream::stream_snapshot();
    ReadStreamSnapshot {
        graph_stream_mode: stream.graph_stream_mode,
        linear_stream_mode: stream.linear_stream_mode,
        graph_stream_access_pattern: stream.graph_stream_access_pattern,
        linear_stream_access_pattern: stream.linear_stream_access_pattern,
        pg18_callback_surface_ready: stream.pg18_callback_surface_ready,
        pg18_scan_wiring_ready: stream.pg18_scan_wiring_ready,
        pg18_vacuum_wiring_ready: stream.pg18_vacuum_wiring_ready,
    }
}

pub(crate) unsafe fn planner_integration_snapshot(
    index_relation: pg_sys::Relation,
) -> PlannerIntegrationSnapshot {
    let admin = unsafe { index_admin_snapshot(index_relation) };
    let explain = unsafe { index_explain_snapshot(index_relation) };
    let cost = unsafe { index_cost_snapshot(index_relation) };
    let diagnostics = pg18_diagnostics_snapshot();
    let stream = read_stream_snapshot();

    PlannerIntegrationSnapshot {
        planner_scan_enabled: explain.planner_scan_enabled,
        ordered_scan_ready: explain.ordered_scan_ready,
        runtime_ordered_scan_ready: true,
        planner_cost_model_ready: true,
        planner_cost_callback_live: true,
        pg18_callback_surface_ready: cost.pg18_tree_height_callback_ready
            && explain.pg18_strategy_translation_ready,
        pg18_diagnostics_surface_ready: diagnostics.pg18_custom_explain_option_ready
            && diagnostics.pg18_explain_per_node_hook_ready
            && diagnostics.pg18_pgstat_kind_ready
            && diagnostics.pg18_stats_sql_function_ready,
        pg18_read_stream_surface_ready: stream.pg18_callback_surface_ready
            && stream.pg18_scan_wiring_ready
            && stream.pg18_vacuum_wiring_ready,
        effective_ef_search: admin.effective_ef_search,
        effective_source: admin.effective_source,
        planner_gate_reason: explain.planner_gate_reason,
        next_runtime_blocker:
            "no merged runtime blocker remains on main; post-vacuum benchmark/reporting is next",
        next_pg18_blocker:
            "pgrx pg18 feature support and callback bindings are not yet implemented",
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DebugPlannerTuningSnapshot {
    pub relation_ef_search: i32,
    pub session_ef_search: Option<i32>,
    pub effective_ef_search: i32,
    pub effective_source: &'static str,
    pub planner_scan_enabled: bool,
}

#[cfg(any(test, feature = "pg_test"))]
fn planner_tuning_snapshot(index_relation: pg_sys::Relation) -> DebugPlannerTuningSnapshot {
    let snapshot = unsafe { index_admin_snapshot(index_relation) };
    DebugPlannerTuningSnapshot {
        relation_ef_search: snapshot.relation_ef_search,
        session_ef_search: snapshot.session_ef_search,
        effective_ef_search: snapshot.effective_ef_search,
        effective_source: snapshot.effective_source,
        planner_scan_enabled: snapshot.planner_scan_enabled,
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_planner_tuning_snapshot(
    index_oid: pg_sys::Oid,
) -> DebugPlannerTuningSnapshot {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let snapshot = planner_tuning_snapshot(index_relation);
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    snapshot
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn read_data_page(
    index_relation: pg_sys::Relation,
    block_number: u32,
) -> DebugIndexDataPage {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let raw_page = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let page_header = raw_page.cast::<pg_sys::PageHeaderData>();
    let line_pointer_count = ((unsafe { (*page_header).pd_lower } as usize
        - size_of::<pg_sys::PageHeaderData>())
        / size_of::<pg_sys::ItemIdData>()) as u16;

    let mut tuples = Vec::with_capacity(line_pointer_count as usize);
    for offset in 1..=line_pointer_count {
        let item_id_ptr = unsafe {
            raw_page
                .add(
                    page::PAGE_HEADER_BYTES
                        + ((offset - 1) as usize * size_of::<pg_sys::ItemIdData>()),
                )
                .cast::<pg_sys::ItemIdData>()
        };
        let item_id = unsafe { &*item_id_ptr };
        if item_id.lp_flags() == 0 {
            continue;
        }
        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > page_size {
            pgrx::error!("tqhnsw debug read found invalid tuple bounds on block {block_number}");
        }
        tuples.push(
            unsafe { std::slice::from_raw_parts(raw_page.add(tuple_offset), tuple_len) }.to_vec(),
        );
    }

    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    DebugIndexDataPage {
        block_number,
        tuples,
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_index_metadata(
    index_oid: pg_sys::Oid,
) -> (u32, i32, i32, page::MetadataPage) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let options = unsafe { super::options::relation_options(index_relation) };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let metadata = unsafe { read_metadata_page(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    (block_count, options.m, options.ef_construction, metadata)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_update_index_metadata(
    index_oid: pg_sys::Oid,
    metadata: page::MetadataPage,
) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    unsafe { update_metadata_page(index_relation, metadata) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_vacuum_stats(index_oid: pg_sys::Oid) -> pg_sys::IndexBulkDeleteResult {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let mut info = PgBox::<pg_sys::IndexVacuumInfo>::alloc0();
    info.index = index_relation;
    let info_ptr = (&mut *info) as *mut pg_sys::IndexVacuumInfo;

    let stats = unsafe {
        super::vacuum::tqhnsw_ambulkdelete(info_ptr, ptr::null_mut(), None, ptr::null_mut())
    };
    let stats = unsafe { super::vacuum::tqhnsw_amvacuumcleanup(info_ptr, stats) };
    let result = unsafe { *stats };

    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result
}
