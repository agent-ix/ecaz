use std::cmp::Ordering;
use std::collections::{hash_map::Entry, BinaryHeap, HashMap};
use std::ptr;

use pgrx::{pg_sys, FromDatum, IntoDatum, PgBox};

use crate::am::common::explain::IvfExplainCounters;
use crate::am::ec_hnsw::source;
use crate::am::stats::{self, TqStatsCounters};
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
    next_candidate_index: u32,
    explain_counters: IvfExplainCounters,
    stats_delta: TqStatsCounters,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProbeBlockRange {
    head_block: u32,
    tail_block: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectedProbePlan {
    selected_lists: Vec<u32>,
    selected_list_mask: Vec<bool>,
    block_sequence: Vec<u32>,
    candidate_bound: usize,
}

impl SelectedProbePlan {
    fn contains_list(&self, list_id: u32) -> bool {
        self.selected_list_mask
            .get(list_id as usize)
            .copied()
            .unwrap_or(false)
    }

    fn posting_page_count(&self) -> Result<u32, String> {
        u32::try_from(self.block_sequence.len())
            .map_err(|_| "ec_ivf posting block sequence exceeds u32".to_owned())
    }
}

#[derive(Debug, Clone, Copy)]
struct ProbeListHeapEntry {
    centroid: EcIvfCentroidScore,
}

impl PartialEq for ProbeListHeapEntry {
    fn eq(&self, other: &Self) -> bool {
        probe_list_heap_cmp(&self.centroid, &other.centroid) == Ordering::Equal
    }
}

impl Eq for ProbeListHeapEntry {}

impl PartialOrd for ProbeListHeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProbeListHeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        probe_list_heap_cmp(&self.centroid, &other.centroid)
    }
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
            let index_options = super::options::relation_options((*scan).indexRelation);
            metadata
                .storage_format
                .validate_v1_supported()
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            metadata
                .rerank
                .validate_v1_supported()
                .unwrap_or_else(|e| pgrx::error!("{e}"));
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
            if opaque.rescan_called {
                flush_scan_stats(opaque);
            }
            free_scan_query_prep(opaque);
            opaque.explain_counters.reset();
            opaque.stats_delta.reset();
            opaque.rescan_called = true;
            stats::record_scan_started();
            opaque.stats_delta.record_scan_started();
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
                opaque
                    .explain_counters
                    .record_centroid_scores(centroid_scores.len());
                record_distance_calcs(opaque, centroid_scores.len());
                opaque
                    .explain_counters
                    .record_selected_lists(selected_lists.len());
                let posting_candidates = materialize_probe_candidates(
                    scan,
                    (*scan).indexRelation,
                    &metadata,
                    &index_options,
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

            let opaque = &mut *opaque_ptr;
            if opaque.scan_dimensions == 0 {
                clear_scan_orderby_output(scan);
                return false;
            }
            match next_posting_candidate(opaque) {
                Some(candidate) => {
                    set_scan_heap_tid(scan, candidate.heap_tid);
                    set_scan_orderby_score(scan, candidate.score);
                    true
                }
                None => {
                    clear_scan_orderby_output(scan);
                    false
                }
            }
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
                let opaque = &mut *opaque_ptr.cast::<EcIvfScanOpaque>();
                flush_scan_stats(opaque);
                free_scan_query_prep(opaque);
                pg_sys::pfree(opaque_ptr);
                (*scan).opaque = ptr::null_mut();
            }
        })
    }
}

fn next_posting_candidate(opaque: &mut EcIvfScanOpaque) -> Option<EcIvfScoredCandidate> {
    if opaque.posting_candidates.is_null()
        || opaque.next_candidate_index >= opaque.posting_candidate_count
    {
        return None;
    }

    let candidate = unsafe {
        *opaque
            .posting_candidates
            .add(opaque.next_candidate_index as usize)
    };
    opaque.next_candidate_index += 1;
    Some(candidate)
}

