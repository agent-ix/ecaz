//! Tuple-level vacuum primitives for `ec_diskann` (task 17 Phase 8A).
//!
//! Pure-Rust deletion + neighbor-repair operations on
//! [`VamanaNodeTuple`]. The pgrx-side three-pass vacuum callback
//! (Phase 8B) is the orchestration layer that walks pages, holds
//! locks per ADR-047, and calls into these primitives.
//!
//! ## Invariants preserved
//!
//! Every primitive in this module preserves the ADR-045 Decision 3
//! fixed-length invariant: encoded byte length is a pure function of
//! the metadata constants `(R, W, C)` and never changes when a tuple
//! is mutated in place. That guarantee is what lets the pgrx vacuum
//! callback use `DataPageChain::update_raw_tuple` to write the
//! mutated tuple back into the same slot.
//!
//! Concretely:
//! - [`mark_deleted`] flips one bit; payload bodies are untouched.
//! - [`strip_dead_primary_heaptid`] overwrites a 6-byte ItemPointer
//!   with `INVALID`; total length unchanged.
//! - [`repair_neighbors`] replaces dead neighbor TIDs with `INVALID`
//!   in place, compacts the live prefix to the front of the
//!   `neighbors` Vec, and tail-pads with `INVALID`. The Vec stays at
//!   length `R`.

use std::collections::HashSet;

use crate::am::ec_diskann::tuple::VamanaNodeTuple;
use crate::storage::page::ItemPointer;

/// Mark a node tuple as logically deleted (ADR-047 vacuum pass 3).
///
/// Sets `deleted = true`. Does **not** clear neighbors or payload
/// bodies — those are still load-bearing for backlink discovery if
/// any concurrent scan / repair pass reaches the tombstone before
/// it's reaped.
///
/// Idempotent: calling twice is a no-op.
pub fn mark_deleted(tuple: &mut VamanaNodeTuple) {
    tuple.deleted = true;
}

/// Strip the `primary_heaptid` if it points at a heap row marked
/// dead by `dead_pred` (ADR-047 vacuum pass 1).
///
/// Returns `true` if the primary heaptid was stripped, `false`
/// otherwise. Sets `primary_heaptid = INVALID` on a strip; does
/// **not** flip `deleted` — pass 3 owns that decision because it
/// also needs to know whether the (future) overflow heaptid chain
/// has any remaining live rows.
///
/// Already-`INVALID` primary heaptids are left alone; `dead_pred` is
/// not invoked on them.
pub fn strip_dead_primary_heaptid<P>(tuple: &mut VamanaNodeTuple, dead_pred: P) -> bool
where
    P: Fn(ItemPointer) -> bool,
{
    if tuple.primary_heaptid == ItemPointer::INVALID {
        return false;
    }
    if dead_pred(tuple.primary_heaptid) {
        tuple.primary_heaptid = ItemPointer::INVALID;
        true
    } else {
        false
    }
}

/// Convenience: a node is "fully dead" when its primary heaptid is
/// `INVALID` and (per the V1 contract) it has no overflow chain.
///
/// Phase 7 (insert) introduces the overflow chain; this helper
/// pessimistically returns `false` whenever `has_overflow_heaptids`
/// is set, so the pass-3 caller knows it must walk the chain to
/// confirm full death. For V1 builds the flag is always false and
/// the helper is exact.
pub fn is_fully_dead(tuple: &VamanaNodeTuple) -> bool {
    tuple.primary_heaptid == ItemPointer::INVALID && !tuple.has_overflow_heaptids
}

