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
