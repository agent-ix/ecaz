//! Bounded coordinator-side routing of canonical graph entries to owners.

use super::handoff_wire::{
    DistannHandoffBatch, DistannHandoffEntry, DistannHandoffShape, DistannOwnerStreamHasher,
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
    entries: Vec<DistannHandoffEntry>,
    encoded_entry_section_bytes: usize,
    next_batch_seq: u64,
    cumulative_record_count: u64,
    cumulative_hasher: DistannOwnerStreamHasher,
}

impl OwnerBuffer {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            encoded_entry_section_bytes: 0,
            next_batch_seq: 0,
            cumulative_record_count: 0,
            cumulative_hasher: DistannOwnerStreamHasher::new(),
        }
    }

    fn encoded_batch_bytes_with(&self, encoded_entry_bytes: usize) -> Result<usize, String> {
        EMPTY_BATCH_ENCODED_BYTES
            .checked_add(self.encoded_entry_section_bytes)
            .and_then(|bytes| bytes.checked_add(4))
            .and_then(|bytes| bytes.checked_add(encoded_entry_bytes))
            .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: owner batch length overflow".to_owned())
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
        Ok(Self {
            identity,
            shape,
            max_batch_bytes,
            owners: (0..owner_count).map(|_| OwnerBuffer::new()).collect(),
            previous_vec_id: None,
        })
    }

    pub(crate) fn push<F>(
        &mut self,
        entry: DistannHandoffEntry,
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
        let encoded_entry_bytes = entry.encode(self.shape)?.len();
        if self.owners[owner].encoded_batch_bytes_with(encoded_entry_bytes)? > self.max_batch_bytes
        {
            if self.owners[owner].entries.is_empty() {
                return Err(
                    "EC_HANDOFF_TOO_LARGE: one complete owner entry exceeds the batch bound"
                        .to_owned(),
                );
            }
            self.flush_owner(owner, stage)?;
        }
        let owner_buffer = &mut self.owners[owner];
        owner_buffer.encoded_entry_section_bytes = owner_buffer
            .encoded_entry_section_bytes
            .checked_add(4 + encoded_entry_bytes)
            .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: owner entry section overflow".to_owned())?;
        owner_buffer.entries.push(entry);
        self.previous_vec_id = Some(owner_buffer.entries.last().unwrap().vec_id);
        Ok(())
    }

    fn batch(&self, owner: usize) -> DistannHandoffBatch {
        let buffer = &self.owners[owner];
        DistannHandoffBatch {
            epoch: self.identity.epoch,
            build_id: self.identity.build_id,
            batch_seq: buffer.next_batch_seq,
            build_spec_digest: self.identity.build_spec_digest,
            row_schema_fingerprint: self.identity.row_schema_fingerprint,
            index_format_version: self.identity.index_format_version,
            neighbor_codec_kind: self.identity.neighbor_codec_kind,
            entries: buffer.entries.clone(),
        }
    }

    fn flush_owner<F>(&mut self, owner: usize, stage: &mut F) -> Result<(), String>
    where
        F: FnMut(usize, u64, &[u8; 32], &[u8]) -> Result<DistannStageAck, String>,
    {
        let batch = self.batch(owner);
        let encoded = batch.encode(self.shape)?;
        if encoded.len() > self.max_batch_bytes {
            return Err("EC_HANDOFF_TOO_LARGE: encoded owner batch exceeds its bound".to_owned());
        }
        let batch_digest = batch.digest(self.shape)?;
        let buffer = &self.owners[owner];
        let mut expected_hasher = buffer.cumulative_hasher.clone();
        for entry in &buffer.entries {
            expected_hasher.update_entry(entry, self.shape)?;
        }
        let accepted_record_count = u64::try_from(buffer.entries.len())
            .map_err(|_| "EC_HANDOFF_TOO_LARGE: owner batch entry count exceeds u64".to_owned())?;
        let expected_cumulative_count = buffer
            .cumulative_record_count
            .checked_add(accepted_record_count)
            .ok_or_else(|| "EC_HANDOFF_TOO_LARGE: owner record count overflow".to_owned())?;
        let expected_cumulative_digest = expected_hasher.digest();

        // Nothing in the owner state mutates until the exact acknowledgement
        // is validated. A transport failure therefore leaves this one batch
        // intact for an identical retry.
        let ack = stage(owner, buffer.next_batch_seq, &batch_digest, &encoded)?;
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
        buffer.cumulative_hasher = expected_hasher;
        buffer.entries.clear();
        buffer.encoded_entry_section_bytes = 0;
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
            if !self.owners[owner].entries.is_empty() || self.owners[owner].next_batch_seq == 0 {
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
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::am::ec_distann::generation_descriptor::{
        DISTANN_PHYSICAL_INDEX_FORMAT_VERSION, DISTANN_PLACEMENT_HASH_VERSION,
    };
    use crate::am::ec_distann::handoff_wire::{owner_stream_digest, sample_handoff_entry};

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
            router.push(entry, &mut stage).unwrap();
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
            .push(sample_handoff_entry(7), &mut |_, _, _, _| unreachable!())
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
}
