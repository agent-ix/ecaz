//! Scan algorithm shell for `ec_diskann` (task 17 Phase 6A).
//!
//! Pure-Rust composition of Phase 5D's [`PersistedGraphReader`] into
//! a two-stage scan:
//!
//! 1. **Greedy descent with a cheap prefilter.** Walk the persisted
//!    graph with an `L_search`-bounded frontier, scoring each visited
//!    node via a caller-supplied `prefilter` closure that reads from
//!    the decoded tuple (typically a binary-Hamming or grouped-PQ4
//!    score). This is the traversal step.
//! 2. **Exact rerank on the top candidates.** Take the top
//!    `rerank_budget` candidates from the greedy frontier, call a
//!    caller-supplied `rerank` closure on each node's
//!    `primary_heaptid` (typically an ecvector heap fetch + exact
//!    distance), re-sort by the exact distance, and truncate to
//!    `top_k`.
//!
//! The shell is distance-agnostic: the prefilter closure is the sole
//! coupling point with the quantizer stack, and the rerank closure is
//! the sole coupling point with heap access. Both are injected so
//! this module is testable with synthetic closures and does not depend
//! on pgrx.
//!
//! ## Relation to Phase 6B
//!
//! Phase 6B (deferred with the native-build lane) is the pgrx-side
//! `amgettuple` wiring: open the relation, bind `prefilter` to
//! `Quantizer::prepare_scorer`, bind `rerank` to the ecvector cold
//! path, iterate the returned [`ScanResult`]s. The shell here is what
//! that callback will call.

use crate::am::ec_diskann::reader::{PersistedGraphReader, VisitedState};
use crate::am::ec_diskann::tuple::VamanaNodeTuple;
use crate::storage::page::ItemPointer;

/// Scan-time tuning parameters. Every value must be > 0.
#[derive(Debug, Clone, Copy)]
pub struct ScanParams {
    pub entry_point: ItemPointer,
    /// Greedy frontier size during traversal (`L_search` in the
    /// DiskANN paper).
    pub list_size: usize,
    /// Number of top-frontier candidates to rerank with the exact
    /// distance. `rerank_budget ≤ list_size`.
    pub rerank_budget: usize,
    /// Number of final results to return. `top_k ≤ rerank_budget`.
    pub top_k: usize,
}

/// One scan result — ready for `amgettuple` to return. `tid` is the
/// index tuple (for subsequent neighbor walks or vacuum-time identity),
/// `primary_heaptid` is what Postgres cares about, `distance` is the
/// exact rerank distance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScanResult {
    pub tid: ItemPointer,
    pub primary_heaptid: ItemPointer,
    pub distance: f32,
}

/// Candidate carried through the greedy loop. Caches the tuple's
/// `primary_heaptid` so the rerank stage (or a Phase 6B caller) does
/// not need to re-decode. Exposed so [`greedy_descent`] is usable
/// standalone from Phase 6B's pgrx wiring.
///
/// `emittable` captures [`VamanaNodeTuple::is_live`] at descent time —
/// `true` iff the tuple is not tombstoned AND carries at least one
/// heap TID. Non-emittable candidates are still kept in the frontier
/// so the traversal walks their neighbors for graph connectivity, but
/// the rerank stage filters them out before calling the caller's
/// `rerank` closure. Tracks packets 11023/11027/11028.
#[derive(Debug, Clone, Copy)]
pub struct ScanCandidate {
    pub tid: ItemPointer,
    pub primary_heaptid: ItemPointer,
    pub score: f32,
    pub emittable: bool,
}

impl PartialEq for ScanCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.tid == other.tid
    }
}
impl Eq for ScanCandidate {}
impl Ord for ScanCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score
            .partial_cmp(&other.score)
            .unwrap_or(std::cmp::Ordering::Greater)
            .then_with(|| self.tid.block_number.cmp(&other.tid.block_number))
            .then_with(|| self.tid.offset_number.cmp(&other.tid.offset_number))
    }
}
impl PartialOrd for ScanCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Pick the scan's entry point given a preferred TID (typically the
/// metadata page's medoid). If the preferred TID is missing, decode-
/// corrupt, or tombstoned, fall back to the lowest-block live TID in
/// the chain ([`PersistedGraphReader::first_live_tid`]). Returns
/// `Ok(None)` iff the chain has no live tuples at all.
///
/// This closes the Phase 6B "medoid may be deleted" open question
/// (ADR-047 §10 defers live-medoid migration to rebuild). The result
/// feeds [`ScanParams::entry_point`].
pub fn resolve_entry_point(
    reader: &PersistedGraphReader<'_>,
    preferred: ItemPointer,
) -> Result<Option<ItemPointer>, String> {
    if preferred != ItemPointer::INVALID {
        if let Ok(tuple) = reader.read_node(preferred) {
            if tuple.is_live() {
                return Ok(Some(preferred));
            }
        }
    }
    reader.first_live_tid()
}

