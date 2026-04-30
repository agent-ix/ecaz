//! Insert-time payload derivation support for `ec_diskann`.
//!
//! Phase 7 needs two insert-side seams before the full graph-mutation
//! path lands:
//!
//! 1. given the built index's metadata + persisted grouped codebooks,
//!    derive the new node's persisted payload (`search_code`,
//!    optional binary sidecar words) from an incoming source vector
//! 2. bootstrap the first live row into an otherwise-empty index
//!
//! This module owns both seams. The general non-empty pgrx callback
//! still lives in `routine.rs`; later slices will move more of that
//! logic here once the page-write / backlink / overflow story is
//! implemented.

use std::{cmp::Ordering, ptr, slice};

use pgrx::pg_sys;

use crate::am::common::training;
use crate::quant::grouped_pq::{encode_grouped_pq, GROUPED_PQ_CENTROIDS};
use crate::quant::prod::ProdQuantizer;
use crate::storage::page::{
    element_or_neighbor_tuple_fits, raw_tuple_storage_bytes, DataPageChain, ItemPointer,
    FIRST_DATA_BLOCK_NUMBER, HEAPTID_INLINE_CAPACITY, ITEM_POINTER_BYTES,
};
use crate::storage::wal;
use crate::{DEFAULT_QUANT_BITS, DEFAULT_QUANT_SEED};

use super::scan_query::{encode_query_srht, read_grouped_codebook_chain};
use super::{
    ambuild,
    build::{build_and_persist_vamana, BuildOutput, BuildParams},
    options,
    page::{
        VamanaMetadataPage, PAYLOAD_FLAG_BINARY_SIDECAR, VAMANA_METADATA_BYTES,
        VAMANA_SEARCH_CODEC_GROUPED_PQ, VAMANA_TRANSFORM_KIND_SRHT,
    },
    persist::{stage_grouped_codebook_chain, NodePayload},
    reader::PersistedGraphReader,
    scan_state,
    tuple::VamanaNodeTuple,
    vamana::{robust_prune, Candidate},
    ECDISKANN_UNIT_NORM_DISTANCE_BIAS,
};

