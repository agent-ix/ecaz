//! FR-080 canonical persisted coordinator head sample.

use std::collections::{BTreeMap, VecDeque};
use std::collections::{HashMap, HashSet};
#[cfg(feature = "distann-head-attribution-benchmark")]
use std::time::Instant;

use pgrx::datum::Uuid;
use pgrx::{pg_sys, Spi};
use sha2::{Digest, Sha256};

use super::generation_catalog::extension_relation_name;
use super::manifest_v2::DistannEpochManifestV2;

const HEAD_SAMPLE_DOMAIN: &[u8] = b"ec_distann_head_sample_v1\0";
const HEAD_GRAPH_DOMAIN: &[u8] = b"ec_distann_head_graph_v1\0";
const TRAINING_QUERY_DOMAIN: &[u8] = b"ec_distann_head_training_queries_v1\0";
const HEAD_SAMPLE_VERSION: u16 = 1;
const HEAD_GRAPH_SEED_WRAP: u64 = 0x6469_7374_5f74_6721;
pub(crate) const TRAINED_HEAD_SEED_COUNT: usize = 32;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DistannHeadSampleEntry {
    pub(crate) vec_id: u64,
    pub(crate) vector: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DistannHeadSample {
    pub(crate) dimensions: u16,
    pub(crate) entries: Vec<DistannHeadSampleEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DistannTrainingQuerySet {
    pub(crate) queries: Vec<Vec<f32>>,
    pub(crate) digest: [u8; 32],
}

impl DistannTrainingQuerySet {
    pub(crate) fn from_ordered_rows(
        rows: Vec<(i64, Vec<f32>)>,
        dimensions: usize,
    ) -> Result<Self, String> {
        if rows.len() != super::generation_descriptor::DISTANN_TRAINING_QUERY_COUNT as usize {
            return Err(format!(
                "EC_HEAD_TRAINING: expected {} training queries, found {}",
                super::generation_descriptor::DISTANN_TRAINING_QUERY_COUNT,
                rows.len()
            ));
        }
        let mut previous = None;
        let mut hasher = Sha256::new();
        hasher.update(TRAINING_QUERY_DOMAIN);
        hasher.update((rows.len() as u32).to_le_bytes());
        hasher.update((dimensions as u32).to_le_bytes());
        let mut queries = Vec::with_capacity(rows.len());
        for (ordinal, vector) in rows {
            if previous.is_some_and(|value| ordinal <= value) {
                return Err(
                    "EC_HEAD_TRAINING: training ordinals must be strictly ascending".to_owned(),
                );
            }
            if vector.len() != dimensions || vector.iter().any(|value| !value.is_finite()) {
                return Err(format!(
                    "EC_HEAD_TRAINING: training ordinal {ordinal} has invalid shape/value"
                ));
            }
            previous = Some(ordinal);
            hasher.update(ordinal.to_le_bytes());
            for value in &vector {
                hasher.update(value.to_le_bytes());
            }
            queries.push(vector);
        }
        let digest = hasher.finalize().into();
        if digest == [0; 32] {
            return Err("EC_HEAD_TRAINING: canonical digest is zero".to_owned());
        }
        Ok(Self { queries, digest })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DistannPersistedHeadGraph {
    pub(crate) entry: u32,
    pub(crate) neighbors: Vec<Vec<u32>>,
}

impl DistannPersistedHeadGraph {
    pub(crate) fn build(
        sample: &DistannHeadSample,
        graph_degree: usize,
        build_list_size: usize,
        alpha: f32,
        seed: u64,
    ) -> Result<Self, String> {
        if sample.entries.is_empty() {
            return Ok(Self {
                entry: 0,
                neighbors: Vec::new(),
            });
        }
        let vectors = sample
            .entries
            .iter()
            .map(|entry| &entry.vector)
            .collect::<Vec<_>>();
        let distance = |left: u32, right: u32| {
            crate::am::ec_diskann::source_inner_product_distance(
                vectors[left as usize],
                vectors[right as usize],
            )
        };
        let seed = seed ^ HEAD_GRAPH_SEED_WRAP;
        let entry = crate::am::approximate_medoid(vectors.len(), vectors.len(), seed, distance);
        let (mut graph, _) = crate::am::build_vamana_graph_with_stats(
            vectors.len(),
            entry,
            graph_degree,
            build_list_size,
            alpha,
            seed,
            distance,
        );
        super::shard_build::repair_reachability(&mut graph, entry, graph_degree, &distance)?;
        let persisted = Self {
            entry,
            neighbors: graph.neighbors,
        };
        persisted.validate(sample.entries.len(), graph_degree)?;
        Ok(persisted)
    }

    fn validate(&self, sample_count: usize, graph_degree: usize) -> Result<(), String> {
        if self.neighbors.len() != sample_count
            || (sample_count == 0 && self.entry != 0)
            || (sample_count > 0 && self.entry as usize >= sample_count)
            || self.neighbors.iter().any(|neighbors| {
                neighbors.len() > graph_degree
                    || neighbors
                        .iter()
                        .any(|neighbor| *neighbor as usize >= sample_count)
            })
        {
            return Err("EC_HEAD_SAMPLE: persisted head graph is invalid".to_owned());
        }
        Ok(())
    }

    fn digest(&self) -> Result<[u8; 32], String> {
        let mut encoder = super::canonical_wire::CanonicalEncoder::default();
        encoder.put_u32(self.entry);
        encoder.put_u32(
            u32::try_from(self.neighbors.len())
                .map_err(|_| "EC_HEAD_SAMPLE: head graph count exceeds u32".to_owned())?,
        );
        for neighbors in &self.neighbors {
            encoder.put_u32(
                u32::try_from(neighbors.len())
                    .map_err(|_| "EC_HEAD_SAMPLE: head degree exceeds u32".to_owned())?,
            );
            for neighbor in neighbors {
                encoder.put_u32(*neighbor);
            }
        }
        Ok(super::canonical_wire::domain_digest(
            HEAD_GRAPH_DOMAIN,
            &encoder.finish()?,
        ))
    }
}

impl DistannHeadSample {
    pub(crate) fn validate(&self, cap: usize) -> Result<(), String> {
        let dimensions = usize::from(self.dimensions);
        if dimensions == 0 || self.entries.len() > cap {
            return Err("EC_HEAD_SAMPLE: invalid dimension or sample count".to_owned());
        }
        let mut ids = std::collections::HashSet::with_capacity(self.entries.len());
        for entry in &self.entries {
            if !ids.insert(entry.vec_id)
                || entry.vector.len() != dimensions
                || entry.vector.iter().any(|value| !value.is_finite())
            {
                return Err("EC_HEAD_SAMPLE: invalid or duplicate sample entry".to_owned());
            }
        }
        Ok(())
    }

    pub(crate) fn digest(&self) -> Result<[u8; 32], String> {
        self.validate(usize::MAX)?;
        let mut hasher = Sha256::new();
        hasher.update(HEAD_SAMPLE_DOMAIN);
        hasher.update(HEAD_SAMPLE_VERSION.to_le_bytes());
        hasher.update(self.dimensions.to_le_bytes());
        hasher.update(
            u32::try_from(self.entries.len())
                .map_err(|_| "EC_HEAD_SAMPLE: sample count exceeds u32".to_owned())?
                .to_le_bytes(),
        );
        for entry in &self.entries {
            hasher.update(entry.vec_id.to_le_bytes());
            for value in &entry.vector {
                hasher.update(value.to_le_bytes());
            }
        }
        Ok(hasher.finalize().into())
    }
}

pub(crate) fn build_head_sample(
    graph: &crate::am::VamanaGraph,
    medoid: u32,
    cap: usize,
    dimensions: u16,
    vec_ids: &[u64],
    vectors: &[Vec<f32>],
) -> Result<DistannHeadSample, String> {
    if vec_ids.len() != vectors.len() || graph.neighbors.len() != vec_ids.len() {
        return Err("EC_HEAD_SAMPLE: graph/source cardinality mismatch".to_owned());
    }
    if vec_ids.is_empty() {
        return Ok(DistannHeadSample {
            dimensions,
            entries: Vec::new(),
        });
    }
    let medoid = usize::try_from(medoid)
        .ok()
        .filter(|node| *node < vec_ids.len())
        .ok_or_else(|| "EC_HEAD_SAMPLE: medoid is outside the graph".to_owned())?;
    let sample_cap = cap.min(vec_ids.len());
    let mut visited = vec![false; vec_ids.len()];
    let mut seed_order = (0..vec_ids.len()).collect::<Vec<_>>();
    seed_order.sort_unstable_by_key(|node| vec_ids[*node]);
    seed_order.retain(|node| *node != medoid);
    seed_order.insert(0, medoid);
    let mut regions = Vec::new();
    for seed in seed_order {
        if visited[seed] {
            continue;
        }
        visited[seed] = true;
        let mut queue = VecDeque::from([seed]);
        let mut region = Vec::new();
        while let Some(node) = queue.pop_front() {
            region.push(node);
            for neighbor in &graph.neighbors[node] {
                let neighbor = *neighbor as usize;
                if neighbor >= visited.len() {
                    return Err("EC_HEAD_SAMPLE: graph neighbor is outside the graph".to_owned());
                }
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
        regions.push(region);
    }
    if regions.len() > sample_cap {
        return Err(format!(
            "EC_HEAD_SAMPLE: head_index_cap {sample_cap} cannot represent {} directed entry regions",
            regions.len()
        ));
    }

    // One entry per region first, then deterministic round-robin BFS tails.
    // This preserves every region under a tight cap instead of allowing the
    // global medoid's region to consume the whole sample budget.
    let mut selected = regions.iter().map(|region| region[0]).collect::<Vec<_>>();
    let mut depth = 1;
    while selected.len() < sample_cap {
        let before = selected.len();
        for region in &regions {
            if let Some(node) = region.get(depth) {
                selected.push(*node);
                if selected.len() == sample_cap {
                    break;
                }
            }
        }
        if selected.len() == before {
            break;
        }
        depth += 1;
    }
    let entries = selected
        .into_iter()
        .map(|node| DistannHeadSampleEntry {
            vec_id: vec_ids[node],
            vector: vectors[node].clone(),
        })
        .collect();
    let sample = DistannHeadSample {
        dimensions,
        entries,
    };
    sample.validate(cap)?;
    Ok(sample)
}

/// Deterministic, query-independent geometric region used by Task 181 both to
/// select landmarks and to report evaluation-query coverage.  Twelve fixed
/// sparse random hyperplanes give 4096 bounded regions without fitting state
/// to either the training or evaluation queries.
pub(crate) fn benchmark_geometry_region(vector: &[f32]) -> u16 {
    fn mix(mut value: u64) -> u64 {
        value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
    if vector.is_empty() {
        return 0;
    }
    let mut signature = 0_u16;
    for plane in 0..12_u64 {
        let mut projection = 0.0_f32;
        for sample in 0..32_u64 {
            let random = mix(0x7461_736b_3138_3100 ^ (plane << 32) ^ sample);
            let coordinate = (random as usize) % vector.len();
            let sign = if random & (1 << 63) == 0 { -1.0 } else { 1.0 };
            projection += sign * vector[coordinate];
        }
        if projection >= 0.0 {
            signature |= 1 << plane;
        }
    }
    signature
}

fn geometry_landmark_nodes(vec_ids: &[u64], vectors: &[Vec<f32>], cap: usize) -> Vec<usize> {
    let mut regions = BTreeMap::<u16, Vec<usize>>::new();
    for (node, vector) in vectors.iter().enumerate() {
        regions
            .entry(benchmark_geometry_region(vector))
            .or_default()
            .push(node);
    }
    for nodes in regions.values_mut() {
        nodes.sort_unstable_by_key(|node| vec_ids[*node]);
    }
    let mut selected = Vec::with_capacity(cap.min(vec_ids.len()));
    let mut depth = 0;
    while selected.len() < cap.min(vec_ids.len()) {
        let before = selected.len();
        for nodes in regions.values() {
            if let Some(node) = nodes.get(depth) {
                selected.push(*node);
                if selected.len() == cap.min(vec_ids.len()) {
                    break;
                }
            }
        }
        if selected.len() == before {
            break;
        }
        depth += 1;
    }
    selected
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn graph_landmark_nodes(
    graph: &crate::am::VamanaGraph,
    medoid: u32,
    vec_ids: &[u64],
    cap: usize,
) -> Result<Vec<usize>, String> {
    let medoid = medoid as usize;
    if medoid >= vec_ids.len() {
        return Err("EC_HEAD_SAMPLE: medoid is outside the graph".to_owned());
    }
    let mut seed_order = (0..vec_ids.len()).collect::<Vec<_>>();
    seed_order.sort_unstable_by_key(|node| vec_ids[*node]);
    seed_order.retain(|node| *node != medoid);
    seed_order.insert(0, medoid);
    let mut visited = vec![false; vec_ids.len()];
    let mut order = Vec::with_capacity(vec_ids.len());
    for seed in seed_order {
        if visited[seed] {
            continue;
        }
        visited[seed] = true;
        let mut queue = VecDeque::from([seed]);
        while let Some(node) = queue.pop_front() {
            order.push(node);
            for neighbor in &graph.neighbors[node] {
                let neighbor = *neighbor as usize;
                if neighbor >= visited.len() {
                    return Err("EC_HEAD_SAMPLE: graph neighbor is outside the graph".to_owned());
                }
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
    }
    let count = cap.min(order.len());
    Ok((0..count)
        .map(|slot| order[slot * order.len() / count])
        .collect())
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn load_training_queries(path: &str, dimensions: usize) -> Result<Vec<Vec<f32>>, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|error| format!("EC_HEAD_SAMPLE: cannot read training queries: {error}"))?;
    let mut queries = Vec::with_capacity(200);
    // Evaluation is the immutable first 200 rows used by Tasks 180/181.  The
    // next 200 rows are the declared disjoint training slice.
    for (line_index, line) in contents.lines().enumerate().skip(200).take(200) {
        let (_, encoded) = line.split_once('\t').ok_or_else(|| {
            format!(
                "EC_HEAD_SAMPLE: malformed training TSV row {}",
                line_index + 1
            )
        })?;
        let vector: Vec<f32> = serde_json::from_str(encoded).map_err(|error| {
            format!(
                "EC_HEAD_SAMPLE: invalid training vector at row {}: {error}",
                line_index + 1
            )
        })?;
        if vector.len() != dimensions || vector.iter().any(|value| !value.is_finite()) {
            return Err(format!(
                "EC_HEAD_SAMPLE: training vector at row {} has invalid shape/value",
                line_index + 1
            ));
        }
        queries.push(vector);
    }
    if queries.len() != 200 {
        return Err(format!(
            "EC_HEAD_SAMPLE: training slice requires 200 rows, found {}",
            queries.len()
        ));
    }
    Ok(queries)
}

#[derive(Debug, Clone)]
struct TrainingCandidateRanking {
    per_query: Vec<Vec<usize>>,
    frequency: HashMap<usize, (u32, usize)>,
}

fn rank_training_candidates(
    queries: &[Vec<f32>],
    vec_ids: &[u64],
    codes: &[Vec<u8>],
    codec_artifact: &super::generation_descriptor::DistannCodecArtifact,
) -> Result<TrainingCandidateRanking, String> {
    let mut frequency = HashMap::<usize, (u32, usize)>::new();
    let mut per_query = Vec::with_capacity(queries.len());
    for query in queries {
        let prepared =
            super::quantizer::DistannPreparedQuery::prepare_artifact(codec_artifact, query)?;
        let mut ranked = codes
            .iter()
            .enumerate()
            .map(|(node, code)| (prepared.score_dist(code), vec_ids[node], node))
            .collect::<Vec<_>>();
        let keep = 32.min(ranked.len());
        if keep < ranked.len() {
            ranked.select_nth_unstable_by(keep, |left, right| {
                left.0
                    .total_cmp(&right.0)
                    .then_with(|| left.1.cmp(&right.1))
            });
            ranked.truncate(keep);
        }
        ranked.sort_unstable_by(|left, right| {
            left.0
                .total_cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
        });
        let nodes = ranked
            .into_iter()
            .map(|(_, _, node)| node)
            .collect::<Vec<_>>();
        for (rank, node) in nodes.iter().copied().enumerate() {
            let entry = frequency.entry(node).or_insert((0, rank));
            entry.0 += 1;
            entry.1 = entry.1.min(rank);
        }
        per_query.push(nodes);
    }
    Ok(TrainingCandidateRanking {
        per_query,
        frequency,
    })
}

fn fill_with_geometry(
    selected: &mut Vec<usize>,
    vec_ids: &[u64],
    vectors: &[Vec<f32>],
    sample_cap: usize,
) {
    let mut seen = selected.iter().copied().collect::<HashSet<_>>();
    for node in geometry_landmark_nodes(vec_ids, vectors, sample_cap) {
        if selected.len() == sample_cap {
            break;
        }
        if seen.insert(node) {
            selected.push(node);
        }
    }
}

fn frequency_landmark_nodes(
    ranking: TrainingCandidateRanking,
    vec_ids: &[u64],
    vectors: &[Vec<f32>],
    sample_cap: usize,
) -> Vec<usize> {
    let mut ranked = ranking.frequency.into_iter().collect::<Vec<_>>();
    ranked.sort_unstable_by(|(left_node, left), (right_node, right)| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| vec_ids[*left_node].cmp(&vec_ids[*right_node]))
    });
    let mut selected = ranked
        .into_iter()
        .map(|(node, _)| node)
        .take(sample_cap)
        .collect::<Vec<_>>();
    fill_with_geometry(&mut selected, vec_ids, vectors, sample_cap);
    selected
}

fn training_landmark_nodes(
    queries: &[Vec<f32>],
    vec_ids: &[u64],
    vectors: &[Vec<f32>],
    codes: &[Vec<u8>],
    codec_artifact: &super::generation_descriptor::DistannCodecArtifact,
    sample_cap: usize,
) -> Result<Vec<usize>, String> {
    let ranking = rank_training_candidates(queries, vec_ids, codes, codec_artifact)?;
    Ok(frequency_landmark_nodes(
        ranking, vec_ids, vectors, sample_cap,
    ))
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn training_region_balanced_nodes(
    ranking: TrainingCandidateRanking,
    vec_ids: &[u64],
    vectors: &[Vec<f32>],
    sample_cap: usize,
) -> Vec<usize> {
    let mut regions = BTreeMap::<u16, Vec<usize>>::new();
    for node in ranking.frequency.keys().copied() {
        regions
            .entry(benchmark_geometry_region(&vectors[node]))
            .or_default()
            .push(node);
    }
    for nodes in regions.values_mut() {
        nodes.sort_unstable_by(|left, right| {
            let left_stats = ranking.frequency[left];
            let right_stats = ranking.frequency[right];
            right_stats
                .0
                .cmp(&left_stats.0)
                .then_with(|| left_stats.1.cmp(&right_stats.1))
                .then_with(|| vec_ids[*left].cmp(&vec_ids[*right]))
        });
    }
    let mut selected = Vec::with_capacity(sample_cap);
    let mut depth = 0;
    while selected.len() < sample_cap {
        let before = selected.len();
        for nodes in regions.values() {
            if let Some(node) = nodes.get(depth) {
                selected.push(*node);
                if selected.len() == sample_cap {
                    break;
                }
            }
        }
        if selected.len() == before {
            break;
        }
        depth += 1;
    }
    fill_with_geometry(&mut selected, vec_ids, vectors, sample_cap);
    selected
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn training_query_facility_nodes(
    ranking: TrainingCandidateRanking,
    vec_ids: &[u64],
    vectors: &[Vec<f32>],
    sample_cap: usize,
) -> Vec<usize> {
    let mut selected = Vec::with_capacity(sample_cap);
    let mut seen = HashSet::with_capacity(sample_cap);
    let query_count = ranking.per_query.len();
    let max_rank = ranking.per_query.iter().map(Vec::len).max().unwrap_or(0);
    for rank in 0..max_rank {
        let start = (97 * rank) % query_count.max(1);
        for offset in 0..query_count {
            let query = (start + offset) % query_count;
            if let Some(node) = ranking.per_query[query].get(rank) {
                if seen.insert(*node) {
                    selected.push(*node);
                    if selected.len() == sample_cap {
                        return selected;
                    }
                }
            }
        }
    }
    fill_with_geometry(&mut selected, vec_ids, vectors, sample_cap);
    selected
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_training_head_sample(
    training: &DistannTrainingQuerySet,
    cap: usize,
    dimensions: u16,
    vec_ids: &[u64],
    vectors: &[Vec<f32>],
    codes: &[Vec<u8>],
    codec_artifact: &super::generation_descriptor::DistannCodecArtifact,
) -> Result<DistannHeadSample, String> {
    if vec_ids.len() != vectors.len() || vec_ids.len() != codes.len() {
        return Err("EC_HEAD_SAMPLE: policy input cardinality mismatch".to_owned());
    }
    let sample_cap = cap.min(vec_ids.len());
    let selected = training_landmark_nodes(
        &training.queries,
        vec_ids,
        vectors,
        codes,
        codec_artifact,
        sample_cap,
    )?;
    let sample = DistannHeadSample {
        dimensions,
        entries: selected
            .into_iter()
            .map(|node| DistannHeadSampleEntry {
                vec_id: vec_ids[node],
                vector: vectors[node].clone(),
            })
            .collect(),
    };
    sample.validate(cap)?;
    Ok(sample)
}

#[cfg(feature = "distann-head-attribution-benchmark")]
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_benchmark_head_sample(
    policy: super::options::BenchmarkHeadPolicy,
    graph: &crate::am::VamanaGraph,
    medoid: u32,
    cap: usize,
    dimensions: u16,
    vec_ids: &[u64],
    vectors: &[Vec<f32>],
    codes: &[Vec<u8>],
    codec_artifact: &super::generation_descriptor::DistannCodecArtifact,
    training_query_path: Option<&str>,
) -> Result<DistannHeadSample, String> {
    if policy == super::options::BenchmarkHeadPolicy::CurrentSample {
        return build_head_sample(graph, medoid, cap, dimensions, vec_ids, vectors);
    }
    if vec_ids.len() != vectors.len() || vec_ids.len() != codes.len() {
        return Err("EC_HEAD_SAMPLE: policy input cardinality mismatch".to_owned());
    }
    let sample_cap = cap.min(vec_ids.len());
    let selected = match policy {
        super::options::BenchmarkHeadPolicy::CurrentSample => unreachable!(),
        super::options::BenchmarkHeadPolicy::GeometryLandmarks => {
            geometry_landmark_nodes(vec_ids, vectors, sample_cap)
        }
        super::options::BenchmarkHeadPolicy::GraphLandmarks => {
            graph_landmark_nodes(graph, medoid, vec_ids, sample_cap)?
        }
        super::options::BenchmarkHeadPolicy::TrainingLandmarks
        | super::options::BenchmarkHeadPolicy::TrainingRegionBalanced
        | super::options::BenchmarkHeadPolicy::TrainingQueryFacility => {
            let path = training_query_path.ok_or_else(|| {
                "EC_HEAD_SAMPLE: training policy requires benchmark_training_query_path".to_owned()
            })?;
            let queries = load_training_queries(path, usize::from(dimensions))?;
            let ranking = rank_training_candidates(&queries, vec_ids, codes, codec_artifact)?;
            match policy {
                super::options::BenchmarkHeadPolicy::TrainingLandmarks => {
                    frequency_landmark_nodes(ranking, vec_ids, vectors, sample_cap)
                }
                super::options::BenchmarkHeadPolicy::TrainingRegionBalanced => {
                    training_region_balanced_nodes(ranking, vec_ids, vectors, sample_cap)
                }
                super::options::BenchmarkHeadPolicy::TrainingQueryFacility => {
                    training_query_facility_nodes(ranking, vec_ids, vectors, sample_cap)
                }
                _ => unreachable!(),
            }
        }
    };
    let sample = DistannHeadSample {
        dimensions,
        entries: selected
            .into_iter()
            .map(|node| DistannHeadSampleEntry {
                vec_id: vec_ids[node],
                vector: vectors[node].clone(),
            })
            .collect(),
    };
    sample.validate(cap)?;
    Ok(sample)
}

pub(crate) fn persist_head_sample(
    client: &mut pgrx::spi::SpiClient<'_>,
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    sample: &DistannHeadSample,
    graph: &DistannPersistedHeadGraph,
    build_options: super::generation_descriptor::DistannBuildOptions,
) -> Result<(), String> {
    graph.validate(sample.entries.len(), usize::MAX)?;
    let digest = sample.digest()?;
    let graph_digest = graph.digest()?;
    let state = extension_relation_name("ec_distann_generation_head_state")?;
    let rows = extension_relation_name("ec_distann_generation_head_sample")?;
    client
        .update(
            &format!(
                "INSERT INTO {state} (
                     index_oid, logical_index_uuid, build_id, dimensions,
                     sample_count, head_sample_digest,
                     head_policy, training_query_count, training_query_digest,
                     head_graph_entry, head_graph_digest
                 ) VALUES ($1::oid, $2::uuid, $3::uuid, $4::integer,
                           $5::integer, $6::bytea, $7::smallint, $8::integer,
                           $9::bytea, $10::integer, $11::bytea)"
            ),
            None,
            &[
                index_oid.into(),
                logical_index_uuid.into(),
                build_id.into(),
                i32::from(sample.dimensions).into(),
                i32::try_from(sample.entries.len())
                    .map_err(|_| "EC_HEAD_SAMPLE: sample count exceeds integer".to_owned())?
                    .into(),
                digest.to_vec().into(),
                i16::from(build_options.head_policy as u8).into(),
                i32::try_from(build_options.training_query_count)
                    .map_err(|_| "EC_HEAD_SAMPLE: training query count exceeds integer".to_owned())?
                    .into(),
                build_options.training_query_digest.to_vec().into(),
                i32::try_from(graph.entry)
                    .map_err(|_| "EC_HEAD_SAMPLE: graph entry exceeds integer".to_owned())?
                    .into(),
                graph_digest.to_vec().into(),
            ],
        )
        .map_err(|error| format!("EC_HEAD_SAMPLE: state insert failed: {error}"))?;
    for (ordinal, entry) in sample.entries.iter().enumerate() {
        client
            .update(
                &format!(
                    "INSERT INTO {rows} (
                         index_oid, logical_index_uuid, build_id, sample_ordinal,
                         vec_id, vector, neighbors
                     ) VALUES ($1::oid, $2::uuid, $3::uuid, $4::integer,
                               $5::bigint, $6::real[], $7::integer[])"
                ),
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    build_id.into(),
                    i32::try_from(ordinal)
                        .map_err(|_| "EC_HEAD_SAMPLE: ordinal exceeds integer".to_owned())?
                        .into(),
                    i64::from_le_bytes(entry.vec_id.to_le_bytes()).into(),
                    entry.vector.as_slice().into(),
                    graph.neighbors[ordinal]
                        .iter()
                        .map(|neighbor| i32::try_from(*neighbor))
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|_| "EC_HEAD_SAMPLE: graph neighbor exceeds integer".to_owned())?
                        .into(),
                ],
            )
            .map_err(|error| format!("EC_HEAD_SAMPLE: sample insert failed: {error}"))?;
    }
    Ok(())
}

pub(crate) fn load_head_sample(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    expected_fingerprint: &[u8; 34],
) -> Result<
    (
        DistannHeadSample,
        DistannPersistedHeadGraph,
        super::manifest_v2::DistannManifestBuildOptions,
    ),
    String,
> {
    let candidate = extension_relation_name("ec_distann_build_candidate")?;
    let state = extension_relation_name("ec_distann_generation_head_state")?;
    let rows = extension_relation_name("ec_distann_generation_head_sample")?;
    let (
        manifest_bytes,
        dimensions,
        sample_count,
        stored_digest,
        stored_policy,
        stored_training_count,
        stored_training_digest,
        graph_entry,
        stored_graph_digest,
    ) = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT candidate.epoch_manifest, state.dimensions,
                            state.sample_count, state.head_sample_digest,
                            state.head_policy, state.training_query_count,
                            state.training_query_digest,
                            state.head_graph_entry, state.head_graph_digest
                       FROM {candidate} candidate
                       JOIN {state} state USING (index_oid, logical_index_uuid, build_id)
                      WHERE candidate.index_oid = $1::oid
                        AND candidate.logical_index_uuid = $2::uuid
                        AND candidate.build_id = $3::uuid
                        AND candidate.epoch_fingerprint = $4::bytea"
                ),
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    build_id.into(),
                    expected_fingerprint.to_vec().into(),
                ],
            )
            .map_err(|error| format!("EC_HEAD_SAMPLE: state lookup failed: {error}"))?
            .map(|row| {
                Ok((
                    row["epoch_manifest"]
                        .value::<Vec<u8>>()?
                        .ok_or("manifest NULL")?,
                    row["dimensions"].value::<i32>()?.ok_or("dimensions NULL")?,
                    row["sample_count"]
                        .value::<i32>()?
                        .ok_or("sample count NULL")?,
                    row["head_sample_digest"]
                        .value::<Vec<u8>>()?
                        .ok_or("digest NULL")?,
                    row["head_policy"]
                        .value::<i16>()?
                        .ok_or("head policy NULL")?,
                    row["training_query_count"]
                        .value::<i32>()?
                        .ok_or("training query count NULL")?,
                    row["training_query_digest"]
                        .value::<Vec<u8>>()?
                        .ok_or("training query digest NULL")?,
                    row["head_graph_entry"]
                        .value::<i32>()?
                        .ok_or("graph entry NULL")?,
                    row["head_graph_digest"]
                        .value::<Vec<u8>>()?
                        .ok_or("graph digest NULL")?,
                ))
            })
            .next()
            .transpose()
            .map_err(|error: Box<dyn std::error::Error + Send + Sync>| {
                format!("EC_HEAD_SAMPLE: state decode failed: {error}")
            })?
            .ok_or_else(|| "EC_HEAD_SAMPLE: exact persisted head state is missing".to_owned())
    })?;
    let manifest = DistannEpochManifestV2::decode(&manifest_bytes)?;
    if manifest.fingerprint()?.as_bytes() != expected_fingerprint {
        return Err("EC_HEAD_SAMPLE: candidate manifest fingerprint mismatch".to_owned());
    }
    let cap = manifest.build_options.options.head_index_cap as usize;
    let dimensions = u16::try_from(dimensions)
        .map_err(|_| "EC_HEAD_SAMPLE: stored dimensions are invalid".to_owned())?;
    let sample_count = usize::try_from(sample_count)
        .map_err(|_| "EC_HEAD_SAMPLE: stored sample count is invalid".to_owned())?;
    let stored_digest: [u8; 32] = stored_digest
        .try_into()
        .map_err(|_| "EC_HEAD_SAMPLE: stored digest is not 32 bytes".to_owned())?;
    let stored_policy = u8::try_from(stored_policy)
        .map_err(|_| "EC_HEAD_SAMPLE: stored head policy is invalid".to_owned())?;
    let stored_training_count = u32::try_from(stored_training_count)
        .map_err(|_| "EC_HEAD_SAMPLE: stored training query count is invalid".to_owned())?;
    let stored_training_digest: [u8; 32] = stored_training_digest
        .try_into()
        .map_err(|_| "EC_HEAD_SAMPLE: stored training query digest is not 32 bytes".to_owned())?;
    let graph_entry = u32::try_from(graph_entry)
        .map_err(|_| "EC_HEAD_SAMPLE: stored graph entry is invalid".to_owned())?;
    let stored_graph_digest: [u8; 32] = stored_graph_digest
        .try_into()
        .map_err(|_| "EC_HEAD_SAMPLE: stored graph digest is not 32 bytes".to_owned())?;
    if sample_count > cap || stored_digest != manifest.head_sample_digest {
        return Err("EC_HEAD_SAMPLE: state exceeds cap or disagrees with manifest".to_owned());
    }
    if stored_policy != manifest.build_options.options.head_policy as u8
        || stored_training_count != manifest.build_options.options.training_query_count
        || stored_training_digest != manifest.build_options.options.training_query_digest
    {
        return Err("EC_HEAD_SAMPLE: stored head policy/input disagrees with manifest".to_owned());
    }
    if (manifest.global_record_count == 0) != (sample_count == 0) {
        return Err("EC_HEAD_SAMPLE: empty/nonempty state disagrees with manifest".to_owned());
    }
    let entries = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT sample_ordinal, vec_id, vector, neighbors FROM {rows}
                      WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                        AND build_id = $3::uuid ORDER BY sample_ordinal"
                ),
                None,
                &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
            )
            .map_err(|error| format!("EC_HEAD_SAMPLE: row lookup failed: {error}"))?
            .enumerate()
            .map(|(expected, row)| {
                let ordinal = row["sample_ordinal"]
                    .value::<i32>()?
                    .ok_or("ordinal NULL")?;
                if usize::try_from(ordinal).ok() != Some(expected) {
                    return Err("head sample ordinals are not contiguous".into());
                }
                let vec_id = row["vec_id"].value::<i64>()?.ok_or("vec_id NULL")?;
                let vector = row["vector"].value::<Vec<f32>>()?.ok_or("vector NULL")?;
                let neighbors = row["neighbors"]
                    .value::<Vec<i32>>()?
                    .ok_or("neighbors NULL")?
                    .into_iter()
                    .map(|neighbor| {
                        u32::try_from(neighbor).map_err(|_| "negative head graph neighbor")
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((
                    DistannHeadSampleEntry {
                        vec_id: u64::from_le_bytes(vec_id.to_le_bytes()),
                        vector,
                    },
                    neighbors,
                ))
            })
            .collect::<Result<Vec<_>, Box<dyn std::error::Error + Send + Sync>>>()
            .map_err(|error| format!("EC_HEAD_SAMPLE: row decode failed: {error}"))
    })?;
    if entries.len() != sample_count {
        return Err("EC_HEAD_SAMPLE: row count differs from persisted state".to_owned());
    }
    let (entries, neighbors) = entries.into_iter().unzip();
    let sample = DistannHeadSample {
        dimensions,
        entries,
    };
    sample.validate(cap)?;
    if sample.digest()? != stored_digest {
        return Err("EC_HEAD_SAMPLE: canonical digest mismatch".to_owned());
    }
    let graph = DistannPersistedHeadGraph {
        entry: graph_entry,
        neighbors,
    };
    graph.validate(
        sample.entries.len(),
        usize::from(manifest.build_options.graph_degree),
    )?;
    if graph.digest()? != stored_graph_digest {
        return Err("EC_HEAD_SAMPLE: persisted head graph digest mismatch".to_owned());
    }
    Ok((sample, graph, manifest.build_options))
}

