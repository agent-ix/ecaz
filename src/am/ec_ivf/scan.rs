use std::collections::HashSet;
use std::ptr;

use pgrx::{pg_sys, FromDatum, PgBox};
#[cfg(any(test, feature = "pg_test"))]
use pgrx::IntoDatum;

use crate::quant::prod::{PreparedQuery, ProdQuantizer};
use crate::storage::page::ItemPointer;

#[derive(Debug, Default)]
struct EcIvfScanOpaque {
    rescan_called: bool,
    query_dimensions: u16,
    query_values: *mut f32,
    scan_dimensions: u16,
    scan_nlists: u32,
    scan_nprobe: u32,
    prepared_query: *mut PreparedQuery,
    centroid_scores: *mut EcIvfCentroidScore,
    centroid_score_count: u32,
    selected_lists: *mut u32,
    selected_list_count: u32,
    posting_candidates: *mut EcIvfScoredCandidate,
    posting_candidate_count: u32,
}

#[derive(Debug, Clone, Copy)]
struct EcIvfCentroidScore {
    list_id: u32,
    score: f32,
}

#[derive(Debug, Clone, Copy)]
struct EcIvfScoredCandidate {
    heap_tid: ItemPointer,
    score: f32,
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_ambeginscan(
    index_relation: pg_sys::Relation,
    nkeys: std::ffi::c_int,
    norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let scan = pg_sys::RelationGetIndexScan(index_relation, nkeys, norderbys);
            if scan.is_null() {
                pgrx::error!("ec_ivf failed to allocate scan descriptor");
            }

            (*scan).parallel_scan = ptr::null_mut();
            (*scan).opaque = PgBox::<EcIvfScanOpaque>::alloc0().into_pg().cast();
            scan
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_amrescan(
    scan: pg_sys::IndexScanDesc,
    _keys: pg_sys::ScanKey,
    nkeys: std::ffi::c_int,
    orderbys: pg_sys::ScanKey,
    norderbys: std::ffi::c_int,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                pgrx::error!("ec_ivf amrescan received a null scan descriptor");
            }
            if nkeys != 0 {
                pgrx::error!("ec_ivf scan does not support index quals yet");
            }
            if norderbys != 1 {
                pgrx::error!("ec_ivf scan currently requires exactly one ORDER BY query");
            }
            if orderbys.is_null() {
                pgrx::error!("ec_ivf amrescan received null order-by scan keys");
            }

            let orderby = &*orderbys;
            if (orderby.sk_flags as u32) & pg_sys::SK_ISNULL != 0 {
                pgrx::error!("ec_ivf scan query must not be NULL");
            }

            let query = Vec::<f32>::from_polymorphic_datum(
                orderby.sk_argument,
                false,
                pg_sys::FLOAT4ARRAYOID,
            )
            .unwrap_or_else(|| pgrx::error!("ec_ivf scan requires a real[] ORDER BY query"));
            if query.is_empty() {
                pgrx::error!("ec_ivf scan query must not be empty");
            }
            if query.len() > u16::MAX as usize {
                pgrx::error!(
                    "ec_ivf scan query dimension {} exceeds maximum {}",
                    query.len(),
                    u16::MAX
                );
            }

            let metadata = super::page::read_metadata_page((*scan).indexRelation);
            if metadata.dimensions != 0 && query.len() != metadata.dimensions as usize {
                pgrx::error!(
                    "ec_ivf scan query dimension mismatch: index dim {}, query dim {}",
                    metadata.dimensions,
                    query.len()
                );
            }

            (*scan).xs_recheck = false;
            (*scan).xs_recheckorderby = false;
            (*scan).xs_orderbyvals = ptr::null_mut();
            (*scan).xs_orderbynulls = ptr::null_mut();

            let opaque_ptr = (*scan).opaque.cast::<EcIvfScanOpaque>();
            if opaque_ptr.is_null() {
                pgrx::error!("ec_ivf amrescan missing scan opaque state");
            }
            let opaque = &mut *opaque_ptr;
            free_scan_query_prep(opaque);
            opaque.rescan_called = true;
            opaque.scan_dimensions = metadata.dimensions;
            opaque.scan_nlists = metadata.nlists;
            opaque.scan_nprobe = if metadata.dimensions == 0 {
                0
            } else {
                resolve_effective_nprobe(&metadata)
            };
            store_scan_query(opaque, &query);
            store_scan_prepared_query(opaque, &query, &metadata);

            if metadata.dimensions != 0 {
                let centroid_scores =
                    load_centroid_scores((*scan).indexRelation, &metadata, &query)
                        .unwrap_or_else(|e| pgrx::error!("{e}"));
                let selected_lists = select_probe_lists(&centroid_scores, opaque.scan_nprobe);
                let posting_candidates = materialize_probe_candidates(
                    (*scan).indexRelation,
                    &metadata,
                    opaque,
                    &selected_lists,
                )
                .unwrap_or_else(|e| pgrx::error!("{e}"));
                store_centroid_scores(opaque, &centroid_scores);
                store_selected_lists(opaque, &selected_lists);
                store_posting_candidates(opaque, &posting_candidates);
            };
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_amgettuple(
    scan: pg_sys::IndexScanDesc,
    direction: pg_sys::ScanDirection::Type,
) -> bool {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                pgrx::error!("ec_ivf amgettuple received a null scan descriptor");
            }
            if direction != pg_sys::ScanDirection::ForwardScanDirection {
                pgrx::error!("ec_ivf amgettuple only supports forward scan direction");
            }
            let opaque_ptr = (*scan).opaque.cast::<EcIvfScanOpaque>();
            if opaque_ptr.is_null() {
                pgrx::error!("ec_ivf amgettuple missing scan opaque state");
            }
            if !(*opaque_ptr).rescan_called {
                pgrx::error!("ec_ivf amgettuple requires amrescan before scan execution");
            }

            false
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_amendscan(scan: pg_sys::IndexScanDesc) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                return;
            }

