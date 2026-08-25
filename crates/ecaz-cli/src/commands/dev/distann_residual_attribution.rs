use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{bail, eyre, Context, Result};
use serde::{Deserialize, Serialize};

use crate::commands::bench::recall::{PredictionFile, TruthSet};

use super::distann_graph_diagnostic::GraphNode;

const CONTROL: &str = "prod-bw4-rabitq";
const BW8: &str = "task226-bw8-rabitq";
const EXACT: &str = "prod-bw4-exact-neighbor";
const OWNER: &str = "owner-bw4-rabitq";
const OWNER_EXACT: &str = "owner-bw4-exact-neighbor";
const REQUIRED_VARIANTS: [&str; 5] = [CONTROL, BW8, EXACT, OWNER, OWNER_EXACT];

pub(super) struct ResidualAttributionInputs<'a> {
    pub(super) query_offset: u32,
    pub(super) query_file_sha256: &'a str,
    pub(super) query_slice_sha256: &'a str,
    pub(super) top_k: usize,
    pub(super) truth_cache_path: &'a Path,
    pub(super) prediction_paths: &'a BTreeMap<String, PathBuf>,
    pub(super) query_trace_paths: &'a BTreeMap<String, PathBuf>,
    pub(super) graph_nodes: &'a [GraphNode],
    pub(super) logical_id_by_vec_id: &'a HashMap<i64, i64>,
}

pub(super) struct ResidualAttribution {
    rows: Vec<ResidualRow>,
    query_features: Vec<QueryFeatureRow>,
    summary: ResidualSummary,
}

impl ResidualAttribution {
    pub(super) fn jsonl(&self) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        for row in &self.rows {
            serde_json::to_writer(&mut output, row)?;
            output.push(b'\n');
        }
        Ok(output)
    }

    pub(super) fn summary_json(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec_pretty(&self.summary)?)
    }

    pub(super) fn query_features_jsonl(&self) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        for row in &self.query_features {
            serde_json::to_writer(&mut output, row)?;
            output.push(b'\n');
        }
        Ok(output)
    }

    pub(super) fn missed_truth_neighbors(&self) -> usize {
        self.summary.missed_truth_neighbors
    }

    pub(super) fn unknown_truth_neighbors(&self) -> usize {
        self.summary.unknown_truth_neighbors
    }

    pub(super) fn reconciliation_pass(&self) -> bool {
        self.summary.reconciliation_pass
    }
}

#[derive(Debug, Serialize)]
struct ResidualSummary {
    schema: &'static str,
    query_prefix: String,
    query_offset: u32,
    queries: usize,
    top_k: usize,
    query_file_sha256: String,
    query_slice_sha256: String,
    generation_identity: String,
    registered_variants: Vec<&'static str>,
    truth_neighbors: usize,
    missed_truth_neighbors: usize,
    classifications: BTreeMap<String, usize>,
    unknown_truth_neighbors: usize,
    reconciliation_pass: bool,
    task189_same_seed_approximate_ordering_trigger: bool,
}

#[derive(Debug, Serialize)]
struct ResidualRow {
    schema: &'static str,
    query_id: i64,
    query_ordinal: usize,
    query_offset: u32,
    query_slice_sha256: String,
    generation_identity: String,
    variant: &'static str,
    truth_rank: usize,
    truth_id: i64,
    truth_vec_id: Option<String>,
    classification: &'static str,
    generation_live: bool,
    production_seed_reachable: Option<bool>,
    owner_seed_reachable: Option<bool>,
    production_requested: bool,
    production_returned: bool,
    production_expanded: bool,
    production_retained: bool,
    production_exact_rerank: bool,
    production_final: bool,
    bw8_retained: bool,
    bw8_final: bool,
    same_seed_exact_retained: bool,
    same_seed_exact_final: bool,
    owner_retained: bool,
    owner_final: bool,
    owner_exact_retained: bool,
    owner_exact_final: bool,
    trace_truncated: bool,
}

#[derive(Debug, Serialize)]
struct QueryFeatureRow {
    schema: &'static str,
    query_id: i64,
    query_ordinal: usize,
    query_offset: u32,
    query_slice_sha256: String,
    generation_identity: String,
    control_variant: &'static str,
    candidate_variant: &'static str,
    control_truth_hits: usize,
    candidate_truth_hits: usize,
    paired_recall_delta: f64,
    candidate_improves: bool,
    candidate_loses: bool,
    round_cap_reached: bool,
    heap_saturated: bool,
    score_gap: Option<f32>,
    frontier_churn_rate: f64,
    repeated_node_rate: f64,
    remote_owner_requests: usize,
    response_bytes: usize,
    trace_truncated: bool,
}

