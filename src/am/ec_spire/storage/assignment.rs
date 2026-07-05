#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafAssignmentRow {
    pub(super) flags: u16,
    pub(super) vec_id: SpireVecId,
    pub(super) heap_tid: ItemPointer,
    pub(super) payload_format: u8,
    pub(super) gamma: f32,
    pub(super) encoded_payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SpireLeafAssignmentRowRef<'a> {
    pub(super) flags: u16,
    pub(super) vec_id: SpireVecIdRef<'a>,
    pub(super) heap_tid: ItemPointer,
    pub(super) payload_format: u8,
    pub(super) gamma: f32,
    pub(super) encoded_payload: &'a [u8],
}

impl<'a> SpireLeafAssignmentRowRef<'a> {
    pub(super) fn to_owned(self) -> SpireLeafAssignmentRow {
        SpireLeafAssignmentRow {
            flags: self.flags,
            vec_id: self.vec_id.to_owned(),
            heap_tid: self.heap_tid,
            payload_format: self.payload_format,
            gamma: self.gamma,
            encoded_payload: self.encoded_payload.to_vec(),
        }
    }
}

impl SpireLeafAssignmentRow {
    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate_wire_shape()?;
        Ok(self.encode_after_validation())
    }

    fn validate_wire_shape(&self) -> Result<(), String> {
        validate_assignment_flags(self.flags)?;
        validate_assignment_payload_format(self.payload_format)?;
        validate_vec_id_bytes(self.vec_id.as_bytes())?;
        if self.heap_tid == ItemPointer::INVALID {
            return Err("ec_spire assignment row heap_tid must be valid".to_owned());
        }
        if !self.gamma.is_finite() {
            return Err("ec_spire assignment row gamma must be finite".to_owned());
        }
        u8::try_from(self.vec_id.as_bytes().len())
            .map_err(|_| "ec_spire vec_id length exceeds u8".to_owned())?;
        u32::try_from(self.encoded_payload.len())
            .map_err(|_| "ec_spire assignment payload length exceeds u32".to_owned())?;
        self.encoded_len_after_validation()?;
        Ok(())
    }

    fn encoded_len_after_validation(&self) -> Result<usize, String> {
        ASSIGNMENT_ROW_FIXED_PREFIX_BYTES
            .checked_add(self.vec_id.as_bytes().len())
            .and_then(|len| len.checked_add(ASSIGNMENT_ROW_FIXED_TAIL_BYTES))
            .and_then(|len| len.checked_add(self.encoded_payload.len()))
            .ok_or_else(|| "ec_spire assignment row encoded length overflow".to_owned())
    }

    fn encode_after_validation(&self) -> Vec<u8> {
        let encoded_len = self
            .encoded_len_after_validation()
            .expect("assignment row was validated before encoding");
        let vec_id_len = u8::try_from(self.vec_id.as_bytes().len())
            .expect("assignment row vec_id length was validated");
        let payload_len = u32::try_from(self.encoded_payload.len())
            .expect("assignment row payload length was validated");

        let mut out = Vec::with_capacity(encoded_len);
        out.extend_from_slice(&self.flags.to_le_bytes());
        out.push(vec_id_len);
        out.extend_from_slice(self.vec_id.as_bytes());
        self.heap_tid.encode_into(&mut out);
        out.push(self.payload_format);
        out.extend_from_slice(&self.gamma.to_le_bytes());
        out.extend_from_slice(&payload_len.to_le_bytes());
        out.extend_from_slice(&self.encoded_payload);
        out
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        let (row, tail) = Self::decode_prefix(input)?;
        if !tail.is_empty() {
            return Err(format!(
                "ec_spire assignment row length mismatch: got trailing {} bytes",
                tail.len()
            ));
        }
        Ok(row)
    }

    fn decode_prefix(input: &[u8]) -> Result<(Self, &[u8]), String> {
        let (row_ref, tail) = Self::decode_prefix_ref(input)?;
        Ok((row_ref.to_owned(), tail))
    }

    pub(super) fn decode_prefix_ref(
        input: &[u8],
    ) -> Result<(SpireLeafAssignmentRowRef<'_>, &[u8]), String> {
        if input.len() < ASSIGNMENT_ROW_FIXED_PREFIX_BYTES + ASSIGNMENT_ROW_FIXED_TAIL_BYTES {
            return Err(format!(
                "ec_spire assignment row too short: got {}, expected at least {}",
                input.len(),
                ASSIGNMENT_ROW_FIXED_PREFIX_BYTES + ASSIGNMENT_ROW_FIXED_TAIL_BYTES
            ));
        }
        let flags = u16::from_le_bytes(input[0..2].try_into().expect("assignment flags bytes"));
        validate_assignment_flags(flags)?;
        let vec_id_len = input[2] as usize;
        if vec_id_len == 0 || vec_id_len > SPIRE_VEC_ID_MAX_BYTES {
            return Err(format!(
                "ec_spire assignment row vec_id length {vec_id_len} is invalid"
            ));
        }
        let min_len =
            ASSIGNMENT_ROW_FIXED_PREFIX_BYTES + vec_id_len + ASSIGNMENT_ROW_FIXED_TAIL_BYTES;
        if input.len() < min_len {
            return Err(format!(
                "ec_spire assignment row length {} is too short for vec_id length {vec_id_len}",
                input.len()
            ));
        }

        let vec_id_start = ASSIGNMENT_ROW_FIXED_PREFIX_BYTES;
        let vec_id_end = vec_id_start + vec_id_len;
        let heap_tid_start = vec_id_end;
        let heap_tid_end = heap_tid_start + ITEM_POINTER_BYTES;
        let payload_format_offset = heap_tid_end;
        let gamma_start = payload_format_offset + 1;
        let gamma_end = gamma_start + size_of::<f32>();
        let payload_len_start = gamma_end;
        let payload_len_end = payload_len_start + size_of::<u32>();

        let heap_tid = ItemPointer::decode(&input[heap_tid_start..heap_tid_end])?;
        if heap_tid == ItemPointer::INVALID {
            return Err("ec_spire assignment row heap_tid must be valid".to_owned());
        }
        let payload_format = input[payload_format_offset];
        validate_assignment_payload_format(payload_format)?;
        let gamma = f32::from_le_bytes(input[gamma_start..gamma_end].try_into().expect("gamma"));
        if !gamma.is_finite() {
            return Err("ec_spire assignment row gamma must be finite".to_owned());
        }
        let payload_len = u32::from_le_bytes(
            input[payload_len_start..payload_len_end]
                .try_into()
                .expect("payload len"),
        ) as usize;
        let expected_len = payload_len_end + payload_len;
        if input.len() < expected_len {
            return Err(format!(
                "ec_spire assignment row length {} is too short for payload length {payload_len}",
                input.len()
            ));
        }

        Ok((
            SpireLeafAssignmentRowRef {
                flags,
                vec_id: SpireVecIdRef::from_bytes(&input[vec_id_start..vec_id_end])?,
                heap_tid,
                payload_format,
                gamma,
                encoded_payload: &input[payload_len_end..expected_len],
            },
            &input[expected_len..],
        ))
    }
}
