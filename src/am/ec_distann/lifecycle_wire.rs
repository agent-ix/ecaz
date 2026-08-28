//! FR-082 canonical publication, abandonment, and retirement identities.
//!
//! These values cross transaction and node boundaries.  Every decoder is
//! bounded by `CanonicalDecoder`, rejects trailing bytes and non-canonical
//! flags/order, and verifies the complete digest chain before returning.

use super::canonical_wire::{
    domain_digest, is_rfc4122_v4_uuid, CanonicalDecoder, CanonicalEncoder,
    DISTANN_MAX_CANONICAL_BYTES,
};
use super::generation_descriptor::{
    decode_roster, roster_digest, validate_endpoint_identity, validate_roster, DistannBuildSpec,
    DistannGenerationDescriptor,
};
use super::manifest_v2::{
    DistannEpochFingerprint, DistannEpochManifestV2, DistannReadyReceipt, DistannSourceSnapshot,
    DISTANN_EPOCH_FINGERPRINT_BYTES, DISTANN_READY_RECEIPT_BYTES, DISTANN_READY_RECEIPT_MAX_BYTES,
};
use super::node_registry::validate_canonical_index_locator;

pub const DISTANN_BUILD_CANDIDATE_VERSION: u16 = 1;
pub const DISTANN_SUCCESSOR_ACTIVATION_VERSION: u16 = 1;
pub const DISTANN_ABANDON_BINDING_AUDIT_VERSION: u16 = 1;
pub const DISTANN_ABANDONED_BINDING_SET_VERSION: u16 = 1;
pub const DISTANN_RETIRE_DECISION_VERSION: u16 = 1;
pub const DISTANN_CANCEL_PUBLISH_AUDIT_VERSION: u16 = 1;

pub const DISTANN_BUILD_CANDIDATE_VERSION_OFFSET: usize = 0;
pub const DISTANN_BUILD_CANDIDATE_REGISTRATION_DIGEST_OFFSET: usize = 2;
pub const DISTANN_BUILD_CANDIDATE_BUILD_SPEC_LENGTH_OFFSET: usize = 34;
pub const DISTANN_BUILD_CANDIDATE_FIXED_PREFIX_BYTES: usize = 38;
pub const DISTANN_SUCCESSOR_ACTIVATION_VERSION_OFFSET: usize = 0;
pub const DISTANN_SUCCESSOR_ACTIVATION_COORDINATOR_UUID_OFFSET: usize = 2;
pub const DISTANN_SUCCESSOR_ACTIVATION_PREDECESSOR_PRESENT_OFFSET: usize = 18;
pub const DISTANN_SUCCESSOR_ACTIVATION_FIXED_PREFIX_BYTES: usize = 19;
pub const DISTANN_ABANDON_BINDING_AUDIT_VERSION_OFFSET: usize = 0;
pub const DISTANN_ABANDON_BINDING_AUDIT_COORDINATOR_UUID_OFFSET: usize = 2;
pub const DISTANN_ABANDON_BINDING_AUDIT_SUCCESSOR_BUILD_ID_OFFSET: usize = 18;
pub const DISTANN_ABANDON_BINDING_AUDIT_SUCCESSOR_EPOCH_OFFSET: usize = 34;
pub const DISTANN_ABANDON_BINDING_AUDIT_SUCCESSOR_FINGERPRINT_LENGTH_OFFSET: usize = 42;
pub const DISTANN_ABANDON_BINDING_AUDIT_FIXED_PREFIX_BYTES: usize = 46;
pub const DISTANN_ABANDONED_BINDING_SET_COUNT_OFFSET: usize = 0;
pub const DISTANN_ABANDONED_BINDING_SET_FIXED_PREFIX_BYTES: usize = 4;
pub const DISTANN_ABANDONED_BINDING_ENTRY_BYTES: usize = 36;
pub const DISTANN_RETIRE_DECISION_VERSION_OFFSET: usize = 0;
pub const DISTANN_RETIRE_DECISION_COORDINATOR_UUID_OFFSET: usize = 2;
pub const DISTANN_RETIRE_DECISION_TARGET_BUILD_ID_OFFSET: usize = 18;
pub const DISTANN_RETIRE_DECISION_EPOCH_OFFSET: usize = 34;
pub const DISTANN_RETIRE_DECISION_FINGERPRINT_LENGTH_OFFSET: usize = 42;
pub const DISTANN_RETIRE_DECISION_FIXED_PREFIX_BYTES: usize = 46;
pub const DISTANN_CANCEL_PUBLISH_AUDIT_VERSION_OFFSET: usize = 0;
pub const DISTANN_CANCEL_PUBLISH_AUDIT_COORDINATOR_UUID_OFFSET: usize = 2;
pub const DISTANN_CANCEL_PUBLISH_AUDIT_BUILD_ID_OFFSET: usize = 18;
pub const DISTANN_CANCEL_PUBLISH_AUDIT_EPOCH_OFFSET: usize = 34;
pub const DISTANN_CANCEL_PUBLISH_AUDIT_FINGERPRINT_LENGTH_OFFSET: usize = 42;
pub const DISTANN_CANCEL_PUBLISH_AUDIT_FIXED_PREFIX_BYTES: usize = 46;

const BUILD_CANDIDATE_DOMAIN: &[u8] = b"ec_distann_build_candidate_v1\0";
const READY_RECEIPT_SET_DOMAIN: &[u8] = b"ec_distann_ready_receipt_set_v1\0";
const SUCCESSOR_ACTIVATION_DOMAIN: &[u8] = b"ec_distann_successor_activation_v1\0";
const ABANDON_BINDING_AUDIT_DOMAIN: &[u8] = b"ec_distann_abandon_predecessor_binding_v1\0";
const ABANDONED_BINDING_SET_DOMAIN: &[u8] = b"ec_distann_abandoned_binding_set_v1\0";
const RETIRE_DECISION_DOMAIN: &[u8] = b"ec_distann_retire_decision_v1\0";
const CANCEL_PUBLISH_AUDIT_DOMAIN: &[u8] = b"ec_distann_cancel_epoch_publish_v1\0";
const DIGEST_BYTES: usize = 32;
const MAX_REASON_BYTES: usize = 1024;

fn predecessor_abandon_error(error: String) -> String {
    let detail = error
        .split_once(": ")
        .map_or(error.as_str(), |(_, detail)| detail);
    format!("EC_PREDECESSOR_ABANDON: {detail}")
}

fn publish_cancel_error(error: String) -> String {
    let detail = error
        .split_once(": ")
        .map_or(error.as_str(), |(_, detail)| detail);
    format!("EC_PUBLISH_CANCEL: {detail}")
}

fn validate_uuid(value: &[u8; 16], field: &str) -> Result<(), String> {
    if !is_rfc4122_v4_uuid(value) {
        return Err(format!("EC_EPOCH_STATE: {field} must be RFC 4122 v4"));
    }
    Ok(())
}

fn validate_fingerprint(
    fingerprint: &[u8; DISTANN_EPOCH_FINGERPRINT_BYTES],
    manifest_digest: Option<&[u8; DIGEST_BYTES]>,
    field: &str,
) -> Result<(), String> {
    let decoded = DistannEpochFingerprint::decode(fingerprint)
        .map_err(|error| format!("EC_EPOCH_STATE: invalid {field}: {error}"))?;
    if manifest_digest.is_some_and(|digest| decoded.manifest_digest() != *digest) {
        return Err(format!(
            "EC_EPOCH_STATE: {field} does not bind its manifest digest"
        ));
    }
    Ok(())
}

fn validate_nonempty_text(value: &str, maximum: usize, field: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > maximum || value.as_bytes().contains(&0) {
        return Err(format!(
            "EC_EPOCH_STATE: {field} must be nonempty canonical UTF-8 of at most {maximum} bytes"
        ));
    }
    Ok(())
}

fn digest_matches(
    domain: &[u8],
    bytes: &[u8],
    expected: &[u8; 32],
    field: &str,
) -> Result<(), String> {
    if domain_digest(domain, bytes) != *expected {
        return Err(format!("EC_PUBLISH_DIGEST: {field} digest mismatch"));
    }
    Ok(())
}

