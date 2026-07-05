#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpirePublishedEpochSnapshot<'a> {
    pub(super) epoch_manifest: &'a SpireEpochManifest,
    pub(super) object_manifest: &'a SpireObjectManifest,
    pub(super) placement_directory: &'a SpirePlacementDirectory,
}

impl<'a> SpirePublishedEpochSnapshot<'a> {
    pub(super) fn new(
        epoch_manifest: &'a SpireEpochManifest,
        object_manifest: &'a SpireObjectManifest,
        placement_directory: &'a SpirePlacementDirectory,
    ) -> Result<Self, String> {
        let snapshot = Self {
            epoch_manifest,
            object_manifest,
            placement_directory,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    fn validate(&self) -> Result<(), String> {
        self.epoch_manifest.validate()?;
        self.object_manifest.validate()?;
        self.placement_directory.validate()?;

        if self.epoch_manifest.state != SpireEpochState::Published {
            return Err("ec_spire published snapshot requires a published epoch".to_owned());
        }

        let epoch = self.epoch_manifest.epoch;
        if self.object_manifest.epoch != epoch {
            return Err(format!(
                "ec_spire published snapshot object manifest epoch mismatch: epoch {}, manifest {}",
                epoch, self.object_manifest.epoch
            ));
        }
        if self.placement_directory.epoch != epoch {
            return Err(format!(
                "ec_spire published snapshot placement directory epoch mismatch: epoch {}, directory {}",
                epoch, self.placement_directory.epoch
            ));
        }

        for entry in &self.object_manifest.entries {
            let placement = self.placement_directory.get(entry.pid).ok_or_else(|| {
                format!(
                    "ec_spire published snapshot missing placement for pid {}",
                    entry.pid
                )
            })?;

            if placement.object_version != entry.object_version {
                return Err(format!(
                    "ec_spire published snapshot object_version mismatch for pid {}: manifest {}, placement {}",
                    entry.pid, entry.object_version, placement.object_version
                ));
            }

            match (self.epoch_manifest.consistency_mode, placement.state) {
                (SpireConsistencyMode::Strict, SpirePlacementState::Available)
                | (SpireConsistencyMode::Degraded, SpirePlacementState::Available)
                | (SpireConsistencyMode::Degraded, SpirePlacementState::Unavailable)
                | (SpireConsistencyMode::Degraded, SpirePlacementState::Skipped) => {}
                (SpireConsistencyMode::Strict, state) => {
                    return Err(format!(
                        "ec_spire strict published snapshot requires available placement for pid {}: got {:?}",
                        entry.pid, state
                    ));
                }
                (SpireConsistencyMode::Degraded, SpirePlacementState::Stale) => {
                    return Err(format!(
                        "ec_spire degraded published snapshot cannot use stale placement for pid {}",
                        entry.pid
                    ));
                }
            }
        }

        for placement in &self.placement_directory.entries {
            if self.object_manifest.get(placement.pid).is_none() {
                return Err(format!(
                    "ec_spire published snapshot has placement without object manifest entry for pid {}",
                    placement.pid
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireSnapshotPidLookup<'a> {
    pub(super) manifest_entry: &'a SpireManifestEntry,
    pub(super) placement: &'a SpirePlacementEntry,
}

#[derive(Debug, Clone)]
pub(super) struct SpireValidatedEpochSnapshot<'a> {
    snapshot: SpirePublishedEpochSnapshot<'a>,
    pid_index: HashMap<u64, SpireSnapshotPidLookup<'a>>,
}

impl<'a> SpireValidatedEpochSnapshot<'a> {
    pub(super) fn new(
        epoch_manifest: &'a SpireEpochManifest,
        object_manifest: &'a SpireObjectManifest,
        placement_directory: &'a SpirePlacementDirectory,
    ) -> Result<Self, String> {
        Self::from_snapshot(SpirePublishedEpochSnapshot::new(
            epoch_manifest,
            object_manifest,
            placement_directory,
        )?)
    }

    pub(super) fn from_snapshot(snapshot: SpirePublishedEpochSnapshot<'a>) -> Result<Self, String> {
        snapshot.validate()?;
        let mut pid_index = HashMap::with_capacity(snapshot.object_manifest.entries.len());
        for manifest_entry in &snapshot.object_manifest.entries {
            let placement = snapshot
                .placement_directory
                .get(manifest_entry.pid)
                .ok_or_else(|| {
                    format!(
                        "ec_spire validated snapshot missing placement for pid {}",
                        manifest_entry.pid
                    )
                })?;
            pid_index.insert(
                manifest_entry.pid,
                SpireSnapshotPidLookup {
                    manifest_entry,
                    placement,
                },
            );
        }
        Ok(Self {
            snapshot,
            pid_index,
        })
    }

    pub(super) fn snapshot(&self) -> SpirePublishedEpochSnapshot<'a> {
        self.snapshot
    }

    pub(super) fn epoch_manifest(&self) -> &'a SpireEpochManifest {
        self.snapshot.epoch_manifest
    }

    pub(super) fn object_manifest(&self) -> &'a SpireObjectManifest {
        self.snapshot.object_manifest
    }

    pub(super) fn placement_directory(&self) -> &'a SpirePlacementDirectory {
        self.snapshot.placement_directory
    }

    pub(super) fn lookup(&self, pid: u64) -> Option<SpireSnapshotPidLookup<'a>> {
        self.pid_index.get(&pid).copied()
    }

    pub(super) fn require_lookup(
        &self,
        pid: u64,
        context: &str,
    ) -> Result<SpireSnapshotPidLookup<'a>, String> {
        self.lookup(pid)
            .ok_or_else(|| format!("ec_spire {context} missing snapshot lookup for pid {pid}"))
    }
}

fn validate_format_version(input: &[u8]) -> Result<(), String> {
    let format_version = u16::from_le_bytes(input.try_into().expect("format version bytes"));
    if format_version != META_FORMAT_VERSION {
        return Err(format!(
            "ec_spire unsupported metadata format version: {format_version}"
        ));
    }
    Ok(())
}