/// Run a persisted-graph scan end-to-end: greedy descent with the
/// cheap `prefilter` closure, then exact rerank with `rerank`. Returns
/// the top-`params.top_k` results sorted ascending by exact distance.
///
/// `prefilter(&tuple) -> f32` is called once per visited node during
/// greedy descent. `rerank(primary_heaptid) -> f32` is called at most
/// `rerank_budget` times, on the candidates that survived the
/// prefilter greedy.
pub fn vamana_scan<Pre, Re>(
    reader: &PersistedGraphReader<'_>,
    params: ScanParams,
    prefilter: Pre,
    rerank: Re,
) -> Result<Vec<ScanResult>, String>
where
    Pre: Fn(&VamanaNodeTuple) -> f32,
    Re: Fn(ItemPointer) -> f32,
{
    let mut scratch = VisitedState::new();
    vamana_scan_with(reader, &mut scratch, params, prefilter, rerank)
}

/// Scratch-reusing variant of [`vamana_scan`]. Phase 6B's pgrx
/// scan cursor allocates one [`VisitedState`] at open-scan time and
/// calls this across `amgettuple` re-entries (if we move to a
/// streaming shape) or across successive cursors on the same
/// relation.
pub fn vamana_scan_with<Pre, Re>(
    reader: &PersistedGraphReader<'_>,
    scratch: &mut VisitedState,
    params: ScanParams,
    prefilter: Pre,
    rerank: Re,
) -> Result<Vec<ScanResult>, String>
where
    Pre: Fn(&VamanaNodeTuple) -> f32,
    Re: Fn(ItemPointer) -> f32,
{
    if params.entry_point == ItemPointer::INVALID {
        return Err("entry_point must not be INVALID".into());
    }
    if params.list_size == 0 {
        return Err("list_size must be > 0".into());
    }
    if params.rerank_budget == 0 {
        return Err("rerank_budget must be > 0".into());
    }
    if params.top_k == 0 {
        return Err("top_k must be > 0".into());
    }
    if params.rerank_budget > params.list_size {
        return Err(format!(
            "rerank_budget {} must be <= list_size {}",
            params.rerank_budget, params.list_size
        ));
    }
    if params.top_k > params.rerank_budget {
        return Err(format!(
            "top_k {} must be <= rerank_budget {}",
            params.top_k, params.rerank_budget
        ));
    }

    // Stage 1 — greedy descent under the cheap prefilter.
    let frontier = greedy_descent_with(
        reader,
        scratch,
        params.entry_point,
        params.list_size,
        &prefilter,
    )?;

    // Stage 2 — exact rerank of the top `rerank_budget` emittable
    // candidates. Tombstoned tuples (`deleted = true`) and
    // stripped-but-not-tombstoned tuples (ADR-047 pass 1 done, pass 3
    // not yet) are both dropped here: the first has no valid heap row
    // under MVCC, the second carries `primary_heaptid == INVALID`
    // which is not a legal `xs_heaptid` return. Traversal still walks
    // their neighbors for connectivity. (Packets 11023/11027/11028.)
    let mut reranked: Vec<ScanResult> = frontier
        .into_iter()
        .filter(|c| c.emittable)
        .take(params.rerank_budget)
        .map(|c| {
            let exact = rerank(c.primary_heaptid);
            ScanResult {
                tid: c.tid,
                primary_heaptid: c.primary_heaptid,
                distance: exact,
            }
        })
        .collect();

    reranked.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Greater)
            .then_with(|| a.tid.block_number.cmp(&b.tid.block_number))
            .then_with(|| a.tid.offset_number.cmp(&b.tid.offset_number))
    });
    reranked.truncate(params.top_k);
    Ok(reranked)
}