/// One roster shard of the FR-080 head: the landmarks whose FR-078 placement
/// hash maps to `owner_ordinal`, plus a head graph built over *that shard's*
/// vectors only.
///
/// NFR-021 clause 3 (Task 210 P2): the head is sharded across the roster
/// regardless of its capacity, so no node holds the whole head. Note the graph
/// is per shard, not a slice of the global head graph — a subgraph of the
/// stitched head is not a navigable index over the shard, which is the same
/// reason `DISTRIBUTEDANN` §3 builds the head from per-partition top layers
/// rather than from the stitched graph.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DistannHeadShard {
    pub(crate) owner_ordinal: u32,
    pub(crate) sample: DistannHeadSample,
    pub(crate) graph: DistannPersistedHeadGraph,
}

/// Partition a head sample across the roster by FR-078 placement hash and
/// build a per-shard head graph. Every landmark lands on exactly one owner;
/// the union of the shards is the input sample.
pub(crate) fn shard_head_sample(
    sample: &DistannHeadSample,
    owner_count: usize,
    placement_hash_version: u16,
    graph_degree: usize,
    build_list_size: usize,
    alpha: f32,
    seed: u64,
) -> Result<Vec<DistannHeadShard>, String> {
    if owner_count == 0 {
        return Err("EC_HEAD_SHARD: roster has no owners".to_owned());
    }
    let mut shards = (0..owner_count)
        .map(|owner_ordinal| DistannHeadShard {
            owner_ordinal: owner_ordinal as u32,
            sample: DistannHeadSample {
                dimensions: sample.dimensions,
                entries: Vec::new(),
            },
            graph: DistannPersistedHeadGraph {
                entry: 0,
                neighbors: Vec::new(),
            },
        })
        .collect::<Vec<_>>();
    for entry in &sample.entries {
        let owner =
            super::placement::owning_node(entry.vec_id, owner_count, placement_hash_version);
        shards[owner].sample.entries.push(entry.clone());
    }
    for shard in &mut shards {
        shard.graph = DistannPersistedHeadGraph::build(
            &shard.sample,
            graph_degree,
            build_list_size,
            alpha,
            // Per-shard seed keeps shard graphs independent yet deterministic.
            seed ^ u64::from(shard.owner_ordinal).wrapping_mul(0x9e37_79b9_7f4a_7c15),
        )?;
    }
    Ok(shards)
}

