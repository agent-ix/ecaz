//! `TRAV-30` bounded gateway copies (Task 210 P3).
//!
//! The withdrawn FR-084 traversal replica removed transport wait by removing
//! distribution: it held every owner's graph record and full-precision vector
//! on one node, O(N) in the corpus. `TRAV-30` is the conforming direction the
//! Task 190 narrowing dropped — copy only a **bounded** set of frequently
//! traversed gateway nodes' *routing* information, never the row tier and never
//! the whole graph.
//!
//! What makes this conforming under `NFR-021`:
//!
//! - capacity is a stated constant, independent of `N`, enforced here;
//! - only neighbour ids and neighbour codes are copied — the routing payload —
//!   never full-precision vectors, so this is not a row-tier replica;
//! - the copy is epoch-scoped and rebuildable, so it is a cache, not an
//!   authoritative shard.
//!
//! The natural gateway set is the FR-080 head's landmarks: they are already
//! bounded by head capacity `C`, and they are exactly the nodes every scan
//! expands first, so caching their routing payload removes the first hop's
//! remote round trip without moving corpus-proportional state anywhere.

use std::sync::atomic::{AtomicU64, Ordering};

use pgrx::iter::TableIterator;
use pgrx::{name, pg_extern};

use super::expand_error::DistannExpandError;
use super::scan::DistannExpandedNode;

/// One gateway node's routing payload. No full-precision vector: routing only.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DistannGatewayCopy {
    pub(crate) vec_id: u64,
    pub(crate) is_tombstone: bool,
    pub(crate) neighbor_vec_ids: Vec<u64>,
    pub(crate) neighbor_codes: Vec<u8>,
}

/// A bounded, epoch-scoped set of gateway copies.
///
/// `capacity` is the NFR-021 bound: a constant, never a function of `N`.
/// Inserts past capacity are refused rather than evicting, so the structure
/// cannot grow past its stated bound under any traversal pattern.
#[derive(Debug, Default)]
pub(crate) struct DistannGatewayCopySet {
    capacity: usize,
    entries: Vec<DistannGatewayCopy>,
}

impl DistannGatewayCopySet {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Vec::new(),
        }
    }

    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Insert a gateway copy if the set is under its bound. Returns whether it
    /// was retained. Duplicate vec_ids replace in place, so the bound holds.
    pub(crate) fn insert(&mut self, copy: DistannGatewayCopy) -> bool {
        if let Some(existing) = self
            .entries
            .iter_mut()
            .find(|entry| entry.vec_id == copy.vec_id)
        {
            *existing = copy;
            return true;
        }
        if self.entries.len() >= self.capacity {
            return false;
        }
        self.entries.push(copy);
        true
    }

    pub(crate) fn get(&self, vec_id: u64) -> Option<&DistannGatewayCopy> {
        self.entries.iter().find(|entry| entry.vec_id == vec_id)
    }

    /// Split a traversal request into ids this set can answer locally and ids
    /// that must still go to their owner. Ordering of both outputs follows the
    /// request, so response reassembly stays position-driven (FR-079-AC-1).
    pub(crate) fn split_request(&self, vec_ids: &[u64]) -> (Vec<u64>, Vec<u64>) {
        let mut local = Vec::new();
        let mut remote = Vec::new();
        for vec_id in vec_ids {
            if self.get(*vec_id).is_some() {
                local.push(*vec_id);
            } else {
                remote.push(*vec_id);
            }
        }
        (local, remote)
    }

    /// Resident bytes of the routing payload, for the NFR-021 per-node audit.
    /// This is what a conformance run reports as the gateway copy's cost, and
    /// it must not grow with `N`.
    pub(crate) fn resident_bytes(&self) -> usize {
        self.entries
            .iter()
            .map(|entry| {
                entry
                    .neighbor_vec_ids
                    .len()
                    .saturating_mul(size_of::<u64>())
                    .saturating_add(entry.neighbor_codes.len())
                    .saturating_add(size_of::<u64>() + 1)
            })
            .sum()
    }
}

/// Per-backend activation accounting. Task 205's pushdown shipped inert and
/// passed a full benchmark, so the gateway copy lands with its own counters:
/// a run that claims the mechanism is active must show `served > 0` here (or
/// via the benchmark stage counters), never infer it from latency.
static GATEWAY_COPY_ENTRIES: AtomicU64 = AtomicU64::new(0);
static GATEWAY_COPY_RESIDENT_BYTES: AtomicU64 = AtomicU64::new(0);
static GATEWAY_COPIES_SERVED: AtomicU64 = AtomicU64::new(0);

pub(crate) fn record_population(set: &DistannGatewayCopySet) {
    GATEWAY_COPY_ENTRIES.store(set.len() as u64, Ordering::Relaxed);
    GATEWAY_COPY_RESIDENT_BYTES.store(set.resident_bytes() as u64, Ordering::Relaxed);
}

pub(crate) fn record_served(count: usize) {
    GATEWAY_COPIES_SERVED.fetch_add(count as u64, Ordering::Relaxed);
}

