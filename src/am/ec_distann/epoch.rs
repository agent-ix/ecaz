//! FR-082 epoch fingerprint — the M2 validation subset (Task 164).
//!
//! Every `ec_distann_expand_nodes` call carries the coordinator's epoch
//! fingerprint; the owning node validates it before any read and raises a
//! retriable epoch-mismatch error on disagreement (FR-079 first bullet,
//! FR-082-AC-2). The fingerprint attests to the epoch's **immutable identity**
//! — roster + ordering, placement hash version, on-disk format version, and
//! the build-time record set — NOT the mutable delta/tombstone state
//! (FR-082 / ADR-085 D10).
//!
//! This module is the M2 subset: it computes and compares the fingerprint. The
//! full Building → Published → Retired lifecycle, publish hand-off, and
//! retention gate are M3 (FR-082 in full); they will attach this fingerprint
//! to the persisted epoch manifest. Keeping the fingerprint pure and versioned
//! here lets M2's two-node read path validate epoch identity without the M3
//! machinery.

// Consumed by the `ec_distann_expand_nodes` endpoint + RemoteNodeExpander in
// the following M2 slices. Allow dead_code until they land on this branch.
#![allow(dead_code)]

/// Fingerprint algorithm version. Bump on any change to `compute` (a change is
/// an epoch rebuild, not a migration — NFR-016).
pub(crate) const DISTANN_EPOCH_FINGERPRINT_V1: u16 = 1;

/// Fingerprint width in bytes (128-bit: two mixed 64-bit lanes, so accidental
/// collision across distinct epochs is negligible at research scales).
pub(crate) const DISTANN_EPOCH_FINGERPRINT_BYTES: usize = 16;

/// The immutable-identity inputs a coordinator and every data node hash
/// identically to agree on an epoch. Everything here is frozen at build /
/// publish; nothing mutable (tombstone flags, delta buffer) participates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DistannEpochIdentity {
    /// Epoch number (monotonic; distinguishes successive publishes).
    pub(crate) epoch: u64,
    /// On-disk format version (`INDEX_FORMAT_V1_DISTANN`).
    pub(crate) format_version: u16,
    /// Placement hash version (`DISTANN_PLACEMENT_HASH_V1`).
    pub(crate) placement_hash_version: u16,
    /// Roster node ids in placement order (the ordering `owning_node` indexes).
    pub(crate) roster_node_ids: Vec<u32>,
    /// Build-time record-set identity: fields that pin the deterministic build
    /// (FR-077) so two nodes only agree when they serve the same graph.
    pub(crate) node_count: u64,
    pub(crate) dimensions: u16,
    pub(crate) graph_degree: u16,
    pub(crate) neighbor_codec_kind: u8,
    pub(crate) build_seed: u64,
}

/// FNV-1a-style streaming mixer with a per-lane offset basis, so the two lanes
/// diverge and the 128-bit output is not just a doubled 64-bit hash.
struct FingerprintHasher {
    lane0: u64,
    lane1: u64,
}

impl FingerprintHasher {
    const FNV_OFFSET_0: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_OFFSET_1: u64 = 0x9e37_79b9_7f4a_7c15;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    fn new() -> Self {
        Self {
            lane0: Self::FNV_OFFSET_0,
            lane1: Self::FNV_OFFSET_1,
        }
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.lane0 ^= u64::from(byte);
            self.lane0 = self.lane0.wrapping_mul(Self::FNV_PRIME);
            // Lane 1 folds in the byte position implicitly via lane0 feedback,
            // so the two lanes stay independent.
            self.lane1 = self.lane1.wrapping_add(self.lane0);
            self.lane1 ^= self.lane1 >> 29;
            self.lane1 = self.lane1.wrapping_mul(Self::FNV_PRIME);
        }
    }

    fn write_u16(&mut self, v: u16) {
        self.write(&v.to_le_bytes());
    }
    fn write_u32(&mut self, v: u32) {
        self.write(&v.to_le_bytes());
    }
    fn write_u64(&mut self, v: u64) {
        self.write(&v.to_le_bytes());
    }

    fn finish(mut self) -> [u8; DISTANN_EPOCH_FINGERPRINT_BYTES] {
        // Final avalanche on each lane.
        self.lane0 ^= self.lane0 >> 33;
        self.lane0 = self.lane0.wrapping_mul(0xff51_afd7_ed55_8ccd);
        self.lane0 ^= self.lane0 >> 33;
        self.lane1 ^= self.lane1 >> 33;
        self.lane1 = self.lane1.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
        self.lane1 ^= self.lane1 >> 33;
        let mut out = [0_u8; DISTANN_EPOCH_FINGERPRINT_BYTES];
        out[0..8].copy_from_slice(&self.lane0.to_le_bytes());
        out[8..16].copy_from_slice(&self.lane1.to_le_bytes());
        out
    }
}

/// Compute the epoch fingerprint. Domain-separated by algorithm version so a
/// future version never collides with v1 for the same identity.
pub(crate) fn compute_epoch_fingerprint(
    identity: &DistannEpochIdentity,
    algorithm_version: u16,
) -> [u8; DISTANN_EPOCH_FINGERPRINT_BYTES] {
    let mut hasher = FingerprintHasher::new();
    hasher.write_u16(algorithm_version);
    hasher.write_u64(identity.epoch);
    hasher.write_u16(identity.format_version);
    hasher.write_u16(identity.placement_hash_version);
    // Roster length then each id in order — length-prefixed so [1,2] and [12]
    // never hash alike.
    hasher.write_u32(identity.roster_node_ids.len() as u32);
    for &node_id in &identity.roster_node_ids {
        hasher.write_u32(node_id);
    }
    hasher.write_u64(identity.node_count);
    hasher.write_u16(identity.dimensions);
    hasher.write_u16(identity.graph_degree);
    hasher.write(&[identity.neighbor_codec_kind]);
    hasher.write_u64(identity.build_seed);
    hasher.finish()
}

