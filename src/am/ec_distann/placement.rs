//! FR-078 hash placement + topology-only placement directory (Task 164 M2).
//!
//! A record's owning node is `hash(vec_id) mod node_count` under the epoch's
//! registered roster. Placement affects load balance only — never recall or
//! graph structure (FR-078) — and is resolvable by any coordinator from the
//! roster alone, with no per-record directory. The co-placed full-precision
//! heap row (the FR-076 rerank tier) lands on the same owning node by the
//! identical `hash(vec_id)`, so exact rerank (FR-079) is always node-local
//! (ADR-085 D11).
//!
//! The placement hash is versioned (`DISTANN_PLACEMENT_HASH_V1`) and applied
//! *on top of* the vec_id (itself already a murmur3 avalanche, `identity.rs`):
//! a second independent mix keeps placement distribution decoupled from any
//! structure in the vec_id space and lets the epoch manifest pin the placement
//! function version separately from the identity function version (FR-078:
//! "the epoch manifest SHALL fix the hash function version and roster
//! ordering so every participant computes identical placement").

// Slice 1 of M2: the directory/roster types and grouping helper are consumed
// by the remote expander + `ec_distann_expand_nodes` endpoint in the following
// M2 slices. Allow dead_code until those land on this branch.
#![allow(dead_code)]

/// Placement-hash function version. Bump on any change to `placement_hash`
/// (a placement change is an epoch rebuild, not a migration — NFR-016).
pub(crate) const DISTANN_PLACEMENT_HASH_V1: u16 = 1;

/// Domain separator so the placement mix never aliases the identity mix even
/// though both use the same fmix64 primitive.
const DISTANN_PLACEMENT_DOMAIN: u64 = 0x6469_7374_616e_6e70; // "distann" | 0x70 ('p')

/// murmur3 64-bit finalizer (same primitive as `identity::fmix64`; duplicated
/// here so placement owns its versioned hash independently of identity).
fn fmix64(mut h: u64) -> u64 {
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    h
}

/// The versioned placement hash of a vec_id. Independent of `node_count` so a
/// roster resize reuses the same hash (only the modulus changes, in a new
/// epoch).
pub(crate) fn placement_hash(vec_id: u64, hash_version: u16) -> u64 {
    match hash_version {
        DISTANN_PLACEMENT_HASH_V1 => fmix64(vec_id ^ DISTANN_PLACEMENT_DOMAIN),
        other => pgrx_unreachable_version(other),
    }
}

#[cfg(not(test))]
fn pgrx_unreachable_version(other: u16) -> u64 {
    pgrx::error!("ec_distann unknown placement hash version {other}")
}

#[cfg(test)]
fn pgrx_unreachable_version(other: u16) -> u64 {
    panic!("ec_distann unknown placement hash version {other}")
}

/// Owning node index (`0..node_count`) for a vec_id under a roster of
/// `node_count` nodes. `node_count` must be >= 1 (a published epoch always has
/// at least one node). FR-078-AC-1: identical wherever it is computed.
pub(crate) fn owning_node(vec_id: u64, node_count: usize, hash_version: u16) -> usize {
    debug_assert!(node_count >= 1, "a published epoch has >= 1 node");
    let nc = node_count.max(1) as u64;
    (placement_hash(vec_id, hash_version) % nc) as usize
}

/// Group `vec_ids` by owning node in O(set size) using only the roster size
/// (FR-078: "group any set of vec_ids by owning node in O(set size) using only
/// the manifest"). Returns one bucket per node index `0..node_count`; empty
/// buckets are retained so the caller can index by node. Each bucket preserves
/// the input order of its members, which — combined with the caller stitching
/// buckets back in request order — satisfies FR-079-AC-1.
pub(crate) fn group_by_owning_node(
    vec_ids: &[u64],
    node_count: usize,
    hash_version: u16,
) -> Vec<Vec<u64>> {
    let mut buckets: Vec<Vec<u64>> = vec![Vec::new(); node_count.max(1)];
    for &vec_id in vec_ids {
        let node = owning_node(vec_id, node_count, hash_version);
        buckets[node].push(vec_id);
    }
    buckets
}

/// Epoch-stamped, topology-only placement directory (FR-078): roster + hash
/// version + per-node record counts. Adapted from `SpirePlacementDirectory`
/// (whose contract stays owned by the SPIRE specs); carries **no per-record
/// entries**. `nodes[i]` is the roster-ordered identity of node index `i`; the
/// ordering is what `owning_node` indexes into, so it is frozen in the epoch.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DistannPlacementDirectory {
    pub(crate) epoch: u64,
    pub(crate) hash_version: u16,
    /// Roster-ordered node identities. Index = owning-node index.
    pub(crate) nodes: Vec<DistannRosterNode>,
    /// Per-node record counts (FR-078-AC-2 load-balance evidence). Parallel to
    /// `nodes`; may be empty until the build stats are attached.
    pub(crate) record_counts: Vec<u64>,
}

