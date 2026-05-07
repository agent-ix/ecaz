fn validate_vec_id_bytes(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() {
        return Err("ec_spire vec_id must not be empty".to_owned());
    }
    if bytes.len() > SPIRE_VEC_ID_MAX_BYTES {
        return Err(format!(
            "ec_spire vec_id length {} exceeds max {SPIRE_VEC_ID_MAX_BYTES}",
            bytes.len()
        ));
    }
    match bytes[0] {
        SPIRE_LOCAL_VEC_ID_DISCRIMINATOR => {
            if bytes.len() != 1 + size_of::<u64>() {
                return Err(format!(
                    "ec_spire local vec_id length mismatch: got {}, expected {}",
                    bytes.len(),
                    1 + size_of::<u64>()
                ));
            }
            let local_vec_seq =
                u64::from_le_bytes(bytes[1..].try_into().expect("local vec_id sequence bytes"));
            if local_vec_seq == 0 {
                return Err("ec_spire local vec_id sequence 0 is invalid".to_owned());
            }
        }
        SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR => {
            if bytes.len() == 1 {
                return Err("ec_spire global vec_id payload must not be empty".to_owned());
            }
        }
        other => {
            return Err(format!("ec_spire unknown vec_id discriminator: {other:#x}"));
        }
    }
    Ok(())
}

pub(super) fn is_visible_primary_assignment_flags(flags: u16) -> bool {
    let blocked_flags = SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA
        | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE
        | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE
        | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR;
    flags & SPIRE_ASSIGNMENT_FLAG_PRIMARY != 0 && flags & blocked_flags == 0
}

pub(super) fn is_visible_primary_assignment(assignment: &SpireLeafAssignmentRow) -> bool {
    is_visible_primary_assignment_flags(assignment.flags)
}

pub(super) fn is_visible_primary_assignment_ref(
    assignment: &SpireLeafAssignmentRowRef<'_>,
) -> bool {
    is_visible_primary_assignment_flags(assignment.flags)
}

pub(super) fn is_delete_delta_assignment_flags(flags: u16) -> bool {
    flags & SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE != 0
}

pub(super) fn is_delete_delta_assignment(assignment: &SpireLeafAssignmentRow) -> bool {
    is_delete_delta_assignment_flags(assignment.flags)
}

fn validate_leaf_v2_header(
    header: &SpirePartitionObjectHeader,
    expected_flag: u32,
) -> Result<(), String> {
    header.validate_for_format_version(PARTITION_OBJECT_FORMAT_VERSION_V2)?;
    if header.kind != SpirePartitionObjectKind::Leaf {
        return Err(format!(
            "ec_spire leaf V2 header kind must be Leaf, got {:?}",
            header.kind
        ));
    }
    if header.child_count != 0 {
        return Err(format!(
            "ec_spire leaf V2 child_count must be 0, got {}",
            header.child_count
        ));
    }
    if header.flags != expected_flag {
        return Err(format!(
            "ec_spire leaf V2 header flags mismatch: got {:#x}, expected {expected_flag:#x}",
            header.flags
        ));
    }
    Ok(())
}

fn validate_leaf_v2_locator(locator: ItemPointer, label: &str) -> Result<(), String> {
    if locator == ItemPointer::INVALID {
        return Ok(());
    }
    if locator.offset_number == 0 {
        return Err(format!(
            "ec_spire leaf V2 {label} locator offset 0 is invalid"
        ));
    }
    if locator.block_number == u32::MAX || locator.offset_number == u16::MAX {
        return Err(format!(
            "ec_spire leaf V2 {label} locator is partially invalid"
        ));
    }
    Ok(())
}

fn encode_leaf_v2_local_vec_id(vec_id: &SpireVecId, out: &mut Vec<u8>) -> Result<(), String> {
    let Some(local_vec_seq) = vec_id.local_sequence() else {
        return Err("ec_spire leaf V2 Phase 1 requires local vec_id rows".to_owned());
    };
    out.push(SPIRE_LOCAL_VEC_ID_DISCRIMINATOR);
    out.extend_from_slice(&local_vec_seq.to_le_bytes());
    out.resize(
        out.len() + (LEAF_V2_LOCAL_VEC_ID_STRIDE - 1 - size_of::<u64>()),
        0,
    );
    Ok(())
}

