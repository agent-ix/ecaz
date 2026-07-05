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
    pub(super) summary_block_rows: u32,
    pub(super) summary_count: u32,
    pub(super) first_summary_segment_locator: ItemPointer,
    pub(super) summary_bytes_total: u64,
    pub(super) summary_representative_count: u16,
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
            summary_block_rows: 0,
            summary_count: 0,
            first_summary_segment_locator: ItemPointer::INVALID,
            summary_bytes_total: 0,
            summary_representative_count: 0,
        };
        meta.validate()?;
        Ok(meta)
    }

    #[allow(clippy::too_many_arguments)]
    fn new_v3(
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
        summary_block_rows: u32,
        summary_count: u32,
        first_summary_segment_locator: ItemPointer,
        summary_bytes_total: u64,
        published_epoch_backref: u64,
    ) -> Result<Self, String> {
        Self::new_v3_with_summary_representatives(
            pid,
            object_version,
            parent_pid,
            assignment_count,
            payload_format,
            payload_stride,
            vec_id_kind,
            vec_id_stride,
            segment_count,
            first_segment_locator,
            object_bytes_total,
            summary_block_rows,
            summary_count,
            first_summary_segment_locator,
            summary_bytes_total,
            published_epoch_backref,
            1,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn new_v3_with_summary_representatives(
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
        summary_block_rows: u32,
        summary_count: u32,
        first_summary_segment_locator: ItemPointer,
        summary_bytes_total: u64,
        published_epoch_backref: u64,
        summary_representative_count: u16,
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
            summary_block_rows,
            summary_count,
            first_summary_segment_locator,
            summary_bytes_total,
            summary_representative_count,
        };
        meta.validate()?;
        Ok(meta)
    }

    pub(super) fn has_block_summaries(&self) -> bool {
        self.summary_count > 0
    }

    pub(super) fn summary_payload_stride(&self) -> Result<usize, String> {
        let payload_stride = usize::try_from(self.payload_stride)
            .map_err(|_| "ec_spire leaf V3 payload stride exceeds usize".to_owned())?;
        payload_stride
            .checked_mul(usize::from(self.summary_representative_count.max(1)))
            .ok_or_else(|| "ec_spire leaf V4 summary payload stride overflow".to_owned())
    }

    pub(super) fn encoded_meta_body_bytes(&self) -> usize {
        if self.has_block_summaries() && self.summary_representative_count > 1 {
            LEAF_V4_META_BODY_BYTES
        } else if self.has_block_summaries() {
            LEAF_V3_META_BODY_BYTES
        } else {
            LEAF_V2_META_BODY_BYTES
        }
    }

    pub(super) fn row_segment_bytes_total(&self) -> Result<u64, String> {
        let meta_bytes = PARTITION_OBJECT_HEADER_BYTES
            .checked_add(self.encoded_meta_body_bytes())
            .ok_or_else(|| "ec_spire leaf V3 meta byte length overflow".to_owned())?;
        self.object_bytes_total
            .checked_sub(self.summary_bytes_total)
            .and_then(|value| value.checked_sub(u64::try_from(meta_bytes).ok()?))
            .ok_or_else(|| "ec_spire leaf V3 row segment byte length underflow".to_owned())
    }

    fn summary_format_version(&self) -> u16 {
        if self.has_block_summaries() && self.summary_representative_count > 1 {
            PARTITION_OBJECT_FORMAT_VERSION_V4
        } else if self.has_block_summaries() {
            PARTITION_OBJECT_FORMAT_VERSION_V3
        } else {
            PARTITION_OBJECT_FORMAT_VERSION_V2
        }
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut out = self
            .header
            .encode_after_validation(self.summary_format_version());
        out.push(self.payload_format);
        out.push(self.vec_id_kind as u8);
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.payload_stride.to_le_bytes());
        out.extend_from_slice(&self.vec_id_stride.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.segment_count.to_le_bytes());
        self.first_segment_locator.encode_into(&mut out);
        out.extend_from_slice(&self.object_bytes_total.to_le_bytes());
        if self.has_block_summaries() {
            out.extend_from_slice(&self.summary_block_rows.to_le_bytes());
            out.extend_from_slice(&self.summary_count.to_le_bytes());
            self.first_summary_segment_locator.encode_into(&mut out);
            out.extend_from_slice(&self.summary_bytes_total.to_le_bytes());
            if self.summary_representative_count > 1 {
                out.extend_from_slice(&self.summary_representative_count.to_le_bytes());
                out.extend_from_slice(&0_u16.to_le_bytes());
            }
            debug_assert_eq!(
                out.len(),
                PARTITION_OBJECT_HEADER_BYTES + self.encoded_meta_body_bytes()
            );
        } else {
            debug_assert_eq!(
                out.len(),
                PARTITION_OBJECT_HEADER_BYTES + LEAF_V2_META_BODY_BYTES
            );
        }
        Ok(out)
    }

    fn decode(input: &[u8]) -> Result<Self, String> {
        let (header, format_version, tail) =
            SpirePartitionObjectHeader::decode_prefix_with_format_version(input)?;
        let expected_tail_len = match format_version {
            PARTITION_OBJECT_FORMAT_VERSION_V2 => LEAF_V2_META_BODY_BYTES,
            PARTITION_OBJECT_FORMAT_VERSION_V3 => LEAF_V3_META_BODY_BYTES,
            PARTITION_OBJECT_FORMAT_VERSION_V4 => LEAF_V4_META_BODY_BYTES,
            other => {
                return Err(format!(
                    "ec_spire leaf V2/V3/V4 meta format version must be 2, 3, or 4, got {other}"
                ));
            }
        };
        if tail.len() != expected_tail_len {
            return Err(format!(
                "ec_spire leaf V2/V3 meta length mismatch: got {}, expected {expected_tail_len}",
                tail.len()
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
        let (
            summary_block_rows,
            summary_count,
            first_summary_segment_locator,
            summary_bytes_total,
        ) = if format_version == PARTITION_OBJECT_FORMAT_VERSION_V3
            || format_version == PARTITION_OBJECT_FORMAT_VERSION_V4
        {
            (
                u32::from_le_bytes(tail[30..34].try_into().expect("summary block rows")),
                u32::from_le_bytes(tail[34..38].try_into().expect("summary count")),
                ItemPointer::decode(&tail[38..44])?,
                u64::from_le_bytes(tail[44..52].try_into().expect("summary bytes total")),
            )
        } else {
            (0, 0, ItemPointer::INVALID, 0)
        };
        let summary_representative_count = if format_version == PARTITION_OBJECT_FORMAT_VERSION_V4
        {
            let representative_count =
                u16::from_le_bytes(tail[52..54].try_into().expect("summary reps"));
            let reserved = u16::from_le_bytes(tail[54..56].try_into().expect("V4 reserved"));
            if reserved != 0 {
                return Err(format!(
                    "ec_spire leaf V4 meta reserved bytes must be zero, got {reserved}"
                ));
            }
            representative_count
        } else if summary_count > 0 {
            1
        } else {
            0
        };
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
            summary_block_rows,
            summary_count,
            first_summary_segment_locator,
            summary_bytes_total,
            summary_representative_count,
        };
        meta.validate()?;
        Ok(meta)
    }

    fn validate(&self) -> Result<(), String> {
        validate_leaf_v2_header(&self.header, LEAF_V2_META_FLAG)?;
        validate_assignment_payload_format(self.payload_format)?;
        validate_leaf_v2_locator(self.first_segment_locator, "first segment")?;
        validate_leaf_v2_locator(self.first_summary_segment_locator, "first summary segment")?;
        if self.header.published_epoch_backref == 0 {
            return Err("ec_spire leaf V2 published epoch backref 0 is invalid".to_owned());
        }
        if self.object_bytes_total == 0 {
            return Err("ec_spire leaf V2 object_bytes_total 0 is invalid".to_owned());
        }
        if self.summary_count == 0 {
            if self.summary_block_rows != 0 {
                return Err(
                    "ec_spire leaf V3 meta without summaries must use summary_block_rows 0"
                        .to_owned(),
                );
            }
            if self.summary_representative_count != 0 {
                return Err(
                    "ec_spire leaf V4 meta without summaries must use representative count 0"
                        .to_owned(),
                );
            }
            if self.first_summary_segment_locator != ItemPointer::INVALID {
                return Err(
                    "ec_spire leaf V3 meta without summaries must not reference summary segments"
                        .to_owned(),
                );
            }
            if self.summary_bytes_total != 0 {
                return Err(
                    "ec_spire leaf V3 meta without summaries must use summary_bytes_total 0"
                        .to_owned(),
                );
            }
        } else {
            if self.header.assignment_count == 0 {
                return Err("ec_spire leaf V3 summaries require non-empty assignments".to_owned());
            }
            if self.summary_block_rows == 0 {
                return Err("ec_spire leaf V3 summary_block_rows 0 is invalid".to_owned());
            }
            if self.summary_count > self.header.assignment_count {
                return Err(format!(
                    "ec_spire leaf V3 summary_count {} exceeds assignment_count {}",
                    self.summary_count, self.header.assignment_count
                ));
            }
            if self.first_summary_segment_locator == ItemPointer::INVALID {
                return Err("ec_spire leaf V3 summaries require a first summary segment".to_owned());
            }
            if self.summary_bytes_total == 0 {
                return Err("ec_spire leaf V3 summary_bytes_total 0 is invalid".to_owned());
            }
            if self.summary_representative_count == 0 {
                return Err(
                    "ec_spire leaf V4 summaries require representative count > 0".to_owned(),
                );
            }
            if self.summary_representative_count > 8 {
                return Err(format!(
                    "ec_spire leaf V4 summary representative count {} exceeds 8",
                    self.summary_representative_count
                ));
            }
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

    pub(super) fn encoded_len(&self, meta: &SpireLeafPartitionObjectV2Meta) -> Result<usize, String> {
        self.validate_against_meta(meta)?;
        PARTITION_OBJECT_HEADER_BYTES
            .checked_add(LEAF_V2_SEGMENT_PREFIX_BYTES)
            .and_then(|len| len.checked_add(self.flags.len().checked_mul(size_of::<u16>())?))
            .and_then(|len| len.checked_add(self.vec_ids.len()))
            .and_then(|len| len.checked_add(self.heap_tids.len().checked_mul(ITEM_POINTER_BYTES)?))
            .and_then(|len| len.checked_add(self.gammas.len().checked_mul(size_of::<f32>())?))
            .and_then(|len| len.checked_add(self.payloads.len()))
            .ok_or_else(|| "ec_spire leaf V2 segment encoded length overflow".to_owned())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpireLeafPartitionObjectV2SegmentReadHeader {
    segment_no: u32,
    row_base: u32,
    row_count: u32,
    next_segment_locator: ItemPointer,
}

impl SpireLeafPartitionObjectV2SegmentReadHeader {
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
        validate_leaf_v2_header(&header, LEAF_V2_SEGMENT_FLAG)?;
        if header.pid != meta.header.pid
            || header.object_version != meta.header.object_version
            || header.parent_pid != meta.header.parent_pid
        {
            return Err("ec_spire leaf V2 segment header does not match meta".to_owned());
        }

        let segment_no = u32::from_le_bytes(tail[0..4].try_into().expect("segment no"));
        let row_base = u32::from_le_bytes(tail[4..8].try_into().expect("row base"));
        let row_count = u32::from_le_bytes(tail[8..12].try_into().expect("row count"));
        let next_segment_locator = ItemPointer::decode(&tail[12..18])?;
        validate_leaf_v2_locator(next_segment_locator, "next segment")?;
        if header.assignment_count != row_count {
            return Err(format!(
                "ec_spire leaf V2 segment row count mismatch: header {}, body {row_count}",
                header.assignment_count
            ));
        }
        if row_count == 0 {
            return Err("ec_spire leaf V2 segment row count 0 is invalid".to_owned());
        }
        let row_end = row_base
            .checked_add(row_count)
            .ok_or_else(|| "ec_spire leaf V2 segment row range overflow".to_owned())?;
        if row_end > meta.header.assignment_count {
            return Err(format!(
                "ec_spire leaf V2 segment row range {row_base}..{row_end} exceeds assignment_count {}",
                meta.header.assignment_count
            ));
        }

        Ok(Self {
            segment_no,
            row_base,
            row_count,
            next_segment_locator,
        })
    }

    fn row_end(&self) -> Result<u32, String> {
        self.row_base
            .checked_add(self.row_count)
            .ok_or_else(|| "ec_spire leaf V2 segment row range overflow".to_owned())
    }
}

fn selected_leaf_row_ranges_intersect(
    row_base: u32,
    row_end: u32,
    selected_row_ranges: Option<&[SpireLeafSelectedRowRange]>,
) -> Result<bool, String> {
    let Some(selected_row_ranges) = selected_row_ranges else {
        return Ok(true);
    };
    for selected in selected_row_ranges {
        if selected.row_end < selected.row_base {
            return Err("ec_spire selected leaf V2 row range is invalid".to_owned());
        }
        if selected.row_base < row_end && selected.row_end > row_base {
            return Ok(true);
        }
    }
    Ok(false)
}

fn validate_leaf_v2_meta_against_placement(
    meta: &SpireLeafPartitionObjectV2Meta,
    placement: &SpirePlacementEntry,
) -> Result<(), String> {
    if meta.header.pid != placement.pid {
        return Err(format!(
            "ec_spire placement pid {} does not match leaf V2 pid {}",
            placement.pid, meta.header.pid
        ));
    }
    if meta.header.object_version != placement.object_version {
        return Err(format!(
            "ec_spire placement object_version {} does not match leaf V2 version {}",
            placement.object_version, meta.header.object_version
        ));
    }
    if meta.header.published_epoch_backref == 0
        || meta.header.published_epoch_backref > placement.epoch
    {
        return Err(format!(
            "ec_spire leaf V2 published epoch backref {} is not valid for placement epoch {}",
            meta.header.published_epoch_backref, placement.epoch
        ));
    }
    if u64::from(placement.object_bytes) != meta.object_bytes_total {
        return Err(format!(
            "ec_spire placement object_bytes {} does not match leaf V2 total {}",
            placement.object_bytes, meta.object_bytes_total
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafPartitionObjectV3SummarySegment {
    pub(super) header: SpirePartitionObjectHeader,
    pub(super) segment_no: u32,
    pub(super) summary_base: u32,
    pub(super) next_segment_locator: ItemPointer,
    pub(super) representative_count: u16,
    pub(super) row_bases: Vec<u32>,
    pub(super) row_counts: Vec<u32>,
    pub(super) row_segment_locators: Vec<ItemPointer>,
    pub(super) gammas: Vec<f32>,
    pub(super) payloads: Vec<u8>,
}

impl SpireLeafPartitionObjectV3SummarySegment {
    fn new(
        meta: &SpireLeafPartitionObjectV2Meta,
        segment_no: u32,
        summary_base: u32,
        next_segment_locator: ItemPointer,
        summaries: &[SpireLeafBlockSummary],
    ) -> Result<Self, String> {
        let summary_count = u32::try_from(summaries.len())
            .map_err(|_| "ec_spire leaf V3 summary segment count exceeds u32".to_owned())?;
        let mut row_bases = Vec::with_capacity(summaries.len());
        let mut row_counts = Vec::with_capacity(summaries.len());
        let mut row_segment_locators = Vec::with_capacity(summaries.len());
        let mut gammas = Vec::with_capacity(summaries.len());
        let summary_payload_stride = meta.summary_payload_stride()?;
        let mut payloads = Vec::with_capacity(summary_payload_stride * summaries.len());
        for summary in summaries {
            validate_leaf_block_summary(meta, summary)?;
            row_bases.push(summary.row_base);
            row_counts.push(summary.row_count);
            row_segment_locators.push(summary.row_segment_locator);
            gammas.push(summary.gamma);
            payloads.extend_from_slice(&summary.encoded_payload);
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
                assignment_count: summary_count,
                flags: LEAF_V3_SUMMARY_SEGMENT_FLAG,
            },
            segment_no,
            summary_base,
            next_segment_locator,
            representative_count: meta.summary_representative_count,
            row_bases,
            row_counts,
            row_segment_locators,
            gammas,
            payloads,
        };
        segment.validate_against_meta(meta)?;
        Ok(segment)
    }

    fn summaries(
        &self,
        meta: &SpireLeafPartitionObjectV2Meta,
    ) -> Result<Vec<SpireLeafBlockSummary>, String> {
        self.validate_against_meta(meta)?;
        let summary_count = usize::try_from(self.header.assignment_count)
            .map_err(|_| "ec_spire leaf V3 summary segment count exceeds usize".to_owned())?;
        let payload_stride = meta.summary_payload_stride()?;
        let mut summaries = Vec::with_capacity(summary_count);
        for summary_offset in 0..summary_count {
            let payload_start = summary_offset
                .checked_mul(payload_stride)
                .ok_or_else(|| "ec_spire leaf V3 summary payload offset overflow".to_owned())?;
            let payload_end = payload_start
                .checked_add(payload_stride)
                .ok_or_else(|| "ec_spire leaf V3 summary payload end overflow".to_owned())?;
            summaries.push(SpireLeafBlockSummary {
                row_base: self.row_bases[summary_offset],
                row_count: self.row_counts[summary_offset],
                row_segment_locator: self.row_segment_locators[summary_offset],
                payload_format: meta.payload_format,
                gamma: self.gammas[summary_offset],
                encoded_payload: self.payloads[payload_start..payload_end].to_vec(),
            });
        }
        Ok(summaries)
    }

    fn encode(&self, meta: &SpireLeafPartitionObjectV2Meta) -> Result<Vec<u8>, String> {
        self.validate_against_meta(meta)?;
        let has_row_segment_locators = self
            .row_segment_locators
            .iter()
            .any(|locator| *locator != ItemPointer::INVALID);
        let format_version = if has_row_segment_locators {
            PARTITION_OBJECT_FORMAT_VERSION_V5
        } else if self.representative_count > 1 {
            PARTITION_OBJECT_FORMAT_VERSION_V4
        } else {
            PARTITION_OBJECT_FORMAT_VERSION_V3
        };
        let mut out = self.header.encode_after_validation(format_version);
        out.extend_from_slice(&self.segment_no.to_le_bytes());
        out.extend_from_slice(&self.summary_base.to_le_bytes());
        out.extend_from_slice(&self.header.assignment_count.to_le_bytes());
        self.next_segment_locator.encode_into(&mut out);
        if self.representative_count > 1 || has_row_segment_locators {
            out.extend_from_slice(&self.representative_count.to_le_bytes());
            out.extend_from_slice(&0_u16.to_le_bytes());
        }
        for row_base in &self.row_bases {
            out.extend_from_slice(&row_base.to_le_bytes());
        }
        for row_count in &self.row_counts {
            out.extend_from_slice(&row_count.to_le_bytes());
        }
        for gamma in &self.gammas {
            out.extend_from_slice(&gamma.to_le_bytes());
        }
        if has_row_segment_locators {
            for locator in &self.row_segment_locators {
                locator.encode_into(&mut out);
            }
        }
        out.extend_from_slice(&self.payloads);
        Ok(out)
    }

    fn decode(input: &[u8], meta: &SpireLeafPartitionObjectV2Meta) -> Result<Self, String> {
        let (header, format_version, tail) =
            SpirePartitionObjectHeader::decode_prefix_with_format_version(input)?;
        if format_version != PARTITION_OBJECT_FORMAT_VERSION_V3
            && format_version != PARTITION_OBJECT_FORMAT_VERSION_V4
            && format_version != PARTITION_OBJECT_FORMAT_VERSION_V5
        {
            return Err(format!(
                "ec_spire leaf V3/V4/V5 summary segment format version must be 3, 4, or 5, got {format_version}"
            ));
        }
        let prefix_bytes = if format_version == PARTITION_OBJECT_FORMAT_VERSION_V4
            || format_version == PARTITION_OBJECT_FORMAT_VERSION_V5
        {
            LEAF_V3_SUMMARY_SEGMENT_PREFIX_BYTES
                .checked_add(LEAF_V4_SUMMARY_SEGMENT_REPRESENTATIVE_PREFIX_BYTES)
                .ok_or_else(|| "ec_spire leaf V4 summary segment prefix overflow".to_owned())?
        } else {
            LEAF_V3_SUMMARY_SEGMENT_PREFIX_BYTES
        };
        if tail.len() < prefix_bytes {
            return Err(format!(
                "ec_spire leaf V3 summary segment too short: got {}, expected at least {}",
                tail.len(),
                prefix_bytes
            ));
        }
        let segment_no = u32::from_le_bytes(tail[0..4].try_into().expect("segment no"));
        let summary_base = u32::from_le_bytes(tail[4..8].try_into().expect("summary base"));
        let summary_count = u32::from_le_bytes(tail[8..12].try_into().expect("summary count"));
        let next_segment_locator = ItemPointer::decode(&tail[12..18])?;
        if header.assignment_count != summary_count {
            return Err(format!(
                "ec_spire leaf V3 summary count mismatch: header {}, body {summary_count}",
                header.assignment_count
            ));
        }
        let representative_count = if format_version == PARTITION_OBJECT_FORMAT_VERSION_V4
            || format_version == PARTITION_OBJECT_FORMAT_VERSION_V5
        {
            let representative_count =
                u16::from_le_bytes(tail[18..20].try_into().expect("representative count"));
            let reserved = u16::from_le_bytes(tail[20..22].try_into().expect("V4 reserved"));
            if reserved != 0 {
                return Err(format!(
                    "ec_spire leaf V4 summary segment reserved bytes must be zero, got {reserved}"
                ));
            }
            representative_count
        } else {
            1
        };

        let summary_count_usize = usize::try_from(summary_count)
            .map_err(|_| "ec_spire leaf V3 summary count exceeds usize".to_owned())?;
        let row_bases_bytes = summary_count_usize
            .checked_mul(size_of::<u32>())
            .ok_or_else(|| "ec_spire leaf V3 row_base byte length overflow".to_owned())?;
        let row_counts_bytes = summary_count_usize
            .checked_mul(size_of::<u32>())
            .ok_or_else(|| "ec_spire leaf V3 row_count byte length overflow".to_owned())?;
        let gamma_bytes = summary_count_usize
            .checked_mul(size_of::<f32>())
            .ok_or_else(|| "ec_spire leaf V3 gamma byte length overflow".to_owned())?;
        let locator_bytes = if format_version == PARTITION_OBJECT_FORMAT_VERSION_V5 {
            summary_count_usize
                .checked_mul(LEAF_V5_SUMMARY_SEGMENT_LOCATOR_BYTES_PER_SUMMARY)
                .ok_or_else(|| {
                    "ec_spire leaf V5 row segment locator byte length overflow".to_owned()
                })?
        } else {
            0
        };
        let payload_bytes = summary_count_usize
            .checked_mul(meta.summary_payload_stride()?)
            .ok_or_else(|| "ec_spire leaf V3 payload byte length overflow".to_owned())?;
        let expected_tail_len = prefix_bytes
            .checked_add(row_bases_bytes)
            .and_then(|len| len.checked_add(row_counts_bytes))
            .and_then(|len| len.checked_add(gamma_bytes))
            .and_then(|len| len.checked_add(locator_bytes))
            .and_then(|len| len.checked_add(payload_bytes))
            .ok_or_else(|| "ec_spire leaf V3 summary segment length overflow".to_owned())?;
        if tail.len() != expected_tail_len {
            return Err(format!(
                "ec_spire leaf V3 summary segment length mismatch: got {}, expected {expected_tail_len}",
                tail.len()
            ));
        }

        let mut cursor = prefix_bytes;
        let mut row_bases = Vec::with_capacity(summary_count_usize);
        for chunk in tail[cursor..cursor + row_bases_bytes].chunks_exact(size_of::<u32>()) {
            row_bases.push(u32::from_le_bytes(chunk.try_into().expect("row_base bytes")));
        }
        cursor += row_bases_bytes;
        let mut row_counts = Vec::with_capacity(summary_count_usize);
        for chunk in tail[cursor..cursor + row_counts_bytes].chunks_exact(size_of::<u32>()) {
            row_counts.push(u32::from_le_bytes(chunk.try_into().expect("row_count bytes")));
        }
        cursor += row_counts_bytes;
        let mut gammas = Vec::with_capacity(summary_count_usize);
        for chunk in tail[cursor..cursor + gamma_bytes].chunks_exact(size_of::<f32>()) {
            let gamma = f32::from_le_bytes(chunk.try_into().expect("gamma bytes"));
            if !gamma.is_finite() {
                return Err("ec_spire leaf V3 summary gamma must be finite".to_owned());
            }
            gammas.push(gamma);
        }
        cursor += gamma_bytes;
        let row_segment_locators = if format_version == PARTITION_OBJECT_FORMAT_VERSION_V5 {
            let locators = tail[cursor..cursor + locator_bytes]
                .chunks_exact(ITEM_POINTER_BYTES)
                .map(ItemPointer::decode)
                .collect::<Result<Vec<_>, _>>()?;
            cursor += locator_bytes;
            locators
        } else {
            vec![ItemPointer::INVALID; summary_count_usize]
        };
        let payloads = tail[cursor..cursor + payload_bytes].to_vec();

        let segment = Self {
            header,
            segment_no,
            summary_base,
            next_segment_locator,
            representative_count,
            row_bases,
            row_counts,
            row_segment_locators,
            gammas,
            payloads,
        };
        segment.validate_against_meta(meta)?;
        Ok(segment)
    }

    fn validate_against_meta(&self, meta: &SpireLeafPartitionObjectV2Meta) -> Result<(), String> {
        validate_leaf_v2_header(&self.header, LEAF_V3_SUMMARY_SEGMENT_FLAG)?;
        if self.header.pid != meta.header.pid
            || self.header.object_version != meta.header.object_version
            || self.header.parent_pid != meta.header.parent_pid
        {
            return Err("ec_spire leaf V3 summary segment header does not match meta".to_owned());
        }
        validate_leaf_v2_locator(self.next_segment_locator, "next summary segment")?;
        if self.representative_count != meta.summary_representative_count {
            return Err(format!(
                "ec_spire leaf V4 summary representative count mismatch: segment {}, meta {}",
                self.representative_count, meta.summary_representative_count
            ));
        }
        let summary_count = usize::try_from(self.header.assignment_count)
            .map_err(|_| "ec_spire leaf V3 summary segment count exceeds usize".to_owned())?;
        if summary_count == 0 {
            return Err("ec_spire leaf V3 summary segment count 0 is invalid".to_owned());
        }
        if self.row_bases.len() != summary_count {
            return Err(format!(
                "ec_spire leaf V3 summary row_base length mismatch: got {}, expected {summary_count}",
                self.row_bases.len()
            ));
        }
        if self.row_counts.len() != summary_count {
            return Err(format!(
                "ec_spire leaf V3 summary row_count length mismatch: got {}, expected {summary_count}",
                self.row_counts.len()
            ));
        }
        if self.row_segment_locators.len() != summary_count {
            return Err(format!(
                "ec_spire leaf V5 summary row segment locator length mismatch: got {}, expected {summary_count}",
                self.row_segment_locators.len()
            ));
        }
        for locator in &self.row_segment_locators {
            validate_leaf_v2_locator(*locator, "summary row segment")?;
        }
        if self.gammas.len() != summary_count {
            return Err(format!(
                "ec_spire leaf V3 summary gamma length mismatch: got {}, expected {summary_count}",
                self.gammas.len()
            ));
        }
        if self.gammas.iter().any(|gamma| !gamma.is_finite()) {
            return Err("ec_spire leaf V3 summary gamma must be finite".to_owned());
        }
        let expected_payload_bytes = summary_count
            .checked_mul(meta.summary_payload_stride()?)
            .ok_or_else(|| "ec_spire leaf V3 summary payload length overflow".to_owned())?;
        if self.payloads.len() != expected_payload_bytes {
            return Err(format!(
                "ec_spire leaf V3 summary payload length mismatch: got {}, expected {expected_payload_bytes}",
                self.payloads.len()
            ));
        }
        for summary_offset in 0..summary_count {
            let row_count = self.row_counts[summary_offset];
            if row_count == 0 {
                return Err("ec_spire leaf V3 summary row_count 0 is invalid".to_owned());
            }
            let row_end = self.row_bases[summary_offset]
                .checked_add(row_count)
                .ok_or_else(|| "ec_spire leaf V3 summary row range overflow".to_owned())?;
            if row_end > meta.header.assignment_count {
                return Err(format!(
                    "ec_spire leaf V3 summary row range {}..{} exceeds assignment_count {}",
                    self.row_bases[summary_offset], row_end, meta.header.assignment_count
                ));
            }
        }
        Ok(())
    }
}

fn validate_leaf_block_summary(
    meta: &SpireLeafPartitionObjectV2Meta,
    summary: &SpireLeafBlockSummary,
) -> Result<(), String> {
    validate_leaf_v2_locator(summary.row_segment_locator, "summary row segment")?;
    if summary.payload_format != meta.payload_format {
        return Err(format!(
            "ec_spire leaf V3 summary payload format mismatch: got {}, expected {}",
            summary.payload_format, meta.payload_format
        ));
    }
    if summary.row_count == 0 {
        return Err("ec_spire leaf V3 summary row_count 0 is invalid".to_owned());
    }
    let row_end = summary
        .row_base
        .checked_add(summary.row_count)
        .ok_or_else(|| "ec_spire leaf V3 summary row range overflow".to_owned())?;
    if row_end > meta.header.assignment_count {
        return Err(format!(
            "ec_spire leaf V3 summary row range {}..{} exceeds assignment_count {}",
            summary.row_base, row_end, meta.header.assignment_count
        ));
    }
    if !summary.gamma.is_finite() {
        return Err("ec_spire leaf V3 summary gamma must be finite".to_owned());
    }
    let summary_payload_stride = meta.summary_payload_stride()?;
    if summary.encoded_payload.len() != summary_payload_stride {
        return Err(format!(
            "ec_spire leaf V3 summary payload stride mismatch: got {}, expected {}",
            summary.encoded_payload.len(),
            summary_payload_stride
        ));
    }
    Ok(())
}

fn attach_leaf_block_summary_row_segment_locators(
    summaries: &[SpireLeafBlockSummary],
    max_segment_rows: usize,
    segment_locators_by_index: &[ItemPointer],
) -> Result<Vec<SpireLeafBlockSummary>, String> {
    if max_segment_rows == 0 {
        return Err("ec_spire leaf V5 max segment rows 0 is invalid".to_owned());
    }
    let mut with_locators = Vec::with_capacity(summaries.len());
    for summary in summaries {
        let segment_index = usize::try_from(summary.row_base)
            .map_err(|_| "ec_spire leaf V5 summary row_base exceeds usize".to_owned())?
            .checked_div(max_segment_rows)
            .ok_or_else(|| "ec_spire leaf V5 segment index calculation failed".to_owned())?;
        let row_segment_locator = *segment_locators_by_index.get(segment_index).ok_or_else(|| {
            format!(
                "ec_spire leaf V5 summary row_base {} maps to missing segment index {}",
                summary.row_base, segment_index
            )
        })?;
        if row_segment_locator == ItemPointer::INVALID {
            return Err(format!(
                "ec_spire leaf V5 summary row_base {} maps to invalid segment locator",
                summary.row_base
            ));
        }
        let mut summary = summary.clone();
        summary.row_segment_locator = row_segment_locator;
        with_locators.push(summary);
    }
    Ok(with_locators)
}

pub(super) fn leaf_block_summary_representative_count(
    summaries: &[SpireLeafBlockSummary],
    payload_stride: usize,
) -> Result<u16, String> {
    if summaries.is_empty() {
        return Ok(0);
    }
    if payload_stride == 0 {
        return Err("ec_spire leaf V4 summary representative payload stride 0".to_owned());
    }
    let mut representative_count = None;
    for summary in summaries {
        if summary.encoded_payload.is_empty() || summary.encoded_payload.len() % payload_stride != 0
        {
            return Err(format!(
                "ec_spire leaf V4 summary payload length {} is not a positive multiple of stride {payload_stride}",
                summary.encoded_payload.len()
            ));
        }
        let count = summary.encoded_payload.len() / payload_stride;
        if count > 8 {
            return Err(format!(
                "ec_spire leaf V4 summary representative count {count} exceeds 8"
            ));
        }
        match representative_count {
            Some(expected) if expected != count => {
                return Err(format!(
                    "ec_spire leaf V4 summary representative count mismatch: got {count}, expected {expected}"
                ));
            }
            Some(_) => {}
            None => representative_count = Some(count),
        }
    }
    let count = representative_count.unwrap_or(0);
    u16::try_from(count)
        .map_err(|_| "ec_spire leaf V4 summary representative count exceeds u16".to_owned())
}

fn validate_leaf_block_summary_coverage(
    meta: &SpireLeafPartitionObjectV2Meta,
    summaries: &[SpireLeafBlockSummary],
) -> Result<(), String> {
    let summary_count = u32::try_from(summaries.len())
        .map_err(|_| "ec_spire leaf V3 summary count exceeds u32".to_owned())?;
    if meta.summary_count != summary_count {
        return Err(format!(
            "ec_spire leaf V3 summary count mismatch: meta {}, summaries {summary_count}",
            meta.summary_count
        ));
    }
    let mut expected_row_base = 0_u32;
    for summary in summaries {
        validate_leaf_block_summary(meta, summary)?;
        if summary.row_base != expected_row_base {
            return Err(format!(
                "ec_spire leaf V3 summary row_base mismatch: got {}, expected {expected_row_base}",
                summary.row_base
            ));
        }
        expected_row_base = expected_row_base
            .checked_add(summary.row_count)
            .ok_or_else(|| "ec_spire leaf V3 summary row range overflow".to_owned())?;
    }
    if expected_row_base != meta.header.assignment_count {
        return Err(format!(
            "ec_spire leaf V3 summary row coverage mismatch: covered {expected_row_base}, assignments {}",
            meta.header.assignment_count
        ));
    }
    Ok(())
}
