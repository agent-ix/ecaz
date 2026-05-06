#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireEncodedManifestBundle {
    pub(super) epoch_manifest: Vec<u8>,
    pub(super) object_manifest: Vec<u8>,
    pub(super) placement_directory: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireEncodedPublishBundle {
    pub(super) manifests: SpireEncodedManifestBundle,
    pub(super) root_control_state: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpirePublishedManifestLocators {
    pub(super) epoch_manifest_tid: ItemPointer,
    pub(super) object_manifest_tid: ItemPointer,
    pub(super) placement_directory_tid: ItemPointer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpirePublishObjectWriteEvidence {
    pub(super) pid: u64,
    pub(super) object_tid: ItemPointer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpirePublishPlacementWriteEvidence {
    pub(super) pid: u64,
    pub(super) placement_tid: ItemPointer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpirePublishStage {
    WritingObjects,
    WritingPlacements,
    WritingManifest,
    Validating,
    PublishingActiveEpoch,
    Published,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpirePublishFailed {
    pub(super) stage: SpirePublishStage,
    pub(super) error: String,
}

impl SpirePublishFailed {
    fn at(stage: SpirePublishStage, error: String) -> Self {
        Self { stage, error }
    }

    fn into_error(self) -> String {
        format!(
            "ec_spire publish coordinator {:?} failed: {}",
            self.stage, self.error
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SpirePublishCoordinatorInput<'a> {
    pub(super) epoch_manifest: &'a SpireEpochManifest,
    pub(super) object_manifest: &'a SpireObjectManifest,
    pub(super) placement_directory: &'a SpirePlacementDirectory,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SpirePublishWritingObjects<'a> {
    input: SpirePublishCoordinatorInput<'a>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SpirePublishWritingPlacements<'a> {
    input: SpirePublishCoordinatorInput<'a>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SpirePublishWritingManifest<'a> {
    input: SpirePublishCoordinatorInput<'a>,
}

#[derive(Debug, Clone)]
pub(super) struct SpirePublishValidating<'a> {
    input: SpirePublishCoordinatorInput<'a>,
    manifests: SpireEncodedManifestBundle,
}

#[derive(Debug, Clone)]
pub(super) struct SpirePublishPublishingActiveEpoch<'a> {
    input: SpirePublishCoordinatorInput<'a>,
    manifests: SpireEncodedManifestBundle,
    snapshot: SpireValidatedEpochSnapshot<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpirePublishPublished {
    pub(super) root_control_state: SpireRootControlState,
    pub(super) bundle: SpireEncodedPublishBundle,
}

impl<'a> SpirePublishWritingObjects<'a> {
    pub(super) fn new(input: SpirePublishCoordinatorInput<'a>) -> Self {
        Self { input }
    }

    pub(super) fn objects_written(
        self,
        evidence: &[SpirePublishObjectWriteEvidence],
    ) -> Result<SpirePublishWritingPlacements<'a>, SpirePublishFailed> {
        validate_object_write_evidence(self.input.placement_directory, evidence)
            .map_err(|error| SpirePublishFailed::at(SpirePublishStage::WritingObjects, error))?;
        Ok(SpirePublishWritingPlacements { input: self.input })
    }
}

impl<'a> SpirePublishWritingPlacements<'a> {
    pub(super) fn placements_written(
        self,
        evidence: &[SpirePublishPlacementWriteEvidence],
    ) -> Result<SpirePublishWritingManifest<'a>, SpirePublishFailed> {
        validate_placement_write_evidence(self.input.object_manifest, evidence)
            .map_err(|error| SpirePublishFailed::at(SpirePublishStage::WritingPlacements, error))?;
        Ok(SpirePublishWritingManifest { input: self.input })
    }
}

impl<'a> SpirePublishWritingManifest<'a> {
    pub(super) fn write_manifests(self) -> Result<SpirePublishValidating<'a>, SpirePublishFailed> {
        let manifests = SpireEncodedManifestBundle {
            epoch_manifest: self.input.epoch_manifest.encode().map_err(|error| {
                SpirePublishFailed::at(SpirePublishStage::WritingManifest, error)
            })?,
            object_manifest: self.input.object_manifest.encode().map_err(|error| {
                SpirePublishFailed::at(SpirePublishStage::WritingManifest, error)
            })?,
            placement_directory: self.input.placement_directory.encode().map_err(|error| {
                SpirePublishFailed::at(SpirePublishStage::WritingManifest, error)
            })?,
        };
        Ok(SpirePublishValidating {
            input: self.input,
            manifests,
        })
    }
}

impl<'a> SpirePublishValidating<'a> {
    pub(super) fn validate(
        self,
    ) -> Result<SpirePublishPublishingActiveEpoch<'a>, SpirePublishFailed> {
        let snapshot = SpireValidatedEpochSnapshot::new(
            self.input.epoch_manifest,
            self.input.object_manifest,
            self.input.placement_directory,
        )
        .map_err(|error| SpirePublishFailed::at(SpirePublishStage::Validating, error))?;
        Ok(SpirePublishPublishingActiveEpoch {
            input: self.input,
            manifests: self.manifests,
            snapshot,
        })
    }
}

impl SpirePublishPublishingActiveEpoch<'_> {
    pub(super) fn manifests(&self) -> &SpireEncodedManifestBundle {
        &self.manifests
    }

    pub(super) fn root_control_state(
        &self,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireRootControlState, SpirePublishFailed> {
        SpireRootControlState::published(
            self.snapshot.epoch_manifest().epoch,
            self.input.next_pid,
            self.input.next_local_vec_seq,
            locators.epoch_manifest_tid,
            locators.object_manifest_tid,
            locators.placement_directory_tid,
        )
        .map_err(|error| SpirePublishFailed::at(SpirePublishStage::PublishingActiveEpoch, error))
    }

    pub(super) fn publish_active_epoch(
        self,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpirePublishPublished, SpirePublishFailed> {
        let root_control_state = self.root_control_state(locators)?;
        let root_control_state_bytes = root_control_state.encode().map_err(|error| {
            SpirePublishFailed::at(SpirePublishStage::PublishingActiveEpoch, error)
        })?;
        Ok(SpirePublishPublished {
            root_control_state,
            bundle: SpireEncodedPublishBundle {
                manifests: self.manifests,
                root_control_state: root_control_state_bytes,
            },
        })
    }
}

fn publish_through_validation(
    input: SpirePublishCoordinatorInput<'_>,
) -> Result<SpirePublishPublishingActiveEpoch<'_>, SpirePublishFailed> {
    let object_evidence = object_write_evidence_from_placement_directory(input.placement_directory);
    let placement_evidence = placement_write_evidence_from_object_manifest(input.object_manifest);
    SpirePublishWritingObjects::new(input)
        .objects_written(&object_evidence)?
        .placements_written(&placement_evidence)?
        .write_manifests()?
        .validate()
}

pub(super) fn object_write_evidence_from_placement_directory(
    placement_directory: &SpirePlacementDirectory,
) -> Vec<SpirePublishObjectWriteEvidence> {
    placement_directory
        .entries
        .iter()
        .map(|entry| SpirePublishObjectWriteEvidence {
            pid: entry.pid,
            object_tid: entry.object_tid,
        })
        .collect()
}

pub(super) fn placement_write_evidence_from_object_manifest(
    object_manifest: &SpireObjectManifest,
) -> Vec<SpirePublishPlacementWriteEvidence> {
    object_manifest
        .entries
        .iter()
        .map(|entry| SpirePublishPlacementWriteEvidence {
            pid: entry.pid,
            placement_tid: entry.placement_tid,
        })
        .collect()
}

pub(super) fn object_manifest_from_placement_writes(
    epoch: u64,
    placement_directory: &SpirePlacementDirectory,
    evidence: &[SpirePublishPlacementWriteEvidence],
) -> Result<SpireObjectManifest, String> {
    if epoch == 0 {
        return Err("ec_spire object manifest epoch 0 is invalid".to_owned());
    }
    if placement_directory.epoch != epoch {
        return Err(format!(
            "ec_spire object manifest placement directory epoch mismatch: got {}, expected {epoch}",
            placement_directory.epoch
        ));
    }
    if evidence.len() != placement_directory.entries.len() {
        return Err(format!(
            "ec_spire placement write evidence count mismatch: got {}, expected {}",
            evidence.len(),
            placement_directory.entries.len()
        ));
    }

    let mut sorted = evidence.to_vec();
    sorted.sort_by_key(|entry| entry.pid);
    let mut previous_pid = None;
    for entry in &sorted {
        if entry.pid == 0 {
            return Err("ec_spire placement write evidence pid 0 is invalid".to_owned());
        }
        if entry.placement_tid == ItemPointer::INVALID {
            return Err(format!(
                "ec_spire placement write evidence for pid {} has invalid placement_tid",
                entry.pid
            ));
        }
        if Some(entry.pid) == previous_pid {
            return Err(format!(
                "ec_spire placement write evidence duplicate pid {}",
                entry.pid
            ));
        }
        previous_pid = Some(entry.pid);
    }

    let mut entries = Vec::with_capacity(placement_directory.entries.len());
    for (placement, write) in placement_directory.entries.iter().zip(sorted.iter()) {
        if placement.pid != write.pid {
            return Err(format!(
                "ec_spire placement write evidence pid mismatch: got {}, expected {}",
                write.pid, placement.pid
            ));
        }
        entries.push(SpireManifestEntry {
            epoch,
            pid: placement.pid,
            object_version: placement.object_version,
            placement_tid: write.placement_tid,
        });
    }
    SpireObjectManifest::from_entries(epoch, entries)
}

fn validate_object_write_evidence(
    placement_directory: &SpirePlacementDirectory,
    evidence: &[SpirePublishObjectWriteEvidence],
) -> Result<(), String> {
    if evidence.len() != placement_directory.entries.len() {
        return Err(format!(
            "ec_spire publish object write evidence count mismatch: got {}, expected {}",
            evidence.len(),
            placement_directory.entries.len()
        ));
    }

    let mut sorted = evidence.to_vec();
    sorted.sort_by_key(|entry| entry.pid);
    let mut previous_pid = None;
    for entry in &sorted {
        if entry.pid == 0 {
            return Err("ec_spire publish object write evidence pid 0 is invalid".to_owned());
        }
        if entry.object_tid == ItemPointer::INVALID {
            return Err(format!(
                "ec_spire publish object write evidence for pid {} has invalid object_tid",
                entry.pid
            ));
        }
        if Some(entry.pid) == previous_pid {
            return Err(format!(
                "ec_spire publish object write evidence duplicate pid {}",
                entry.pid
            ));
        }
        previous_pid = Some(entry.pid);
    }

    for (expected, actual) in placement_directory.entries.iter().zip(sorted.iter()) {
        if expected.pid != actual.pid {
            return Err(format!(
                "ec_spire publish object write evidence pid mismatch: got {}, expected {}",
                actual.pid, expected.pid
            ));
        }
        if expected.object_tid != actual.object_tid {
            return Err(format!(
                "ec_spire publish object write evidence object_tid mismatch for pid {}",
                expected.pid
            ));
        }
    }
    Ok(())
}

fn validate_placement_write_evidence(
    object_manifest: &SpireObjectManifest,
    evidence: &[SpirePublishPlacementWriteEvidence],
) -> Result<(), String> {
    if evidence.len() != object_manifest.entries.len() {
        return Err(format!(
            "ec_spire publish placement write evidence count mismatch: got {}, expected {}",
            evidence.len(),
            object_manifest.entries.len()
        ));
    }

    let mut sorted = evidence.to_vec();
    sorted.sort_by_key(|entry| entry.pid);
    let mut previous_pid = None;
    for entry in &sorted {
        if entry.pid == 0 {
            return Err("ec_spire publish placement write evidence pid 0 is invalid".to_owned());
        }
        if entry.placement_tid == ItemPointer::INVALID {
            return Err(format!(
                "ec_spire publish placement write evidence for pid {} has invalid placement_tid",
                entry.pid
            ));
        }
        if Some(entry.pid) == previous_pid {
            return Err(format!(
                "ec_spire publish placement write evidence duplicate pid {}",
                entry.pid
            ));
        }
        previous_pid = Some(entry.pid);
    }

    for (expected, actual) in object_manifest.entries.iter().zip(sorted.iter()) {
        if expected.pid != actual.pid {
            return Err(format!(
                "ec_spire publish placement write evidence pid mismatch: got {}, expected {}",
                actual.pid, expected.pid
            ));
        }
        if expected.placement_tid != actual.placement_tid {
            return Err(format!(
                "ec_spire publish placement write evidence placement_tid mismatch for pid {}",
                expected.pid
            ));
        }
    }
    Ok(())
}

pub(super) fn encode_manifest_bundle_for_publish(
    input: SpirePublishCoordinatorInput<'_>,
) -> Result<SpireEncodedManifestBundle, String> {
    let publish = publish_through_validation(input).map_err(SpirePublishFailed::into_error)?;
    Ok(publish.manifests().clone())
}

pub(super) fn root_control_state_for_publish(
    input: SpirePublishCoordinatorInput<'_>,
    locators: SpirePublishedManifestLocators,
) -> Result<SpireRootControlState, String> {
    publish_through_validation(input)
        .and_then(|publish| publish.root_control_state(locators))
        .map_err(SpirePublishFailed::into_error)
}

pub(super) fn encode_publish_bundle_for_publish(
    input: SpirePublishCoordinatorInput<'_>,
    locators: SpirePublishedManifestLocators,
) -> Result<SpireEncodedPublishBundle, String> {
    publish_through_validation(input)
        .and_then(|publish| publish.publish_active_epoch(locators))
        .map(|published| published.bundle)
        .map_err(SpirePublishFailed::into_error)
}

pub(super) unsafe fn write_manifest_bundle_to_relation(
    index_relation: pg_sys::Relation,
    manifests: &SpireEncodedManifestBundle,
) -> Result<SpirePublishedManifestLocators, String> {
    let epoch_manifest_tid =
        unsafe { page::append_object_tuple(index_relation, &manifests.epoch_manifest)? };
    let object_manifest_tid =
        unsafe { page::append_object_tuple(index_relation, &manifests.object_manifest)? };
    let placement_directory_tid =
        unsafe { page::append_object_tuple(index_relation, &manifests.placement_directory)? };
    Ok(SpirePublishedManifestLocators {
        epoch_manifest_tid,
        object_manifest_tid,
        placement_directory_tid,
    })
}

fn retired_epoch_manifest_from(
    previous_epoch_manifest: SpireEpochManifest,
) -> Result<SpireEpochManifest, String> {
    if previous_epoch_manifest.state != SpireEpochState::Published {
        return Err("ec_spire can only retire a previously published epoch manifest".to_owned());
    }
    let retired_epoch_manifest = SpireEpochManifest {
        state: SpireEpochState::Retired,
        active_query_count: 0,
        ..previous_epoch_manifest
    };
    retired_epoch_manifest.validate()?;
    Ok(retired_epoch_manifest)
}

pub(super) unsafe fn write_retired_epoch_manifest_to_relation(
    index_relation: pg_sys::Relation,
    previous_epoch_manifest: SpireEpochManifest,
) -> Result<ItemPointer, String> {
    let retired_epoch_manifest = retired_epoch_manifest_from(previous_epoch_manifest)?;
    let encoded = retired_epoch_manifest.encode()?;
    // Replacement publishes append this retired copy before the new manifest
    // bundle while holding the publish/extension lock, so its TID orders after
    // the original published manifest for snapshot dedupe.
    unsafe { page::append_object_tuple(index_relation, &encoded) }
}

pub(super) unsafe fn publish_replacement_epoch_to_relation(
    index_relation: pg_sys::Relation,
    previous_epoch_manifest: SpireEpochManifest,
    input: SpirePublishCoordinatorInput<'_>,
) -> Result<(), String> {
    let manifests = encode_manifest_bundle_for_publish(input)?;
    unsafe { write_retired_epoch_manifest_to_relation(index_relation, previous_epoch_manifest)? };
    let locators = unsafe { write_manifest_bundle_to_relation(index_relation, &manifests)? };
    let root_control = root_control_state_for_publish(input, locators)?;
    unsafe { page::initialize_root_control_page(index_relation, root_control) };
    Ok(())
}

pub(super) unsafe fn write_placement_entries_to_relation(
    index_relation: pg_sys::Relation,
    placement_directory: &SpirePlacementDirectory,
) -> Result<Vec<SpirePublishPlacementWriteEvidence>, String> {
    let mut evidence = Vec::with_capacity(placement_directory.entries.len());
    for entry in &placement_directory.entries {
        let encoded = entry.encode()?;
        let placement_tid = unsafe { page::append_object_tuple(index_relation, &encoded)? };
        evidence.push(SpirePublishPlacementWriteEvidence {
            pid: entry.pid,
            placement_tid,
        });
    }
    Ok(evidence)
}