#[derive(Debug, Deserialize)]
struct TruthCacheEnvelope {
    truth: TruthSet,
}

#[derive(Debug, Deserialize)]
struct TraceFile {
    schema: String,
    query_offset: u32,
    queries: usize,
    query_file_sha256: String,
    query_slice_sha256: String,
    traces: Vec<TraceEnvelope>,
}

#[derive(Debug, Deserialize)]
struct TraceEnvelope {
    query_id: i64,
    trace: RawTrace,
}

#[derive(Debug, Deserialize)]
struct RawTrace {
    epoch_fingerprint: String,
    truncated: bool,
    seed_ids: Vec<String>,
    expanded_live_ids: Vec<String>,
    rounds: Vec<RawRound>,
    exact_rerank_ids: Vec<String>,
    final_ids: Vec<String>,
    #[serde(default)]
    expanded_unique: usize,
    #[serde(default)]
    expanded_overlap: usize,
    #[serde(default)]
    rounds_executed: usize,
}

#[derive(Debug, Deserialize)]
struct RawRound {
    requested_ids: Vec<String>,
    returned_ids: Vec<String>,
    exact_input_ids: Vec<String>,
    retained_ids: Vec<String>,
    #[serde(default)]
    heap_saturated: bool,
    #[serde(default)]
    frontier_score_gap: Option<f32>,
    #[serde(default)]
    owner_ordinals: Vec<u32>,
    #[serde(default)]
    owner_request_counts: Vec<usize>,
    #[serde(default)]
    response_bytes: usize,
}

#[derive(Debug)]
struct NormalizedTrace {
    generation_identity: String,
    truncated: bool,
    seeds: Vec<u64>,
    requested: HashSet<u64>,
    returned: HashSet<u64>,
    expanded: HashSet<u64>,
    retained: HashSet<u64>,
    exact_rerank: HashSet<u64>,
    final_order: Vec<u64>,
    final_ids: HashSet<u64>,
    round_cap_reached: bool,
    heap_saturated: bool,
    score_gap: Option<f32>,
    frontier_churn_rate: f64,
    repeated_node_rate: f64,
    remote_owner_requests: usize,
    response_bytes: usize,
}

#[derive(Debug)]
struct VariantTraces {
    traces: BTreeMap<i64, NormalizedTrace>,
}

struct GraphIndex {
    ids: Vec<u64>,
    index: HashMap<u64, usize>,
    adjacency: Vec<Vec<usize>>,
}

#[derive(Clone, Copy, Default)]
struct ClassificationEvidence {
    generation_mapped: bool,
    generation_live: bool,
    trace_truncated: bool,
    production_seed_reachable: bool,
    owner_seed_reachable: bool,
    production_requested: bool,
    production_returned: bool,
    production_expanded: bool,
    production_retained: bool,
    production_exact_rerank: bool,
    same_seed_exact_retained: bool,
}