/// Merge bounded per-owner head results into the coordinator's seed list.
///
/// The coordinator holds only `seed_count` seeds — NFR-021 clause 2 bounded
/// state — and the merge is deterministic on `(dist, vec_id)`, so a sharded
/// head returns a stable ordered seed list.
pub(crate) fn merge_head_seeds(
    per_owner: Vec<Vec<super::scan::DistannSeedCandidate>>,
    seed_count: usize,
) -> Vec<super::scan::DistannSeedCandidate> {
    let mut merged = per_owner.into_iter().flatten().collect::<Vec<_>>();
    merged.sort_unstable_by(|left, right| {
        left.dist
            .total_cmp(&right.dist)
            .then_with(|| left.vec_id.cmp(&right.vec_id))
    });
    merged.dedup_by(|left, right| left.vec_id == right.vec_id);
    merged.truncate(seed_count.min(merged.len()));
    merged
}

pub(crate) struct DistannPhysicalHeadIndex {
    graph: crate::am::VamanaGraph,
    entry: u32,
    vec_ids: Vec<u64>,
    vectors: Vec<Vec<f32>>,
    policy: super::generation_descriptor::DistannHeadPolicy,
}

impl DistannPhysicalHeadIndex {
    pub(crate) fn load(
        sample: DistannHeadSample,
        persisted: DistannPersistedHeadGraph,
        graph_degree: usize,
        policy: super::generation_descriptor::DistannHeadPolicy,
    ) -> Result<Option<Self>, String> {
        if sample.entries.is_empty() {
            persisted.validate(0, graph_degree)?;
            return Ok(None);
        }
        persisted.validate(sample.entries.len(), graph_degree)?;
        let vec_ids = sample.entries.iter().map(|entry| entry.vec_id).collect();
        let vectors = sample
            .entries
            .into_iter()
            .map(|entry| entry.vector)
            .collect::<Vec<_>>();
        Ok(Some(Self {
            graph: crate::am::VamanaGraph {
                neighbors: persisted.neighbors,
                max_degree: graph_degree,
            },
            entry: persisted.entry,
            vec_ids,
            vectors,
            policy,
        }))
    }

