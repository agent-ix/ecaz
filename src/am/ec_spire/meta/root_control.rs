#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireRootControlState {
    pub(super) active_epoch: u64,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
    pub(super) epoch_manifest_tid: ItemPointer,
    pub(super) object_manifest_tid: ItemPointer,
    pub(super) placement_directory_tid: ItemPointer,
}

impl SpireRootControlState {
    pub(super) fn encoded_len() -> usize {
        ROOT_CONTROL_STATE_BYTES
    }

    pub(super) fn empty() -> Self {
        Self {
            active_epoch: 0,
            next_pid: SPIRE_FIRST_PID,
            next_local_vec_seq: SPIRE_FIRST_LOCAL_VEC_SEQ,
            epoch_manifest_tid: ItemPointer::INVALID,
            object_manifest_tid: ItemPointer::INVALID,
            placement_directory_tid: ItemPointer::INVALID,
        }
    }

    pub(super) fn published(
        active_epoch: u64,
        next_pid: u64,
        next_local_vec_seq: u64,
        epoch_manifest_tid: ItemPointer,
        object_manifest_tid: ItemPointer,
        placement_directory_tid: ItemPointer,
    ) -> Result<Self, String> {
        let state = Self {
            active_epoch,
            next_pid,
            next_local_vec_seq,
            epoch_manifest_tid,
            object_manifest_tid,
            placement_directory_tid,
        };
        state.validate()?;
        Ok(state)
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;

        let mut out = Vec::with_capacity(ROOT_CONTROL_STATE_BYTES);
        out.extend_from_slice(&ROOT_CONTROL_MAGIC.to_le_bytes());
        out.extend_from_slice(&META_FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.active_epoch.to_le_bytes());
        out.extend_from_slice(&self.next_pid.to_le_bytes());
        out.extend_from_slice(&self.next_local_vec_seq.to_le_bytes());
        self.epoch_manifest_tid.encode_into(&mut out);
        self.object_manifest_tid.encode_into(&mut out);
        self.placement_directory_tid.encode_into(&mut out);
        debug_assert_eq!(out.len(), ROOT_CONTROL_STATE_BYTES);
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != ROOT_CONTROL_STATE_BYTES {
            return Err(format!(
                "ec_spire root/control state length mismatch: got {}, expected {ROOT_CONTROL_STATE_BYTES}",
                input.len()
            ));
        }
        let magic = u32::from_le_bytes(input[0..4].try_into().expect("root magic bytes"));
        if magic != ROOT_CONTROL_MAGIC {
            return Err(format!(
                "ec_spire invalid root/control state magic: {magic:#x}"
            ));
        }
        validate_format_version(&input[4..6])?;
        let reserved = u16::from_le_bytes(input[6..8].try_into().expect("reserved bytes"));
        if reserved != 0 {
            return Err(format!(
                "ec_spire root/control state reserved bytes must be zero, got {reserved}"
            ));
        }

        let state = Self {
            active_epoch: u64::from_le_bytes(input[8..16].try_into().expect("active epoch bytes")),
            next_pid: u64::from_le_bytes(input[16..24].try_into().expect("next pid bytes")),
            next_local_vec_seq: u64::from_le_bytes(
                input[24..32].try_into().expect("next local vec seq bytes"),
            ),
            epoch_manifest_tid: ItemPointer::decode(&input[32..38])?,
            object_manifest_tid: ItemPointer::decode(&input[38..44])?,
            placement_directory_tid: ItemPointer::decode(&input[44..50])?,
        };
        state.validate()?;
        Ok(state)
    }

    fn validate(&self) -> Result<(), String> {
        if self.next_pid == 0 {
            return Err("ec_spire root/control next_pid 0 is invalid".to_owned());
        }
        if self.next_local_vec_seq == 0 {
            return Err("ec_spire root/control next_local_vec_seq 0 is invalid".to_owned());
        }
        if self.active_epoch == 0 {
            if self.epoch_manifest_tid != ItemPointer::INVALID
                || self.object_manifest_tid != ItemPointer::INVALID
                || self.placement_directory_tid != ItemPointer::INVALID
            {
                return Err(
                    "ec_spire empty root/control state must not reference active manifests"
                        .to_owned(),
                );
            }
            return Ok(());
        }
        if self.epoch_manifest_tid == ItemPointer::INVALID {
            return Err("ec_spire active root/control state needs an epoch manifest".to_owned());
        }
        if self.object_manifest_tid == ItemPointer::INVALID {
            return Err("ec_spire active root/control state needs an object manifest".to_owned());
        }
        if self.placement_directory_tid == ItemPointer::INVALID {
            return Err(
                "ec_spire active root/control state needs a placement directory".to_owned(),
            );
        }
        Ok(())
    }
}