            let opaque_ptr = (*scan).opaque;
            if !opaque_ptr.is_null() {
                free_scan_query_prep(&mut *opaque_ptr.cast::<EcIvfScanOpaque>());
                pg_sys::pfree(opaque_ptr);
                (*scan).opaque = ptr::null_mut();
            }
        })
    }
}

unsafe fn store_scan_query(opaque: &mut EcIvfScanOpaque, query: &[f32]) {
    free_scan_query(opaque);

    let query_bytes = std::mem::size_of_val(query);
    let query_values = unsafe { pg_sys::palloc(query_bytes) }.cast::<f32>();
    if query_values.is_null() {
        pgrx::error!("ec_ivf failed to allocate scan query state");
    }
    unsafe { ptr::copy_nonoverlapping(query.as_ptr(), query_values, query.len()) };
    opaque.query_dimensions = u16::try_from(query.len()).expect("query length should fit in u16");
    opaque.query_values = query_values;
}

unsafe fn free_scan_query(opaque: &mut EcIvfScanOpaque) {
    if !opaque.query_values.is_null() {
        unsafe { pg_sys::pfree(opaque.query_values.cast()) };
        opaque.query_values = ptr::null_mut();
    }
    opaque.query_dimensions = 0;
}

fn store_scan_prepared_query(
    opaque: &mut EcIvfScanOpaque,
    query: &[f32],
    metadata: &super::page::MetadataPage,
) {
    free_scan_prepared_query(opaque);
    if metadata.dimensions == 0 {
        return;
    }

    let quantizer = ProdQuantizer::cached(
        metadata.dimensions as usize,
        crate::DEFAULT_QUANT_BITS,
        crate::DEFAULT_QUANT_SEED,
    );
    let prepared = quantizer.prepare_ip_query(query);
    opaque.prepared_query = Box::into_raw(Box::new(prepared));
}

fn free_scan_prepared_query(opaque: &mut EcIvfScanOpaque) {
    if !opaque.prepared_query.is_null() {
        drop(unsafe { Box::from_raw(opaque.prepared_query) });
        opaque.prepared_query = ptr::null_mut();
    }
}

