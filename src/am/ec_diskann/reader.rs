//! Persisted-graph reader for `ec_diskann` (task 17 Phase 5D).
//!
//! Bridge between Phase 5C-1's persist output ([`DataPageChain`] of
//! encoded [`VamanaNodeTuple`]s) and the eventual scan algorithm
//! (Phase 6A). Callers hand us a `&DataPageChain` plus the metadata
//! `(R, W, C)` triple and get:
//!
//! - [`PersistedGraphReader::read_node`] — decode one tuple at a TID.
//! - [`PersistedGraphReader::neighbors`] — the filled prefix of a
//!   node's neighbor TIDs (tail `INVALID` slots are dropped).
//! - [`greedy_search_persisted`] — same greedy best-first traversal
//!   as [`crate::am::ec_diskann::vamana::greedy_search`], but keyed on
//!   `ItemPointer` instead of dense `u32` node ids.
//!
//! This module is pure-Rust and does not know about tombstones,
//! visibility maps, or MVCC. Phase 6A's scan layer composes this
//! reader with a tombstone filter + quantizer-backed query distance
//! closure to implement `amgettuple`; tombstone-aware logic
//! deliberately does not live here so the reader stays cheaply
//! testable with synthetic distances.
//!
//! ## Invariant reliance
//!
//! - Every tuple in `chain` decodes cleanly at the stored `(R, W, C)`
//!   triple — guaranteed by Phase 5C-1 + ADR-045 Decision 3.
//! - The reader is a reference holder only; it does not mutate the
//!   chain and does not cache decoded tuples. Callers that need to
//!   read the same node repeatedly should decode once and reuse.

use std::collections::HashSet;

use crate::am::ec_diskann::tuple::VamanaNodeTuple;
use crate::storage::page::{DataPageChain, ItemPointer};

/// Handle to a persisted Vamana graph. Holds a borrowed
/// [`DataPageChain`] and the metadata-page `(R, W, C)` triple needed
/// to decode tuples.
#[derive(Debug, Clone, Copy)]
pub struct PersistedGraphReader<'a> {
    pub chain: &'a DataPageChain,
    pub graph_degree_r: u16,
    pub binary_word_count: usize,
    pub search_code_len: usize,
}

impl<'a> PersistedGraphReader<'a> {
    pub fn new(
        chain: &'a DataPageChain,
        graph_degree_r: u16,
        binary_word_count: usize,
        search_code_len: usize,
    ) -> Self {
        Self {
            chain,
            graph_degree_r,
            binary_word_count,
            search_code_len,
        }
    }

    /// Decode the tuple at `tid`. Errors surface from page lookup,
    /// raw-tuple lookup, or decode.
    pub fn read_node(&self, tid: ItemPointer) -> Result<VamanaNodeTuple, String> {
        let page = self
            .chain
            .get_page(tid.block_number)
            .ok_or_else(|| format!("page {} not found in chain", tid.block_number))?;
        let raw = page.raw_tuple(tid)?;
        VamanaNodeTuple::decode(
            raw,
            self.graph_degree_r,
            self.binary_word_count,
            self.search_code_len,
        )
    }

    /// Return only the filled prefix of `tid`'s neighbors. Length is
    /// the tuple's `neighbor_count`; tail `INVALID` slots are
    /// dropped. Does not distinguish tombstoned neighbors — the scan
    /// layer filters those after reading.
    pub fn neighbors(&self, tid: ItemPointer) -> Result<Vec<ItemPointer>, String> {
        let tuple = self.read_node(tid)?;
        let count = tuple.neighbor_count as usize;
        Ok(tuple.neighbors.into_iter().take(count).collect())
    }