fn decode_leaf_v2_local_vec_id(input: &[u8]) -> Result<u64, String> {
    if input.len() != LEAF_V2_LOCAL_VEC_ID_STRIDE {
        return Err(format!(
            "ec_spire leaf V2 local vec_id stride mismatch: got {}, expected {LEAF_V2_LOCAL_VEC_ID_STRIDE}",
            input.len()
        ));
    }
    if input[0] != SPIRE_LOCAL_VEC_ID_DISCRIMINATOR {
        return Err(format!(
            "ec_spire leaf V2 local vec_id discriminator mismatch: got {:#x}",
            input[0]
        ));
    }
    if input[1 + size_of::<u64>()..].iter().any(|byte| *byte != 0) {
        return Err("ec_spire leaf V2 local vec_id padding must be zero".to_owned());
    }
    let local_vec_seq = u64::from_le_bytes(
        input[1..1 + size_of::<u64>()]
            .try_into()
            .expect("local vec_id bytes"),
    );
    if local_vec_seq == 0 {
        return Err("ec_spire leaf V2 local vec_id sequence 0 is invalid".to_owned());
    }
    Ok(local_vec_seq)
}

fn leaf_v2_payload_layout(assignments: &[SpireLeafAssignmentRow]) -> Result<(u8, usize), String> {
    let Some(first) = assignments.first() else {
        return Ok((SPIRE_PAYLOAD_FORMAT_NONE, 0));
    };
    validate_leaf_assignment(first)?;
    if first.payload_format == SPIRE_PAYLOAD_FORMAT_NONE {
        return Err("ec_spire non-empty leaf V2 payload format must not be NONE".to_owned());
    }
    let payload_format = first.payload_format;
    let payload_stride = first.encoded_payload.len();
    if payload_stride == 0 {
        return Err("ec_spire non-empty leaf V2 payload stride 0 is invalid".to_owned());
    }
    for assignment in assignments {
        validate_leaf_assignment(assignment)?;
        if assignment.payload_format != payload_format {
            return Err(format!(
                "ec_spire leaf V2 requires one payload format per object: got {}, expected {payload_format}",
                assignment.payload_format
            ));
        }
        if assignment.encoded_payload.len() != payload_stride {
            return Err(format!(
                "ec_spire leaf V2 requires one payload stride per object: got {}, expected {payload_stride}",
                assignment.encoded_payload.len()
            ));
        }
        if assignment.vec_id.local_sequence().is_none() {
            return Err("ec_spire leaf V2 Phase 1 requires local vec_id rows".to_owned());
        }
    }
    Ok((payload_format, payload_stride))
}

fn leaf_v2_max_segment_rows(
    page_size: usize,
    payload_stride: usize,
    vec_id_stride: usize,
) -> Result<usize, String> {
    let row_bytes = size_of::<u16>()
        .checked_add(vec_id_stride)
        .and_then(|len| len.checked_add(ITEM_POINTER_BYTES))
        .and_then(|len| len.checked_add(size_of::<f32>()))
        .and_then(|len| len.checked_add(payload_stride))
        .ok_or_else(|| "ec_spire leaf V2 row byte length overflow".to_owned())?;
    if row_bytes == 0 {
        return Ok(usize::MAX);
    }
    let fixed_bytes = PARTITION_OBJECT_HEADER_BYTES
        .checked_add(LEAF_V2_SEGMENT_PREFIX_BYTES)
        .ok_or_else(|| "ec_spire leaf V2 segment fixed byte length overflow".to_owned())?;
    let usable_bytes = usable_page_bytes(page_size);
    if fixed_bytes >= usable_bytes {
        return Err(format!(
            "ec_spire leaf V2 segment fixed bytes {fixed_bytes} exceed page usable bytes {usable_bytes}"
        ));
    }
    let mut rows = (usable_bytes - fixed_bytes) / row_bytes;
    while rows > 0
        && !element_or_neighbor_tuple_fits(fixed_bytes + row_bytes.saturating_mul(rows), page_size)
    {
        rows -= 1;
    }
    if rows == 0 {
        return Err(format!(
            "ec_spire leaf V2 row bytes {row_bytes} do not fit page size {page_size}"
        ));
    }
    Ok(rows)
}

fn validate_assignment_flags(flags: u16) -> Result<(), String> {
    let unknown = flags & !SPIRE_ASSIGNMENT_KNOWN_FLAGS;
    if unknown != 0 {
        return Err(format!(
            "ec_spire unknown assignment row flags: {unknown:#x}"
        ));
    }
    Ok(())
}

