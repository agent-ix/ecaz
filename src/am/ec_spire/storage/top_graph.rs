#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireTopGraphNodeRecord {
    pub(super) child_pid: u64,
    pub(super) centroid_ordinal: u32,
    pub(super) neighbors: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireTopGraphPartitionObject {
    pub(super) header: SpirePartitionObjectHeader,
    pub(super) root_pid: u64,
    pub(super) dimensions: u16,
    pub(super) graph_degree: u32,
    pub(super) build_list_size: u32,
    pub(super) alpha: f32,
    pub(super) entry_node: u32,
    pub(super) nodes: Vec<SpireTopGraphNodeRecord>,
}

impl SpireTopGraphPartitionObject {
    pub(super) fn new(
        pid: u64,
        object_version: u64,
        root_pid: u64,
        root_level: u16,
        dimensions: u16,
        graph_degree: u32,
        build_list_size: u32,
        alpha: f32,
        entry_node: u32,
        nodes: Vec<SpireTopGraphNodeRecord>,
    ) -> Result<Self, String> {
        let node_count = u32::try_from(nodes.len())
            .map_err(|_| "ec_spire top graph node count exceeds u32".to_owned())?;
        let object = Self {
            header: SpirePartitionObjectHeader {
                kind: SpirePartitionObjectKind::TopGraph,
                pid,
                object_version,
                published_epoch_backref: 0,
                level: root_level,
                parent_pid: root_pid,
                child_count: node_count,
                assignment_count: 0,
                flags: 0,
            },
            root_pid,
            dimensions,
            graph_degree,
            build_list_size,
            alpha,
            entry_node,
            nodes,
        };
        object.validate()?;
        Ok(object)
    }

    pub(super) fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;

        let mut out = self
            .header
            .encode_after_validation(PARTITION_OBJECT_FORMAT_VERSION_V1);
        out.extend_from_slice(&self.root_pid.to_le_bytes());
        out.extend_from_slice(&self.dimensions.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.graph_degree.to_le_bytes());
        out.extend_from_slice(&self.build_list_size.to_le_bytes());
        out.extend_from_slice(&self.alpha.to_le_bytes());
        out.extend_from_slice(&self.entry_node.to_le_bytes());
        for node in &self.nodes {
            out.extend_from_slice(&node.child_pid.to_le_bytes());
            out.extend_from_slice(&node.centroid_ordinal.to_le_bytes());
            let neighbor_count = u32::try_from(node.neighbors.len())
                .map_err(|_| "ec_spire top graph neighbor count exceeds u32".to_owned())?;
            out.extend_from_slice(&neighbor_count.to_le_bytes());
            for neighbor in &node.neighbors {
                out.extend_from_slice(&neighbor.to_le_bytes());
            }
        }
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        let (header, tail) = SpirePartitionObjectHeader::decode_prefix(input)?;
        if tail.len() < TOP_GRAPH_OBJECT_BODY_PREFIX_BYTES {
            return Err(format!(
                "ec_spire top graph object body too short: got {}, expected at least {TOP_GRAPH_OBJECT_BODY_PREFIX_BYTES}",
                tail.len()
            ));
        }

        let root_pid = u64::from_le_bytes(tail[0..8].try_into().expect("top graph root pid"));
        let dimensions = u16::from_le_bytes(tail[8..10].try_into().expect("top graph dimensions"));
        let reserved = u16::from_le_bytes(tail[10..12].try_into().expect("top graph reserved"));
        if reserved != 0 {
            return Err(format!(
                "ec_spire top graph reserved bytes must be zero, got {reserved}"
            ));
        }
        let graph_degree =
            u32::from_le_bytes(tail[12..16].try_into().expect("top graph degree"));
        let build_list_size =
            u32::from_le_bytes(tail[16..20].try_into().expect("top graph build list size"));
        let alpha = f32::from_le_bytes(tail[20..24].try_into().expect("top graph alpha"));
        let entry_node = u32::from_le_bytes(tail[24..28].try_into().expect("top graph entry"));

        let node_count = usize::try_from(header.child_count)
            .map_err(|_| "ec_spire top graph node count exceeds usize".to_owned())?;
        let mut cursor = TOP_GRAPH_OBJECT_BODY_PREFIX_BYTES;
        let mut nodes = Vec::with_capacity(node_count);
        for node_index in 0..node_count {
            let fixed_end = cursor
                .checked_add(TOP_GRAPH_NODE_FIXED_BYTES)
                .ok_or_else(|| "ec_spire top graph node cursor overflow".to_owned())?;
            if fixed_end > tail.len() {
                return Err(format!(
                    "ec_spire top graph node {node_index} body too short"
                ));
            }
            let child_pid =
                u64::from_le_bytes(tail[cursor..cursor + 8].try_into().expect("child pid"));
            cursor += 8;
            let centroid_ordinal = u32::from_le_bytes(
                tail[cursor..cursor + 4]
                    .try_into()
                    .expect("centroid ordinal"),
            );
            cursor += 4;
            let neighbor_count = u32::from_le_bytes(
                tail[cursor..cursor + 4]
                    .try_into()
                    .expect("neighbor count"),
            );
            cursor += 4;
            let neighbor_count = usize::try_from(neighbor_count)
                .map_err(|_| "ec_spire top graph neighbor count exceeds usize".to_owned())?;
            let neighbor_bytes = neighbor_count
                .checked_mul(size_of::<u32>())
                .ok_or_else(|| "ec_spire top graph neighbor bytes overflow".to_owned())?;
            let neighbor_end = cursor
                .checked_add(neighbor_bytes)
                .ok_or_else(|| "ec_spire top graph neighbor cursor overflow".to_owned())?;
            if neighbor_end > tail.len() {
                return Err(format!(
                    "ec_spire top graph node {node_index} neighbors extend past object body"
                ));
            }
            let mut neighbors = Vec::with_capacity(neighbor_count);
            while cursor < neighbor_end {
                neighbors.push(u32::from_le_bytes(
                    tail[cursor..cursor + 4]
                        .try_into()
                        .expect("neighbor ordinal"),
                ));
                cursor += 4;
            }
            nodes.push(SpireTopGraphNodeRecord {
                child_pid,
                centroid_ordinal,
                neighbors,
            });
        }
        if cursor != tail.len() {
            return Err(format!(
                "ec_spire top graph object has {} trailing bytes",
                tail.len() - cursor
            ));
        }