    /// Iterate every occupied TID in the chain in
    /// `(block_number, offset_number)` order, without decoding.
    /// Includes tombstoned tuples; callers that want live-only
    /// filtering should use [`Self::iter_live_tids`].
    pub fn iter_tids(&self) -> impl Iterator<Item = ItemPointer> + '_ {
        self.chain.pages().iter().flat_map(|page| {
            let blk = page.block_number();
            let count = page.tuple_count();
            (1..=count).map(move |offset| ItemPointer {
                block_number: blk,
                offset_number: offset as u16,
            })
        })
    }

    /// Iterate every live tuple in the chain in `(block, offset)`
    /// order, yielding `(tid, tuple)` pairs. "Live" means
    /// [`VamanaNodeTuple::is_live`]: not tombstoned AND still has at
    /// least one heap TID (primary or overflow). A stripped-but-not-
    /// tombstoned tuple (vacuum pass 1 done, pass 3 not yet) is
    /// skipped — it has no heap row to emit and is not a valid scan
    /// entry point. A decode error is yielded as `Err` and callers
    /// should stop iteration.
    pub fn iter_live_tids(
        &self,
    ) -> impl Iterator<Item = Result<(ItemPointer, VamanaNodeTuple), String>> + '_ {
        self.iter_tids()
            .filter_map(move |tid| match self.read_node(tid) {
                Ok(t) if t.is_live() => Some(Ok((tid, t))),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
    }

    /// Return the lowest-block, lowest-offset live TID in the chain,
    /// or `None` if every tuple is tombstoned. Phase 6B uses this as
    /// an entry-point fallback when the medoid TID itself points to
    /// a deleted element (ADR-047 §10 defers medoid migration to
    /// rebuild). Also the scaffold for ADR-047 pass-3 orphan
    /// detection.
    pub fn first_live_tid(&self) -> Result<Option<ItemPointer>, String> {
        match self.iter_live_tids().next() {
            Some(item) => item.map(|(tid, _)| Some(tid)),
            None => Ok(None),
        }
    }
}

/// Result of [`greedy_search_persisted`]: final frontier (top-`L`
/// candidates by distance) and the full visited set, both keyed on
/// `ItemPointer`.
#[derive(Debug, Clone)]
pub struct PersistedGreedyResult {
    /// Candidates in distance order, ascending. At most `list_size`
    /// entries.
    pub frontier: Vec<TidCandidate>,
    /// Every TID whose neighbors were expanded, in expansion order.
    pub visited: Vec<ItemPointer>,
}

/// Reusable scratch for greedy traversal. Phase 6B's pgrx scan
/// path will allocate one of these per index cursor and `clear()`
/// between queries so `greedy_search_persisted_with` /
/// `greedy_descent_with` don't allocate on the hot path.
#[derive(Debug, Default)]
pub struct VisitedState {
    pub(crate) in_frontier: HashSet<ItemPointer>,
    pub(crate) visited: HashSet<ItemPointer>,
}

impl VisitedState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Drop all membership state. O(n) in current entry count.
    pub fn clear(&mut self) {
        self.in_frontier.clear();
        self.visited.clear();
    }

    /// Pre-reserve capacity for both sets. Useful if the caller
    /// knows a rough bound on distinct visited TIDs (e.g., 2 ×
    /// `list_size`).
    pub fn reserve(&mut self, additional: usize) {
        self.in_frontier.reserve(additional);
        self.visited.reserve(additional);
    }
}

/// `(tid, distance)` pair. Same semantics as [`Candidate`] but keyed
/// on `ItemPointer` instead of `u32`.
#[derive(Debug, Clone, Copy)]
pub struct TidCandidate {
    pub tid: ItemPointer,
    pub distance: f32,
}

impl PartialEq for TidCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance && self.tid == other.tid
    }
}
impl Eq for TidCandidate {}
impl Ord for TidCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // NaN sorts last, matching vamana::Candidate.
        self.distance
            .partial_cmp(&other.distance)
            .unwrap_or(std::cmp::Ordering::Greater)
            .then_with(|| self.tid.block_number.cmp(&other.tid.block_number))
            .then_with(|| self.tid.offset_number.cmp(&other.tid.offset_number))
    }
}
impl PartialOrd for TidCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Greedy best-first search over a persisted graph. Same loop shape
/// as [`crate::am::ec_diskann::vamana::greedy_search`] but reads
/// neighbors via the reader and tracks frontier membership in a
/// `HashSet<ItemPointer>` (TIDs are sparse; a dense `Vec<bool>`
/// doesn't apply).
///
/// Returns the top-`list_size` frontier sorted ascending by distance
/// and the visited set in expansion order.
pub fn greedy_search_persisted<D>(
    reader: &PersistedGraphReader<'_>,
    entry_point: ItemPointer,
    list_size: usize,
    query_dist: D,
) -> Result<PersistedGreedyResult, String>
where
    D: Fn(ItemPointer) -> f32,
{
    let mut scratch = VisitedState::new();
    greedy_search_persisted_with(reader, &mut scratch, entry_point, list_size, query_dist)
}

