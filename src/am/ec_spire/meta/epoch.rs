#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireEpochManifest {
    pub(super) epoch: u64,
    pub(super) state: SpireEpochState,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) active_query_count: u64,
}

impl SpireEpochManifest {
    pub(super) fn encoded_len() -> usize {
        EPOCH_MANIFEST_BYTES
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;

        let mut out = Vec::with_capacity(EPOCH_MANIFEST_BYTES);
        out.extend_from_slice(&EPOCH_MANIFEST_MAGIC.to_le_bytes());
        out.extend_from_slice(&META_FORMAT_VERSION.to_le_bytes());
        out.push(self.state as u8);
        out.push(self.consistency_mode as u8);
        out.extend_from_slice(&self.epoch.to_le_bytes());
        out.extend_from_slice(&self.published_at_micros.to_le_bytes());
        out.extend_from_slice(&self.retain_until_micros.to_le_bytes());
        out.extend_from_slice(&self.active_query_count.to_le_bytes());
        debug_assert_eq!(out.len(), EPOCH_MANIFEST_BYTES);
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != EPOCH_MANIFEST_BYTES {
            return Err(format!(
                "ec_spire epoch manifest length mismatch: got {}, expected {EPOCH_MANIFEST_BYTES}",
                input.len()
            ));
        }
        let magic = u32::from_le_bytes(input[0..4].try_into().expect("epoch magic bytes"));
        if magic != EPOCH_MANIFEST_MAGIC {
            return Err(format!("ec_spire invalid epoch manifest magic: {magic:#x}"));
        }
        validate_format_version(&input[4..6])?;

        let manifest = Self {
            state: SpireEpochState::decode(input[6])?,
            consistency_mode: SpireConsistencyMode::decode(input[7])?,
            epoch: u64::from_le_bytes(input[8..16].try_into().expect("epoch bytes")),
            published_at_micros: i64::from_le_bytes(
                input[16..24].try_into().expect("published_at bytes"),
            ),
            retain_until_micros: i64::from_le_bytes(
                input[24..32].try_into().expect("retain_until bytes"),
            ),
            active_query_count: u64::from_le_bytes(
                input[32..40].try_into().expect("active query count bytes"),
            ),
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub(super) fn validate(&self) -> Result<(), String> {
        if self.epoch == 0 {
            return Err("ec_spire epoch manifest epoch 0 is invalid".to_owned());
        }
        if matches!(
            self.state,
            SpireEpochState::Published | SpireEpochState::Retired
        ) && self.published_at_micros <= 0
        {
            return Err(
                "ec_spire published or retired epoch must have a publish timestamp".to_owned(),
            );
        }
        if matches!(self.state, SpireEpochState::Retired)
            && self.retain_until_micros < self.published_at_micros
        {
            return Err("ec_spire retired epoch retain_until precedes published_at".to_owned());
        }
        Ok(())
    }

    pub(super) fn cleanup_eligible_at(&self, now_micros: i64) -> bool {
        match self.state {
            SpireEpochState::Building | SpireEpochState::Published => false,
            SpireEpochState::Retired => {
                self.active_query_count == 0 && now_micros >= self.retain_until_micros
            }
            SpireEpochState::Failed => now_micros >= self.retain_until_micros,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireEpochCleanupPlan {
    pub(super) cleanup_epochs: Vec<u64>,
    pub(super) retained_retired_epochs: Vec<u64>,
}

pub(super) fn plan_epoch_cleanup(
    manifests: &[SpireEpochManifest],
    active_epoch: u64,
    now_micros: i64,
) -> Result<SpireEpochCleanupPlan, String> {
    let mut epochs = Vec::with_capacity(manifests.len());
    for manifest in manifests {
        manifest.validate()?;
        if epochs.contains(&manifest.epoch) {
            return Err(format!(
                "ec_spire cleanup plan duplicate epoch: {}",
                manifest.epoch
            ));
        }
        epochs.push(manifest.epoch);
    }

    let mut retained_retired_epochs: Vec<u64> = manifests
        .iter()
        .filter(|manifest| manifest.state == SpireEpochState::Retired)
        .map(|manifest| manifest.epoch)
        .collect();
    retained_retired_epochs.sort_unstable_by(|left, right| right.cmp(left));
    retained_retired_epochs.truncate(usize::from(SPIRE_MAX_RETAINED_RETIRED_EPOCHS));
    retained_retired_epochs.sort_unstable();

    let mut cleanup_epochs = Vec::new();
    for manifest in manifests {
        if manifest.epoch == active_epoch {
            continue;
        }
        if !manifest.cleanup_eligible_at(now_micros) {
            continue;
        }
        if manifest.state == SpireEpochState::Retired
            && retained_retired_epochs.contains(&manifest.epoch)
        {
            continue;
        }
        cleanup_epochs.push(manifest.epoch);
    }
    cleanup_epochs.sort_unstable();

    Ok(SpireEpochCleanupPlan {
        cleanup_epochs,
        retained_retired_epochs,
    })
}
