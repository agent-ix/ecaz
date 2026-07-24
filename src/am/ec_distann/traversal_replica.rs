//! FR-084 coordinator traversal replica identity and canonical content digest.
//!
//! The replica is a derived, coordinator-local performance object. Owner
//! generations remain authoritative and payload materialization remains
//! owner-side.

use sha2::{Digest, Sha256};

use super::canonical_wire::is_rfc4122_v4_uuid;

const REPLICA_CONTENT_DOMAIN: &[u8] = b"ec_distann_traversal_replica_v1\0";
pub(crate) const TRAVERSAL_REPLICA_FORMAT_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TraversalReplicaState {
    Building,
    Ready,
    Stale,
    Retiring,
}

impl TraversalReplicaState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Building => "Building",
            Self::Ready => "Ready",
            Self::Stale => "Stale",
            Self::Retiring => "Retiring",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "Building" => Ok(Self::Building),
            "Ready" => Ok(Self::Ready),
            "Stale" => Ok(Self::Stale),
            "Retiring" => Ok(Self::Retiring),
            other => Err(format!(
                "EC_REPLICA_STATE: unknown traversal replica state {other:?}"
            )),
        }
    }

    pub(crate) fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Building, Self::Ready)
                | (Self::Ready, Self::Stale)
                | (Self::Ready | Self::Stale, Self::Retiring)
        ) || self == next
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TraversalReplicaIdentity {
    pub(crate) logical_index_uuid: [u8; 16],
    pub(crate) build_id: [u8; 16],
    pub(crate) epoch_fingerprint: [u8; 34],
    pub(crate) generation_descriptor_digest: [u8; 32],
    pub(crate) dimensions: u16,
    pub(crate) graph_degree: u16,
    pub(crate) neighbor_codec_kind: u8,
    pub(crate) owner_count: u32,
    pub(crate) expected_record_count: u64,
}

impl TraversalReplicaIdentity {
    fn validate(self) -> Result<(), String> {
        if self.logical_index_uuid == [0; 16] {
            return Err("EC_REPLICA_IDENTITY: logical index UUID is zero".to_owned());
        }
        if !is_rfc4122_v4_uuid(&self.build_id) {
            return Err("EC_REPLICA_IDENTITY: build id is not an RFC 4122 v4 UUID".to_owned());
        }
        if self.epoch_fingerprint[..2] != 2_u16.to_le_bytes() {
            return Err(
                "EC_REPLICA_IDENTITY: epoch fingerprint is not canonical version 2".to_owned(),
            );
        }
        if self.dimensions == 0
            || self.graph_degree == 0
            || self.owner_count == 0
            || self.expected_record_count == 0
        {
            return Err("EC_REPLICA_IDENTITY: replica shape contains zero".to_owned());
        }
        Ok(())
    }
}

pub(crate) struct TraversalReplicaContentHasher {
    hasher: Sha256,
    identity: TraversalReplicaIdentity,
    rows: u64,
    last_key: Option<(u32, u64)>,
}

impl TraversalReplicaContentHasher {
    pub(crate) fn new(identity: TraversalReplicaIdentity) -> Result<Self, String> {
        identity.validate()?;
        let mut hasher = Sha256::new();
        hasher.update(REPLICA_CONTENT_DOMAIN);
        hasher.update(TRAVERSAL_REPLICA_FORMAT_VERSION.to_le_bytes());
        hasher.update(identity.logical_index_uuid);
        hasher.update(identity.build_id);
        hasher.update(identity.epoch_fingerprint);
        hasher.update(identity.generation_descriptor_digest);
        hasher.update(identity.dimensions.to_le_bytes());
        hasher.update(identity.graph_degree.to_le_bytes());
        hasher.update([identity.neighbor_codec_kind]);
        hasher.update(identity.owner_count.to_le_bytes());
        hasher.update(identity.expected_record_count.to_le_bytes());
        Ok(Self {
            hasher,
            identity,
            rows: 0,
            last_key: None,
        })
    }