pub(super) fn classify_residuals(
    input: ResidualAttributionInputs<'_>,
) -> Result<ResidualAttribution> {
    if input.top_k == 0 {
        bail!("residual attribution top_k must be positive");
    }
    let truth: TruthCacheEnvelope =
        serde_json::from_slice(&fs::read(input.truth_cache_path).wrap_err_with(|| {
            format!("reading truth cache {}", input.truth_cache_path.display())
        })?)
        .wrap_err("decoding residual-attribution truth cache")?;

    let mut predictions = BTreeMap::new();
    let mut variants = BTreeMap::new();
    for variant in REQUIRED_VARIANTS {
        let prediction_path = input
            .prediction_paths
            .get(variant)
            .ok_or_else(|| eyre!("residual attribution missing {variant} predictions"))?;
        predictions.insert(variant, load_predictions(prediction_path, input.top_k)?);
        let trace_path = input
            .query_trace_paths
            .get(variant)
            .ok_or_else(|| eyre!("residual attribution missing {variant} query trace"))?;
        variants.insert(
            variant,
            load_trace_file(
                trace_path,
                input.query_offset,
                input.query_file_sha256,
                input.query_slice_sha256,
            )?,
        );
    }

    let control_predictions = predictions
        .get(CONTROL)
        .expect("required control predictions");
    if truth.truth.ids.len() != control_predictions.query_ids.len() {
        bail!(
            "truth rows {} do not match control query rows {}",
            truth.truth.ids.len(),
            control_predictions.query_ids.len()
        );
    }
    for variant in REQUIRED_VARIANTS {
        let prediction = predictions.get(variant).expect("required predictions");
        if prediction.query_ids != control_predictions.query_ids {
            bail!("residual attribution query ids differ for variant {variant}");
        }
        let trace = variants.get(variant).expect("required trace");
        let trace_ids = trace.traces.keys().copied().collect::<Vec<_>>();
        if trace_ids != control_predictions.query_ids {
            bail!("residual attribution trace query ids differ for variant {variant}");
        }
    }

    let generation_identity = variants
        .get(CONTROL)
        .and_then(|variant| variant.traces.values().next())
        .map(|trace| trace.generation_identity.clone())
        .ok_or_else(|| eyre!("residual attribution has no control traces"))?;
    for (variant_name, variant) in &variants {
        if variant
            .traces
            .values()
            .any(|trace| trace.generation_identity != generation_identity)
        {
            bail!("residual attribution generation identity differs for {variant_name}");
        }
    }

    let logical_to_vec = input
        .logical_id_by_vec_id
        .iter()
        .map(|(vec_id, logical_id)| (*logical_id, u64::from_le_bytes(vec_id.to_le_bytes())))
        .collect::<HashMap<_, _>>();
    let graph = GraphIndex::new(input.graph_nodes);
    let mut rows = Vec::new();
    let mut query_features = Vec::new();
    let mut counts = BTreeMap::<String, usize>::new();
    let mut truth_neighbors = 0_usize;

    for (query_ordinal, query_id) in control_predictions.query_ids.iter().enumerate() {
        let control = trace_for(&variants, CONTROL, *query_id)?;
        validate_prediction_trace(
            CONTROL,
            *query_id,
            &control_predictions.predictions[query_ordinal],
            control,
            &logical_to_vec,
        )?;
        for variant in [BW8, EXACT, OWNER, OWNER_EXACT] {
            validate_prediction_trace(
                variant,
                *query_id,
                &predictions
                    .get(variant)
                    .expect("required predictions")
                    .predictions[query_ordinal],
                trace_for(&variants, variant, *query_id)?,
                &logical_to_vec,
            )?;
        }

        let missed = truth.truth.ids[query_ordinal]
            .iter()
            .take(input.top_k)
            .enumerate()
            .filter(|(_, truth_id)| {
                !control_predictions.predictions[query_ordinal]
                    .iter()
                    .take(input.top_k)
                    .any(|prediction| prediction == *truth_id)
            })
            .map(|(rank, truth_id)| (rank + 1, *truth_id))
            .collect::<Vec<_>>();
        truth_neighbors += truth.truth.ids[query_ordinal].len().min(input.top_k);
        let live_targets = missed
            .iter()
            .filter_map(|(_, logical_id)| logical_to_vec.get(logical_id).copied())
            .filter(|vec_id| graph.index.contains_key(vec_id))
            .collect::<HashSet<_>>();
        let production_reachable = graph.reachable_targets(&control.seeds, &live_targets);
        let owner = trace_for(&variants, OWNER, *query_id)?;
        let owner_reachable = graph.reachable_targets(&owner.seeds, &live_targets);
        let bw8 = trace_for(&variants, BW8, *query_id)?;
        let exact = trace_for(&variants, EXACT, *query_id)?;
        let owner_exact = trace_for(&variants, OWNER_EXACT, *query_id)?;
        let truth_ids = truth.truth.ids[query_ordinal]
            .iter()
            .take(input.top_k)
            .copied()
            .collect::<HashSet<_>>();
        let control_hits = control_predictions.predictions[query_ordinal]
            .iter()
            .take(input.top_k)
            .filter(|id| truth_ids.contains(id))
            .collect::<HashSet<_>>()
            .len();
        let candidate_hits = predictions
            .get(BW8)
            .expect("required BW8 predictions")
            .predictions[query_ordinal]
            .iter()
            .take(input.top_k)
            .filter(|id| truth_ids.contains(id))
            .collect::<HashSet<_>>()
            .len();
        query_features.push(QueryFeatureRow {
            schema: "ec_distann_residual_query_feature_v1",
            query_id: *query_id,
            query_ordinal: query_ordinal + 1,
            query_offset: input.query_offset,
            query_slice_sha256: input.query_slice_sha256.to_owned(),
            generation_identity: generation_identity.clone(),
            control_variant: CONTROL,
            candidate_variant: BW8,
            control_truth_hits: control_hits,
            candidate_truth_hits: candidate_hits,
            paired_recall_delta: (candidate_hits as f64 - control_hits as f64) / input.top_k as f64,
            candidate_improves: candidate_hits > control_hits,
            candidate_loses: candidate_hits < control_hits,
            round_cap_reached: control.round_cap_reached,
            heap_saturated: control.heap_saturated,
            score_gap: control.score_gap,
            frontier_churn_rate: control.frontier_churn_rate,
            repeated_node_rate: control.repeated_node_rate,
            remote_owner_requests: control.remote_owner_requests,
            response_bytes: control.response_bytes,
            trace_truncated: control.truncated,
        });

        for (truth_rank, truth_id) in missed {
            let vec_id = logical_to_vec.get(&truth_id).copied();
            let generation_live = vec_id.is_some_and(|vec_id| graph.index.contains_key(&vec_id));
            let prod_reachable = vec_id.map(|vec_id| production_reachable.contains(&vec_id));
            let oracle_reachable = vec_id.map(|vec_id| owner_reachable.contains(&vec_id));
            let truncated = control.truncated
                || bw8.truncated
                || exact.truncated
                || owner.truncated
                || owner_exact.truncated;
            let classification = classify_truth_neighbor(&ClassificationEvidence {
                generation_mapped: vec_id.is_some(),
                generation_live,
                trace_truncated: truncated,
                production_seed_reachable: prod_reachable.unwrap_or(false),
                owner_seed_reachable: oracle_reachable.unwrap_or(false),
                production_requested: contains(control, vec_id, |trace| &trace.requested),
                production_returned: contains(control, vec_id, |trace| &trace.returned),
                production_expanded: contains(control, vec_id, |trace| &trace.expanded),
                production_retained: contains(control, vec_id, |trace| &trace.retained),
                production_exact_rerank: contains(control, vec_id, |trace| &trace.exact_rerank),
                same_seed_exact_retained: contains(exact, vec_id, |trace| &trace.exact_rerank),
            });
            *counts.entry(classification.to_owned()).or_default() += 1;
            rows.push(ResidualRow {
                schema: "ec_distann_residual_attribution_row_v1",
                query_id: *query_id,
                query_ordinal: query_ordinal + 1,
                query_offset: input.query_offset,
                query_slice_sha256: input.query_slice_sha256.to_owned(),
                generation_identity: generation_identity.clone(),
                variant: CONTROL,
                truth_rank,
                truth_id,
                truth_vec_id: vec_id.map(|id| format!("{id:016x}")),
                classification,
                generation_live,
                production_seed_reachable: prod_reachable,
                owner_seed_reachable: oracle_reachable,
                production_requested: contains(control, vec_id, |trace| &trace.requested),
                production_returned: contains(control, vec_id, |trace| &trace.returned),
                production_expanded: contains(control, vec_id, |trace| &trace.expanded),
                production_retained: contains(control, vec_id, |trace| &trace.retained),
                production_exact_rerank: contains(control, vec_id, |trace| &trace.exact_rerank),
                production_final: contains(control, vec_id, |trace| &trace.final_ids),
                bw8_retained: contains(bw8, vec_id, |trace| &trace.retained),
                bw8_final: contains(bw8, vec_id, |trace| &trace.final_ids),
                same_seed_exact_retained: contains(exact, vec_id, |trace| &trace.exact_rerank),
                same_seed_exact_final: contains(exact, vec_id, |trace| &trace.final_ids),
                owner_retained: contains(owner, vec_id, |trace| &trace.exact_rerank),
                owner_final: contains(owner, vec_id, |trace| &trace.final_ids),
                owner_exact_retained: contains(owner_exact, vec_id, |trace| &trace.exact_rerank),
                owner_exact_final: contains(owner_exact, vec_id, |trace| &trace.final_ids),
                trace_truncated: truncated,
            });
        }
    }

    let unknown = counts.get("unknown").copied().unwrap_or(0);
    let classified = counts.values().sum::<usize>();
    let reconciliation_pass = classified == rows.len() && unknown == 0;
    Ok(ResidualAttribution {
        summary: ResidualSummary {
            schema: "ec_distann_residual_attribution_summary_v1",
            query_prefix: format!(
                "rows_{}_{}",
                input.query_offset + 1,
                input.query_offset + control_predictions.query_ids.len() as u32
            ),
            query_offset: input.query_offset,
            queries: control_predictions.query_ids.len(),
            top_k: input.top_k,
            query_file_sha256: input.query_file_sha256.to_owned(),
            query_slice_sha256: input.query_slice_sha256.to_owned(),
            generation_identity,
            registered_variants: REQUIRED_VARIANTS.to_vec(),
            truth_neighbors,
            missed_truth_neighbors: rows.len(),
            classifications: counts,
            unknown_truth_neighbors: unknown,
            reconciliation_pass,
            task189_same_seed_approximate_ordering_trigger: rows
                .iter()
                .any(|row| row.classification == "approximate_ordering"),
        },
        rows,
        query_features,
    })
}