    pub(crate) fn search_configured(
        &self,
        query: &[f32],
        search_width: usize,
        seed_count: usize,
    ) -> Vec<super::scan::DistannSeedCandidate> {
        match self.policy {
            super::generation_descriptor::DistannHeadPolicy::CurrentSampleGraph => {
                self.search(query, search_width, seed_count)
            }
            super::generation_descriptor::DistannHeadPolicy::TrainingLandmarksExact => {
                self.search_exact(query, TRAINED_HEAD_SEED_COUNT.min(seed_count))
            }
        }
    }

    pub(crate) const fn policy(&self) -> super::generation_descriptor::DistannHeadPolicy {
        self.policy
    }

    pub(crate) fn search(
        &self,
        query: &[f32],
        search_width: usize,
        seed_count: usize,
    ) -> Vec<super::scan::DistannSeedCandidate> {
        #[cfg(feature = "distann-head-attribution-benchmark")]
        if seed_count > search_width {
            use std::cell::RefCell;

            // The Vamana frontier contains at most `search_width` entries, but
            // the attribution sweep intentionally varies returned seed count
            // independently. Retain every candidate the bounded search already
            // scored so a larger seed request does not silently collapse back
            // to the frontier width. This does not expand extra graph nodes or
            // perform an exact sample scan; head work remains bounded by the
            // persisted sample and the configured search width.
            let scored = RefCell::new(vec![None; self.vectors.len()]);
            let result = crate::am::greedy_search(
                &self.graph,
                self.entry,
                search_width.max(1).min(self.vectors.len()),
                |node| {
                    let dist = -crate::am::ec_diskann::source_inner_product(
                        query,
                        &self.vectors[node as usize],
                    );
                    scored.borrow_mut()[node as usize] = Some(dist);
                    dist
                },
            );
            let mut seeds = scored
                .into_inner()
                .into_iter()
                .enumerate()
                .filter_map(|(node, dist)| {
                    dist.map(|dist| super::scan::DistannSeedCandidate {
                        vec_id: self.vec_ids[node],
                        dist,
                    })
                })
                .collect::<Vec<_>>();
            seeds.sort_unstable_by(|left, right| {
                left.dist
                    .total_cmp(&right.dist)
                    .then_with(|| left.vec_id.cmp(&right.vec_id))
            });
            seeds.truncate(seed_count.min(seeds.len()));

            debug_assert!(result.frontier.len() <= search_width.max(1));
            return seeds;
        }

        let mut seeds = crate::am::greedy_search(
            &self.graph,
            self.entry,
            search_width.max(1).min(self.vectors.len()),
            |node| {
                -crate::am::ec_diskann::source_inner_product(query, &self.vectors[node as usize])
            },
        )
        .frontier
        .into_iter()
        .map(|candidate| super::scan::DistannSeedCandidate {
            vec_id: self.vec_ids[candidate.node as usize],
            dist: candidate.distance,
        })
        .collect::<Vec<_>>();
        seeds.truncate(seed_count.min(seeds.len()));
        seeds
    }

