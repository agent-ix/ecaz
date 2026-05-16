//! Persisted-graph diagnostics for `ec_diskann`.
//!
//! This module is intentionally read-only. It materializes the on-disk
//! Vamana chain through the same scan-state reader used by `amgettuple`,
//! then computes graph-shape counters for local tuning packets.

use std::collections::{HashMap, HashSet};

use pgrx::pg_sys;

use crate::storage::page::ItemPointer;

use super::{
    reader::PersistedGraphReader,
    scan_state::{self, metadata_binary_word_count, metadata_search_code_len},
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DiskannGraphSummary {
    pub block_count: u32,
    pub graph_degree_r: u16,
    pub build_list_size_l: u16,
    pub alpha: f32,
    pub dimensions: u16,
    pub inserted_since_rebuild: u64,
    pub needs_medoid_refresh: bool,
    pub node_count: usize,
    pub live_node_count: usize,
    pub non_live_node_count: usize,
    pub entry_point_live: bool,
    pub reachable_live_node_count: usize,
    pub unreachable_live_node_count: usize,
    pub reachable_live_fraction: f64,
    pub neighbor_ref_count: usize,
    pub live_neighbor_ref_count: usize,
    pub dead_neighbor_ref_count: usize,
    pub invalid_neighbor_ref_count: usize,
    pub self_neighbor_ref_count: usize,
    pub duplicate_neighbor_ref_count: usize,
    pub unresolvable_neighbor_ref_count: usize,
    pub zero_out_degree_count: usize,
    pub min_out_degree: usize,
    pub avg_out_degree: f64,
    pub p50_out_degree: usize,
    pub p95_out_degree: usize,
    pub p99_out_degree: usize,
    pub max_out_degree: usize,
    pub zero_in_degree_count: usize,
    pub min_in_degree: usize,
    pub avg_in_degree: f64,
    pub p50_in_degree: usize,
    pub p95_in_degree: usize,
    pub p99_in_degree: usize,
    pub max_in_degree: usize,
}

pub(crate) unsafe fn graph_summary(
    index_relation: pg_sys::Relation,
) -> Result<DiskannGraphSummary, String> {
    // SAFETY: The caller provides a live DiskANN index relation; this read-only
    // diagnostic only asks PostgreSQL for the current MAIN fork block count.
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    // SAFETY: The same live DiskANN index relation is materialized through the
    // scan-state reader without modifying pages.
    let (metadata, chain) = unsafe { scan_state::materialize_chain_from_index(index_relation)? };
    let reader = PersistedGraphReader::new(
        &chain,
        metadata.graph_degree_r,
        metadata_binary_word_count(&metadata),
        metadata_search_code_len(&metadata),
    );

    let mut node_tids = Vec::new();
    let mut live_tids = HashSet::new();
    let mut live_nodes = Vec::new();
    for item in reader.iter_node_tids() {
        let tid = item?;
        let tuple = reader.read_node(tid)?;
        node_tids.push(tid);
        if tuple.is_live() {
            live_tids.insert(tid);
            live_nodes.push((tid, tuple));
        }
    }

    let mut out_degrees = Vec::with_capacity(live_nodes.len());
    let mut in_degrees: HashMap<ItemPointer, usize> =
        live_tids.iter().map(|tid| (*tid, 0)).collect();
    let mut neighbor_ref_count = 0;
    let mut live_neighbor_ref_count = 0;
    let mut dead_neighbor_ref_count = 0;
    let mut invalid_neighbor_ref_count = 0;
    let mut self_neighbor_ref_count = 0;
    let mut duplicate_neighbor_ref_count = 0;
    let mut unresolvable_neighbor_ref_count = 0;

    for (tid, tuple) in &live_nodes {
        let count = usize::from(tuple.neighbor_count).min(tuple.neighbors.len());
        out_degrees.push(count);
        neighbor_ref_count += count;

        let mut seen_for_source = HashSet::new();
        for neighbor_tid in tuple.neighbors.iter().copied().take(count) {
            if neighbor_tid == ItemPointer::INVALID {
                invalid_neighbor_ref_count += 1;
                continue;
            }
            if neighbor_tid == *tid {
                self_neighbor_ref_count += 1;
            }
            if !seen_for_source.insert(neighbor_tid) {
                duplicate_neighbor_ref_count += 1;
            }

            match reader.read_node(neighbor_tid) {
                Ok(neighbor_tuple) if neighbor_tuple.is_live() => {
                    live_neighbor_ref_count += 1;
                    if let Some(in_degree) = in_degrees.get_mut(&neighbor_tid) {
                        *in_degree += 1;
                    }
                }
                Ok(_) => {
                    dead_neighbor_ref_count += 1;
                }
                Err(_) => {
                    unresolvable_neighbor_ref_count += 1;
                }
            }
        }
    }

    let reachable = reachable_live_tids(&reader, metadata.entry_point, &live_tids);
    let reachable_live_node_count = reachable.len();
    let live_node_count = live_nodes.len();
    let unreachable_live_node_count = live_node_count.saturating_sub(reachable_live_node_count);
    let reachable_live_fraction = if live_node_count == 0 {
        0.0
    } else {
        reachable_live_node_count as f64 / live_node_count as f64
    };

    let in_degree_values: Vec<usize> = live_tids
        .iter()
        .map(|tid| in_degrees.get(tid).copied().unwrap_or(0))
        .collect();

    Ok(DiskannGraphSummary {
        block_count,
        graph_degree_r: metadata.graph_degree_r,
        build_list_size_l: metadata.build_list_size_l,
        alpha: metadata.alpha,
        dimensions: metadata.dimensions,
        inserted_since_rebuild: metadata.inserted_since_rebuild,
        needs_medoid_refresh: metadata.needs_medoid_refresh,
        node_count: node_tids.len(),
        live_node_count,
        non_live_node_count: node_tids.len().saturating_sub(live_node_count),
        entry_point_live: live_tids.contains(&metadata.entry_point),
        reachable_live_node_count,
        unreachable_live_node_count,
        reachable_live_fraction,
        neighbor_ref_count,
        live_neighbor_ref_count,
        dead_neighbor_ref_count,
        invalid_neighbor_ref_count,
        self_neighbor_ref_count,
        duplicate_neighbor_ref_count,
        unresolvable_neighbor_ref_count,
        zero_out_degree_count: out_degrees.iter().filter(|degree| **degree == 0).count(),
        min_out_degree: min_value(&out_degrees),
        avg_out_degree: avg_value(&out_degrees),
        p50_out_degree: percentile_nearest(&out_degrees, 0.50),
        p95_out_degree: percentile_nearest(&out_degrees, 0.95),
        p99_out_degree: percentile_nearest(&out_degrees, 0.99),
        max_out_degree: max_value(&out_degrees),
        zero_in_degree_count: in_degree_values
            .iter()
            .filter(|degree| **degree == 0)
            .count(),
        min_in_degree: min_value(&in_degree_values),
        avg_in_degree: avg_value(&in_degree_values),
        p50_in_degree: percentile_nearest(&in_degree_values, 0.50),
        p95_in_degree: percentile_nearest(&in_degree_values, 0.95),
        p99_in_degree: percentile_nearest(&in_degree_values, 0.99),
        max_in_degree: max_value(&in_degree_values),
    })
}

fn reachable_live_tids(
    reader: &PersistedGraphReader<'_>,
    entry_point: ItemPointer,
    live_tids: &HashSet<ItemPointer>,
) -> HashSet<ItemPointer> {
    let mut reachable = HashSet::new();
    if !live_tids.contains(&entry_point) {
        return reachable;
    }

    let mut stack = vec![entry_point];
    while let Some(tid) = stack.pop() {
        if !reachable.insert(tid) {
            continue;
        }
        if let Ok(neighbors) = reader.neighbors(tid) {
            for neighbor_tid in neighbors {
                if live_tids.contains(&neighbor_tid) && !reachable.contains(&neighbor_tid) {
                    stack.push(neighbor_tid);
                }
            }
        }
    }
    reachable
}

fn min_value(values: &[usize]) -> usize {
    values.iter().copied().min().unwrap_or(0)
}

fn max_value(values: &[usize]) -> usize {
    values.iter().copied().max().unwrap_or(0)
}

fn avg_value(values: &[usize]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<usize>() as f64 / values.len() as f64
}

fn percentile_nearest(values: &[usize], percentile: f64) -> usize {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let rank = ((sorted.len() - 1) as f64 * percentile).round() as usize;
    sorted[rank.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::{avg_value, max_value, min_value, percentile_nearest};

    #[test]
    fn degree_summary_helpers_handle_empty_input() {
        assert_eq!(min_value(&[]), 0);
        assert_eq!(max_value(&[]), 0);
        assert_eq!(avg_value(&[]), 0.0);
        assert_eq!(percentile_nearest(&[], 0.95), 0);
    }

    #[test]
    fn percentile_nearest_uses_sorted_values() {
        let values = [10, 2, 4, 8, 6];
        assert_eq!(percentile_nearest(&values, 0.50), 6);
        assert_eq!(percentile_nearest(&values, 0.95), 10);
    }
}