fn classify_truth_neighbor(evidence: &ClassificationEvidence) -> &'static str {
    if !evidence.generation_mapped || !evidence.generation_live {
        "generation_missing"
    } else if evidence.trace_truncated {
        "unknown"
    } else if !evidence.production_seed_reachable && evidence.owner_seed_reachable {
        "seed_reachability"
    } else if !evidence.production_seed_reachable && !evidence.owner_seed_reachable {
        "graph_unreachable"
    } else if !evidence.production_requested && !evidence.production_expanded {
        "budget_frontier"
    } else if (evidence.production_returned || evidence.production_expanded)
        && !evidence.production_retained
        && evidence.same_seed_exact_retained
    {
        "approximate_ordering"
    } else if evidence.production_retained && !evidence.production_exact_rerank {
        "rerank_containment"
    } else if evidence.production_exact_rerank {
        "exact_competition"
    } else {
        "unknown"
    }
}

struct LoadedPredictions {
    query_ids: Vec<i64>,
    predictions: Vec<Vec<i64>>,
}

fn load_predictions(path: &Path, top_k: usize) -> Result<LoadedPredictions> {
    let file: PredictionFile = serde_json::from_slice(
        &fs::read(path).wrap_err_with(|| format!("reading predictions {}", path.display()))?,
    )
    .wrap_err_with(|| format!("decoding predictions {}", path.display()))?;
    if file.version != 1 || file.k != top_k || file.rows.len() != 1 {
        bail!(
            "predictions {} have unexpected version/k/sweep count",
            path.display()
        );
    }
    let predictions = file.rows[0].predictions.clone();
    if predictions.len() != file.query_ids.len() {
        bail!("predictions {} have mismatched query rows", path.display());
    }
    Ok(LoadedPredictions {
        query_ids: file.query_ids,
        predictions,
    })
}

