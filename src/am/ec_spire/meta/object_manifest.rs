#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireManifestEntry {
    pub(super) epoch: u64,
    pub(super) pid: u64,
    pub(super) object_version: u64,
    pub(super) placement_tid: ItemPointer,
}

impl SpireManifestEntry {
    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;

        let mut out = Vec::with_capacity(MANIFEST_ENTRY_BYTES);
        out.extend_from_slice(&META_FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.epoch.to_le_bytes());
        out.extend_from_slice(&self.pid.to_le_bytes());
        out.extend_from_slice(&self.object_version.to_le_bytes());
        self.placement_tid.encode_into(&mut out);
        debug_assert_eq!(out.len(), MANIFEST_ENTRY_BYTES);
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != MANIFEST_ENTRY_BYTES {
            return Err(format!(
                "ec_spire manifest entry length mismatch: got {}, expected {MANIFEST_ENTRY_BYTES}",
                input.len()
            ));
        }
        validate_format_version(&input[0..2])?;
        let reserved = u16::from_le_bytes(input[2..4].try_into().expect("reserved bytes"));
        if reserved != 0 {
            return Err(format!(
                "ec_spire manifest entry reserved bytes must be zero, got {reserved}"
            ));
        }

        let entry = Self {
            epoch: u64::from_le_bytes(input[4..12].try_into().expect("epoch bytes")),
            pid: u64::from_le_bytes(input[12..20].try_into().expect("pid bytes")),
            object_version: u64::from_le_bytes(
                input[20..28].try_into().expect("object version bytes"),
            ),
            placement_tid: ItemPointer::decode(&input[28..34])?,
        };
        entry.validate()?;
        Ok(entry)
    }

    fn validate(&self) -> Result<(), String> {
        if self.epoch == 0 {
            return Err("ec_spire manifest entry epoch 0 is invalid".to_owned());
        }
        if self.pid == 0 {
            return Err("ec_spire manifest entry pid 0 is invalid".to_owned());
        }
        if self.object_version == 0 {
            return Err("ec_spire manifest entry object_version 0 is invalid".to_owned());
        }
        if self.placement_tid == ItemPointer::INVALID {
            return Err("ec_spire manifest entry placement_tid must be valid".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireObjectManifest {
    pub(super) epoch: u64,
    pub(super) entries: Vec<SpireManifestEntry>,
}

impl SpireObjectManifest {
    pub(super) fn from_entries(
        epoch: u64,
        mut entries: Vec<SpireManifestEntry>,
    ) -> Result<Self, String> {
        if epoch == 0 {
            return Err("ec_spire object manifest epoch 0 is invalid".to_owned());
        }
        entries.sort_by_key(|entry| entry.pid);
        let manifest = Self { epoch, entries };
        manifest.validate()?;
        Ok(manifest)
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let entry_count = u32::try_from(self.entries.len())
            .map_err(|_| "ec_spire object manifest entry count exceeds u32".to_owned())?;

        let mut out = Vec::with_capacity(
            OBJECT_MANIFEST_HEADER_BYTES + self.entries.len() * MANIFEST_ENTRY_BYTES,
        );
        out.extend_from_slice(&OBJECT_MANIFEST_MAGIC.to_le_bytes());
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
        if input.len() < OBJECT_MANIFEST_HEADER_BYTES {
            return Err(format!(
                "ec_spire object manifest too short: got {}, expected at least {OBJECT_MANIFEST_HEADER_BYTES}",
                input.len()
            ));
        }
        let magic = u32::from_le_bytes(input[0..4].try_into().expect("manifest magic bytes"));
        if magic != OBJECT_MANIFEST_MAGIC {
            return Err(format!(
                "ec_spire invalid object manifest magic: {magic:#x}"
            ));
        }
        validate_format_version(&input[4..6])?;
        let reserved = u16::from_le_bytes(input[6..8].try_into().expect("reserved bytes"));
        if reserved != 0 {
            return Err(format!(
                "ec_spire object manifest reserved bytes must be zero, got {reserved}"
            ));
        }
        let epoch = u64::from_le_bytes(input[8..16].try_into().expect("manifest epoch bytes"));
        let entry_count =
            u32::from_le_bytes(input[16..20].try_into().expect("manifest count bytes")) as usize;
        let expected_len = entry_count
            .checked_mul(MANIFEST_ENTRY_BYTES)
            .and_then(|entry_bytes| entry_bytes.checked_add(OBJECT_MANIFEST_HEADER_BYTES))
            .ok_or_else(|| "ec_spire object manifest length overflow".to_owned())?;
        if input.len() != expected_len {
            return Err(format!(
                "ec_spire object manifest length mismatch: got {}, expected {expected_len}",
                input.len()
            ));
        }

        let mut entries = Vec::with_capacity(entry_count);
        let mut cursor = OBJECT_MANIFEST_HEADER_BYTES;
        for _ in 0..entry_count {
            let entry = SpireManifestEntry::decode(&input[cursor..cursor + MANIFEST_ENTRY_BYTES])?;
            entries.push(entry);
            cursor += MANIFEST_ENTRY_BYTES;
        }
        Self::from_entries(epoch, entries)
    }

    pub(super) fn get(&self, pid: u64) -> Option<&SpireManifestEntry> {
        self.entries
            .binary_search_by_key(&pid, |entry| entry.pid)
            .ok()
            .map(|index| &self.entries[index])
    }

    fn validate(&self) -> Result<(), String> {
        if self.epoch == 0 {
            return Err("ec_spire object manifest epoch 0 is invalid".to_owned());
        }
        let mut previous_pid = None;
        for entry in &self.entries {
            entry.validate()?;
            if entry.epoch != self.epoch {
                return Err(format!(
                    "ec_spire object manifest epoch mismatch: manifest {}, entry {}",
                    self.epoch, entry.epoch
                ));
            }
            if let Some(previous_pid) = previous_pid {
                if entry.pid == previous_pid {
                    return Err(format!(
                        "ec_spire object manifest duplicate pid: {}",
                        entry.pid
                    ));
                }
                if entry.pid < previous_pid {
                    return Err("ec_spire object manifest entries must be sorted".to_owned());
                }
            }
            previous_pid = Some(entry.pid);
        }
        Ok(())
    }
}
