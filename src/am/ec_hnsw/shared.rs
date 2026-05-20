use std::ptr;

use pgrx::{itemptr::item_pointer_get_both, pg_sys, PgBox};

#[cfg(feature = "pg18")]
use super::stream;
use super::{graph, options, page, EC_HNSW_PLANNER_SCAN_ENABLED, P_NEW};
use crate::storage::buffer_guard::LockedBufferGuard;
#[cfg(any(test, feature = "pg_test"))]
use crate::storage::relation_guard::IndexRelationGuard;
use crate::storage::wal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LiveEntryCandidate {
    pub tid: page::ItemPointer,
    pub level: u8,
}

pub(super) unsafe fn initialize_metadata_page(
    index_relation: pg_sys::Relation,
    metadata: page::MetadataPage,
) {
    let existing_blocks = hnsw_main_block_count(index_relation);
    let target_block = if existing_blocks == 0 {
        P_NEW
    } else {
        page::METADATA_BLOCK_NUMBER
    };
    let buffer = if target_block == P_NEW {
        LockedBufferGuard::read_main_locked(
            index_relation,
            target_block,
            pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
        )
    } else {
        LockedBufferGuard::read_main(
            index_relation,
            target_block,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
    }
    .unwrap_or_else(|| pgrx::error!("ec_hnsw failed to allocate metadata buffer"));
    let page_size = buffer.page_size();
    rewrite_metadata_buffer(index_relation, &buffer, page_size, metadata);
}

unsafe fn write_metadata_bytes(page: pg_sys::Page, metadata_bytes: &[u8]) {
    // SAFETY: The caller initialized the page with a special area large enough
    // to hold `metadata_bytes`.
    let page_contents = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    // SAFETY: Source and destination are non-overlapping and the destination
    // special area was sized by the caller.
    unsafe {
        ptr::copy_nonoverlapping(metadata_bytes.as_ptr(), page_contents, metadata_bytes.len());
    }
}

fn hnsw_main_block_count(index_relation: pg_sys::Relation) -> pg_sys::BlockNumber {
    // SAFETY: Callers hold a live index relation while copying its current
    // main-fork block count.
    unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    }
}

fn read_main_buffer(
    index_relation: pg_sys::Relation,
    block_number: pg_sys::BlockNumber,
    mode: pg_sys::ReadBufferMode::Type,
    lock_mode: i32,
    context: &str,
) -> LockedBufferGuard {
    // SAFETY: Callers derive block numbers from this live relation and choose a
    // PostgreSQL buffer mode/lock appropriate to the page access.
    unsafe { LockedBufferGuard::read_main(index_relation, block_number, mode, lock_mode) }
        .unwrap_or_else(|| pgrx::error!("ec_hnsw failed to open data buffer while {context}"))
}

pub(super) unsafe fn update_metadata_page(
    index_relation: pg_sys::Relation,
    metadata: page::MetadataPage,
) {
    let buffer = LockedBufferGuard::read_main(
        index_relation,
        page::METADATA_BLOCK_NUMBER,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
    )
    .unwrap_or_else(|| pgrx::error!("ec_hnsw failed to open metadata buffer"));
    let page_size = buffer.page_size();
    rewrite_metadata_buffer(index_relation, &buffer, page_size, metadata);
}

pub(super) unsafe fn with_locked_metadata_page<T>(
    index_relation: pg_sys::Relation,
    f: impl FnOnce(&mut page::MetadataPage) -> T,
) -> T {
    let buffer = LockedBufferGuard::read_main(
        index_relation,
        page::METADATA_BLOCK_NUMBER,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
    )
    .unwrap_or_else(|| pgrx::error!("ec_hnsw failed to open metadata buffer"));
    let raw_page = buffer.page().cast::<u8>();
    let page_size = buffer.page_size();
    // SAFETY: The buffer guard pins the metadata page for the duration of this
    // borrow and `page_size` bounds the slice.
    let page_bytes = unsafe { std::slice::from_raw_parts(raw_page, page_size) };
    let mut metadata =
        page::MetadataPage::decode_page(page_bytes).expect("metadata page should decode");
    let result = f(&mut metadata);

    rewrite_metadata_buffer(index_relation, &buffer, page_size, metadata);
    result
}