    pub(crate) fn search_exact(
        &self,
        query: &[f32],
        seed_count: usize,
    ) -> Vec<super::scan::DistannSeedCandidate> {
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let score_started = Instant::now();
        let mut seeds = self
            .vectors
            .iter()
            .enumerate()
            .map(|(node, vector)| super::scan::DistannSeedCandidate {
                vec_id: self.vec_ids[node],
                dist: -crate::am::ec_diskann::source_inner_product(query, vector),
            })
            .collect::<Vec<_>>();
        #[cfg(feature = "distann-head-attribution-benchmark")]
        super::stage_counters::record(
            super::stage_counters::DistannQueryStage::HeadScore,
            score_started.elapsed(),
        );
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let select_started = Instant::now();
        seeds.sort_unstable_by(|left, right| {
            left.dist
                .total_cmp(&right.dist)
                .then_with(|| left.vec_id.cmp(&right.vec_id))
        });
        seeds.truncate(seed_count.min(seeds.len()));
        #[cfg(feature = "distann-head-attribution-benchmark")]
        super::stage_counters::record(
            super::stage_counters::DistannQueryStage::SeedSelect,
            select_started.elapsed(),
        );
        seeds
    }

    /// Task 181 pre-registered two-level bounded hierarchy: score at most one
    /// representative for each of 256 deterministic geometric regions, open
    /// at most 16 regions, score at most 512 second-level landmarks, and return
    /// the requested bounded seed prefix.
    pub(crate) fn search_hierarchy(
        &self,
        query: &[f32],
        seed_count: usize,
    ) -> Vec<super::scan::DistannSeedCandidate> {
        let mut regions = BTreeMap::<u8, Vec<usize>>::new();
        for (node, vector) in self.vectors.iter().enumerate() {
            regions
                .entry((benchmark_geometry_region(vector) >> 4) as u8)
                .or_default()
                .push(node);
        }
        let mut first_level = regions
            .iter()
            .filter_map(|(region, nodes)| {
                nodes.first().map(|node| {
                    (
                        -crate::am::ec_diskann::source_inner_product(query, &self.vectors[*node]),
                        *region,
                    )
                })
            })
            .collect::<Vec<_>>();
        first_level.sort_unstable_by(|left, right| {
            left.0
                .total_cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
        });
        first_level.truncate(16);
        let mut second_level = Vec::with_capacity(512);
        for (_, region) in first_level {
            for node in regions.get(&region).into_iter().flatten() {
                if second_level.len() == 512 {
                    break;
                }
                second_level.push(super::scan::DistannSeedCandidate {
                    vec_id: self.vec_ids[*node],
                    dist: -crate::am::ec_diskann::source_inner_product(query, &self.vectors[*node]),
                });
            }
            if second_level.len() == 512 {
                break;
            }
        }
        second_level.sort_unstable_by(|left, right| {
            left.dist
                .total_cmp(&right.dist)
                .then_with(|| left.vec_id.cmp(&right.vec_id))
        });
        second_level.truncate(seed_count.min(second_level.len()));
        second_level
    }