    pub(crate) fn update_row(
        &mut self,
        owner_ordinal: u32,
        vec_id: u64,
        graph_record: &[u8],
        exact_vector: &[u8],
    ) -> Result<(), String> {
        if owner_ordinal >= self.identity.owner_count {
            return Err(format!(
                "EC_REPLICA_CONTENT: owner ordinal {owner_ordinal} is outside {} owners",
                self.identity.owner_count
            ));
        }
        let key = (owner_ordinal, vec_id);
        if self.last_key.is_some_and(|last| key <= last) {
            return Err(
                "EC_REPLICA_CONTENT: rows are duplicated or not in canonical order".to_owned(),
            );
        }
        if graph_record.is_empty() {
            return Err("EC_REPLICA_CONTENT: graph record is empty".to_owned());
        }
        let expected_vector_bytes = usize::from(self.identity.dimensions)
            .checked_mul(std::mem::size_of::<f32>())
            .ok_or_else(|| "EC_REPLICA_CONTENT: exact-vector length overflow".to_owned())?;
        if exact_vector.len() != expected_vector_bytes {
            return Err(format!(
                "EC_REPLICA_CONTENT: exact vector is {} bytes, expected {expected_vector_bytes}",
                exact_vector.len()
            ));
        }
        if exact_vector
            .chunks_exact(4)
            .any(|word| !f32::from_le_bytes(word.try_into().expect("four-byte chunk")).is_finite())
        {
            return Err("EC_REPLICA_CONTENT: exact vector contains a non-finite value".to_owned());
        }
        let graph_len = u32::try_from(graph_record.len())
            .map_err(|_| "EC_REPLICA_CONTENT: graph record exceeds u32".to_owned())?;
        let vector_len = u32::try_from(exact_vector.len())
            .map_err(|_| "EC_REPLICA_CONTENT: exact vector exceeds u32".to_owned())?;
        self.hasher.update(owner_ordinal.to_le_bytes());
        self.hasher.update(vec_id.to_le_bytes());
        self.hasher.update(graph_len.to_le_bytes());
        self.hasher.update(graph_record);
        self.hasher.update(vector_len.to_le_bytes());
        self.hasher.update(exact_vector);
        self.last_key = Some(key);
        self.rows = self
            .rows
            .checked_add(1)
            .ok_or_else(|| "EC_REPLICA_CONTENT: record count overflow".to_owned())?;
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<[u8; 32], String> {
        if self.rows != self.identity.expected_record_count {
            return Err(format!(
                "EC_REPLICA_CONTENT: copied {} records, expected {}",
                self.rows, self.identity.expected_record_count
            ));
        }
        Ok(self.hasher.finalize().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(records: u64) -> TraversalReplicaIdentity {
        let mut build_id = [0x22; 16];
        build_id[6] = 0x42;
        build_id[8] = 0x82;
        let mut fingerprint = [0x33; 34];
        fingerprint[..2].copy_from_slice(&2_u16.to_le_bytes());
        TraversalReplicaIdentity {
            logical_index_uuid: [0x11; 16],
            build_id,
            epoch_fingerprint: fingerprint,
            generation_descriptor_digest: [0x44; 32],
            dimensions: 2,
            graph_degree: 32,
            neighbor_codec_kind: 3,
            owner_count: 2,
            expected_record_count: records,
        }
    }

    fn vector(left: f32, right: f32) -> Vec<u8> {
        [left.to_le_bytes(), right.to_le_bytes()].concat()
    }

    #[test]
    fn content_digest_is_deterministic_and_identity_bound() {
        let build = || {
            let mut hasher = TraversalReplicaContentHasher::new(identity(2)).unwrap();
            hasher
                .update_row(0, 7, b"graph-a", &vector(1.0, -2.0))
                .unwrap();
            hasher
                .update_row(1, 9, b"graph-b", &vector(3.0, 4.0))
                .unwrap();
            hasher.finish().unwrap()
        };
        assert_eq!(build(), build());
        let mut changed = identity(2);
        changed.generation_descriptor_digest[0] ^= 1;
        let mut hasher = TraversalReplicaContentHasher::new(changed).unwrap();
        hasher
            .update_row(0, 7, b"graph-a", &vector(1.0, -2.0))
            .unwrap();
        hasher
            .update_row(1, 9, b"graph-b", &vector(3.0, 4.0))
            .unwrap();
        assert_ne!(build(), hasher.finish().unwrap());
    }

    #[test]
    fn content_digest_rejects_duplicate_order_shape_and_cardinality() {
        let mut hasher = TraversalReplicaContentHasher::new(identity(2)).unwrap();
        hasher
            .update_row(0, 7, b"graph-a", &vector(1.0, 2.0))
            .unwrap();
        assert!(hasher
            .update_row(0, 7, b"graph-a", &vector(1.0, 2.0))
            .is_err());
        assert!(hasher.finish().is_err());

        let mut hasher = TraversalReplicaContentHasher::new(identity(1)).unwrap();
        assert!(hasher
            .update_row(2, 7, b"graph-a", &vector(1.0, 2.0))
            .is_err());
        assert!(hasher
            .update_row(0, 7, b"graph-a", &vector(f32::NAN, 2.0))
            .is_err());
        assert!(hasher.update_row(0, 7, b"graph-a", &[0; 4]).is_err());
    }

    #[test]
    fn state_machine_allows_only_contract_transitions() {
        assert!(TraversalReplicaState::Building.can_transition_to(TraversalReplicaState::Ready));
        assert!(TraversalReplicaState::Ready.can_transition_to(TraversalReplicaState::Stale));
        assert!(TraversalReplicaState::Ready.can_transition_to(TraversalReplicaState::Retiring));
        assert!(TraversalReplicaState::Stale.can_transition_to(TraversalReplicaState::Retiring));
        assert!(!TraversalReplicaState::Building.can_transition_to(TraversalReplicaState::Stale));
        assert!(!TraversalReplicaState::Stale.can_transition_to(TraversalReplicaState::Ready));
    }
}