/// One roster entry: a node identity plus how a coordinator reaches it. For
/// the M2 loopback multi-instance substrate (ADR-085 D2) this is a libpq
/// conninfo; `SPIRE_LOCAL_NODE`-style local ownership is marked by
/// `is_local`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DistannRosterNode {
    /// Stable node id (roster identity, not the roster index).
    pub(crate) node_id: u32,
    /// True when this node is the coordinator's own instance (expand locally,
    /// no round trip).
    pub(crate) is_local: bool,
    /// libpq conninfo for the remote expand call; empty for the local node.
    pub(crate) conninfo: String,
}

impl DistannPlacementDirectory {
    pub(crate) fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Owning roster node for a vec_id under this epoch's roster + hash version.
    pub(crate) fn owner(&self, vec_id: u64) -> &DistannRosterNode {
        let index = owning_node(vec_id, self.nodes.len(), self.hash_version);
        &self.nodes[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // FR-078-AC-1: placement is a pure function of (vec_id, node_count,
    // version) — identical wherever computed.
    #[test]
    fn placement_is_deterministic() {
        for vec_id in [0_u64, 1, 42, u64::MAX, 0xDEAD_BEEF_CAFE_F00D] {
            let a = owning_node(vec_id, 3, DISTANN_PLACEMENT_HASH_V1);
            let b = owning_node(vec_id, 3, DISTANN_PLACEMENT_HASH_V1);
            assert_eq!(a, b);
            assert!(a < 3);
        }
    }

    // FR-078-AC-3: within an epoch the roster (node_count) is fixed, so
    // placement never changes. A *different* node_count is a new epoch.
    #[test]
    fn placement_changes_only_with_node_count() {
        let vec_id = 0x0123_4567_89AB_CDEF;
        let three = owning_node(vec_id, 3, DISTANN_PLACEMENT_HASH_V1);
        let three_again = owning_node(vec_id, 3, DISTANN_PLACEMENT_HASH_V1);
        assert_eq!(three, three_again);
        // Not asserting equality across node_count — that is by construction a
        // new epoch's placement.
        let _four = owning_node(vec_id, 4, DISTANN_PLACEMENT_HASH_V1);
    }

    // FR-078-AC-2: record counts per node within 10% of uniform at 100k across
    // 3 nodes. Use murmur3-mixed vec_ids (as identity.rs produces) so the test
    // reflects real placement, not raw sequential ids.
    #[test]
    fn placement_is_balanced_across_three_nodes() {
        let node_count = 3;
        let n = 100_000_u64;
        let mut counts = vec![0_u64; node_count];
        for i in 0..n {
            // Emulate an identity-hashed vec_id.
            let vec_id = super::fmix64(i ^ 0x9E37_79B9_7F4A_7C15);
            counts[owning_node(vec_id, node_count, DISTANN_PLACEMENT_HASH_V1)] += 1;
        }
        let uniform = n as f64 / node_count as f64;
        for (node, &count) in counts.iter().enumerate() {
            let deviation = (count as f64 - uniform).abs() / uniform;
            assert!(
                deviation < 0.10,
                "node {node} count {count} deviates {:.3} from uniform {uniform:.0} (>10%)",
                deviation
            );
        }
    }

    #[test]
    fn group_by_owning_node_covers_all_ids_in_order() {
        let vec_ids: Vec<u64> = (0..50).map(|i| super::fmix64(i)).collect();
        let node_count = 4;
        let buckets = group_by_owning_node(&vec_ids, node_count, DISTANN_PLACEMENT_HASH_V1);
        assert_eq!(buckets.len(), node_count);
        // Every id appears exactly once, in its owner's bucket, order preserved.
        let mut seen: HashMap<u64, usize> = HashMap::new();
        for (node, bucket) in buckets.iter().enumerate() {
            for &vec_id in bucket {
                assert_eq!(owning_node(vec_id, node_count, DISTANN_PLACEMENT_HASH_V1), node);
                *seen.entry(vec_id).or_insert(0) += 1;
            }
        }
        assert_eq!(seen.len(), vec_ids.len());
        assert!(seen.values().all(|&c| c == 1));
        // Order within a bucket matches input order.
        for bucket in &buckets {
            let mut expected: Vec<u64> = vec_ids
                .iter()
                .copied()
                .filter(|id| bucket.contains(id))
                .collect();
            expected.retain(|id| bucket.contains(id));
            assert_eq!(bucket, &expected);
        }
    }

    // Degenerate single-node (M0/M2 coordinator-only): every id owned locally.
    #[test]
    fn single_node_owns_everything() {
        for vec_id in [0_u64, 7, u64::MAX] {
            assert_eq!(owning_node(vec_id, 1, DISTANN_PLACEMENT_HASH_V1), 0);
        }
    }
}
