//! FR-089 bounded coordinator crown cache.
//!
//! The crown contains only `(vec_id, search_code)` pairs.  It is a
//! rebuildable, epoch-scoped candidate cache; exact distances and graph
//! payloads remain owner-authoritative.

use std::sync::atomic::{AtomicU64, Ordering};

use pgrx::iter::TableIterator;
use pgrx::{name, pg_extern};

use super::canonical_wire::{domain_digest, CanonicalEncoder};
use super::scan::DistannSeedCandidate;

const CROWN_DOMAIN: &[u8] = b"ec_distann_crown_v1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DistannCrownEntry {
    pub(crate) vec_id: u64,
    pub(crate) search_code: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DistannCrownCache {
    capacity: usize,
    epoch_fingerprint: [u8; 34],
    entries: Vec<DistannCrownEntry>,
    selection_digest: [u8; 32],
}

impl DistannCrownCache {
    pub(crate) fn select_member_ids(members: &[u64], capacity: usize) -> Vec<u64> {
        if capacity == 0 || members.is_empty() {
            return Vec::new();
        }
        let mut ordered = members.to_vec();
        ordered.sort_unstable();
        ordered.dedup();
        if ordered.len() <= capacity {
            return ordered;
        }
        (0..capacity)
            .map(|index| ordered[index * ordered.len() / capacity])
            .collect()
    }

    pub(crate) fn from_entries(
        capacity: usize,
        epoch_fingerprint: [u8; 34],
        selected_members: &[u64],
        entries: Vec<DistannCrownEntry>,
    ) -> Result<Self, String> {
        if capacity == 0 || entries.len() > capacity {
            return Err("EC_CROWN_CAPACITY: crown admission exceeds capacity".to_owned());
        }
        let mut expected = selected_members.to_vec();
        expected.sort_unstable();
        expected.dedup();
        let mut actual = entries.iter().map(|entry| entry.vec_id).collect::<Vec<_>>();
        actual.sort_unstable();
        actual.dedup();
        if actual != expected || entries.iter().any(|entry| entry.search_code.is_empty()) {
            return Err("EC_CROWN_POPULATION: selected crown entries are incomplete".to_owned());
        }
        let mut encoder = CanonicalEncoder::with_capacity(
            34 + 4 + entries.iter().map(|entry| 12 + entry.search_code.len()).sum::<usize>(),
        );
        encoder.put_fixed(&epoch_fingerprint);
        encoder.put_u32(u32::try_from(capacity).map_err(|_| "crown capacity exceeds u32".to_owned())?);
        for entry in &entries {
            encoder.put_u64(entry.vec_id);
            encoder.put_len_prefixed(&entry.search_code)?;
        }
        let selection_digest = domain_digest(CROWN_DOMAIN, &encoder.finish()?);
        Ok(Self {
            capacity,
            epoch_fingerprint,
            entries,
            selection_digest,
        })
    }

    pub(crate) fn capacity(&self) -> usize { self.capacity }
    pub(crate) fn len(&self) -> usize { self.entries.len() }
    pub(crate) fn resident_bytes(&self) -> usize {
        self.entries
            .iter()
            .map(|entry| 8usize.saturating_add(entry.search_code.len()))
            .sum()
    }
    pub(crate) fn selection_digest(&self) -> [u8; 32] { self.selection_digest }
    pub(crate) fn epoch_fingerprint(&self) -> [u8; 34] { self.epoch_fingerprint }
    pub(crate) fn contains(&self, vec_id: u64) -> bool {
        self.entries.iter().any(|entry| entry.vec_id == vec_id)
    }

    pub(crate) fn rank<F>(
        &self,
        query: &[f32],
        seed_count: usize,
        mut score: F,
    ) -> Result<Vec<DistannSeedCandidate>, String>
    where
        F: FnMut(&[u8]) -> Result<f32, String>,
    {
        let mut ranked = self
            .entries
            .iter()
            .map(|entry| Ok((entry.vec_id, score(&entry.search_code)?)))
            .collect::<Result<Vec<_>, String>>()?;
        ranked.sort_unstable_by(|left, right| {
            left.1
                .total_cmp(&right.1)
                .then_with(|| left.0.cmp(&right.0))
        });
        let _ = query;
        Ok(ranked
            .into_iter()
            .take(seed_count)
            .map(|(vec_id, dist)| DistannSeedCandidate { vec_id, dist })
            .collect())
    }
}

static CROWN_SEEDS_SERVED: AtomicU64 = AtomicU64::new(0);
static CROWN_FALLBACKS: AtomicU64 = AtomicU64::new(0);
static FUSED_HEAD_HOPS: AtomicU64 = AtomicU64::new(0);

pub(crate) fn record_seeds_served(count: usize) {
    CROWN_SEEDS_SERVED.fetch_add(count as u64, Ordering::Relaxed);
}
pub(crate) fn record_fallback() { CROWN_FALLBACKS.fetch_add(1, Ordering::Relaxed); }
pub(crate) fn record_fused_head_hop() { FUSED_HEAD_HOPS.fetch_add(1, Ordering::Relaxed); }

#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_crown_stats() -> TableIterator<'static, (name!(crown_seeds_served, i64), name!(crown_fallbacks, i64), name!(fused_head_hops, i64))> {
    TableIterator::once((
        i64::try_from(CROWN_SEEDS_SERVED.load(Ordering::Relaxed)).unwrap_or(i64::MAX),
        i64::try_from(CROWN_FALLBACKS.load(Ordering::Relaxed)).unwrap_or(i64::MAX),
        i64::try_from(FUSED_HEAD_HOPS.load(Ordering::Relaxed)).unwrap_or(i64::MAX),
    ))
}

#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_reset_crown_stats() {
    CROWN_SEEDS_SERVED.store(0, Ordering::Relaxed);
    CROWN_FALLBACKS.store(0, Ordering::Relaxed);
    FUSED_HEAD_HOPS.store(0, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_is_deterministic_and_capacity_bounded() {
        let members = vec![90, 10, 70, 30, 50];
        assert_eq!(DistannCrownCache::select_member_ids(&members, 3), vec![10, 30, 70]);
        assert_eq!(DistannCrownCache::select_member_ids(&members, 99), vec![10, 30, 50, 70, 90]);
    }

    #[test]
    fn population_requires_the_complete_selected_set() {
        let fp = [7; 34];
        let selected = vec![10, 20];
        let missing = vec![DistannCrownEntry { vec_id: 10, search_code: vec![1, 2] }];
        assert!(DistannCrownCache::from_entries(2, fp, &selected, missing).is_err());
        let complete = vec![
            DistannCrownEntry { vec_id: 10, search_code: vec![1, 2] },
            DistannCrownEntry { vec_id: 20, search_code: vec![3, 4] },
        ];
        let cache = DistannCrownCache::from_entries(2, fp, &selected, complete).unwrap();
        assert_eq!(cache.len(), 2);
        assert!(cache.resident_bytes() <= 2 * (8 + 2));
    }
}