const EMPTY_INSERT_BOOTSTRAP_KMEANS_ITERS: usize = 8;
const P_NEW: pg_sys::BlockNumber = u32::MAX;
pub(super) const MAX_BACKLINK_REPLAN_PASSES: usize = 3;
const TQ_VAMANA_OVERFLOW_TAG: u8 = 0x08;
const VAMANA_OVERFLOW_HEADER_BYTES: usize = 1 + 2 + ITEM_POINTER_BYTES + ITEM_POINTER_BYTES;
const VAMANA_OVERFLOW_HEAPTID_CAPACITY: usize = HEAPTID_INLINE_CAPACITY;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DerivedInsertPayload {
    pub(super) binary_words: Vec<u64>,
    pub(super) search_code: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ForwardNeighborCandidate {
    pub(super) tid: ItemPointer,
    pub(super) source_vector: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VamanaOverflowTuple {
    owner_tid: ItemPointer,
    nexttid: ItemPointer,
    heap_tids: Vec<ItemPointer>,
    heap_tid_count: u16,
}

impl VamanaOverflowTuple {
    fn encoded_len() -> usize {
        VAMANA_OVERFLOW_HEADER_BYTES + VAMANA_OVERFLOW_HEAPTID_CAPACITY * ITEM_POINTER_BYTES
    }

    fn placeholder(owner_tid: ItemPointer) -> Self {
        Self {
            owner_tid,
            nexttid: ItemPointer::INVALID,
            heap_tids: vec![ItemPointer::INVALID; VAMANA_OVERFLOW_HEAPTID_CAPACITY],
            heap_tid_count: 0,
        }
    }

    fn contains(&self, heap_tid: ItemPointer) -> bool {
        self.heap_tids
            .iter()
            .take(self.heap_tid_count as usize)
            .any(|tid| *tid == heap_tid)
    }

    fn push_heap_tid(&mut self, heap_tid: ItemPointer) -> Result<(), String> {
        if heap_tid == ItemPointer::INVALID {
            return Err("ec_diskann overflow tuple cannot store INVALID heap tid".into());
        }
        if self.contains(heap_tid) {
            return Ok(());
        }
        let next_index = usize::from(self.heap_tid_count);
        if next_index >= self.heap_tids.len() {
            return Err(format!(
                "ec_diskann overflow tuple is full at capacity {}",
                self.heap_tids.len()
            ));
        }
        self.heap_tids[next_index] = heap_tid;
        self.heap_tid_count = u16::try_from(next_index + 1).expect("overflow count fits in u16");
        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        if self.owner_tid == ItemPointer::INVALID {
            return Err("ec_diskann overflow tuple requires a valid owner_tid".into());
        }
        if self.heap_tids.len() != VAMANA_OVERFLOW_HEAPTID_CAPACITY {
            return Err(format!(
                "ec_diskann overflow tuple heap_tids length mismatch: got {}, expected {}",
                self.heap_tids.len(),
                VAMANA_OVERFLOW_HEAPTID_CAPACITY
            ));
        }
        if usize::from(self.heap_tid_count) > self.heap_tids.len() {
            return Err(format!(
                "ec_diskann overflow tuple heap_tid_count {} exceeds capacity {}",
                self.heap_tid_count,
                self.heap_tids.len()
            ));
        }
        Ok(())
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut out = Vec::with_capacity(Self::encoded_len());
        out.push(TQ_VAMANA_OVERFLOW_TAG);
        out.extend_from_slice(&self.heap_tid_count.to_le_bytes());
        self.owner_tid.encode_into(&mut out);
        self.nexttid.encode_into(&mut out);
        for heap_tid in &self.heap_tids {
            heap_tid.encode_into(&mut out);
        }
        debug_assert_eq!(out.len(), Self::encoded_len());
        Ok(out)
    }

    fn decode(input: &[u8]) -> Result<Self, String> {
        let expected_len = Self::encoded_len();
        if input.len() != expected_len {
            return Err(format!(
                "ec_diskann overflow tuple length mismatch: got {}, expected {expected_len}",
                input.len()
            ));
        }
        if input[0] != TQ_VAMANA_OVERFLOW_TAG {
            return Err(format!(
                "invalid ec_diskann overflow tuple tag: got 0x{:02x}, expected 0x{:02x}",
                input[0], TQ_VAMANA_OVERFLOW_TAG
            ));
        }
        let heap_tid_count =
            u16::from_le_bytes(input[1..3].try_into().expect("heap tid count bytes"));
        let owner_tid = ItemPointer::decode(&input[3..3 + ITEM_POINTER_BYTES])?;
        let nexttid = ItemPointer::decode(&input[9..9 + ITEM_POINTER_BYTES])?;
        let mut cursor = VAMANA_OVERFLOW_HEADER_BYTES;
        let mut heap_tids = Vec::with_capacity(VAMANA_OVERFLOW_HEAPTID_CAPACITY);
        for _ in 0..VAMANA_OVERFLOW_HEAPTID_CAPACITY {
            heap_tids.push(ItemPointer::decode(
                &input[cursor..cursor + ITEM_POINTER_BYTES],
            )?);
            cursor += ITEM_POINTER_BYTES;
        }
        let tuple = Self {
            owner_tid,
            nexttid,
            heap_tids,
            heap_tid_count,
        };
        tuple.validate()?;
        Ok(tuple)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OverflowTupleRef {
    tid: ItemPointer,
    tuple: VamanaOverflowTuple,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DuplicateBindPlan {
    AlreadyBound,
    Apply {
        append_tuple: Option<VamanaOverflowTuple>,
        patches: Vec<DuplicateBindPatch>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DuplicateBindPatch {
    target_tid: ItemPointer,
    kind: DuplicateBindPatchKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DuplicateBindPatchKind {
    SetNodeOverflowFlag,
    AppendHeapTidToOverflow {
        expected_nexttid: ItemPointer,
        expected_heap_tid_count: u16,
        heap_tid: ItemPointer,
    },
    SetOverflowNextTid {
        expected_nexttid: ItemPointer,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DuplicateBindApplyOutcome {
    NoChange,
    Changed,
    RetryReplan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DuplicateBindResult {
    AlreadyBound,
    Bound,
}

pub(super) fn derive_insert_payload_from_persisted(
    metadata: &VamanaMetadataPage,
    chain: &DataPageChain,
    source_vector: &[f32],
) -> Result<DerivedInsertPayload, String> {
    let dimensions = usize::from(metadata.dimensions);
    if dimensions == 0 {
        return Err("ec_diskann insert payload derivation requires non-zero dimensions".into());
    }
    if source_vector.len() != dimensions {
        return Err(format!(
            "ec_diskann insert payload dimension mismatch: source dim {}, index dim {}",
            source_vector.len(),
            dimensions
        ));
    }
    if metadata.transform_kind != VAMANA_TRANSFORM_KIND_SRHT {
        return Err(format!(
            "ec_diskann insert payload derivation only supports SRHT transform kind {}, got {}",
            VAMANA_TRANSFORM_KIND_SRHT, metadata.transform_kind
        ));
    }
    if metadata.search_codec_kind != VAMANA_SEARCH_CODEC_GROUPED_PQ {
        return Err(format!(
            "ec_diskann insert payload derivation only supports grouped-PQ codec kind {}, got {}",
            VAMANA_SEARCH_CODEC_GROUPED_PQ, metadata.search_codec_kind
        ));
    }

    let group_count = usize::from(metadata.search_subvector_count);
    let group_size = usize::from(metadata.search_subvector_dim);
    if group_count == 0 || group_size == 0 {
        return Err(
            "ec_diskann insert payload derivation requires non-zero grouped search shape".into(),
        );
    }
    if metadata.grouped_codebook_head == ItemPointer::INVALID {
        return Err(
            "ec_diskann insert payload derivation requires persisted grouped codebooks".into(),
        );
    }

    let centroid_count = group_size * GROUPED_PQ_CENTROIDS;
    let flat_codebooks = read_grouped_codebook_chain(
        chain,
        metadata.grouped_codebook_head,
        group_count,
        centroid_count,
    )?;

    let rotated = encode_query_srht(source_vector, dimensions, metadata.seed);
    let expected_rotated_len = group_count
        .checked_mul(group_size)
        .ok_or_else(|| "ec_diskann grouped search shape overflows usize".to_owned())?;
    if rotated.len() != expected_rotated_len {
        return Err(format!(
            "ec_diskann insert payload rotated query length mismatch: got {}, expected {} from metadata",
            rotated.len(),
            expected_rotated_len
        ));
    }

    let codebook_chunk_len = GROUPED_PQ_CENTROIDS * group_size;
    let search_code = encode_grouped_pq(
        &rotated,
        flat_codebooks.chunks_exact(codebook_chunk_len),
        group_size,
    );
    let expected_search_code_len = group_count.div_ceil(2);
    if search_code.len() != expected_search_code_len {
        return Err(format!(
            "ec_diskann insert payload search code length mismatch: got {}, expected {}",
            search_code.len(),
            expected_search_code_len
        ));
    }

    let binary_words = if (metadata.payload_flags & PAYLOAD_FLAG_BINARY_SIDECAR) != 0 {
        let quantizer = ProdQuantizer::cached(dimensions, DEFAULT_QUANT_BITS, metadata.seed);
        let encoded = quantizer.encode(source_vector);
        let mut code = encoded.mse_packed;
        code.extend_from_slice(&encoded.qjl_packed);
        training::derive_persisted_binary_words(&quantizer, &code)
    } else {
        Vec::new()
    };

    Ok(DerivedInsertPayload {
        binary_words,
        search_code,
    })
}

pub(super) fn cmp_item_pointer_physical(left: &ItemPointer, right: &ItemPointer) -> Ordering {
    left.block_number
        .cmp(&right.block_number)
        .then_with(|| left.offset_number.cmp(&right.offset_number))
}

fn overflow_tuple_refs_for_owner(
    chain: &DataPageChain,
    owner_tid: ItemPointer,
) -> Result<Vec<OverflowTupleRef>, String> {
    let mut matches = Vec::new();
    for page in chain.pages() {
        for offset_number in 1..=page.tuple_count() {
            let tid = ItemPointer {
                block_number: page.block_number(),
                offset_number: offset_number as u16,
            };
            let raw = page.raw_tuple(tid)?;
            if raw.first().copied() != Some(TQ_VAMANA_OVERFLOW_TAG) {
                continue;
            }
            let tuple = VamanaOverflowTuple::decode(raw)?;
            if tuple.owner_tid == owner_tid {
                matches.push(OverflowTupleRef { tid, tuple });
            }
        }
    }
    matches.sort_unstable_by(|left, right| cmp_item_pointer_physical(&left.tid, &right.tid));
    Ok(matches)
}

pub(super) fn bound_heap_tids_for_owner(
    chain: &DataPageChain,
    owner_tid: ItemPointer,
    primary_heaptid: ItemPointer,
) -> Result<Vec<ItemPointer>, String> {
    if owner_tid == ItemPointer::INVALID {
        return Err("ec_diskann bound heap tid expansion requires a valid owner tid".into());
    }
    if primary_heaptid == ItemPointer::INVALID {
        return Err("ec_diskann bound heap tid expansion requires a valid primary heap tid".into());
    }

    let mut heap_tids = vec![primary_heaptid];
    for overflow in overflow_tuple_refs_for_owner(chain, owner_tid)? {
        heap_tids.extend(
            overflow
                .tuple
                .heap_tids
                .iter()
                .take(overflow.tuple.heap_tid_count as usize)
                .copied(),
        );
    }
    Ok(heap_tids)
}

fn rewrite_overflow_tuple_in_chain(
    chain: &mut DataPageChain,
    tid: ItemPointer,
    tuple: &VamanaOverflowTuple,
) -> Result<(), String> {
    let encoded = tuple.encode()?;
    let page = chain.get_page_mut(tid.block_number).ok_or_else(|| {
        format!(
            "ec_diskann overflow rewrite could not find page {} for ({},{})",
            tid.block_number, tid.block_number, tid.offset_number
        )
    })?;
    page.update_raw_tuple(tid, encoded)
}

fn rewrite_node_tuple_in_chain(
    chain: &mut DataPageChain,
    graph_degree_r: u16,
    binary_word_count: usize,
    search_code_len: usize,
    tid: ItemPointer,
    tuple: &VamanaNodeTuple,
) -> Result<(), String> {
    let encoded = tuple.encode(graph_degree_r, binary_word_count, search_code_len)?;
    let page = chain.get_page_mut(tid.block_number).ok_or_else(|| {
        format!(
            "ec_diskann node rewrite could not find page {} for ({},{})",
            tid.block_number, tid.block_number, tid.offset_number
        )
    })?;
    page.update_raw_tuple(tid, encoded)
}

pub(super) fn stage_overflow_heap_tids_in_chain(
    chain: &mut DataPageChain,
    graph_degree_r: u16,
    binary_word_count: usize,
    search_code_len: usize,
    owner_tid: ItemPointer,
    overflow_heap_tids: &[ItemPointer],
) -> Result<(), String> {
    if overflow_heap_tids.is_empty() {
        return Ok(());
    }
    if owner_tid == ItemPointer::INVALID {
        return Err("ec_diskann overflow staging requires a valid owner_tid".into());
    }

    let owner_raw = {
        let page = chain.get_page(owner_tid.block_number).ok_or_else(|| {
            format!(
                "ec_diskann overflow staging could not find page {} for ({},{})",
                owner_tid.block_number, owner_tid.block_number, owner_tid.offset_number
            )
        })?;
        page.raw_tuple(owner_tid)?.to_vec()
    };
    let mut owner_tuple = VamanaNodeTuple::decode(
        &owner_raw,
        graph_degree_r,
        binary_word_count,
        search_code_len,
    )?;
    if owner_tuple.primary_heaptid == ItemPointer::INVALID {
        return Err(format!(
            "ec_diskann overflow staging owner ({},{}) has no primary heap tid",
            owner_tid.block_number, owner_tid.offset_number
        ));
    }
    if owner_tuple.has_overflow_heaptids {
        return Err(format!(
            "ec_diskann overflow staging owner ({},{}) already has overflow heap tids",
            owner_tid.block_number, owner_tid.offset_number
        ));
    }
    if overflow_heap_tids.contains(&owner_tuple.primary_heaptid) {
        return Err(format!(
            "ec_diskann overflow staging owner ({},{}) received a duplicate primary heap tid",
            owner_tid.block_number, owner_tid.offset_number
        ));
    }

    let mut overflow_tuples = Vec::new();
    for chunk in overflow_heap_tids.chunks(VAMANA_OVERFLOW_HEAPTID_CAPACITY) {
        let mut tuple = VamanaOverflowTuple::placeholder(owner_tid);
        for heap_tid in chunk {
            tuple.push_heap_tid(*heap_tid)?;
        }
        let tid = chain.insert_raw_tuple(tuple.encode()?)?;
        overflow_tuples.push((tid, tuple));
    }

    for index in 0..overflow_tuples.len().saturating_sub(1) {
        overflow_tuples[index].1.nexttid = overflow_tuples[index + 1].0;
        rewrite_overflow_tuple_in_chain(
            chain,
            overflow_tuples[index].0,
            &overflow_tuples[index].1,
        )?;
    }

    owner_tuple.has_overflow_heaptids = true;
    rewrite_node_tuple_in_chain(
        chain,
        graph_degree_r,
        binary_word_count,
        search_code_len,
        owner_tid,
        &owner_tuple,
    )
}

pub(super) fn vacuum_bound_heap_rows<P>(
    chain: &mut DataPageChain,
    owner_tid: ItemPointer,
    tuple: &mut VamanaNodeTuple,
    dead_pred: P,
) -> Result<usize, String>
where
    P: Fn(ItemPointer) -> bool,
{
    if owner_tid == ItemPointer::INVALID {
        return Err("ec_diskann vacuum bound heap rows requires a valid owner tid".into());
    }

    let overflow_refs = overflow_tuple_refs_for_owner(chain, owner_tid)?;
    let mut removed_heap_tids = 0usize;

    let mut primary_heaptid = if tuple.primary_heaptid != ItemPointer::INVALID {
        if dead_pred(tuple.primary_heaptid) {
            removed_heap_tids += 1;
            None
        } else {
            Some(tuple.primary_heaptid)
        }
    } else {
        None
    };

    let mut live_overflow_heap_tids = Vec::new();
    for overflow_ref in &overflow_refs {
        for heap_tid in overflow_ref
            .tuple
            .heap_tids
            .iter()
            .take(overflow_ref.tuple.heap_tid_count as usize)
            .copied()
        {
            if dead_pred(heap_tid) {
                removed_heap_tids += 1;
            } else {
                live_overflow_heap_tids.push(heap_tid);
            }
        }
    }

    if primary_heaptid.is_none() && !live_overflow_heap_tids.is_empty() {
        primary_heaptid = Some(live_overflow_heap_tids.remove(0));
    }

    tuple.primary_heaptid = primary_heaptid.unwrap_or(ItemPointer::INVALID);
    tuple.has_overflow_heaptids = !live_overflow_heap_tids.is_empty();

    let used_overflow_tuple_count = live_overflow_heap_tids
        .len()
        .div_ceil(VAMANA_OVERFLOW_HEAPTID_CAPACITY);
    for (index, overflow_ref) in overflow_refs.iter().enumerate() {
        let start = index * VAMANA_OVERFLOW_HEAPTID_CAPACITY;
        let end = live_overflow_heap_tids
            .len()
            .min(start + VAMANA_OVERFLOW_HEAPTID_CAPACITY);
        let mut updated = VamanaOverflowTuple::placeholder(owner_tid);
        for heap_tid in &live_overflow_heap_tids[start..end] {
            updated.push_heap_tid(*heap_tid)?;
        }
        if index + 1 < used_overflow_tuple_count {
            updated.nexttid = overflow_refs[index + 1].tid;
        }
        rewrite_overflow_tuple_in_chain(chain, overflow_ref.tid, &updated)?;
    }

    Ok(removed_heap_tids)
}

fn plan_duplicate_bind(
    existing_node_tid: ItemPointer,
    existing_node: &VamanaNodeTuple,
    overflow_refs: &[OverflowTupleRef],
    new_heap_tid: ItemPointer,
) -> Result<DuplicateBindPlan, String> {
    if existing_node_tid == ItemPointer::INVALID {
        return Err("ec_diskann duplicate bind requires a valid node tid".into());
    }
    if new_heap_tid == ItemPointer::INVALID {
        return Err("ec_diskann duplicate bind requires a valid heap tid".into());
    }
    if existing_node.primary_heaptid == new_heap_tid
        || overflow_refs
            .iter()
            .any(|overflow| overflow.tuple.contains(new_heap_tid))
    {
        return Ok(DuplicateBindPlan::AlreadyBound);
    }

    let ensure_node_overflow = !existing_node.has_overflow_heaptids;
    let mut patches = Vec::new();
    if ensure_node_overflow {
        patches.push(DuplicateBindPatch {
            target_tid: existing_node_tid,
            kind: DuplicateBindPatchKind::SetNodeOverflowFlag,
        });
    }

    if let Some(tail) = overflow_refs.last() {
        if usize::from(tail.tuple.heap_tid_count) < tail.tuple.heap_tids.len() {
            patches.push(DuplicateBindPatch {
                target_tid: tail.tid,
                kind: DuplicateBindPatchKind::AppendHeapTidToOverflow {
                    expected_nexttid: tail.tuple.nexttid,
                    expected_heap_tid_count: tail.tuple.heap_tid_count,
                    heap_tid: new_heap_tid,
                },
            });
            return Ok(DuplicateBindPlan::Apply {
                append_tuple: None,
                patches,
            });
        }

        let mut append_tuple = VamanaOverflowTuple::placeholder(existing_node_tid);
        append_tuple.push_heap_tid(new_heap_tid)?;
        patches.push(DuplicateBindPatch {
            target_tid: tail.tid,
            kind: DuplicateBindPatchKind::SetOverflowNextTid {
                expected_nexttid: tail.tuple.nexttid,
            },
        });
        return Ok(DuplicateBindPlan::Apply {
            append_tuple: Some(append_tuple),
            patches,
        });
    }

    let mut append_tuple = VamanaOverflowTuple::placeholder(existing_node_tid);
    append_tuple.push_heap_tid(new_heap_tid)?;
    Ok(DuplicateBindPlan::Apply {
        append_tuple: Some(append_tuple),
        patches,
    })
}

unsafe fn apply_duplicate_bind_patches(
    index_relation: pg_sys::Relation,
    metadata: &VamanaMetadataPage,
    patches: &[DuplicateBindPatch],
    appended_overflow_tid: ItemPointer,
) -> Result<DuplicateBindApplyOutcome, String> {
    if patches.is_empty() {
        return Ok(DuplicateBindApplyOutcome::NoChange);
    }

    let mut sorted = patches.to_vec();
    sorted.sort_unstable_by(|left, right| {
        cmp_item_pointer_physical(&left.target_tid, &right.target_tid)
    });
    sorted.dedup_by(|left, right| left.target_tid == right.target_tid && left.kind == right.kind);

    let binary_word_count = scan_state::metadata_binary_word_count(metadata);
    let search_code_len = scan_state::metadata_search_code_len(metadata);
    let mut changed_any = false;
    let mut start = 0usize;

    while start < sorted.len() {
        let block_number = sorted[start].target_tid.block_number;
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        if !unsafe { pg_sys::BufferIsValid(buffer) } {
            return Err(format!(
                "ec_diskann duplicate bind could not open target block {block_number}"
            ));
        }

        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
        let writable_page =
            unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
        let mut page_changed = false;
        let mut page_retry = false;

        let page_result = (|| -> Result<(), String> {
            while start < sorted.len() && sorted[start].target_tid.block_number == block_number {
                let patch = &sorted[start];
                start += 1;

                let (tuple_ptr, tuple_len) =
                    unsafe { page_tuple_location(writable_page, page_size, patch.target_tid)? };
                let tuple_bytes =
                    unsafe { slice::from_raw_parts(tuple_ptr.cast_const(), tuple_len) };

                match &patch.kind {
                    DuplicateBindPatchKind::SetNodeOverflowFlag => {
                        let mut tuple = VamanaNodeTuple::decode(
                            tuple_bytes,
                            metadata.graph_degree_r,
                            binary_word_count,
                            search_code_len,
                        )?;
                        if tuple.has_overflow_heaptids {
                            continue;
                        }
                        tuple.has_overflow_heaptids = true;
                        let encoded = tuple.encode(
                            metadata.graph_degree_r,
                            binary_word_count,
                            search_code_len,
                        )?;
                        if encoded.len() != tuple_len {
                            return Err(format!(
                                "ec_diskann duplicate bind node tuple size changed from {} to {} at ({},{})",
                                tuple_len,
                                encoded.len(),
                                patch.target_tid.block_number,
                                patch.target_tid.offset_number
                            ));
                        }
                        unsafe {
                            ptr::copy_nonoverlapping(encoded.as_ptr(), tuple_ptr, encoded.len())
                        };
                        page_changed = true;
                        changed_any = true;
                    }
                    DuplicateBindPatchKind::AppendHeapTidToOverflow {
                        expected_nexttid,
                        expected_heap_tid_count,
                        heap_tid,
                    } => {
                        let mut tuple = VamanaOverflowTuple::decode(tuple_bytes)?;
                        if tuple.contains(*heap_tid) {
                            continue;
                        }
                        if tuple.nexttid != *expected_nexttid
                            || tuple.heap_tid_count != *expected_heap_tid_count
                        {
                            page_retry = true;
                            break;
                        }
                        tuple.push_heap_tid(*heap_tid)?;
                        let encoded = tuple.encode()?;
                        if encoded.len() != tuple_len {
                            return Err(format!(
                                "ec_diskann duplicate bind overflow tuple size changed from {} to {} at ({},{})",
                                tuple_len,
                                encoded.len(),
                                patch.target_tid.block_number,
                                patch.target_tid.offset_number
                            ));
                        }
                        unsafe {
                            ptr::copy_nonoverlapping(encoded.as_ptr(), tuple_ptr, encoded.len())
                        };
                        page_changed = true;
                        changed_any = true;
                    }
                    DuplicateBindPatchKind::SetOverflowNextTid { expected_nexttid } => {
                        let mut tuple = VamanaOverflowTuple::decode(tuple_bytes)?;
                        if tuple.nexttid == appended_overflow_tid {
                            continue;
                        }
                        if tuple.nexttid != *expected_nexttid {
                            page_retry = true;
                            break;
                        }
                        tuple.nexttid = appended_overflow_tid;
                        let encoded = tuple.encode()?;
                        if encoded.len() != tuple_len {
                            return Err(format!(
                                "ec_diskann duplicate bind overflow tuple size changed from {} to {} at ({},{})",
                                tuple_len,
                                encoded.len(),
                                patch.target_tid.block_number,
                                patch.target_tid.offset_number
                            ));
                        }
                        unsafe {
                            ptr::copy_nonoverlapping(encoded.as_ptr(), tuple_ptr, encoded.len())
                        };
                        page_changed = true;
                        changed_any = true;
                    }
                }
            }
            Ok(())
        })();

        match page_result {
            Ok(()) => {
                if page_changed {
                    unsafe { wal_txn.finish() };
                } else {
                    std::mem::drop(wal_txn);
                }
            }
            Err(error) => {
                std::mem::drop(wal_txn);
                unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
                return Err(error);
            }
        }
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };

        if page_retry {
            return Ok(DuplicateBindApplyOutcome::RetryReplan);
        }
    }

    if changed_any {
        Ok(DuplicateBindApplyOutcome::Changed)
    } else {
        Ok(DuplicateBindApplyOutcome::NoChange)
    }
}

pub(super) unsafe fn bind_duplicate_heap_tid(
    index_relation: pg_sys::Relation,
    existing_node_tid: ItemPointer,
    new_heap_tid: ItemPointer,
) -> Result<DuplicateBindResult, String> {
    for _ in 0..MAX_BACKLINK_REPLAN_PASSES {
        let (metadata, chain) =
            unsafe { scan_state::materialize_chain_from_index(index_relation)? };
        let reader = PersistedGraphReader::new(
            &chain,
            metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&metadata),
            scan_state::metadata_search_code_len(&metadata),
        );
        let existing_node = reader.read_node(existing_node_tid)?;
        if !existing_node.is_live() || existing_node.primary_heaptid == ItemPointer::INVALID {
            return Err(format!(
                "ec_diskann duplicate bind target ({},{}) is not a live node",
                existing_node_tid.block_number, existing_node_tid.offset_number
            ));
        }

        let overflow_refs = overflow_tuple_refs_for_owner(&chain, existing_node_tid)?;
        let plan = plan_duplicate_bind(
            existing_node_tid,
            &existing_node,
            &overflow_refs,
            new_heap_tid,
        )?;
        let DuplicateBindPlan::Apply {
            append_tuple,
            patches,
        } = plan
        else {
            return Ok(DuplicateBindResult::AlreadyBound);
        };

        let appended_overflow_tid = if let Some(overflow_tuple) = append_tuple {
            let encoded = overflow_tuple.encode()?;
            let existing_blocks = unsafe {
                pg_sys::RelationGetNumberOfBlocksInFork(
                    index_relation,
                    pg_sys::ForkNumber::MAIN_FORKNUM,
                )
            };
            let target_block = if existing_blocks > FIRST_DATA_BLOCK_NUMBER {
                existing_blocks - 1
            } else {
                P_NEW
            };
            unsafe {
                append_raw_tuple_payload(
                    index_relation,
                    &encoded,
                    raw_tuple_storage_bytes(encoded.len()),
                    target_block,
                )?
            }
        } else {
            ItemPointer::INVALID
        };

        match unsafe {
            apply_duplicate_bind_patches(
                index_relation,
                &metadata,
                &patches,
                appended_overflow_tid,
            )?
        } {
            DuplicateBindApplyOutcome::RetryReplan => continue,
            DuplicateBindApplyOutcome::Changed => return Ok(DuplicateBindResult::Bound),
            DuplicateBindApplyOutcome::NoChange => return Ok(DuplicateBindResult::AlreadyBound),
        }
    }

    Err(format!(
        "ec_diskann duplicate bind exceeded {} replan passes for node ({},{})",
        MAX_BACKLINK_REPLAN_PASSES, existing_node_tid.block_number, existing_node_tid.offset_number
    ))
}

pub(super) fn duplicate_candidate_tids_by_payload(
    reader: &PersistedGraphReader<'_>,
    payload: &DerivedInsertPayload,
) -> Result<Vec<ItemPointer>, String> {
    let mut matches = Vec::new();
    for tid in reader.iter_node_tids() {
        let tid = tid?;
        let tuple = reader.read_node(tid)?;
        if tuple.deleted || tuple.primary_heaptid == ItemPointer::INVALID {
            continue;
        }
        if tuple.binary_words == payload.binary_words && tuple.search_code == payload.search_code {
            matches.push(tid);
        }
    }
    Ok(matches)
}

pub(super) fn select_insert_forward_neighbors(
    source_vector: &[f32],
    candidates: &[ForwardNeighborCandidate],
    alpha: f32,
    max_degree: usize,
) -> Result<Vec<ItemPointer>, String> {
    if source_vector.is_empty() {
        return Err("ec_diskann insert planning requires a non-empty source vector".into());
    }
    if !(alpha.is_finite() && alpha >= 1.0) {
        return Err(format!(
            "ec_diskann insert planning alpha must be finite and >= 1.0, got {alpha}"
        ));
    }
    if max_degree == 0 {
        return Err("ec_diskann insert planning max_degree must be > 0".into());
    }
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let source_distances = candidates
        .iter()
        .map(|candidate| source_inner_product_distance(source_vector, &candidate.source_vector))
        .collect::<Result<Vec<_>, _>>()?;
    let mut pairwise_distances = vec![vec![0.0_f32; candidates.len()]; candidates.len()];
    for left in 0..candidates.len() {
        for right in (left + 1)..candidates.len() {
            let distance = source_inner_product_distance(
                &candidates[left].source_vector,
                &candidates[right].source_vector,
            )?;
            pairwise_distances[left][right] = distance;
            pairwise_distances[right][left] = distance;
        }
    }

    let initial = source_distances
        .into_iter()
        .enumerate()
        .map(|(idx, distance)| Candidate {
            node: idx as u32,
            distance,
        })
        .collect::<Vec<_>>();
    let kept = robust_prune(u32::MAX, initial, alpha, max_degree, |left, right| {
        pairwise_distances[left as usize][right as usize]
    });
    Ok(kept
        .into_iter()
        .map(|idx| candidates[idx as usize].tid)
        .collect())
}

pub(super) fn insert_backlink_if_free(
    tuple: &mut VamanaNodeTuple,
    backlink_tid: ItemPointer,
) -> bool {
    if backlink_tid == ItemPointer::INVALID {
        return false;
    }
    if tuple.neighbors.contains(&backlink_tid) {
        return false;
    }

    let Some((slot_idx, slot)) = tuple
        .neighbors
        .iter_mut()
        .enumerate()
        .find(|(_, tid)| **tid == ItemPointer::INVALID)
    else {
        return false;
    };
    *slot = backlink_tid;

    let neighbor_count = usize::from(tuple.neighbor_count);
    if slot_idx >= neighbor_count {
        tuple.neighbor_count = u16::try_from(slot_idx + 1).expect("neighbor count fits in u16");
    }
    true
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BacklinkMutation {
    pub(super) target_tid: ItemPointer,
    pub(super) kind: BacklinkMutationKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum BacklinkMutationKind {
    InsertIfFree,
    RewriteFullSlice {
        expected_neighbors: Vec<ItemPointer>,
        expected_neighbor_count: u16,
        replacement_neighbors: Vec<ItemPointer>,
        replacement_neighbor_count: u16,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BacklinkMutationOutcome {
    NoChange,
    Changed,
    RetryReplan,
}

pub(super) fn plan_backlink_mutation(
    target_tid: ItemPointer,
    target_tuple: &VamanaNodeTuple,
    target_source_vector: &[f32],
    existing_candidates: &[ForwardNeighborCandidate],
    new_tid: ItemPointer,
    new_source_vector: &[f32],
    alpha: f32,
    max_degree: usize,
) -> Result<Option<BacklinkMutation>, String> {
    if target_tid == ItemPointer::INVALID {
        return Err("ec_diskann backlink plan requires a valid target tid".into());
    }
    if new_tid == ItemPointer::INVALID {
        return Err("ec_diskann backlink plan requires a valid new tid".into());
    }
    if target_tuple.neighbors.contains(&new_tid) {
        return Ok(None);
    }
    if target_tuple.neighbors.contains(&ItemPointer::INVALID) {
        return Ok(Some(BacklinkMutation {
            target_tid,
            kind: BacklinkMutationKind::InsertIfFree,
        }));
    }

    let mut candidates = existing_candidates.to_vec();
    candidates.push(ForwardNeighborCandidate {
        tid: new_tid,
        source_vector: new_source_vector.to_vec(),
    });
    let selected =
        select_insert_forward_neighbors(target_source_vector, &candidates, alpha, max_degree)?;
    if !selected.contains(&new_tid) {
        return Ok(None);
    }

    let replacement_neighbor_count = u16::try_from(selected.len())
        .map_err(|_| "ec_diskann backlink rewrite neighbor_count exceeds u16".to_owned())?;
    let mut replacement_neighbors = vec![ItemPointer::INVALID; target_tuple.neighbors.len()];
    for (slot, tid) in replacement_neighbors.iter_mut().zip(selected.iter()) {
        *slot = *tid;
    }

    Ok(Some(BacklinkMutation {
        target_tid,
        kind: BacklinkMutationKind::RewriteFullSlice {
            expected_neighbors: target_tuple.neighbors.clone(),
            expected_neighbor_count: target_tuple.neighbor_count,
            replacement_neighbors,
            replacement_neighbor_count,
        },
    }))
}

pub(super) fn apply_backlink_mutation(
    tuple: &mut VamanaNodeTuple,
    new_tid: ItemPointer,
    mutation: &BacklinkMutation,
) -> BacklinkMutationOutcome {
    match &mutation.kind {
        BacklinkMutationKind::InsertIfFree => {
            if insert_backlink_if_free(tuple, new_tid) {
                BacklinkMutationOutcome::Changed
            } else {
                BacklinkMutationOutcome::NoChange
            }
        }
        BacklinkMutationKind::RewriteFullSlice {
            expected_neighbors,
            expected_neighbor_count,
            replacement_neighbors,
            replacement_neighbor_count,
        } => {
            if tuple.neighbors.contains(&new_tid) {
                return BacklinkMutationOutcome::NoChange;
            }
            if insert_backlink_if_free(tuple, new_tid) {
                return BacklinkMutationOutcome::Changed;
            }
            if tuple.neighbors != *expected_neighbors
                || tuple.neighbor_count != *expected_neighbor_count
            {
                return BacklinkMutationOutcome::RetryReplan;
            }
            if tuple.neighbors == *replacement_neighbors
                && tuple.neighbor_count == *replacement_neighbor_count
            {
                return BacklinkMutationOutcome::NoChange;
            }
            tuple.neighbors.clone_from(replacement_neighbors);
            tuple.neighbor_count = *replacement_neighbor_count;
            BacklinkMutationOutcome::Changed
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct EmptyInsertBootstrapOutput {
    pub(super) metadata: VamanaMetadataPage,
    pub(super) chain: DataPageChain,
}

pub(super) unsafe fn read_metadata_page(
    index_relation: pg_sys::Relation,
) -> Result<VamanaMetadataPage, String> {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            crate::storage::page::METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        return Err("ec_diskann failed to open metadata buffer".into());
    }
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let page = unsafe { pg_sys::BufferGetPage(buffer) };
    let special = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    let metadata_bytes = unsafe { slice::from_raw_parts(special, VAMANA_METADATA_BYTES) };
    let metadata = VamanaMetadataPage::decode(metadata_bytes);
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    metadata
}

pub(super) unsafe fn with_locked_metadata_page<T>(
    index_relation: pg_sys::Relation,
    f: impl FnOnce(&mut VamanaMetadataPage) -> Result<T, String>,
) -> Result<T, String> {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            crate::storage::page::METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        return Err("ec_diskann failed to open metadata buffer".into());
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let page = unsafe { pg_sys::BufferGetPage(buffer) };
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let special = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    let metadata_bytes = unsafe { slice::from_raw_parts(special, VAMANA_METADATA_BYTES) };
    let mut metadata = VamanaMetadataPage::decode(metadata_bytes)?;
    let result = f(&mut metadata)?;

    let encoded = metadata.encode();
    let special_size = (encoded.len() + 7) & !7;
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let writable_page =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    unsafe { pg_sys::PageInit(writable_page, page_size, special_size) };
    let dst = unsafe { pg_sys::PageGetSpecialPointer(writable_page) }.cast::<u8>();
    unsafe { ptr::copy_nonoverlapping(encoded.as_ptr(), dst, encoded.len()) };
    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    Ok(result)
}

pub(super) unsafe fn bootstrap_empty_insert_output(
    index_relation: pg_sys::Relation,
    heap_tid: ItemPointer,
    source_vector: &[f32],
) -> Result<EmptyInsertBootstrapOutput, String> {
    if source_vector.is_empty() {
        return Err("ec_diskann empty-index bootstrap requires a non-empty source vector".into());
    }

    let dimensions = u16::try_from(source_vector.len()).map_err(|_| {
        format!(
            "ec_diskann insert source dimension {} exceeds u16",
            source_vector.len()
        )
    })?;
    let seed = DEFAULT_QUANT_SEED;
    let group_size = ambuild::default_group_size(dimensions);
    let source_refs = vec![source_vector];
    let model = training::train_grouped_pq4_model(
        &source_refs,
        source_vector.len(),
        seed,
        group_size,
        1,
        EMPTY_INSERT_BOOTSTRAP_KMEANS_ITERS,
    )?;

    let sidecar_word_count =
        training::persisted_binary_sidecar_word_count(dimensions, DEFAULT_QUANT_BITS, seed);
    let has_binary_sidecar = sidecar_word_count > 0;
    let binary_words = if has_binary_sidecar {
        let quantizer = ProdQuantizer::cached(source_vector.len(), DEFAULT_QUANT_BITS, seed);
        let encoded = quantizer.encode(source_vector);
        let mut code = encoded.mse_packed;
        code.extend_from_slice(&encoded.qjl_packed);
        training::derive_persisted_binary_words(&quantizer, &code)
    } else {
        Vec::new()
    };

    let payloads = vec![NodePayload {
        primary_heaptid: heap_tid,
        binary_words,
        search_code: training::derive_grouped_pq4_code(source_vector, &model),
    }];

    let relopts = unsafe { options::relation_options(index_relation) };
    let params = BuildParams {
        graph_degree_r: u16::try_from(relopts.graph_degree)
            .map_err(|_| "graph_degree does not fit in u16".to_owned())?,
        build_list_size_l: u16::try_from(relopts.build_list_size)
            .map_err(|_| "build_list_size does not fit in u16".to_owned())?,
        alpha: relopts.alpha,
        dimensions,
        search_subvector_count: u16::try_from(model.group_count)
            .map_err(|_| "search_subvector_count does not fit in u16".to_owned())?,
        search_subvector_dim: u16::try_from(model.group_size)
            .map_err(|_| "search_subvector_dim does not fit in u16".to_owned())?,
        seed,
        page_size: pg_sys::BLCKSZ as usize,
        has_binary_sidecar,
    };

    let BuildOutput {
        mut metadata,
        persisted,
        ..
    } = build_and_persist_vamana(params, &payloads, |_, _| 0.0)?;
    let mut chain = persisted.chain;
    let codebook_head = stage_grouped_codebook_chain(&mut chain, &model)?;
    metadata.grouped_codebook_head = codebook_head;
    metadata.inserted_since_rebuild = 1;

    Ok(EmptyInsertBootstrapOutput { metadata, chain })
}

pub(super) unsafe fn append_live_node(
    index_relation: pg_sys::Relation,
    metadata: &VamanaMetadataPage,
    heap_tid: ItemPointer,
    payload: &DerivedInsertPayload,
    forward_neighbors: &[ItemPointer],
) -> Result<ItemPointer, String> {
    if heap_tid == ItemPointer::INVALID {
        return Err("ec_diskann append requires a valid heap tid".into());
    }
    if forward_neighbors.len() > metadata.graph_degree_r as usize {
        return Err(format!(
            "ec_diskann append forward-neighbor count {} exceeds graph degree {}",
            forward_neighbors.len(),
            metadata.graph_degree_r
        ));
    }

    let mut tuple = VamanaNodeTuple::placeholder(
        metadata.graph_degree_r,
        payload.binary_words.len(),
        payload.search_code.len(),
    );
    tuple.primary_heaptid = heap_tid;
    tuple.binary_words = payload.binary_words.clone();
    tuple.search_code = payload.search_code.clone();
    tuple.neighbor_count = u16::try_from(forward_neighbors.len())
        .map_err(|_| "forward neighbor count does not fit in u16".to_owned())?;
    for (slot, neighbor) in forward_neighbors.iter().copied().enumerate() {
        tuple.neighbors[slot] = neighbor;
    }

    let encoded = tuple.encode(
        metadata.graph_degree_r,
        payload.binary_words.len(),
        payload.search_code.len(),
    )?;
    if !element_or_neighbor_tuple_fits(encoded.len(), pg_sys::BLCKSZ as usize) {
        return Err(format!(
            "ec_diskann append node payload {} exceeds page capacity {}",
            encoded.len(),
            pg_sys::BLCKSZ as usize
        ));
    }

    let existing_blocks = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let target_block = if existing_blocks > FIRST_DATA_BLOCK_NUMBER {
        existing_blocks - 1
    } else {
        P_NEW
    };
    unsafe {
        append_raw_tuple_payload(
            index_relation,
            &encoded,
            raw_tuple_storage_bytes(encoded.len()),
            target_block,
        )
    }
}

unsafe fn append_raw_tuple_payload(
    index_relation: pg_sys::Relation,
    encoded: &[u8],
    required_bytes: usize,
    target_block: pg_sys::BlockNumber,
) -> Result<ItemPointer, String> {
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
        return Err("ec_diskann failed to allocate append buffer".into());
    }

    if target_block != P_NEW {
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    if target_block == P_NEW {
        unsafe { pg_sys::PageInit(page_ptr, page_size, 0) };
    } else {
        let free_space = unsafe { pg_sys::PageGetFreeSpace(page_ptr) as usize };
        if free_space < required_bytes {
            std::mem::drop(wal_txn);
            unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
            return unsafe {
                append_raw_tuple_payload(index_relation, encoded, required_bytes, P_NEW)
            };
        }
    }

    let block_number = unsafe { pg_sys::BufferGetBlockNumber(buffer) };
    let offset_number = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            encoded.as_ptr().cast_mut().cast(),
            encoded.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if offset_number == pg_sys::InvalidOffsetNumber {
        return Err("ec_diskann failed to append live node tuple".into());
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    Ok(ItemPointer {
        block_number,
        offset_number,
    })
}

pub(super) unsafe fn add_backlinks_if_free(
    index_relation: pg_sys::Relation,
    metadata: &VamanaMetadataPage,
    backlink_targets: &[ItemPointer],
    new_tid: ItemPointer,
) -> Result<usize, String> {
    if new_tid == ItemPointer::INVALID {
        return Err("ec_diskann backlink write requires a valid new node tid".into());
    }
    if backlink_targets.is_empty() {
        return Ok(0);
    }

    let mut targets = backlink_targets.to_vec();
    sort_and_dedup_backlink_targets(&mut targets);
    let binary_word_count = scan_state::metadata_binary_word_count(metadata);
    let search_code_len = scan_state::metadata_search_code_len(metadata);
    let mut changed = 0usize;
    let mut start = 0usize;

    while start < targets.len() {
        let block_number = targets[start].block_number;
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        if !unsafe { pg_sys::BufferIsValid(buffer) } {
            return Err(format!(
                "ec_diskann backlink write could not open target block {block_number}"
            ));
        }

        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
        let writable_page =
            unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
        let mut page_changed = false;
        let page_result = (|| -> Result<usize, String> {
            let mut page_changes = 0usize;
            while start < targets.len() && targets[start].block_number == block_number {
                let target_tid = targets[start];
                start += 1;

                let (tuple_ptr, tuple_len) =
                    unsafe { page_tuple_location(writable_page, page_size, target_tid)? };
                let tuple_bytes =
                    unsafe { slice::from_raw_parts(tuple_ptr.cast_const(), tuple_len) };
                let mut tuple = VamanaNodeTuple::decode(
                    tuple_bytes,
                    metadata.graph_degree_r,
                    binary_word_count,
                    search_code_len,
                )?;
                if !tuple.is_live() {
                    continue;
                }
                if !insert_backlink_if_free(&mut tuple, new_tid) {
                    continue;
                }

                let encoded =
                    tuple.encode(metadata.graph_degree_r, binary_word_count, search_code_len)?;
                if encoded.len() != tuple_len {
                    return Err(format!(
                        "ec_diskann backlink target tuple size changed from {} to {} at ({},{})",
                        tuple_len,
                        encoded.len(),
                        target_tid.block_number,
                        target_tid.offset_number
                    ));
                }
                unsafe { ptr::copy_nonoverlapping(encoded.as_ptr(), tuple_ptr, encoded.len()) };
                page_changed = true;
                page_changes += 1;
            }
            Ok(page_changes)
        })();

        match page_result {
            Ok(page_changes) => {
                if page_changed {
                    unsafe { wal_txn.finish() };
                    changed += page_changes;
                } else {
                    std::mem::drop(wal_txn);
                }
            }
            Err(error) => {
                std::mem::drop(wal_txn);
                unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
                return Err(error);
            }
        }
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }

    Ok(changed)
}

pub(super) unsafe fn apply_backlink_mutations(
    index_relation: pg_sys::Relation,
    metadata: &VamanaMetadataPage,
    mutations: &[BacklinkMutation],
    new_tid: ItemPointer,
) -> Result<Vec<ItemPointer>, String> {
    if new_tid == ItemPointer::INVALID {
        return Err("ec_diskann backlink rewrite requires a valid new node tid".into());
    }
    if mutations.is_empty() {
        return Ok(Vec::new());
    }

    let mut sorted = mutations.to_vec();
    sort_and_dedup_backlink_mutations(&mut sorted);
    let binary_word_count = scan_state::metadata_binary_word_count(metadata);
    let search_code_len = scan_state::metadata_search_code_len(metadata);
    let mut retries = Vec::new();
    let mut start = 0usize;

    while start < sorted.len() {
        let block_number = sorted[start].target_tid.block_number;
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        if !unsafe { pg_sys::BufferIsValid(buffer) } {
            return Err(format!(
                "ec_diskann backlink rewrite could not open target block {block_number}"
            ));
        }

        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
        let writable_page =
            unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
        let mut page_changed = false;
        let page_result = (|| -> Result<(), String> {
            while start < sorted.len() && sorted[start].target_tid.block_number == block_number {
                let mutation = &sorted[start];
                start += 1;

                let (tuple_ptr, tuple_len) =
                    unsafe { page_tuple_location(writable_page, page_size, mutation.target_tid)? };
                let tuple_bytes =
                    unsafe { slice::from_raw_parts(tuple_ptr.cast_const(), tuple_len) };
                let mut tuple = VamanaNodeTuple::decode(
                    tuple_bytes,
                    metadata.graph_degree_r,
                    binary_word_count,
                    search_code_len,
                )?;
                if !tuple.is_live() {
                    continue;
                }

                match apply_backlink_mutation(&mut tuple, new_tid, mutation) {
                    BacklinkMutationOutcome::NoChange => {}
                    BacklinkMutationOutcome::RetryReplan => retries.push(mutation.target_tid),
                    BacklinkMutationOutcome::Changed => {
                        let encoded = tuple.encode(
                            metadata.graph_degree_r,
                            binary_word_count,
                            search_code_len,
                        )?;
                        if encoded.len() != tuple_len {
                            return Err(format!(
                                "ec_diskann backlink rewrite target tuple size changed from {} to {} at ({},{})",
                                tuple_len,
                                encoded.len(),
                                mutation.target_tid.block_number,
                                mutation.target_tid.offset_number
                            ));
                        }
                        unsafe {
                            ptr::copy_nonoverlapping(encoded.as_ptr(), tuple_ptr, encoded.len())
                        };
                        page_changed = true;
                    }
                }
            }
            Ok(())
        })();

        match page_result {
            Ok(()) => {
                if page_changed {
                    unsafe { wal_txn.finish() };
                } else {
                    std::mem::drop(wal_txn);
                }
            }
            Err(error) => {
                std::mem::drop(wal_txn);
                unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
                return Err(error);
            }
        }
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }

    sort_and_dedup_backlink_targets(&mut retries);
    Ok(retries)
}

pub(super) unsafe fn increment_inserted_since_rebuild(
    index_relation: pg_sys::Relation,
) -> Result<u64, String> {
    unsafe {
        with_locked_metadata_page(index_relation, |metadata| {
            metadata.inserted_since_rebuild = metadata
                .inserted_since_rebuild
                .checked_add(1)
                .ok_or_else(|| "ec_diskann inserted_since_rebuild overflowed u64".to_owned())?;
            Ok(metadata.inserted_since_rebuild)
        })
    }
}

fn sort_and_dedup_backlink_targets(targets: &mut Vec<ItemPointer>) {
    targets.sort_unstable_by(cmp_item_pointer_physical);
    targets.dedup();
}

fn sort_and_dedup_backlink_mutations(mutations: &mut Vec<BacklinkMutation>) {
    mutations.sort_unstable_by(|left, right| {
        cmp_item_pointer_physical(&left.target_tid, &right.target_tid)
    });
    mutations.dedup_by(|left, right| left.target_tid == right.target_tid);
}

unsafe fn page_tuple_location(
    page: pg_sys::Page,
    page_size: usize,
    tid: ItemPointer,
) -> Result<(*mut u8, usize), String> {
    let max_offset = unsafe { pg_sys::PageGetMaxOffsetNumber(page) };
    if tid.offset_number == pg_sys::InvalidOffsetNumber || tid.offset_number > max_offset {
        return Err(format!(
            "ec_diskann backlink target ({},{}) has invalid offset {} (max {})",
            tid.block_number, tid.offset_number, tid.offset_number, max_offset
        ));
    }

    let item_id = unsafe { pg_sys::PageGetItemId(page, tid.offset_number) };
    if item_id.is_null() {
        return Err(format!(
            "ec_diskann backlink target ({},{}) returned a null item id",
            tid.block_number, tid.offset_number
        ));
    }
    let item_id_ref = unsafe { &*item_id };
    if item_id_ref.lp_flags() == 0 {
        return Err(format!(
            "ec_diskann backlink target ({},{}) points at an unused slot",
            tid.block_number, tid.offset_number
        ));
    }

    let tuple_offset = item_id_ref.lp_off() as usize;
    let tuple_len = item_id_ref.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        return Err(format!(
            "ec_diskann backlink target ({},{}) has invalid tuple bounds",
            tid.block_number, tid.offset_number
        ));
    }

    let tuple_ptr = unsafe { pg_sys::PageGetItem(page, item_id) }.cast::<u8>();
    if tuple_ptr.is_null() {
        return Err(format!(
            "ec_diskann backlink target ({},{}) returned a null tuple pointer",
            tid.block_number, tid.offset_number
        ));
    }
    Ok((tuple_ptr, tuple_len))
}

fn source_inner_product_distance(left: &[f32], right: &[f32]) -> Result<f32, String> {
    if left.len() != right.len() {
        return Err(format!(
            "ec_diskann exact distance dimension mismatch: left dim {}, right dim {}",
            left.len(),
            right.len()
        ));
    }
    let ip = left
        .iter()
        .zip(right.iter())
        .map(|(lhs, rhs)| lhs * rhs)
        .sum::<f32>();
    Ok((ECDISKANN_UNIT_NORM_DISTANCE_BIAS - ip).max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::common::training::{self, train_grouped_pq4_model};
    use crate::am::ec_diskann::page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE;
    use crate::am::ec_diskann::persist::stage_grouped_codebook_chain;
    use crate::am::ec_diskann::tuple::VamanaNodeTuple;
    use crate::storage::page::DEFAULT_PAGE_SIZE;

    fn training_vectors() -> Vec<Vec<f32>> {
        vec![
            vec![1.0, 0.0, 0.5, -1.0, 0.25, -0.5, 0.75, -0.25],
            vec![0.9, 0.1, 0.45, -0.95, 0.2, -0.45, 0.7, -0.2],
            vec![0.0, 1.0, 0.25, -0.5, -0.1, 0.3, 0.2, -0.7],
            vec![-1.0, 0.5, 0.0, 1.0, -0.2, 0.4, -0.6, 0.8],
            vec![0.3, -0.7, 0.8, -0.1, 0.9, -0.4, 0.6, -0.2],
            vec![-0.4, 0.6, -0.9, 0.2, -0.8, 0.5, -0.3, 0.1],
        ]
    }

    #[test]
    fn source_inner_product_distance_keeps_positive_ip_pairs_distinct() {
        let identical =
            super::source_inner_product_distance(&[1.0, 0.0], &[1.0, 0.0]).expect("same dim");
        let merely_similar =
            super::source_inner_product_distance(&[1.0, 0.0], &[0.8, 0.6]).expect("same dim");
        let orthogonal =
            super::source_inner_product_distance(&[1.0, 0.0], &[0.0, 1.0]).expect("same dim");

        assert_eq!(identical, 0.0);
        assert!(merely_similar > identical);
        assert!(orthogonal > merely_similar);
    }

    fn staged_metadata(
        with_binary_sidecar: bool,
    ) -> (
        VamanaMetadataPage,
        DataPageChain,
        Vec<Vec<f32>>,
        training::GroupedPq4Model,
    ) {
        let vectors = training_vectors();
        let refs: Vec<&[f32]> = vectors.iter().map(Vec::as_slice).collect();
        let dimensions = vectors[0].len();
        let seed = 42_u64;
        let group_size = 4_usize;
        let model = train_grouped_pq4_model(&refs, dimensions, seed, group_size, refs.len(), 6)
            .expect("train");

        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let codebook_head =
            stage_grouped_codebook_chain(&mut chain, &model).expect("stage codebooks");

        let mut metadata = VamanaMetadataPage::empty(32, 100, 1.2, dimensions as u16, seed);
        metadata.search_subvector_count = model.group_count as u16;
        metadata.search_subvector_dim = model.group_size as u16;
        metadata.grouped_codebook_head = codebook_head;
        metadata.payload_flags = PAYLOAD_FLAG_GROUPED_SEARCH_CODE;
        if with_binary_sidecar {
            metadata.payload_flags |= PAYLOAD_FLAG_BINARY_SIDECAR;
        }

        (metadata, chain, vectors, model)
    }

    fn single_node_chain() -> (VamanaMetadataPage, DataPageChain, ItemPointer, ItemPointer) {
        let metadata = VamanaMetadataPage::empty(4, 16, 1.2, 8, 42);
        let owner_primary = ItemPointer {
            block_number: 1000,
            offset_number: 1,
        };
        let owner_tuple = VamanaNodeTuple {
            deleted: false,
            has_overflow_heaptids: false,
            primary_heaptid: owner_primary,
            rerank_tid: ItemPointer::INVALID,
            binary_words: Vec::new(),
            search_code: Vec::new(),
            neighbors: vec![ItemPointer::INVALID; metadata.graph_degree_r as usize],
            neighbor_count: 0,
        };
        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let owner_tid = chain
            .insert_raw_tuple(
                owner_tuple
                    .encode(metadata.graph_degree_r, 0, 0)
                    .expect("owner tuple should encode"),
            )
            .expect("owner tuple should stage");
        (metadata, chain, owner_tid, owner_primary)
    }

    // IN-001: derive_insert_payload_from_persisted matches the build-side
    // grouped-PQ search code and persisted binary sidecar derivation.
    #[test]
    fn in_001_payload_matches_training_model_with_binary_sidecar() {
        let (metadata, chain, vectors, model) = staged_metadata(true);
        let source = &vectors[0];

        let observed =
            derive_insert_payload_from_persisted(&metadata, &chain, source).expect("derive");

        let expected_search_code = training::derive_grouped_pq4_code(source, &model);
        let quantizer = ProdQuantizer::cached(source.len(), DEFAULT_QUANT_BITS, metadata.seed);
        let encoded = quantizer.encode(source);
        let mut code = encoded.mse_packed;
        code.extend_from_slice(&encoded.qjl_packed);
        let expected_binary_words = training::derive_persisted_binary_words(&quantizer, &code);

        assert_eq!(observed.search_code, expected_search_code);
        assert_eq!(observed.binary_words, expected_binary_words);
    }

    // IN-002: the helper honors metadata with no binary sidecar bit.
    #[test]
    fn in_002_payload_omits_binary_words_without_sidecar_flag() {
        let (metadata, chain, vectors, model) = staged_metadata(false);
        let source = &vectors[1];

        let observed =
            derive_insert_payload_from_persisted(&metadata, &chain, source).expect("derive");

        assert_eq!(
            observed.search_code,
            training::derive_grouped_pq4_code(source, &model)
        );
        assert!(
            observed.binary_words.is_empty(),
            "payload should omit binary sidecar words when the flag is clear"
        );
    }

    // IN-003: grouped codebooks are mandatory.
    #[test]
    fn in_003_missing_codebook_head_errors() {
        let (mut metadata, chain, vectors, _) = staged_metadata(true);
        metadata.grouped_codebook_head = ItemPointer::INVALID;
        let err = derive_insert_payload_from_persisted(&metadata, &chain, &vectors[0])
            .expect_err("missing codebooks should fail");
        assert!(err.contains("persisted grouped codebooks"), "got: {err}");
    }

    // IN-004: source dimension must match metadata.
    #[test]
    fn in_004_dimension_mismatch_errors() {
        let (metadata, chain, _, _) = staged_metadata(true);
        let err = derive_insert_payload_from_persisted(&metadata, &chain, &[1.0, 2.0, 3.0])
            .expect_err("dim mismatch should fail");
        assert!(err.contains("dimension mismatch"), "got: {err}");
    }

    // IN-005: unsupported metadata transform / codec is rejected up front.
    #[test]
    fn in_005_transform_and_codec_are_validated() {
        let (mut metadata, chain, vectors, _) = staged_metadata(true);
        metadata.transform_kind = 99;
        let err = derive_insert_payload_from_persisted(&metadata, &chain, &vectors[0])
            .expect_err("bad transform should fail");
        assert!(err.contains("transform kind"), "got: {err}");

        let (mut metadata, chain, vectors, _) = staged_metadata(true);
        metadata.search_codec_kind = 99;
        let err = derive_insert_payload_from_persisted(&metadata, &chain, &vectors[0])
            .expect_err("bad codec should fail");
        assert!(err.contains("codec kind"), "got: {err}");
    }

    #[test]
    fn in_005b_stage_overflow_heap_tids_in_chain_roundtrips_multiple_chunks() {
        let (metadata, mut chain, owner_tid, owner_primary) = single_node_chain();
        let overflow_heap_tids = (0..12_u32)
            .map(|offset| ItemPointer {
                block_number: 2000 + offset,
                offset_number: 1,
            })
            .collect::<Vec<_>>();

        stage_overflow_heap_tids_in_chain(
            &mut chain,
            metadata.graph_degree_r,
            0,
            0,
            owner_tid,
            &overflow_heap_tids,
        )
        .expect("overflow staging should succeed");

        let reader = PersistedGraphReader::new(&chain, metadata.graph_degree_r, 0, 0);
        let owner_tuple = reader
            .read_node(owner_tid)
            .expect("owner tuple should decode");
        assert!(owner_tuple.has_overflow_heaptids);

        let bound_heap_tids =
            bound_heap_tids_for_owner(&chain, owner_tid, owner_tuple.primary_heaptid)
                .expect("bound heap tids should decode");
        assert_eq!(bound_heap_tids[0], owner_primary);
        assert_eq!(bound_heap_tids[1..], overflow_heap_tids);
    }

    #[test]
    fn in_006_duplicate_lookup_finds_first_live_match() {
        let (mut metadata, chain, vectors, model) = staged_metadata(true);
        let source = &vectors[0];
        let payload =
            derive_insert_payload_from_persisted(&metadata, &chain, source).expect("derive");
        let mut node_chain = DataPageChain::new(DEFAULT_PAGE_SIZE);

        let mut first = VamanaNodeTuple::placeholder(
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );
        first.binary_words = payload.binary_words.clone();
        first.search_code = payload.search_code.clone();
        first.primary_heaptid = ItemPointer {
            block_number: 500,
            offset_number: 1,
        };
        node_chain
            .insert_raw_tuple(
                first
                    .encode(
                        metadata.graph_degree_r,
                        payload.binary_words.len(),
                        payload.search_code.len(),
                    )
                    .expect("first tuple should encode"),
            )
            .expect("first tuple");

        let mut other = VamanaNodeTuple::placeholder(
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );
        other.binary_words = payload.binary_words.clone();
        other.search_code = payload.search_code.clone();
        other.primary_heaptid = ItemPointer {
            block_number: 501,
            offset_number: 1,
        };
        node_chain
            .insert_raw_tuple(
                other
                    .encode(
                        metadata.graph_degree_r,
                        payload.binary_words.len(),
                        payload.search_code.len(),
                    )
                    .expect("second tuple should encode"),
            )
            .expect("second tuple");
        metadata.grouped_codebook_head =
            stage_grouped_codebook_chain(&mut node_chain, &model).expect("stage codebooks");

        let reader = PersistedGraphReader::new(
            &node_chain,
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );

        let matches = duplicate_candidate_tids_by_payload(&reader, &payload).expect("lookup");

        assert_eq!(
            matches.len(),
            2,
            "both live payload matches should be returned"
        );
        assert_eq!(matches[0].block_number, 1);
        assert_eq!(matches[0].offset_number, 1);
    }

    #[test]
    fn in_007_duplicate_lookup_skips_deleted_and_stripped_tuples() {
        let (mut metadata, chain, vectors, model) = staged_metadata(true);
        let source = &vectors[0];
        let payload =
            derive_insert_payload_from_persisted(&metadata, &chain, source).expect("derive");
        let mut node_chain = DataPageChain::new(DEFAULT_PAGE_SIZE);

        let mut deleted = VamanaNodeTuple::placeholder(
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );
        deleted.binary_words = payload.binary_words.clone();
        deleted.search_code = payload.search_code.clone();
        deleted.primary_heaptid = ItemPointer {
            block_number: 500,
            offset_number: 1,
        };
        deleted.deleted = true;
        node_chain
            .insert_raw_tuple(
                deleted
                    .encode(
                        metadata.graph_degree_r,
                        payload.binary_words.len(),
                        payload.search_code.len(),
                    )
                    .expect("deleted tuple should encode"),
            )
            .expect("deleted tuple");

        let mut stripped = VamanaNodeTuple::placeholder(
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );
        stripped.binary_words = payload.binary_words.clone();
        stripped.search_code = payload.search_code.clone();
        stripped.primary_heaptid = ItemPointer::INVALID;
        stripped.has_overflow_heaptids = true;
        node_chain
            .insert_raw_tuple(
                stripped
                    .encode(
                        metadata.graph_degree_r,
                        payload.binary_words.len(),
                        payload.search_code.len(),
                    )
                    .expect("stripped tuple should encode"),
            )
            .expect("stripped tuple");
        metadata.grouped_codebook_head =
            stage_grouped_codebook_chain(&mut node_chain, &model).expect("stage codebooks");

        let reader = PersistedGraphReader::new(
            &node_chain,
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );

        let tid = duplicate_candidate_tids_by_payload(&reader, &payload).expect("lookup");
        assert!(
            tid.is_empty(),
            "deleted or stripped tuples must not be eligible duplicate targets"
        );
    }

    #[test]
    fn in_008_forward_neighbor_selection_prunes_on_exact_vectors() {
        let source = vec![1.0_f32, 0.0];
        let candidates = vec![
            ForwardNeighborCandidate {
                tid: ItemPointer {
                    block_number: 1,
                    offset_number: 1,
                },
                source_vector: vec![0.0, 1.0],
            },
            ForwardNeighborCandidate {
                tid: ItemPointer {
                    block_number: 1,
                    offset_number: 2,
                },
                source_vector: vec![0.0, -1.0],
            },
            ForwardNeighborCandidate {
                tid: ItemPointer {
                    block_number: 1,
                    offset_number: 3,
                },
                source_vector: vec![-1.0, 0.0],
            },
        ];

        let selected =
            select_insert_forward_neighbors(&source, &candidates, 1.2, 2).expect("select");

        assert_eq!(
            selected,
            vec![candidates[0].tid, candidates[1].tid],
            "the exact-vector alpha prune should retain the two orthogonal neighbors"
        );
    }

    #[test]
    fn in_009_forward_neighbor_selection_rejects_dimension_mismatch() {
        let err = select_insert_forward_neighbors(
            &[1.0, 0.0],
            &[ForwardNeighborCandidate {
                tid: ItemPointer {
                    block_number: 1,
                    offset_number: 1,
                },
                source_vector: vec![1.0, 0.0, -1.0],
            }],
            1.2,
            4,
        )
        .expect_err("dimension mismatch should fail");
        assert!(err.contains("dimension mismatch"), "got: {err}");
    }

    #[test]
    fn in_010_duplicate_lookup_finds_match_after_codebook_tail() {
        let (metadata, chain, vectors, model) = staged_metadata(true);
        let source = &vectors[0];
        let payload =
            derive_insert_payload_from_persisted(&metadata, &chain, source).expect("derive");
        let mut node_chain = DataPageChain::new(DEFAULT_PAGE_SIZE);

        let mut first = VamanaNodeTuple::placeholder(
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );
        first.binary_words = payload.binary_words.clone();
        first.search_code = payload.search_code.clone();
        first.primary_heaptid = ItemPointer {
            block_number: 500,
            offset_number: 1,
        };
        node_chain
            .insert_raw_tuple(
                first
                    .encode(
                        metadata.graph_degree_r,
                        payload.binary_words.len(),
                        payload.search_code.len(),
                    )
                    .expect("first tuple should encode"),
            )
            .expect("first tuple");

        stage_grouped_codebook_chain(&mut node_chain, &model).expect("stage codebooks");

        let mut appended = VamanaNodeTuple::placeholder(
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );
        appended.binary_words = payload.binary_words.clone();
        appended.search_code = payload.search_code.clone();
        appended.primary_heaptid = ItemPointer {
            block_number: 501,
            offset_number: 1,
        };
        node_chain
            .insert_raw_tuple(
                appended
                    .encode(
                        metadata.graph_degree_r,
                        payload.binary_words.len(),
                        payload.search_code.len(),
                    )
                    .expect("appended tuple should encode"),
            )
            .expect("appended tuple");

        let reader = PersistedGraphReader::new(
            &node_chain,
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );

        let matches = duplicate_candidate_tids_by_payload(&reader, &payload).expect("lookup");
        assert_eq!(
            matches,
            vec![
                ItemPointer {
                    block_number: 1,
                    offset_number: 1,
                },
                ItemPointer {
                    block_number: 1,
                    offset_number: 4,
                },
            ]
        );
    }

    #[test]
    fn in_011_insert_backlink_if_free_uses_first_open_slot() {
        let backlink_tid = ItemPointer {
            block_number: 9,
            offset_number: 4,
        };
        let mut tuple = VamanaNodeTuple::placeholder(4, 0, 0);
        tuple.neighbor_count = 1;
        tuple.neighbors[0] = ItemPointer {
            block_number: 3,
            offset_number: 1,
        };

        let changed = insert_backlink_if_free(&mut tuple, backlink_tid);

        assert!(
            changed,
            "a tuple with free neighbor capacity should admit a backlink"
        );
        assert_eq!(tuple.neighbor_count, 2);
        assert_eq!(tuple.neighbors[1], backlink_tid);
    }

    #[test]
    fn in_012_insert_backlink_if_free_rejects_duplicate_and_full_tuples() {
        let backlink_tid = ItemPointer {
            block_number: 9,
            offset_number: 4,
        };
        let mut duplicate = VamanaNodeTuple::placeholder(2, 0, 0);
        duplicate.neighbor_count = 2;
        duplicate.neighbors[0] = backlink_tid;
        duplicate.neighbors[1] = ItemPointer {
            block_number: 5,
            offset_number: 2,
        };
        assert!(
            !insert_backlink_if_free(&mut duplicate, backlink_tid),
            "duplicate backlinks must not rewrite the tuple"
        );

        let mut full = VamanaNodeTuple::placeholder(2, 0, 0);
        full.neighbor_count = 2;
        full.neighbors[0] = ItemPointer {
            block_number: 7,
            offset_number: 1,
        };
        full.neighbors[1] = ItemPointer {
            block_number: 7,
            offset_number: 2,
        };
        assert!(
            !insert_backlink_if_free(&mut full, backlink_tid),
            "full tuples must stay unchanged in the free-capacity slice"
        );
    }

    #[test]
    fn in_013_plan_backlink_mutation_rewrites_full_slice_for_kept_candidate() {
        let target_tid = ItemPointer {
            block_number: 4,
            offset_number: 1,
        };
        let existing_tid = ItemPointer {
            block_number: 5,
            offset_number: 1,
        };
        let new_tid = ItemPointer {
            block_number: 6,
            offset_number: 1,
        };
        let mut target = VamanaNodeTuple::placeholder(1, 0, 0);
        target.neighbor_count = 1;
        target.neighbors[0] = existing_tid;

        let mutation = plan_backlink_mutation(
            target_tid,
            &target,
            &[1.0, 0.0],
            &[ForwardNeighborCandidate {
                tid: existing_tid,
                source_vector: vec![-1.0, 0.0],
            }],
            new_tid,
            &[1.0, 0.0],
            1.2,
            1,
        )
        .expect("plan should succeed")
        .expect("new candidate should survive prune");

        assert_eq!(
            mutation,
            BacklinkMutation {
                target_tid,
                kind: BacklinkMutationKind::RewriteFullSlice {
                    expected_neighbors: vec![existing_tid],
                    expected_neighbor_count: 1,
                    replacement_neighbors: vec![new_tid],
                    replacement_neighbor_count: 1,
                },
            }
        );
    }

    #[test]
    fn in_014_apply_backlink_mutation_requests_retry_for_stale_full_slice() {
        let old_tid = ItemPointer {
            block_number: 5,
            offset_number: 1,
        };
        let drifted_tid = ItemPointer {
            block_number: 7,
            offset_number: 1,
        };
        let new_tid = ItemPointer {
            block_number: 6,
            offset_number: 1,
        };
        let mutation = BacklinkMutation {
            target_tid: ItemPointer {
                block_number: 4,
                offset_number: 1,
            },
            kind: BacklinkMutationKind::RewriteFullSlice {
                expected_neighbors: vec![old_tid],
                expected_neighbor_count: 1,
                replacement_neighbors: vec![new_tid],
                replacement_neighbor_count: 1,
            },
        };
        let mut tuple = VamanaNodeTuple::placeholder(1, 0, 0);
        tuple.neighbor_count = 1;
        tuple.neighbors[0] = drifted_tid;

        assert_eq!(
            apply_backlink_mutation(&mut tuple, new_tid, &mutation),
            BacklinkMutationOutcome::RetryReplan,
        );
        assert_eq!(
            tuple.neighbors,
            vec![drifted_tid],
            "stale plans must leave the reopened tuple unchanged",
        );
    }

    #[test]
    fn in_015_apply_backlink_mutation_rewrites_full_slice_after_replan() {
        let old_tid = ItemPointer {
            block_number: 5,
            offset_number: 1,
        };
        let new_tid = ItemPointer {
            block_number: 6,
            offset_number: 1,
        };
        let mutation = BacklinkMutation {
            target_tid: ItemPointer {
                block_number: 4,
                offset_number: 1,
            },
            kind: BacklinkMutationKind::RewriteFullSlice {
                expected_neighbors: vec![old_tid],
                expected_neighbor_count: 1,
                replacement_neighbors: vec![new_tid],
                replacement_neighbor_count: 1,
            },
        };
        let mut tuple = VamanaNodeTuple::placeholder(1, 0, 0);
        tuple.neighbor_count = 1;
        tuple.neighbors[0] = old_tid;

        assert_eq!(
            apply_backlink_mutation(&mut tuple, new_tid, &mutation),
            BacklinkMutationOutcome::Changed,
        );
        assert_eq!(tuple.neighbors, vec![new_tid]);
        assert_eq!(tuple.neighbor_count, 1);
    }
}