unsafe fn store_centroid_scores(
    opaque: &mut EcIvfScanOpaque,
    scores: &[EcIvfCentroidScore],
) {
    free_centroid_scores(opaque);
    if scores.is_empty() {
        return;
    }

    let bytes = std::mem::size_of_val(scores);
    let score_ptr = unsafe { pg_sys::palloc(bytes) }.cast::<EcIvfCentroidScore>();
    if score_ptr.is_null() {
        pgrx::error!("ec_ivf failed to allocate centroid score state");
    }
    unsafe { ptr::copy_nonoverlapping(scores.as_ptr(), score_ptr, scores.len()) };
    opaque.centroid_score_count =
        u32::try_from(scores.len()).expect("centroid score count should fit in u32");
    opaque.centroid_scores = score_ptr;
}

unsafe fn free_centroid_scores(opaque: &mut EcIvfScanOpaque) {
    if !opaque.centroid_scores.is_null() {
        unsafe { pg_sys::pfree(opaque.centroid_scores.cast()) };
        opaque.centroid_scores = ptr::null_mut();
    }
    opaque.centroid_score_count = 0;
}

unsafe fn store_selected_lists(opaque: &mut EcIvfScanOpaque, selected_lists: &[u32]) {
    free_selected_lists(opaque);
    if selected_lists.is_empty() {
        return;
    }

    let bytes = std::mem::size_of_val(selected_lists);
    let list_ptr = unsafe { pg_sys::palloc(bytes) }.cast::<u32>();
    if list_ptr.is_null() {
        pgrx::error!("ec_ivf failed to allocate selected-list state");
    }
    unsafe { ptr::copy_nonoverlapping(selected_lists.as_ptr(), list_ptr, selected_lists.len()) };
    opaque.selected_list_count =
        u32::try_from(selected_lists.len()).expect("selected list count should fit in u32");
    opaque.selected_lists = list_ptr;
}

unsafe fn free_selected_lists(opaque: &mut EcIvfScanOpaque) {
    if !opaque.selected_lists.is_null() {
        unsafe { pg_sys::pfree(opaque.selected_lists.cast()) };
        opaque.selected_lists = ptr::null_mut();
    }
    opaque.selected_list_count = 0;
}

unsafe fn store_posting_candidates(
    opaque: &mut EcIvfScanOpaque,
    candidates: &[EcIvfScoredCandidate],
) {
    free_posting_candidates(opaque);
    if candidates.is_empty() {
        return;
    }

    let bytes = std::mem::size_of_val(candidates);
    let candidate_ptr = unsafe { pg_sys::palloc(bytes) }.cast::<EcIvfScoredCandidate>();
    if candidate_ptr.is_null() {
        pgrx::error!("ec_ivf failed to allocate posting candidate state");
    }
    unsafe { ptr::copy_nonoverlapping(candidates.as_ptr(), candidate_ptr, candidates.len()) };
    opaque.posting_candidate_count =
        u32::try_from(candidates.len()).expect("posting candidate count should fit in u32");
    opaque.posting_candidates = candidate_ptr;
}

unsafe fn free_posting_candidates(opaque: &mut EcIvfScanOpaque) {
    if !opaque.posting_candidates.is_null() {
        unsafe { pg_sys::pfree(opaque.posting_candidates.cast()) };
        opaque.posting_candidates = ptr::null_mut();
    }
    opaque.posting_candidate_count = 0;
}

unsafe fn free_scan_query_prep(opaque: &mut EcIvfScanOpaque) {
    unsafe { free_scan_query(opaque) };
    free_scan_prepared_query(opaque);
    unsafe {
        free_centroid_scores(opaque);
        free_selected_lists(opaque);
        free_posting_candidates(opaque);
    }
    opaque.scan_dimensions = 0;
    opaque.scan_nlists = 0;
    opaque.scan_nprobe = 0;
}

fn resolve_effective_nprobe(metadata: &super::page::MetadataPage) -> u32 {
    if metadata.nlists == 0 {
        return 0;
    }

    let session_nprobe = super::options::current_session_nprobe();
    let requested = if session_nprobe > 0 {
        session_nprobe as u32
    } else {
        metadata.nprobe
    };
    let resolved = if requested == 0 {
        auto_nprobe(metadata.nlists)
    } else {
        requested
    };
    resolved.clamp(1, metadata.nlists)
}