fn load_trace_file(
    path: &Path,
    query_offset: u32,
    query_file_sha256: &str,
    query_slice_sha256: &str,
) -> Result<VariantTraces> {
    let file: TraceFile = serde_json::from_slice(
        &fs::read(path).wrap_err_with(|| format!("reading query trace {}", path.display()))?,
    )
    .wrap_err_with(|| format!("decoding query trace {}", path.display()))?;
    if file.schema != "ec_distann_query_trace_file_v1"
        || file.query_offset != query_offset
        || file.query_file_sha256 != query_file_sha256
        || file.query_slice_sha256 != query_slice_sha256
        || file.queries != file.traces.len()
    {
        bail!(
            "query trace {} has mismatched slice provenance",
            path.display()
        );
    }
    let mut traces = BTreeMap::new();
    for envelope in file.traces {
        let raw = envelope.trace;
        let mut requested = HashSet::new();
        let mut returned = HashSet::new();
        let mut retained = HashSet::new();
        let mut prior_retained = None::<HashSet<u64>>;
        let mut churn_changed = 0_usize;
        let mut churn_union = 0_usize;
        let mut heap_saturated = false;
        let mut score_gap = None;
        let mut remote_owner_requests = 0_usize;
        let mut response_bytes = 0_usize;
        for round in raw.rounds {
            requested.extend(parse_ids(&round.requested_ids)?);
            returned.extend(parse_ids(&round.returned_ids)?);
            returned.extend(parse_ids(&round.exact_input_ids)?);
            let round_retained = parse_ids(&round.retained_ids)?
                .into_iter()
                .collect::<HashSet<_>>();
            if let Some(prior) = &prior_retained {
                churn_changed += prior.symmetric_difference(&round_retained).count();
                churn_union += prior.union(&round_retained).count();
            }
            retained.extend(round_retained.iter().copied());
            prior_retained = Some(round_retained);
            heap_saturated = round.heap_saturated;
            score_gap = round.frontier_score_gap.filter(|value| value.is_finite());
            if round.owner_ordinals.len() != round.owner_request_counts.len() {
                bail!(
                    "query trace {} has mismatched owner ordinal/request counts",
                    path.display()
                );
            }
            remote_owner_requests += round
                .owner_ordinals
                .iter()
                .zip(&round.owner_request_counts)
                .filter(|(owner, _)| **owner != 0)
                .map(|(_, count)| *count)
                .sum::<usize>();
            response_bytes += round.response_bytes;
        }
        let final_order = parse_ids(&raw.final_ids)?;
        let expanded_total = raw.expanded_unique.saturating_add(raw.expanded_overlap);
        let trace = NormalizedTrace {
            generation_identity: raw.epoch_fingerprint,
            truncated: raw.truncated,
            seeds: parse_ids(&raw.seed_ids)?,
            requested,
            returned,
            expanded: parse_ids(&raw.expanded_live_ids)?.into_iter().collect(),
            retained,
            exact_rerank: parse_ids(&raw.exact_rerank_ids)?.into_iter().collect(),
            final_ids: final_order.iter().copied().collect(),
            final_order,
            round_cap_reached: raw.rounds_executed >= 100,
            heap_saturated,
            score_gap,
            frontier_churn_rate: if churn_union == 0 {
                0.0
            } else {
                churn_changed as f64 / churn_union as f64
            },
            repeated_node_rate: if expanded_total == 0 {
                0.0
            } else {
                raw.expanded_overlap as f64 / expanded_total as f64
            },
            remote_owner_requests,
            response_bytes,
        };
        if traces.insert(envelope.query_id, trace).is_some() {
            bail!(
                "query trace {} repeats query {}",
                path.display(),
                envelope.query_id
            );
        }
    }
    Ok(VariantTraces { traces })
}

