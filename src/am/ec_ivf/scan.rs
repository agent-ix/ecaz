use std::cmp::Ordering;
use std::collections::{hash_map::Entry, BinaryHeap, HashMap};
use std::ptr;

use pgrx::{pg_sys, FromDatum, IntoDatum, PgBox};

use crate::am::common::explain::IvfExplainCounters;
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

#[derive(Debug, Clone, Copy)]
struct CandidateHeapEntry {
    candidate: EcIvfScoredCandidate,
}

impl PartialEq for CandidateHeapEntry {
    fn eq(&self, other: &Self) -> bool {
        candidate_cmp(&self.candidate, &other.candidate) == Ordering::Equal
    }
}

impl Eq for CandidateHeapEntry {}

impl PartialOrd for CandidateHeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CandidateHeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
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
        if self
            .retained
            .peek()
            .is_some_and(|worst| entry < *worst)
        {
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

    let candidate =
        unsafe { *opaque.posting_candidates.add(opaque.next_candidate_index as usize) };
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

fn candidate_cmp(left: &EcIvfScoredCandidate, right: &EcIvfScoredCandidate) -> Ordering {
    left.score
        .total_cmp(&right.score)
        .then_with(|| left.heap_tid.block_number.cmp(&right.heap_tid.block_number))
        .then_with(|| left.heap_tid.offset_number.cmp(&right.heap_tid.offset_number))
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
    index_relation: pg_sys::Relation,
    metadata: &super::page::MetadataPage,
    opaque: &mut EcIvfScanOpaque,
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
    let candidate_bound = selected_list_live_count_bound(&directories, selected_lists)?;
    let payload_len = crate::code_len(metadata.dimensions as usize, crate::DEFAULT_QUANT_BITS);
    let mut best_by_heap_tid = HashMap::new();
    for list_id in selected_lists {
        let directory = directories
            .get(*list_id as usize)
            .ok_or_else(|| format!("ec_ivf selected list {list_id} is out of range"))?;
        let posting_pages = posting_block_count(directory)?;
        opaque
            .explain_counters
            .record_posting_pages_read(posting_pages);
        record_posting_pages_read(opaque, posting_pages);
        unsafe {
            super::page::visit_ivf_postings_for_list_blocks(
                index_relation,
                *list_id,
                directory.head_block,
                directory.tail_block,
                payload_len,
                |_, posting| {
                    if posting.deleted {
                        return Ok(());
                    }
                    let score = -quantizer.score_ip_from_parts(
                        prepared_query,
                        posting.gamma,
                        &posting.payload,
                    );
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
        }
    }

    let mut top_k = CandidateTopK::new(candidate_bound);
    for candidate in best_by_heap_tid.into_values() {
        top_k.push(candidate);
    }
    Ok(top_k.into_sorted_candidates())
}

fn posting_block_count(directory: &super::page::IvfListDirectoryTuple) -> Result<u32, String> {
    match (
        directory.head_block == super::page::BlockRef::INVALID,
        directory.tail_block == super::page::BlockRef::INVALID,
    ) {
        (true, true) => Ok(0),
        (false, false) if directory.head_block.block_number <= directory.tail_block.block_number => {
            directory
                .tail_block
                .block_number
                .checked_sub(directory.head_block.block_number)
                .and_then(|delta| delta.checked_add(1))
                .ok_or_else(|| format!("ec_ivf list {} posting block count overflow", directory.list_id))
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

fn selected_list_live_count_bound(
    directories: &[super::page::IvfListDirectoryTuple],
    selected_lists: &[u32],
) -> Result<usize, String> {
    let mut total = 0_usize;
    for list_id in selected_lists {
        let directory = directories
            .get(*list_id as usize)
            .ok_or_else(|| format!("ec_ivf selected list {list_id} is out of range"))?;
        let live_count = usize::try_from(directory.live_count)
            .map_err(|_| format!("ec_ivf list {list_id} live count exceeds usize"))?;
        total = total
            .checked_add(live_count)
            .ok_or_else(|| "ec_ivf selected live count overflow".to_owned())?;
    }
    Ok(total)
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
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { ec_ivf_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { ec_ivf_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let mut outputs = Vec::new();
    while unsafe {
        ec_ivf_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection)
    } {
        let (block_number, offset_number) =
            pgrx::itemptr::item_pointer_get_both(unsafe { (*scan).xs_heaptid });
        let score = if unsafe { (*scan).xs_orderbyvals.is_null() }
            || unsafe { (*scan).xs_orderbynulls.is_null() }
            || unsafe { *(*scan).xs_orderbynulls }
        {
            pgrx::error!("ec_ivf debug gettuple output is missing order-by score");
        } else {
            f32::from_datum(unsafe { *(*scan).xs_orderbyvals }, false)
                .expect("score datum should decode as f32")
        };
        outputs.push((block_number, offset_number, score));
    }
    let orderby_cleared = if unsafe { (*scan).xs_orderbynulls.is_null() } {
        false
    } else {
        unsafe { *(*scan).xs_orderbynulls }
    };

    unsafe { ec_ivf_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
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
    use super::{select_probe_lists, CandidateTopK, EcIvfCentroidScore, EcIvfScoredCandidate};
    use crate::storage::page::ItemPointer;

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
}