/// A capacity change discards the resident set (004 review, 2026-07-31); the
/// size gauges reset with it so stats never describe a set that no longer
/// exists. The `served` total is a monotone activation counter and is kept.
pub(crate) fn record_cleared() {
    GATEWAY_COPY_ENTRIES.store(0, Ordering::Relaxed);
    GATEWAY_COPY_RESIDENT_BYTES.store(0, Ordering::Relaxed);
}

/// Observability for the TRAV-30 gateway copy: how big the bounded copy set is
/// on this backend and how many expansions it has actually served. Read-only.
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_gateway_copy_stats() -> TableIterator<
    'static,
    (
        name!(capacity, i64),
        name!(entries, i64),
        name!(resident_bytes, i64),
        name!(served, i64),
    ),
> {
    TableIterator::new(std::iter::once((
        i64::try_from(super::options::gateway_copy_capacity()).unwrap_or(i64::MAX),
        i64::try_from(GATEWAY_COPY_ENTRIES.load(Ordering::Relaxed)).unwrap_or(i64::MAX),
        i64::try_from(GATEWAY_COPY_RESIDENT_BYTES.load(Ordering::Relaxed)).unwrap_or(i64::MAX),
        i64::try_from(GATEWAY_COPIES_SERVED.load(Ordering::Relaxed)).unwrap_or(i64::MAX),
    )))
}

