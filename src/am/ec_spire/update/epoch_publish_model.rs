#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum SpireEpochPublishVisibility {
    Old { epoch: u64 },
    New { epoch: u64 },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct SpireEpochPublishModel {
    active_epoch: u64,
    publishing_epoch: Option<u64>,
}

impl SpireEpochPublishModel {
    pub(crate) const fn new(active_epoch: u64) -> Self {
        Self {
            active_epoch,
            publishing_epoch: None,
        }
    }

    pub(crate) fn begin_publish(&mut self, new_epoch: u64) -> Result<(), &'static str> {
        if new_epoch <= self.active_epoch {
            return Err("new epoch must be greater than active epoch");
        }
        if self.publishing_epoch.is_some() {
            return Err("publish already in progress");
        }
        self.publishing_epoch = Some(new_epoch);
        Ok(())
    }

    pub(crate) fn scanner_visibility(&self) -> SpireEpochPublishVisibility {
        if self.publishing_epoch.is_some() {
            SpireEpochPublishVisibility::Old {
                epoch: self.active_epoch,
            }
        } else {
            SpireEpochPublishVisibility::New {
                epoch: self.active_epoch,
            }
        }
    }

    pub(crate) fn commit_publish(&mut self) -> Result<u64, &'static str> {
        let Some(new_epoch) = self.publishing_epoch.take() else {
            return Err("no publish in progress");
        };
        if new_epoch <= self.active_epoch {
            return Err("publish would move epoch backwards");
        }
        self.active_epoch = new_epoch;
        Ok(self.active_epoch)
    }

    pub(crate) fn active_visibility(&self) -> SpireEpochPublishVisibility {
        SpireEpochPublishVisibility::New {
            epoch: self.active_epoch,
        }
    }
}