/// Repair a node's neighbor list by removing TIDs that point at
/// nodes in `dead_set` (ADR-047 vacuum pass 2 fill-half).
///
/// Walks the filled prefix (`neighbors[..neighbor_count]`), drops
/// any neighbor whose TID is in `dead_set`, compacts the survivors
/// into the prefix, and pads the tail with `INVALID`. Updates
/// `neighbor_count` to the survivor count.
///
/// Returns the number of neighbors removed. Length of the
/// `neighbors` Vec is unchanged (still `R`); ADR-045 fixed-length
/// invariant holds.
///
/// Note: this is **fill-only** w.r.t. ADR-047. New repair candidates
/// (chosen by the pgrx caller via greedy_search under shared lock)
/// are appended in a separate step that this primitive does not
/// own — it only removes the dead.
pub fn repair_neighbors(tuple: &mut VamanaNodeTuple, dead_set: &HashSet<ItemPointer>) -> usize {
    if dead_set.is_empty() {
        return 0;
    }
    let r = tuple.neighbors.len();
    let live_prefix_len = tuple.neighbor_count as usize;
    debug_assert!(
        live_prefix_len <= r,
        "neighbor_count {live_prefix_len} > slots {r}",
    );

    let mut write = 0usize;
    let mut removed = 0usize;
    for read in 0..live_prefix_len {
        let nbr = tuple.neighbors[read];
        if dead_set.contains(&nbr) {
            removed += 1;
            continue;
        }
        if write != read {
            tuple.neighbors[write] = nbr;
        }
        write += 1;
    }
    for slot in tuple.neighbors[write..].iter_mut() {
        *slot = ItemPointer::INVALID;
    }
    tuple.neighbor_count = u16::try_from(write).expect("neighbor count fits in u16");
    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::ec_diskann::tuple::VamanaNodeTuple;

    fn tid(b: u32, o: u16) -> ItemPointer {
        ItemPointer {
            block_number: b,
            offset_number: o,
        }
    }

    fn make_tuple(neighbors: &[ItemPointer], r: u16) -> VamanaNodeTuple {
        let r = r as usize;
        assert!(neighbors.len() <= r);
        let mut slots = vec![ItemPointer::INVALID; r];
        for (i, &n) in neighbors.iter().enumerate() {
            slots[i] = n;
        }
        VamanaNodeTuple {
            deleted: false,
            has_overflow_heaptids: false,
            primary_heaptid: tid(100, 1),
            rerank_tid: ItemPointer::INVALID,
            binary_words: vec![],
            search_code: vec![],
            neighbors: slots,
            neighbor_count: neighbors.len() as u16,
        }
    }

    // VC-001: mark_deleted flips the bit and is idempotent.
    #[test]
    fn vc_001_mark_deleted_is_idempotent() {
        let mut t = make_tuple(&[], 4);
        assert!(!t.deleted);
        mark_deleted(&mut t);
        assert!(t.deleted);
        mark_deleted(&mut t);
        assert!(t.deleted);
    }

    // VC-002: mark_deleted does not clear neighbors or primary heaptid.
    #[test]
    fn vc_002_mark_deleted_preserves_payload() {
        let mut t = make_tuple(&[tid(1, 1), tid(2, 2)], 4);
        let original = t.clone();
        mark_deleted(&mut t);
        assert_eq!(t.neighbors, original.neighbors);
        assert_eq!(t.neighbor_count, original.neighbor_count);
        assert_eq!(t.primary_heaptid, original.primary_heaptid);
    }

    // VC-003: strip_dead_primary_heaptid only fires when predicate
    // matches, returns the right boolean.
    #[test]
    fn vc_003_strip_dead_primary_heaptid_predicate() {
        let mut t = make_tuple(&[], 4);
        assert!(!strip_dead_primary_heaptid(&mut t, |_| false));
        assert_eq!(t.primary_heaptid, tid(100, 1));

        assert!(strip_dead_primary_heaptid(&mut t, |p| p == tid(100, 1)));
        assert_eq!(t.primary_heaptid, ItemPointer::INVALID);
    }

    // VC-004: strip skips already-INVALID heaptids without invoking
    // the predicate.
    #[test]
    fn vc_004_strip_skips_already_invalid() {
        use std::cell::Cell;
        let mut t = make_tuple(&[], 4);
        t.primary_heaptid = ItemPointer::INVALID;
        let called = Cell::new(false);
        let stripped = strip_dead_primary_heaptid(&mut t, |_| {
            called.set(true);
            true
        });
        assert!(!stripped);
        assert!(!called.get(), "predicate must not be invoked on INVALID");
    }

    // VC-005: is_fully_dead — INVALID + no overflow ⇒ true; alive
    // primary or overflow ⇒ false.
    #[test]
    fn vc_005_is_fully_dead_semantics() {
        let mut t = make_tuple(&[], 4);
        assert!(!is_fully_dead(&t));

        t.primary_heaptid = ItemPointer::INVALID;
        assert!(is_fully_dead(&t));

        t.has_overflow_heaptids = true;
        assert!(!is_fully_dead(&t), "overflow chain blocks fully-dead");
    }

    // VC-006: repair_neighbors removes dead, compacts live, pads
    // INVALID, updates neighbor_count.
    #[test]
    fn vc_006_repair_neighbors_compacts_and_pads() {
        let mut t = make_tuple(&[tid(1, 1), tid(2, 2), tid(3, 3), tid(4, 4)], 6);
        assert_eq!(t.neighbor_count, 4);

        let mut dead = HashSet::new();
        dead.insert(tid(2, 2));
        dead.insert(tid(4, 4));

        let removed = repair_neighbors(&mut t, &dead);
        assert_eq!(removed, 2);
        assert_eq!(t.neighbor_count, 2);
        assert_eq!(t.neighbors[0], tid(1, 1));
        assert_eq!(t.neighbors[1], tid(3, 3));
        for slot in &t.neighbors[2..] {
            assert_eq!(*slot, ItemPointer::INVALID, "tail must be INVALID");
        }
        assert_eq!(t.neighbors.len(), 6, "Vec length stays at R");
    }

    // VC-007: empty dead_set is a no-op.
    #[test]
    fn vc_007_repair_with_empty_dead_set_noop() {
        let mut t = make_tuple(&[tid(1, 1), tid(2, 2)], 4);
        let original = t.clone();
        let removed = repair_neighbors(&mut t, &HashSet::new());
        assert_eq!(removed, 0);
        assert_eq!(t, original);
    }

    // VC-008: repair preserves order of survivors (stable
    // compaction). Important so that vacuum doesn't reshuffle the
    // graph topology beyond strict removals.
    #[test]
    fn vc_008_repair_is_stable() {
        let mut t = make_tuple(&[tid(1, 1), tid(2, 2), tid(3, 3), tid(4, 4), tid(5, 5)], 8);
        let mut dead = HashSet::new();
        dead.insert(tid(3, 3));

        repair_neighbors(&mut t, &dead);
        assert_eq!(t.neighbor_count, 4);
        assert_eq!(t.neighbors[0], tid(1, 1));
        assert_eq!(t.neighbors[1], tid(2, 2));
        assert_eq!(t.neighbors[2], tid(4, 4));
        assert_eq!(t.neighbors[3], tid(5, 5));
    }

    // VC-009: ADR-045 Decision 3 — repair preserves encoded length.
    // Critical for the placeholder-then-patch / update_raw_tuple
    // contract.
    #[test]
    fn vc_009_repair_preserves_encoded_length() {
        let r = 8u16;
        let w = 4usize;
        let c = 8usize;
        let mut t = VamanaNodeTuple {
            deleted: false,
            has_overflow_heaptids: false,
            primary_heaptid: tid(100, 1),
            rerank_tid: ItemPointer::INVALID,
            binary_words: vec![0xdead_beefu64; w],
            search_code: vec![0xab; c],
            neighbors: {
                let mut v = vec![ItemPointer::INVALID; r as usize];
                for i in 0..6 {
                    v[i] = tid(10 + i as u32, 1);
                }
                v
            },
            neighbor_count: 6,
        };
        let len_before = t.encode(r, w, c).expect("encode pre").len();

        let mut dead = HashSet::new();
        dead.insert(tid(11, 1));
        dead.insert(tid(13, 1));
        repair_neighbors(&mut t, &dead);

        let len_after = t.encode(r, w, c).expect("encode post").len();
        assert_eq!(len_after, len_before, "ADR-045 length invariant violated");
        assert_eq!(len_after, VamanaNodeTuple::encoded_len(r, w, c));
    }

    // VC-010: deletion state machine — alive → primary stripped →
    // neighbors repaired → marked deleted. Each step is independent;
    // none clears state set by another.
    #[test]
    fn vc_010_full_deletion_state_machine() {
        let mut t = make_tuple(&[tid(1, 1), tid(2, 2), tid(3, 3)], 4);

        // Pass 1: heap row dies, primary stripped.
        let stripped = strip_dead_primary_heaptid(&mut t, |p| p == tid(100, 1));
        assert!(stripped);
        assert_eq!(t.primary_heaptid, ItemPointer::INVALID);
        assert!(!t.deleted, "stripping primary doesn't auto-delete");
        assert!(is_fully_dead(&t), "no overflow chain ⇒ fully dead");

        // Pass 2: neighbors not stripped (this node's neighbors are
        // still live nodes); repair on a different node would touch
        // them. Sanity: repair_neighbors with dead_set containing
        // *this* node's TID is not a self-mutation; it's the
        // responsibility of *other* tuples' pass 2 to forget us.
        // Verified: repair on this tuple with empty dead_set is noop.
        let removed = repair_neighbors(&mut t, &HashSet::new());
        assert_eq!(removed, 0);
        assert_eq!(t.neighbor_count, 3);

        // Pass 3: tombstone.
        mark_deleted(&mut t);
        assert!(t.deleted);
        // Neighbors retained until a later vacuum reaps the page;
        // backlink discovery on dead nodes is allowed.
        assert_eq!(t.neighbor_count, 3);
    }
}