/// Fill the neighbour payload of gateway-cached response rows from the local
/// copy set. `cached` is index-aligned with `responses`; a cached row arrived
/// from its owner with an empty neighbour payload (the owner skipped scoring
/// and the wire skipped the bytes), and this reconstructs Algorithm 1's
/// candidate half locally: score the copied neighbour codes, then apply the
/// same per-row threshold the owner applies (`prune_and_limit_neighbors` with
/// no per-row limit — the batch L limit is re-applied by the caller across the
/// whole owner batch, preserving the Task 205 batch semantics).
///
/// The result half (`exact_dist`, `is_tombstone`) stays owner-authoritative:
/// reproducing it locally would require full-precision vectors at the
/// coordinator, which is exactly the FR-084 trap (NFR-021).
pub(crate) fn fill_gateway_rows<F>(
    responses: &mut [DistannExpandedNode],
    cached: &[bool],
    set: &DistannGatewayCopySet,
    code_threshold: Option<f32>,
    mut score: F,
) -> Result<usize, DistannExpandError>
where
    F: FnMut(&DistannGatewayCopy) -> Result<Vec<f32>, String>,
{
    if responses.len() != cached.len() {
        return Err(DistannExpandError::Internal(
            "gateway fill mask is not aligned with the owner response".to_owned(),
        ));
    }
    let mut filled = 0;
    for (response, is_cached) in responses.iter_mut().zip(cached) {
        if !is_cached {
            continue;
        }
        let copy = set.get(response.vec_id).ok_or_else(|| {
            DistannExpandError::Internal(format!(
                "gateway copy for vec_id {:#018x} disappeared between request and response",
                response.vec_id
            ))
        })?;
        if !response.neighbor_vec_ids.is_empty() {
            return Err(DistannExpandError::Internal(format!(
                "owner returned a neighbour payload for gateway-cached vec_id {:#018x}",
                response.vec_id
            )));
        }
        let dists = score(copy).map_err(DistannExpandError::Internal)?;
        let (neighbor_vec_ids, neighbor_code_dists) = super::scan::prune_and_limit_neighbors(
            &copy.neighbor_vec_ids,
            &dists,
            code_threshold,
            None,
        )?;
        response.neighbor_vec_ids = neighbor_vec_ids;
        response.neighbor_code_dists = neighbor_code_dists;
        filled += 1;
    }
    Ok(filled)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn copy(vec_id: u64) -> DistannGatewayCopy {
        DistannGatewayCopy {
            vec_id,
            is_tombstone: false,
            neighbor_vec_ids: vec![vec_id + 1, vec_id + 2],
            neighbor_codes: vec![0u8; 8],
        }
    }

    #[test]
    fn gateway_copies_never_exceed_their_stated_capacity() {
        let mut set = DistannGatewayCopySet::with_capacity(3);
        for vec_id in 0..10 {
            set.insert(copy(vec_id));
        }
        assert_eq!(set.len(), 3, "the bound is enforced, not merely intended");
        assert_eq!(set.capacity(), 3);
        // Refusal, not eviction: an admitted gateway stays admitted, so the
        // structure is stable rather than thrashing under traversal load.
        assert!(set.get(0).is_some());
        assert!(set.get(9).is_none());
    }

    #[test]
    fn reinserting_a_gateway_replaces_rather_than_growing() {
        let mut set = DistannGatewayCopySet::with_capacity(2);
        assert!(set.insert(copy(1)));
        assert!(set.insert(copy(2)));
        assert!(!set.insert(copy(3)), "at capacity");
        let mut updated = copy(1);
        updated.is_tombstone = true;
        assert!(set.insert(updated), "duplicate replaces in place");
        assert_eq!(set.len(), 2);
        assert!(set.get(1).expect("still present").is_tombstone);
    }

    #[test]
    fn split_request_separates_locally_answerable_ids_in_request_order() {
        let mut set = DistannGatewayCopySet::with_capacity(4);
        set.insert(copy(10));
        set.insert(copy(30));

        let (local, remote) = set.split_request(&[30, 20, 10, 40]);

        assert_eq!(local, vec![30, 10]);
        assert_eq!(remote, vec![20, 40]);
    }

    fn expanded(vec_id: u64, neighbors: &[(u64, f32)]) -> super::DistannExpandedNode {
        super::DistannExpandedNode {
            vec_id,
            exact_dist: Some(0.0),
            is_tombstone: false,
            heap_tid: crate::storage::page::ItemPointer::INVALID,
            owner_heap_tid: crate::storage::page::ItemPointer::INVALID,
            neighbor_vec_ids: neighbors.iter().map(|(id, _)| *id).collect(),
            neighbor_code_dists: neighbors.iter().map(|(_, dist)| *dist).collect(),
            neighbors_pruned: 0,
            owner_total_ns: 0,
            owner_open_validate_ns: 0,
            owner_graph_read_ns: 0,
            owner_score_ns: 0,
            owner_response_encode_ns: 0,
            owner_response_bytes: 0,
            coordinator_rpc_ns: 0,
            coordinator_decode_ns: 0,
        }
    }

    /// The Task 205 semantics-preservation property: (owner threshold + batch-L
    /// over the uncached subset) → coordinator fill of cached rows → batch-L
    /// re-applied over the whole owner batch in original row positions must
    /// select exactly what the owner-only path selects. Duplicated neighbour
    /// ids with equal scores across cached and uncached rows exercise the
    /// positional tie-break.
    #[test]
    fn gateway_fill_and_rebatch_matches_the_owner_only_batch_semantics() {
        let threshold = Some(0.35_f32); // keeps neighbours with code_dist <= -0.35
        let limit = Some(5);
        let rows: [(u64, &[(u64, f32)]); 4] = [
            (10, &[(100, -0.9), (101, -0.3), (102, -0.5)]),
            (11, &[(100, -0.9), (103, -0.7), (104, -0.4)]),
            (12, &[(105, -0.6), (106, -0.5)]),
            (13, &[(107, -0.8), (104, -0.4), (108, -0.36)]),
        ];
        let cached = [false, true, false, true];

        let mut baseline = rows
            .iter()
            .map(|(vec_id, neighbors)| expanded(*vec_id, neighbors))
            .collect::<Vec<_>>();
        super::super::scan::prune_and_limit_neighbor_batch(&mut baseline, threshold, limit)
            .expect("baseline batch");

        // The owner served cached rows with an empty neighbour payload and
        // applied threshold + batch-L to what remained.
        let mut candidate = rows
            .iter()
            .zip(cached)
            .map(|((vec_id, neighbors), is_cached)| {
                expanded(*vec_id, if is_cached { &[] } else { neighbors })
            })
            .collect::<Vec<_>>();
        super::super::scan::prune_and_limit_neighbor_batch(&mut candidate, threshold, limit)
            .expect("owner subset batch");

        let mut set = DistannGatewayCopySet::with_capacity(4);
        let mut scores = std::collections::HashMap::new();
        for ((vec_id, neighbors), is_cached) in rows.iter().zip(cached) {
            if is_cached {
                set.insert(DistannGatewayCopy {
                    vec_id: *vec_id,
                    is_tombstone: false,
                    neighbor_vec_ids: neighbors.iter().map(|(id, _)| *id).collect(),
                    neighbor_codes: Vec::new(),
                });
                scores.insert(
                    *vec_id,
                    neighbors.iter().map(|(_, dist)| *dist).collect::<Vec<_>>(),
                );
            }
        }
        let filled = fill_gateway_rows(&mut candidate, &cached, &set, threshold, |copy| {
            Ok(scores[&copy.vec_id].clone())
        })
        .expect("fill");
        assert_eq!(filled, 2);
        super::super::scan::prune_and_limit_neighbor_batch(&mut candidate, None, limit)
            .expect("re-batch");

        for (base, cand) in baseline.iter().zip(&candidate) {
            assert_eq!(base.neighbor_vec_ids, cand.neighbor_vec_ids);
            assert_eq!(base.neighbor_code_dists, cand.neighbor_code_dists);
        }
    }

    #[test]
    fn resident_bytes_is_bounded_by_capacity_not_by_corpus() {
        let mut small = DistannGatewayCopySet::with_capacity(2);
        let mut large = DistannGatewayCopySet::with_capacity(2);
        for vec_id in 0..2 {
            small.insert(copy(vec_id));
        }
        // A "bigger corpus" cannot make the copy set bigger: only capacity can.
        for vec_id in 0..1_000 {
            large.insert(copy(vec_id));
        }
        assert_eq!(small.resident_bytes(), large.resident_bytes());
    }
}