/// Constant-time-ish equality check for a received fingerprint against the
/// node's locally-computed one. (Not a security boundary — a consistency
/// check — but avoids early-exit ambiguity.)
pub(crate) fn fingerprints_match(
    a: &[u8; DISTANN_EPOCH_FINGERPRINT_BYTES],
    b: &[u8; DISTANN_EPOCH_FINGERPRINT_BYTES],
) -> bool {
    let mut diff = 0_u8;
    for i in 0..DISTANN_EPOCH_FINGERPRINT_BYTES {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

/// Parse a wire `bytea` fingerprint (FR-079 `epoch_fingerprint bytea`) into the
/// fixed-width array, rejecting the wrong length.
pub(crate) fn fingerprint_from_bytes(
    bytes: &[u8],
) -> Result<[u8; DISTANN_EPOCH_FINGERPRINT_BYTES], String> {
    <[u8; DISTANN_EPOCH_FINGERPRINT_BYTES]>::try_from(bytes).map_err(|_| {
        format!(
            "ec_distann epoch fingerprint must be {DISTANN_EPOCH_FINGERPRINT_BYTES} bytes, got {}",
            bytes.len()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_identity() -> DistannEpochIdentity {
        DistannEpochIdentity {
            epoch: 7,
            format_version: 1,
            placement_hash_version: 1,
            roster_node_ids: vec![10, 20, 30],
            node_count: 100_000,
            dimensions: 1536,
            graph_degree: 32,
            neighbor_codec_kind: 2,
            build_seed: 42,
        }
    }

    // FR-082-AC-2 groundwork: identical identity => identical fingerprint,
    // wherever computed (coordinator and every data node agree).
    #[test]
    fn fingerprint_is_deterministic() {
        let id = base_identity();
        let a = compute_epoch_fingerprint(&id, DISTANN_EPOCH_FINGERPRINT_V1);
        let b = compute_epoch_fingerprint(&id, DISTANN_EPOCH_FINGERPRINT_V1);
        assert_eq!(a, b);
        assert!(fingerprints_match(&a, &b));
    }

    // Every identity field participates: changing any one changes the digest.
    #[test]
    fn fingerprint_is_sensitive_to_every_field() {
        let base = compute_epoch_fingerprint(&base_identity(), DISTANN_EPOCH_FINGERPRINT_V1);

        let mutators: Vec<Box<dyn Fn(&mut DistannEpochIdentity)>> = vec![
            Box::new(|i| i.epoch += 1),
            Box::new(|i| i.format_version += 1),
            Box::new(|i| i.placement_hash_version += 1),
            Box::new(|i| i.roster_node_ids.push(40)),
            Box::new(|i| i.roster_node_ids[0] = 11),
            Box::new(|i| i.node_count += 1),
            Box::new(|i| i.dimensions += 1),
            Box::new(|i| i.graph_degree += 1),
            Box::new(|i| i.neighbor_codec_kind += 1),
            Box::new(|i| i.build_seed += 1),
        ];
        for (index, mutate) in mutators.iter().enumerate() {
            let mut id = base_identity();
            mutate(&mut id);
            let changed = compute_epoch_fingerprint(&id, DISTANN_EPOCH_FINGERPRINT_V1);
            assert_ne!(base, changed, "mutator {index} did not change the fingerprint");
            assert!(!fingerprints_match(&base, &changed));
        }
    }

    // Roster ORDER matters (placement indexes into the ordering).
    #[test]
    fn fingerprint_is_roster_order_sensitive() {
        let mut a = base_identity();
        a.roster_node_ids = vec![10, 20, 30];
        let mut b = base_identity();
        b.roster_node_ids = vec![30, 20, 10];
        assert_ne!(
            compute_epoch_fingerprint(&a, DISTANN_EPOCH_FINGERPRINT_V1),
            compute_epoch_fingerprint(&b, DISTANN_EPOCH_FINGERPRINT_V1)
        );
    }

    // Length-prefixing prevents roster-concatenation aliasing ([1,2] vs [12]).
    #[test]
    fn fingerprint_length_prefix_prevents_aliasing() {
        let mut a = base_identity();
        a.roster_node_ids = vec![1, 2];
        let mut b = base_identity();
        b.roster_node_ids = vec![12];
        assert_ne!(
            compute_epoch_fingerprint(&a, DISTANN_EPOCH_FINGERPRINT_V1),
            compute_epoch_fingerprint(&b, DISTANN_EPOCH_FINGERPRINT_V1)
        );
    }

    #[test]
    fn fingerprint_bytes_round_trip() {
        let id = base_identity();
        let fp = compute_epoch_fingerprint(&id, DISTANN_EPOCH_FINGERPRINT_V1);
        let parsed = fingerprint_from_bytes(&fp).expect("valid width");
        assert_eq!(parsed, fp);
        assert!(fingerprint_from_bytes(&[0_u8; 8]).is_err());
        assert!(fingerprint_from_bytes(&[0_u8; 17]).is_err());
    }
}
