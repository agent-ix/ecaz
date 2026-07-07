//! ADR-085 D6 vec_id derivation: `vec_id = hash64(source_identity)` with
//! build-time collision detection (collision = build error; insert-time
//! collision = insert error per FR-083).
//!
//! Two identity modes mirror ADR-063:
//!
//! - **Global** (`source_identity = 'include'`): the 16-byte canonical
//!   identity payload (uuid or bytea16 include column) is hashed. Stable
//!   across rebuilds, nodes, and epochs for the same logical row.
//! - **Local** (no `source_identity` reloption): the heap TID is hashed
//!   under a separate domain tag. Stable across *index* rebuilds
//!   (FR-076-AC-2) but not across table rewrites — the same limitation
//!   ADR-063 assigns to local `0x01` identities. A multinode deployment
//!   (FR-078 placement) requires the global mode.
//!
//! The hash must be deterministic and stable across Rust releases (it is
//! persisted), so it is implemented here (murmur3 fmix64 avalanche over the
//! input words) instead of relying on `std` hashers, which document no
//! cross-release stability.

use crate::storage::page::ItemPointer;

/// Domain tags keep the two identity namespaces from aliasing each other.
const DISTANN_VEC_ID_DOMAIN_GLOBAL: u64 = 0x6469_7374_616e_6e01; // "distann" | 0x01
const DISTANN_VEC_ID_DOMAIN_LOCAL: u64 = 0x6469_7374_616e_6e02; // "distann" | 0x02

/// murmur3 64-bit finalizer: full-avalanche bijective mixer.
fn fmix64(mut h: u64) -> u64 {
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    h
}

/// vec_id for a 16-byte ADR-063 canonical source identity (global mode).
pub(crate) fn vec_id_from_source_identity(identity: &[u8; 16]) -> u64 {
    let low = u64::from_le_bytes(identity[0..8].try_into().expect("identity low bytes"));
    let high = u64::from_le_bytes(identity[8..16].try_into().expect("identity high bytes"));
    fmix64(fmix64(low ^ DISTANN_VEC_ID_DOMAIN_GLOBAL).wrapping_add(high))
}

/// vec_id for a heap TID (local mode; single-node posture only).
pub(crate) fn vec_id_from_local_heap_tid(heap_tid: ItemPointer) -> u64 {
    let packed =
        (u64::from(heap_tid.block_number) << 16) | u64::from(heap_tid.offset_number);
    fmix64(packed ^ DISTANN_VEC_ID_DOMAIN_LOCAL)
}

#[cfg(test)]
mod tests {
    use super::{vec_id_from_local_heap_tid, vec_id_from_source_identity};
    use crate::storage::page::ItemPointer;

    #[test]
    fn distann_vec_id_is_deterministic_for_identical_identity() {
        // FR-076-AC-2 groundwork: same identity bytes -> same vec_id, always.
        let identity = *b"0123456789abcdef";
        assert_eq!(
            vec_id_from_source_identity(&identity),
            vec_id_from_source_identity(&identity)
        );
    }

    #[test]
    fn distann_vec_id_differs_for_distinct_identities() {
        let a = vec_id_from_source_identity(b"0123456789abcdef");
        let b = vec_id_from_source_identity(b"0123456789abcdeg");
        assert_ne!(a, b);
    }

    #[test]
    fn distann_vec_id_local_mode_is_deterministic_and_tid_sensitive() {
        let tid_a = ItemPointer {
            block_number: 7,
            offset_number: 3,
        };
        let tid_b = ItemPointer {
            block_number: 7,
            offset_number: 4,
        };
        assert_eq!(
            vec_id_from_local_heap_tid(tid_a),
            vec_id_from_local_heap_tid(tid_a)
        );
        assert_ne!(
            vec_id_from_local_heap_tid(tid_a),
            vec_id_from_local_heap_tid(tid_b)
        );
    }

    #[test]
    fn distann_vec_id_domains_do_not_alias() {
        // A local TID hash whose packed input equals the low half of a global
        // identity must not produce the same vec_id (distinct domain tags).
        let tid = ItemPointer {
            block_number: 0,
            offset_number: 1,
        };
        let mut identity = [0_u8; 16];
        identity[0] = 1;
        assert_ne!(
            vec_id_from_local_heap_tid(tid),
            vec_id_from_source_identity(&identity)
        );
    }

    #[test]
    fn distann_vec_id_values_are_pinned_against_accidental_hash_changes() {
        // These values are persisted on disk; changing the hash is a
        // format-version bump (NFR-016), not a refactor. If this test fails,
        // bump INDEX_FORMAT_V*_DISTANN instead of updating the expectations.
        assert_eq!(
            vec_id_from_source_identity(b"0123456789abcdef"),
            0xD5C7_F0DC_EA57_6A41_u64
        );
        assert_eq!(
            vec_id_from_local_heap_tid(ItemPointer {
                block_number: 7,
                offset_number: 3,
            }),
            0x035F_79F2_EC90_D5A7_u64
        );
    }
}