fn validate_assignment_payload_format(payload_format: u8) -> Result<(), String> {
    match payload_format {
        SPIRE_PAYLOAD_FORMAT_NONE
        | SPIRE_PAYLOAD_FORMAT_TURBOQUANT
        | SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN
        | SPIRE_PAYLOAD_FORMAT_RABITQ => Ok(()),
        other => Err(format!(
            "ec_spire unknown assignment payload_format: {other}"
        )),
    }
}

fn validate_scored_assignment_payload(assignment: &SpireLeafAssignmentRow) -> Result<(), String> {
    if assignment.payload_format == SPIRE_PAYLOAD_FORMAT_NONE {
        return Err("ec_spire scored assignment payload_format must not be 0".to_owned());
    }
    if assignment.encoded_payload.is_empty() {
        return Err("ec_spire scored assignment payload must not be empty".to_owned());
    }
    Ok(())
}

fn validate_delta_assignment(assignment: &SpireLeafAssignmentRow) -> Result<(), String> {
    assignment.validate_wire_shape()?;
    let is_insert = assignment.flags & SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT != 0;
    let is_delete = assignment.flags & SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE != 0;
    if is_insert == is_delete {
        return Err(
            "ec_spire delta assignment must set exactly one insert/delete delta flag".to_owned(),
        );
    }
    if assignment.flags & SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA != 0 {
        return Err("ec_spire delta assignment cannot be a boundary replica in Phase 1".to_owned());
    }
    if assignment.flags & SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR != 0 {
        return Err("ec_spire delta assignment cannot be a stale locator".to_owned());
    }
    if is_insert && assignment.flags & SPIRE_ASSIGNMENT_FLAG_PRIMARY == 0 {
        return Err("ec_spire insert delta assignment must be primary".to_owned());
    }
    if is_insert && assignment.flags & SPIRE_ASSIGNMENT_FLAG_TOMBSTONE != 0 {
        return Err("ec_spire insert delta assignment cannot be tombstoned".to_owned());
    }
    if is_insert {
        validate_scored_assignment_payload(assignment)?;
    }
    if is_delete && assignment.flags & SPIRE_ASSIGNMENT_FLAG_PRIMARY != 0 {
        return Err("ec_spire delete delta assignment cannot be primary".to_owned());
    }
    if is_delete && assignment.flags & SPIRE_ASSIGNMENT_FLAG_TOMBSTONE == 0 {
        return Err("ec_spire delete delta assignment must be tombstoned".to_owned());
    }
    if is_delete && assignment.payload_format != SPIRE_PAYLOAD_FORMAT_NONE {
        return Err("ec_spire delete delta assignment payload format must be 0".to_owned());
    }
    if is_delete && assignment.gamma != 0.0 {
        return Err("ec_spire delete delta assignment gamma must be 0".to_owned());
    }
    if is_delete && !assignment.encoded_payload.is_empty() {
        return Err("ec_spire delete delta assignment payload must be empty".to_owned());
    }
    Ok(())
}

fn validate_leaf_assignments(assignments: &[SpireLeafAssignmentRow]) -> Result<(), String> {
    let mut seen_vec_ids = HashSet::new();
    for assignment in assignments {
        validate_leaf_assignment(assignment)?;
        if !seen_vec_ids.insert(assignment.vec_id.clone()) {
            return Err(
                "ec_spire leaf partition object contains duplicate vec_id assignments".to_owned(),
            );
        }
    }
    Ok(())
}

fn validate_leaf_assignment(assignment: &SpireLeafAssignmentRow) -> Result<(), String> {
    assignment.validate_wire_shape()?;
    if assignment.flags & (SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE)
        != 0
    {
        return Err("ec_spire leaf partition object assignment cannot set delta flags".to_owned());
    }
    let role_flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY
        | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA
        | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE
        | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR;
    if assignment.flags & role_flags == 0 {
        return Err("ec_spire leaf partition object assignment must set a role flag".to_owned());
    }
    if assignment.flags & (SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA)
        != 0
    {
        validate_scored_assignment_payload(assignment)?;
    }
    Ok(())
}

fn validate_delta_assignments(assignments: &[SpireLeafAssignmentRow]) -> Result<(), String> {
    let mut seen_vec_ids = HashSet::new();
    for assignment in assignments {
        validate_delta_assignment(assignment)?;
        if !seen_vec_ids.insert(assignment.vec_id.clone()) {
            return Err(
                "ec_spire delta partition object contains duplicate vec_id assignments".to_owned(),
            );
        }
    }
    Ok(())
}
