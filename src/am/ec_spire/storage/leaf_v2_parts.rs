#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafPartitionObjectV2Meta {
    pub(super) header: SpirePartitionObjectHeader,
    pub(super) payload_format: u8,
    pub(super) payload_stride: u32,
    pub(super) vec_id_kind: SpireVecIdKind,
    pub(super) vec_id_stride: u16,
    pub(super) segment_count: u32,
    pub(super) first_segment_locator: ItemPointer,
    pub(super) object_bytes_total: u64,
}

impl SpireLeafPartitionObjectV2Meta {
    fn new(
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        assignment_count: u32,
        payload_format: u8,
        payload_stride: u32,
        vec_id_kind: SpireVecIdKind,
        vec_id_stride: u16,
        segment_count: u32,
        first_segment_locator: ItemPointer,
        object_bytes_total: u64,
        published_epoch_backref: u64,
    ) -> Result<Self, String> {
        let meta = Self {
            header: SpirePartitionObjectHeader {
                kind: SpirePartitionObjectKind::Leaf,
                pid,
                object_version,
                published_epoch_backref,
                level: 0,
                parent_pid,
                child_count: 0,
                assignment_count,
                flags: LEAF_V2_META_FLAG,
            },
            payload_format,
            payload_stride,
            vec_id_kind,
            vec_id_stride,
            segment_count,
            first_segment_locator,
            object_bytes_total,
        };
        meta.validate()?;
        Ok(meta)
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut out = self
            .header
            .encode_after_validation(PARTITION_OBJECT_FORMAT_VERSION_V2);
        out.push(self.payload_format);
        out.push(self.vec_id_kind as u8);
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.payload_stride.to_le_bytes());
        out.extend_from_slice(&self.vec_id_stride.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.segment_count.to_le_bytes());
        self.first_segment_locator.encode_into(&mut out);
        out.extend_from_slice(&self.object_bytes_total.to_le_bytes());
        debug_assert_eq!(
            out.len(),
            PARTITION_OBJECT_HEADER_BYTES + LEAF_V2_META_BODY_BYTES
        );
        Ok(out)
    }

    fn decode(input: &[u8]) -> Result<Self, String> {
        let (header, format_version, tail) =
            SpirePartitionObjectHeader::decode_prefix_with_format_version(input)?;
        if format_version != PARTITION_OBJECT_FORMAT_VERSION_V2 {
            return Err(format!(
                "ec_spire leaf V2 meta format version must be 2, got {format_version}"
            ));
        }
        if tail.len() != LEAF_V2_META_BODY_BYTES {
            return Err(format!(
                "ec_spire leaf V2 meta length mismatch: got {}, expected {}",
                tail.len(),
                LEAF_V2_META_BODY_BYTES
            ));
        }
        let reserved = u16::from_le_bytes(tail[2..4].try_into().expect("leaf V2 reserved"));
        if reserved != 0 {
            return Err(format!(
                "ec_spire leaf V2 meta reserved bytes must be zero, got {reserved}"
            ));
        }
        let reserved2 = u16::from_le_bytes(tail[10..12].try_into().expect("leaf V2 reserved2"));
        if reserved2 != 0 {
            return Err(format!(
                "ec_spire leaf V2 meta reserved2 bytes must be zero, got {reserved2}"
            ));
        }
        let first_segment_locator = ItemPointer::decode(&tail[16..22])?;
        let meta = Self {
            header,
            payload_format: tail[0],
            vec_id_kind: SpireVecIdKind::decode(tail[1])?,
            payload_stride: u32::from_le_bytes(tail[4..8].try_into().expect("payload stride")),
            vec_id_stride: u16::from_le_bytes(tail[8..10].try_into().expect("vec id stride")),
            segment_count: u32::from_le_bytes(tail[12..16].try_into().expect("segment count")),
            first_segment_locator,
            object_bytes_total: u64::from_le_bytes(
                tail[22..30].try_into().expect("object bytes total"),
            ),
        };
        meta.validate()?;
        Ok(meta)
    }

    fn validate(&self) -> Result<(), String> {
        validate_leaf_v2_header(&self.header, LEAF_V2_META_FLAG)?;
        validate_assignment_payload_format(self.payload_format)?;
        validate_leaf_v2_locator(self.first_segment_locator, "first segment")?;
        if self.header.published_epoch_backref == 0 {
            return Err("ec_spire leaf V2 published epoch backref 0 is invalid".to_owned());
        }
        if self.object_bytes_total == 0 {
            return Err("ec_spire leaf V2 object_bytes_total 0 is invalid".to_owned());
        }
        match self.vec_id_kind {
            SpireVecIdKind::LocalU64 => {
                if usize::from(self.vec_id_stride) != LEAF_V2_LOCAL_VEC_ID_STRIDE {
                    return Err(format!(
                        "ec_spire leaf V2 local vec_id stride mismatch: got {}, expected {LEAF_V2_LOCAL_VEC_ID_STRIDE}",
                        self.vec_id_stride
                    ));
                }
            }
            SpireVecIdKind::GlobalBytes => {
                let stride = usize::from(self.vec_id_stride);
                if !(2..=SPIRE_VEC_ID_MAX_BYTES).contains(&stride) {
                    return Err(format!(
                        "ec_spire leaf V2 global vec_id stride {stride} is outside 2..={SPIRE_VEC_ID_MAX_BYTES}"
                    ));
                }
            }
        }
        if self.header.assignment_count == 0 {
            if self.segment_count != 0 {
                return Err("ec_spire empty leaf V2 meta cannot reference segments".to_owned());
            }
            if self.first_segment_locator != ItemPointer::INVALID {
                return Err("ec_spire empty leaf V2 meta first segment must be invalid".to_owned());
            }
            return Ok(());
        }
        if self.segment_count == 0 {
            return Err("ec_spire non-empty leaf V2 meta requires at least one segment".to_owned());
        }
        if self.first_segment_locator == ItemPointer::INVALID {
            return Err("ec_spire non-empty leaf V2 meta requires a first segment".to_owned());
        }
        if self.payload_format == SPIRE_PAYLOAD_FORMAT_NONE {
            return Err(
                "ec_spire non-empty leaf V2 meta payload format must not be NONE".to_owned(),
            );
        }
        if self.payload_stride == 0 {
            return Err("ec_spire non-empty leaf V2 meta payload stride 0 is invalid".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafPartitionObjectV2Segment {
    pub(super) header: SpirePartitionObjectHeader,
    pub(super) segment_no: u32,
    pub(super) row_base: u32,
    pub(super) next_segment_locator: ItemPointer,
    pub(super) flags: Vec<u16>,
    pub(super) vec_ids: Vec<u8>,
    pub(super) heap_tids: Vec<ItemPointer>,
    pub(super) gammas: Vec<f32>,
    pub(super) payloads: Vec<u8>,
}

impl SpireLeafPartitionObjectV2Segment {
    fn new(
        meta: &SpireLeafPartitionObjectV2Meta,
        segment_no: u32,
        row_base: u32,
        next_segment_locator: ItemPointer,
        rows: &[SpireLeafAssignmentRow],
    ) -> Result<Self, String> {
        let row_count = u32::try_from(rows.len())
            .map_err(|_| "ec_spire leaf V2 segment row count exceeds u32".to_owned())?;
        let mut flags = Vec::with_capacity(rows.len());
        let mut vec_ids = Vec::with_capacity(usize::from(meta.vec_id_stride) * rows.len());
        let mut heap_tids = Vec::with_capacity(rows.len());
        let mut gammas = Vec::with_capacity(rows.len());
        let mut payloads =
            Vec::with_capacity(usize::try_from(meta.payload_stride).unwrap_or(0) * rows.len());
        for row in rows {
            validate_leaf_assignment(row)?;
            if row.payload_format != meta.payload_format {
                return Err(format!(
                    "ec_spire leaf V2 segment payload format mismatch: got {}, expected {}",
                    row.payload_format, meta.payload_format
                ));
            }
            if row.encoded_payload.len() != meta.payload_stride as usize {
                return Err(format!(
                    "ec_spire leaf V2 segment payload stride mismatch: got {}, expected {}",
                    row.encoded_payload.len(),
                    meta.payload_stride
                ));
            }
            encode_leaf_v2_vec_id(
                &row.vec_id,
                meta.vec_id_kind,
                usize::from(meta.vec_id_stride),
                &mut vec_ids,
            )?;
            flags.push(row.flags);
            heap_tids.push(row.heap_tid);
            gammas.push(row.gamma);
            payloads.extend_from_slice(&row.encoded_payload);
        }

        let segment = Self {
            header: SpirePartitionObjectHeader {
                kind: SpirePartitionObjectKind::Leaf,
                pid: meta.header.pid,
                object_version: meta.header.object_version,
                published_epoch_backref: meta.header.published_epoch_backref,
                level: meta.header.level,
                parent_pid: meta.header.parent_pid,
                child_count: 0,
                assignment_count: row_count,
                flags: LEAF_V2_SEGMENT_FLAG,
            },
            segment_no,
            row_base,
            next_segment_locator,
            flags,
            vec_ids,
            heap_tids,
            gammas,
            payloads,
        };
        segment.validate_against_meta(meta)?;
        Ok(segment)
    }

    pub(super) fn columns<'a>(
        &'a self,
        meta: &'a SpireLeafPartitionObjectV2Meta,
    ) -> Result<SpireLeafObjectColumns<'a>, String> {
        self.validate_against_meta(meta)?;
        Ok(SpireLeafObjectColumns {
            header: self.header,
            payload_format: meta.payload_format,
            payload_stride: usize::try_from(meta.payload_stride)
                .map_err(|_| "ec_spire leaf V2 payload stride exceeds usize".to_owned())?,
            vec_id_kind: meta.vec_id_kind,
            vec_id_stride: usize::from(meta.vec_id_stride),
            row_base: self.row_base,
            flags: &self.flags,
            vec_ids: &self.vec_ids,
            heap_tids: &self.heap_tids,
            gammas: &self.gammas,
            payloads: &self.payloads,
        })
    }

    fn encode(&self, meta: &SpireLeafPartitionObjectV2Meta) -> Result<Vec<u8>, String> {
        self.validate_against_meta(meta)?;
        let mut out = self
            .header
            .encode_after_validation(PARTITION_OBJECT_FORMAT_VERSION_V2);
        out.extend_from_slice(&self.segment_no.to_le_bytes());
        out.extend_from_slice(&self.row_base.to_le_bytes());
        out.extend_from_slice(&self.header.assignment_count.to_le_bytes());
        self.next_segment_locator.encode_into(&mut out);
        for flag in &self.flags {
            out.extend_from_slice(&flag.to_le_bytes());
        }
        out.extend_from_slice(&self.vec_ids);
        for heap_tid in &self.heap_tids {
            heap_tid.encode_into(&mut out);
        }
        for gamma in &self.gammas {
            out.extend_from_slice(&gamma.to_le_bytes());
        }
        out.extend_from_slice(&self.payloads);
        Ok(out)
    }

    fn decode(input: &[u8], meta: &SpireLeafPartitionObjectV2Meta) -> Result<Self, String> {
        let (header, format_version, tail) =
            SpirePartitionObjectHeader::decode_prefix_with_format_version(input)?;
        if format_version != PARTITION_OBJECT_FORMAT_VERSION_V2 {
            return Err(format!(
                "ec_spire leaf V2 segment format version must be 2, got {format_version}"
            ));
        }
        if tail.len() < LEAF_V2_SEGMENT_PREFIX_BYTES {
            return Err(format!(
                "ec_spire leaf V2 segment too short: got {}, expected at least {}",
                tail.len(),
                LEAF_V2_SEGMENT_PREFIX_BYTES
            ));
        }
        let segment_no = u32::from_le_bytes(tail[0..4].try_into().expect("segment no"));
        let row_base = u32::from_le_bytes(tail[4..8].try_into().expect("row base"));
        let row_count = u32::from_le_bytes(tail[8..12].try_into().expect("row count"));
        let next_segment_locator = ItemPointer::decode(&tail[12..18])?;
        if header.assignment_count != row_count {
            return Err(format!(
                "ec_spire leaf V2 segment row count mismatch: header {}, body {row_count}",
                header.assignment_count
            ));
        }

        let row_count_usize = usize::try_from(row_count)
            .map_err(|_| "ec_spire leaf V2 segment row count exceeds usize".to_owned())?;
        let flags_bytes = row_count_usize
            .checked_mul(size_of::<u16>())
            .ok_or_else(|| "ec_spire leaf V2 flags byte length overflow".to_owned())?;
        let vec_id_bytes = row_count_usize
            .checked_mul(usize::from(meta.vec_id_stride))
            .ok_or_else(|| "ec_spire leaf V2 vec_id byte length overflow".to_owned())?;
        let heap_tid_bytes = row_count_usize
            .checked_mul(ITEM_POINTER_BYTES)
            .ok_or_else(|| "ec_spire leaf V2 heap_tid byte length overflow".to_owned())?;
        let gamma_bytes = row_count_usize
            .checked_mul(size_of::<f32>())
            .ok_or_else(|| "ec_spire leaf V2 gamma byte length overflow".to_owned())?;
        let payload_bytes = row_count_usize
            .checked_mul(
                usize::try_from(meta.payload_stride)
                    .map_err(|_| "ec_spire leaf V2 payload stride exceeds usize".to_owned())?,
            )
            .ok_or_else(|| "ec_spire leaf V2 payload byte length overflow".to_owned())?;
        let expected_tail_len = LEAF_V2_SEGMENT_PREFIX_BYTES
            .checked_add(flags_bytes)
            .and_then(|len| len.checked_add(vec_id_bytes))
            .and_then(|len| len.checked_add(heap_tid_bytes))
            .and_then(|len| len.checked_add(gamma_bytes))
            .and_then(|len| len.checked_add(payload_bytes))
            .ok_or_else(|| "ec_spire leaf V2 segment length overflow".to_owned())?;
        if tail.len() != expected_tail_len {
            return Err(format!(
                "ec_spire leaf V2 segment length mismatch: got {}, expected {expected_tail_len}",
                tail.len()
            ));
        }

        let mut cursor = LEAF_V2_SEGMENT_PREFIX_BYTES;
        let mut flags = Vec::with_capacity(row_count_usize);
        for chunk in tail[cursor..cursor + flags_bytes].chunks_exact(size_of::<u16>()) {
            flags.push(u16::from_le_bytes(chunk.try_into().expect("flag bytes")));
        }
        cursor += flags_bytes;
        let vec_ids = tail[cursor..cursor + vec_id_bytes].to_vec();
        cursor += vec_id_bytes;
        let mut heap_tids = Vec::with_capacity(row_count_usize);
        for chunk in tail[cursor..cursor + heap_tid_bytes].chunks_exact(ITEM_POINTER_BYTES) {
            heap_tids.push(ItemPointer::decode(chunk)?);
        }
        cursor += heap_tid_bytes;
        let mut gammas = Vec::with_capacity(row_count_usize);
        for chunk in tail[cursor..cursor + gamma_bytes].chunks_exact(size_of::<f32>()) {
            let gamma = f32::from_le_bytes(chunk.try_into().expect("gamma bytes"));
            if !gamma.is_finite() {
                return Err("ec_spire leaf V2 segment gamma must be finite".to_owned());
            }
            gammas.push(gamma);
        }
        cursor += gamma_bytes;
        let payloads = tail[cursor..cursor + payload_bytes].to_vec();

        let segment = Self {
            header,
            segment_no,
            row_base,
            next_segment_locator,
            flags,
            vec_ids,
            heap_tids,
            gammas,
            payloads,
        };
        segment.validate_against_meta(meta)?;
        Ok(segment)
    }

    fn validate_against_meta(&self, meta: &SpireLeafPartitionObjectV2Meta) -> Result<(), String> {
        validate_leaf_v2_header(&self.header, LEAF_V2_SEGMENT_FLAG)?;
        if self.header.pid != meta.header.pid
            || self.header.object_version != meta.header.object_version
            || self.header.parent_pid != meta.header.parent_pid
        {
            return Err("ec_spire leaf V2 segment header does not match meta".to_owned());
        }
        validate_leaf_v2_locator(self.next_segment_locator, "next segment")?;
        let row_count = usize::try_from(self.header.assignment_count)
            .map_err(|_| "ec_spire leaf V2 segment row count exceeds usize".to_owned())?;
        if row_count == 0 {
            return Err("ec_spire leaf V2 segment row count 0 is invalid".to_owned());
        }
        if self.flags.len() != row_count {
            return Err(format!(
                "ec_spire leaf V2 segment flags length mismatch: got {}, expected {row_count}",
                self.flags.len()
            ));
        }
        for flags in &self.flags {
            validate_assignment_flags(*flags)?;
            if flags & (SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE)
                != 0
            {
                return Err("ec_spire leaf V2 segment rows cannot set delta flags".to_owned());
            }
        }
        let expected_vec_id_bytes = row_count
            .checked_mul(usize::from(meta.vec_id_stride))
            .ok_or_else(|| "ec_spire leaf V2 vec_id length overflow".to_owned())?;
        if self.vec_ids.len() != expected_vec_id_bytes {
            return Err(format!(
                "ec_spire leaf V2 segment vec_id length mismatch: got {}, expected {expected_vec_id_bytes}",
                self.vec_ids.len()
            ));
        }
        for chunk in self.vec_ids.chunks_exact(usize::from(meta.vec_id_stride)) {
            decode_leaf_v2_vec_id(meta.vec_id_kind, chunk)?;
        }
        if self.heap_tids.len() != row_count {
            return Err(format!(
                "ec_spire leaf V2 segment heap_tid length mismatch: got {}, expected {row_count}",
                self.heap_tids.len()
            ));
        }
        if self.heap_tids.contains(&ItemPointer::INVALID) {
            return Err("ec_spire leaf V2 segment heap_tid must be valid".to_owned());
        }
        if self.gammas.len() != row_count {
            return Err(format!(
                "ec_spire leaf V2 segment gamma length mismatch: got {}, expected {row_count}",
                self.gammas.len()
            ));
        }
        if self.gammas.iter().any(|gamma| !gamma.is_finite()) {
            return Err("ec_spire leaf V2 segment gamma must be finite".to_owned());
        }
        let expected_payload_bytes = row_count
            .checked_mul(
                usize::try_from(meta.payload_stride)
                    .map_err(|_| "ec_spire leaf V2 payload stride exceeds usize".to_owned())?,
            )
            .ok_or_else(|| "ec_spire leaf V2 payload length overflow".to_owned())?;
        if self.payloads.len() != expected_payload_bytes {
            return Err(format!(
                "ec_spire leaf V2 segment payload length mismatch: got {}, expected {expected_payload_bytes}",
                self.payloads.len()
            ));
        }
        Ok(())
    }
}