fn parse_ids(values: &[String]) -> Result<Vec<u64>> {
    values
        .iter()
        .map(|value| {
            if value.len() != 16 {
                bail!("stable trace id {value:?} is not 16 hexadecimal characters");
            }
            u64::from_str_radix(value, 16)
                .wrap_err_with(|| format!("decoding stable trace id {value:?}"))
        })
        .collect()
}

fn trace_for<'a>(
    variants: &'a BTreeMap<&str, VariantTraces>,
    variant: &str,
    query_id: i64,
) -> Result<&'a NormalizedTrace> {
    variants
        .get(variant)
        .and_then(|traces| traces.traces.get(&query_id))
        .ok_or_else(|| eyre!("missing trace for variant {variant} query {query_id}"))
}

fn validate_prediction_trace(
    variant: &str,
    query_id: i64,
    predictions: &[i64],
    trace: &NormalizedTrace,
    logical_to_vec: &HashMap<i64, u64>,
) -> Result<()> {
    let predicted = predictions
        .iter()
        .map(|logical_id| {
            logical_to_vec.get(logical_id).copied().ok_or_else(|| {
                eyre!("variant {variant} query {query_id} prediction {logical_id} has no vec_id")
            })
        })
        .collect::<Result<Vec<_>>>()?;
    if predicted != trace.final_order {
        bail!("variant {variant} query {query_id} predictions differ from trace final ids");
    }
    Ok(())
}

fn contains(
    trace: &NormalizedTrace,
    vec_id: Option<u64>,
    field: impl FnOnce(&NormalizedTrace) -> &HashSet<u64>,
) -> bool {
    vec_id.is_some_and(|vec_id| field(trace).contains(&vec_id))
}

impl GraphIndex {
    fn new(nodes: &[GraphNode]) -> Self {
        let mut ids = nodes
            .iter()
            .filter(|node| !node.tombstone)
            .map(|node| node.vec_id)
            .collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        let index = ids
            .iter()
            .enumerate()
            .map(|(position, vec_id)| (*vec_id, position))
            .collect::<HashMap<_, _>>();
        let by_id = nodes
            .iter()
            .filter(|node| !node.tombstone)
            .map(|node| (node.vec_id, node))
            .collect::<HashMap<_, _>>();
        let adjacency = ids
            .iter()
            .map(|vec_id| {
                by_id
                    .get(vec_id)
                    .into_iter()
                    .flat_map(|node| node.neighbors.iter())
                    .filter_map(|neighbor| index.get(neighbor).copied())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect()
            })
            .collect();
        Self {
            ids,
            index,
            adjacency,
        }
    }

