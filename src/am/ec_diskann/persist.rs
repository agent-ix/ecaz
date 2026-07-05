//! Vamana graph persistence sequencer (ADR-045 Decisions 4 + 5).
//!
//! Phase 5C-1 landing: takes an in-memory [`VamanaGraph`] plus per-node
//! payloads (primary heap TID, optional binary sidecar, grouped-PQ4
//! search code) and writes them into a [`DataPageChain`] using the
//! placeholder-then-patch sequence from ADR-045 Decision 5.
//!
//! ## Persistence sequence
//!
//! 1. Compute the persistence order: BFS from the medoid (per ADR-045
//!    Decision 4 — "scan-traversal order from the entry point"), then
//!    append any unreached nodes in node-id order so every live node
//!    gets a TID. Disconnected components are not an error here; the
//!    build test (Phase 5C-3) asserts the reachable fraction.
//! 2. **Pass 1 — placeholders.** Walk the order. For each node, encode
//!    a fixed-length tuple with neighbor slots `INVALID` and
//!    `neighbor_count = 0`, insert via
//!    [`DataPageChain::insert_raw_tuple`], record the returned TID
//!    into a dense `Vec<ItemPointer>` keyed by node id.
//! 3. **Pass 2 — patch.** Walk the same order. For each node, re-encode
//!    with the resolved neighbor TIDs from the pass-1 map and replace
//!    the placeholder via [`DataPageChain::update_raw_tuple`]. Same
//!    encoded length is guaranteed by ADR-045 Decision 3 (the slim
//!    tuple's encoded length is a pure function of `(R, W, C)`).
//!
//! No pgrx dependency. The pgrx-side build-callback wiring (Phase
//! 5C-3) drives this module through a heap scan that produces
//! [`NodePayload`] entries, then ships the resulting page chain into a
//! GenericXLog transaction.

use crate::am::common::training::GroupedPq4Model;
use crate::am::ec_diskann::tuple::{VamanaCodebookTuple, VamanaNodeTuple};
use crate::am::ec_diskann::vamana::{bfs_reachable, VamanaGraph};
use crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS;
use crate::storage::page::{DataPageChain, ItemPointer};

/// Per-node fixed payload (primary heap TID + binary sidecar + search
/// code). The neighbor slots are not part of `NodePayload` — they are
/// derived from [`VamanaGraph`] at persistence time and resolved to
/// TIDs in pass 2.
#[derive(Debug, Clone, PartialEq)]
pub struct NodePayload {
    pub primary_heaptid: ItemPointer,
    /// Length must equal the index `binary_word_count` (W). Empty when
    /// `PAYLOAD_FLAG_BINARY_SIDECAR` is off.
    pub binary_words: Vec<u64>,
    /// Length must equal the index `search_code_len` (C).
    pub search_code: Vec<u8>,
}

/// Result of [`persist_vamana_graph`].
#[derive(Debug, Clone)]
pub struct PersistedGraph {
    pub chain: DataPageChain,
    /// Indexed by node id. `node_to_tid[i]` is the TID assigned to
    /// node `i`. Always populated for every node in `0..node_count`
    /// (guaranteed when `Ok` is returned).
    pub node_to_tid: Vec<ItemPointer>,
    /// TID of the medoid — what the metadata page records as
    /// `entry_point`.
    pub entry_point_tid: ItemPointer,
    /// Persistence order (BFS-reached prefix + unreached suffix).
    /// Useful for diagnostics; the metadata page does not store this.
    pub persistence_order: Vec<u32>,
    /// Nodes not reached by BFS from the medoid. Empty for a fully
    /// connected graph; non-empty cases are persisted but warrant
    /// logging at the build callback layer.
    pub unreached: Vec<u32>,
}