fn encode_ready_receipt_set(receipts: &[DistannReadyReceipt]) -> Result<Vec<u8>, String> {
    if receipts.is_empty() {
        return Err("EC_BUILD_INCOMPLETE: Ready receipt set is empty".to_owned());
    }
    let mut encoder = CanonicalEncoder::with_capacity(
        4 + receipts
            .len()
            .saturating_mul(DISTANN_READY_RECEIPT_MAX_BYTES + 4),
    );
    encoder.put_u32(
        u32::try_from(receipts.len())
            .map_err(|_| "EC_BUILD_INCOMPLETE: Ready receipt count exceeds u32".to_owned())?,
    );
    for receipt in receipts {
        encoder.put_len_prefixed(&receipt.encode()?)?;
    }
    encoder.finish()
}

fn decode_ready_receipt_set(input: &[u8]) -> Result<Vec<DistannReadyReceipt>, String> {
    let mut decoder = CanonicalDecoder::new(input, "Ready receipt set")?;
    let count = decoder.get_u32("Ready receipt set count")? as usize;
    let minimum_entry = DISTANN_READY_RECEIPT_BYTES + 4;
    let minimum_bytes = count.checked_mul(minimum_entry);
    if count == 0 || !matches!(minimum_bytes, Some(minimum) if minimum <= decoder.remaining()) {
        return Err("EC_BUILD_INCOMPLETE: invalid Ready receipt set count".to_owned());
    }
    let mut receipts = Vec::with_capacity(count);
    for _ in 0..count {
        receipts.push(DistannReadyReceipt::decode(
            decoder.get_len_prefixed("Ready receipt")?,
        )?);
    }
    decoder.finish("Ready receipt set")?;
    Ok(receipts)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistannBuildCandidateV1 {
    pub registration_digest: [u8; DIGEST_BYTES],
    pub build_spec: Vec<u8>,
    pub build_spec_digest: [u8; DIGEST_BYTES],
    pub generation_descriptor: Vec<u8>,
    pub generation_descriptor_digest: [u8; DIGEST_BYTES],
    pub source_snapshot: Vec<u8>,
    pub source_snapshot_digest: [u8; DIGEST_BYTES],
    pub ready_receipt_set: Vec<u8>,
    pub ready_receipt_set_digest: [u8; DIGEST_BYTES],
    pub epoch_manifest: Vec<u8>,
    pub manifest_digest: [u8; DIGEST_BYTES],
    pub epoch_fingerprint: [u8; DISTANN_EPOCH_FINGERPRINT_BYTES],
}

impl DistannBuildCandidateV1 {
    pub fn from_components(
        registration_digest: [u8; DIGEST_BYTES],
        build_spec: &DistannBuildSpec,
        descriptor: &DistannGenerationDescriptor,
        snapshot: &DistannSourceSnapshot,
        manifest: &DistannEpochManifestV2,
    ) -> Result<Self, String> {
        let build_spec_bytes = build_spec.encode()?;
        let descriptor_bytes = descriptor.encode()?;
        let snapshot_bytes = snapshot.encode()?;
        let receipt_set = encode_ready_receipt_set(&manifest.participant_receipts)?;
        let manifest_bytes = manifest.encode()?;
        let candidate = Self {
            registration_digest,
            build_spec_digest: build_spec.digest()?,
            build_spec: build_spec_bytes,
            generation_descriptor_digest: descriptor.digest()?,
            generation_descriptor: descriptor_bytes,
            source_snapshot_digest: snapshot.digest()?,
            source_snapshot: snapshot_bytes,
            ready_receipt_set_digest: domain_digest(READY_RECEIPT_SET_DOMAIN, &receipt_set),
            ready_receipt_set: receipt_set,
            manifest_digest: manifest.digest()?,
            epoch_manifest: manifest_bytes,
            epoch_fingerprint: *manifest.fingerprint()?.as_bytes(),
        };
        candidate.validate()?;
        Ok(candidate)
    }

    pub fn validate(&self) -> Result<(), String> {
        let build_spec = DistannBuildSpec::decode(&self.build_spec)?;
        if build_spec.digest()? != self.build_spec_digest {
            return Err("EC_PUBLISH_DIGEST: build specification digest mismatch".to_owned());
        }
        let descriptor = DistannGenerationDescriptor::decode(&self.generation_descriptor)?;
        if descriptor.digest()? != self.generation_descriptor_digest {
            return Err("EC_PUBLISH_DIGEST: generation descriptor digest mismatch".to_owned());
        }
        let snapshot = DistannSourceSnapshot::decode(&self.source_snapshot)?;
        if snapshot.digest()? != self.source_snapshot_digest {
            return Err("EC_PUBLISH_DIGEST: source snapshot digest mismatch".to_owned());
        }
        let receipts = decode_ready_receipt_set(&self.ready_receipt_set)?;
        digest_matches(
            READY_RECEIPT_SET_DOMAIN,
            &self.ready_receipt_set,
            &self.ready_receipt_set_digest,
            "Ready receipt set",
        )?;
        let manifest = DistannEpochManifestV2::decode(&self.epoch_manifest)?;
        if manifest.digest()? != self.manifest_digest {
            return Err("EC_PUBLISH_DIGEST: epoch manifest digest mismatch".to_owned());
        }
        validate_fingerprint(
            &self.epoch_fingerprint,
            Some(&self.manifest_digest),
            "candidate fingerprint",
        )?;
        if DistannEpochFingerprint::decode(&self.epoch_fingerprint)?.version() != manifest.version()
        {
            return Err(
                "EC_PUBLISH_DIGEST: candidate fingerprint version disagrees with manifest"
                    .to_owned(),
            );
        }
        let descriptor_cover_digest = descriptor
            .payload_cover()
            .map(|cover| cover.digest())
            .transpose()?;
        if manifest.payload_cover_descriptor_digest != descriptor_cover_digest {
            return Err(
                "EC_PUBLISH_DIGEST: manifest payload cover disagrees with descriptor".to_owned(),
            );
        }
        if build_spec.epoch != manifest.epoch
            || build_spec.build_id != manifest.build_id
            || build_spec.parent_fingerprint != manifest.parent_fingerprint
            || build_spec.source_snapshot_digest != self.source_snapshot_digest
            || build_spec.generation_descriptor_digest != self.generation_descriptor_digest
            || manifest.source_snapshot_digest != self.source_snapshot_digest
            || manifest.build_spec_digest != self.build_spec_digest
            || manifest.generation_descriptor_digest != self.generation_descriptor_digest
            || manifest.roster != descriptor.roster
            || manifest.row_schema_fingerprint != descriptor.row_schema.fingerprint()?
            || manifest.build_options.graph_degree != descriptor.graph_degree
            || manifest.build_options.options != build_spec.build_options
            || receipts != manifest.participant_receipts
            || build_spec.expected_global_count != manifest.global_record_count
            || build_spec.expected_global_graph_digest != manifest.global_graph_digest
            || build_spec.expected_global_row_tier_digest != manifest.global_row_tier_digest
            || build_spec.head_sample_digest != manifest.head_sample_digest
        {
            return Err("EC_PUBLISH_DIGEST: build candidate components disagree".to_owned());
        }
        manifest
            .codec_parameters
            .validate_artifact(&descriptor.codec_artifact)
            .map_err(|_| {
                "EC_PUBLISH_DIGEST: manifest codec parameters disagree with descriptor artifact"
                    .to_owned()
            })?;
        if build_spec.owner_expectations.len() != receipts.len()
            || build_spec
                .owner_expectations
                .iter()
                .zip(&receipts)
                .any(|(expected, receipt)| {
                    expected.node_id != receipt.node_id
                        || expected.expected_count != receipt.owned_record_count
                        || expected.expected_owner_digest != receipt.owner_stream_digest
                })
        {
            return Err("EC_PUBLISH_DIGEST: build candidate owner receipts disagree".to_owned());
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::with_capacity(
            232 + self.build_spec.len()
                + self.generation_descriptor.len()
                + self.source_snapshot.len()
                + self.ready_receipt_set.len()
                + self.epoch_manifest.len(),
        );
        encoder.put_u16(DISTANN_BUILD_CANDIDATE_VERSION);
        encoder.put_fixed(&self.registration_digest);
        encoder.put_len_prefixed(&self.build_spec)?;
        encoder.put_fixed(&self.build_spec_digest);
        encoder.put_len_prefixed(&self.generation_descriptor)?;
        encoder.put_fixed(&self.generation_descriptor_digest);
        encoder.put_len_prefixed(&self.source_snapshot)?;
        encoder.put_fixed(&self.source_snapshot_digest);
        encoder.put_len_prefixed(&self.ready_receipt_set)?;
        encoder.put_fixed(&self.ready_receipt_set_digest);
        encoder.put_len_prefixed(&self.epoch_manifest)?;
        encoder.put_fixed(&self.manifest_digest);
        encoder.put_fixed(&self.epoch_fingerprint);
        encoder.finish()
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "build candidate v1")?;
        let version = decoder.get_u16("build candidate version")?;
        if version != DISTANN_BUILD_CANDIDATE_VERSION {
            return Err(format!(
                "EC_PUBLISH_DIGEST: unsupported build candidate version {version}"
            ));
        }
        let candidate = Self {
            registration_digest: decoder.get_fixed("registration digest")?,
            build_spec: decoder.get_owned_bytes("build specification")?,
            build_spec_digest: decoder.get_fixed("build specification digest")?,
            generation_descriptor: decoder.get_owned_bytes("generation descriptor")?,
            generation_descriptor_digest: decoder.get_fixed("generation descriptor digest")?,
            source_snapshot: decoder.get_owned_bytes("source snapshot")?,
            source_snapshot_digest: decoder.get_fixed("source snapshot digest")?,
            ready_receipt_set: decoder.get_owned_bytes("Ready receipt set")?,
            ready_receipt_set_digest: decoder.get_fixed("Ready receipt set digest")?,
            epoch_manifest: decoder.get_owned_bytes("epoch manifest")?,
            manifest_digest: decoder.get_fixed("manifest digest")?,
            epoch_fingerprint: decoder.get_fixed("epoch fingerprint")?,
        };
        decoder.finish("build candidate v1")?;
        candidate.validate()?;
        Ok(candidate)
    }

    pub fn digest(&self) -> Result<[u8; DIGEST_BYTES], String> {
        Ok(domain_digest(BUILD_CANDIDATE_DOMAIN, &self.encode()?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistannPublishedEpochIdentity {
    pub build_id: [u8; 16],
    pub epoch: u64,
    pub fingerprint: [u8; DISTANN_EPOCH_FINGERPRINT_BYTES],
    pub manifest_digest: [u8; DIGEST_BYTES],
}

impl DistannPublishedEpochIdentity {
    fn validate(&self, field: &str) -> Result<(), String> {
        validate_uuid(&self.build_id, &format!("{field} build id"))?;
        if self.epoch == 0 {
            return Err(format!("EC_EPOCH_STATE: {field} epoch is zero"));
        }
        validate_fingerprint(
            &self.fingerprint,
            Some(&self.manifest_digest),
            &format!("{field} fingerprint"),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistannSuccessorActivationV1 {
    pub coordinator_logical_index_uuid: [u8; 16],
    pub predecessor: Option<DistannPublishedEpochIdentity>,
    pub successor: DistannPublishedEpochIdentity,
}

impl DistannSuccessorActivationV1 {
    pub fn validate(&self) -> Result<(), String> {
        validate_uuid(
            &self.coordinator_logical_index_uuid,
            "coordinator logical-index UUID",
        )?;
        if let Some(predecessor) = self.predecessor {
            predecessor.validate("predecessor")?;
            if predecessor == self.successor {
                return Err("EC_EPOCH_STATE: predecessor and successor are identical".to_owned());
            }
        }
        self.successor.validate("successor")
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::with_capacity(180);
        encoder.put_u16(DISTANN_SUCCESSOR_ACTIVATION_VERSION);
        encoder.put_fixed(&self.coordinator_logical_index_uuid);
        encoder.put_u8(u8::from(self.predecessor.is_some()));
        if let Some(predecessor) = self.predecessor {
            encoder.put_fixed(&predecessor.build_id);
            encoder.put_u64(predecessor.epoch);
            encoder.put_len_prefixed(&predecessor.fingerprint)?;
            encoder.put_fixed(&predecessor.manifest_digest);
        }
        encoder.put_fixed(&self.successor.build_id);
        encoder.put_u64(self.successor.epoch);
        encoder.put_len_prefixed(&self.successor.fingerprint)?;
        encoder.put_fixed(&self.successor.manifest_digest);
        encoder.finish()
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "successor activation v1")?;
        let version = decoder.get_u16("successor activation version")?;
        if version != DISTANN_SUCCESSOR_ACTIVATION_VERSION {
            return Err(format!(
                "EC_EPOCH_STATE: unsupported successor activation version {version}"
            ));
        }
        let coordinator_logical_index_uuid = decoder.get_fixed("coordinator logical-index UUID")?;
        let predecessor = match decoder.get_u8("predecessor-present flag")? {
            0 => None,
            1 => Some(DistannPublishedEpochIdentity {
                build_id: decoder.get_fixed("predecessor build id")?,
                epoch: decoder.get_u64("predecessor epoch")?,
                fingerprint: decoder
                    .get_len_prefixed("predecessor fingerprint")?
                    .try_into()
                    .map_err(|_| {
                        "EC_EPOCH_STATE: predecessor fingerprint length is not 34".to_owned()
                    })?,
                manifest_digest: decoder.get_fixed("predecessor manifest digest")?,
            }),
            other => {
                return Err(format!(
                    "EC_EPOCH_STATE: invalid predecessor-present flag {other}"
                ))
            }
        };
        let activation = Self {
            coordinator_logical_index_uuid,
            predecessor,
            successor: DistannPublishedEpochIdentity {
                build_id: decoder.get_fixed("successor build id")?,
                epoch: decoder.get_u64("successor epoch")?,
                fingerprint: decoder
                    .get_len_prefixed("successor fingerprint")?
                    .try_into()
                    .map_err(|_| {
                        "EC_EPOCH_STATE: successor fingerprint length is not 34".to_owned()
                    })?,
                manifest_digest: decoder.get_fixed("successor manifest digest")?,
            },
        };
        decoder.finish("successor activation v1")?;
        activation.validate()?;
        Ok(activation)
    }

    pub fn digest(&self) -> Result<[u8; DIGEST_BYTES], String> {
        Ok(domain_digest(SUCCESSOR_ACTIVATION_DOMAIN, &self.encode()?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistannCancelPublishAuditV1 {
    pub coordinator_logical_index_uuid: [u8; 16],
    pub build_id: [u8; 16],
    pub epoch: u64,
    pub epoch_fingerprint: [u8; DISTANN_EPOCH_FINGERPRINT_BYTES],
    pub manifest_digest: [u8; DIGEST_BYTES],
    pub decision_time_unix_micros: i64,
    pub caller_name: String,
    pub reason: String,
}

impl DistannCancelPublishAuditV1 {
    fn validate_inner(&self) -> Result<(), String> {
        validate_uuid(
            &self.coordinator_logical_index_uuid,
            "coordinator logical-index UUID",
        )?;
        validate_uuid(&self.build_id, "cancelled build id")?;
        if self.epoch == 0 {
            return Err("EC_PUBLISH_CANCEL: cancelled epoch must be nonzero".to_owned());
        }
        validate_fingerprint(
            &self.epoch_fingerprint,
            Some(&self.manifest_digest),
            "cancelled epoch fingerprint",
        )?;
        validate_nonempty_text(
            &self.caller_name,
            DISTANN_MAX_CANONICAL_BYTES,
            "cancellation caller",
        )?;
        validate_nonempty_text(&self.reason, MAX_REASON_BYTES, "cancellation reason")
    }

    pub fn validate(&self) -> Result<(), String> {
        self.validate_inner().map_err(publish_cancel_error)
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate_inner().map_err(publish_cancel_error)?;
        let mut encoder = CanonicalEncoder::with_capacity(
            160usize
                .saturating_add(self.caller_name.len())
                .saturating_add(self.reason.len()),
        );
        encoder.put_u16(DISTANN_CANCEL_PUBLISH_AUDIT_VERSION);
        encoder.put_fixed(&self.coordinator_logical_index_uuid);
        encoder.put_fixed(&self.build_id);
        encoder.put_u64(self.epoch);
        encoder.put_len_prefixed(&self.epoch_fingerprint)?;
        encoder.put_fixed(&self.manifest_digest);
        encoder.put_i64(self.decision_time_unix_micros);
        encoder.put_string(&self.caller_name)?;
        encoder.put_string(&self.reason)?;
        encoder.finish().map_err(publish_cancel_error)
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let result = (|| {
            let mut decoder = CanonicalDecoder::new(input, "cancel-publish audit v1")?;
            let version = decoder.get_u16("cancel-publish audit version")?;
            if version != DISTANN_CANCEL_PUBLISH_AUDIT_VERSION {
                return Err(format!(
                    "EC_PUBLISH_CANCEL: unsupported cancel-publish audit version {version}"
                ));
            }
            let audit = Self {
                coordinator_logical_index_uuid: decoder
                    .get_fixed("coordinator logical-index UUID")?,
                build_id: decoder.get_fixed("cancelled build id")?,
                epoch: decoder.get_u64("cancelled epoch")?,
                epoch_fingerprint: decoder
                    .get_len_prefixed("cancelled epoch fingerprint")?
                    .try_into()
                    .map_err(|_| {
                        "EC_PUBLISH_CANCEL: cancelled fingerprint length is not 34".to_owned()
                    })?,
                manifest_digest: decoder.get_fixed("cancelled manifest digest")?,
                decision_time_unix_micros: decoder.get_i64("cancellation timestamp")?,
                caller_name: decoder.get_string("cancellation caller")?,
                reason: decoder.get_string("cancellation reason")?,
            };
            decoder.finish("cancel-publish audit v1")?;
            audit.validate_inner()?;
            Ok(audit)
        })();
        result.map_err(publish_cancel_error)
    }

    pub fn digest(&self) -> Result<[u8; DIGEST_BYTES], String> {
        Ok(domain_digest(CANCEL_PUBLISH_AUDIT_DOMAIN, &self.encode()?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistannAbandonBindingAuditV1 {
    pub coordinator_logical_index_uuid: [u8; 16],
    pub successor_build_id: [u8; 16],
    pub successor_epoch: u64,
    pub successor_fingerprint: [u8; DISTANN_EPOCH_FINGERPRINT_BYTES],
    pub predecessor_build_id: [u8; 16],
    pub predecessor_epoch: u64,
    pub predecessor_fingerprint: [u8; DISTANN_EPOCH_FINGERPRINT_BYTES],
    pub predecessor_manifest_digest: [u8; DIGEST_BYTES],
    pub predecessor_roster_ordinal: u32,
    pub node_id: u32,
    pub participant_logical_index_uuid: [u8; 16],
    pub endpoint_identity: String,
    pub remote_index_regclass: String,
    pub successor_activation_digest: [u8; DIGEST_BYTES],
    pub decision_time_unix_micros: i64,
    pub caller_name: String,
    pub reason: String,
}

impl DistannAbandonBindingAuditV1 {
    pub fn validate(&self) -> Result<(), String> {
        self.validate_inner().map_err(predecessor_abandon_error)
    }

    fn validate_inner(&self) -> Result<(), String> {
        validate_uuid(
            &self.coordinator_logical_index_uuid,
            "coordinator logical-index UUID",
        )?;
        validate_uuid(&self.successor_build_id, "successor build id")?;
        validate_uuid(&self.predecessor_build_id, "predecessor build id")?;
        validate_uuid(
            &self.participant_logical_index_uuid,
            "participant logical-index UUID",
        )?;
        if self.successor_epoch == 0 || self.predecessor_epoch == 0 || self.node_id == 0 {
            return Err("EC_PREDECESSOR_ABANDON: epoch and node id must be nonzero".to_owned());
        }
        validate_fingerprint(&self.successor_fingerprint, None, "successor fingerprint")?;
        validate_fingerprint(
            &self.predecessor_fingerprint,
            Some(&self.predecessor_manifest_digest),
            "predecessor fingerprint",
        )?;
        validate_endpoint_identity(&self.endpoint_identity)?;
        validate_canonical_index_locator(&self.remote_index_regclass)?;
        validate_nonempty_text(
            &self.caller_name,
            DISTANN_MAX_CANONICAL_BYTES,
            "caller name",
        )?;
        validate_nonempty_text(&self.reason, MAX_REASON_BYTES, "abandon reason")
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.encode_inner().map_err(predecessor_abandon_error)
    }

    fn encode_inner(&self) -> Result<Vec<u8>, String> {
        self.validate_inner()?;
        let mut encoder = CanonicalEncoder::with_capacity(
            256 + self.endpoint_identity.len()
                + self.remote_index_regclass.len()
                + self.caller_name.len()
                + self.reason.len(),
        );
        encoder.put_u16(DISTANN_ABANDON_BINDING_AUDIT_VERSION);
        encoder.put_fixed(&self.coordinator_logical_index_uuid);
        encoder.put_fixed(&self.successor_build_id);
        encoder.put_u64(self.successor_epoch);
        encoder.put_len_prefixed(&self.successor_fingerprint)?;
        encoder.put_fixed(&self.predecessor_build_id);
        encoder.put_u64(self.predecessor_epoch);
        encoder.put_len_prefixed(&self.predecessor_fingerprint)?;
        encoder.put_fixed(&self.predecessor_manifest_digest);
        encoder.put_u32(self.predecessor_roster_ordinal);
        encoder.put_u32(self.node_id);
        encoder.put_fixed(&self.participant_logical_index_uuid);
        encoder.put_string(&self.endpoint_identity)?;
        encoder.put_string(&self.remote_index_regclass)?;
        encoder.put_fixed(&self.successor_activation_digest);
        encoder.put_i64(self.decision_time_unix_micros);
        encoder.put_string(&self.caller_name)?;
        encoder.put_string(&self.reason)?;
        encoder.finish()
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        Self::decode_inner(input).map_err(predecessor_abandon_error)
    }

    fn decode_inner(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "abandon-binding audit v1")?;
        let version = decoder.get_u16("abandon-binding audit version")?;
        if version != DISTANN_ABANDON_BINDING_AUDIT_VERSION {
            return Err(format!(
                "EC_PREDECESSOR_ABANDON: unsupported abandon-binding audit version {version}"
            ));
        }
        let audit = Self {
            coordinator_logical_index_uuid: decoder.get_fixed("coordinator logical-index UUID")?,
            successor_build_id: decoder.get_fixed("successor build id")?,
            successor_epoch: decoder.get_u64("successor epoch")?,
            successor_fingerprint: decoder
                .get_len_prefixed("successor fingerprint")?
                .try_into()
                .map_err(|_| {
                    "EC_PREDECESSOR_ABANDON: successor fingerprint length is not 34".to_owned()
                })?,
            predecessor_build_id: decoder.get_fixed("predecessor build id")?,
            predecessor_epoch: decoder.get_u64("predecessor epoch")?,
            predecessor_fingerprint: decoder
                .get_len_prefixed("predecessor fingerprint")?
                .try_into()
                .map_err(|_| {
                    "EC_PREDECESSOR_ABANDON: predecessor fingerprint length is not 34".to_owned()
                })?,
            predecessor_manifest_digest: decoder.get_fixed("predecessor manifest digest")?,
            predecessor_roster_ordinal: decoder.get_u32("predecessor roster ordinal")?,
            node_id: decoder.get_u32("predecessor node id")?,
            participant_logical_index_uuid: decoder.get_fixed("participant logical-index UUID")?,
            endpoint_identity: decoder.get_string("endpoint identity")?,
            remote_index_regclass: decoder.get_string("remote index locator")?,
            successor_activation_digest: decoder.get_fixed("successor activation digest")?,
            decision_time_unix_micros: decoder.get_i64("decision timestamp")?,
            caller_name: decoder.get_string("caller name")?,
            reason: decoder.get_string("abandon reason")?,
        };
        decoder.finish("abandon-binding audit v1")?;
        audit.validate_inner()?;
        Ok(audit)
    }

    pub fn digest(&self) -> Result<[u8; DIGEST_BYTES], String> {
        Ok(domain_digest(ABANDON_BINDING_AUDIT_DOMAIN, &self.encode()?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistannAbandonedBinding {
    pub roster_ordinal: u32,
    pub abandon_audit_digest: [u8; DIGEST_BYTES],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistannAbandonedBindingSetV1 {
    pub entries: Vec<DistannAbandonedBinding>,
}

impl DistannAbandonedBindingSetV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self
            .entries
            .windows(2)
            .any(|pair| pair[0].roster_ordinal >= pair[1].roster_ordinal)
        {
            return Err(
                "EC_EPOCH_STATE: abandoned bindings must be strictly ascending by ordinal"
                    .to_owned(),
            );
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::with_capacity(
            4 + self
                .entries
                .len()
                .saturating_mul(DISTANN_ABANDONED_BINDING_ENTRY_BYTES),
        );
        encoder.put_u32(
            u32::try_from(self.entries.len())
                .map_err(|_| "EC_EPOCH_STATE: abandoned-binding count exceeds u32".to_owned())?,
        );
        for entry in &self.entries {
            encoder.put_u32(entry.roster_ordinal);
            encoder.put_fixed(&entry.abandon_audit_digest);
        }
        encoder.finish()
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "abandoned-binding set v1")?;
        let count = decoder.get_u32("abandoned-binding count")? as usize;
        let encoded_bytes = count.checked_mul(DISTANN_ABANDONED_BINDING_ENTRY_BYTES);
        if !matches!(encoded_bytes, Some(bytes) if bytes == decoder.remaining()) {
            return Err("EC_EPOCH_STATE: invalid abandoned-binding count".to_owned());
        }
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            entries.push(DistannAbandonedBinding {
                roster_ordinal: decoder.get_u32("abandoned roster ordinal")?,
                abandon_audit_digest: decoder.get_fixed("abandon audit digest")?,
            });
        }
        decoder.finish("abandoned-binding set v1")?;
        let set = Self { entries };
        set.validate()?;
        Ok(set)
    }

    pub fn digest(&self) -> Result<[u8; DIGEST_BYTES], String> {
        Ok(domain_digest(ABANDONED_BINDING_SET_DOMAIN, &self.encode()?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistannRetireDecisionV1 {
    pub coordinator_logical_index_uuid: [u8; 16],
    pub target_build_id: [u8; 16],
    pub epoch: u64,
    pub target_fingerprint: [u8; DISTANN_EPOCH_FINGERPRINT_BYTES],
    pub target_manifest_digest: [u8; DIGEST_BYTES],
    pub target_roster_snapshot: Vec<u8>,
    pub roster_digest: [u8; DIGEST_BYTES],
    pub abandoned_bindings: DistannAbandonedBindingSetV1,
    pub forced: bool,
    pub overridden_in_flight_count: u64,
    pub decision_time_unix_micros: i64,
    pub caller_name: String,
    pub reason: String,
}

impl DistannRetireDecisionV1 {
    pub fn validate(&self) -> Result<(), String> {
        validate_uuid(
            &self.coordinator_logical_index_uuid,
            "coordinator logical-index UUID",
        )?;
        validate_uuid(&self.target_build_id, "target build id")?;
        if self.epoch == 0 {
            return Err("EC_EPOCH_STATE: retire target epoch is zero".to_owned());
        }
        validate_fingerprint(
            &self.target_fingerprint,
            Some(&self.target_manifest_digest),
            "retire target fingerprint",
        )?;
        let mut roster_decoder = CanonicalDecoder::new(
            &self.target_roster_snapshot,
            "retire target roster snapshot",
        )?;
        let roster = decode_roster(&mut roster_decoder)?;
        roster_decoder.finish("retire target roster snapshot")?;
        validate_roster(&roster)?;
        if roster_digest(&roster)? != self.roster_digest {
            return Err("EC_EPOCH_STATE: retire target roster digest mismatch".to_owned());
        }
        self.abandoned_bindings.validate()?;
        if self
            .abandoned_bindings
            .entries
            .iter()
            .any(|entry| entry.roster_ordinal as usize >= roster.len())
        {
            return Err("EC_EPOCH_STATE: abandoned ordinal is outside target roster".to_owned());
        }
        validate_nonempty_text(
            &self.caller_name,
            DISTANN_MAX_CANONICAL_BYTES,
            "caller name",
        )?;
        validate_nonempty_text(&self.reason, MAX_REASON_BYTES, "retire reason")?;
        if !self.forced && (self.overridden_in_flight_count != 0 || self.reason != "normal") {
            return Err(
                "EC_EPOCH_STATE: retire force/count/reason combination is invalid".to_owned(),
            );
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let abandoned = self.abandoned_bindings.encode()?;
        let mut encoder = CanonicalEncoder::with_capacity(
            190 + self.target_roster_snapshot.len()
                + abandoned.len()
                + self.caller_name.len()
                + self.reason.len(),
        );
        encoder.put_u16(DISTANN_RETIRE_DECISION_VERSION);
        encoder.put_fixed(&self.coordinator_logical_index_uuid);
        encoder.put_fixed(&self.target_build_id);
        encoder.put_u64(self.epoch);
        encoder.put_len_prefixed(&self.target_fingerprint)?;
        encoder.put_fixed(&self.target_manifest_digest);
        encoder.put_len_prefixed(&self.target_roster_snapshot)?;
        encoder.put_fixed(&self.roster_digest);
        encoder.put_fixed(&abandoned);
        encoder.put_u8(u8::from(self.forced));
        encoder.put_u64(self.overridden_in_flight_count);
        encoder.put_i64(self.decision_time_unix_micros);
        encoder.put_string(&self.caller_name)?;
        encoder.put_string(&self.reason)?;
        encoder.finish()
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "retire decision v1")?;
        let version = decoder.get_u16("retire decision version")?;
        if version != DISTANN_RETIRE_DECISION_VERSION {
            return Err(format!(
                "EC_EPOCH_STATE: unsupported retire decision version {version}"
            ));
        }
        let coordinator_logical_index_uuid = decoder.get_fixed("coordinator logical-index UUID")?;
        let target_build_id = decoder.get_fixed("target build id")?;
        let epoch = decoder.get_u64("retire target epoch")?;
        let target_fingerprint = decoder
            .get_len_prefixed("retire target fingerprint")?
            .try_into()
            .map_err(|_| "EC_EPOCH_STATE: retire target fingerprint length is not 34".to_owned())?;
        let target_manifest_digest = decoder.get_fixed("retire target manifest digest")?;
        let target_roster_snapshot = decoder.get_owned_bytes("retire target roster snapshot")?;
        let roster_digest = decoder.get_fixed("retire target roster digest")?;
        let abandoned_count = decoder.get_u32("abandoned-binding count")? as usize;
        let abandoned_bytes_len = abandoned_count
            .checked_mul(DISTANN_ABANDONED_BINDING_ENTRY_BYTES)
            .and_then(|bytes| bytes.checked_add(4))
            .ok_or_else(|| "EC_EPOCH_STATE: abandoned-binding byte count overflow".to_owned())?;
        if abandoned_bytes_len > decoder.remaining() {
            return Err("EC_EPOCH_STATE: invalid abandoned-binding count".to_owned());
        }
        let mut abandoned_bytes = Vec::with_capacity(abandoned_bytes_len);
        abandoned_bytes.extend_from_slice(&(abandoned_count as u32).to_le_bytes());
        abandoned_bytes.extend_from_slice(
            decoder.get_bytes(abandoned_bytes_len - 4, "abandoned-binding entries")?,
        );
        let abandoned_bindings = DistannAbandonedBindingSetV1::decode(&abandoned_bytes)?;
        let forced = match decoder.get_u8("retire forced flag")? {
            0 => false,
            1 => true,
            other => {
                return Err(format!(
                    "EC_EPOCH_STATE: invalid retire forced flag {other}"
                ))
            }
        };
        let decision = Self {
            coordinator_logical_index_uuid,
            target_build_id,
            epoch,
            target_fingerprint,
            target_manifest_digest,
            target_roster_snapshot,
            roster_digest,
            abandoned_bindings,
            forced,
            overridden_in_flight_count: decoder.get_u64("overridden in-flight count")?,
            decision_time_unix_micros: decoder.get_i64("retire decision timestamp")?,
            caller_name: decoder.get_string("retire caller name")?,
            reason: decoder.get_string("retire reason")?,
        };
        decoder.finish("retire decision v1")?;
        decision.validate()?;
        Ok(decision)
    }

    pub fn digest(&self) -> Result<[u8; DIGEST_BYTES], String> {
        Ok(domain_digest(RETIRE_DECISION_DOMAIN, &self.encode()?))
    }

    pub fn abandoned_binding_set_bytes(&self) -> Result<Vec<u8>, String> {
        self.abandoned_bindings.encode()
    }

    pub fn abandoned_binding_set_digest(&self) -> Result<[u8; DIGEST_BYTES], String> {
        self.abandoned_bindings.digest()
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::super::canonical_wire::sample_rfc4122_v4_uuid;
    use super::super::generation_descriptor::{
        encode_roster, sample_generation_descriptor, DistannBuildOptions, DistannOwnerExpectation,
    };
    use super::super::manifest_v2::{
        sample_manifest_v2, DistannReadyReceiptPayloadSidecar, DistannSourceSnapshot,
    };
    use super::*;

    fn sample_snapshot() -> DistannSourceSnapshot {
        DistannSourceSnapshot {
            system_identifier: 0x0102_0304_0506_0708,
            database_name: "ecaz".to_owned(),
            xmin_full: 100,
            xmax_full: 200,
            curcid: 3,
            xip: vec![101, 103, 107],
            subxip: vec![109, 113],
            suboverflowed: false,
            taken_during_recovery: true,
        }
    }

    fn sample_candidate() -> DistannBuildCandidateV1 {
        sample_candidate_with_cover(false)
    }

    fn sample_candidate_with_cover(covered: bool) -> DistannBuildCandidateV1 {
        let snapshot = sample_snapshot();
        let mut descriptor = sample_generation_descriptor();
        if covered {
            descriptor.payload_cover = super::super::payload_sidecar::resolve_payload_cover(
                &descriptor.row_schema,
                3,
                Some(&[1]),
            )
            .unwrap();
        }
        let mut manifest = sample_manifest_v2();
        manifest.source_snapshot_digest = snapshot.digest().unwrap();
        manifest.generation_descriptor_digest = descriptor.digest().unwrap();
        manifest.row_schema_fingerprint = descriptor.row_schema.fingerprint().unwrap();
        for receipt in &mut manifest.participant_receipts {
            receipt.generation_descriptor_digest = manifest.generation_descriptor_digest;
        }
        if let Some(cover) = descriptor.payload_cover() {
            manifest.payload_cover_descriptor_digest = Some(cover.digest().unwrap());
            for (index, receipt) in manifest.participant_receipts.iter_mut().enumerate() {
                receipt.payload_sidecar = Some(DistannReadyReceiptPayloadSidecar {
                    initial_content_digest: [0xD0 + index as u8; DIGEST_BYTES],
                    heap_bytes: 4096,
                    index_bytes: 8192,
                });
            }
            manifest.global_payload_sidecar_initial_content_digest = Some(
                DistannEpochManifestV2::payload_sidecar_global_initial_content_digest(
                    &manifest.participant_receipts,
                )
                .unwrap(),
            );
        }
        let build_spec = DistannBuildSpec {
            epoch: manifest.epoch,
            build_id: manifest.build_id,
            parent_fingerprint: manifest.parent_fingerprint.clone(),
            source_snapshot_digest: manifest.source_snapshot_digest,
            generation_descriptor_digest: manifest.generation_descriptor_digest,
            build_options: DistannBuildOptions {
                build_list_size: 100,
                alpha: 1.2,
                seed: 42,
                closure_epsilon: 0.3,
                head_index_cap: 4096,
                build_shards: 0,
                head_policy:
                    crate::am::ec_distann::generation_descriptor::DistannHeadPolicy::CurrentSampleGraph,
                training_query_count: 0,
                training_query_digest: [0; 32],
                head_sizing: None,
            },
            expected_global_count: manifest.global_record_count,
            expected_global_graph_digest: manifest.global_graph_digest,
            expected_global_row_tier_digest: manifest.global_row_tier_digest,
            head_sample_digest: manifest.head_sample_digest,
            owner_expectations: manifest
                .participant_receipts
                .iter()
                .map(|receipt| DistannOwnerExpectation {
                    node_id: receipt.node_id,
                    expected_count: receipt.owned_record_count,
                    expected_owner_digest: receipt.owner_stream_digest,
                })
                .collect(),
        };
        manifest.build_spec_digest = build_spec.digest().unwrap();
        for receipt in &mut manifest.participant_receipts {
            receipt.build_spec_digest = manifest.build_spec_digest;
        }
        DistannBuildCandidateV1::from_components(
            [0xA0; 32],
            &build_spec,
            &descriptor,
            &snapshot,
            &manifest,
        )
        .unwrap()
    }

    fn identity(marker: u8, epoch: u64) -> DistannPublishedEpochIdentity {
        let digest = [marker; 32];
        DistannPublishedEpochIdentity {
            build_id: sample_rfc4122_v4_uuid(marker),
            epoch,
            fingerprint: *DistannEpochFingerprint::from_manifest_digest(digest).as_bytes(),
            manifest_digest: digest,
        }
    }

    fn sample_activation() -> DistannSuccessorActivationV1 {
        DistannSuccessorActivationV1 {
            coordinator_logical_index_uuid: sample_rfc4122_v4_uuid(0xC0),
            predecessor: Some(identity(0x31, 7)),
            successor: identity(0x41, 8),
        }
    }

    fn sample_audit() -> DistannAbandonBindingAuditV1 {
        let activation = sample_activation();
        DistannAbandonBindingAuditV1 {
            coordinator_logical_index_uuid: activation.coordinator_logical_index_uuid,
            successor_build_id: activation.successor.build_id,
            successor_epoch: activation.successor.epoch,
            successor_fingerprint: activation.successor.fingerprint,
            predecessor_build_id: activation.predecessor.unwrap().build_id,
            predecessor_epoch: activation.predecessor.unwrap().epoch,
            predecessor_fingerprint: activation.predecessor.unwrap().fingerprint,
            predecessor_manifest_digest: activation.predecessor.unwrap().manifest_digest,
            predecessor_roster_ordinal: 1,
            node_id: 20,
            participant_logical_index_uuid: sample_rfc4122_v4_uuid(0x20),
            endpoint_identity: "cluster-a/node-20".to_owned(),
            remote_index_regclass: "public.distann_idx".to_owned(),
            successor_activation_digest: activation.digest().unwrap(),
            decision_time_unix_micros: 1_750_000_000_123_456,
            caller_name: "ecaz_operator".to_owned(),
            reason: "participant permanently unavailable".to_owned(),
        }
    }

    fn sample_cancel_audit() -> DistannCancelPublishAuditV1 {
        let successor = sample_activation().successor;
        DistannCancelPublishAuditV1 {
            coordinator_logical_index_uuid: sample_activation().coordinator_logical_index_uuid,
            build_id: successor.build_id,
            epoch: successor.epoch,
            epoch_fingerprint: successor.fingerprint,
            manifest_digest: successor.manifest_digest,
            decision_time_unix_micros: 1_750_000_000_654_321,
            caller_name: "ecaz_operator".to_owned(),
            reason: "successor participant permanently unavailable".to_owned(),
        }
    }

    fn sample_abandoned_set() -> DistannAbandonedBindingSetV1 {
        DistannAbandonedBindingSetV1 {
            entries: vec![
                DistannAbandonedBinding {
                    roster_ordinal: 0,
                    abandon_audit_digest: [0xA1; 32],
                },
                DistannAbandonedBinding {
                    roster_ordinal: 1,
                    abandon_audit_digest: sample_audit().digest().unwrap(),
                },
            ],
        }
    }

    fn sample_retire_decision() -> DistannRetireDecisionV1 {
        let target = identity(0x31, 7);
        let roster = sample_generation_descriptor().roster;
        let mut roster_encoder = CanonicalEncoder::with_capacity(128);
        encode_roster(&mut roster_encoder, &roster).unwrap();
        DistannRetireDecisionV1 {
            coordinator_logical_index_uuid: sample_rfc4122_v4_uuid(0xC0),
            target_build_id: target.build_id,
            epoch: target.epoch,
            target_fingerprint: target.fingerprint,
            target_manifest_digest: target.manifest_digest,
            target_roster_snapshot: roster_encoder.finish().unwrap(),
            roster_digest: roster_digest(&roster).unwrap(),
            abandoned_bindings: sample_abandoned_set(),
            forced: true,
            overridden_in_flight_count: 3,
            decision_time_unix_micros: 1_750_000_001_654_321,
            caller_name: "ecaz_operator".to_owned(),
            reason: "forced after audited drain timeout".to_owned(),
        }
    }

    fn assert_error_category<T: Debug>(result: Result<T, String>, category: &str) -> String {
        let error = result.expect_err("malformed lifecycle value must fail");
        assert!(
            error.starts_with(&format!("{category}:")),
            "expected {category}, got {error}"
        );
        error
    }

    fn append_trailing_byte(mut bytes: Vec<u8>) -> Vec<u8> {
        bytes.push(0xFF);
        bytes
    }

    #[test]
    fn lifecycle_formats_round_trip_and_bind_digests() {
        let candidate = sample_candidate();
        assert_eq!(
            DistannBuildCandidateV1::decode(&candidate.encode().unwrap()).unwrap(),
            candidate
        );
        let activation = sample_activation();
        assert_eq!(
            DistannSuccessorActivationV1::decode(&activation.encode().unwrap()).unwrap(),
            activation
        );
        let cancel_audit = sample_cancel_audit();
        assert_eq!(
            DistannCancelPublishAuditV1::decode(&cancel_audit.encode().unwrap()).unwrap(),
            cancel_audit
        );
        let audit = sample_audit();
        assert_eq!(
            DistannAbandonBindingAuditV1::decode(&audit.encode().unwrap()).unwrap(),
            audit
        );
        let set = sample_abandoned_set();
        assert_eq!(
            DistannAbandonedBindingSetV1::decode(&set.encode().unwrap()).unwrap(),
            set
        );
        let decision = sample_retire_decision();
        assert_eq!(
            DistannRetireDecisionV1::decode(&decision.encode().unwrap()).unwrap(),
            decision
        );
        assert_eq!(
            decision.abandoned_binding_set_bytes().unwrap(),
            set.encode().unwrap()
        );
    }

    #[test]
    fn ready_receipt_set_framing_accepts_bounded_v1_and_v2_entries() {
        let legacy = super::super::manifest_v2::sample_manifest_v2().participant_receipts;
        let legacy_bytes = encode_ready_receipt_set(&legacy).unwrap();
        assert_eq!(decode_ready_receipt_set(&legacy_bytes).unwrap(), legacy);

        let mut covered = super::super::manifest_v2::sample_manifest_v2().participant_receipts;
        for (index, receipt) in covered.iter_mut().enumerate() {
            receipt.payload_sidecar = Some(
                super::super::manifest_v2::DistannReadyReceiptPayloadSidecar {
                    initial_content_digest: [0xC0 + index as u8; DIGEST_BYTES],
                    heap_bytes: 4096,
                    index_bytes: 8192,
                },
            );
        }
        let covered_bytes = encode_ready_receipt_set(&covered).unwrap();
        assert!(covered_bytes.len() > legacy_bytes.len());
        assert_eq!(decode_ready_receipt_set(&covered_bytes).unwrap(), covered);
    }

    #[test]
    fn covered_build_candidate_binds_v3_descriptor_manifest_and_fingerprint() {
        let candidate = sample_candidate_with_cover(true);
        assert_eq!(candidate.epoch_fingerprint[..2], [3, 0]);
        assert_eq!(
            DistannBuildCandidateV1::decode(&candidate.encode().unwrap()).unwrap(),
            candidate
        );

        let mut wrong_fingerprint_version = candidate;
        wrong_fingerprint_version.epoch_fingerprint[..2].copy_from_slice(&2_u16.to_le_bytes());
        assert!(wrong_fingerprint_version.validate().is_err());
    }

    #[test]
    fn lifecycle_formats_reject_versions_flags_order_and_corruption() {
        let mut candidate_version = sample_candidate().encode().unwrap();
        candidate_version[0..2].copy_from_slice(&99_u16.to_le_bytes());
        assert!(DistannBuildCandidateV1::decode(&candidate_version).is_err());

        let mut activation_version = sample_activation().encode().unwrap();
        activation_version[0..2].copy_from_slice(&99_u16.to_le_bytes());
        assert!(DistannSuccessorActivationV1::decode(&activation_version).is_err());

        let mut audit_version = sample_audit().encode().unwrap();
        audit_version[0..2].copy_from_slice(&99_u16.to_le_bytes());
        assert!(DistannAbandonBindingAuditV1::decode(&audit_version).is_err());

        let mut cancel_version = sample_cancel_audit().encode().unwrap();
        cancel_version[0..2].copy_from_slice(&99_u16.to_le_bytes());
        assert!(DistannCancelPublishAuditV1::decode(&cancel_version).is_err());

        let mut decision_version = sample_retire_decision().encode().unwrap();
        decision_version[0..2].copy_from_slice(&99_u16.to_le_bytes());
        assert!(DistannRetireDecisionV1::decode(&decision_version).is_err());

        let mut activation = sample_activation().encode().unwrap();
        activation[DISTANN_SUCCESSOR_ACTIVATION_PREDECESSOR_PRESENT_OFFSET] = 2;
        assert!(DistannSuccessorActivationV1::decode(&activation).is_err());

        let mut set = sample_abandoned_set();
        set.entries.swap(0, 1);
        assert!(set.encode().is_err());

        let mut candidate = sample_candidate().encode().unwrap();
        candidate[DISTANN_BUILD_CANDIDATE_FIXED_PREFIX_BYTES] ^= 1;
        assert!(DistannBuildCandidateV1::decode(&candidate).is_err());
    }

    #[test]
    fn lifecycle_decoders_reject_flags_counts_order_duplicates_and_trailing_bytes() {
        let mut activation_flag = sample_activation().encode().unwrap();
        activation_flag[DISTANN_SUCCESSOR_ACTIVATION_PREDECESSOR_PRESENT_OFFSET] = 2;
        assert_error_category(
            DistannSuccessorActivationV1::decode(&activation_flag),
            "EC_EPOCH_STATE",
        );

        let decision = sample_retire_decision();
        let abandoned_bytes = decision.abandoned_binding_set_bytes().unwrap();
        let mut retire_flag = decision.encode().unwrap();
        let abandoned_offset = retire_flag
            .windows(abandoned_bytes.len())
            .position(|window| window == abandoned_bytes)
            .unwrap();
        retire_flag[abandoned_offset + abandoned_bytes.len()] = 2;
        assert_error_category(
            DistannRetireDecisionV1::decode(&retire_flag),
            "EC_EPOCH_STATE",
        );
        let mut retire_count = decision.encode().unwrap();
        retire_count[abandoned_offset..abandoned_offset + 4].copy_from_slice(&3_u32.to_le_bytes());
        assert!(DistannRetireDecisionV1::decode(&retire_count).is_err());

        let mut bad_count = sample_abandoned_set().encode().unwrap();
        bad_count[0..4].copy_from_slice(&3_u32.to_le_bytes());
        assert_error_category(
            DistannAbandonedBindingSetV1::decode(&bad_count),
            "EC_EPOCH_STATE",
        );

        let mut descending = sample_abandoned_set();
        descending.entries.swap(0, 1);
        assert_error_category(descending.validate(), "EC_EPOCH_STATE");
        let mut duplicate = sample_abandoned_set();
        duplicate.entries[1].roster_ordinal = duplicate.entries[0].roster_ordinal;
        assert_error_category(duplicate.validate(), "EC_EPOCH_STATE");

        let mut outside_roster = sample_retire_decision();
        outside_roster.abandoned_bindings.entries[1].roster_ordinal = u32::MAX;
        assert_error_category(outside_roster.validate(), "EC_EPOCH_STATE");

        assert!(DistannBuildCandidateV1::decode(&append_trailing_byte(
            sample_candidate().encode().unwrap(),
        ))
        .is_err());
        assert!(DistannSuccessorActivationV1::decode(&append_trailing_byte(
            sample_activation().encode().unwrap(),
        ))
        .is_err());
        assert_error_category(
            DistannAbandonBindingAuditV1::decode(&append_trailing_byte(
                sample_audit().encode().unwrap(),
            )),
            "EC_PREDECESSOR_ABANDON",
        );
        assert!(DistannAbandonedBindingSetV1::decode(&append_trailing_byte(
            sample_abandoned_set().encode().unwrap(),
        ))
        .is_err());
        assert!(DistannRetireDecisionV1::decode(&append_trailing_byte(
            sample_retire_decision().encode().unwrap(),
        ))
        .is_err());
    }

    #[test]
    fn retire_force_count_and_reason_combinations_follow_fr_082() {
        let mut normal = sample_retire_decision();
        normal.forced = false;
        normal.overridden_in_flight_count = 0;
        normal.reason = "normal".to_owned();
        assert_eq!(
            DistannRetireDecisionV1::decode(&normal.encode().unwrap()).unwrap(),
            normal
        );

        let mut normal_with_count = normal.clone();
        normal_with_count.overridden_in_flight_count = 1;
        assert_error_category(normal_with_count.validate(), "EC_EPOCH_STATE");

        let mut normal_with_custom_reason = normal.clone();
        normal_with_custom_reason.reason = "operator supplied".to_owned();
        assert_error_category(normal_with_custom_reason.validate(), "EC_EPOCH_STATE");

        let mut forced_named_normal = normal;
        forced_named_normal.forced = true;
        forced_named_normal.overridden_in_flight_count = 3;
        assert_eq!(
            DistannRetireDecisionV1::decode(&forced_named_normal.encode().unwrap()).unwrap(),
            forced_named_normal,
            "FR-082 does not reserve the caller-supplied forced reason text"
        );

        let mut forced_with_zero_count = sample_retire_decision();
        forced_with_zero_count.overridden_in_flight_count = 0;
        assert!(forced_with_zero_count.validate().is_ok());

        let mut empty_reason = sample_retire_decision();
        empty_reason.reason.clear();
        assert_error_category(empty_reason.validate(), "EC_EPOCH_STATE");
    }

    #[test]
    fn lifecycle_identity_and_fingerprint_validation_fails_closed() {
        let mut activation_uuid = sample_activation();
        activation_uuid.coordinator_logical_index_uuid = [0; 16];
        assert_error_category(activation_uuid.validate(), "EC_EPOCH_STATE");

        let mut activation_fingerprint = sample_activation();
        activation_fingerprint.successor.manifest_digest[0] ^= 1;
        assert_error_category(activation_fingerprint.validate(), "EC_EPOCH_STATE");

        let mut retire_uuid = sample_retire_decision();
        retire_uuid.target_build_id = [0; 16];
        assert_error_category(retire_uuid.validate(), "EC_EPOCH_STATE");

        let mut retire_fingerprint = sample_retire_decision();
        retire_fingerprint.target_manifest_digest[0] ^= 1;
        assert_error_category(retire_fingerprint.validate(), "EC_EPOCH_STATE");
    }

    #[test]
    fn abandon_audit_normalizes_every_malformed_codec_outcome() {
        for malformed in [
            {
                let mut audit = sample_audit();
                audit.coordinator_logical_index_uuid = [0; 16];
                audit
            },
            {
                let mut audit = sample_audit();
                audit.participant_logical_index_uuid = [0; 16];
                audit
            },
            {
                let mut audit = sample_audit();
                audit.successor_build_id = [0; 16];
                audit
            },
            {
                let mut audit = sample_audit();
                audit.predecessor_build_id = [0; 16];
                audit
            },
            {
                let mut audit = sample_audit();
                audit.successor_fingerprint[0] ^= 1;
                audit
            },
            {
                let mut audit = sample_audit();
                audit.predecessor_manifest_digest[0] ^= 1;
                audit
            },
            {
                let mut audit = sample_audit();
                audit.endpoint_identity = "secret endpoint".to_owned();
                audit
            },
            {
                let mut audit = sample_audit();
                audit.remote_index_regclass = "Distann.Index".to_owned();
                audit
            },
            {
                let mut audit = sample_audit();
                audit.node_id = 0;
                audit
            },
            {
                let mut audit = sample_audit();
                audit.caller_name.clear();
                audit
            },
            {
                let mut audit = sample_audit();
                audit.reason.clear();
                audit
            },
            {
                let mut audit = sample_audit();
                audit.reason = "x".repeat(MAX_REASON_BYTES + 1);
                audit
            },
        ] {
            assert_error_category(malformed.validate(), "EC_PREDECESSOR_ABANDON");
            assert_error_category(malformed.encode(), "EC_PREDECESSOR_ABANDON");
        }

        let encoded = sample_audit().encode().unwrap();
        assert_error_category(
            DistannAbandonBindingAuditV1::decode(&encoded[..encoded.len() - 1]),
            "EC_PREDECESSOR_ABANDON",
        );

        let mut unknown_version = encoded.clone();
        unknown_version[0..2].copy_from_slice(&99_u16.to_le_bytes());
        assert_error_category(
            DistannAbandonBindingAuditV1::decode(&unknown_version),
            "EC_PREDECESSOR_ABANDON",
        );

        let mut invalid_uuid = encoded.clone();
        invalid_uuid[DISTANN_ABANDON_BINDING_AUDIT_COORDINATOR_UUID_OFFSET
            ..DISTANN_ABANDON_BINDING_AUDIT_COORDINATOR_UUID_OFFSET + 16]
            .fill(0);
        assert_error_category(
            DistannAbandonBindingAuditV1::decode(&invalid_uuid),
            "EC_PREDECESSOR_ABANDON",
        );

        let mut invalid_fingerprint = encoded.clone();
        invalid_fingerprint[DISTANN_ABANDON_BINDING_AUDIT_FIXED_PREFIX_BYTES] ^= 1;
        assert_error_category(
            DistannAbandonBindingAuditV1::decode(&invalid_fingerprint),
            "EC_PREDECESSOR_ABANDON",
        );

        let mut invalid_fingerprint_length = encoded.clone();
        invalid_fingerprint_length
            [DISTANN_ABANDON_BINDING_AUDIT_SUCCESSOR_FINGERPRINT_LENGTH_OFFSET
                ..DISTANN_ABANDON_BINDING_AUDIT_SUCCESSOR_FINGERPRINT_LENGTH_OFFSET + 4]
            .copy_from_slice(&33_u32.to_le_bytes());
        assert_error_category(
            DistannAbandonBindingAuditV1::decode(&invalid_fingerprint_length),
            "EC_PREDECESSOR_ABANDON",
        );

        for text in [b"cluster-a/node-20".as_slice(), b"public.distann_idx"] {
            let offset = encoded
                .windows(text.len())
                .position(|window| window == text)
                .unwrap();
            let mut invalid_grammar = encoded.clone();
            invalid_grammar[offset] = b' ';
            assert_error_category(
                DistannAbandonBindingAuditV1::decode(&invalid_grammar),
                "EC_PREDECESSOR_ABANDON",
            );

            let mut invalid_utf8 = encoded.clone();
            invalid_utf8[offset] = 0xFF;
            assert_error_category(
                DistannAbandonBindingAuditV1::decode(&invalid_utf8),
                "EC_PREDECESSOR_ABANDON",
            );
        }
    }

    /// Regeneration helper for TC-050 lifecycle golden fixtures.
    #[test]
    #[ignore]
    fn emit_distann_lifecycle_golden_fixture_hex() {
        fn emit(name: &str, bytes: &[u8]) {
            println!("FIXTURE {name} {}", hex::encode(bytes));
        }
        emit(
            "distann_build_candidate_v1.hex",
            &sample_candidate().encode().unwrap(),
        );
        emit(
            "distann_successor_activation_v1.hex",
            &sample_activation().encode().unwrap(),
        );
        emit(
            "distann_abandon_binding_audit_v1.hex",
            &sample_audit().encode().unwrap(),
        );
        emit(
            "distann_cancel_publish_audit_v1.hex",
            &sample_cancel_audit().encode().unwrap(),
        );
        emit(
            "distann_abandoned_binding_set_v1.hex",
            &sample_abandoned_set().encode().unwrap(),
        );
        emit(
            "distann_retire_decision_v1.hex",
            &sample_retire_decision().encode().unwrap(),
        );
    }
}
