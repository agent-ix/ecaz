#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpirePlacementState {
    Available = 1,
    Stale = 2,
    Unavailable = 3,
    Skipped = 4,
}

impl SpirePlacementState {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(Self::Available),
            2 => Ok(Self::Stale),
            3 => Ok(Self::Unavailable),
            4 => Ok(Self::Skipped),
            other => Err(format!("ec_spire invalid placement state: {other}")),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireLocalStoreState {
    Available = 1,
    Unavailable = 2,
}

impl SpireLocalStoreState {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(Self::Available),
            2 => Ok(Self::Unavailable),
            other => Err(format!("ec_spire invalid local store state: {other}")),
        }
    }
}
