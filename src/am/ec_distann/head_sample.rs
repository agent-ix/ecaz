//! FR-080 canonical persisted coordinator head sample.

use std::collections::VecDeque;

use pgrx::datum::Uuid;
use pgrx::{pg_sys, Spi};
use sha2::{Digest, Sha256};

use super::generation_catalog::extension_relation_name;
use super::manifest_v2::DistannEpochManifestV2;

const HEAD_SAMPLE_DOMAIN: &[u8] = b"ec_distann_head_sample_v1\0";
const HEAD_SAMPLE_VERSION: u16 = 1;
const HEAD_GRAPH_SEED_WRAP: u64 = 0x6469_7374_5f74_6721;

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

pub(crate) fn persist_head_sample(
    client: &mut pgrx::spi::SpiClient<'_>,
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    sample: &DistannHeadSample,
) -> Result<(), String> {
    let digest = sample.digest()?;
    let state = extension_relation_name("ec_distann_generation_head_state")?;
    let rows = extension_relation_name("ec_distann_generation_head_sample")?;
    client
        .update(
            &format!(
                "INSERT INTO {state} (
                     index_oid, logical_index_uuid, build_id, dimensions,
                     sample_count, head_sample_digest
                 ) VALUES ($1::oid, $2::uuid, $3::uuid, $4::integer,
                           $5::integer, $6::bytea)"
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
            ],
        )
        .map_err(|error| format!("EC_HEAD_SAMPLE: state insert failed: {error}"))?;
    for (ordinal, entry) in sample.entries.iter().enumerate() {
        client
            .update(
                &format!(
                    "INSERT INTO {rows} (
                         index_oid, logical_index_uuid, build_id, sample_ordinal,
                         vec_id, vector
                     ) VALUES ($1::oid, $2::uuid, $3::uuid, $4::integer,
                               $5::bigint, $6::real[])"
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
        super::manifest_v2::DistannManifestBuildOptions,
    ),
    String,
> {
    let candidate = extension_relation_name("ec_distann_build_candidate")?;
    let state = extension_relation_name("ec_distann_generation_head_state")?;
    let rows = extension_relation_name("ec_distann_generation_head_sample")?;
    let (manifest_bytes, dimensions, sample_count, stored_digest) = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT candidate.epoch_manifest, state.dimensions,
                            state.sample_count, state.head_sample_digest
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
    if sample_count > cap || stored_digest != manifest.head_sample_digest {
        return Err("EC_HEAD_SAMPLE: state exceeds cap or disagrees with manifest".to_owned());
    }
    if (manifest.global_record_count == 0) != (sample_count == 0) {
        return Err("EC_HEAD_SAMPLE: empty/nonempty state disagrees with manifest".to_owned());
    }
    let entries = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT sample_ordinal, vec_id, vector FROM {rows}
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
                Ok(DistannHeadSampleEntry {
                    vec_id: u64::from_le_bytes(vec_id.to_le_bytes()),
                    vector,
                })
            })
            .collect::<Result<Vec<_>, Box<dyn std::error::Error + Send + Sync>>>()
            .map_err(|error| format!("EC_HEAD_SAMPLE: row decode failed: {error}"))
    })?;
    if entries.len() != sample_count {
        return Err("EC_HEAD_SAMPLE: row count differs from persisted state".to_owned());
    }
    let sample = DistannHeadSample {
        dimensions,
        entries,
    };
    sample.validate(cap)?;
    if sample.digest()? != stored_digest {
        return Err("EC_HEAD_SAMPLE: canonical digest mismatch".to_owned());
    }
    Ok((sample, manifest.build_options))
}

pub(crate) struct DistannPhysicalHeadIndex {
    graph: crate::am::VamanaGraph,
    entry: u32,
    vec_ids: Vec<u64>,
    vectors: Vec<Vec<f32>>,
}

impl DistannPhysicalHeadIndex {
    pub(crate) fn build(
        sample: DistannHeadSample,
        graph_degree: usize,
        build_list_size: usize,
        alpha: f32,
        seed: u64,
    ) -> Result<Option<Self>, String> {
        if sample.entries.is_empty() {
            return Ok(None);
        }
        let vec_ids = sample.entries.iter().map(|entry| entry.vec_id).collect();
        let vectors = sample
            .entries
            .into_iter()
            .map(|entry| entry.vector)
            .collect::<Vec<_>>();
        let distance = |left: u32, right: u32| {
            crate::am::ec_diskann::source_inner_product_distance(
                &vectors[left as usize],
                &vectors[right as usize],
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
        Ok(Some(Self {
            graph,
            entry,
            vec_ids,
            vectors,
        }))
    }

    pub(crate) fn search(
        &self,
        query: &[f32],
        limit: usize,
    ) -> Vec<super::scan::DistannSeedCandidate> {
        if limit >= self.vectors.len() {
            let mut seeds = self
                .vectors
                .iter()
                .enumerate()
                .map(|(node, vector)| super::scan::DistannSeedCandidate {
                    vec_id: self.vec_ids[node],
                    dist: -crate::am::ec_diskann::source_inner_product(query, vector),
                })
                .collect::<Vec<_>>();
            seeds.sort_unstable_by(|left, right| left.dist.total_cmp(&right.dist));
            return seeds;
        }
        crate::am::greedy_search(
            &self.graph,
            self.entry,
            limit.min(self.vectors.len()),
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
        .collect()
    }
}