fn rewrite_metadata_buffer(
    index_relation: pg_sys::Relation,
    buffer: &LockedBufferGuard,
    page_size: usize,
    metadata: page::MetadataPage,
) {
    let metadata_bytes = metadata.encode();
    let special_size = (metadata_bytes.len() + 7) & !7;
    // SAFETY: Callers supply a live index relation and an exclusively locked
    // metadata buffer. The registered page is initialized with a special area
    // large enough for the encoded metadata before the bytes are copied.
    unsafe {
        let mut wal_txn = wal::GenericXLogTxn::start(index_relation);
        let page = wal_txn.register_buffer(buffer.buffer(), pg_sys::GENERIC_XLOG_FULL_IMAGE as i32);
        pg_sys::PageInit(page, page_size, special_size);
        write_metadata_bytes(page, &metadata_bytes);
        wal_txn.finish();
    }
}

pub(super) unsafe fn ec_hnsw_noop_vacuum_stats(
    index_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    let stats = if stats.is_null() {
        // SAFETY: PostgreSQL memory-context allocation creates a zeroed stats
        // struct owned by the current vacuum callback.
        unsafe { PgBox::<pg_sys::IndexBulkDeleteResult>::alloc0().into_pg() }
    } else {
        stats
    };

    // SAFETY: `stats` is either PostgreSQL-supplied or allocated above, and the
    // index relation is live while page/live-tuple stats are computed.
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
    // SAFETY: The index relation is live and the metadata page is read under a
    // shared buffer lock.
    let metadata = unsafe { read_metadata_page(index_relation) };
    // SAFETY: Metadata was decoded from this index and describes its graph
    // storage layout.
    let storage =
        unsafe { graph::GraphStorageDescriptor::from_index_relation(index_relation, &metadata) }
            .unwrap_or_else(|e| pgrx::error!("{e}"));
    let block_count = hnsw_main_block_count(index_relation);
    if block_count <= page::FIRST_DATA_BLOCK_NUMBER {
        return 0;
    }
    let mut count = 0_usize;

    #[cfg(feature = "pg18")]
    {
        let mut linear_state = stream::LinearPrefetchState::new(
            page::FIRST_DATA_BLOCK_NUMBER,
            block_count
                .saturating_sub(1)
                .max(page::FIRST_DATA_BLOCK_NUMBER),
        );
        // SAFETY: The index relation and callback state live until
        // `read_stream_end`; the stream callback yields main-fork block numbers.
        let stream = unsafe {
            pg_sys::read_stream_begin_relation(
                pg_sys::READ_STREAM_SEQUENTIAL as i32,
                ptr::null_mut(),
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                Some(stream::linear_prefetch_cb),
                (&mut linear_state as *mut stream::LinearPrefetchState).cast(),
                std::mem::size_of::<pg_sys::BlockNumber>(),
            )
        };
        // SAFETY: `stream` was just opened and can be reset before consumption.
        unsafe { pg_sys::read_stream_reset(stream) };
        loop {
            let mut per_buffer_data = ptr::null_mut();
            // SAFETY: `stream` remains valid until `read_stream_end` below.
            let buffer = unsafe { pg_sys::read_stream_next_buffer(stream, &mut per_buffer_data) };
            if buffer == pg_sys::InvalidBuffer as pg_sys::Buffer {
                break;
            }
            let block_number = if per_buffer_data.is_null() {
                page::FIRST_DATA_BLOCK_NUMBER
            } else {
                // SAFETY: The stream callback stores a BlockNumber-sized payload
                // when per-buffer data is non-null.
                unsafe { *per_buffer_data.cast::<pg_sys::BlockNumber>() }
            };
            // SAFETY: `read_stream_next_buffer` returns a pinned buffer that is
            // locked here for shared tuple inspection.
            let buffer =
                unsafe { LockedBufferGuard::lock_pinned(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) }
                    .unwrap_or_else(|| {
                        pgrx::error!(
                            "ec_hnsw failed to open data buffer while counting live tuples"
                        )
                    });
            // SAFETY: The buffer guard owns a shared lock and `storage` matches
            // the decoded index metadata.
            count += unsafe { count_live_elements_on_buffer(storage, &buffer, block_number) };
        }
        // SAFETY: Ends the read stream opened above after all buffers are consumed.
        unsafe { pg_sys::read_stream_end(stream) };
    }

    #[cfg(not(feature = "pg18"))]
    {
        for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
            let buffer = read_main_buffer(
                index_relation,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                pg_sys::BUFFER_LOCK_SHARE as i32,
                "counting live tuples",
            );
            // SAFETY: The buffer guard owns a shared lock and `storage` matches
            // the decoded index metadata.
            count += unsafe { count_live_elements_on_buffer(storage, &buffer, block_number) };
        }
    }

    count
}