/// Same contract as [`greedy_search_persisted`] but reuses a
/// caller-owned [`VisitedState`] scratch. The scratch is cleared on
/// entry so previous contents (if any) are discarded. Use this from
/// hot paths (Phase 6B pgrx scan cursor) to avoid the per-call
/// `HashSet` allocations.
pub fn greedy_search_persisted_with<D>(
    reader: &PersistedGraphReader<'_>,
    scratch: &mut VisitedState,
    entry_point: ItemPointer,
    list_size: usize,
    query_dist: D,
) -> Result<PersistedGreedyResult, String>
where
    D: Fn(ItemPointer) -> f32,
{
    if list_size == 0 {
        return Err("list_size must be > 0".into());
    }
    if entry_point == ItemPointer::INVALID {
        return Err("entry_point must not be INVALID".into());
    }

    scratch.clear();
    let mut visited_order: Vec<ItemPointer> = Vec::new();

    let start_dist = query_dist(entry_point);
    let mut frontier: Vec<TidCandidate> = vec![TidCandidate {
        tid: entry_point,
        distance: start_dist,
    }];
    scratch.in_frontier.insert(entry_point);

    loop {
        let next = frontier
            .iter()
            .copied()
            .filter(|c| !scratch.visited.contains(&c.tid))
            .min_by(|a, b| a.cmp(b));
        let Some(picked) = next else {
            break;
        };
        scratch.visited.insert(picked.tid);
        visited_order.push(picked.tid);

        let neighbors = reader.neighbors(picked.tid)?;
        for nbr in neighbors {
            if nbr == ItemPointer::INVALID {
                // Defensive: fill-only prefix shouldn't contain
                // INVALID, but a future repair primitive may
                // interleave INVALID before compaction — skip.
                continue;
            }
            if scratch.in_frontier.contains(&nbr) {
                continue;
            }
            let d = query_dist(nbr);
            frontier.push(TidCandidate {
                tid: nbr,
                distance: d,
            });
            scratch.in_frontier.insert(nbr);
        }

        if frontier.len() > list_size {
            frontier.sort_by(|a, b| a.cmp(b));
            for c in &frontier[list_size..] {
                scratch.in_frontier.remove(&c.tid);
            }
            frontier.truncate(list_size);
        }
    }

    frontier.sort_by(|a, b| a.cmp(b));
    Ok(PersistedGreedyResult {
        frontier,
        visited: visited_order,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::ec_diskann::build::{build_and_persist_vamana, BuildParams};
    use crate::am::ec_diskann::persist::{persist_vamana_graph, NodePayload};
    use crate::am::ec_diskann::vamana::{
        approximate_medoid, build_vamana_graph, greedy_search, VamanaGraph,
    };
    use crate::storage::page::DEFAULT_PAGE_SIZE;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;

    fn synth_payloads(n: usize, w: usize, c: usize) -> Vec<NodePayload> {
        (0..n)
            .map(|i| NodePayload {
                primary_heaptid: ItemPointer {
                    block_number: 1000 + i as u32,
                    offset_number: 1,
                },
                binary_words: vec![i as u64; w],
                search_code: vec![(i & 0xff) as u8; c],
            })
            .collect()
    }

    fn chain_graph(n: usize, max_degree: usize) -> VamanaGraph {
        let mut g = VamanaGraph::empty(n, max_degree);
        for i in 0..n.saturating_sub(1) {
            g.neighbors[i].push((i + 1) as u32);
            g.neighbors[i + 1].push(i as u32);
        }
        g
    }

    // RD-001: read_node round-trips a single persisted tuple.
    #[test]
    fn rd_001_read_node_round_trip() {
        let g = VamanaGraph::empty(1, 4);
        let payloads = synth_payloads(1, 2, 8);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 2, 8).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 2, 8);
        let node = reader
            .read_node(persisted.entry_point_tid)
            .expect("read_node");
        assert_eq!(node.primary_heaptid, payloads[0].primary_heaptid);
        assert_eq!(node.binary_words, payloads[0].binary_words);
        assert_eq!(node.search_code, payloads[0].search_code);
        assert_eq!(node.neighbor_count, 0);
    }

    // RD-002: read_node errors on unknown block.
    #[test]
    fn rd_002_read_node_unknown_block_errors() {
        let g = VamanaGraph::empty(1, 4);
        let payloads = synth_payloads(1, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 0, 0);
        let bogus = ItemPointer {
            block_number: 9999,
            offset_number: 1,
        };
        let err = reader.read_node(bogus).expect_err("unknown block");
        assert!(err.contains("page"), "got: {err}");
    }

    // RD-003: neighbors() returns only the filled prefix — no
    // trailing INVALID slots.
    #[test]
    fn rd_003_neighbors_drops_invalid_tail() {
        let n = 4;
        let g = chain_graph(n, 8);
        let payloads = synth_payloads(n, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 8, 0, 0).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 8, 0, 0);

        let tid0 = persisted.node_to_tid[0];
        let nbrs0 = reader.neighbors(tid0).expect("neighbors");
        assert_eq!(nbrs0.len(), 1, "node 0 has one chain neighbor");
        assert_eq!(nbrs0[0], persisted.node_to_tid[1]);
        assert!(nbrs0.iter().all(|n| *n != ItemPointer::INVALID));

        let tid_mid = persisted.node_to_tid[1];
        let nbrs_mid = reader.neighbors(tid_mid).expect("neighbors");
        assert_eq!(nbrs_mid.len(), 2);
    }

    // RD-004: BFS via the reader reaches every node in a connected
    // chain and the reached set matches the in-memory graph.
    #[test]
    fn rd_004_reader_bfs_reaches_all() {
        let n = 12;
        let g = chain_graph(n, 4);
        let payloads = synth_payloads(n, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 0, 0);

        let mut seen: HashSet<ItemPointer> = HashSet::new();
        let mut queue: Vec<ItemPointer> = vec![persisted.entry_point_tid];
        seen.insert(persisted.entry_point_tid);
        while let Some(tid) = queue.pop() {
            for nbr in reader.neighbors(tid).expect("neighbors") {
                if seen.insert(nbr) {
                    queue.push(nbr);
                }
            }
        }
        assert_eq!(seen.len(), n, "BFS must reach every persisted node");
    }

    // RD-005: adjacency via the reader matches the in-memory graph
    // for every node after persistence.
    #[test]
    fn rd_005_adjacency_matches_in_memory() {
        let n = 8;
        let g = chain_graph(n, 4);
        let payloads = synth_payloads(n, 1, 4);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 1, 4).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 1, 4);

        for node_id in 0..n as u32 {
            let tid = persisted.node_to_tid[node_id as usize];
            let got: Vec<ItemPointer> = reader.neighbors(tid).expect("neighbors");
            let expected: Vec<ItemPointer> = g.neighbors[node_id as usize]
                .iter()
                .map(|&nbr| persisted.node_to_tid[nbr as usize])
                .collect();
            assert_eq!(got, expected, "adjacency mismatch at node {node_id}");
        }
    }

    // RD-006: greedy_search_persisted and the in-memory greedy_search
    // agree on the visited set and top-1 result when driven with the
    // same distance function (oracle test). Uses a real Vamana build.
    #[test]
    fn rd_006_greedy_matches_in_memory_oracle() {
        let n = 40;
        let mut rng = ChaCha8Rng::seed_from_u64(11);
        let points: Vec<(f32, f32)> =
            (0..n).map(|_| (rng.gen::<f32>(), rng.gen::<f32>())).collect();
        let dist = |a: u32, b: u32| {
            let (ax, ay) = points[a as usize];
            let (bx, by) = points[b as usize];
            let dx = ax - bx;
            let dy = ay - by;
            dx * dx + dy * dy
        };
        let medoid = approximate_medoid(n, n, 11, dist);
        let graph = build_vamana_graph(n, medoid, 8, 32, 1.2, 13, dist);

        let payloads = synth_payloads(n, 0, 0);
        let persisted =
            persist_vamana_graph(&graph, medoid, DEFAULT_PAGE_SIZE, &payloads, 8, 0, 0)
                .expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 8, 0, 0);

        // Query point is a fresh random 2D.
        let query = (rng.gen::<f32>(), rng.gen::<f32>());
        let query_dist_id = |n: u32| {
            let (ax, ay) = points[n as usize];
            let dx = ax - query.0;
            let dy = ay - query.1;
            dx * dx + dy * dy
        };
        let tid_to_node: std::collections::HashMap<ItemPointer, u32> = persisted
            .node_to_tid
            .iter()
            .enumerate()
            .map(|(i, &tid)| (tid, i as u32))
            .collect();
        let query_dist_tid =
            |t: ItemPointer| query_dist_id(*tid_to_node.get(&t).expect("tid in map"));

        let list_size = 10;
        let in_mem = greedy_search(&graph, medoid, list_size, query_dist_id);
        let on_disk = greedy_search_persisted(
            &reader,
            persisted.entry_point_tid,
            list_size,
            query_dist_tid,
        )
        .expect("greedy_search_persisted");

        let in_mem_top = in_mem.frontier[0].node;
        let on_disk_top = tid_to_node[&on_disk.frontier[0].tid];
        assert_eq!(
            in_mem_top, on_disk_top,
            "top-1 must match between in-memory and persisted greedy"
        );

        let in_mem_visited: HashSet<u32> = in_mem.visited.into_iter().collect();
        let on_disk_visited: HashSet<u32> = on_disk
            .visited
            .into_iter()
            .map(|t| tid_to_node[&t])
            .collect();
        assert_eq!(
            in_mem_visited, on_disk_visited,
            "visited sets must match between in-memory and persisted greedy"
        );
    }

    // RD-007: greedy_search_persisted descends a distance gradient
    // and ends with the target at the head of the frontier. The
    // frontier must be large enough to carry the chain hop-by-hop
    // (list_size ≥ 2 for a chain graph); list_size=3 gives comfortable
    // headroom and still asserts the top-1 result.
    #[test]
    fn rd_007_greedy_descends_gradient() {
        let n = 6;
        let g = chain_graph(n, 4);
        let payloads = synth_payloads(n, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 0, 0);
        // Distance is (n-1 - node_id): node 5 is the target (dist 0).
        let tid_to_node: std::collections::HashMap<ItemPointer, u32> = persisted
            .node_to_tid
            .iter()
            .enumerate()
            .map(|(i, &tid)| (tid, i as u32))
            .collect();
        let qd = |t: ItemPointer| ((n - 1) as u32 - tid_to_node[&t]) as f32;
        let res = greedy_search_persisted(&reader, persisted.entry_point_tid, 3, qd)
            .expect("greedy");
        let target = persisted.node_to_tid[5];
        assert_eq!(res.frontier[0].tid, target, "top-1 must be the target");
        assert_eq!(res.frontier[0].distance, 0.0);
    }

    // RD-008: greedy_search_persisted rejects INVALID entry_point and
    // list_size=0.
    #[test]
    fn rd_008_greedy_rejects_bad_args() {
        let g = VamanaGraph::empty(1, 4);
        let payloads = synth_payloads(1, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 0, 0);

        let err = greedy_search_persisted(&reader, ItemPointer::INVALID, 4, |_| 0.0)
            .expect_err("INVALID entry");
        assert!(err.contains("entry_point"), "got: {err}");

        let err = greedy_search_persisted(&reader, persisted.entry_point_tid, 0, |_| 0.0)
            .expect_err("zero list_size");
        assert!(err.contains("list_size"), "got: {err}");
    }

    // RD-009: frontier returned from greedy_search_persisted is
    // sorted ascending by distance.
    #[test]
    fn rd_009_frontier_is_sorted() {
        let n = 20;
        let g = chain_graph(n, 4);
        let payloads = synth_payloads(n, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 0, 0);
        // Node id from tid for distance = node_id as f32 (so sorted
        // order is deterministic and non-trivial).
        let tid_to_node: std::collections::HashMap<ItemPointer, u32> = persisted
            .node_to_tid
            .iter()
            .enumerate()
            .map(|(i, &tid)| (tid, i as u32))
            .collect();
        let qd = |t: ItemPointer| tid_to_node[&t] as f32;

        let res =
            greedy_search_persisted(&reader, persisted.entry_point_tid, 5, qd).expect("greedy");
        assert_eq!(res.frontier.len(), 5);
        for pair in res.frontier.windows(2) {
            assert!(
                pair[0].distance <= pair[1].distance,
                "frontier must be ascending by distance"
            );
        }
    }

    // RD-010: end-to-end bridge from build_and_persist_vamana to the
    // reader — BuildOutput's chain is directly usable by the reader
    // with the metadata-derived (R, W, C). Confirms Phase 5D is the
    // intended hand-off from Phase 5C-2.
    #[test]
    fn rd_010_bridge_from_build_output() {
        let n = 32;
        let mut rng = ChaCha8Rng::seed_from_u64(17);
        let points: Vec<(f32, f32)> =
            (0..n).map(|_| (rng.gen::<f32>(), rng.gen::<f32>())).collect();
        let dist = |a: u32, b: u32| {
            let (ax, ay) = points[a as usize];
            let (bx, by) = points[b as usize];
            let dx = ax - bx;
            let dy = ay - by;
            dx * dx + dy * dy
        };
        let params = BuildParams {
            graph_degree_r: 8,
            build_list_size_l: 24,
            alpha: 1.2,
            dimensions: 64,
            search_subvector_count: 8,
            search_subvector_dim: 8,
            seed: 19,
            page_size: DEFAULT_PAGE_SIZE,
            has_binary_sidecar: true,
        };
        let payloads = synth_payloads(n, params.binary_word_count(), params.search_code_len());
        let out = build_and_persist_vamana(params, &payloads, dist).expect("build");

        let reader = PersistedGraphReader::new(
            &out.persisted.chain,
            out.metadata.graph_degree_r,
            params.binary_word_count(),
            params.search_code_len(),
        );
        // entry_point on the metadata page reads back a valid node.
        let entry_tuple = reader.read_node(out.metadata.entry_point).expect("entry");
        assert_ne!(entry_tuple.primary_heaptid, ItemPointer::INVALID);

        // Every node reachable from entry_point matches the count.
        let mut seen: HashSet<ItemPointer> = HashSet::new();
        let mut stack = vec![out.metadata.entry_point];
        seen.insert(out.metadata.entry_point);
        while let Some(tid) = stack.pop() {
            for nbr in reader.neighbors(tid).expect("nbrs") {
                if seen.insert(nbr) {
                    stack.push(nbr);
                }
            }
        }
        assert_eq!(seen.len(), n, "reader must reach every persisted node");
    }

    // RD-011: reusing a VisitedState across two calls yields the
    // same result as two independent calls. Scratch is cleared on
    // entry so stale membership from the first call doesn't leak.
    #[test]
    fn rd_011_visited_state_reuse_matches_fresh() {
        let n = 16;
        let g = chain_graph(n, 4);
        let payloads = synth_payloads(n, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 0, 0);
        let tid_to_node: std::collections::HashMap<ItemPointer, u32> = persisted
            .node_to_tid
            .iter()
            .enumerate()
            .map(|(i, &tid)| (tid, i as u32))
            .collect();

        let qd_a = |t: ItemPointer| tid_to_node[&t] as f32;
        let qd_b = |t: ItemPointer| n as f32 - tid_to_node[&t] as f32;

        let fresh_a =
            greedy_search_persisted(&reader, persisted.entry_point_tid, 4, qd_a).expect("fresh a");
        let fresh_b =
            greedy_search_persisted(&reader, persisted.entry_point_tid, 4, qd_b).expect("fresh b");

        let mut scratch = VisitedState::new();
        let reused_a = greedy_search_persisted_with(
            &reader,
            &mut scratch,
            persisted.entry_point_tid,
            4,
            qd_a,
        )
        .expect("reused a");
        let reused_b = greedy_search_persisted_with(
            &reader,
            &mut scratch,
            persisted.entry_point_tid,
            4,
            qd_b,
        )
        .expect("reused b");

        let tids = |r: &PersistedGreedyResult| -> Vec<ItemPointer> {
            r.frontier.iter().map(|c| c.tid).collect()
        };
        assert_eq!(tids(&fresh_a), tids(&reused_a), "first reuse must match fresh");
        assert_eq!(tids(&fresh_b), tids(&reused_b), "second reuse must match fresh (clear worked)");
    }

    // RD-012: VisitedState::clear + reserve are independently
    // testable — clear empties, reserve bumps capacity.
    #[test]
    fn rd_012_visited_state_clear_and_reserve() {
        let mut s = VisitedState::new();
        s.in_frontier.insert(ItemPointer {
            block_number: 1,
            offset_number: 1,
        });
        s.visited.insert(ItemPointer {
            block_number: 1,
            offset_number: 2,
        });
        assert_eq!(s.in_frontier.len(), 1);
        assert_eq!(s.visited.len(), 1);
        s.clear();
        assert!(s.in_frontier.is_empty());
        assert!(s.visited.is_empty());

        s.reserve(64);
        assert!(s.in_frontier.capacity() >= 64);
        assert!(s.visited.capacity() >= 64);
    }

    /// What to apply to each marked node in a persisted chain fixture.
    #[derive(Clone, Copy)]
    enum DeathKind {
        /// Vacuum pass 3: `deleted = true`, payload retained.
        Tombstone,
        /// Vacuum pass 1: `primary_heaptid = INVALID`, no overflow,
        /// `deleted` stays `false`. Used to exercise the live-tuple
        /// predicate (packets 11023/11027/11028).
        StripNoTombstone,
    }

    // Helper: persist N nodes, mutate the given nodes per their
    // `DeathKind`, and return the updated chain alongside the node→TID
    // map. Used by the live-TID iteration tests below.
    fn persisted_with_deaths(
        n: usize,
        max_degree: u16,
        deaths: &[(u32, DeathKind)],
    ) -> (crate::storage::page::DataPageChain, Vec<ItemPointer>) {
        use crate::am::ec_diskann::vacuum::{mark_deleted, strip_dead_primary_heaptid};
        let g = chain_graph(n, max_degree as usize);
        let payloads = synth_payloads(n, 0, 0);
        let mut persisted = persist_vamana_graph(
            &g,
            0,
            DEFAULT_PAGE_SIZE,
            &payloads,
            max_degree,
            0,
            0,
        )
        .expect("persist");
        for &(node_id, kind) in deaths {
            let tid = persisted.node_to_tid[node_id as usize];
            let page = persisted
                .chain
                .get_page_mut(tid.block_number)
                .expect("page");
            let bytes = page.raw_tuple(tid).expect("raw").to_vec();
            let mut tuple = VamanaNodeTuple::decode(&bytes, max_degree, 0, 0).expect("decode");
            match kind {
                DeathKind::Tombstone => mark_deleted(&mut tuple),
                DeathKind::StripNoTombstone => {
                    let stripped = strip_dead_primary_heaptid(&mut tuple, |_| true);
                    assert!(stripped, "strip primary heaptid");
                }
            }
            let patched = tuple.encode(max_degree, 0, 0).expect("encode");
            page.update_raw_tuple(tid, patched).expect("patch");
        }
        (persisted.chain, persisted.node_to_tid)
    }

    fn persisted_with_tombstones(
        n: usize,
        max_degree: u16,
        to_tombstone: &[u32],
    ) -> (crate::storage::page::DataPageChain, Vec<ItemPointer>) {
        let deaths: Vec<(u32, DeathKind)> = to_tombstone
            .iter()
            .map(|&id| (id, DeathKind::Tombstone))
            .collect();
        persisted_with_deaths(n, max_degree, &deaths)
    }

    // RD-013: iter_tids walks every TID in (block, offset) order and
    // count matches the persisted node count.
    #[test]
    fn rd_013_iter_tids_walks_every_slot_in_order() {
        let n = 20;
        let (chain, node_to_tid) = persisted_with_tombstones(n, 4, &[]);
        let reader = PersistedGraphReader::new(&chain, 4, 0, 0);

        let collected: Vec<ItemPointer> = reader.iter_tids().collect();
        assert_eq!(collected.len(), n, "one TID per node");

        for i in 1..collected.len() {
            let a = collected[i - 1];
            let b = collected[i];
            assert!(
                (a.block_number, a.offset_number) < (b.block_number, b.offset_number),
                "iter_tids must be strictly increasing in (block, offset): got {:?} then {:?}",
                a,
                b
            );
        }

        // Persisted TIDs form a subset of iter_tids output.
        let seen: HashSet<ItemPointer> = collected.into_iter().collect();
        for tid in &node_to_tid {
            assert!(seen.contains(tid));
        }
    }

    // RD-014: iter_live_tids skips tombstoned tuples and preserves
    // (block, offset) order on the survivors.
    #[test]
    fn rd_014_iter_live_tids_skips_tombstoned() {
        let n = 12;
        // Tombstone nodes 0, 3, 7 (arbitrary pattern spanning front,
        // middle, late).
        let dead = [0u32, 3, 7];
        let (chain, node_to_tid) = persisted_with_tombstones(n, 4, &dead);
        let reader = PersistedGraphReader::new(&chain, 4, 0, 0);

        let dead_tids: HashSet<ItemPointer> = dead
            .iter()
            .map(|&node| node_to_tid[node as usize])
            .collect();

        let live: Vec<ItemPointer> = reader
            .iter_live_tids()
            .map(|item| item.expect("decode ok"))
            .map(|(tid, tuple)| {
                assert!(!tuple.deleted, "live iterator yielded a deleted tuple");
                tid
            })
            .collect();

        assert_eq!(live.len(), n - dead.len(), "live count");
        for tid in &live {
            assert!(!dead_tids.contains(tid), "tombstoned TID leaked");
        }

        for i in 1..live.len() {
            assert!(
                (live[i - 1].block_number, live[i - 1].offset_number)
                    < (live[i].block_number, live[i].offset_number),
                "iter_live_tids must preserve order"
            );
        }
    }

    // RD-015: first_live_tid returns the lowest-block, lowest-offset
    // live TID, skipping over a tombstoned leading run.
    #[test]
    fn rd_015_first_live_tid_skips_leading_tombstones() {
        let n = 10;
        // Tombstone nodes 0 and 1 — the persisted layout places node 0
        // first in block order, so the first live TID must belong to
        // node 2 (or whichever node the persist sequencer put third).
        let (chain, node_to_tid) = persisted_with_tombstones(n, 4, &[0, 1]);
        let reader = PersistedGraphReader::new(&chain, 4, 0, 0);

        let got = reader.first_live_tid().expect("ok").expect("live exists");
        assert_ne!(got, node_to_tid[0]);
        assert_ne!(got, node_to_tid[1]);

        // And it must equal iter_live_tids().next().
        let via_iter = reader
            .iter_live_tids()
            .next()
            .expect("some")
            .expect("ok")
            .0;
        assert_eq!(got, via_iter);
    }

    // RD-016: first_live_tid returns None when every tuple is
    // tombstoned.
    #[test]
    fn rd_016_first_live_tid_none_when_all_dead() {
        let n = 6;
        let all: Vec<u32> = (0..n as u32).collect();
        let (chain, _) = persisted_with_tombstones(n, 4, &all);
        let reader = PersistedGraphReader::new(&chain, 4, 0, 0);

        let got = reader.first_live_tid().expect("ok");
        assert!(got.is_none(), "expected None, got {got:?}");
    }

    // RD-017: strip-without-tombstone regression (packets 11023/11027/
    // 11028). A tuple with `deleted = false` but `primary_heaptid ==
    // INVALID && !has_overflow_heaptids` — the transient state between
    // ADR-047 pass 1 (strip) and pass 3 (tombstone) — must NOT be
    // reported as live.
    #[test]
    fn rd_017_iter_live_tids_skips_stripped_without_tombstone() {
        let n = 8;
        // Node 2 stripped (pass 1, no tombstone yet); node 5
        // tombstoned; the rest alive. Both kinds must be skipped.
        let deaths = [
            (2u32, DeathKind::StripNoTombstone),
            (5u32, DeathKind::Tombstone),
        ];
        let (chain, node_to_tid) = persisted_with_deaths(n, 4, &deaths);
        let reader = PersistedGraphReader::new(&chain, 4, 0, 0);

        let live: Vec<ItemPointer> = reader
            .iter_live_tids()
            .map(|item| item.expect("ok").0)
            .collect();
        assert_eq!(live.len(), n - 2, "stripped and tombstoned excluded");
        assert!(!live.contains(&node_to_tid[2]), "stripped node leaked");
        assert!(!live.contains(&node_to_tid[5]), "tombstoned node leaked");

        // Sanity — the stripped tuple decodes with deleted=false but
        // is_live()=false.
        let bytes = chain
            .get_page(node_to_tid[2].block_number)
            .expect("page")
            .raw_tuple(node_to_tid[2])
            .expect("raw")
            .to_vec();
        let stripped_tuple = VamanaNodeTuple::decode(&bytes, 4, 0, 0).expect("decode");
        assert!(!stripped_tuple.deleted);
        assert_eq!(stripped_tuple.primary_heaptid, ItemPointer::INVALID);
        assert!(!stripped_tuple.is_live());
    }

    // RD-018: first_live_tid skips a leading stripped-without-tombstone
    // run. Regression for packets 11023/11027/11028 — even though
    // `deleted` is still false, a tuple with no primary heap TID is
    // not a valid entry point.
    #[test]
    fn rd_018_first_live_tid_skips_stripped_without_tombstone() {
        let n = 6;
        let deaths = [
            (0u32, DeathKind::StripNoTombstone),
            (1u32, DeathKind::StripNoTombstone),
        ];
        let (chain, node_to_tid) = persisted_with_deaths(n, 4, &deaths);
        let reader = PersistedGraphReader::new(&chain, 4, 0, 0);

        let got = reader.first_live_tid().expect("ok").expect("live exists");
        assert_ne!(got, node_to_tid[0]);
        assert_ne!(got, node_to_tid[1]);
    }

    // RD-019: overflow-only tuples ARE live — primary stripped but
    // `has_overflow_heaptids` carries heap TIDs. Complementary to
    // RD-017: confirms is_live() doesn't over-filter.
    #[test]
    fn rd_019_overflow_only_tuple_is_live() {
        // Synthesise a tuple directly (no helper yet for "strip primary
        // but keep overflow"); assert the predicate only, not persist.
        let mut t = VamanaNodeTuple::placeholder(4, 0, 0);
        t.primary_heaptid = ItemPointer::INVALID;
        t.has_overflow_heaptids = true;
        assert!(t.is_live(), "overflow chain keeps the tuple live");

        t.has_overflow_heaptids = false;
        assert!(!t.is_live(), "no primary, no overflow ⇒ not live");

        t.primary_heaptid = ItemPointer {
            block_number: 42,
            offset_number: 1,
        };
        assert!(t.is_live(), "primary TID restored ⇒ live");

        t.deleted = true;
        assert!(!t.is_live(), "tombstoned overrides heap TIDs");
    }
}
