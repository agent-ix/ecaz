//! Local (single-node) implementation of the frozen expansion seam: the
//! degenerate FR-079 case where every vec_id is owned by this node. Each
//! expansion performs exactly one index-record read (neighbor scoring uses
//! only the embedded codes) plus one co-located heap read for the exact
//! distance — the ec_diskann coarse-search-then-heap-rerank split executed
//! in place (ADR-085 D11).

use pgrx::pg_sys;

use crate::am::ec_diskann::exact_heap_rerank_distance;
use crate::storage::{
    buffer_guard::{LockedBufferGuard, LockedPageTupleVisit},
    page::ItemPointer,
    relation::RelationHandle,
};

use super::{
    quantizer::DistannPreparedQuery,
    reader::directory_lookup,
    scan::{DistannExpandedNode, DistannNodeExpander},
    tuple::{DistannNodeTuple, DISTANN_NODE_TAG},
};

pub(crate) struct LocalNodeExpander<'a> {
    pub(crate) index_handle: RelationHandle,
    pub(crate) directory: &'a [(u64, ItemPointer)],
    pub(crate) graph_degree_r: u16,
    pub(crate) code_len: usize,
    pub(crate) prepared_query: &'a DistannPreparedQuery,
    pub(crate) heap_relation: pg_sys::Relation,
    pub(crate) snapshot: pg_sys::Snapshot,
    pub(crate) slot: *mut pg_sys::TupleTableSlot,
    pub(crate) source_attnum: i32,
    pub(crate) raw_query: &'a [f32],
    /// Pooled record buffer (Task-168 zero-alloc pattern).
    pub(crate) pooled_node: DistannNodeTuple,
}

impl LocalNodeExpander<'_> {
    fn read_node_into_pool(&mut self, tid: ItemPointer) -> Result<(), String> {
        let buffer = LockedBufferGuard::read_main_handle(
            self.index_handle,
            tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
        .ok_or_else(|| {
            format!(
                "ec_distann expansion could not open block {}",
                tid.block_number
            )
        })?;
        let visit = buffer.visit_tuple_bytes(tid, "ec_distann node record", |raw| {
            if raw.first().copied() != Some(DISTANN_NODE_TAG) {
                return Err(format!(
                    "ec_distann structural fault: tuple at ({},{}) is not a graph-node record",
                    tid.block_number, tid.offset_number
                ));
            }
            DistannNodeTuple::decode_into(raw, self.graph_degree_r, self.code_len, &mut self.pooled_node)
        })?;
        match visit {
            LockedPageTupleVisit::Present(()) => Ok(()),
            _ => Err(format!(
                "ec_distann structural fault: record owned but absent at ({},{})",
                tid.block_number, tid.offset_number
            )),
        }
    }
}

impl DistannNodeExpander for LocalNodeExpander<'_> {
    fn expand_nodes(
        &mut self,
        vec_ids: &[u64],
        code_threshold: Option<f32>,
    ) -> Result<Vec<DistannExpandedNode>, String> {
        let mut responses = Vec::with_capacity(vec_ids.len());
        for vec_id in vec_ids {
            // Single-node deployment owns every vec_id: an unresolved id is
            // the FR-079 owned-but-absent structural fault, never a miss.
            let tid = directory_lookup(self.directory, *vec_id).ok_or_else(|| {
                format!(
                    "ec_distann structural fault: vec_id {vec_id:#018x} is not in the directory"
                )
            })?;
            self.read_node_into_pool(tid)?;
            let node = &self.pooled_node;
            if node.vec_id != *vec_id {
                return Err(format!(
                    "ec_distann structural fault: directory maps vec_id {vec_id:#018x} to a record carrying {:#018x}",
                    node.vec_id
                ));
            }

            let is_tombstone = node.tombstoned;
            let heap_tid = node.heap_tid;
            let neighbor_count = usize::from(node.neighbor_count);
            let mut neighbor_vec_ids = Vec::with_capacity(neighbor_count);
            let mut neighbor_code_dists = Vec::with_capacity(neighbor_count);
            for slot in 0..neighbor_count {
                let code =
                    &node.neighbor_codes[slot * self.code_len..(slot + 1) * self.code_len];
                let code_dist = self.prepared_query.score_dist(code);
                // FR-079 code_threshold: an ip-score floor; neighbors whose
                // estimated ip (-code_dist) falls below it MAY be omitted.
                if let Some(threshold) = code_threshold {
                    if -code_dist < threshold {
                        continue;
                    }
                }
                neighbor_vec_ids.push(node.neighbor_vec_ids[slot]);
                neighbor_code_dists.push(code_dist);
            }

            // Tombstones skip the vector read (FR-079); their edges above
            // keep traversal continuity.
            let exact_dist = if is_tombstone {
                None
            } else {
                Some(exact_heap_rerank_distance(
                    self.heap_relation,
                    self.snapshot,
                    self.slot,
                    self.source_attnum,
                    self.raw_query,
                    heap_tid,
                )?)
            };

            responses.push(DistannExpandedNode {
                vec_id: *vec_id,
                exact_dist,
                is_tombstone,
                heap_tid,
                neighbor_vec_ids,
                neighbor_code_dists,
            });
        }
        Ok(responses)
    }
}