unsafe fn count_live_elements_on_buffer(
    storage: graph::GraphStorageDescriptor,
    buffer: &LockedBufferGuard,
    block_number: u32,
) -> usize {
    let page_ptr = buffer.page().cast::<u8>();
    let page_size = buffer.page_size();
    let line_pointer_count = page_line_pointer_count(page_ptr);
    let mut count = 0_usize;

    for offset in 1..=line_pointer_count {
        // SAFETY: The buffer is shared-locked by the caller; offsets are bounded
        // by this page's line pointer count before tuple bytes are visited.
        unsafe {
            with_page_line_tuple_bytes(
                page_ptr,
                page_size,
                block_number,
                offset,
                "counting vacuum tuples",
                |tuple_bytes| match storage {
                    graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                        if tuple_bytes.first().copied() == Some(page::TQ_ELEMENT_TAG) {
                            let element = page::TqElementTuple::decode(tuple_bytes, code_len)
                                    .unwrap_or_else(|e| {
                                        pgrx::error!(
                                            "ec_hnsw failed to decode element tuple while counting: {e}"
                                        )
                                    });
                            if !element.deleted && !element.heaptids.is_empty() {
                                count += 1;
                            }
                        }
                    }
                    graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
                        if tuple_bytes.first().copied() == Some(page::TQ_TURBO_HOT_TAG) {
                            let element = page::TqTurboHotTuple::decode(
                                    tuple_bytes,
                                    layout.binary_word_count,
                                )
                                .unwrap_or_else(|e| {
                                    pgrx::error!(
                                        "ec_hnsw failed to decode TurboQuant V3 tuple while counting: {e}"
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
                                        "ec_hnsw failed to decode grouped hot tuple while counting: {e}"
                                    )
                                });
                            if !element.deleted && !element.heaptids.is_empty() {
                                count += 1;
                            }
                        }
                    }
                },
            )
            .unwrap_or_else(|e| pgrx::error!("{e}"))
        };
    }

    count
}