    fn reachable_targets(&self, seeds: &[u64], targets: &HashSet<u64>) -> HashSet<u64> {
        if targets.is_empty() {
            return HashSet::new();
        }
        let target_indices = targets
            .iter()
            .filter_map(|vec_id| self.index.get(vec_id).map(|index| (*index, *vec_id)))
            .collect::<HashMap<_, _>>();
        let mut found = HashSet::new();
        let mut visited = vec![false; self.ids.len()];
        let mut queue = VecDeque::new();
        for seed in seeds {
            if let Some(index) = self.index.get(seed) {
                if !visited[*index] {
                    visited[*index] = true;
                    queue.push_back(*index);
                }
            }
        }
        while let Some(node) = queue.pop_front() {
            if let Some(vec_id) = target_indices.get(&node) {
                found.insert(*vec_id);
                if found.len() == target_indices.len() {
                    break;
                }
            }
            for neighbor in &self.adjacency[node] {
                if !visited[*neighbor] {
                    visited[*neighbor] = true;
                    queue.push_back(*neighbor);
                }
            }
        }
        found
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, HashMap, HashSet},
        fs,
    };

    use serde_json::json;
    use tempfile::tempdir;

    use crate::commands::bench::recall::{PredictionFile, PredictionSweep};

    use super::{
        classify_residuals, classify_truth_neighbor, ClassificationEvidence, GraphIndex, GraphNode,
        ResidualAttributionInputs, REQUIRED_VARIANTS,
    };

    fn node(id: u64, neighbors: &[u64]) -> GraphNode {
        GraphNode {
            owner_ordinal: 0,
            vec_id: id,
            tombstone: false,
            neighbors: neighbors.to_vec(),
        }
    }

    #[test]
    fn directed_reachability_stops_on_registered_targets() {
        let graph = GraphIndex::new(&[node(1, &[2]), node(2, &[3]), node(3, &[]), node(4, &[1])]);
        let targets = [3, 4].into_iter().collect::<HashSet<_>>();
        assert_eq!(
            graph.reachable_targets(&[1], &targets),
            [3].into_iter().collect()
        );
        assert_eq!(graph.reachable_targets(&[4], &targets), targets);
    }

    #[test]
    fn classification_priority_is_total_and_unknown_is_explicit() {
        let live = ClassificationEvidence {
            generation_mapped: true,
            generation_live: true,
            production_seed_reachable: true,
            owner_seed_reachable: true,
            ..ClassificationEvidence::default()
        };
        assert_eq!(classify_truth_neighbor(&live), "budget_frontier");
        assert_eq!(
            classify_truth_neighbor(&ClassificationEvidence {
                production_seed_reachable: false,
                owner_seed_reachable: false,
                ..live
            }),
            "graph_unreachable"
        );
        assert_eq!(
            classify_truth_neighbor(&ClassificationEvidence {
                generation_mapped: true,
                generation_live: true,
                owner_seed_reachable: true,
                ..ClassificationEvidence::default()
            }),
            "seed_reachability"
        );
        assert_eq!(
            classify_truth_neighbor(&ClassificationEvidence {
                production_requested: true,
                production_returned: true,
                same_seed_exact_retained: true,
                ..live
            }),
            "approximate_ordering"
        );
        assert_eq!(
            classify_truth_neighbor(&ClassificationEvidence {
                production_requested: true,
                production_retained: true,
                ..live
            }),
            "rerank_containment"
        );
        assert_eq!(
            classify_truth_neighbor(&ClassificationEvidence {
                production_requested: true,
                production_retained: true,
                production_exact_rerank: true,
                ..live
            }),
            "exact_competition"
        );
        assert_eq!(
            classify_truth_neighbor(&ClassificationEvidence {
                production_requested: true,
                ..live
            }),
            "unknown"
        );
        assert_eq!(
            classify_truth_neighbor(&ClassificationEvidence {
                trace_truncated: true,
                production_exact_rerank: true,
                ..live
            }),
            "unknown",
            "truncated evidence must never infer a later category"
        );
        assert_eq!(
            classify_truth_neighbor(&ClassificationEvidence {
                generation_mapped: false,
                trace_truncated: true,
                ..ClassificationEvidence::default()
            }),
            "generation_missing",
            "generation identity has first priority even when traces truncate"
        );
    }

    #[test]
    fn file_join_reconciles_budget_miss_and_fails_closed_on_truncation() {
        for (truncated, expected, pass) in
            [(false, "budget_frontier", true), (true, "unknown", false)]
        {
            let dir = tempdir().expect("temporary attribution fixture");
            let truth_path = dir.path().join("truth.json");
            fs::write(
                &truth_path,
                serde_json::to_vec(&json!({
                    "truth": {"ids": [[1, 2]], "scores": [[1.0, 0.9]]}
                }))
                .unwrap(),
            )
            .unwrap();
            let mut prediction_paths = BTreeMap::new();
            let mut trace_paths = BTreeMap::new();
            for variant in REQUIRED_VARIANTS {
                let prediction_path = dir.path().join(format!("{variant}-predictions.json"));
                fs::write(
                    &prediction_path,
                    serde_json::to_vec(&PredictionFile {
                        version: 1,
                        prefix: "fixture".to_owned(),
                        profile: "ec_distann".to_owned(),
                        k: 1,
                        query_ids: vec![201],
                        rows: vec![PredictionSweep {
                            sweep_axis: "rerank_width".to_owned(),
                            sweep_value: 32,
                            rerank_width: None,
                            predictions: vec![vec![1]],
                        }],
                    })
                    .unwrap(),
                )
                .unwrap();
                prediction_paths.insert(variant.to_owned(), prediction_path);

                let trace_path = dir.path().join(format!("{variant}-trace.json"));
                fs::write(
                    &trace_path,
                    serde_json::to_vec(&json!({
                        "schema": "ec_distann_query_trace_file_v1",
                        "query_offset": 200,
                        "queries": 1,
                        "query_file_sha256": "parent",
                        "query_slice_sha256": "slice",
                        "traces": [{
                            "query_id": 201,
                            "trace": {
                                "epoch_fingerprint": "generation",
                                "truncated": truncated,
                                "seed_ids": ["0000000000000065"],
                                "expanded_live_ids": ["0000000000000065"],
                                "rounds": [{
                                    "requested_ids": ["0000000000000065"],
                                    "returned_ids": ["0000000000000065"],
                                    "exact_input_ids": ["0000000000000065"],
                                    "retained_ids": ["0000000000000065"]
                                }],
                                "exact_rerank_ids": ["0000000000000065"],
                                "final_ids": ["0000000000000065"]
                            }
                        }]
                    }))
                    .unwrap(),
                )
                .unwrap();
                trace_paths.insert(variant.to_owned(), trace_path);
            }
            let nodes = [node(101, &[102]), node(102, &[])];
            let logical_id_by_vec_id = HashMap::from([(101_i64, 1_i64), (102_i64, 2_i64)]);
            let result = classify_residuals(ResidualAttributionInputs {
                query_offset: 200,
                query_file_sha256: "parent",
                query_slice_sha256: "slice",
                top_k: 1,
                truth_cache_path: &truth_path,
                prediction_paths: &prediction_paths,
                query_trace_paths: &trace_paths,
                graph_nodes: &nodes,
                logical_id_by_vec_id: &logical_id_by_vec_id,
            })
            .expect("fixture attribution classifies");
            assert_eq!(result.rows.len(), 0, "k1 truth is fully returned");

            // Rewrite truth so logical id 2 is the sole truth neighbor while
            // the prediction/trace still return id 1.
            fs::write(
                &truth_path,
                serde_json::to_vec(&json!({
                    "truth": {"ids": [[2]], "scores": [[1.0]]}
                }))
                .unwrap(),
            )
            .unwrap();
            let result = classify_residuals(ResidualAttributionInputs {
                query_offset: 200,
                query_file_sha256: "parent",
                query_slice_sha256: "slice",
                top_k: 1,
                truth_cache_path: &truth_path,
                prediction_paths: &prediction_paths,
                query_trace_paths: &trace_paths,
                graph_nodes: &nodes,
                logical_id_by_vec_id: &logical_id_by_vec_id,
            })
            .expect("miss attribution classifies");
            assert_eq!(result.rows.len(), 1);
            assert_eq!(result.rows[0].classification, expected);
            assert_eq!(result.reconciliation_pass(), pass);
            assert_eq!(result.query_features.len(), 1);
            assert_eq!(result.query_features[0].control_truth_hits, 0);
            assert_eq!(result.query_features[0].candidate_truth_hits, 0);
            assert_eq!(result.query_features[0].trace_truncated, truncated);
        }
    }
}
