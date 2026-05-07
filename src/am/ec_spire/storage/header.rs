#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpirePartitionObjectHeader {
    pub(super) kind: SpirePartitionObjectKind,
    pub(super) pid: u64,
    pub(super) object_version: u64,
    pub(super) published_epoch_backref: u64,
    pub(super) level: u16,
    pub(super) parent_pid: u64,
    pub(super) child_count: u32,
    pub(super) assignment_count: u32,
    pub(super) flags: u32,
}

impl SpirePartitionObjectHeader {
    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.encode_with_format_version(PARTITION_OBJECT_FORMAT_VERSION_V1)
    }

    fn encode_with_format_version(&self, format_version: u16) -> Result<Vec<u8>, String> {
        self.validate_for_format_version(format_version)?;
        Ok(self.encode_after_validation(format_version))
    }

    fn validate_for_format_version(&self, format_version: u16) -> Result<(), String> {
        if self.pid == 0 {
            return Err("ec_spire partition object pid 0 is invalid".to_owned());
        }
        if self.object_version == 0 {
            return Err("ec_spire partition object version 0 is invalid".to_owned());
        }
        if format_version != PARTITION_OBJECT_FORMAT_VERSION_V1
            && format_version != PARTITION_OBJECT_FORMAT_VERSION_V2
        {
            return Err(format!(
                "ec_spire unsupported partition object format version: {format_version}"
            ));
        }
        Ok(())
    }

    fn encode_after_validation(&self, format_version: u16) -> Vec<u8> {
        let mut out = Vec::with_capacity(PARTITION_OBJECT_HEADER_BYTES);
        out.extend_from_slice(&PARTITION_OBJECT_MAGIC.to_le_bytes());
        out.extend_from_slice(&format_version.to_le_bytes());
        out.push(self.kind as u8);
        out.push(0);
        out.extend_from_slice(&self.pid.to_le_bytes());
        out.extend_from_slice(&self.object_version.to_le_bytes());
        out.extend_from_slice(&self.published_epoch_backref.to_le_bytes());
        out.extend_from_slice(&self.level.to_le_bytes());
        out.extend_from_slice(&self.parent_pid.to_le_bytes());
        out.extend_from_slice(&self.child_count.to_le_bytes());
        out.extend_from_slice(&self.assignment_count.to_le_bytes());
        out.extend_from_slice(&self.flags.to_le_bytes());
        debug_assert_eq!(out.len(), PARTITION_OBJECT_HEADER_BYTES);
        out
    }

    pub(super) fn decode_prefix(input: &[u8]) -> Result<(Self, &[u8]), String> {
        let (header, format_version, tail) = Self::decode_prefix_with_format_version(input)?;
        if format_version != PARTITION_OBJECT_FORMAT_VERSION_V1 {
            return Err(format!(
                "ec_spire unsupported partition object format version: {format_version}"
            ));
        }
        Ok((header, tail))
    }

    fn decode_prefix_with_format_version(input: &[u8]) -> Result<(Self, u16, &[u8]), String> {
        if input.len() < PARTITION_OBJECT_HEADER_BYTES {
            return Err(format!(
                "ec_spire partition object header too short: got {}, expected at least {PARTITION_OBJECT_HEADER_BYTES}",
                input.len()
            ));
        }
        let magic = u32::from_le_bytes(input[0..4].try_into().expect("magic bytes"));
        if magic != PARTITION_OBJECT_MAGIC {
            return Err(format!(
                "ec_spire invalid partition object magic: {magic:#x}"
            ));
        }
        let format_version =
            u16::from_le_bytes(input[4..6].try_into().expect("format version bytes"));
        if format_version != PARTITION_OBJECT_FORMAT_VERSION_V1
            && format_version != PARTITION_OBJECT_FORMAT_VERSION_V2
        {
            return Err(format!(
                "ec_spire unsupported partition object format version: {format_version}"
            ));
        }
        if input[7] != 0 {
            return Err(format!(
                "ec_spire partition object reserved byte must be zero, got {}",
                input[7]
            ));
        }

        let header = Self {
            kind: SpirePartitionObjectKind::decode(input[6])?,
            pid: u64::from_le_bytes(input[8..16].try_into().expect("pid bytes")),
            object_version: u64::from_le_bytes(
                input[16..24].try_into().expect("object version bytes"),
            ),
            published_epoch_backref: u64::from_le_bytes(
                input[24..32].try_into().expect("published epoch bytes"),
            ),
            level: u16::from_le_bytes(input[32..34].try_into().expect("level bytes")),
            parent_pid: u64::from_le_bytes(input[34..42].try_into().expect("parent pid bytes")),
            child_count: u32::from_le_bytes(input[42..46].try_into().expect("child count bytes")),
            assignment_count: u32::from_le_bytes(
                input[46..50].try_into().expect("assignment count bytes"),
            ),
            flags: u32::from_le_bytes(input[50..54].try_into().expect("flags bytes")),
        };
        if header.pid == 0 {
            return Err("ec_spire partition object pid 0 is invalid".to_owned());
        }
        if header.object_version == 0 {
            return Err("ec_spire partition object version 0 is invalid".to_owned());
        }
        Ok((
            header,
            format_version,
            &input[PARTITION_OBJECT_HEADER_BYTES..],
        ))
    }
}