pub(super) unsafe fn highest_level_live_entry_candidate(
    index_relation: pg_sys::Relation,
    storage: graph::GraphStorageDescriptor,
) -> Option<LiveEntryCandidate> {
    let block_count = hnsw_main_block_count(index_relation);
    let mut best_level = None;
    let mut best = None;

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        let buffer = read_main_buffer(
            index_relation,
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
            "selecting live entry candidate",
        );
        let page_ptr = buffer.page().cast::<u8>();
        let page_size = buffer.page_size();
        let line_pointer_count = page_line_pointer_count(page_ptr);

        for offset in 1..=line_pointer_count {
            let tid = page::ItemPointer {
                block_number,
                offset_number: offset,
            };
            // SAFETY: The buffer is shared-locked and the offset is bounded by
            // this page's line pointer count before tuple bytes are decoded.
            let candidate = unsafe {
                with_page_line_tuple_bytes(
                    page_ptr,
                    page_size,
                    block_number,
                    offset,
                    "selecting a live entry candidate",
                    |tuple_bytes| match storage {
                        graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                            if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
                                None
                            } else {
                                let element =
                                    page::TqElementTuple::decode(tuple_bytes, code_len)
                                        .unwrap_or_else(|e| {
                                            pgrx::error!(
                                                "ec_hnsw failed to decode element tuple while selecting a live entry candidate: {e}"
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
                                let element = page::TqTurboHotTuple::decode(
                                    tuple_bytes,
                                    layout.binary_word_count,
                                )
                                .unwrap_or_else(|e| {
                                    pgrx::error!(
                                        "ec_hnsw failed to decode TurboQuant V3 tuple while selecting a live entry candidate: {e}"
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
                                        "ec_hnsw failed to decode grouped hot tuple while selecting a live entry candidate: {e}"
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
                    },
                )
            }
            .unwrap_or_else(|e| pgrx::error!("{e}"))
            .flatten();
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
    }

    best
}

pub(super) unsafe fn page_item_id(page_ptr: *mut u8, offset: u16) -> *const pg_sys::ItemIdData {
    // SAFETY: The caller bounds-checks `offset` against the page line pointer
    // count before using the returned ItemId pointer.
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
    // SAFETY: `page_ptr` points at a pinned PostgreSQL page and `pd_lower`
    // identifies the end of the line pointer array.
    ((unsafe { (*page_header).pd_lower } as usize - size_of::<pg_sys::PageHeaderData>())
        / size_of::<pg_sys::ItemIdData>()) as u16
}

pub(super) unsafe fn with_page_line_tuple_bytes<R, F>(
    page_ptr: *mut u8,
    page_size: usize,
    block_number: pg_sys::BlockNumber,
    offset: u16,
    context: &str,
    visit: F,
) -> Result<Option<R>, String>
where
    F: for<'a> FnOnce(&'a [u8]) -> R,
{
    // SAFETY: The caller bounds-checks `offset` against the page line pointer
    // count before this helper reads the ItemId.
    let item_id = unsafe { &*page_item_id(page_ptr, offset) };
    if item_id.lp_flags() == 0 {
        return Ok(None);
    }

    let tuple_offset = item_id.lp_off() as usize;
    let tuple_len = item_id.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        return Err(format!(
            "ec_hnsw found invalid tuple bounds while {context} on block {block_number}"
        ));
    }

    // SAFETY: Tuple bounds were checked against `page_size` above and the page
    // is shared-locked for immutable tuple inspection.
    let tuple_bytes = unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
    Ok(Some(visit(tuple_bytes)))
}

pub(super) unsafe fn with_writable_page_tuple_bytes<R, F>(
    page_ptr: *mut u8,
    page_size: usize,
    tuple_tid: page::ItemPointer,
    tuple_kind: &str,
    visit: F,
) -> R
where
    F: for<'a> FnOnce(&'a mut [u8]) -> R,
{
    // SAFETY: The caller owns an exclusive page lock and supplies a tuple offset
    // selected from this page.
    let item_id = unsafe { &*page_item_id(page_ptr, tuple_tid.offset_number) };
    if item_id.lp_flags() == 0 {
        pgrx::error!(
            "ec_hnsw {tuple_kind} tuple slot {}/{} is unused",
            tuple_tid.block_number,
            tuple_tid.offset_number
        );
    }

    let tuple_offset = item_id.lp_off() as usize;
    let tuple_len = item_id.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        pgrx::error!(
            "ec_hnsw found invalid {tuple_kind} tuple bounds on block {}",
            tuple_tid.block_number
        );
    }

    // SAFETY: Tuple bounds were checked against `page_size` above and the caller
    // owns an exclusive lock for mutable tuple access.
    let tuple_bytes =
        unsafe { std::slice::from_raw_parts_mut(page_ptr.add(tuple_offset), tuple_len) };
    visit(tuple_bytes)
}

pub(super) unsafe fn decode_heap_tid(tid: pg_sys::ItemPointer) -> page::ItemPointer {
    if tid.is_null() {
        pgrx::error!("ec_hnsw ambuild received a null heap tid");
    }
    // SAFETY: Null was checked above and PostgreSQL supplied this heap TID for
    // the AM callback currently decoding it.
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
    let index_relation = IndexRelationGuard::access_share(index_oid, "debug_index_pages");
    let block_count = hnsw_main_block_count(index_relation.as_ptr());

    // SAFETY: The index relation guard keeps the metadata page readable.
    let metadata = unsafe { read_metadata_page(index_relation.as_ptr()) };
    let mut data_pages = Vec::new();
    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        // SAFETY: `block_number` is within the current main-fork block range and
        // the guard keeps the relation open.
        data_pages.push(unsafe { read_data_page(index_relation.as_ptr(), block_number) });
    }

    (block_count, metadata, data_pages)
}

pub(crate) unsafe fn read_metadata_page(index_relation: pg_sys::Relation) -> page::MetadataPage {
    let buffer = read_main_buffer(
        index_relation,
        page::METADATA_BLOCK_NUMBER,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_SHARE as i32,
        "reading metadata",
    );
    let raw_page = buffer.page().cast::<u8>();
    let page_size = buffer.page_size();
    // SAFETY: The buffer guard pins the metadata page for the duration of this
    // borrow and `page_size` bounds the slice.
    let page_bytes = unsafe { std::slice::from_raw_parts(raw_page, page_size) };
    let metadata =
        page::MetadataPage::decode_page(page_bytes).expect("metadata page should decode");
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
    // SAFETY: The index relation is live while its reloptions are decoded for
    // admin diagnostics.
    let relation_options = unsafe { options::relation_options(index_relation) };
    let tuning = options::resolve_scan_tuning(&relation_options);
    // SAFETY: The index relation is live and metadata is read under a shared
    // buffer lock.
    let metadata = unsafe { read_metadata_page(index_relation) };
    // SAFETY: The index relation is live while shared page traversal counts
    // live HNSW element tuples.
    let total_live_nodes = unsafe { count_element_tuples(index_relation) };
    let inserted_since_rebuild =
        usize::try_from(metadata.inserted_since_rebuild).unwrap_or_else(|_| {
            pgrx::error!(
                "ec_hnsw metadata inserted_since_rebuild {} exceeds usize",
                metadata.inserted_since_rebuild
            )
        });
    IndexAdminSnapshot {
        block_count: hnsw_main_block_count(index_relation),
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
        planner_scan_enabled: EC_HNSW_PLANNER_SCAN_ENABLED,
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
    // SAFETY: The index relation is live for the duration of this diagnostic
    // snapshot and admin snapshot reads only relation metadata.
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
    // SAFETY: The index relation is live while its reloptions are decoded for
    // cost diagnostics.
    let relation_options = unsafe { options::relation_options(index_relation) };
    let tuning = options::resolve_scan_tuning(&relation_options);
    // SAFETY: The index relation is live and metadata is read under a shared
    // buffer lock.
    let metadata = unsafe { read_metadata_page(index_relation) };
    let block_count = hnsw_main_block_count(index_relation);
    let index_pages = f64::from(block_count);
    // SAFETY: `index_relation` is a live Relation and `rd_rel` points at
    // PostgreSQL's relation catalog tuple for the relation lifetime.
    let reltuples = unsafe { (*(*index_relation).rd_rel).reltuples } as f64;
    let tree_height = super::cost::resolved_tree_height_input(metadata.max_level);
    // SAFETY: Planner cost constants are read from PostgreSQL GUC state for this
    // backend without retaining raw pointers.
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
        planner_scan_enabled: EC_HNSW_PLANNER_SCAN_ENABLED,
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
    // SAFETY: The index relation is live for the duration of this diagnostic
    // snapshot and admin snapshot reads only relation metadata.
    let admin = unsafe { index_admin_snapshot(index_relation) };
    // SAFETY: Delegates to snapshot helpers using the same live index relation.
    let explain = unsafe { index_explain_snapshot(index_relation) };
    // SAFETY: Delegates to snapshot helpers using the same live index relation.
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
        next_pg18_blocker: if diagnostics.pg18_pgstat_kind_ready {
            "no merged PG18 blocker remains on main"
        } else {
            super::stats::pgstat_kind_blocker()
                .unwrap_or("custom pgstat kind registration remains gated outside this build")
        },
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
    // SAFETY: Test/debug callers hold the index relation open while the admin
    // snapshot reads metadata and reloptions.
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
        IndexRelationGuard::access_share(index_oid, "debug_planner_tuning_snapshot");
    planner_tuning_snapshot(index_relation.as_ptr())
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn read_data_page(
    index_relation: pg_sys::Relation,
    block_number: u32,
) -> DebugIndexDataPage {
    let buffer = read_main_buffer(
        index_relation,
        block_number,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_SHARE as i32,
        "debug read",
    );
    let raw_page = buffer.page().cast::<u8>();
    let page_size = buffer.page_size();
    let page_header = raw_page.cast::<pg_sys::PageHeaderData>();
    // SAFETY: The buffer guard pins the page while the line pointer count is
    // read from the page header.
    let line_pointer_count = ((unsafe { (*page_header).pd_lower } as usize
        - size_of::<pg_sys::PageHeaderData>())
        / size_of::<pg_sys::ItemIdData>()) as u16;

    let mut tuples = Vec::with_capacity(line_pointer_count as usize);
    for offset in 1..=line_pointer_count {
        // SAFETY: The page is shared-locked and `offset` is bounded by the page
        // line pointer count before tuple bytes are copied.
        if let Some(tuple) = unsafe {
            with_page_line_tuple_bytes(
                raw_page,
                page_size,
                block_number,
                offset,
                "debug read",
                |tuple_bytes| tuple_bytes.to_vec(),
            )
        }
        .unwrap_or_else(|e| pgrx::error!("{e}"))
        {
            tuples.push(tuple);
        }
    }

    DebugIndexDataPage {
        block_number,
        tuples,
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_index_metadata(
    index_oid: pg_sys::Oid,
) -> (u32, i32, i32, page::MetadataPage) {
    let index_relation = IndexRelationGuard::access_share(index_oid, "debug_index_metadata");
    // SAFETY: The index relation guard keeps the relation open while reloptions
    // are decoded.
    let options = unsafe { super::options::relation_options(index_relation.as_ptr()) };
    let block_count = hnsw_main_block_count(index_relation.as_ptr());
    // SAFETY: The index relation guard keeps the metadata page readable.
    let metadata = unsafe { read_metadata_page(index_relation.as_ptr()) };

    (block_count, options.m, options.ef_construction, metadata)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_update_index_metadata(
    index_oid: pg_sys::Oid,
    metadata: page::MetadataPage,
) {
    let index_relation = IndexRelationGuard::access_share(index_oid, "debug_update_index_metadata");
    // SAFETY: The index relation guard keeps the relation open while the
    // metadata page is rewritten under exclusive lock.
    unsafe { update_metadata_page(index_relation.as_ptr(), metadata) };
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_vacuum_stats(index_oid: pg_sys::Oid) -> pg_sys::IndexBulkDeleteResult {
    let index_relation = IndexRelationGuard::access_share(index_oid, "debug_vacuum_stats");
    let mut info = PgBox::<pg_sys::IndexVacuumInfo>::alloc0();
    info.index = index_relation.as_ptr();
    let info_ptr = (&mut *info) as *mut pg_sys::IndexVacuumInfo;

    // SAFETY: The test constructs callback-duration vacuum info and invokes the
    // AM bulkdelete entry with no delete callback for stats.
    let stats = unsafe {
        super::vacuum::ec_hnsw_ambulkdelete(info_ptr, ptr::null_mut(), None, ptr::null_mut())
    };
    // SAFETY: The same vacuum info and stats pointer are valid for cleanup.
    let stats = unsafe { super::vacuum::ec_hnsw_amvacuumcleanup(info_ptr, stats) };
    // SAFETY: The AM returned a valid stats pointer; copy it before guards leave
    // scope.
    let result = unsafe { *stats };

    result
}
