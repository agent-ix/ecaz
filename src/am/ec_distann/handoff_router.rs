//! Bounded coordinator-side routing of canonical graph entries to owners.

use super::handoff_wire::{
    DistannEncodedHandoffBatch, DistannHandoffEntry, DistannHandoffShape, DistannOwnerStreamHasher,
    DISTANN_HANDOFF_MAX_BYTES,
};
use super::placement::owning_node;

const EMPTY_BATCH_ENCODED_BYTES: usize = 141;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DistannStageAck {
    pub(crate) accepted_record_count: u64,
    pub(crate) cumulative_record_count: u64,
    pub(crate) cumulative_owner_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DistannOwnerRouteSummary {
    pub(crate) owner_ordinal: usize,
    pub(crate) acknowledged_batches: u64,
    pub(crate) record_count: u64,
    pub(crate) owner_stream_digest: [u8; 32],
}

#[derive(Clone)]
pub(crate) struct DistannHandoffRouteIdentity {
    pub(crate) epoch: u64,
    pub(crate) build_id: [u8; 16],
    pub(crate) build_spec_digest: [u8; 32],
    pub(crate) row_schema_fingerprint: [u8; 32],
    pub(crate) index_format_version: u16,
    pub(crate) neighbor_codec_kind: u8,
    pub(crate) placement_hash_version: u16,
}

struct OwnerBuffer {
    wire: DistannEncodedHandoffBatch,
    next_batch_seq: u64,
    cumulative_record_count: u64,
    cumulative_hasher: DistannOwnerStreamHasher,
    pending_hasher: DistannOwnerStreamHasher,
}

impl OwnerBuffer {
    fn new(identity: &DistannHandoffRouteIdentity, max_batch_bytes: usize) -> Result<Self, String> {
        let cumulative_hasher = DistannOwnerStreamHasher::new();
        Ok(Self {
            wire: DistannEncodedHandoffBatch::new(
                identity.epoch,
                identity.build_id,
                0,
                identity.build_spec_digest,
                identity.row_schema_fingerprint,
                identity.index_format_version,
                identity.neighbor_codec_kind,
                max_batch_bytes,
            )?,
            next_batch_seq: 0,
            cumulative_record_count: 0,
            pending_hasher: cumulative_hasher.clone(),
            cumulative_hasher,
        })
    }
}

pub(crate) struct DistannOwnerBatchRouter {
    identity: DistannHandoffRouteIdentity,
    shape: DistannHandoffShape,
    max_batch_bytes: usize,
    owners: Vec<OwnerBuffer>,
    previous_vec_id: Option<u64>,
}

impl DistannOwnerBatchRouter {
    pub(crate) fn new(
        identity: DistannHandoffRouteIdentity,
        shape: DistannHandoffShape,
        owner_count: usize,
    ) -> Result<Self, String> {
        Self::with_max_batch_bytes(identity, shape, owner_count, DISTANN_HANDOFF_MAX_BYTES)
    }

    fn with_max_batch_bytes(
        identity: DistannHandoffRouteIdentity,
        shape: DistannHandoffShape,
        owner_count: usize,
        max_batch_bytes: usize,
    ) -> Result<Self, String> {
        if owner_count == 0 {
            return Err("EC_NODE_DESCRIPTOR: owner roster is empty".to_owned());
        }
        if max_batch_bytes < EMPTY_BATCH_ENCODED_BYTES
            || max_batch_bytes > DISTANN_HANDOFF_MAX_BYTES
        {
            return Err("EC_HANDOFF_TOO_LARGE: invalid owner batch bound".to_owned());
        }
        let owners = (0..owner_count)
            .map(|_| OwnerBuffer::new(&identity, max_batch_bytes))
            .collect::<Result<Vec<_>, _>>()?;
        let router = Self {
            identity,
            shape,
            max_batch_bytes,
            owners,
            previous_vec_id: None,
        };
        router.check_retained_capacity()?;
        Ok(router)
    }

    pub(crate) fn push<F>(
        &mut self,
        entry: &DistannHandoffEntry,
        stage: &mut F,
    ) -> Result<(), String>
    where
        F: FnMut(usize, u64, &[u8; 32], &[u8]) -> Result<DistannStageAck, String>,
    {
        entry.validate(self.shape)?;
        if self
            .previous_vec_id
            .is_some_and(|previous| entry.vec_id <= previous)
        {
            return Err(
                "EC_HANDOFF_FORMAT: coordinator entries are not globally increasing".to_owned(),
            );
        }
        let owner = owning_node(
            entry.vec_id,
            self.owners.len(),
            self.identity.placement_hash_version,
        );
        let encoded_entry_bytes = entry.encoded_len(self.shape)?;
        if self.owners[owner].wire.is_finalized() {
            self.flush_owner(owner, stage)?;
        }
        if !self.owners[owner].wire.can_append(encoded_entry_bytes)? {
            if self.owners[owner].wire.entry_count() == 0 {
                return Err(
                    "EC_HANDOFF_TOO_LARGE: one complete owner entry exceeds the batch bound"
                        .to_owned(),
                );
            }
            self.flush_owner(owner, stage)?;
        }
        let owner_buffer = &mut self.owners[owner];
        let previous_wire_len = owner_buffer.wire.wire().len();
        let previous_entry_count = owner_buffer.wire.entry_count();
        let encoded_range = owner_buffer.wire.append_entry(entry, self.shape)?;
        let mut pending_hasher = owner_buffer.pending_hasher.clone();
        if let Err(error) =
            pending_hasher.update_encoded_entry(&owner_buffer.wire.wire()[encoded_range])
        {
            owner_buffer
                .wire
                .rollback_unfinalized_append(previous_wire_len, previous_entry_count);
            return Err(error);
        }
        owner_buffer.pending_hasher = pending_hasher;
        self.previous_vec_id = Some(entry.vec_id);
        self.check_retained_capacity()?;
        Ok(())
    }

    fn flush_owner<F>(&mut self, owner: usize, stage: &mut F) -> Result<(), String>
    where
        F: FnMut(usize, u64, &[u8; 32], &[u8]) -> Result<DistannStageAck, String>,
    {
        let batch_digest = self.owners[owner].wire.finalize()?;
        self.check_retained_capacity()?;
        let buffer = &self.owners[owner];
        let accepted_record_count = u64::from(buffer.wire.entry_count());
        let expected_cumulative_count = buffer
            .cumulative_record_count
            .checked_add(accepted_record_count)
            .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: owner record count overflow".to_owned())?;
        let expected_cumulative_digest = buffer.pending_hasher.digest();

        // Nothing in the owner state mutates until the exact acknowledgement
        // is validated. A transport failure therefore leaves this one batch
        // intact for an identical retry.
        let ack = stage(
            owner,
            buffer.next_batch_seq,
            &batch_digest,
            buffer.wire.wire(),
        )?;
        if ack.accepted_record_count != accepted_record_count
            || ack.cumulative_record_count != expected_cumulative_count
            || ack.cumulative_owner_digest != expected_cumulative_digest
        {
            return Err("EC_HANDOFF_DIGEST: participant acknowledgement mismatch".to_owned());
        }

        let buffer = &mut self.owners[owner];
        buffer.next_batch_seq = buffer
            .next_batch_seq
            .checked_add(1)
            .ok_or_else(|| "EC_BATCH_SEQUENCE: owner batch sequence overflow".to_owned())?;
        buffer.cumulative_record_count = expected_cumulative_count;
        buffer.cumulative_hasher = buffer.pending_hasher.clone();
        buffer.pending_hasher = buffer.cumulative_hasher.clone();
        buffer.wire.reset(buffer.next_batch_seq);
        self.check_retained_capacity()?;
        Ok(())
    }

    pub(crate) fn finish<F>(
        &mut self,
        stage: &mut F,
    ) -> Result<Vec<DistannOwnerRouteSummary>, String>
    where
        F: FnMut(usize, u64, &[u8; 32], &[u8]) -> Result<DistannStageAck, String>,
    {
        for owner in 0..self.owners.len() {
            if self.owners[owner].wire.entry_count() != 0
                || self.owners[owner].next_batch_seq == 0
                || self.owners[owner].wire.is_finalized()
            {
                self.flush_owner(owner, stage)?;
            }
        }
        Ok(self
            .owners
            .iter()
            .enumerate()
            .map(|(owner_ordinal, owner)| DistannOwnerRouteSummary {
                owner_ordinal,
                acknowledged_batches: owner.next_batch_seq,
                record_count: owner.cumulative_record_count,
                owner_stream_digest: owner.cumulative_hasher.digest(),
            })
            .collect())
    }

    fn check_retained_capacity(&self) -> Result<(), String> {
        let maximum = self
            .max_batch_bytes
            .checked_mul(self.owners.len())
            .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: roster memory bound overflow".to_owned())?;
        if self.retained_wire_capacity() > maximum
            || self
                .owners
                .iter()
                .any(|owner| owner.wire.retained_capacity() > self.max_batch_bytes)
        {
            return Err(
                "EC_HANDOFF_TOO_LARGE: retained owner buffers exceed roster bound".to_owned(),
            );
        }
        Ok(())
    }

    pub(crate) fn retained_wire_capacity(&self) -> usize {
        self.owners
            .iter()
            .map(|owner| owner.wire.retained_capacity())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::am::ec_distann::generation_descriptor::{
        DISTANN_PHYSICAL_INDEX_FORMAT_VERSION, DISTANN_PLACEMENT_HASH_VERSION,
    };
    use crate::am::ec_distann::handoff_wire::{
        owner_stream_digest, sample_handoff_entry, DistannHandoffBatch,
    };

    fn identity() -> DistannHandoffRouteIdentity {
        DistannHandoffRouteIdentity {
            epoch: 7,
            build_id: super::super::canonical_wire::sample_rfc4122_v4_uuid(0xb7),
            build_spec_digest: [1; 32],
            row_schema_fingerprint: [2; 32],
            index_format_version: DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
            neighbor_codec_kind: 1,
            placement_hash_version: DISTANN_PLACEMENT_HASH_VERSION,
        }
    }

    fn shape() -> DistannHandoffShape {
        DistannHandoffShape {
            code_stride: 2,
            graph_degree: 4,
            non_dropped_attribute_count: 3,
        }
    }

    #[test]
    fn router_bounds_batches_routes_owners_and_sends_empty_sequence_zero() {
        let mut router =
            DistannOwnerBatchRouter::with_max_batch_bytes(identity(), shape(), 4, 250).unwrap();
        let mut owner_zero = Vec::new();
        let mut owner_one = Vec::new();
        for vec_id in 1..10_000 {
            match owning_node(vec_id, 4, DISTANN_PLACEMENT_HASH_VERSION) {
                0 if owner_zero.len() < 3 => owner_zero.push(vec_id),
                1 if owner_one.is_empty() => owner_one.push(vec_id),
                _ => {}
            }
            if owner_zero.len() == 3 && owner_one.len() == 1 {
                break;
            }
        }
        let mut vec_ids = owner_zero;
        vec_ids.extend(owner_one);
        vec_ids.sort_unstable();
        let entries = vec_ids
            .into_iter()
            .map(sample_handoff_entry)
            .collect::<Vec<_>>();
        let mut received = BTreeMap::<usize, Vec<DistannHandoffEntry>>::new();
        let mut calls = Vec::new();
        let mut stage = |owner: usize, seq: u64, digest: &[u8; 32], encoded: &[u8]| {
            assert!(encoded.len() <= 250);
            let batch = DistannHandoffBatch::decode(encoded, shape()).unwrap();
            assert_eq!(batch.batch_seq, seq);
            assert_eq!(batch.digest(shape()).unwrap(), *digest);
            received
                .entry(owner)
                .or_default()
                .extend(batch.entries.clone());
            calls.push((owner, seq, batch.entries.len()));
            let owner_entries = &received[&owner];
            Ok(DistannStageAck {
                accepted_record_count: batch.entries.len() as u64,
                cumulative_record_count: owner_entries.len() as u64,
                cumulative_owner_digest: owner_stream_digest(owner_entries, shape()).unwrap(),
            })
        };
        for entry in entries.clone() {
            router.push(&entry, &mut stage).unwrap();
        }
        let summaries = router.finish(&mut stage).unwrap();
        assert_eq!(summaries.len(), 4);
        assert!(calls.iter().any(|(_, _, count)| *count == 0));
        assert!(calls.iter().any(|(_, seq, _)| *seq > 0));
        for entry in entries {
            let owner = owning_node(entry.vec_id, 4, DISTANN_PLACEMENT_HASH_VERSION);
            assert!(received[&owner]
                .iter()
                .any(|received_entry| received_entry.vec_id == entry.vec_id));
        }
    }

    #[test]
    fn router_retains_exact_unacknowledged_batch_after_failure() {
        let mut router = DistannOwnerBatchRouter::new(identity(), shape(), 1).unwrap();
        router
            .push(&sample_handoff_entry(7), &mut |_, _, _, _| unreachable!())
            .unwrap();
        let mut first_encoded = None;
        let failure = router.finish(&mut |_, _, _, encoded| {
            first_encoded = Some(encoded.to_vec());
            Err("transport unavailable".to_owned())
        });
        assert_eq!(failure.unwrap_err(), "transport unavailable");
        let mut retried = None;
        let summaries = router
            .finish(&mut |_, _, _, encoded| {
                retried = Some(encoded.to_vec());
                let batch = DistannHandoffBatch::decode(encoded, shape()).unwrap();
                Ok(DistannStageAck {
                    accepted_record_count: 1,
                    cumulative_record_count: 1,
                    cumulative_owner_digest: owner_stream_digest(&batch.entries, shape()).unwrap(),
                })
            })
            .unwrap();
        assert_eq!(retried, first_encoded);
        assert_eq!(summaries[0].record_count, 1);
    }

    fn entry_for_batch_len(vec_id: u64, target_batch_len: usize) -> DistannHandoffEntry {
        let mut entry = sample_handoff_entry(vec_id);
        let current_batch_len = EMPTY_BATCH_ENCODED_BYTES + 4 + entry.encoded_len(shape()).unwrap();
        let growth = target_batch_len
            .checked_sub(current_batch_len)
            .expect("target accommodates the sample entry");
        let value = &mut entry.row_values[0];
        value.resize(value.len() + growth, 0x5a);
        assert_eq!(
            EMPTY_BATCH_ENCODED_BYTES + 4 + entry.encoded_len(shape()).unwrap(),
            target_batch_len
        );
        entry
    }

    #[test]
    fn router_enforces_real_multi_owner_capacity_minus_exact_and_plus_one() {
        let owner_count = 3;
        let mut vec_ids = vec![None; owner_count];
        for vec_id in 1..10_000 {
            let owner = owning_node(vec_id, owner_count, DISTANN_PLACEMENT_HASH_VERSION);
            vec_ids[owner].get_or_insert(vec_id);
            if vec_ids.iter().all(Option::is_some) {
                break;
            }
        }
        let mut ordered = vec_ids
            .into_iter()
            .enumerate()
            .map(|(owner, vec_id)| (vec_id.unwrap(), owner))
            .collect::<Vec<_>>();
        ordered.sort_unstable();

        let mut router = DistannOwnerBatchRouter::new(identity(), shape(), owner_count).unwrap();
        let mut expected_digests = vec![DistannOwnerStreamHasher::new().digest(); owner_count];
        let targets = [
            DISTANN_HANDOFF_MAX_BYTES - 1,
            DISTANN_HANDOFF_MAX_BYTES,
            DISTANN_HANDOFF_MAX_BYTES + 1,
        ];
        for ((vec_id, owner), target) in ordered.iter().copied().zip(targets) {
            let entry = entry_for_batch_len(vec_id, target);
            if target <= DISTANN_HANDOFF_MAX_BYTES {
                expected_digests[owner] =
                    owner_stream_digest(std::slice::from_ref(&entry), shape()).unwrap();
                router
                    .push(&entry, &mut |_, _, _, _| unreachable!())
                    .unwrap();
            } else {
                let before_capacity = router.retained_wire_capacity();
                let error = router
                    .push(&entry, &mut |_, _, _, _| unreachable!())
                    .unwrap_err();
                assert!(error.contains("one complete owner entry exceeds"));
                assert_eq!(router.owners[owner].wire.entry_count(), 0);
                assert_eq!(router.owners[owner].wire.wire().len(), 109);
                assert_eq!(router.retained_wire_capacity(), before_capacity);
            }
            assert!(router.retained_wire_capacity() <= owner_count * DISTANN_HANDOFF_MAX_BYTES);
            assert!(router
                .owners
                .iter()
                .all(|buffer| buffer.wire.retained_capacity() <= DISTANN_HANDOFF_MAX_BYTES));
        }

        let mut lengths = BTreeMap::new();
        router
            .finish(&mut |owner, seq, digest, encoded| {
                assert_eq!(seq, 0);
                assert_eq!(
                    *digest,
                    DistannHandoffBatch::verified_digest(encoded).unwrap()
                );
                lengths.insert(owner, encoded.len());
                let accepted = u64::from(encoded.len() != EMPTY_BATCH_ENCODED_BYTES);
                Ok(DistannStageAck {
                    accepted_record_count: accepted,
                    cumulative_record_count: accepted,
                    cumulative_owner_digest: expected_digests[owner],
                })
            })
            .unwrap();
        let observed = ordered
            .iter()
            .map(|(_, owner)| lengths[owner])
            .collect::<Vec<_>>();
        assert_eq!(
            observed,
            vec![
                DISTANN_HANDOFF_MAX_BYTES - 1,
                DISTANN_HANDOFF_MAX_BYTES,
                EMPTY_BATCH_ENCODED_BYTES,
            ]
        );
    }
}
