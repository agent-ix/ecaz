//! FR-080 canonical persisted coordinator head sample.

use std::collections::VecDeque;

use pgrx::datum::Uuid;
use pgrx::{pg_sys, Spi};
use sha2::{Digest, Sha256};

use super::generation_catalog::extension_relation_name;
use super::manifest_v2::DistannEpochManifestV2;

const HEAD_SAMPLE_DOMAIN: &[u8] = b"ec_distann_head_sample_v1\0";
const HEAD_GRAPH_DOMAIN: &[u8] = b"ec_distann_head_graph_v1\0";
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

pub(crate) fn persist_head_sample(
    client: &mut pgrx::spi::SpiClient<'_>,
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    sample: &DistannHeadSample,
    graph: &DistannPersistedHeadGraph,
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
                     head_graph_entry, head_graph_digest
                 ) VALUES ($1::oid, $2::uuid, $3::uuid, $4::integer,
                           $5::integer, $6::bytea, $7::integer, $8::bytea)"
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
        graph_entry,
        stored_graph_digest,
    ) = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT candidate.epoch_manifest, state.dimensions,
                            state.sample_count, state.head_sample_digest,
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
    let graph_entry = u32::try_from(graph_entry)
        .map_err(|_| "EC_HEAD_SAMPLE: stored graph entry is invalid".to_owned())?;
    let stored_graph_digest: [u8; 32] = stored_graph_digest
        .try_into()
        .map_err(|_| "EC_HEAD_SAMPLE: stored graph digest is not 32 bytes".to_owned())?;
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

pub(crate) struct DistannPhysicalHeadIndex {
    graph: crate::am::VamanaGraph,
    entry: u32,
    vec_ids: Vec<u64>,
    vectors: Vec<Vec<f32>>,
}

impl DistannPhysicalHeadIndex {
    pub(crate) fn load(
        sample: DistannHeadSample,
        persisted: DistannPersistedHeadGraph,
        graph_degree: usize,
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn persisted_head_graph_is_deterministic_and_loadable() {
        let sample = sample();
        let first = DistannPersistedHeadGraph::build(&sample, 2, 4, 1.2, 17).unwrap();
        let second = DistannPersistedHeadGraph::build(&sample, 2, 4, 1.2, 17).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.digest().unwrap(), second.digest().unwrap());

        let index = DistannPhysicalHeadIndex::load(sample, first, 2)
            .unwrap()
            .unwrap();
        let seeds = index.search(&[1.0, 0.0], 2);
        assert!(!seeds.is_empty());
        assert!(seeds.len() <= 2);
    }

    #[test]
    fn persisted_head_graph_rejects_out_of_range_neighbors() {
        let sample = sample();
        let graph = DistannPersistedHeadGraph {
            entry: 0,
            neighbors: vec![vec![4], vec![], vec![], vec![]],
        };
        assert!(DistannPhysicalHeadIndex::load(sample, graph, 2).is_err());
    }
}