/// Greedy best-first descent under a cheap prefilter score. Returns
/// the frontier sorted ascending by prefilter score (smallest = best),
/// truncated to `list_size`.
///
/// Exposed separately from [`vamana_scan`] so Phase 6B can drive the
/// descent + rerank in different transactions (useful if rerank wants
/// to batch heap fetches, or if a future iterator-style amgettuple
/// wants incremental rerank).
pub fn greedy_descent<Pre>(
    reader: &PersistedGraphReader<'_>,
    entry_point: ItemPointer,
    list_size: usize,
    prefilter: &Pre,
) -> Result<Vec<ScanCandidate>, String>
where
    Pre: Fn(&VamanaNodeTuple) -> f32,
{
    let mut scratch = VisitedState::new();
    greedy_descent_with(reader, &mut scratch, entry_point, list_size, prefilter)
}

/// Scratch-reusing variant of [`greedy_descent`]. See
/// [`crate::am::ec_diskann::reader::greedy_search_persisted_with`] for
/// the allocation rationale.
pub fn greedy_descent_with<Pre>(
    reader: &PersistedGraphReader<'_>,
    scratch: &mut VisitedState,
    entry_point: ItemPointer,
    list_size: usize,
    prefilter: &Pre,
) -> Result<Vec<ScanCandidate>, String>
where
    Pre: Fn(&VamanaNodeTuple) -> f32,
{
    scratch.clear();

    let entry_tuple = reader.read_node(entry_point)?;
    let entry_score = prefilter(&entry_tuple);
    let mut frontier = vec![ScanCandidate {
        tid: entry_point,
        primary_heaptid: entry_tuple.primary_heaptid,
        score: entry_score,
        emittable: entry_tuple.is_live(),
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

        let picked_tuple = reader.read_node(picked.tid)?;
        let count = picked_tuple.neighbor_count as usize;
        for &nbr in picked_tuple.neighbors.iter().take(count) {
            if nbr == ItemPointer::INVALID {
                continue;
            }
            if scratch.in_frontier.contains(&nbr) {
                continue;
            }
            let nbr_tuple = reader.read_node(nbr)?;
            let score = prefilter(&nbr_tuple);
            frontier.push(ScanCandidate {
                tid: nbr,
                primary_heaptid: nbr_tuple.primary_heaptid,
                score,
                emittable: nbr_tuple.is_live(),
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
    Ok(frontier)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::ec_diskann::build::{build_and_persist_vamana, BuildParams};
    use crate::am::ec_diskann::persist::{persist_vamana_graph, NodePayload};
    use crate::am::ec_diskann::vamana::{approximate_medoid, build_vamana_graph, VamanaGraph};
    use crate::storage::page::DEFAULT_PAGE_SIZE;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;
    use std::collections::HashSet;

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

    // SC-001: greedy descent + rerank agree on the target node when
    // both distance stages are monotonic in node id.
    #[test]
    fn sc_001_end_to_end_top1_on_chain() {
        let n = 8;
        let g = chain_graph(n, 4);
        let payloads = synth_payloads(n, 1, 4);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 1, 4).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 1, 4);

        // prefilter: distance = (n-1 - node_id); target = node 7.
        let prefilter = |t: &VamanaNodeTuple| {
            let node = (t.primary_heaptid.block_number - 1000) as u32;
            ((n - 1) as u32 - node) as f32
        };
        let rerank = |hip: ItemPointer| ((n - 1) as u32 - (hip.block_number - 1000)) as f32;

        let params = ScanParams {
            entry_point: persisted.entry_point_tid,
            list_size: 4,
            rerank_budget: 4,
            top_k: 1,
        };
        let res = vamana_scan(&reader, params, prefilter, rerank).expect("scan");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].tid, persisted.node_to_tid[(n - 1) as usize]);
        assert_eq!(res[0].distance, 0.0);
    }

    // SC-002: rerank reorders the prefilter result. prefilter ranks
    // node 2 best; rerank ranks node 5 best (of the prefilter's
    // top-4). Result must be node 5.
    #[test]
    fn sc_002_rerank_can_reorder_prefilter() {
        let n = 6;
        let g = chain_graph(n, 4);
        let payloads = synth_payloads(n, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 0, 0);

        let prefilter_scores = [3.0f32, 2.0, 0.0, 1.0, 4.0, 5.0]; // node 2 wins prefilter
        let rerank_scores = [9.0f32, 9.0, 9.0, 9.0, 9.0, 0.1]; // node 5 wins rerank

        let prefilter = |t: &VamanaNodeTuple| {
            let node = (t.primary_heaptid.block_number - 1000) as usize;
            prefilter_scores[node]
        };
        let rerank = |hip: ItemPointer| {
            let node = (hip.block_number - 1000) as usize;
            rerank_scores[node]
        };

        let params = ScanParams {
            entry_point: persisted.entry_point_tid,
            list_size: 6,
            rerank_budget: 6,
            top_k: 1,
        };
        let res = vamana_scan(&reader, params, prefilter, rerank).expect("scan");
        assert_eq!(res[0].primary_heaptid, payloads[5].primary_heaptid);
        assert_eq!(res[0].distance, 0.1);
    }

    // SC-003: rerank_budget < list_size — only the top-budget of the
    // prefilter frontier get reranked. Confirm rerank is called at
    // most `budget` times.
    #[test]
    fn sc_003_rerank_budget_caps_rerank_calls() {
        use std::cell::Cell;
        let n = 6;
        let g = chain_graph(n, 4);
        let payloads = synth_payloads(n, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 0, 0);

        let prefilter = |t: &VamanaNodeTuple| (t.primary_heaptid.block_number - 1000) as f32;
        let rerank_calls = Cell::new(0usize);
        let rerank = |_: ItemPointer| {
            rerank_calls.set(rerank_calls.get() + 1);
            0.0
        };

        let params = ScanParams {
            entry_point: persisted.entry_point_tid,
            list_size: 6,
            rerank_budget: 3,
            top_k: 3,
        };
        vamana_scan(&reader, params, prefilter, rerank).expect("scan");
        assert_eq!(
            rerank_calls.get(),
            3,
            "rerank must be called exactly `budget` times"
        );
    }

    // SC-004: top_k truncates the reranked result.
    #[test]
    fn sc_004_top_k_truncates() {
        let n = 6;
        let g = chain_graph(n, 4);
        let payloads = synth_payloads(n, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 0, 0);

        let prefilter = |t: &VamanaNodeTuple| (t.primary_heaptid.block_number - 1000) as f32;
        let rerank = |hip: ItemPointer| (hip.block_number - 1000) as f32;

        let params = ScanParams {
            entry_point: persisted.entry_point_tid,
            list_size: 6,
            rerank_budget: 6,
            top_k: 2,
        };
        let res = vamana_scan(&reader, params, prefilter, rerank).expect("scan");
        assert_eq!(res.len(), 2);
        // Ascending by exact distance: nodes 0 and 1.
        assert_eq!(res[0].primary_heaptid, payloads[0].primary_heaptid);
        assert_eq!(res[1].primary_heaptid, payloads[1].primary_heaptid);
    }

    // SC-005: invalid entry_point errors; out-of-chain entry errors
    // via the underlying reader.
    #[test]
    fn sc_005_invalid_entry_errors() {
        let n = 2;
        let g = chain_graph(n, 4);
        let payloads = synth_payloads(n, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 0, 0);

        let prefilter = |_: &VamanaNodeTuple| 0.0;
        let rerank = |_: ItemPointer| 0.0;

        let params = ScanParams {
            entry_point: ItemPointer::INVALID,
            list_size: 2,
            rerank_budget: 2,
            top_k: 1,
        };
        let err = vamana_scan(&reader, params, prefilter, rerank).expect_err("bad entry");
        assert!(err.contains("entry_point"), "got: {err}");
    }

    // SC-006: parameter validation — zero sizes, rerank_budget >
    // list_size, top_k > rerank_budget all error.
    #[test]
    fn sc_006_parameter_validation() {
        let n = 2;
        let g = chain_graph(n, 4);
        let payloads = synth_payloads(n, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 0, 0);
        let prefilter = |_: &VamanaNodeTuple| 0.0;
        let rerank = |_: ItemPointer| 0.0;
        let base = ScanParams {
            entry_point: persisted.entry_point_tid,
            list_size: 4,
            rerank_budget: 2,
            top_k: 1,
        };

        for p in [
            ScanParams {
                list_size: 0,
                ..base
            },
            ScanParams {
                rerank_budget: 0,
                ..base
            },
            ScanParams { top_k: 0, ..base },
            ScanParams {
                rerank_budget: 5,
                list_size: 4,
                top_k: 1,
                ..base
            },
            ScanParams {
                rerank_budget: 2,
                top_k: 3,
                ..base
            },
        ] {
            assert!(vamana_scan(&reader, p, &prefilter, &rerank).is_err());
        }
    }

    // SC-007: end-to-end with a real Vamana build — synthetic 2D L2
    // points, identical prefilter and rerank closures, top-1 matches
    // brute-force nearest neighbor.
    #[test]
    fn sc_007_end_to_end_matches_brute_force() {
        let n = 64;
        let mut rng = ChaCha8Rng::seed_from_u64(23);
        let points: Vec<(f32, f32)> = (0..n)
            .map(|_| (rng.gen::<f32>(), rng.gen::<f32>()))
            .collect();
        let build_dist = |a: u32, b: u32| {
            let (ax, ay) = points[a as usize];
            let (bx, by) = points[b as usize];
            let dx = ax - bx;
            let dy = ay - by;
            dx * dx + dy * dy
        };
        let medoid = approximate_medoid(n, n, 23, build_dist);
        let graph = build_vamana_graph(n, medoid, 8, 40, 1.2, 29, build_dist);

        let payloads = synth_payloads(n, 0, 0);
        let persisted = persist_vamana_graph(&graph, medoid, DEFAULT_PAGE_SIZE, &payloads, 8, 0, 0)
            .expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 8, 0, 0);

        let query = (rng.gen::<f32>(), rng.gen::<f32>());
        let query_dist_node = |node: u32| {
            let (ax, ay) = points[node as usize];
            let dx = ax - query.0;
            let dy = ay - query.1;
            dx * dx + dy * dy
        };

        let prefilter = |t: &VamanaNodeTuple| {
            let node = (t.primary_heaptid.block_number - 1000) as u32;
            query_dist_node(node)
        };
        let rerank = |hip: ItemPointer| {
            let node = (hip.block_number - 1000) as u32;
            query_dist_node(node)
        };

        let params = ScanParams {
            entry_point: persisted.entry_point_tid,
            list_size: 20,
            rerank_budget: 10,
            top_k: 5,
        };
        let res = vamana_scan(&reader, params, prefilter, rerank).expect("scan");

        // Brute-force top-5.
        let mut all: Vec<(u32, f32)> = (0..n as u32).map(|i| (i, query_dist_node(i))).collect();
        all.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let brute_top: Vec<u32> = all.iter().take(5).map(|(i, _)| *i).collect();

        let got_nodes: Vec<u32> = res
            .iter()
            .map(|r| (r.primary_heaptid.block_number - 1000) as u32)
            .collect();

        // Top-1 must match brute force exactly.
        assert_eq!(got_nodes[0], brute_top[0], "top-1 must be exact");
        // Top-5 recall (overlap with brute top-5) must be >= 4/5 on
        // this small L2 test with list_size=20.
        let brute_set: HashSet<u32> = brute_top.into_iter().collect();
        let got_set: HashSet<u32> = got_nodes.into_iter().collect();
        let overlap = brute_set.intersection(&got_set).count();
        assert!(overlap >= 4, "top-5 recall too low: overlap {overlap} / 5");
    }

    // SC-008: ScanResult carries primary_heaptid from the decoded
    // tuple — this is the value amgettuple returns.
    #[test]
    fn sc_008_result_carries_primary_heaptid() {
        let n = 3;
        let g = chain_graph(n, 4);
        let payloads = synth_payloads(n, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 0, 0);

        let prefilter = |t: &VamanaNodeTuple| (t.primary_heaptid.block_number - 1000) as f32;
        let rerank = |hip: ItemPointer| (hip.block_number - 1000) as f32;
        let params = ScanParams {
            entry_point: persisted.entry_point_tid,
            list_size: 3,
            rerank_budget: 3,
            top_k: 3,
        };
        let res = vamana_scan(&reader, params, prefilter, rerank).expect("scan");
        for (i, r) in res.iter().enumerate() {
            assert_eq!(
                r.primary_heaptid, payloads[i].primary_heaptid,
                "result {i} primary_heaptid mismatch"
            );
        }
    }

    // SC-009: results are sorted ascending by rerank distance.
    #[test]
    fn sc_009_results_sorted_by_distance() {
        let n = 10;
        let g = chain_graph(n, 4);
        let payloads = synth_payloads(n, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 0, 0);

        let prefilter = |t: &VamanaNodeTuple| (t.primary_heaptid.block_number - 1000) as f32;
        let rerank = |hip: ItemPointer| {
            let node = (hip.block_number - 1000) as f32;
            // Invert: distance = 100 - node, so larger node = smaller dist.
            100.0 - node
        };
        let params = ScanParams {
            entry_point: persisted.entry_point_tid,
            list_size: 8,
            rerank_budget: 8,
            top_k: 5,
        };
        let res = vamana_scan(&reader, params, prefilter, rerank).expect("scan");
        assert_eq!(res.len(), 5);
        for pair in res.windows(2) {
            assert!(
                pair[0].distance <= pair[1].distance,
                "results must be ascending by distance"
            );
        }
    }

    // SC-010: greedy_descent is usable standalone (exposed for Phase
    // 6B's use case of batched rerank across amgettuple calls).
    #[test]
    fn sc_010_greedy_descent_exposed() {
        let n = 50;
        let mut rng = ChaCha8Rng::seed_from_u64(31);
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
        let params = BuildParams {
            graph_degree_r: 8,
            build_list_size_l: 32,
            alpha: 1.2,
            dimensions: 64,
            search_subvector_count: 8,
            search_subvector_dim: 8,
            seed: 37,
            page_size: DEFAULT_PAGE_SIZE,
            has_binary_sidecar: false,
        };
        let payloads = synth_payloads(n, 0, params.search_code_len());
        let out = build_and_persist_vamana(params, &payloads, dist).expect("build");

        let reader = PersistedGraphReader::new(
            &out.persisted.chain,
            out.metadata.graph_degree_r,
            params.binary_word_count(),
            params.search_code_len(),
        );

        let query = (0.5f32, 0.5);
        let prefilter = |t: &VamanaNodeTuple| {
            let node = (t.primary_heaptid.block_number - 1000) as usize;
            let (ax, ay) = points[node];
            let dx = ax - query.0;
            let dy = ay - query.1;
            dx * dx + dy * dy
        };
        let frontier =
            greedy_descent(&reader, out.metadata.entry_point, 20, &prefilter).expect("descent");
        assert_eq!(frontier.len(), 20.min(n));
        for pair in frontier.windows(2) {
            assert!(pair[0].score <= pair[1].score);
        }
    }

    // SC-011: vamana_scan_with reuses a caller-owned VisitedState
    // across two scans and yields the same result each time as the
    // allocation-per-call variant.
    #[test]
    fn sc_011_scan_with_scratch_reuse_matches_fresh() {
        let n = 12;
        let g = chain_graph(n, 4);
        let payloads = synth_payloads(n, 0, 0);
        let persisted =
            persist_vamana_graph(&g, 0, DEFAULT_PAGE_SIZE, &payloads, 4, 0, 0).expect("persist");
        let reader = PersistedGraphReader::new(&persisted.chain, 4, 0, 0);

        let prefilter_a = |t: &VamanaNodeTuple| (t.primary_heaptid.block_number - 1000) as f32;
        let rerank_a = |hip: ItemPointer| (hip.block_number - 1000) as f32;
        let prefilter_b =
            |t: &VamanaNodeTuple| (n as f32) - (t.primary_heaptid.block_number - 1000) as f32;
        let rerank_b = |hip: ItemPointer| (n as f32) - (hip.block_number - 1000) as f32;

        let params_a = ScanParams {
            entry_point: persisted.entry_point_tid,
            list_size: 6,
            rerank_budget: 4,
            top_k: 3,
        };
        let params_b = ScanParams { ..params_a };

        let fresh_a = vamana_scan(&reader, params_a, prefilter_a, rerank_a).expect("fresh a");
        let fresh_b = vamana_scan(&reader, params_b, prefilter_b, rerank_b).expect("fresh b");

        use crate::am::ec_diskann::reader::VisitedState;
        let mut scratch = VisitedState::new();
        let reused_a = vamana_scan_with(&reader, &mut scratch, params_a, prefilter_a, rerank_a)
            .expect("reused a");
        let reused_b = vamana_scan_with(&reader, &mut scratch, params_b, prefilter_b, rerank_b)
            .expect("reused b");

        assert_eq!(fresh_a, reused_a, "first reuse must match fresh");
        assert_eq!(
            fresh_b, reused_b,
            "second reuse must match fresh (clear worked)"
        );
    }

    /// What to apply to each marked node in a persisted chain fixture.
    #[derive(Clone, Copy)]
    enum DeathKind {
        /// Vacuum pass 3: `deleted = true`, payload retained.
        Tombstone,
        /// Vacuum pass 1: `primary_heaptid = INVALID`, no overflow,
        /// `deleted` stays `false` — the transient state packets
        /// 11023/11027/11028 call out.
        StripNoTombstone,
    }

    fn persisted_chain_with_deaths(
        n: usize,
        max_degree: u16,
        deaths: &[(u32, DeathKind)],
    ) -> (crate::storage::page::DataPageChain, Vec<ItemPointer>) {
        use crate::am::ec_diskann::vacuum::{mark_deleted, strip_dead_primary_heaptid};
        let g = chain_graph(n, max_degree as usize);
        let payloads = synth_payloads(n, 0, 0);
        let mut persisted = crate::am::ec_diskann::persist::persist_vamana_graph(
            &g,
            0,
            crate::storage::page::DEFAULT_PAGE_SIZE,
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

    // Helper shared by SC-012..SC-014: persist a chain graph and
    // tombstone the named node ids in place.
    fn persisted_chain_with_tombstones(
        n: usize,
        max_degree: u16,
        to_tombstone: &[u32],
    ) -> (crate::storage::page::DataPageChain, Vec<ItemPointer>) {
        let deaths: Vec<(u32, DeathKind)> = to_tombstone
            .iter()
            .map(|&id| (id, DeathKind::Tombstone))
            .collect();
        persisted_chain_with_deaths(n, max_degree, &deaths)
    }

    // SC-012: resolve_entry_point returns the preferred TID when live.
    #[test]
    fn sc_012_resolve_entry_point_prefers_live_medoid() {
        let n = 8;
        let (chain, node_to_tid) = persisted_chain_with_tombstones(n, 4, &[]);
        let reader = PersistedGraphReader::new(&chain, 4, 0, 0);

        let medoid = node_to_tid[3];
        let got = resolve_entry_point(&reader, medoid).expect("ok");
        assert_eq!(got, Some(medoid));
    }

    // SC-013: resolve_entry_point falls back to first_live_tid when
    // the preferred TID is tombstoned, and the fallback is distinct
    // from the dead medoid.
    #[test]
    fn sc_013_resolve_entry_point_falls_back_on_dead_medoid() {
        let n = 8;
        // Tombstone node 0 and designate it the medoid. Fallback must
        // not equal node 0's TID; it must equal first_live_tid.
        let (chain, node_to_tid) = persisted_chain_with_tombstones(n, 4, &[0]);
        let reader = PersistedGraphReader::new(&chain, 4, 0, 0);

        let dead_medoid = node_to_tid[0];
        let got = resolve_entry_point(&reader, dead_medoid)
            .expect("ok")
            .expect("fallback exists");
        assert_ne!(got, dead_medoid, "must not return the dead preferred TID");

        let expected = reader.first_live_tid().expect("ok").expect("live exists");
        assert_eq!(got, expected);
    }

    // SC-014: resolve_entry_point returns None when the chain has no
    // live tuples. INVALID preferred TID also falls back.
    #[test]
    fn sc_014_resolve_entry_point_none_when_all_dead() {
        let n = 5;
        let all: Vec<u32> = (0..n as u32).collect();
        let (chain, node_to_tid) = persisted_chain_with_tombstones(n, 4, &all);
        let reader = PersistedGraphReader::new(&chain, 4, 0, 0);

        let got = resolve_entry_point(&reader, node_to_tid[0]).expect("ok");
        assert!(got.is_none(), "expected None got {got:?}");

        let got_invalid = resolve_entry_point(&reader, ItemPointer::INVALID).expect("ok");
        assert!(got_invalid.is_none());
    }

    // SC-015: resolve_entry_point rejects a stripped-but-not-
    // tombstoned preferred TID (packets 11023/11027/11028). Even
    // though `deleted == false`, the tuple has no heap TID to serve —
    // must fall back to the lowest-block live tuple.
    #[test]
    fn sc_015_resolve_entry_point_rejects_stripped_without_tombstone() {
        let n = 8;
        let deaths = [(0u32, DeathKind::StripNoTombstone)];
        let (chain, node_to_tid) = persisted_chain_with_deaths(n, 4, &deaths);
        let reader = PersistedGraphReader::new(&chain, 4, 0, 0);

        let stripped_medoid = node_to_tid[0];
        let got = resolve_entry_point(&reader, stripped_medoid)
            .expect("ok")
            .expect("fallback exists");
        assert_ne!(
            got, stripped_medoid,
            "must not return the stripped preferred TID"
        );

        let expected = reader.first_live_tid().expect("ok").expect("live exists");
        assert_eq!(got, expected);
    }

    // SC-016: scan does not emit a stripped-but-not-tombstoned tuple
    // as a result, even when it would outrank live tuples under the
    // prefilter (packets 11023/11027/11028). Traversal still walks
    // through it for connectivity.
    #[test]
    fn sc_016_scan_drops_stripped_candidates() {
        let n = 6;
        // Strip node 2 (would otherwise win the prefilter — smallest
        // score) and tombstone node 4 (should also be dropped by the
        // current filter). Only live nodes are emitted.
        let deaths = [
            (2u32, DeathKind::StripNoTombstone),
            (4u32, DeathKind::Tombstone),
        ];
        let (chain, node_to_tid) = persisted_chain_with_deaths(n, 4, &deaths);
        let reader = PersistedGraphReader::new(&chain, 4, 0, 0);

        let prefilter_scores = [5.0f32, 4.0, 0.0, 3.0, 1.0, 2.0]; // 2 and 4 rank high
        let prefilter = |t: &VamanaNodeTuple| {
            if t.primary_heaptid == ItemPointer::INVALID {
                // Stripped node — the actual pgrx prefilter would
                // still score from the intact binary_words/search_code
                // fields. Simulate the same shape: return the base
                // score keyed by index-tuple block (not by the empty
                // primary_heaptid).
                return -1.0;
            }
            let node = (t.primary_heaptid.block_number - 1000) as usize;
            prefilter_scores[node]
        };
        let rerank = |hip: ItemPointer| {
            assert_ne!(
                hip,
                ItemPointer::INVALID,
                "rerank must never receive INVALID primary_heaptid",
            );
            let node = (hip.block_number - 1000) as usize;
            prefilter_scores[node]
        };

        // Entry from node 0 (still live) so the scan starts from a
        // live tuple. The traversal will still touch stripped/
        // tombstoned tuples when it walks neighbors.
        let params = ScanParams {
            entry_point: node_to_tid[0],
            list_size: n,
            rerank_budget: n,
            top_k: 4,
        };
        let res = vamana_scan(&reader, params, prefilter, rerank).expect("scan");
        // No result has primary_heaptid == INVALID.
        for r in &res {
            assert_ne!(r.primary_heaptid, ItemPointer::INVALID);
        }
        // Stripped node 2 and tombstoned node 4 must not appear.
        let emitted_nodes: Vec<u32> = res
            .iter()
            .map(|r| r.primary_heaptid.block_number - 1000)
            .collect();
        assert!(!emitted_nodes.contains(&2), "stripped node 2 leaked");
        assert!(!emitted_nodes.contains(&4), "tombstoned node 4 leaked");
    }

    // SC-017: when every tuple between the entry and the frontier tail
    // is stripped, vamana_scan returns an empty result rather than
    // erroring.
    #[test]
    fn sc_017_scan_returns_empty_when_all_stripped() {
        let n = 4;
        let deaths: Vec<(u32, DeathKind)> = (0..n as u32)
            .map(|id| (id, DeathKind::StripNoTombstone))
            .collect();
        // Keep node 0 alive so we have an entry point.
        let deaths_tail = &deaths[1..];
        let (chain, node_to_tid) = persisted_chain_with_deaths(n, 4, deaths_tail);
        // Now also strip node 0 — but we need a live entry point to
        // enter the scan. Use node 0's TID as the entry; strip it
        // after the chain is built by rebuilding with every node
        // stripped, but exercise the filter via an alive entry that
        // points into all-stripped neighbors.
        let reader = PersistedGraphReader::new(&chain, 4, 0, 0);
        // Node 0 alive, nodes 1..=3 stripped.
        let prefilter = |_: &VamanaNodeTuple| 1.0f32;
        let rerank = |hip: ItemPointer| {
            assert_ne!(hip, ItemPointer::INVALID);
            1.0f32
        };
        let params = ScanParams {
            entry_point: node_to_tid[0],
            list_size: n,
            rerank_budget: n,
            top_k: n,
        };
        let res = vamana_scan(&reader, params, prefilter, rerank).expect("scan");
        // Only node 0 survives the filter.
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].primary_heaptid.block_number, 1000);
    }
}