fn auto_nprobe(nlists: u32) -> u32 {
    if nlists == 0 {
        return 0;
    }
    (nlists as f64).sqrt().ceil() as u32
}

unsafe fn load_centroid_scores(
    index_relation: pg_sys::Relation,
    metadata: &super::page::MetadataPage,
    query: &[f32],
) -> Result<Vec<EcIvfCentroidScore>, String> {
    if metadata.nlists == 0 {
        return Ok(Vec::new());
    }
    if metadata.centroid_head == ItemPointer::INVALID {
        return Err("ec_ivf metadata has lists but no centroid head".to_owned());
    }

    let dimensions = metadata.dimensions as usize;
    let mut next_tid = metadata.centroid_head;
    let mut scores = Vec::with_capacity(metadata.nlists as usize);
    for expected_list_id in 0..metadata.nlists {
        let (centroid, following_tid) = unsafe {
            super::page::read_ivf_centroid_and_next(index_relation, next_tid, dimensions)?
        };
        if centroid.list_id != expected_list_id {
            return Err(format!(
                "ec_ivf centroid order mismatch: got list {}, expected {}",
                centroid.list_id, expected_list_id
            ));
        }
        scores.push(EcIvfCentroidScore {
            list_id: centroid.list_id,
            score: inner_product(query, &centroid.centroid),
        });
        next_tid = following_tid;
    }
    Ok(scores)
}

fn inner_product(left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| left * right)
        .sum()
}

fn select_probe_lists(scores: &[EcIvfCentroidScore], nprobe: u32) -> Vec<u32> {
    let mut ranked = scores.to_vec();
    ranked.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.list_id.cmp(&right.list_id))
    });
    ranked
        .into_iter()
        .take(nprobe as usize)
        .map(|score| score.list_id)
        .collect()
}

unsafe fn materialize_probe_candidates(
    index_relation: pg_sys::Relation,
    metadata: &super::page::MetadataPage,
    opaque: &EcIvfScanOpaque,
    selected_lists: &[u32],
) -> Result<Vec<EcIvfScoredCandidate>, String> {
    if selected_lists.is_empty() {
        return Ok(Vec::new());
    }
    if opaque.prepared_query.is_null() {
        return Err("ec_ivf posting-list scan requires a prepared query".to_owned());
    }

    let directories = unsafe { load_directory_entries(index_relation, metadata)? };
    let prepared_query = unsafe { &*opaque.prepared_query };
    let quantizer = ProdQuantizer::cached(
        metadata.dimensions as usize,
        crate::DEFAULT_QUANT_BITS,
        crate::DEFAULT_QUANT_SEED,
    );
    let payload_len = crate::payload_len(metadata.dimensions as usize, crate::DEFAULT_QUANT_BITS);
    let mut seen_heap_tids = HashSet::new();
    let mut candidates = Vec::new();
    for list_id in selected_lists {
        let directory = directories
            .get(*list_id as usize)
            .ok_or_else(|| format!("ec_ivf selected list {list_id} is out of range"))?;
        let postings = unsafe {
            super::page::read_ivf_postings_for_list_blocks(
                index_relation,
                *list_id,
                directory.head_block,
                directory.tail_block,
                payload_len,
            )?
        };
        for posting in postings {
            if posting.deleted {
                continue;
            }
            let score =
                -quantizer.score_ip_from_parts(prepared_query, posting.gamma, &posting.payload);
            for heap_tid in posting.heaptids {
                if seen_heap_tids.insert(heap_tid) {
                    candidates.push(EcIvfScoredCandidate { heap_tid, score });
                }
            }
        }
    }

    candidates.sort_by(|left, right| {
        left.score.total_cmp(&right.score).then_with(|| {
            left.heap_tid
                .block_number
                .cmp(&right.heap_tid.block_number)
                .then_with(|| left.heap_tid.offset_number.cmp(&right.heap_tid.offset_number))
        })
    });
    Ok(candidates)
}

