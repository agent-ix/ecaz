#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafPartitionObject {
    pub(super) header: SpirePartitionObjectHeader,
    pub(super) assignments: Vec<SpireLeafAssignmentRow>,
}

impl SpireLeafPartitionObject {
    pub(super) fn new(
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        assignments: Vec<SpireLeafAssignmentRow>,
    ) -> Result<Self, String> {
        let assignment_count = u32::try_from(assignments.len())
            .map_err(|_| "ec_spire leaf assignment count exceeds u32".to_owned())?;
        let object = Self {
            header: SpirePartitionObjectHeader {
                kind: SpirePartitionObjectKind::Leaf,
                pid,
                object_version,
                published_epoch_backref: 0,
                level: 0,
                parent_pid,
                child_count: 0,
                assignment_count,
                flags: 0,
            },
            assignments,
        };
        object.validate_header()?;
        Ok(object)
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate_header()?;

        let mut out = self
            .header
            .encode_after_validation(PARTITION_OBJECT_FORMAT_VERSION_V1);
        for assignment in &self.assignments {
            out.extend_from_slice(&assignment.encode_after_validation());
        }
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        let (header, mut tail) = SpirePartitionObjectHeader::decode_prefix(input)?;
        let mut object = Self {
            header,
            assignments: Vec::with_capacity(header.assignment_count as usize),
        };
        object.validate_header_without_assignment_len()?;

        for _ in 0..header.assignment_count {
            let (assignment, next_tail) = SpireLeafAssignmentRow::decode_prefix(tail)?;
            object.assignments.push(assignment);
            tail = next_tail;
        }
        if !tail.is_empty() {
            return Err(format!(
                "ec_spire leaf partition object has {} trailing bytes",
                tail.len()
            ));
        }
        object.validate_header()?;
        Ok(object)
    }

    fn validate_header(&self) -> Result<(), String> {
        let assignment_count = u32::try_from(self.assignments.len())
            .map_err(|_| "ec_spire leaf assignment count exceeds u32".to_owned())?;
        if self.header.assignment_count != assignment_count {
            return Err(format!(
                "ec_spire leaf assignment count mismatch: header {}, rows {assignment_count}",
                self.header.assignment_count
            ));
        }
        self.validate_header_without_assignment_len()?;
        validate_leaf_assignments(&self.assignments)?;
        Ok(())
    }

    fn validate_header_without_assignment_len(&self) -> Result<(), String> {
        self.header
            .validate_for_format_version(PARTITION_OBJECT_FORMAT_VERSION_V1)?;
        if self.header.kind != SpirePartitionObjectKind::Leaf {
            return Err(format!(
                "ec_spire leaf partition object header kind must be Leaf, got {:?}",
                self.header.kind
            ));
        }
        if self.header.child_count != 0 {
            return Err(format!(
                "ec_spire leaf partition object child_count must be 0, got {}",
                self.header.child_count
            ));
        }
        Ok(())
    }
}
