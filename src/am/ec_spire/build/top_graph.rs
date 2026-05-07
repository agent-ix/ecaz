#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireTopGraphNodeInput {
    pub(super) child_pid: u64,
    pub(super) centroid_ordinal: u32,
    pub(super) centroid: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireTopGraphBuildInput {
    pub(super) root_pid: u64,
    pub(super) dimensions: u16,
    pub(super) graph_degree: u32,
    pub(super) build_list_size: u32,
    pub(super) alpha: f32,
    pub(super) seed: u64,
    pub(super) nodes: Vec<SpireTopGraphNodeInput>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SpireTopGraphBuildParams {
    pub(super) graph_degree: u32,
    pub(super) build_list_size: u32,
    pub(super) alpha: f32,
    pub(super) seed: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireTopGraphNode {
    pub(super) child_pid: u64,
    pub(super) centroid_ordinal: u32,
    pub(super) neighbors: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireTopGraphBuildDraft {
    pub(super) root_pid: u64,
    pub(super) dimensions: u16,
    pub(super) node_count: u32,
    pub(super) graph_degree: u32,
    pub(super) build_list_size: u32,
    pub(super) alpha: f32,
    pub(super) entry_node: u32,
    pub(super) nodes: Vec<SpireTopGraphNode>,
    pub(super) stats: crate::am::VamanaBuildStats,
}

pub(super) fn build_spire_top_graph_draft_from_routing_object(
    routing_object: &SpireRoutingPartitionObject,
    params: SpireTopGraphBuildParams,
) -> Result<SpireTopGraphBuildDraft, String> {
    if routing_object.header.kind != SpirePartitionObjectKind::Root {
        return Err(format!(
            "ec_spire top graph requires root routing object, got {:?}",
            routing_object.header.kind
        ));
    }
    let nodes = routing_object
        .children()
        .map(|child| SpireTopGraphNodeInput {
            child_pid: child.child_pid,
            centroid_ordinal: child.centroid_index,
            centroid: child.centroid.to_vec(),
        })
        .collect::<Vec<_>>();
    build_spire_top_graph_draft(SpireTopGraphBuildInput {
        root_pid: routing_object.header.pid,
        dimensions: routing_object.dimensions,
        graph_degree: params.graph_degree,
        build_list_size: params.build_list_size,
        alpha: params.alpha,
        seed: params.seed,
        nodes,
    })
}

pub(super) fn build_spire_top_graph_draft(
    input: SpireTopGraphBuildInput,
) -> Result<SpireTopGraphBuildDraft, String> {
    input.validate()?;
    let node_count = input.nodes.len();
    let graph_degree = usize::try_from(input.graph_degree)
        .map_err(|_| "ec_spire top graph degree exceeds usize".to_owned())?;
    let build_list_size = usize::try_from(input.build_list_size)
        .map_err(|_| "ec_spire top graph build list size exceeds usize".to_owned())?;
    let distance_offset = max_centroid_inner_product(&input.nodes);
    let dist = |a: u32, b: u32| -> f32 {
        let a = &input.nodes[a as usize].centroid;
        let b = &input.nodes[b as usize].centroid;
        (distance_offset - top_graph_inner_product(a, b)).max(0.0)
    };
    let entry_node = crate::am::approximate_medoid(
        node_count,
        build_list_size,
        input.seed.wrapping_add(0x7370_6972_655f_7467),
        dist,
    );
    let (graph, stats) = crate::am::build_vamana_graph_with_stats(
        node_count,
        entry_node,
        graph_degree,
        build_list_size,
        input.alpha,
        input.seed,
        dist,
    );
    let nodes = input
        .nodes
        .iter()
        .zip(graph.neighbors)
        .map(|(node, neighbors)| SpireTopGraphNode {
            child_pid: node.child_pid,
            centroid_ordinal: node.centroid_ordinal,
            neighbors,
        })
        .collect::<Vec<_>>();
    Ok(SpireTopGraphBuildDraft {
        root_pid: input.root_pid,
        dimensions: input.dimensions,
        node_count: u32::try_from(node_count)
            .map_err(|_| "ec_spire top graph node count exceeds u32".to_owned())?,
        graph_degree: input.graph_degree,
        build_list_size: input.build_list_size,
        alpha: input.alpha,
        entry_node,
        nodes,
        stats,
    })
}

impl SpireTopGraphBuildInput {
    fn validate(&self) -> Result<(), String> {
        if self.root_pid == 0 {
            return Err("ec_spire top graph root pid 0 is invalid".to_owned());
        }
        if self.dimensions == 0 {
            return Err("ec_spire top graph requires dimensions > 0".to_owned());
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
        if self.nodes.len() > u32::MAX as usize {
            return Err("ec_spire top graph node count exceeds u32".to_owned());
        }

        let mut child_pids = HashSet::with_capacity(self.nodes.len());
        let mut centroid_ordinals = HashSet::with_capacity(self.nodes.len());
        let expected_dimensions = usize::from(self.dimensions);
        for node in &self.nodes {
            if node.child_pid == 0 {
                return Err("ec_spire top graph child pid 0 is invalid".to_owned());
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
            if node.centroid.len() != expected_dimensions {
                return Err(format!(
                    "ec_spire top graph child pid {} centroid dimensions mismatch: got {}, expected {expected_dimensions}",
                    node.child_pid,
                    node.centroid.len()
                ));
            }
            if node.centroid.iter().any(|component| !component.is_finite()) {
                return Err(format!(
                    "ec_spire top graph child pid {} centroid must be finite",
                    node.child_pid
                ));
            }
        }
        Ok(())
    }
}

fn max_centroid_inner_product(nodes: &[SpireTopGraphNodeInput]) -> f32 {
    let mut max_ip = f32::NEG_INFINITY;
    for lhs in nodes {
        for rhs in nodes {
            max_ip = max_ip.max(top_graph_inner_product(&lhs.centroid, &rhs.centroid));
        }
    }
    max_ip
}

fn top_graph_inner_product(lhs: &[f32], rhs: &[f32]) -> f32 {
    lhs.iter().zip(rhs).map(|(a, b)| a * b).sum::<f32>()
}
