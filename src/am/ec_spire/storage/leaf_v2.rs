#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafPartitionObjectV2 {
    pub(super) meta: SpireLeafPartitionObjectV2Meta,
    pub(super) segments: Vec<SpireLeafPartitionObjectV2Segment>,
}

impl SpireLeafPartitionObjectV2 {
    fn new(
        meta: SpireLeafPartitionObjectV2Meta,
        segments: Vec<SpireLeafPartitionObjectV2Segment>,
    ) -> Result<Self, String> {
        let object = Self { meta, segments };
        object.validate()?;
        Ok(object)
    }

    fn validate(&self) -> Result<(), String> {
        let segment_count = u32::try_from(self.segments.len())
            .map_err(|_| "ec_spire leaf V2 segment count exceeds u32".to_owned())?;
        if self.meta.segment_count != segment_count {
            return Err(format!(
                "ec_spire leaf V2 segment count mismatch: meta {}, segments {segment_count}",
                self.meta.segment_count
            ));
        }
        let mut expected_row_base = 0_u32;
        for (expected_segment_no, segment) in self.segments.iter().enumerate() {
            segment.validate_against_meta(&self.meta)?;
            let expected_segment_no = u32::try_from(expected_segment_no)
                .map_err(|_| "ec_spire leaf V2 segment index exceeds u32".to_owned())?;
            if segment.segment_no != expected_segment_no {
                return Err(format!(
                    "ec_spire leaf V2 segment number mismatch: got {}, expected {expected_segment_no}",
                    segment.segment_no
                ));
            }
            if segment.row_base != expected_row_base {
                return Err(format!(
                    "ec_spire leaf V2 segment row_base mismatch: got {}, expected {expected_row_base}",
                    segment.row_base
                ));
            }
            expected_row_base = expected_row_base
                .checked_add(segment.header.assignment_count)
                .ok_or_else(|| "ec_spire leaf V2 assignment count overflow".to_owned())?;
            if expected_segment_no + 1 == segment_count {
                if segment.next_segment_locator != ItemPointer::INVALID {
                    return Err(
                        "ec_spire leaf V2 final segment next locator must be invalid".to_owned(),
                    );
                }
            } else if segment.next_segment_locator == ItemPointer::INVALID {
                return Err("ec_spire leaf V2 non-final segment requires next locator".to_owned());
            }
        }
        if self.meta.header.assignment_count != expected_row_base {
            return Err(format!(
                "ec_spire leaf V2 assignment count mismatch: meta {}, segments {expected_row_base}",
                self.meta.header.assignment_count
            ));
        }
        Ok(())
    }

    pub(super) fn column_segments(
        &self,
    ) -> Result<impl Iterator<Item = Result<SpireLeafObjectColumns<'_>, String>> + '_, String> {
        self.validate()?;
        Ok(self
            .segments
            .iter()
            .map(|segment| segment.columns(&self.meta)))
    }

    pub(super) fn assignment_rows(&self) -> Result<Vec<SpireLeafAssignmentRow>, String> {
        let row_count = usize::try_from(self.meta.header.assignment_count)
            .map_err(|_| "ec_spire leaf V2 assignment count exceeds usize".to_owned())?;
        let mut rows = Vec::with_capacity(row_count);
        for columns in self.column_segments()? {
            let columns = columns?;
            for row_offset in 0..columns.row_count() {
                let row = columns.row(row_offset)?;
                rows.push(SpireLeafAssignmentRow {
                    flags: row.flags,
                    vec_id: SpireVecId::local(row.local_vec_seq()?),
                    heap_tid: row.heap_tid,
                    payload_format: columns.payload_format,
                    gamma: row.gamma,
                    encoded_payload: row.encoded_payload.to_vec(),
                });
            }
        }
        Ok(rows)
    }
}