fn set_scan_heap_tid(scan: pg_sys::IndexScanDesc, heap_tid: ItemPointer) {
    unsafe {
        pgrx::itemptr::item_pointer_set_all(
            &mut (*scan).xs_heaptid,
            heap_tid.block_number,
            heap_tid.offset_number,
        );
    }
}

fn set_scan_orderby_score(scan: pg_sys::IndexScanDesc, score: f32) {
    unsafe {
        if (*scan).xs_orderbyvals.is_null() {
            (*scan).xs_orderbyvals =
                pg_sys::palloc0(std::mem::size_of::<pg_sys::Datum>()).cast::<pg_sys::Datum>();
        }
        if (*scan).xs_orderbynulls.is_null() {
            (*scan).xs_orderbynulls = pg_sys::palloc0(std::mem::size_of::<bool>()).cast::<bool>();
        }

        *(*scan).xs_orderbyvals = score.into_datum().expect("score should convert to datum");
        *(*scan).xs_orderbynulls = false;
    }
}

fn clear_scan_orderby_output(scan: pg_sys::IndexScanDesc) {
    unsafe {
        if !(*scan).xs_orderbynulls.is_null() {
            *(*scan).xs_orderbynulls = true;
        }
    }
}

fn record_distance_calcs(opaque: &mut EcIvfScanOpaque, count: usize) {
    for _ in 0..count {
        stats::record_distance_calc();
        opaque.stats_delta.record_distance_calc();
    }
}

fn record_posting_pages_read(opaque: &mut EcIvfScanOpaque, count: u32) {
    for _ in 0..count {
        stats::record_linear_page();
        opaque.stats_delta.record_linear_page();
    }
}