unsafe fn load_directory_entries(
    index_relation: pg_sys::Relation,
    metadata: &super::page::MetadataPage,
) -> Result<Vec<super::page::IvfListDirectoryTuple>, String> {
    if metadata.nlists == 0 {
        return Ok(Vec::new());
    }
    if metadata.directory_head == ItemPointer::INVALID {
        return Err("ec_ivf metadata has lists but no directory head".to_owned());
    }

    let mut next_tid = metadata.directory_head;
    let mut directories = Vec::with_capacity(metadata.nlists as usize);
    for expected_list_id in 0..metadata.nlists {
        let (directory, following_tid) =
            unsafe { super::page::read_ivf_list_directory_and_next(index_relation, next_tid)? };
        if directory.list_id != expected_list_id {
            return Err(format!(
                "ec_ivf directory order mismatch: got list {}, expected {}",
                directory.list_id, expected_list_id
            ));
        }
        directories.push(directory);
        next_tid = following_tid;
    }
    Ok(directories)
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_scan_query(opaque: &EcIvfScanOpaque) -> Vec<f32> {
    if opaque.query_values.is_null() || opaque.query_dimensions == 0 {
        return Vec::new();
    }
    unsafe { std::slice::from_raw_parts(opaque.query_values, opaque.query_dimensions as usize) }
        .to_vec()
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_selected_lists(opaque: &EcIvfScanOpaque) -> Vec<u32> {
    if opaque.selected_lists.is_null() || opaque.selected_list_count == 0 {
        return Vec::new();
    }
    unsafe {
        std::slice::from_raw_parts(opaque.selected_lists, opaque.selected_list_count as usize)
    }
    .to_vec()
}

#[cfg(any(test, feature = "pg_test"))]
struct DebugHeapBackedScan {
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
    scan: pg_sys::IndexScanDesc,
    registered_snapshot: pg_sys::Snapshot,
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn debug_push_latest_snapshot() -> pg_sys::Snapshot {
    unsafe { pg_sys::CommandCounterIncrement() };
    let snapshot = unsafe { pg_sys::RegisterSnapshot(pg_sys::GetLatestSnapshot()) };
    if snapshot.is_null() {
        pgrx::error!("ec_ivf debug scan could not acquire a latest snapshot");
    }
    unsafe { pg_sys::PushActiveSnapshot(snapshot) };
    snapshot
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn debug_begin_heap_backed_scan(index_oid: pg_sys::Oid) -> DebugHeapBackedScan {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let heap_oid = unsafe { pg_sys::IndexGetRelation((*index_relation).rd_id, false) };
    if heap_oid == pg_sys::InvalidOid {
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        pgrx::error!("ec_ivf debug scan could not resolve heap relation for index {index_oid}");
    }

    let heap_relation =
        unsafe { pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let registered_snapshot = unsafe { debug_push_latest_snapshot() };
    #[cfg(feature = "pg18")]
    let scan = unsafe {
        pg_sys::index_beginscan(
            heap_relation,
            index_relation,
            registered_snapshot,
            ptr::null_mut(),
            0,
            1,
        )
    };
    #[cfg(not(feature = "pg18"))]
    let scan = unsafe {
        pg_sys::index_beginscan(heap_relation, index_relation, registered_snapshot, 0, 1)
    };
    if scan.is_null() {
        unsafe {
            pg_sys::PopActiveSnapshot();
            pg_sys::UnregisterSnapshot(registered_snapshot);
            pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
            pg_sys::table_close(heap_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        }
        pgrx::error!("ec_ivf debug scan failed to begin heap-backed index scan");
    }

    DebugHeapBackedScan {
        index_relation,
        heap_relation,
        scan,
        registered_snapshot,
    }
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn debug_end_heap_backed_scan(state: DebugHeapBackedScan) {
    unsafe {
        pg_sys::index_endscan(state.scan);
        pg_sys::PopActiveSnapshot();
        pg_sys::UnregisterSnapshot(state.registered_snapshot);
        pg_sys::index_close(
            state.index_relation,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        );
        pg_sys::table_close(
            state.heap_relation,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        );
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_gettuple_after_rescan_result(index_oid: pg_sys::Oid) -> bool {
    let state = unsafe { debug_begin_heap_backed_scan(index_oid) };
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: IntoDatum::into_datum(vec![1.0_f32])
            .expect("debug query should convert to datum"),
        ..Default::default()
    };
    unsafe { pg_sys::index_rescan(state.scan, ptr::null_mut(), 0, &mut orderby, 1) };
    let tid = unsafe {
        pg_sys::index_getnext_tid(state.scan, pg_sys::ScanDirection::ForwardScanDirection)
    };
    let found = !tid.is_null();

    unsafe { debug_end_heap_backed_scan(state) };
    found
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_rescan_query_prep(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (
    bool,
    u16,
    Vec<f32>,
    u16,
    u32,
    u32,
    bool,
    usize,
    usize,
    u32,
    u32,
    Vec<u32>,
) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { ec_ivf_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { ec_ivf_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let opaque = unsafe { &*(*scan).opaque.cast::<EcIvfScanOpaque>() };
    let result = (
        opaque.rescan_called,
        opaque.query_dimensions,
        debug_scan_query(opaque),
        opaque.scan_dimensions,
        opaque.scan_nlists,
        opaque.scan_nprobe,
        !opaque.prepared_query.is_null(),
        if opaque.prepared_query.is_null() {
            0
        } else {
            unsafe { (*opaque.prepared_query).lut.len() }
        },
        if opaque.prepared_query.is_null() {
            0
        } else {
            unsafe { (*opaque.prepared_query).sq.len() }
        },
        opaque.centroid_score_count,
        opaque.posting_candidate_count,
        debug_selected_lists(opaque),
    );

    unsafe { ec_ivf_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_metadata(index_oid: pg_sys::Oid) -> (u16, u32, u32, u32, u64) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let metadata = unsafe { super::page::read_metadata_page(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        metadata.format_version,
        metadata.nlists,
        metadata.nprobe,
        metadata.training_sample_rows,
        metadata.seed,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_build_metadata(
    index_oid: pg_sys::Oid,
) -> (u16, u32, u16, u64, bool, bool) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let metadata = unsafe { super::page::read_metadata_page(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        metadata.dimensions,
        metadata.nlists,
        metadata.training_version,
        metadata.total_live_tuples,
        metadata.centroid_head != crate::storage::page::ItemPointer::INVALID,
        metadata.directory_head != crate::storage::page::ItemPointer::INVALID,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_directory_summary(
    index_oid: pg_sys::Oid,
) -> (u32, u32, u64, u64, u64) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let metadata = unsafe { super::page::read_metadata_page(index_relation) };

    if metadata.directory_head == crate::storage::page::ItemPointer::INVALID {
        if metadata.total_live_tuples != 0 {
            unsafe {
                pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE)
            };
            pgrx::error!("ec_ivf metadata has live tuples but no directory head");
        }
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        return (metadata.nlists, metadata.nlists, 0, 0, 0);
    }

    let mut next_tid = metadata.directory_head;
    let mut empty_lists = 0_u32;
    let mut live_sum = 0_u64;
    let mut dead_sum = 0_u64;
    let mut inserted_sum = 0_u64;
    for expected_list_id in 0..metadata.nlists {
        let (directory, following_tid) = unsafe {
            super::page::read_ivf_list_directory_and_next(index_relation, next_tid)
                .unwrap_or_else(|e| pgrx::error!("{e}"))
        };
        if directory.list_id != expected_list_id {
            unsafe {
                pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE)
            };
            pgrx::error!(
                "ec_ivf directory order mismatch: got list {}, expected {}",
                directory.list_id,
                expected_list_id
            );
        }
        if directory.live_count == 0 {
            empty_lists += 1;
        }
        live_sum += directory.live_count;
        dead_sum += directory.dead_count;
        inserted_sum += directory.inserted_since_build;
        next_tid = following_tid;
    }

    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        metadata.nlists,
        empty_lists,
        live_sum,
        dead_sum,
        inserted_sum,
    )
}
