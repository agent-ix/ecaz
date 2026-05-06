#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpirePlacementDirectory {
    pub(super) epoch: u64,
    pub(super) entries: Vec<SpirePlacementEntry>,
}

impl SpirePlacementDirectory {
    pub(super) fn from_entries(
        epoch: u64,
        mut entries: Vec<SpirePlacementEntry>,
    ) -> Result<Self, String> {
        if epoch == 0 {
            return Err("ec_spire placement directory epoch 0 is invalid".to_owned());
        }
        entries.sort_by_key(|entry| entry.pid);
        let directory = Self { epoch, entries };
        directory.validate()?;
        Ok(directory)
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let entry_count = u32::try_from(self.entries.len())
            .map_err(|_| "ec_spire placement directory entry count exceeds u32".to_owned())?;

        let mut out = Vec::with_capacity(
            PLACEMENT_DIRECTORY_HEADER_BYTES + self.entries.len() * PLACEMENT_ENTRY_BYTES,
        );
        out.extend_from_slice(&PLACEMENT_DIRECTORY_MAGIC.to_le_bytes());
        out.extend_from_slice(&META_FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.epoch.to_le_bytes());
        out.extend_from_slice(&entry_count.to_le_bytes());
        for entry in &self.entries {
            out.extend_from_slice(&entry.encode()?);
        }
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() < PLACEMENT_DIRECTORY_HEADER_BYTES {
            return Err(format!(
                "ec_spire placement directory too short: got {}, expected at least {PLACEMENT_DIRECTORY_HEADER_BYTES}",
                input.len()
            ));
        }
        let magic = u32::from_le_bytes(input[0..4].try_into().expect("directory magic bytes"));
        if magic != PLACEMENT_DIRECTORY_MAGIC {
            return Err(format!(
                "ec_spire invalid placement directory magic: {magic:#x}"
            ));
        }
        validate_format_version(&input[4..6])?;
        let reserved = u16::from_le_bytes(input[6..8].try_into().expect("reserved bytes"));
        if reserved != 0 {
            return Err(format!(
                "ec_spire placement directory reserved bytes must be zero, got {reserved}"
            ));
        }
        let epoch = u64::from_le_bytes(input[8..16].try_into().expect("directory epoch bytes"));
        let entry_count =
            u32::from_le_bytes(input[16..20].try_into().expect("directory count bytes")) as usize;
        let expected_len = entry_count
            .checked_mul(PLACEMENT_ENTRY_BYTES)
            .and_then(|entry_bytes| entry_bytes.checked_add(PLACEMENT_DIRECTORY_HEADER_BYTES))
            .ok_or_else(|| "ec_spire placement directory length overflow".to_owned())?;
        if input.len() != expected_len {
            return Err(format!(
                "ec_spire placement directory length mismatch: got {}, expected {expected_len}",
                input.len()
            ));
        }

        let mut entries = Vec::with_capacity(entry_count);
        let mut cursor = PLACEMENT_DIRECTORY_HEADER_BYTES;
        for _ in 0..entry_count {
            let entry =
                SpirePlacementEntry::decode(&input[cursor..cursor + PLACEMENT_ENTRY_BYTES])?;
            entries.push(entry);
            cursor += PLACEMENT_ENTRY_BYTES;
        }
        Self::from_entries(epoch, entries)
    }

    pub(super) fn get(&self, pid: u64) -> Option<&SpirePlacementEntry> {
        self.entries
            .binary_search_by_key(&pid, |entry| entry.pid)
            .ok()
            .map(|index| &self.entries[index])
    }

    fn validate(&self) -> Result<(), String> {
        if self.epoch == 0 {
            return Err("ec_spire placement directory epoch 0 is invalid".to_owned());
        }
        let mut previous_pid = None;
        for entry in &self.entries {
            entry.validate()?;
            if entry.epoch != self.epoch {
                return Err(format!(
                    "ec_spire placement directory epoch mismatch: directory {}, entry {}",
                    self.epoch, entry.epoch
                ));
            }
            if let Some(previous_pid) = previous_pid {
                if entry.pid == previous_pid {
                    return Err(format!(
                        "ec_spire placement directory duplicate pid: {}",
                        entry.pid
                    ));
                }
                if entry.pid < previous_pid {
                    return Err("ec_spire placement directory entries must be sorted".to_owned());
                }
            }
            previous_pid = Some(entry.pid);
        }
        Ok(())
    }
}