    pub(crate) fn sample_count(&self) -> usize {
        self.vectors.len()
    }

    #[cfg(feature = "distann-head-attribution-benchmark")]
    pub(crate) fn contains_vec_id(&self, vec_id: u64) -> bool {
        self.vec_ids.contains(&vec_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::ec_distann::generation_descriptor::DistannHeadPolicy;

    fn sample() -> DistannHeadSample {
        DistannHeadSample {
            dimensions: 2,
            entries: vec![
                DistannHeadSampleEntry {
                    vec_id: 10,
                    vector: vec![1.0, 0.0],
                },
                DistannHeadSampleEntry {
                    vec_id: 20,
                    vector: vec![0.0, 1.0],
                },
                DistannHeadSampleEntry {
                    vec_id: 30,
                    vector: vec![-1.0, 0.0],
                },
                DistannHeadSampleEntry {
                    vec_id: 40,
                    vector: vec![0.0, -1.0],
                },
            ],
        }
    }

    fn wide_sample(count: u64) -> DistannHeadSample {
        DistannHeadSample {
            dimensions: 2,
            entries: (0..count)
                .map(|index| {
                    let angle = index as f32 * 0.37;
                    DistannHeadSampleEntry {
                        vec_id: (index + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15),
                        vector: vec![angle.cos(), angle.sin()],
                    }
                })
                .collect(),
        }
    }

    #[test]
    fn head_shards_partition_every_landmark_exactly_once() {
        let sample = wide_sample(64);
        let shards = shard_head_sample(
            &sample,
            3,
            crate::am::ec_distann::placement::DISTANN_PLACEMENT_HASH_V1,
            8,
            16,
            1.2,
            17,
        )
        .unwrap();

        assert_eq!(shards.len(), 3);
        let mut union = shards
            .iter()
            .flat_map(|shard| shard.sample.entries.iter().map(|entry| entry.vec_id))
            .collect::<Vec<_>>();
        union.sort_unstable();
        let mut expected = sample
            .entries
            .iter()
            .map(|entry| entry.vec_id)
            .collect::<Vec<_>>();
        expected.sort_unstable();
        assert_eq!(union, expected, "shards must partition the head sample");
        // Every shard holds strictly less than the whole head: that is the
        // property NFR-021 clause 3 asks for.
        for shard in &shards {
            assert!(shard.sample.entries.len() < sample.entries.len());
            assert_eq!(shard.graph.neighbors.len(), shard.sample.entries.len());
        }
    }

    #[test]
    fn sharded_exact_head_search_is_identical_to_the_unsharded_head() {
        let sample = wide_sample(64);
        let query = vec![0.4_f32, -0.9];
        let seed_count = 8;

        let global_graph = DistannPersistedHeadGraph::build(&sample, 8, 16, 1.2, 17).unwrap();
        let global = DistannPhysicalHeadIndex::load(
            sample.clone(),
            global_graph,
            8,
            DistannHeadPolicy::TrainingLandmarksExact,
        )
        .unwrap()
        .unwrap();
        let unsharded = global.search_exact(&query, seed_count);

        let shards = shard_head_sample(
            &sample,
            3,
            crate::am::ec_distann::placement::DISTANN_PLACEMENT_HASH_V1,
            8,
            16,
            1.2,
            17,
        )
        .unwrap();
        let per_owner = shards
            .into_iter()
            .map(|shard| {
                DistannPhysicalHeadIndex::load(
                    shard.sample,
                    shard.graph,
                    8,
                    DistannHeadPolicy::TrainingLandmarksExact,
                )
                .unwrap()
                .map(|index| index.search_exact(&query, seed_count))
                .unwrap_or_default()
            })
            .collect::<Vec<_>>();
        let merged = merge_head_seeds(per_owner, seed_count);

        // Exact scoring over a partition is exact scoring over the union, so
        // sharding the head is result-identical for the promoted policy.
        assert_eq!(merged.len(), unsharded.len());
        for (sharded, reference) in merged.iter().zip(unsharded.iter()) {
            assert_eq!(sharded.vec_id, reference.vec_id);
            assert_eq!(sharded.dist, reference.dist);
        }
    }

    #[test]
    fn merged_head_seeds_are_bounded_deduplicated_and_deterministic() {
        let left = vec![
            super::super::scan::DistannSeedCandidate {
                vec_id: 7,
                dist: -0.9,
            },
            super::super::scan::DistannSeedCandidate {
                vec_id: 3,
                dist: -0.5,
            },
        ];
        let right = vec![
            super::super::scan::DistannSeedCandidate {
                vec_id: 7,
                dist: -0.9,
            },
            super::super::scan::DistannSeedCandidate {
                vec_id: 9,
                dist: -0.7,
            },
        ];

        let merged = merge_head_seeds(vec![left, right], 2);

        assert_eq!(merged.len(), 2, "coordinator state stays bounded by k_head");
        assert_eq!(merged[0].vec_id, 7);
        assert_eq!(merged[1].vec_id, 9);
    }

    #[test]
    fn persisted_head_graph_is_deterministic_and_loadable() {
        let sample = sample();
        let first = DistannPersistedHeadGraph::build(&sample, 2, 4, 1.2, 17).unwrap();
        let second = DistannPersistedHeadGraph::build(&sample, 2, 4, 1.2, 17).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.digest().unwrap(), second.digest().unwrap());

        let index =
            DistannPhysicalHeadIndex::load(sample, first, 2, DistannHeadPolicy::CurrentSampleGraph)
                .unwrap()
                .unwrap();
        let seeds = index.search(&[1.0, 0.0], 2, 2);
        assert!(!seeds.is_empty());
        assert!(seeds.len() <= 2);
        assert_eq!(index.sample_count(), 4);

        let exact = index.search_exact(&[1.0, 0.0], 2);
        assert_eq!(exact.len(), 2);
        assert_eq!(exact[0].vec_id, 10);
        assert!(exact[0].dist <= exact[1].dist);
    }

    #[cfg(feature = "distann-head-attribution-benchmark")]
    #[test]
    fn benchmark_search_can_return_more_seeds_than_frontier_width() {
        let sample = sample();
        let graph = DistannPersistedHeadGraph::build(&sample, 2, 4, 1.2, 17).unwrap();
        let index =
            DistannPhysicalHeadIndex::load(sample, graph, 2, DistannHeadPolicy::CurrentSampleGraph)
                .unwrap()
                .unwrap();

        let seeds = index.search(&[1.0, 0.0], 1, 4);

        assert!(seeds.len() > 1);
        assert!(seeds.len() <= 4);
        assert_eq!(seeds[0].vec_id, 10);

        let hierarchical = index.search_hierarchy(&[1.0, 0.0], 4);
        assert!(!hierarchical.is_empty());
        assert!(hierarchical.len() <= 4);
    }

    #[cfg(feature = "distann-head-attribution-benchmark")]
    #[test]
    fn benchmark_landmark_policies_are_deterministic_and_bounded() {
        let vec_ids = (0_u64..16).map(|value| value + 100).collect::<Vec<_>>();
        let vectors = (0..16)
            .map(|value| vec![value as f32 / 16.0, 1.0 - value as f32 / 16.0])
            .collect::<Vec<_>>();
        let graph = crate::am::VamanaGraph {
            neighbors: (0..16).map(|node| vec![((node + 1) % 16) as u32]).collect(),
            max_degree: 1,
        };

        let geometry = geometry_landmark_nodes(&vec_ids, &vectors, 8);
        assert_eq!(geometry, geometry_landmark_nodes(&vec_ids, &vectors, 8));
        assert_eq!(geometry.len(), 8);
        let graph_landmarks = graph_landmark_nodes(&graph, 0, &vec_ids, 8).unwrap();
        assert_eq!(
            graph_landmarks,
            graph_landmark_nodes(&graph, 0, &vec_ids, 8).unwrap()
        );
        assert_eq!(graph_landmarks.len(), 8);
        assert_eq!(
            graph_landmarks
                .iter()
                .copied()
                .collect::<HashSet<_>>()
                .len(),
            8
        );

        let ranking = TrainingCandidateRanking {
            per_query: vec![
                vec![0, 1, 2, 3],
                vec![4, 5, 6, 7],
                vec![8, 9, 10, 11],
                vec![12, 13, 14, 15],
            ],
            frequency: (0..16).map(|node| (node, (1, node % 4))).collect(),
        };
        let region = training_region_balanced_nodes(ranking.clone(), &vec_ids, &vectors, 8);
        assert_eq!(
            region,
            training_region_balanced_nodes(ranking.clone(), &vec_ids, &vectors, 8)
        );
        let facility = training_query_facility_nodes(ranking.clone(), &vec_ids, &vectors, 8);
        assert_eq!(
            facility,
            training_query_facility_nodes(ranking, &vec_ids, &vectors, 8)
        );
        for selected in [region, facility] {
            assert_eq!(selected.len(), 8);
            assert_eq!(selected.iter().copied().collect::<HashSet<_>>().len(), 8);
        }
    }

    #[test]
    fn persisted_head_graph_rejects_out_of_range_neighbors() {
        let sample = sample();
        let graph = DistannPersistedHeadGraph {
            entry: 0,
            neighbors: vec![vec![4], vec![], vec![], vec![]],
        };
        assert!(DistannPhysicalHeadIndex::load(
            sample,
            graph,
            2,
            DistannHeadPolicy::CurrentSampleGraph,
        )
        .is_err());
    }

    #[test]
    fn training_query_set_digest_and_exact_policy_are_deterministic() {
        let rows = (0..super::super::generation_descriptor::DISTANN_TRAINING_QUERY_COUNT)
            .map(|ordinal| (i64::from(ordinal), vec![ordinal as f32, 1.0]))
            .collect::<Vec<_>>();
        let first = DistannTrainingQuerySet::from_ordered_rows(rows.clone(), 2).unwrap();
        let second = DistannTrainingQuerySet::from_ordered_rows(rows, 2).unwrap();
        assert_eq!(first, second);
        assert_ne!(first.digest, [0; 32]);

        let mut duplicates = (0..super::super::generation_descriptor::DISTANN_TRAINING_QUERY_COUNT)
            .map(|ordinal| (i64::from(ordinal), vec![ordinal as f32, 1.0]))
            .collect::<Vec<_>>();
        duplicates[1].0 = duplicates[0].0;
        assert!(DistannTrainingQuerySet::from_ordered_rows(duplicates, 2).is_err());

        let sample = sample();
        let graph = DistannPersistedHeadGraph::build(&sample, 2, 4, 1.2, 17).unwrap();
        let index = DistannPhysicalHeadIndex::load(
            sample,
            graph,
            2,
            DistannHeadPolicy::TrainingLandmarksExact,
        )
        .unwrap()
        .unwrap();
        let configured = index.search_configured(&[1.0, 0.0], 1, 2);
        assert_eq!(configured.len(), 2);
        assert_eq!(configured[0].vec_id, 10);
        assert_eq!(index.policy(), DistannHeadPolicy::TrainingLandmarksExact);
    }
}