/// Drive the placeholder-then-patch persistence sequence over a built
/// [`VamanaGraph`].
///
/// `(graph_degree_r, binary_word_count, search_code_len)` is the
/// metadata-page constants triple. Every encoded tuple must use these
/// values — see ADR-045 Decision 1.
pub fn persist_vamana_graph(
    graph: &VamanaGraph,
    medoid: u32,
    page_size: usize,
    payloads: &[NodePayload],
    graph_degree_r: u16,
    binary_word_count: usize,
    search_code_len: usize,
) -> Result<PersistedGraph, String> {
    let n = graph.node_count();
    if n == 0 {
        return Err("cannot persist an empty graph".into());
    }
    if payloads.len() != n {
        return Err(format!(
            "payload count {} does not match graph node count {}",
            payloads.len(),
            n
        ));
    }
    if (medoid as usize) >= n {
        return Err(format!(
            "medoid node id {medoid} out of range (graph has {n} nodes)"
        ));
    }
    if graph.max_degree > graph_degree_r as usize {
        return Err(format!(
            "graph max_degree {} exceeds index graph_degree_r {graph_degree_r}",
            graph.max_degree
        ));
    }
    for (i, p) in payloads.iter().enumerate() {
        if p.binary_words.len() != binary_word_count {
            return Err(format!(
                "payload {i} binary_words length {} != index W {binary_word_count}",
                p.binary_words.len()
            ));
        }
        if p.search_code.len() != search_code_len {
            return Err(format!(
                "payload {i} search_code length {} != index C {search_code_len}",
                p.search_code.len()
            ));
        }
    }

    // Persistence order: BFS-from-medoid prefix + unreached suffix.
    let bfs = bfs_reachable(graph, medoid);
    let mut seen_in_bfs = vec![false; n];
    for &node in &bfs {
        seen_in_bfs[node as usize] = true;
    }
    let unreached: Vec<u32> = (0..n as u32)
        .filter(|node| !seen_in_bfs[*node as usize])
        .collect();
    let mut persistence_order = bfs;
    persistence_order.extend(unreached.iter().copied());
    debug_assert_eq!(
        persistence_order.len(),
        n,
        "persistence order must cover every node exactly once"
    );

    let mut chain = DataPageChain::new(page_size);
    let mut node_to_tid = vec![ItemPointer::INVALID; n];

    // Pass 1 — placeholders. Each node gets a TID; neighbors are
    // filled with INVALID for now.
    for &node in &persistence_order {
        let payload = &payloads[node as usize];
        let placeholder = VamanaNodeTuple {
            deleted: false,
            has_overflow_heaptids: false,
            primary_heaptid: payload.primary_heaptid,
            rerank_tid: ItemPointer::INVALID,
            binary_words: payload.binary_words.clone(),
            search_code: payload.search_code.clone(),
            neighbors: vec![ItemPointer::INVALID; graph_degree_r as usize],
            neighbor_count: 0,
        };
        let encoded = placeholder.encode(graph_degree_r, binary_word_count, search_code_len)?;
        let tid = chain.insert_raw_tuple(encoded)?;
        node_to_tid[node as usize] = tid;
    }

    // Pass 2 — patch. Re-encode with resolved neighbor TIDs and
    // replace via update_raw_tuple. Same encoded length per ADR-045
    // Decision 3; update_raw_tuple's same-length invariant holds
    // trivially.
    for &node in &persistence_order {
        let payload = &payloads[node as usize];
        let neighbor_ids = &graph.neighbors[node as usize];
        let neighbor_count = u16::try_from(neighbor_ids.len())
            .map_err(|_| format!("neighbor count overflow for node {node}"))?;

        let mut neighbor_slots = vec![ItemPointer::INVALID; graph_degree_r as usize];
        for (slot, &nbr_id) in neighbor_ids.iter().enumerate() {
            let tid = node_to_tid[nbr_id as usize];
            if tid == ItemPointer::INVALID {
                return Err(format!(
                    "node {node} references neighbor {nbr_id} with no assigned TID"
                ));
            }
            neighbor_slots[slot] = tid;
        }

        let patched = VamanaNodeTuple {
            deleted: false,
            has_overflow_heaptids: false,
            primary_heaptid: payload.primary_heaptid,
            rerank_tid: ItemPointer::INVALID,
            binary_words: payload.binary_words.clone(),
            search_code: payload.search_code.clone(),
            neighbors: neighbor_slots,
            neighbor_count,
        };
        let encoded = patched.encode(graph_degree_r, binary_word_count, search_code_len)?;
        let tid = node_to_tid[node as usize];
        let page = chain
            .get_page_mut(tid.block_number)
            .ok_or_else(|| format!("page {} missing during patch", tid.block_number))?;
        page.update_raw_tuple(tid, encoded)?;
    }

    let entry_point_tid = node_to_tid[medoid as usize];
    debug_assert_ne!(entry_point_tid, ItemPointer::INVALID);

    Ok(PersistedGraph {
        chain,
        node_to_tid,
        entry_point_tid,
        persistence_order,
        unreached,
    })
}