fn flush_scan_stats(opaque: &mut EcIvfScanOpaque) {
    if !opaque.rescan_called || opaque.stats_delta.is_zero() {
        return;
    }
    stats::flush_shared_delta(opaque.stats_delta);
    opaque.stats_delta.reset();
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

unsafe fn store_centroid_scores(opaque: &mut EcIvfScanOpaque, scores: &[EcIvfCentroidScore]) {
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
    opaque.next_candidate_index = 0;
}

unsafe fn free_posting_candidates(opaque: &mut EcIvfScanOpaque) {
    if !opaque.posting_candidates.is_null() {
        unsafe { pg_sys::pfree(opaque.posting_candidates.cast()) };
        opaque.posting_candidates = ptr::null_mut();
    }
    opaque.posting_candidate_count = 0;
    opaque.next_candidate_index = 0;
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
    super::options::resolve_scan_nprobe(metadata.nlists, metadata.nprobe).effective_nprobe
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

fn scan_query_values(opaque: &EcIvfScanOpaque) -> &[f32] {
    if opaque.query_values.is_null() || opaque.query_dimensions == 0 {
        pgrx::error!("ec_ivf scan query state is missing");
    }
    unsafe { std::slice::from_raw_parts(opaque.query_values, opaque.query_dimensions as usize) }
}

fn candidate_cmp(left: &EcIvfScoredCandidate, right: &EcIvfScoredCandidate) -> Ordering {
    left.score
        .total_cmp(&right.score)
        .then_with(|| left.heap_tid.block_number.cmp(&right.heap_tid.block_number))
        .then_with(|| {
            left.heap_tid
                .offset_number
                .cmp(&right.heap_tid.offset_number)
        })
}

fn probe_list_cmp(left: &EcIvfCentroidScore, right: &EcIvfCentroidScore) -> Ordering {
    right
        .score
        .total_cmp(&left.score)
        .then_with(|| left.list_id.cmp(&right.list_id))
}

fn probe_list_heap_cmp(left: &EcIvfCentroidScore, right: &EcIvfCentroidScore) -> Ordering {
    left.score
        .total_cmp(&right.score)
        .reverse()
        .then_with(|| left.list_id.cmp(&right.list_id))
}

fn select_probe_lists(scores: &[EcIvfCentroidScore], nprobe: u32) -> Vec<u32> {
    let limit = nprobe as usize;
    if limit == 0 || scores.is_empty() {
        return Vec::new();
    }
    if limit >= scores.len() {
        let mut ranked = scores.to_vec();
        ranked.sort_by(probe_list_cmp);
        return ranked.into_iter().map(|score| score.list_id).collect();
    }

    let mut retained = BinaryHeap::with_capacity(limit);
    for centroid in scores {
        let entry = ProbeListHeapEntry {
            centroid: *centroid,
        };
        if retained.len() < limit {
            retained.push(entry);
            continue;
        }
        if retained.peek().is_some_and(|worst| entry < *worst) {
            retained.pop();
            retained.push(entry);
        }
    }

    let mut ranked = retained
        .into_iter()
        .map(|entry| entry.centroid)
        .collect::<Vec<_>>();
    ranked.sort_by(probe_list_cmp);
    ranked
        .into_iter()
        .take(nprobe as usize)
        .map(|score| score.list_id)
        .collect()
}

unsafe fn materialize_probe_candidates(
    scan: pg_sys::IndexScanDesc,
    index_relation: pg_sys::Relation,
    metadata: &super::page::MetadataPage,
    index_options: &super::options::EcIvfOptions,
    opaque: &mut EcIvfScanOpaque,
    selected_lists: &[u32],
) -> Result<Vec<EcIvfScoredCandidate>, String> {
    if selected_lists.is_empty() {
        return Ok(Vec::new());
    }
    if opaque.prepared_query.is_null() {
        return Err("ec_ivf posting-list scan requires a prepared query".to_owned());
    }

    let prepared_query = unsafe { &*opaque.prepared_query };
    let quantizer = ProdQuantizer::cached(
        metadata.dimensions as usize,
        crate::DEFAULT_QUANT_BITS,
        crate::DEFAULT_QUANT_SEED,
    );
    let payload_len = crate::code_len(metadata.dimensions as usize, crate::DEFAULT_QUANT_BITS);
    let probe_plan =
        unsafe { build_selected_probe_plan(index_relation, metadata, selected_lists)? };
    let mut best_by_heap_tid = HashMap::with_capacity(probe_plan.candidate_bound);
    let posting_pages = probe_plan.posting_page_count()?;
    opaque
        .explain_counters
        .record_posting_pages_read(posting_pages);
    record_posting_pages_read(opaque, posting_pages);
    unsafe {
        super::page::visit_ivf_postings_for_block_sequence(
            index_relation,
            &probe_plan.block_sequence,
            payload_len,
            |_, posting| {
                if !probe_plan.contains_list(posting.list_id) || posting.deleted {
                    return Ok(());
                }
                let score =
                    -quantizer.score_ip_from_parts(prepared_query, posting.gamma, &posting.payload);
                record_distance_calcs(opaque, 1);
                for heap_tid in posting.heaptids {
                    opaque.explain_counters.record_candidate_scored();
                    let candidate = EcIvfScoredCandidate { heap_tid, score };
                    match best_by_heap_tid.entry(heap_tid) {
                        Entry::Occupied(mut entry) => {
                            opaque.explain_counters.record_filtered_duplicate();
                            let existing = entry.get_mut();
                            if candidate_cmp(&candidate, existing) == Ordering::Less {
                                *existing = candidate;
                            }
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(candidate);
                        }
                    }
                }
                Ok(())
            },
        )?
    };

    let mut candidates = best_by_heap_tid.into_values().collect::<Vec<_>>();
    candidates.sort_by(candidate_cmp);
    unsafe { rerank_probe_candidates(scan, metadata, index_options, opaque, &mut candidates) };
    Ok(candidates)
}

unsafe fn rerank_probe_candidates(
    scan: pg_sys::IndexScanDesc,
    metadata: &super::page::MetadataPage,
    index_options: &super::options::EcIvfOptions,
    opaque: &EcIvfScanOpaque,
    candidates: &mut Vec<EcIvfScoredCandidate>,
) {
    match metadata.rerank.v1_effective() {
        super::options::RerankMode::Auto | super::options::RerankMode::Off => {}
        super::options::RerankMode::HeapF32 => {
            let rerank_len = resolve_rerank_len(index_options.rerank_width, candidates.len());
            unsafe {
                rerank_probe_candidates_heap_f32(scan, opaque, &mut candidates[..rerank_len])
            };
            candidates[..rerank_len].sort_by(candidate_cmp);
            if index_options.rerank_width > 0 {
                candidates.truncate(rerank_len);
            }
        }
        super::options::RerankMode::SourceColumn => {
            pgrx::error!("ec_ivf rerank mode source_column is not supported yet")
        }
    }
}

fn resolve_rerank_len(rerank_width: i32, candidate_len: usize) -> usize {
    if rerank_width <= 0 {
        return candidate_len;
    }
    usize::try_from(rerank_width)
        .unwrap_or(usize::MAX)
        .min(candidate_len)
}

unsafe fn rerank_probe_candidates_heap_f32(
    scan: pg_sys::IndexScanDesc,
    opaque: &EcIvfScanOpaque,
    candidates: &mut [EcIvfScoredCandidate],
) {
    if candidates.is_empty() {
        return;
    }
    let (heap_relation, heap_relation_owned) = unsafe { resolve_scan_heap_relation(scan) };
    let snapshot = unsafe { resolve_scan_snapshot(scan) };
    let source_attribute = unsafe {
        source::resolve_indexed_ecvector_attribute(
            heap_relation,
            (*scan).indexRelation,
            "ec_ivf heap_f32 rerank indexed column",
        )
    };
    let slot = unsafe {
        source::allocate_heap_slot(
            heap_relation,
            "ec_ivf heap_f32 rerank failed to allocate a heap tuple slot",
        )
    };

    unsafe { prefetch_heap_rerank_blocks(heap_relation, candidates) };

    for candidate in candidates {
        unsafe {
            source::fetch_heap_row_version(
                heap_relation,
                candidate.heap_tid,
                snapshot,
                slot,
                "ec_ivf heap_f32 rerank source vector",
            )
        };
        let source_vector = unsafe {
            source::FlatFloat4SourceRef::from_datum(
                source::required_slot_datum(
                    slot,
                    source_attribute.attnum,
                    "ec_ivf heap_f32 rerank source vector",
                ),
                source_attribute.kind,
                "ec_ivf heap_f32 rerank source vector",
            )
        };
        candidate.score =
            source::negative_inner_product(scan_query_values(opaque), source_vector.as_slice());
        drop(source_vector);
        unsafe { pg_sys::ExecClearTuple(slot) };
    }

    unsafe { pg_sys::ExecDropSingleTupleTableSlot(slot) };
    if heap_relation_owned {
        unsafe { pg_sys::table_close(heap_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    }
}

#[cfg(feature = "pg18")]
unsafe fn prefetch_heap_rerank_blocks(
    heap_relation: pg_sys::Relation,
    candidates: &[EcIvfScoredCandidate],
) {
    let block_numbers = candidates
        .iter()
        .map(|candidate| candidate.heap_tid.block_number)
        .collect::<Vec<_>>();
    let mut state = crate::am::stream::BlockSequencePrefetchState::new(block_numbers);
    let stream = unsafe {
        pg_sys::read_stream_begin_relation(
            pg_sys::READ_STREAM_DEFAULT as i32,
            ptr::null_mut(),
            heap_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            Some(crate::am::stream::block_sequence_prefetch_cb),
            (&mut state as *mut crate::am::stream::BlockSequencePrefetchState).cast(),
            std::mem::size_of::<pg_sys::BlockNumber>(),
        )
    };

    loop {
        let mut per_buffer_data = ptr::null_mut();
        let buffer = unsafe { pg_sys::read_stream_next_buffer(stream, &mut per_buffer_data) };
        if buffer == pg_sys::InvalidBuffer as pg_sys::Buffer {
            break;
        }
        unsafe { pg_sys::ReleaseBuffer(buffer) };
    }

    unsafe { pg_sys::read_stream_end(stream) };
}

#[cfg(not(feature = "pg18"))]
unsafe fn prefetch_heap_rerank_blocks(
    heap_relation: pg_sys::Relation,
    candidates: &[EcIvfScoredCandidate],
) {
    for candidate in candidates {
        unsafe {
            pg_sys::PrefetchBuffer(
                heap_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                candidate.heap_tid.block_number,
            )
        };
    }
}

unsafe fn resolve_scan_heap_relation(scan: pg_sys::IndexScanDesc) -> (pg_sys::Relation, bool) {
    if !unsafe { (*scan).heapRelation }.is_null() {
        return (unsafe { (*scan).heapRelation }, false);
    }

    let heap_oid = unsafe { pg_sys::IndexGetRelation((*(*scan).indexRelation).rd_id, false) };
    if heap_oid == pg_sys::InvalidOid {
        pgrx::error!("ec_ivf heap_f32 rerank could not resolve heap relation");
    }
    (
        unsafe { pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) },
        true,
    )
}

unsafe fn resolve_scan_snapshot(scan: pg_sys::IndexScanDesc) -> pg_sys::Snapshot {
    if !unsafe { (*scan).xs_snapshot }.is_null() {
        return unsafe { (*scan).xs_snapshot };
    }

    let active_snapshot = unsafe { pg_sys::GetActiveSnapshot() };
    if !active_snapshot.is_null() {
        return active_snapshot;
    }

    pgrx::error!("ec_ivf heap_f32 rerank requires an executor or active snapshot");
}

fn posting_block_count(directory: &super::page::IvfListDirectoryTuple) -> Result<u32, String> {
    match (
        directory.head_block == super::page::BlockRef::INVALID,
        directory.tail_block == super::page::BlockRef::INVALID,
    ) {
        (true, true) => Ok(0),
        (false, false)
            if directory.head_block.block_number <= directory.tail_block.block_number =>
        {
            directory
                .tail_block
                .block_number
                .checked_sub(directory.head_block.block_number)
                .and_then(|delta| delta.checked_add(1))
                .ok_or_else(|| {
                    format!(
                        "ec_ivf list {} posting block count overflow",
                        directory.list_id
                    )
                })
        }
        (false, false) => Err(format!(
            "ec_ivf list {} head block {} exceeds tail block {}",
            directory.list_id, directory.head_block.block_number, directory.tail_block.block_number
        )),
        _ => Err(format!(
            "ec_ivf list {} has partial posting block refs",
            directory.list_id
        )),
    }
}

fn build_probe_block_sequence(ranges: &mut [ProbeBlockRange]) -> Result<Vec<u32>, String> {
    if ranges.is_empty() {
        return Ok(Vec::new());
    }

    ranges.sort_by(|left, right| {
        left.head_block
            .cmp(&right.head_block)
            .then_with(|| left.tail_block.cmp(&right.tail_block))
    });

    let mut blocks = Vec::new();
    let mut next_block: Option<u32> = None;
    for range in ranges {
        let start = next_block
            .map(|next| next.max(range.head_block))
            .unwrap_or(range.head_block);
        if start > range.tail_block {
            continue;
        }
        for block_number in start..=range.tail_block {
            blocks.push(block_number);
        }
        next_block = range.tail_block.checked_add(1);
    }

    Ok(blocks)
}

unsafe fn build_selected_probe_plan(
    index_relation: pg_sys::Relation,
    metadata: &super::page::MetadataPage,
    selected_lists: &[u32],
) -> Result<SelectedProbePlan, String> {
    if selected_lists.is_empty() {
        return Ok(SelectedProbePlan {
            selected_lists: Vec::new(),
            selected_list_mask: Vec::new(),
            block_sequence: Vec::new(),
            candidate_bound: 0,
        });
    }
    if metadata.nlists == 0 {
        return Err("ec_ivf selected probe plan requires nonzero nlists".to_owned());
    }
    if metadata.directory_head == ItemPointer::INVALID {
        return Err("ec_ivf metadata has lists but no directory head".to_owned());
    }

    let mut sorted_selected_lists = selected_lists.to_vec();
    sorted_selected_lists.sort_unstable();
    sorted_selected_lists.dedup();
    if sorted_selected_lists
        .iter()
        .any(|list_id| *list_id >= metadata.nlists)
    {
        return Err("ec_ivf selected list is out of range".to_owned());
    }

    let mut selected_list_mask = vec![false; metadata.nlists as usize];
    for list_id in &sorted_selected_lists {
        selected_list_mask[*list_id as usize] = true;
    }

    let mut candidate_bound = 0_usize;
    let mut ranges = Vec::new();
    let mut selected_index = 0_usize;
    let mut next_tid = metadata.directory_head;

    for expected_list_id in 0..metadata.nlists {
        let (directory, following_tid) =
            unsafe { super::page::read_ivf_list_directory_and_next(index_relation, next_tid)? };
        if directory.list_id != expected_list_id {
            return Err(format!(
                "ec_ivf directory order mismatch: got list {}, expected {}",
                directory.list_id, expected_list_id
            ));
        }

        if selected_index < sorted_selected_lists.len()
            && sorted_selected_lists[selected_index] == expected_list_id
        {
            let live_count = usize::try_from(directory.live_count)
                .map_err(|_| format!("ec_ivf list {expected_list_id} live count exceeds usize"))?;
            candidate_bound = candidate_bound
                .checked_add(live_count)
                .ok_or_else(|| "ec_ivf selected live count overflow".to_owned())?;

            if posting_block_count(&directory)? != 0 {
                ranges.push(ProbeBlockRange {
                    head_block: directory.head_block.block_number,
                    tail_block: directory.tail_block.block_number,
                });
            }
            selected_index += 1;
        }

        next_tid = following_tid;
    }

    if selected_index != sorted_selected_lists.len() {
        return Err("ec_ivf selected probe plan did not resolve every selected list".to_owned());
    }

    let block_sequence = build_probe_block_sequence(&mut ranges)?;
    Ok(SelectedProbePlan {
        selected_lists: sorted_selected_lists,
        selected_list_mask,
        block_sequence,
        candidate_bound,
    })
}

pub(crate) unsafe fn explain_counters_from_index_scan_state(
    index_state: *mut pg_sys::IndexScanState,
) -> IvfExplainCounters {
    if index_state.is_null() {
        return IvfExplainCounters::default();
    }

    let scan_desc = unsafe { (*index_state).iss_ScanDesc };
    if scan_desc.is_null() {
        return IvfExplainCounters::default();
    }

    let opaque = unsafe { (*scan_desc).opaque };
    if opaque.is_null() {
        return IvfExplainCounters::default();
    }

    unsafe { (*opaque.cast::<EcIvfScanOpaque>()).explain_counters }
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
pub(crate) struct EcIvfRescanDebugSnapshot {
    pub(crate) rescan_called: bool,
    pub(crate) query_dimensions: u16,
    pub(crate) query_values: Vec<f32>,
    pub(crate) scan_dimensions: u16,
    pub(crate) scan_nlists: u32,
    pub(crate) scan_nprobe: u32,
    pub(crate) has_prepared_query: bool,
    pub(crate) prepared_lut_len: usize,
    pub(crate) prepared_sq_len: usize,
    pub(crate) centroid_score_count: u32,
    pub(crate) posting_candidate_count: u32,
    pub(crate) selected_lists: Vec<u32>,
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_rescan_query_prep(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> EcIvfRescanDebugSnapshot {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { ec_ivf_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { ec_ivf_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let opaque = unsafe { &*(*scan).opaque.cast::<EcIvfScanOpaque>() };
    let result = EcIvfRescanDebugSnapshot {
        rescan_called: opaque.rescan_called,
        query_dimensions: opaque.query_dimensions,
        query_values: debug_scan_query(opaque),
        scan_dimensions: opaque.scan_dimensions,
        scan_nlists: opaque.scan_nlists,
        scan_nprobe: opaque.scan_nprobe,
        has_prepared_query: !opaque.prepared_query.is_null(),
        prepared_lut_len: if opaque.prepared_query.is_null() {
            0
        } else {
            unsafe { (*opaque.prepared_query).lut.len() }
        },
        prepared_sq_len: if opaque.prepared_query.is_null() {
            0
        } else {
            unsafe { (*opaque.prepared_query).sq.len() }
        },
        centroid_score_count: opaque.centroid_score_count,
        posting_candidate_count: opaque.posting_candidate_count,
        selected_lists: debug_selected_lists(opaque),
    };

    unsafe { ec_ivf_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_gettuple_outputs(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (Vec<(u32, u16, f32)>, bool) {
    let state = unsafe { debug_begin_heap_backed_scan(index_oid) };
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { pg_sys::index_rescan(state.scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let mut outputs = Vec::new();
    while unsafe { ec_ivf_amgettuple(state.scan, pg_sys::ScanDirection::ForwardScanDirection) } {
        let (block_number, offset_number) =
            pgrx::itemptr::item_pointer_get_both(unsafe { (*state.scan).xs_heaptid });
        let score = if unsafe { (*state.scan).xs_orderbyvals.is_null() }
            || unsafe { (*state.scan).xs_orderbynulls.is_null() }
            || unsafe { *(*state.scan).xs_orderbynulls }
        {
            pgrx::error!("ec_ivf debug gettuple output is missing order-by score");
        } else {
            f32::from_datum(unsafe { *(*state.scan).xs_orderbyvals }, false)
                .expect("score datum should decode as f32")
        };
        outputs.push((block_number, offset_number, score));
    }
    let orderby_cleared = if unsafe { (*state.scan).xs_orderbynulls.is_null() } {
        false
    } else {
        unsafe { *(*state.scan).xs_orderbynulls }
    };

    unsafe { debug_end_heap_backed_scan(state) };
    (outputs, orderby_cleared)
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
pub(crate) unsafe fn debug_ec_ivf_rerank_mode(index_oid: pg_sys::Oid) -> &'static str {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let metadata = unsafe { super::page::read_metadata_page(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    metadata.rerank.reloption_name()
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

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_directory_entry(
    index_oid: pg_sys::Oid,
    list_id: u32,
) -> (u32, u32, u64, u64, u64) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let metadata = unsafe { super::page::read_metadata_page(index_relation) };
    let directories = unsafe { load_directory_entries(index_relation, &metadata) }
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    let directory = directories
        .get(list_id as usize)
        .unwrap_or_else(|| pgrx::error!("ec_ivf directory list {list_id} is out of range"));
    let result = (
        directory.head_block.block_number,
        directory.tail_block.block_number,
        directory.live_count,
        directory.dead_count,
        directory.inserted_since_build,
    );

    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result
}

#[cfg(test)]
mod tests {
    use super::{
        build_probe_block_sequence, candidate_cmp, select_probe_lists, EcIvfCentroidScore,
        EcIvfScoredCandidate, ProbeBlockRange,
    };
    use crate::storage::page::ItemPointer;
    use std::collections::BinaryHeap;

    #[derive(Debug, Clone, Copy)]
    struct CandidateHeapEntry {
        candidate: EcIvfScoredCandidate,
    }

    impl PartialEq for CandidateHeapEntry {
        fn eq(&self, other: &Self) -> bool {
            candidate_cmp(&self.candidate, &other.candidate) == std::cmp::Ordering::Equal
        }
    }

    impl Eq for CandidateHeapEntry {}

    impl PartialOrd for CandidateHeapEntry {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for CandidateHeapEntry {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            candidate_cmp(&self.candidate, &other.candidate)
        }
    }

    #[derive(Debug)]
    struct CandidateTopK {
        limit: usize,
        retained: BinaryHeap<CandidateHeapEntry>,
    }

    impl CandidateTopK {
        fn new(limit: usize) -> Self {
            Self {
                limit,
                retained: BinaryHeap::with_capacity(limit),
            }
        }

        fn push(&mut self, candidate: EcIvfScoredCandidate) {
            if self.limit == 0 {
                return;
            }
            let entry = CandidateHeapEntry { candidate };
            if self.retained.len() < self.limit {
                self.retained.push(entry);
                return;
            }
            if self.retained.peek().is_some_and(|worst| entry < *worst) {
                self.retained.pop();
                self.retained.push(entry);
            }
        }

        fn into_sorted_candidates(self) -> Vec<EcIvfScoredCandidate> {
            let mut candidates = self
                .retained
                .into_iter()
                .map(|entry| entry.candidate)
                .collect::<Vec<_>>();
            candidates.sort_by(candidate_cmp);
            candidates
        }
    }

    fn candidate(block_number: u32, offset_number: u16, score: f32) -> EcIvfScoredCandidate {
        EcIvfScoredCandidate {
            heap_tid: ItemPointer {
                block_number,
                offset_number,
            },
            score,
        }
    }

    fn centroid(list_id: u32, score: f32) -> EcIvfCentroidScore {
        EcIvfCentroidScore { list_id, score }
    }

    fn range(head_block: u32, tail_block: u32) -> ProbeBlockRange {
        ProbeBlockRange {
            head_block,
            tail_block,
        }
    }

    #[test]
    fn candidate_top_k_keeps_best_scores_in_output_order() {
        let mut top_k = CandidateTopK::new(2);
        top_k.push(candidate(1, 1, 3.0));
        top_k.push(candidate(1, 2, 1.0));
        top_k.push(candidate(1, 3, 2.0));

        let retained = top_k.into_sorted_candidates();
        assert_eq!(retained.len(), 2);
        assert_eq!(retained[0].heap_tid.offset_number, 2);
        assert_eq!(retained[0].score, 1.0);
        assert_eq!(retained[1].heap_tid.offset_number, 3);
        assert_eq!(retained[1].score, 2.0);
    }

    #[test]
    fn candidate_top_k_uses_heap_tid_as_score_tiebreaker() {
        let mut top_k = CandidateTopK::new(2);
        top_k.push(candidate(1, 3, 1.0));
        top_k.push(candidate(1, 1, 1.0));
        top_k.push(candidate(1, 2, 1.0));

        let retained = top_k.into_sorted_candidates();
        assert_eq!(retained.len(), 2);
        assert_eq!(retained[0].heap_tid.offset_number, 1);
        assert_eq!(retained[1].heap_tid.offset_number, 2);
    }

    #[test]
    fn select_probe_lists_keeps_best_nprobe_without_full_sort_requirement() {
        let selected = select_probe_lists(
            &[
                centroid(0, 0.1),
                centroid(1, 0.9),
                centroid(2, 0.2),
                centroid(3, 0.8),
                centroid(4, 0.3),
            ],
            2,
        );

        assert_eq!(selected, vec![1, 3]);
    }

    #[test]
    fn select_probe_lists_uses_list_id_as_tiebreaker() {
        let selected = select_probe_lists(
            &[
                centroid(9, 0.5),
                centroid(3, 0.7),
                centroid(7, 0.7),
                centroid(1, 0.7),
            ],
            3,
        );

        assert_eq!(selected, vec![1, 3, 7]);
    }

    #[test]
    fn build_probe_block_sequence_merges_overlapping_ranges_once() {
        let mut ranges = vec![range(12, 14), range(10, 12), range(18, 19)];

        let sequence = build_probe_block_sequence(&mut ranges).unwrap();

        assert_eq!(sequence, vec![10, 11, 12, 13, 14, 18, 19]);
    }

    #[test]
    fn build_probe_block_sequence_skips_empty_lists() {
        let mut ranges = vec![range(8, 9)];

        let sequence = build_probe_block_sequence(&mut ranges).unwrap();

        assert_eq!(sequence, vec![8, 9]);
    }
}