        let object = Self {
            header,
            root_pid,
            dimensions,
            graph_degree,
            build_list_size,
            alpha,
            entry_node,
            nodes,
        };
        object.validate()?;
        Ok(object)
    }

    fn validate(&self) -> Result<(), String> {
        self.header
            .validate_for_format_version(PARTITION_OBJECT_FORMAT_VERSION_V1)?;
        if self.header.kind != SpirePartitionObjectKind::TopGraph {
            return Err(format!(
                "ec_spire top graph header kind must be TopGraph, got {:?}",
                self.header.kind
            ));
        }
        if self.root_pid == 0 {
            return Err("ec_spire top graph root pid 0 is invalid".to_owned());
        }
        if self.header.parent_pid != self.root_pid {
            return Err(format!(
                "ec_spire top graph parent_pid {} does not match root pid {}",
                self.header.parent_pid, self.root_pid
            ));
        }
        if self.header.level == 0 {
            return Err("ec_spire top graph root level 0 is invalid".to_owned());
        }
        if self.header.assignment_count != 0 {
            return Err(format!(
                "ec_spire top graph assignment_count must be 0, got {}",
                self.header.assignment_count
            ));
        }
        if self.header.flags != 0 {
            return Err(format!(
                "ec_spire top graph flags must be 0, got {:#x}",
                self.header.flags
            ));
        }
        let node_count = u32::try_from(self.node_count())
            .map_err(|_| "ec_spire top graph node count exceeds u32".to_owned())?;
        if self.header.child_count != node_count {
            return Err(format!(
                "ec_spire top graph node count mismatch: header {}, nodes {node_count}",
                self.header.child_count
            ));
        }
        if self.dimensions == 0 {
            return Err("ec_spire top graph dimensions 0 is invalid".to_owned());
        }
        if self.graph_degree == 0 {
            return Err("ec_spire top graph degree must be greater than 0".to_owned());
        }
        if self.build_list_size == 0 {
            return Err("ec_spire top graph build list size must be greater than 0".to_owned());
        }
        if !self.alpha.is_finite() || self.alpha < 1.0 {
            return Err("ec_spire top graph alpha must be finite and at least 1.0".to_owned());
        }
        if self.nodes.is_empty() {
            return Err("ec_spire top graph requires at least one node".to_owned());
        }
        if self.entry_node >= node_count {
            return Err(format!(
                "ec_spire top graph entry node {} is outside node count {node_count}",
                self.entry_node
            ));
        }

        let mut child_pids = HashSet::with_capacity(self.nodes.len());
        let mut centroid_ordinals = HashSet::with_capacity(self.nodes.len());
        let max_degree = usize::try_from(self.graph_degree)
            .map_err(|_| "ec_spire top graph degree exceeds usize".to_owned())?;
        let node_count_usize = self.nodes.len();
        for (node_index, node) in self.nodes.iter().enumerate() {
            if node.child_pid == 0 {
                return Err(format!(
                    "ec_spire top graph node {node_index} child pid 0 is invalid"
                ));
            }
            if !child_pids.insert(node.child_pid) {
                return Err(format!(
                    "ec_spire top graph duplicate child pid {}",
                    node.child_pid
                ));
            }
            if !centroid_ordinals.insert(node.centroid_ordinal) {
                return Err(format!(
                    "ec_spire top graph duplicate centroid ordinal {}",
                    node.centroid_ordinal
                ));
            }
            if node.neighbors.len() > max_degree {
                return Err(format!(
                    "ec_spire top graph node {node_index} neighbor count {} exceeds graph degree {max_degree}",
                    node.neighbors.len()
                ));
            }
            let mut seen_neighbors = HashSet::with_capacity(node.neighbors.len());
            for &neighbor in &node.neighbors {
                let neighbor_index = usize::try_from(neighbor)
                    .map_err(|_| "ec_spire top graph neighbor exceeds usize".to_owned())?;
                if neighbor_index >= node_count_usize {
                    return Err(format!(
                        "ec_spire top graph node {node_index} neighbor {neighbor} is outside node count {node_count_usize}"
                    ));
                }
                if neighbor_index == node_index {
                    return Err(format!(
                        "ec_spire top graph node {node_index} cannot neighbor itself"
                    ));
                }
                if !seen_neighbors.insert(neighbor) {
                    return Err(format!(
                        "ec_spire top graph node {node_index} duplicate neighbor {neighbor}"
                    ));
                }
            }
        }
        Ok(())
    }
}