/// Stage a grouped-PQ4 codebook as a `nexttid`-linked chain of
/// [`VamanaCodebookTuple`]s in `chain` and return the head TID.
///
/// One tuple per group is appended in group-index order. Each shard's
/// `nexttid` points to the next group's shard; the last shard carries
/// `ItemPointer::INVALID` as its terminator. Uses the same
/// placeholder-then-patch technique as node persistence: insert every
/// shard with `nexttid = INVALID` first (so we know every TID), then
/// re-encode each non-terminal shard with its successor's TID via
/// [`crate::storage::page::DataPage::update_raw_tuple`]. Encoded length
/// is a pure function of `centroid_count`, so the update always fits.
pub fn stage_grouped_codebook_chain(
    chain: &mut DataPageChain,
    model: &GroupedPq4Model,
) -> Result<ItemPointer, String> {
    if model.group_count == 0 {
        return Err("grouped codebook staging requires at least one group".into());
    }
    if model.codebooks.len() != model.group_count {
        return Err(format!(
            "model codebook count {} does not match group_count {}",
            model.codebooks.len(),
            model.group_count
        ));
    }
    let centroid_count = model.group_size * GROUPED_PQ_CENTROIDS;
    for (group_index, codebook) in model.codebooks.iter().enumerate() {
        if codebook.len() != centroid_count {
            return Err(format!(
                "grouped codebook {} length mismatch: got {}, expected {}",
                group_index,
                codebook.len(),
                centroid_count
            ));
        }
    }

    let mut shard_tids = Vec::with_capacity(model.group_count);
    for (group_index, codebook) in model.codebooks.iter().enumerate() {
        let group_index_u16 = u16::try_from(group_index)
            .map_err(|_| format!("grouped codebook index {group_index} does not fit in u16"))?;
        let placeholder = VamanaCodebookTuple {
            group_index: group_index_u16,
            nexttid: ItemPointer::INVALID,
            centroids: codebook.clone(),
        };
        let tid = chain.insert_raw_tuple(placeholder.encode())?;
        shard_tids.push(tid);
    }

    for i in 0..shard_tids.len().saturating_sub(1) {
        let patched = VamanaCodebookTuple {
            group_index: u16::try_from(i).expect("validated group index fits in u16"),
            nexttid: shard_tids[i + 1],
            centroids: model.codebooks[i].clone(),
        };
        let tid = shard_tids[i];
        let page = chain
            .get_page_mut(tid.block_number)
            .ok_or_else(|| format!("page {} missing during codebook patch", tid.block_number))?;
        page.update_raw_tuple(tid, patched.encode())?;
    }

    Ok(shard_tids[0])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::ec_diskann::tuple::VamanaNodeTuple;
    use crate::am::ec_diskann::vamana::{approximate_medoid, build_vamana_graph};
    use crate::storage::page::{DEFAULT_PAGE_SIZE, FIRST_DATA_BLOCK_NUMBER};

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

    fn make_chain_graph(n: usize) -> VamanaGraph {
        // Linear chain 0—1—2—…—(n-1) so BFS from 0 visits 0,1,2,…,(n-1).
        let mut g = VamanaGraph::empty(n, 4);
        for i in 0..n - 1 {
            g.neighbors[i].push((i + 1) as u32);
            g.neighbors[i + 1].push(i as u32);
        }
        g
    }

    // PE-001: empty graph errors cleanly.
    #[test]
    fn pe_001_empty_graph_errors() {
        let g = VamanaGraph::empty(0, 4);
        let err = persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &[], 4, 0, 0)
            .expect_err("empty should fail");
        assert!(err.contains("empty graph"), "got: {err}");
    }

    // PE-002: payload count mismatch errors.
    #[test]
    fn pe_002_payload_count_mismatch_errors() {
        let g = make_chain_graph(5);
        let payloads = synth_payloads(4, 0, 0);
        let err = persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0)
            .expect_err("count mismatch should fail");
        assert!(err.contains("payload count"), "got: {err}");
    }

    // PE-003: medoid out of range errors.
    #[test]
    fn pe_003_medoid_out_of_range_errors() {
        let g = make_chain_graph(3);
        let payloads = synth_payloads(3, 0, 0);
        let err = persist_vamana_graph(&g, 99, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0)
            .expect_err("medoid out of range should fail");
        assert!(err.contains("medoid"), "got: {err}");
    }

    // PE-004: payload binary_words length mismatch errors.
    #[test]
    fn pe_004_payload_body_size_mismatch_errors() {
        let g = make_chain_graph(2);
        let mut payloads = synth_payloads(2, 4, 8);
        payloads[0].binary_words.pop();
        let err = persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 4, 8)
            .expect_err("body size mismatch should fail");
        assert!(err.contains("binary_words"), "got: {err}");
    }

    // PE-005: graph max_degree > index R errors.
    #[test]
    fn pe_005_max_degree_exceeds_r_errors() {
        let mut g = VamanaGraph::empty(2, 8); // max_degree = 8
        g.neighbors[0].push(1);
        g.neighbors[1].push(0);
        let payloads = synth_payloads(2, 0, 0);
        let err = persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0)
            .expect_err("max_degree > R should fail");
        assert!(err.contains("max_degree"), "got: {err}");
    }

    // PE-006: single-node graph persists; entry point resolves to its tid.
    #[test]
    fn pe_006_single_node_persists() {
        let g = VamanaGraph::empty(1, 4);
        let payloads = synth_payloads(1, 2, 8);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 2, 8).expect("persist");
        assert_eq!(persisted.node_to_tid.len(), 1);
        assert_eq!(persisted.entry_point_tid, persisted.node_to_tid[0]);
        assert_eq!(persisted.persistence_order, vec![0]);
        assert!(persisted.unreached.is_empty());
        // Verify we can decode the tuple back from the chain.
        let page = persisted
            .chain
            .get_page(persisted.entry_point_tid.block_number)
            .expect("page exists");
        let raw = page.raw_tuple(persisted.entry_point_tid).expect("raw");
        let decoded = VamanaNodeTuple::decode(raw, 4, 2, 8).expect("decode");
        assert_eq!(decoded.primary_heaptid, payloads[0].primary_heaptid);
        assert_eq!(decoded.neighbor_count, 0);
    }

    // PE-007: connected chain → BFS order equals 0..n; every node
    // tuple round-trips with neighbor TIDs resolved.
    #[test]
    fn pe_007_connected_chain_round_trip() {
        let n = 10;
        let g = make_chain_graph(n);
        let payloads = synth_payloads(n, 1, 4);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 1, 4).expect("persist");

        assert_eq!(
            persisted.persistence_order,
            (0..n as u32).collect::<Vec<_>>()
        );
        assert!(persisted.unreached.is_empty());

        // Every node decodes; neighbor slots resolve to other nodes' TIDs.
        for node_id in 0..n as u32 {
            let tid = persisted.node_to_tid[node_id as usize];
            assert_ne!(tid, ItemPointer::INVALID);
            let page = persisted.chain.get_page(tid.block_number).expect("page");
            let raw = page.raw_tuple(tid).expect("raw");
            let decoded = VamanaNodeTuple::decode(raw, 4, 1, 4).expect("decode");
            assert_eq!(
                decoded.primary_heaptid,
                payloads[node_id as usize].primary_heaptid
            );
            assert_eq!(
                decoded.neighbor_count as usize,
                g.neighbors[node_id as usize].len()
            );
            for (slot, &nbr_id) in g.neighbors[node_id as usize].iter().enumerate() {
                assert_eq!(
                    decoded.neighbors[slot],
                    persisted.node_to_tid[nbr_id as usize]
                );
            }
            // Tail slots remain INVALID.
            for slot in &decoded.neighbors[g.neighbors[node_id as usize].len()..] {
                assert_eq!(*slot, ItemPointer::INVALID);
            }
        }
    }

    // PE-008: disconnected graph → unreached nodes still get TIDs and
    // appear in `unreached`. Persistence order is BFS prefix + node-id
    // suffix.
    #[test]
    fn pe_008_disconnected_graph_persists_unreached() {
        // Two components: {0,1} and {2,3}. Medoid is 0.
        let mut g = VamanaGraph::empty(4, 4);
        g.neighbors[0].push(1);
        g.neighbors[1].push(0);
        g.neighbors[2].push(3);
        g.neighbors[3].push(2);
        let payloads = synth_payloads(4, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0).expect("persist");

        // BFS from 0 reaches {0, 1}; unreached are {2, 3} in node-id order.
        assert_eq!(&persisted.persistence_order[..2], &[0u32, 1u32]);
        assert!(persisted.persistence_order[2..].contains(&2));
        assert!(persisted.persistence_order[2..].contains(&3));
        assert_eq!(persisted.unreached.len(), 2);

        // Every node has a TID.
        for tid in &persisted.node_to_tid {
            assert_ne!(*tid, ItemPointer::INVALID);
        }
    }

    // PE-009: graph forces multi-page chain → tuples spill into a
    // second block; BFS-from-medoid order means medoid lands on the
    // first block.
    #[test]
    fn pe_009_multi_page_chain() {
        // Each tuple at R=32, W=24, C=192 ≈ 464 bytes; 8KB / 464 ≈
        // 17 tuples per page. Build a 40-node connected ring so we
        // overflow to ≥ 3 pages.
        let n = 40;
        let mut g = VamanaGraph::empty(n, 32);
        for i in 0..n {
            g.neighbors[i].push(((i + 1) % n) as u32);
            g.neighbors[i].push(((i + n - 1) % n) as u32);
        }
        let payloads = synth_payloads(n, 24, 192);
        let persisted = persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 32, 24, 192)
            .expect("persist");
        assert!(
            persisted.chain.pages().len() >= 2,
            "expected multi-page chain, got {} pages",
            persisted.chain.pages().len()
        );
        assert_eq!(
            persisted.entry_point_tid.block_number, FIRST_DATA_BLOCK_NUMBER,
            "medoid should land on the first data block (BFS prefix)"
        );
    }

    // PE-010: ADR-045 Decision 5 — placeholder and patched encoded
    // lengths are identical. Verified via update_raw_tuple succeeding;
    // here we cross-check by reading the on-page tuple and confirming
    // the patched neighbor_count matches the graph.
    #[test]
    fn pe_010_placeholder_patched_lengths_align() {
        let n = 5;
        let g = make_chain_graph(n);
        let payloads = synth_payloads(n, 2, 16);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 8, 2, 16).expect("persist");
        for node_id in 0..n as u32 {
            let tid = persisted.node_to_tid[node_id as usize];
            let page = persisted.chain.get_page(tid.block_number).expect("page");
            let raw = page.raw_tuple(tid).expect("raw");
            // After patch, the read-back tuple's neighbor_count must
            // match the original graph (i.e., not the placeholder's 0).
            let decoded = VamanaNodeTuple::decode(raw, 8, 2, 16).expect("decode");
            assert_eq!(
                decoded.neighbor_count as usize,
                g.neighbors[node_id as usize].len(),
                "patched tuple should carry the resolved neighbor count, not placeholder 0"
            );
        }
    }

    // PE-011: end-to-end with a real Vamana build (synthetic 2D L2)
    // — the persisted graph's medoid TID is valid, every neighbor
    // TID resolves to a live tuple, and the round-trip preserves the
    // adjacency.
    #[test]
    fn pe_011_end_to_end_with_built_graph() {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;

        let n = 50;
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let points: Vec<(f32, f32)> = (0..n)
            .map(|_| (rng.gen::<f32>(), rng.gen::<f32>()))
            .collect();
        let dist = |a: u32, b: u32| {
            let (ax, ay) = points[a as usize];
            let (bx, by) = points[b as usize];
            let dx = ax - bx;
            let dy = ay - by;
            dx * dx + dy * dy
        };
        let medoid = approximate_medoid(n, n, 7, dist);
        let graph = build_vamana_graph(n, medoid, 8, 32, 1.2, 11, dist);

        let payloads = synth_payloads(n, 1, 4);
        let persisted = persist_vamana_graph(&graph, medoid, DEFAULT_PAGE_SIZE, &payloads, 8, 1, 4)
            .expect("persist");

        assert_ne!(persisted.entry_point_tid, ItemPointer::INVALID);
        // For each node, the on-page tuple's neighbor TIDs must match
        // the in-memory graph.
        for node_id in 0..n as u32 {
            let tid = persisted.node_to_tid[node_id as usize];
            let page = persisted.chain.get_page(tid.block_number).expect("page");
            let raw = page.raw_tuple(tid).expect("raw");
            let decoded = VamanaNodeTuple::decode(raw, 8, 1, 4).expect("decode");
            for (slot, &nbr_id) in graph.neighbors[node_id as usize].iter().enumerate() {
                assert_eq!(
                    decoded.neighbors[slot],
                    persisted.node_to_tid[nbr_id as usize]
                );
            }
        }
    }

    fn synth_grouped_pq4_model(group_count: usize, group_size: usize) -> GroupedPq4Model {
        let centroid_count = group_size * GROUPED_PQ_CENTROIDS;
        let codebooks = (0..group_count)
            .map(|g| {
                (0..centroid_count)
                    .map(|i| (g * 1000 + i) as f32 * 0.125)
                    .collect()
            })
            .collect();
        GroupedPq4Model {
            codebooks,
            group_count,
            group_size,
            transform_dim: group_size * group_count,
            signs: vec![1.0; group_size * group_count],
        }
    }

    // CB-001: empty model errors cleanly.
    #[test]
    fn cb_001_empty_model_errors() {
        let model = GroupedPq4Model {
            codebooks: vec![],
            group_count: 0,
            group_size: 4,
            transform_dim: 0,
            signs: vec![],
        };
        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let err = stage_grouped_codebook_chain(&mut chain, &model).expect_err("empty should fail");
        assert!(err.contains("at least one group"), "got: {err}");
    }

    // CB-002: centroid-count mismatch errors and names the bad group.
    #[test]
    fn cb_002_centroid_count_mismatch_errors() {
        let mut model = synth_grouped_pq4_model(2, 4);
        model.codebooks[1].pop();
        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let err =
            stage_grouped_codebook_chain(&mut chain, &model).expect_err("mismatch should fail");
        assert!(err.contains("length mismatch"), "got: {err}");
    }

    // CB-003: single-group model stages one shard with nexttid = INVALID.
    #[test]
    fn cb_003_single_group_terminates_chain() {
        let model = synth_grouped_pq4_model(1, 4);
        let centroid_count = model.group_size * GROUPED_PQ_CENTROIDS;
        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let head = stage_grouped_codebook_chain(&mut chain, &model).expect("stage");
        let page = chain.get_page(head.block_number).expect("head page");
        let raw = page.raw_tuple(head).expect("raw");
        let decoded = VamanaCodebookTuple::decode(raw, centroid_count).expect("decode");
        assert_eq!(decoded.group_index, 0);
        assert_eq!(decoded.nexttid, ItemPointer::INVALID);
        assert_eq!(decoded.centroids, model.codebooks[0]);
    }

    // CB-004: multi-group chain — traversing nexttid reaches every shard
    // in group-index order; the last shard's nexttid is INVALID.
    #[test]
    fn cb_004_multi_group_chain_links_all_shards() {
        let group_count = 4;
        let group_size = 8;
        let centroid_count = group_size * GROUPED_PQ_CENTROIDS;
        let model = synth_grouped_pq4_model(group_count, group_size);
        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let head = stage_grouped_codebook_chain(&mut chain, &model).expect("stage");

        let mut cursor = head;
        for expected_group in 0..group_count {
            assert_ne!(cursor, ItemPointer::INVALID);
            let page = chain.get_page(cursor.block_number).expect("page");
            let raw = page.raw_tuple(cursor).expect("raw");
            let decoded = VamanaCodebookTuple::decode(raw, centroid_count).expect("decode");
            assert_eq!(decoded.group_index as usize, expected_group);
            assert_eq!(decoded.centroids, model.codebooks[expected_group]);
            cursor = decoded.nexttid;
        }
        assert_eq!(cursor, ItemPointer::INVALID, "chain must terminate");
    }
}
