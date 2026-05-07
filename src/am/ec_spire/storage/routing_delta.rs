pub(super) trait SpireObjectReader {
    fn prefetch_object(&self, _placement: &SpirePlacementEntry) -> Result<(), String> {
        Ok(())
    }

    fn prefetch_objects(&self, placements: &[SpirePlacementEntry]) -> Result<(), String> {
        for placement in placements {
            self.prefetch_object(placement)?;
        }
        Ok(())
    }

    fn read_object_header(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpirePartitionObjectHeader, String>;

    fn read_routing_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireRoutingPartitionObject, String>;

    fn read_leaf_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObject, String>;

    fn read_leaf_object_v2(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObjectV2, String>;

    fn read_delta_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireDeltaPartitionObject, String>;

    fn read_top_graph_object(
        &self,
        _placement: &SpirePlacementEntry,
    ) -> Result<SpireTopGraphPartitionObject, String> {
        Err("ec_spire object reader does not support top graph objects".to_owned())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRoutingChildEntry {
    pub(super) centroid_index: u32,
    pub(super) child_pid: u64,
    pub(super) centroid: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SpireRoutingChildView<'a> {
    pub(super) centroid_index: u32,
    pub(super) child_pid: u64,
    pub(super) centroid: &'a [f32],
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRoutingPartitionObject {
    pub(super) header: SpirePartitionObjectHeader,
    pub(super) dimensions: u16,
    pub(super) centroid_ordinals: Vec<u32>,
    pub(super) child_pids: Vec<u64>,
    pub(super) centroids: Vec<f32>,
}

impl SpireRoutingPartitionObject {
    pub(super) fn root(
        pid: u64,
        object_version: u64,
        dimensions: u16,
        children: Vec<SpireRoutingChildEntry>,
    ) -> Result<Self, String> {
        Self::root_at_level(pid, object_version, 1, dimensions, children)
    }

    pub(super) fn root_at_level(
        pid: u64,
        object_version: u64,
        level: u16,
        dimensions: u16,
        children: Vec<SpireRoutingChildEntry>,
    ) -> Result<Self, String> {
        Self::new(
            SpirePartitionObjectKind::Root,
            pid,
            object_version,
            level,
            0,
            dimensions,
            children,
        )
    }

    pub(super) fn internal(
        pid: u64,
        object_version: u64,
        level: u16,
        parent_pid: u64,
        dimensions: u16,
        children: Vec<SpireRoutingChildEntry>,
    ) -> Result<Self, String> {
        Self::new(
            SpirePartitionObjectKind::Internal,
            pid,
            object_version,
            level,
            parent_pid,
            dimensions,
            children,
        )
    }

    fn new(
        kind: SpirePartitionObjectKind,
        pid: u64,
        object_version: u64,
        level: u16,
        parent_pid: u64,
        dimensions: u16,
        children: Vec<SpireRoutingChildEntry>,
    ) -> Result<Self, String> {
        let child_count = u32::try_from(children.len())
            .map_err(|_| "ec_spire routing child count exceeds u32".to_owned())?;
        let dimensions_usize = usize::from(dimensions);
        let centroid_capacity = children
            .len()
            .checked_mul(dimensions_usize)
            .ok_or_else(|| "ec_spire routing centroid component count overflow".to_owned())?;
        let mut centroid_ordinals = Vec::with_capacity(children.len());
        let mut child_pids = Vec::with_capacity(children.len());
        let mut centroids = Vec::with_capacity(centroid_capacity);
        for child in children {
            centroid_ordinals.push(child.centroid_index);
            child_pids.push(child.child_pid);
            centroids.extend_from_slice(&child.centroid);
        }
        let object = Self {
            header: SpirePartitionObjectHeader {
                kind,
                pid,
                object_version,
                published_epoch_backref: 0,
                level,
                parent_pid,
                child_count,
                assignment_count: 0,
                flags: 0,
            },
            dimensions,
            centroid_ordinals,
            child_pids,
            centroids,
        };
        object.validate()?;
        Ok(object)
    }

    pub(super) fn child_count(&self) -> usize {
        self.child_pids.len()
    }

    pub(super) fn child_centroid(&self, child_index: usize) -> Option<&[f32]> {
        let dimensions = usize::from(self.dimensions);
        let start = child_index.checked_mul(dimensions)?;
        let end = start.checked_add(dimensions)?;
        self.centroids.get(start..end)
    }

    pub(super) fn children(&self) -> impl Iterator<Item = SpireRoutingChildView<'_>> + '_ {
        let dimensions = usize::from(self.dimensions);
        self.centroid_ordinals
            .iter()
            .copied()
            .zip(self.child_pids.iter().copied())
            .zip(self.centroids.chunks_exact(dimensions))
            .map(
                |((centroid_index, child_pid), centroid)| SpireRoutingChildView {
                    centroid_index,
                    child_pid,
                    centroid,
                },
            )
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;

        let mut out = self
            .header
            .encode_after_validation(PARTITION_OBJECT_FORMAT_VERSION_V1);
        out.extend_from_slice(&self.dimensions.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        for child in self.children() {
            out.extend_from_slice(&child.centroid_index.to_le_bytes());
            out.extend_from_slice(&child.child_pid.to_le_bytes());
            for component in child.centroid {
                out.extend_from_slice(&component.to_le_bytes());
            }
        }
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        let (header, tail) = SpirePartitionObjectHeader::decode_prefix(input)?;
        if tail.len() < ROUTING_OBJECT_BODY_PREFIX_BYTES {
            return Err(format!(
                "ec_spire routing partition object body too short: got {}, expected at least {ROUTING_OBJECT_BODY_PREFIX_BYTES}",
                tail.len()
            ));
        }
        let dimensions = u16::from_le_bytes(tail[0..2].try_into().expect("routing dimensions"));
        let reserved = u16::from_le_bytes(tail[2..4].try_into().expect("routing reserved"));
        if reserved != 0 {
            return Err(format!(
                "ec_spire routing partition object reserved bytes must be zero, got {reserved}"
            ));
        }

        let child_count = usize::try_from(header.child_count)
            .map_err(|_| "ec_spire routing child count exceeds usize".to_owned())?;
        let centroid_bytes = usize::from(dimensions)
            .checked_mul(size_of::<f32>())
            .ok_or_else(|| "ec_spire routing centroid byte length overflow".to_owned())?;
        let child_bytes = ROUTING_CHILD_ENTRY_FIXED_BYTES
            .checked_add(centroid_bytes)
            .ok_or_else(|| "ec_spire routing child byte length overflow".to_owned())?;
        let expected_tail_len = child_count
            .checked_mul(child_bytes)
            .and_then(|children_bytes| children_bytes.checked_add(ROUTING_OBJECT_BODY_PREFIX_BYTES))
            .ok_or_else(|| "ec_spire routing partition object length overflow".to_owned())?;
        if tail.len() != expected_tail_len {
            return Err(format!(
                "ec_spire routing partition object length mismatch: got {}, expected {}",
                tail.len(),
                expected_tail_len
            ));
        }

        let centroid_capacity = child_count
            .checked_mul(usize::from(dimensions))
            .ok_or_else(|| "ec_spire routing centroid component count overflow".to_owned())?;
        let mut centroid_ordinals = Vec::with_capacity(child_count);
        let mut child_pids = Vec::with_capacity(child_count);
        let mut centroids = Vec::with_capacity(centroid_capacity);
        let mut cursor = ROUTING_OBJECT_BODY_PREFIX_BYTES;
        for _ in 0..child_count {
            let centroid_index =
                u32::from_le_bytes(tail[cursor..cursor + 4].try_into().expect("centroid index"));
            cursor += 4;
            let child_pid =
                u64::from_le_bytes(tail[cursor..cursor + 8].try_into().expect("child pid"));
            cursor += 8;
            centroid_ordinals.push(centroid_index);
            child_pids.push(child_pid);
            for _ in 0..dimensions {
                centroids.push(f32::from_le_bytes(
                    tail[cursor..cursor + 4]
                        .try_into()
                        .expect("centroid component"),
                ));
                cursor += 4;
            }
        }

        let object = Self {
            header,
            dimensions,
            centroid_ordinals,
            child_pids,
            centroids,
        };
        object.validate()?;
        Ok(object)
    }

    fn validate(&self) -> Result<(), String> {
        self.header
            .validate_for_format_version(PARTITION_OBJECT_FORMAT_VERSION_V1)?;
        match self.header.kind {
            SpirePartitionObjectKind::Root => {
                if self.header.parent_pid != 0 {
                    return Err("ec_spire root routing object parent_pid must be 0".to_owned());
                }
            }
            SpirePartitionObjectKind::Internal => {
                if self.header.parent_pid == 0 {
                    return Err(
                        "ec_spire internal routing object parent_pid 0 is invalid".to_owned()
                    );
                }
            }
            other => {
                return Err(format!(
                    "ec_spire routing partition object kind must be Root or Internal, got {other:?}"
                ));
            }
        }
        if self.header.level == 0 {
            return Err("ec_spire routing partition object level 0 is invalid".to_owned());
        }
        if self.header.assignment_count != 0 {
            return Err(format!(
                "ec_spire routing partition object assignment_count must be 0, got {}",
                self.header.assignment_count
            ));
        }
        let child_count = u32::try_from(self.child_count())
            .map_err(|_| "ec_spire routing child count exceeds u32".to_owned())?;
        if self.header.child_count != child_count {
            return Err(format!(
                "ec_spire routing child count mismatch: header {}, children {child_count}",
                self.header.child_count
            ));
        }
        if self.dimensions == 0 {
            return Err("ec_spire routing partition object dimensions 0 is invalid".to_owned());
        }
        if self.child_pids.is_empty() {
            return Err("ec_spire routing partition object requires at least one child".to_owned());
        }

        let dimensions = usize::from(self.dimensions);
        if self.centroid_ordinals.len() != self.child_pids.len() {
            return Err(format!(
                "ec_spire routing centroid ordinal count {} does not match child pid count {}",
                self.centroid_ordinals.len(),
                self.child_pids.len()
            ));
        }
        let expected_centroid_components = self
            .child_pids
            .len()
            .checked_mul(dimensions)
            .ok_or_else(|| "ec_spire routing centroid component count overflow".to_owned())?;
        if self.centroids.len() != expected_centroid_components {
            return Err(format!(
                "ec_spire routing centroid component count mismatch: got {}, expected {expected_centroid_components}",
                self.centroids.len()
            ));
        }

        for (expected_index, child) in self.children().enumerate() {
            let expected_index = u32::try_from(expected_index)
                .map_err(|_| "ec_spire routing child centroid index exceeds u32".to_owned())?;
            if child.centroid_index != expected_index {
                return Err(format!(
                    "ec_spire routing child centroid index mismatch: got {}, expected {expected_index}",
                    child.centroid_index
                ));
            }
            if child.child_pid == 0 {
                return Err("ec_spire routing child pid 0 is invalid".to_owned());
            }
            if child
                .centroid
                .iter()
                .any(|component| !component.is_finite())
            {
                return Err(format!(
                    "ec_spire routing child centroid {} must be finite",
                    child.centroid_index
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireDeltaPartitionObject {
    pub(super) header: SpirePartitionObjectHeader,
    pub(super) assignments: Vec<SpireLeafAssignmentRow>,
}

impl SpireDeltaPartitionObject {
    pub(super) fn new(
        pid: u64,
        object_version: u64,
        base_pid: u64,
        assignments: Vec<SpireLeafAssignmentRow>,
    ) -> Result<Self, String> {
        if base_pid == 0 {
            return Err("ec_spire delta partition object base_pid 0 is invalid".to_owned());
        }
        let assignment_count = u32::try_from(assignments.len())
            .map_err(|_| "ec_spire delta assignment count exceeds u32".to_owned())?;
        let object = Self {
            header: SpirePartitionObjectHeader {
                kind: SpirePartitionObjectKind::Delta,
                pid,
                object_version,
                published_epoch_backref: 0,
                level: 0,
                parent_pid: base_pid,
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
            validate_delta_assignment(&assignment)?;
            object.assignments.push(assignment);
            tail = next_tail;
        }
        if !tail.is_empty() {
            return Err(format!(
                "ec_spire delta partition object has {} trailing bytes",
                tail.len()
            ));
        }
        object.validate_header()?;
        Ok(object)
    }

    fn validate_header(&self) -> Result<(), String> {
        let assignment_count = u32::try_from(self.assignments.len())
            .map_err(|_| "ec_spire delta assignment count exceeds u32".to_owned())?;
        if self.header.assignment_count != assignment_count {
            return Err(format!(
                "ec_spire delta assignment count mismatch: header {}, rows {assignment_count}",
                self.header.assignment_count
            ));
        }
        self.validate_header_without_assignment_len()?;
        validate_delta_assignments(&self.assignments)?;
        Ok(())
    }

    fn validate_header_without_assignment_len(&self) -> Result<(), String> {
        self.header
            .validate_for_format_version(PARTITION_OBJECT_FORMAT_VERSION_V1)?;
        if self.header.kind != SpirePartitionObjectKind::Delta {
            return Err(format!(
                "ec_spire delta partition object header kind must be Delta, got {:?}",
                self.header.kind
            ));
        }
        if self.header.parent_pid == 0 {
            return Err("ec_spire delta partition object base_pid 0 is invalid".to_owned());
        }
        if self.header.child_count != 0 {
            return Err(format!(
                "ec_spire delta partition object child_count must be 0, got {}",
                self.header.child_count
            ));
        }
        Ok(())
    }
}
