//! `ecaz bench spire-pipeline` — SPIRE routing and pipeline counters.
//!
//! The recall, latency, and storage commands own the scalar performance
//! measurements. This command owns the structural counters Phase 9/10 need:
//! routing budgets, local scan pipeline counts, and optional remote fanout
//! diagnostics from the SQL-visible operator surfaces.

use clap::{Args, ValueEnum};
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use indicatif::{ProgressBar, ProgressStyle};
use ndarray::Array2;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio_postgres::{Client, Row};

use crate::profiles::{self, EC_SPIRE};
use crate::psql::{self, ConnectionOptions};

const EC_SPIRE_MAX_NPROBE: i32 = 1_000_000;
const EC_SPIRE_MAX_RERANK_WIDTH: i32 = 10_000_000;
const EC_SPIRE_MAX_CANDIDATE_ROWS: i32 = 10_000_000;
const EC_SPIRE_MAX_COST_SCALE: f64 = 1_000_000.0;

#[derive(Args, Debug)]
pub struct SpirePipelineArgs {
    /// Prefix identifying the SPIRE corpus.
    #[arg(long)]
    pub prefix: String,
    /// SPIRE index name. Defaults to the only ec_spire index on `<prefix>_corpus`.
    #[arg(long)]
    pub index: Option<String>,
    /// Number of queries to sample from `<prefix>_queries`.
    #[arg(long, default_value_t = 1)]
    pub queries_limit: usize,
    /// Sweep values for `ec_spire.nprobe`. Accepts `--sweep 8,16,32`
    /// or repeated `--sweep 8 --sweep 16`.
    #[arg(long, value_delimiter = ',')]
    pub sweep: Vec<i32>,
    /// Session override for heap-f32 rerank frontier width.
    /// Use -1 for the index reloption, 0 for the full retained frontier.
    #[arg(long)]
    pub rerank_width: Option<i32>,
    /// Session override for hard candidate-row budget.
    /// Use -1 for the index reloption and 0 for the automatic ceiling.
    #[arg(long)]
    pub max_candidate_rows: Option<i32>,
    /// Session cap for routed leaf assignment rows. Use 0 to disable.
    #[arg(long)]
    pub max_routed_candidate_rows: Option<i32>,
    /// Enable deterministic adaptive nprobe while collecting counters.
    #[arg(long)]
    pub adaptive_nprobe: bool,
    /// Score-gap threshold for adaptive nprobe decisions.
    #[arg(long)]
    pub adaptive_nprobe_score_gap_micros: Option<i32>,
    /// Also call `ec_spire_remote_pipeline_steps`. When no selected PIDs are
    /// provided this records the empty-fanout remote diagnostic shape.
    #[arg(long)]
    pub include_remote: bool,
    /// Fail before reporting when the SPIRE placement directory has no remote
    /// placements. This is the AWS distributed smoke gate; it prevents
    /// local-only fixtures from being mistaken for distributed reads.
    #[arg(long)]
    pub require_remote_placements: bool,
    /// Also aggregate local-store read-overlap counters for sampled queries.
    #[arg(long)]
    pub include_local_store_overlap: bool,
    /// Remote partition/object PIDs to pass to `ec_spire_remote_pipeline_steps`.
    #[arg(long, value_delimiter = ',')]
    pub remote_selected_pids: Vec<i64>,
    /// Epoch to pass to `ec_spire_remote_pipeline_steps`.
    /// Defaults to the active epoch observed from the local pipeline snapshot.
    #[arg(long)]
    pub remote_requested_epoch: Option<i64>,
    /// top_k to pass to `ec_spire_remote_pipeline_steps`.
    #[arg(long, default_value_t = 10)]
    pub top_k: i32,
    /// Consistency mode to pass to `ec_spire_remote_pipeline_steps`.
    #[arg(long, default_value = "epoch")]
    pub consistency_mode: String,
    /// Session tuple-payload transport policy for remote CustomScan payloads.
    #[arg(long, value_enum)]
    pub remote_tuple_transport: Option<SpireRemoteTupleTransportMode>,
    /// Also print the SPIRE cost-model tuning snapshot for each sweep value.
    #[arg(long)]
    pub include_cost_snapshot: bool,
    /// Session override for ec_spire.cost_routing_dimension_scale.
    #[arg(long)]
    pub cost_routing_dimension_scale: Option<f64>,
    /// Session override for ec_spire.cost_leaf_dimension_scale.
    #[arg(long)]
    pub cost_leaf_dimension_scale: Option<f64>,
    /// Session override for ec_spire.cost_index_page_scale.
    #[arg(long)]
    pub cost_index_page_scale: Option<f64>,
    /// Session override for ec_spire.cost_local_store_page_fanout_scale.
    #[arg(long)]
    pub cost_local_store_page_fanout_scale: Option<f64>,
    /// Session override for ec_spire.cost_storage_scoring_multiplier.
    #[arg(long)]
    pub cost_storage_scoring_multiplier: Option<f64>,
    /// Session override for ec_spire.cost_rerank_multiplier.
    #[arg(long)]
    pub cost_rerank_multiplier: Option<f64>,
    /// Run coordinator KNN queries and report single-connection latency stats.
    #[arg(long)]
    pub include_query_metrics: bool,
    /// Also compute exact local truth and report recall@k for coordinator KNN queries.
    #[arg(long)]
    pub include_recall: bool,
    /// Optional local `<id>\t<json_array>` corpus TSV to use for exact truth.
    ///
    /// This avoids streaming the full corpus table through a remote SQL tunnel
    /// when the same staged corpus file is already available to the operator.
    #[arg(long)]
    pub truth_corpus_file: Option<PathBuf>,
    /// Optional exact top-k ground-truth cache file produced by `bench recall`.
    ///
    /// When present, the cache is validated against the sampled query set and
    /// k, then reused instead of fetching/scoring the full corpus.
    #[arg(long)]
    pub truth_cache_file: Option<PathBuf>,
    /// Write per-exact-neighbor SPIRE leaf-block rank rows as JSONL.
    ///
    /// Requires `--include-recall`. For local corpora without SPIRE
    /// source_identity, use the default local sequence mapping `id + 1`.
    #[arg(long)]
    pub leaf_block_rank_output: Option<PathBuf>,
    /// Write per-exact-neighbor target-containing leaf-block rank rows as JSONL.
    ///
    /// Unlike `--leaf-block-rank-output`, this locates each truth row in routed
    /// leaves first and emits only the containing block rank.
    #[arg(long)]
    pub target_block_rank_output: Option<PathBuf>,
    /// Write per-exact-neighbor recall miss attribution rows as JSONL.
    ///
    /// Requires `--include-recall` and query metrics. The output joins exact
    /// truth ids, returned ids, and SPIRE leaf-block rank diagnostics.
    #[arg(long)]
    pub miss_attribution_output: Option<PathBuf>,
    /// Offset added to exact-truth ids to form SPIRE local vec_id sequences.
    #[arg(long, default_value_t = 1)]
    pub leaf_block_rank_local_sequence_offset: i64,
    /// Also call ec_spire_remote_search_production_read_profile for each sampled
    /// query and report production read-path transport counters.
    #[arg(long)]
    pub include_production_read_profile: bool,
    /// Skip SQL-visible local/remote pipeline diagnostics and measure only the
    /// production KNN/read-profile path. Use this when remote placements make
    /// local heap diagnostics intentionally fail closed.
    #[arg(long)]
    pub production_read_only: bool,
    /// k for optional query latency and recall metrics.
    #[arg(long, default_value_t = 10)]
    pub query_metric_k: usize,
    /// Extra corpus columns to project while measuring coordinator KNN query
    /// latency. `id` is always selected first for recall accounting.
    #[arg(long, value_delimiter = ',')]
    pub query_metric_projection_columns: Vec<String>,
    /// Extra session GUCs to set while collecting SPIRE pipeline counters, as name=value.
    #[arg(long = "session-guc")]
    pub session_gucs: Vec<String>,
    /// Reset and snapshot Task 87 CandidateBatch scoring counters for each sweep value.
    #[arg(long)]
    pub task87_candidate_batch_counters: bool,
    /// Write the pipeline report to this path in addition to stdout.
    #[arg(long)]
    pub log_output: Option<PathBuf>,
    /// Write per-query candidate funnel rows as JSONL.
    #[arg(long)]
    pub funnel_output: Option<PathBuf>,
    /// Write per-query, per-stage exact-truth containment rows as JSONL.
    ///
    /// Requires `--include-recall` and query metrics. Candidate/rerank
    /// containment is reported as a lower bound until a target candidate-rank
    /// SQL snapshot is available.
    #[arg(long)]
    pub stage_containment_output: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum SpireRemoteTupleTransportMode {
    #[value(name = "auto")]
    Auto,
    #[value(name = "json_tuple_payload_v1")]
    JsonTuplePayloadV1,
    #[value(name = "pg_binary_attr_v1")]
    PgBinaryAttrV1,
}

impl SpireRemoteTupleTransportMode {
    fn as_guc_value(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::JsonTuplePayloadV1 => "json_tuple_payload_v1",
            Self::PgBinaryAttrV1 => "pg_binary_attr_v1",
        }
    }
}

impl fmt::Display for SpireRemoteTupleTransportMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_guc_value())
    }
}

pub async fn run(conn: &ConnectionOptions, args: SpirePipelineArgs) -> Result<()> {
    validate_args(&args)?;
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    if let Some(index) = &args.index {
        profiles::validate_ident(index).wrap_err_with(|| format!("invalid index {:?}", index))?;
    }

    let corpus_table = format!("{}_corpus", args.prefix);
    let queries_table = format!("{}_queries", args.prefix);
    let sweep_values = sweep_values(&args)?;
    let remote_enabled = args.include_remote
        || !args.remote_selected_pids.is_empty()
        || args.remote_requested_epoch.is_some();
    let adaptive_nprobe_options = super::AdaptiveNprobeBenchOptions {
        enabled: args.adaptive_nprobe,
        score_gap_micros: args.adaptive_nprobe_score_gap_micros,
        score_margin_ratio_bps: None,
    };
    let cost_tuning_options = SpireCostTuningOptions {
        routing_dimension_scale: args.cost_routing_dimension_scale,
        leaf_dimension_scale: args.cost_leaf_dimension_scale,
        index_page_scale: args.cost_index_page_scale,
        local_store_page_fanout_scale: args.cost_local_store_page_fanout_scale,
        storage_scoring_multiplier: args.cost_storage_scoring_multiplier,
        rerank_multiplier: args.cost_rerank_multiplier,
    };
    super::validate_adaptive_nprobe_options(&EC_SPIRE, adaptive_nprobe_options)?;
    let session_gucs = super::parse_session_gucs(&args.session_gucs)?;
    let query_metrics_enabled = args.include_query_metrics || args.include_recall;

    let client = psql::connect(conn).await?;
    if !psql::relation_exists(&client, &corpus_table, 'r').await? {
        return Err(eyre!("no corpus table {:?} in this database", corpus_table));
    }
    if !psql::relation_exists(&client, &queries_table, 'r').await? {
        return Err(eyre!(
            "no queries table {:?} in this database",
            queries_table
        ));
    }
    let index = resolve_spire_index(&client, &corpus_table, args.index.as_deref()).await?;
    if args.require_remote_placements {
        let placement_gate = query_remote_placement_gate(&client, &index).await?;
        enforce_remote_placement_gate(&index, &placement_gate)?;
    }
    let endpoint_identity = query_endpoint_identity(&client, &index).await?;
    let queries = fetch_queries(&client, &queries_table, args.queries_limit).await?;
    if queries.is_empty() {
        return Err(eyre!("queries table {queries_table:?} is empty"));
    }
    let query_truth = if args.include_recall {
        if let Some(path) = args.truth_cache_file.as_deref() {
            let query_ids: Vec<i64> = queries.iter().map(|query| query.id).collect();
            let query_matrix = query_matrix(&queries)?;
            let truth = super::recall::load_truth_cache_file_if_valid(
                path,
                &query_ids,
                &query_matrix,
                args.query_metric_k,
            )
            .await?
            .ok_or_else(|| eyre!("truth cache file {} does not exist", path.display()))?;
            Some(truth.ids)
        } else {
            let (corpus_ids, corpus) = if let Some(path) = args.truth_corpus_file.as_deref() {
                super::recall::load_sources_tsv_file(path).wrap_err_with(|| {
                    format!("loading exact-truth corpus from {}", path.display())
                })?
            } else {
                super::recall::fetch_sources_public(&client, &corpus_table, None)
                    .await
                    .wrap_err_with(|| format!("fetching exact-truth corpus from {corpus_table}"))?
            };
            let query_matrix = query_matrix(&queries)?;
            if corpus.nrows() == 0 {
                return Err(eyre!("exact-truth corpus is empty"));
            }
            if corpus.ncols() != query_matrix.ncols() {
                return Err(eyre!(
                    "exact-truth corpus dim {} does not match query dim {}",
                    corpus.ncols(),
                    query_matrix.ncols()
                ));
            }
            let truth =
                super::recall::brute_force_top_k(&corpus, &query_matrix, args.query_metric_k);
            Some(super::recall::map_indices_to_ids(
                &truth.indices,
                &corpus_ids,
            ))
        }
    } else {
        None
    };
    if query_metrics_enabled {
        psql::prefer_ordered_ann_path(&client).await?;
        if args.production_read_only {
            client
                .batch_execute("SET enable_indexscan = off")
                .await
                .wrap_err("forcing SPIRE CustomScan tuple delivery path")?;
        }
    }
    let query_metric_stmt = if query_metrics_enabled {
        let sql = build_query_metric_sql(&corpus_table, &args.query_metric_projection_columns);
        Some(
            client
                .prepare(&sql)
                .await
                .wrap_err("preparing SPIRE pipeline query-metrics KNN statement")?,
        )
    } else {
        None
    };

    let mut routing = BTreeMap::<RoutingKey, RoutingAggregate>::new();
    let mut local = BTreeMap::<StepKey, LocalStepAggregate>::new();
    let mut remote = BTreeMap::<StepKey, RemoteStepAggregate>::new();
    let mut local_store_overlap =
        BTreeMap::<LocalStoreOverlapKey, LocalStoreOverlapAggregate>::new();
    let mut degraded_skip = BTreeMap::<DegradedSkipKey, DegradedSkipAggregate>::new();
    let mut query_metrics = BTreeMap::<i32, QueryMetricAggregate>::new();
    let mut production_read_profile = BTreeMap::<i32, ProductionReadProfileAggregate>::new();
    let mut cost_tuning = BTreeMap::<i32, CostTuningRow>::new();
    let mut funnel_rows = Vec::<FunnelRecord>::new();
    let mut stage_containment_rows = Vec::<StageContainmentRecord>::new();
    let mut leaf_block_rank_rows = Vec::<LeafBlockRankRecord>::new();
    let mut target_block_rank_rows = Vec::<LeafBlockRankRecord>::new();
    let mut miss_attribution_rows = Vec::<MissAttributionRecord>::new();
    let mut remote_epoch = args.remote_requested_epoch;
    let mut task87_counter_lines = Vec::new();

    for nprobe in &sweep_values {
        apply_session_options(
            &client,
            *nprobe,
            args.rerank_width,
            args.max_candidate_rows,
            args.max_routed_candidate_rows,
            args.remote_tuple_transport,
            adaptive_nprobe_options,
            cost_tuning_options,
            &session_gucs,
        )
        .await?;
        if args.task87_candidate_batch_counters {
            super::reset_block_kernel_counters(&client).await?;
        }

        if args.include_cost_snapshot {
            cost_tuning.insert(*nprobe, query_cost_tuning_row(&client, &index).await?);
        }

        let bar = ProgressBar::new(queries.len() as u64);
        bar.set_style(
            ProgressStyle::with_template(
                "[spire-pipeline {msg}] {wide_bar} {pos}/{len} ({per_sec})",
            )
            .unwrap(),
        );
        bar.set_message(spire_pipeline_progress_label(*nprobe, &args));
        bar.enable_steady_tick(Duration::from_millis(250));

        let target_assignments_by_ordinal =
            if args.miss_attribution_output.is_some() && args.leaf_block_rank_output.is_none() {
                let truth_ids = query_truth
                    .as_ref()
                    .expect("miss attribution output is validated to require recall truth");
                let flattened_target_local_sequences = truth_ids
                    .iter()
                    .flat_map(|ids| ids.iter())
                    .map(|truth_id| {
                        truth_id
                            .checked_add(args.leaf_block_rank_local_sequence_offset)
                            .ok_or_else(|| {
                                eyre!(
                                "target assignment local sequence overflow for truth id {truth_id}"
                            )
                            })
                    })
                    .collect::<Result<Vec<_>>>()?;
                Some(
                    query_leaf_target_assignment_rows(
                        &client,
                        &index,
                        &flattened_target_local_sequences,
                    )
                    .await?
                    .into_iter()
                    .fold(
                        HashMap::<i64, Vec<LeafTargetAssignmentRow>>::new(),
                        |mut acc, row| {
                            acc.entry(row.target_ordinal).or_default().push(row);
                            acc
                        },
                    ),
                )
            } else {
                None
            };

        for (query_index, query) in queries.iter().enumerate() {
            let mut funnel_local_rows = Vec::new();
            let mut funnel_leaf_rows = Vec::new();
            let mut funnel_rerank_locality = None;
            let mut returned_to_k_count = None;
            let mut predicted_ids_for_query = None;
            if !args.production_read_only {
                let routing_rows = query_routing_rows(&client, &index, &query.source).await?;
                for row in routing_rows {
                    routing
                        .entry(RoutingKey {
                            nprobe: *nprobe,
                            routing_level: row.routing_level,
                        })
                        .or_default()
                        .record(row);
                }

                let local_rows = query_local_pipeline_rows(&client, &index, &query.source).await?;
                if remote_epoch.is_none() {
                    remote_epoch = local_rows
                        .iter()
                        .find(|row| row.active_epoch > 0)
                        .map(|row| row.active_epoch);
                }
                if args.funnel_output.is_some() || args.stage_containment_output.is_some() {
                    funnel_local_rows = local_rows.clone();
                    funnel_leaf_rows =
                        query_leaf_candidate_rows(&client, &index, &query.source).await?;
                    funnel_rerank_locality =
                        query_rerank_locality_row(&client, &index, &query.source).await?;
                }
                for row in local_rows {
                    local
                        .entry(StepKey {
                            nprobe: *nprobe,
                            step_ordinal: row.step_ordinal,
                            step_name: row.step_name.clone(),
                        })
                        .or_default()
                        .record(row);
                }
                if args.include_local_store_overlap {
                    let overlap_rows =
                        query_local_store_overlap_rows(&client, &index, &query.source).await?;
                    for row in overlap_rows {
                        local_store_overlap
                            .entry(LocalStoreOverlapKey {
                                nprobe: *nprobe,
                                node_id: row.node_id,
                                local_store_id: row.local_store_id,
                            })
                            .or_default()
                            .record(row);
                    }
                }

                if remote_enabled {
                    let requested_epoch = remote_epoch.ok_or_else(|| {
                        eyre!(
                            "remote pipeline requested but no active epoch was observed; pass --remote-requested-epoch"
                        )
                    })?;
                    let remote_rows = query_remote_pipeline_rows(
                        &client,
                        &index,
                        requested_epoch,
                        &query.source,
                        &args.remote_selected_pids,
                        args.top_k,
                        &args.consistency_mode,
                    )
                    .await?;
                    for row in remote_rows {
                        remote
                            .entry(StepKey {
                                nprobe: *nprobe,
                                step_ordinal: row.step_ordinal,
                                step_name: row.step_name.clone(),
                            })
                            .or_default()
                            .record(row);
                    }
                    let degraded_skip_rows = query_degraded_skip_rows(
                        &client,
                        &index,
                        requested_epoch,
                        &query.source,
                        &args.remote_selected_pids,
                        args.top_k,
                        &args.consistency_mode,
                    )
                    .await?;
                    for row in degraded_skip_rows {
                        degraded_skip
                            .entry(DegradedSkipKey {
                                nprobe: *nprobe,
                                node_id: row.node_id,
                            })
                            .or_default()
                            .record(row);
                    }
                }
            }

            if let Some(stmt) = &query_metric_stmt {
                let measured =
                    execute_query_metric(&client, stmt, &query.source, args.query_metric_k).await?;
                returned_to_k_count = Some(measured.predicted_ids.len());
                predicted_ids_for_query = Some(measured.predicted_ids.clone());
                query_metrics
                    .entry(*nprobe)
                    .or_default()
                    .record(measured.duration, measured.predicted_ids);
            }
            if args.leaf_block_rank_output.is_some() {
                let truth_ids = query_truth
                    .as_ref()
                    .expect("rank/miss attribution output is validated to require recall truth");
                let query_truth_ids = truth_ids.get(query_index).ok_or_else(|| {
                    eyre!(
                        "exact-truth ids missing for query ordinal {}",
                        query_index + 1
                    )
                })?;
                let target_local_sequences = query_truth_ids
                    .iter()
                    .map(|truth_id| {
                        truth_id
                            .checked_add(args.leaf_block_rank_local_sequence_offset)
                            .ok_or_else(|| {
                                eyre!(
                                    "leaf block rank local sequence overflow for truth id {truth_id}"
                                )
                            })
                    })
                    .collect::<Result<Vec<_>>>()?;
                let rank_rows = query_leaf_block_rank_rows(
                    &client,
                    &index,
                    &query.source,
                    &target_local_sequences,
                )
                .await?;
                if args.miss_attribution_output.is_some() {
                    let predicted_ids = predicted_ids_for_query.as_ref().ok_or_else(|| {
                        eyre!(
                            "--miss-attribution-output requires query metrics for query ordinal {}",
                            query_index + 1
                        )
                    })?;
                    miss_attribution_rows.extend(MissAttributionRecord::from_query(
                        *nprobe,
                        query_index + 1,
                        query.id,
                        query_truth_ids,
                        &target_local_sequences,
                        predicted_ids,
                        &rank_rows,
                    )?);
                }
                for row in rank_rows {
                    let truth_index = usize::try_from(row.target_ordinal).map_err(|_| {
                        eyre!(
                            "leaf block rank target ordinal {} exceeds usize",
                            row.target_ordinal
                        )
                    })?;
                    let truth_id = *query_truth_ids.get(truth_index).ok_or_else(|| {
                        eyre!(
                            "leaf block rank target ordinal {} exceeds truth id count {}",
                            row.target_ordinal,
                            query_truth_ids.len()
                        )
                    })?;
                    leaf_block_rank_rows.push(LeafBlockRankRecord::from_row(
                        *nprobe,
                        query_index + 1,
                        query.id,
                        truth_id,
                        row,
                    ));
                }
            } else if args.miss_attribution_output.is_some() {
                let truth_ids = query_truth
                    .as_ref()
                    .expect("miss attribution output is validated to require recall truth");
                let query_truth_ids = truth_ids.get(query_index).ok_or_else(|| {
                    eyre!(
                        "exact-truth ids missing for query ordinal {}",
                        query_index + 1
                    )
                })?;
                let predicted_ids = predicted_ids_for_query.as_ref().ok_or_else(|| {
                    eyre!(
                        "--miss-attribution-output requires query metrics for query ordinal {}",
                        query_index + 1
                    )
                })?;
                let has_miss = query_truth_ids
                    .iter()
                    .any(|truth_id| !predicted_ids.contains(truth_id));
                let leaf_candidate_rows = if has_miss {
                    query_leaf_candidate_rows(&client, &index, &query.source).await?
                } else {
                    Vec::new()
                };
                let target_assignments_by_ordinal =
                    target_assignments_by_ordinal.as_ref().ok_or_else(|| {
                        eyre!("target assignment snapshot missing for miss attribution")
                    })?;
                miss_attribution_rows.extend(MissAttributionRecord::from_target_assignments(
                    *nprobe,
                    query_index + 1,
                    query.id,
                    query_truth_ids,
                    args.leaf_block_rank_local_sequence_offset,
                    predicted_ids,
                    &leaf_candidate_rows,
                    target_assignments_by_ordinal,
                )?);
            }
            if args.target_block_rank_output.is_some() {
                let truth_ids = query_truth
                    .as_ref()
                    .expect("target block rank output is validated to require recall truth");
                let query_truth_ids = truth_ids.get(query_index).ok_or_else(|| {
                    eyre!(
                        "exact-truth ids missing for query ordinal {}",
                        query_index + 1
                    )
                })?;
                let target_local_sequences = query_truth_ids
                    .iter()
                    .map(|truth_id| {
                        truth_id
                            .checked_add(args.leaf_block_rank_local_sequence_offset)
                            .ok_or_else(|| {
                                eyre!(
                                    "target block rank local sequence overflow for truth id {truth_id}"
                                )
                            })
                    })
                    .collect::<Result<Vec<_>>>()?;
                let rank_rows = query_leaf_target_block_rank_rows(
                    &client,
                    &index,
                    &query.source,
                    &target_local_sequences,
                )
                .await?;
                for row in rank_rows {
                    let truth_index = usize::try_from(row.target_ordinal).map_err(|_| {
                        eyre!(
                            "target block rank target ordinal {} exceeds usize",
                            row.target_ordinal
                        )
                    })?;
                    let truth_id = *query_truth_ids.get(truth_index).ok_or_else(|| {
                        eyre!(
                            "target block rank target ordinal {} exceeds truth id count {}",
                            row.target_ordinal,
                            query_truth_ids.len()
                        )
                    })?;
                    target_block_rank_rows.push(LeafBlockRankRecord::from_row_with_kind(
                        "spire_leaf_target_block_rank",
                        *nprobe,
                        query_index + 1,
                        query.id,
                        truth_id,
                        row,
                    ));
                }
            }
            if args.stage_containment_output.is_some() {
                let truth_ids = query_truth
                    .as_ref()
                    .expect("stage containment output is validated to require recall truth");
                let query_truth_ids = truth_ids.get(query_index).ok_or_else(|| {
                    eyre!(
                        "exact-truth ids missing for query ordinal {}",
                        query_index + 1
                    )
                })?;
                let predicted_ids = predicted_ids_for_query.as_ref().ok_or_else(|| {
                    eyre!(
                        "--stage-containment-output requires query metrics for query ordinal {}",
                        query_index + 1
                    )
                })?;
                let target_local_sequences = query_truth_ids
                    .iter()
                    .map(|truth_id| {
                        truth_id
                            .checked_add(args.leaf_block_rank_local_sequence_offset)
                            .ok_or_else(|| {
                                eyre!(
                                    "stage containment local sequence overflow for truth id {truth_id}"
                                )
                            })
                    })
                    .collect::<Result<Vec<_>>>()?;
                let rank_rows = query_leaf_target_block_rank_rows(
                    &client,
                    &index,
                    &query.source,
                    &target_local_sequences,
                )
                .await?;
                stage_containment_rows.extend(StageContainmentRecord::from_query(
                    *nprobe,
                    query_index + 1,
                    query.id,
                    query_truth_ids,
                    predicted_ids,
                    &funnel_local_rows,
                    &funnel_leaf_rows,
                    funnel_rerank_locality.as_ref(),
                    &rank_rows,
                )?);
            }
            if args.include_production_read_profile {
                let row = query_production_read_profile_row(
                    &client,
                    &index,
                    &query.source,
                    args.query_metric_k,
                )
                .await?;
                if returned_to_k_count.is_none() {
                    returned_to_k_count =
                        usize::try_from(row.i64_metric("returned_candidate_count")).ok();
                }
                production_read_profile
                    .entry(*nprobe)
                    .or_default()
                    .record(row);
            }
            if args.funnel_output.is_some() && !args.production_read_only {
                funnel_rows.push(FunnelRecord::from_query(
                    *nprobe,
                    query_index + 1,
                    query.id,
                    &funnel_local_rows,
                    &funnel_leaf_rows,
                    funnel_rerank_locality.as_ref(),
                    returned_to_k_count,
                )?);
            }
            bar.inc(1);
        }
        bar.finish_and_clear();
        if args.task87_candidate_batch_counters {
            let snapshots = super::snapshot_block_kernel_counters(&client).await?;
            task87_counter_lines.push(super::format_block_kernel_counter_lines(
                "spire-pipeline",
                &format!("nprobe={nprobe}"),
                &snapshots,
            ));
        }
    }
    if let Some(truth_ids) = query_truth.as_ref() {
        for aggregate in query_metrics.values_mut() {
            aggregate.record_recall(truth_ids, args.query_metric_k);
        }
    }

    let mut output = render_report(ReportInput {
        prefix: &args.prefix,
        index: &index,
        queries: queries.len(),
        sweep_values: &sweep_values,
        rerank_width: args.rerank_width,
        max_candidate_rows: args.max_candidate_rows,
        max_routed_candidate_rows: args.max_routed_candidate_rows,
        remote_tuple_transport: args.remote_tuple_transport,
        endpoint_identity: &endpoint_identity,
        adaptive_nprobe_options,
        cost_snapshot_enabled: args.include_cost_snapshot,
        cost_tuning: &cost_tuning,
        remote_enabled,
        remote_selected_pids: &args.remote_selected_pids,
        remote_epoch,
        query_metrics_enabled,
        include_recall: args.include_recall,
        query_metric_k: args.query_metric_k,
        query_metric_projection_columns: &args.query_metric_projection_columns,
        production_read_profile_enabled: args.include_production_read_profile,
        production_read_only: args.production_read_only,
        local_store_overlap_enabled: args.include_local_store_overlap,
        routing: &routing,
        local: &local,
        remote: &remote,
        local_store_overlap: &local_store_overlap,
        degraded_skip: &degraded_skip,
        query_metrics: &query_metrics,
        production_read_profile: &production_read_profile,
    });
    if !task87_counter_lines.is_empty() {
        output.push('\n');
        output.push_str(&task87_counter_lines.join("\n"));
    }
    println!("{output}");
    if let Some(path) = args.log_output {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .wrap_err_with(|| format!("creating {}", parent.display()))?;
        }
        tokio::fs::write(&path, format!("{output}\n"))
            .await
            .wrap_err_with(|| format!("writing {}", path.display()))?;
    }
    if let Some(path) = args.funnel_output {
        write_funnel_jsonl(&path, &funnel_rows).await?;
    }
    if let Some(path) = args.stage_containment_output {
        write_stage_containment_jsonl(&path, &stage_containment_rows).await?;
    }
    if let Some(path) = args.leaf_block_rank_output {
        write_leaf_block_rank_jsonl(&path, &leaf_block_rank_rows).await?;
    }
    if let Some(path) = args.target_block_rank_output {
        write_leaf_block_rank_jsonl(&path, &target_block_rank_rows).await?;
    }
    if let Some(path) = args.miss_attribution_output {
        write_miss_attribution_jsonl(&path, &miss_attribution_rows).await?;
    }
    Ok(())
}

fn spire_pipeline_progress_label(nprobe: i32, args: &SpirePipelineArgs) -> String {
    let mut parts = vec![format!("nprobe={nprobe}")];
    if args.include_recall {
        parts.push("recall".to_string());
    }
    if args.include_query_metrics {
        parts.push("query-metrics".to_string());
    }
    if args.include_production_read_profile {
        parts.push("production-read".to_string());
    }
    if args.production_read_only {
        parts.push("read-only".to_string());
    }
    parts.join(" ")
}

fn validate_args(args: &SpirePipelineArgs) -> Result<()> {
    if args.queries_limit == 0 {
        return Err(eyre!("--queries-limit must be >= 1"));
    }
    if args.top_k < 0 {
        return Err(eyre!("--top-k must be >= 0"));
    }
    if args.query_metric_k == 0 {
        return Err(eyre!("--query-metric-k must be >= 1"));
    }
    if args.production_read_only
        && !args.include_query_metrics
        && !args.include_recall
        && !args.include_production_read_profile
    {
        return Err(eyre!(
            "--production-read-only requires --include-query-metrics, --include-recall, or --include-production-read-profile"
        ));
    }
    if args.production_read_only && args.funnel_output.is_some() {
        return Err(eyre!(
            "--funnel-output requires SQL-visible local pipeline diagnostics; remove --production-read-only"
        ));
    }
    if args.production_read_only && args.stage_containment_output.is_some() {
        return Err(eyre!(
            "--stage-containment-output requires SQL-visible local pipeline diagnostics; remove --production-read-only"
        ));
    }
    if args.leaf_block_rank_output.is_some() && !args.include_recall {
        return Err(eyre!(
            "--leaf-block-rank-output requires --include-recall so exact truth ids are available"
        ));
    }
    if args.target_block_rank_output.is_some() && !args.include_recall {
        return Err(eyre!(
            "--target-block-rank-output requires --include-recall so exact truth ids are available"
        ));
    }
    if args.miss_attribution_output.is_some() && !args.include_recall {
        return Err(eyre!(
            "--miss-attribution-output requires --include-recall so exact truth ids are available"
        ));
    }
    if args.stage_containment_output.is_some() && !args.include_recall {
        return Err(eyre!(
            "--stage-containment-output requires --include-recall so exact truth ids are available"
        ));
    }
    if args.miss_attribution_output.is_some() && !args.include_query_metrics {
        return Err(eyre!(
            "--miss-attribution-output requires --include-query-metrics so returned ids are available"
        ));
    }
    if args.stage_containment_output.is_some() && !args.include_query_metrics {
        return Err(eyre!(
            "--stage-containment-output requires --include-query-metrics so returned ids are available"
        ));
    }
    if args.truth_cache_file.is_some() && !args.include_recall {
        return Err(eyre!(
            "--truth-cache-file requires --include-recall so cached truth ids are consumed"
        ));
    }
    if (args.leaf_block_rank_output.is_some()
        || args.target_block_rank_output.is_some()
        || args.miss_attribution_output.is_some()
        || args.stage_containment_output.is_some())
        && args.leaf_block_rank_local_sequence_offset < 0
    {
        return Err(eyre!(
            "--leaf-block-rank-local-sequence-offset must be >= 0"
        ));
    }
    for column in &args.query_metric_projection_columns {
        profiles::validate_ident(column).wrap_err_with(|| {
            format!("invalid --query-metric-projection-columns entry {column:?}")
        })?;
    }
    for pid in &args.remote_selected_pids {
        if *pid < 0 {
            return Err(eyre!("--remote-selected-pids entries must be >= 0"));
        }
    }
    if let Some(epoch) = args.remote_requested_epoch {
        if epoch <= 0 {
            return Err(eyre!("--remote-requested-epoch must be greater than 0"));
        }
    }
    if let Some(rerank_width) = args.rerank_width {
        if !(-1..=EC_SPIRE_MAX_RERANK_WIDTH).contains(&rerank_width) {
            return Err(eyre!(
                "--rerank-width must be between -1 and {}",
                EC_SPIRE_MAX_RERANK_WIDTH
            ));
        }
    }
    if let Some(max_candidate_rows) = args.max_candidate_rows {
        if !(-1..=EC_SPIRE_MAX_CANDIDATE_ROWS).contains(&max_candidate_rows) {
            return Err(eyre!(
                "--max-candidate-rows must be between -1 and {}",
                EC_SPIRE_MAX_CANDIDATE_ROWS
            ));
        }
    }
    if let Some(max_routed_candidate_rows) = args.max_routed_candidate_rows {
        if !(0..=EC_SPIRE_MAX_CANDIDATE_ROWS).contains(&max_routed_candidate_rows) {
            return Err(eyre!(
                "--max-routed-candidate-rows must be between 0 and {}",
                EC_SPIRE_MAX_CANDIDATE_ROWS
            ));
        }
    }
    validate_optional_cost_scale(
        "--cost-routing-dimension-scale",
        args.cost_routing_dimension_scale,
    )?;
    validate_optional_cost_scale(
        "--cost-leaf-dimension-scale",
        args.cost_leaf_dimension_scale,
    )?;
    validate_optional_cost_scale("--cost-index-page-scale", args.cost_index_page_scale)?;
    validate_optional_cost_scale(
        "--cost-local-store-page-fanout-scale",
        args.cost_local_store_page_fanout_scale,
    )?;
    validate_optional_cost_scale(
        "--cost-storage-scoring-multiplier",
        args.cost_storage_scoring_multiplier,
    )?;
    validate_optional_cost_scale("--cost-rerank-multiplier", args.cost_rerank_multiplier)?;
    Ok(())
}

fn validate_optional_cost_scale(flag: &str, value: Option<f64>) -> Result<()> {
    if let Some(value) = value {
        if !(value.is_finite() && (0.0..=EC_SPIRE_MAX_COST_SCALE).contains(&value)) {
            return Err(eyre!(
                "{flag} must be finite and between 0 and {}",
                EC_SPIRE_MAX_COST_SCALE
            ));
        }
    }
    Ok(())
}

fn build_query_metric_sql(corpus_table: &str, projection_columns: &[String]) -> String {
    let mut select_columns = vec!["id".to_owned()];
    for column in projection_columns {
        if column != "id" && !select_columns.iter().any(|existing| existing == column) {
            select_columns.push(column.clone());
        }
    }
    format!(
        "SELECT {select_columns} FROM {corpus_table} \
         ORDER BY embedding <#> \
         $1::real[] \
         LIMIT $2",
        select_columns = select_columns.join(", ")
    )
}

fn query_matrix(queries: &[QueryVector]) -> Result<Array2<f32>> {
    let Some(first) = queries.first() else {
        return Err(eyre!("query metrics require at least one query"));
    };
    let dimensions = first.source.len();
    if dimensions == 0 {
        return Err(eyre!("query metrics require non-empty query vectors"));
    }
    let mut values = Vec::with_capacity(queries.len() * dimensions);
    for query in queries {
        if query.source.len() != dimensions {
            return Err(eyre!(
                "query metrics require fixed dimensions; query {} has {}, expected {}",
                query.id,
                query.source.len(),
                dimensions
            ));
        }
        values.extend_from_slice(&query.source);
    }
    Array2::from_shape_vec((queries.len(), dimensions), values)
        .wrap_err("building query metrics matrix")
}

struct QueryMetricRow {
    duration: Duration,
    predicted_ids: Vec<i64>,
}

async fn execute_query_metric(
    client: &Client,
    statement: &tokio_postgres::Statement,
    query: &[f32],
    k: usize,
) -> Result<QueryMetricRow> {
    let k = i64::try_from(k).wrap_err("--query-metric-k exceeds i64")?;
    let query = query.to_vec();
    let started = Instant::now();
    let rows = client
        .query(statement, &[&query, &k])
        .await
        .wrap_err("executing SPIRE pipeline query-metrics KNN query")?;
    let duration = started.elapsed();
    Ok(QueryMetricRow {
        duration,
        predicted_ids: rows.into_iter().map(|row| row.get(0)).collect(),
    })
}

fn sweep_values(args: &SpirePipelineArgs) -> Result<Vec<i32>> {
    let values = if args.sweep.is_empty() {
        EC_SPIRE.default_sweep.to_vec()
    } else {
        args.sweep.clone()
    };
    for value in &values {
        if !(0..=EC_SPIRE_MAX_NPROBE).contains(value) {
            return Err(eyre!(
                "--sweep values must be between 0 and {}",
                EC_SPIRE_MAX_NPROBE
            ));
        }
    }
    Ok(values)
}

async fn resolve_spire_index(
    client: &Client,
    corpus_table: &str,
    requested_index: Option<&str>,
) -> Result<String> {
    if let Some(index) = requested_index {
        let row = client
            .query_one(
                "SELECT EXISTS (
                    SELECT 1
                    FROM pg_class t
                    JOIN pg_index ix ON ix.indrelid = t.oid
                    JOIN pg_class i ON i.oid = ix.indexrelid
                    JOIN pg_am am ON am.oid = i.relam
                    WHERE t.relname = $1
                      AND i.relname = $2
                      AND am.amname = 'ec_spire'
                )",
                &[&corpus_table, &index],
            )
            .await
            .wrap_err("validating SPIRE index")?;
        if !row.get::<_, bool>(0) {
            return Err(eyre!(
                "index {:?} is not an ec_spire index on {:?}",
                index,
                corpus_table
            ));
        }
        return Ok(index.to_owned());
    }

    let rows = client
        .query(
            "SELECT i.relname
             FROM pg_class t
             JOIN pg_index ix ON ix.indrelid = t.oid
             JOIN pg_class i ON i.oid = ix.indexrelid
             JOIN pg_am am ON am.oid = i.relam
             WHERE t.relname = $1
               AND am.amname = 'ec_spire'
             ORDER BY i.relname",
            &[&corpus_table],
        )
        .await
        .wrap_err("finding SPIRE index")?;
    match rows.len() {
        0 => Err(eyre!(
            "no ec_spire index found on {:?}; build one first with `ecaz corpus load --profile ec_spire ...`",
            corpus_table
        )),
        1 => Ok(rows[0].get::<_, String>(0)),
        _ => Err(eyre!(
            "multiple ec_spire indexes found on {:?}; pass --index",
            corpus_table
        )),
    }
}

async fn fetch_queries(
    client: &Client,
    queries_table: &str,
    queries_limit: usize,
) -> Result<Vec<QueryVector>> {
    let sql =
        format!("SELECT id::bigint, source FROM {queries_table} ORDER BY id LIMIT {queries_limit}");
    let rows = client
        .query(&sql, &[])
        .await
        .wrap_err_with(|| format!("reading {queries_table}"))?;
    Ok(rows
        .into_iter()
        .map(|row| QueryVector {
            id: row.get(0),
            source: row.get(1),
        })
        .collect())
}

async fn apply_session_options(
    client: &Client,
    nprobe: i32,
    rerank_width: Option<i32>,
    max_candidate_rows: Option<i32>,
    max_routed_candidate_rows: Option<i32>,
    remote_tuple_transport: Option<SpireRemoteTupleTransportMode>,
    adaptive_nprobe_options: super::AdaptiveNprobeBenchOptions,
    cost_tuning_options: SpireCostTuningOptions,
    session_gucs: &[(String, String)],
) -> Result<()> {
    super::apply_session_gucs(client, session_gucs).await?;
    client
        .batch_execute(&format!("SET ec_spire.nprobe = {nprobe}"))
        .await
        .wrap_err_with(|| format!("SET ec_spire.nprobe = {nprobe}"))?;
    if let Some(rerank_width) = rerank_width {
        client
            .batch_execute(&format!("SET ec_spire.rerank_width = {rerank_width}"))
            .await
            .wrap_err_with(|| format!("SET ec_spire.rerank_width = {rerank_width}"))?;
    }
    if let Some(max_candidate_rows) = max_candidate_rows {
        client
            .batch_execute(&format!(
                "SET ec_spire.max_candidate_rows = {max_candidate_rows}"
            ))
            .await
            .wrap_err_with(|| format!("SET ec_spire.max_candidate_rows = {max_candidate_rows}"))?;
    }
    if let Some(max_routed_candidate_rows) = max_routed_candidate_rows {
        client
            .batch_execute(&format!(
                "SET ec_spire.max_routed_candidate_rows = {max_routed_candidate_rows}"
            ))
            .await
            .wrap_err_with(|| {
                format!("SET ec_spire.max_routed_candidate_rows = {max_routed_candidate_rows}")
            })?;
    }
    if let Some(remote_tuple_transport) = remote_tuple_transport {
        client
            .batch_execute(&format!(
                "SET ec_spire.remote_tuple_transport = '{}'",
                remote_tuple_transport.as_guc_value()
            ))
            .await
            .wrap_err_with(|| {
                format!(
                    "SET ec_spire.remote_tuple_transport = '{}'",
                    remote_tuple_transport.as_guc_value()
                )
            })?;
    }
    super::apply_adaptive_nprobe_options(client, &EC_SPIRE, adaptive_nprobe_options).await?;
    apply_cost_tuning_options(client, cost_tuning_options).await?;
    Ok(())
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct SpireCostTuningOptions {
    routing_dimension_scale: Option<f64>,
    leaf_dimension_scale: Option<f64>,
    index_page_scale: Option<f64>,
    local_store_page_fanout_scale: Option<f64>,
    storage_scoring_multiplier: Option<f64>,
    rerank_multiplier: Option<f64>,
}

async fn apply_cost_tuning_options(client: &Client, options: SpireCostTuningOptions) -> Result<()> {
    apply_cost_tuning_option(
        client,
        "ec_spire.cost_routing_dimension_scale",
        options.routing_dimension_scale,
    )
    .await?;
    apply_cost_tuning_option(
        client,
        "ec_spire.cost_leaf_dimension_scale",
        options.leaf_dimension_scale,
    )
    .await?;
    apply_cost_tuning_option(
        client,
        "ec_spire.cost_index_page_scale",
        options.index_page_scale,
    )
    .await?;
    apply_cost_tuning_option(
        client,
        "ec_spire.cost_local_store_page_fanout_scale",
        options.local_store_page_fanout_scale,
    )
    .await?;
    apply_cost_tuning_option(
        client,
        "ec_spire.cost_storage_scoring_multiplier",
        options.storage_scoring_multiplier,
    )
    .await?;
    apply_cost_tuning_option(
        client,
        "ec_spire.cost_rerank_multiplier",
        options.rerank_multiplier,
    )
    .await?;
    Ok(())
}

async fn apply_cost_tuning_option(client: &Client, guc: &str, value: Option<f64>) -> Result<()> {
    if let Some(value) = value {
        client
            .batch_execute(&format!("SET {guc} = {value}"))
            .await
            .wrap_err_with(|| format!("SET {guc} = {value}"))?;
    }
    Ok(())
}

async fn query_routing_rows(
    client: &Client,
    index: &str,
    query: &[f32],
) -> Result<Vec<RoutingRow>> {
    let rows = client
        .query(routing_snapshot_sql(), &[&index, &query])
        .await
        .wrap_err("querying ec_spire_index_scan_routing_snapshot")?;
    Ok(rows.into_iter().map(RoutingRow::from).collect())
}

async fn query_local_pipeline_rows(
    client: &Client,
    index: &str,
    query: &[f32],
) -> Result<Vec<LocalPipelineRow>> {
    let rows = client
        .query(local_pipeline_snapshot_sql(), &[&index, &query])
        .await
        .wrap_err("querying ec_spire_index_scan_pipeline_snapshot")?;
    Ok(rows.into_iter().map(LocalPipelineRow::from).collect())
}

async fn query_local_store_overlap_rows(
    client: &Client,
    index: &str,
    query: &[f32],
) -> Result<Vec<LocalStoreOverlapRow>> {
    let rows = client
        .query(local_store_overlap_sql(), &[&index, &query])
        .await
        .wrap_err("querying ec_spire_index_scan_local_store_read_overlap_harness")?;
    Ok(rows.into_iter().map(LocalStoreOverlapRow::from).collect())
}

async fn query_leaf_candidate_rows(
    client: &Client,
    index: &str,
    query: &[f32],
) -> Result<Vec<LeafCandidateRow>> {
    match client
        .query(leaf_candidate_snapshot_sql(), &[&index, &query])
        .await
    {
        Ok(rows) => Ok(rows.into_iter().map(LeafCandidateRow::from).collect()),
        Err(err) if is_missing_leaf_row_segment_snapshot_column(&err) => {
            let rows = client
                .query(legacy_leaf_candidate_snapshot_sql(), &[&index, &query])
                .await
                .wrap_err(
                    "querying legacy ec_spire_index_scan_leaf_candidate_snapshot without row segment metrics",
                )?;
            Ok(rows
                .into_iter()
                .map(LeafCandidateRow::from_legacy)
                .collect())
        }
        Err(err) => Err(err).wrap_err("querying ec_spire_index_scan_leaf_candidate_snapshot"),
    }
}

fn is_missing_leaf_row_segment_snapshot_column(err: &tokio_postgres::Error) -> bool {
    let db_message = err.as_db_error().map(|db_error| db_error.message());
    let display_message;
    let message = match db_message {
        Some(message) => message,
        None => {
            display_message = err.to_string();
            display_message.as_str()
        }
    };
    message.contains("leaf_row_segment_read_count")
        || message.contains("leaf_row_segment_read_bytes")
}

async fn query_rerank_locality_row(
    client: &Client,
    index: &str,
    query: &[f32],
) -> Result<Option<RerankLocalityRow>> {
    match client
        .query_opt(rerank_locality_snapshot_sql(), &[&index, &query])
        .await
    {
        Ok(row) => Ok(row.map(RerankLocalityRow::from)),
        Err(err) if is_missing_rerank_locality_snapshot(&err) => Ok(None),
        Err(err) => Err(err).wrap_err("querying ec_spire_index_scan_rerank_locality_snapshot"),
    }
}

fn is_missing_rerank_locality_snapshot(err: &tokio_postgres::Error) -> bool {
    let db_message = err.as_db_error().map(|db_error| db_error.message());
    let display_message;
    let message = match db_message {
        Some(message) => message,
        None => {
            display_message = err.to_string();
            display_message.as_str()
        }
    };
    message.contains("ec_spire_index_scan_rerank_locality_snapshot")
}

async fn query_leaf_target_assignment_rows(
    client: &Client,
    index: &str,
    target_local_sequences: &[i64],
) -> Result<Vec<LeafTargetAssignmentRow>> {
    let target_local_sequences = target_local_sequences.to_vec();
    let rows = client
        .query(
            leaf_target_assignment_snapshot_sql(),
            &[&index, &target_local_sequences],
        )
        .await
        .wrap_err("querying ec_spire_index_leaf_target_assignment_snapshot")?;
    Ok(rows
        .into_iter()
        .map(LeafTargetAssignmentRow::from)
        .collect())
}

async fn query_leaf_block_rank_rows(
    client: &Client,
    index: &str,
    query: &[f32],
    target_local_sequences: &[i64],
) -> Result<Vec<LeafBlockRankRow>> {
    let target_local_sequences = target_local_sequences.to_vec();
    let rows = client
        .query(
            leaf_block_rank_snapshot_sql(),
            &[&index, &query, &target_local_sequences],
        )
        .await
        .wrap_err("querying ec_spire_index_scan_leaf_block_rank_snapshot")?;
    Ok(rows.into_iter().map(LeafBlockRankRow::from).collect())
}

async fn query_leaf_target_block_rank_rows(
    client: &Client,
    index: &str,
    query: &[f32],
    target_local_sequences: &[i64],
) -> Result<Vec<LeafBlockRankRow>> {
    let target_local_sequences = target_local_sequences.to_vec();
    let rows = client
        .query(
            leaf_target_block_rank_snapshot_sql(),
            &[&index, &query, &target_local_sequences],
        )
        .await
        .wrap_err("querying ec_spire_index_scan_leaf_target_block_rank_snapshot")?;
    Ok(rows.into_iter().map(LeafBlockRankRow::from).collect())
}

async fn query_remote_pipeline_rows(
    client: &Client,
    index: &str,
    requested_epoch: i64,
    query: &[f32],
    selected_pids: &[i64],
    top_k: i32,
    consistency_mode: &str,
) -> Result<Vec<RemotePipelineRow>> {
    let selected_pids = selected_pids.to_vec();
    let rows = client
        .query(
            remote_pipeline_steps_sql(),
            &[
                &index,
                &requested_epoch,
                &query,
                &selected_pids,
                &top_k,
                &consistency_mode,
            ],
        )
        .await
        .wrap_err("querying ec_spire_remote_pipeline_steps")?;
    Ok(rows.into_iter().map(RemotePipelineRow::from).collect())
}

async fn query_degraded_skip_rows(
    client: &Client,
    index: &str,
    requested_epoch: i64,
    query: &[f32],
    selected_pids: &[i64],
    top_k: i32,
    consistency_mode: &str,
) -> Result<Vec<DegradedSkipRow>> {
    let query = query.to_vec();
    let selected_pids = selected_pids.to_vec();
    let rows = client
        .query(
            degraded_skip_report_sql(),
            &[
                &index,
                &requested_epoch,
                &query,
                &selected_pids,
                &top_k,
                &consistency_mode,
            ],
        )
        .await
        .wrap_err("querying ec_spire_remote_search_degraded_skip_report")?;
    Ok(rows.into_iter().map(DegradedSkipRow::from).collect())
}

async fn query_production_read_profile_row(
    client: &Client,
    index: &str,
    query: &[f32],
    top_k: usize,
) -> Result<ProductionReadProfileRow> {
    let top_k = i32::try_from(top_k).map_err(|_| eyre!("query metric k exceeds i32"))?;
    let rows = client
        .query(production_read_profile_sql(), &[&index, &query, &top_k])
        .await
        .wrap_err("querying ec_spire_remote_search_production_read_profile")?;
    Ok(ProductionReadProfileRow::from_metric_rows(rows))
}

async fn query_remote_placement_gate(client: &Client, index: &str) -> Result<RemotePlacementGate> {
    let row = client
        .query_one(remote_placement_gate_sql(), &[&index])
        .await
        .wrap_err("querying ec_spire_index_placement_snapshot remote placement gate")?;
    Ok(RemotePlacementGate::from(row))
}

async fn query_cost_tuning_row(client: &Client, index: &str) -> Result<CostTuningRow> {
    let row = client
        .query_one(cost_tuning_snapshot_sql(), &[&index])
        .await
        .wrap_err("querying ec_spire_index_cost_tuning_snapshot")?;
    Ok(CostTuningRow::from(row))
}

async fn query_endpoint_identity(client: &Client, index: &str) -> Result<EndpointIdentityRow> {
    let row = client
        .query_one(endpoint_identity_sql(), &[&index])
        .await
        .wrap_err("querying ec_spire_remote_search_endpoint_identity")?;
    Ok(EndpointIdentityRow::from(row))
}

fn routing_snapshot_sql() -> &'static str {
    "SELECT active_epoch, effective_nprobe, effective_nprobe_source,
            adaptive_nprobe_decision, recursive_beam_width, max_leaf_routes,
            max_routing_expansions, routing_level, input_frontier_width,
            expanded_parent_count, selected_child_count, deduped_route_count,
            truncation_reason
     FROM ec_spire_index_scan_routing_snapshot($1::text::regclass::oid, $2::real[])
     ORDER BY routing_level"
}

fn local_pipeline_snapshot_sql() -> &'static str {
    "SELECT step_ordinal, step_name, active_epoch, status, item_count,
            ready_count, blocked_count, route_count, candidate_count,
            heap_rerank_row_count, remote_fanout_count, next_blocker,
            recommendation
     FROM ec_spire_index_scan_pipeline_snapshot($1::text::regclass::oid, $2::real[])
     ORDER BY step_ordinal"
}

fn local_store_overlap_sql() -> &'static str {
    "SELECT node_id, local_store_id, route_count, leaf_route_count,
            delta_route_count, candidate_row_count, prefetched_object_bytes,
            read_batch_count, delta_decode_count
     FROM ec_spire_index_scan_local_store_read_overlap_harness($1::text::regclass::oid, $2::real[])
     ORDER BY node_id, local_store_id"
}

fn leaf_candidate_snapshot_sql() -> &'static str {
    "SELECT pid, node_id, local_store_id, object_bytes, route_count, scanned_count,
            candidate_row_count, leaf_block_available_count, leaf_block_selected_count,
            leaf_block_skipped_count, leaf_summary_object_bytes, leaf_row_object_bytes,
            primary_candidate_row_count,
            boundary_replica_candidate_row_count, deduped_candidate_row_count,
            truncated_candidate_row_count, candidate_winner_count,
            leaf_object_read_nanos, leaf_summary_score_nanos, leaf_row_score_nanos,
            candidate_score_nanos,
            candidate_materialize_nanos, candidate_heap_append_nanos,
            leaf_row_segment_read_count, leaf_row_segment_read_bytes
     FROM ec_spire_index_scan_leaf_candidate_snapshot($1::text::regclass::oid, $2::real[])
     ORDER BY pid"
}

fn legacy_leaf_candidate_snapshot_sql() -> &'static str {
    "SELECT pid, node_id, local_store_id, object_bytes, route_count, scanned_count,
            candidate_row_count, leaf_block_available_count, leaf_block_selected_count,
            leaf_block_skipped_count, leaf_summary_object_bytes, leaf_row_object_bytes,
            primary_candidate_row_count,
            boundary_replica_candidate_row_count, deduped_candidate_row_count,
            truncated_candidate_row_count, candidate_winner_count,
            leaf_object_read_nanos, leaf_summary_score_nanos, leaf_row_score_nanos,
            candidate_score_nanos,
            candidate_materialize_nanos, candidate_heap_append_nanos
     FROM ec_spire_index_scan_leaf_candidate_snapshot($1::text::regclass::oid, $2::real[])
     ORDER BY pid"
}

fn rerank_locality_snapshot_sql() -> &'static str {
    "SELECT candidate_count, rerank_prefix_count, unique_heap_block_count,
            heap_block_transition_count, heap_block_span, heap_block_jump_sum,
            heap_block_jump_max
     FROM ec_spire_index_scan_rerank_locality_snapshot($1::text::regclass::oid, $2::real[])"
}

fn leaf_target_assignment_snapshot_sql() -> &'static str {
    "SELECT target_ordinal, target_local_sequence, status, leaf_pid, parent_pid,
            object_version, row_index, assignment_flags
     FROM ec_spire_index_leaf_target_assignment_snapshot(
            $1::text::regclass::oid, $2::bigint[])
     ORDER BY target_ordinal, leaf_pid NULLS LAST, row_index NULLS LAST"
}

fn leaf_block_rank_snapshot_sql() -> &'static str {
    "SELECT target_ordinal, target_local_sequence, status, max_global_blocks,
            radius_weight, scored_block_count, block_rank, selected_by_global_cap,
            pid, node_id, local_store_id, object_version, row_index, row_base,
            row_end, row_count, block_ip, cap_block_ip, block_ip_margin_to_cap,
            route_rank, route_score, assignment_flags
     FROM ec_spire_index_scan_leaf_block_rank_snapshot(
            $1::text::regclass::oid, $2::real[], $3::bigint[])
     ORDER BY target_ordinal, block_rank NULLS LAST, pid NULLS LAST, row_index NULLS LAST"
}

fn leaf_target_block_rank_snapshot_sql() -> &'static str {
    "SELECT target_ordinal, target_local_sequence, status, max_global_blocks,
            radius_weight, scored_block_count, block_rank, selected_by_global_cap,
            pid, node_id, local_store_id, object_version, row_index, row_base,
            row_end, row_count, block_ip, cap_block_ip, block_ip_margin_to_cap,
            route_rank, route_score, assignment_flags
     FROM ec_spire_index_scan_leaf_target_block_rank_snapshot(
            $1::text::regclass::oid, $2::real[], $3::bigint[])
     ORDER BY target_ordinal, block_rank NULLS LAST, pid NULLS LAST, row_index NULLS LAST"
}

fn remote_pipeline_steps_sql() -> &'static str {
    "SELECT step_ordinal, step_name, requested_epoch, status, item_count,
            ready_count, blocked_count, remote_pid_count, next_blocker,
            recommendation
     FROM ec_spire_remote_pipeline_steps(
            $1::text::regclass::oid, $2::bigint, $3::real[], $4::bigint[],
            $5::integer, $6::text)
     ORDER BY step_ordinal"
}

fn degraded_skip_report_sql() -> &'static str {
    "SELECT requested_epoch, node_id, skipped_pid_count, first_skip_category,
            status
     FROM ec_spire_remote_search_degraded_skip_report(
            $1::text::regclass::oid, $2::bigint, $3::real[], $4::bigint[],
            $5::integer, $6::text)
     ORDER BY node_id"
}

fn production_read_profile_sql() -> &'static str {
    "SELECT metric, value
       FROM ec_spire_remote_search_production_read_profile(
            $1::text::regclass::oid, $2::real[], $3::integer)"
}

fn endpoint_identity_sql() -> &'static str {
    "SELECT tuple_transport_capabilities, tuple_transport_default,
            tuple_transport_status, status, recommendation
     FROM ec_spire_remote_search_endpoint_identity($1::text::regclass::oid)"
}

fn remote_placement_gate_sql() -> &'static str {
    "SELECT coalesce(sum(placement_count), 0)::bigint AS total_placement_count,
            coalesce(sum(placement_count) FILTER (WHERE node_id > 1), 0)::bigint
                AS remote_placement_count,
            coalesce(sum(placement_count) FILTER (WHERE node_id <= 1), 0)::bigint
                AS local_placement_count,
            count(*) FILTER (WHERE node_id > 1)::bigint AS remote_node_count
     FROM ec_spire_remote_node_snapshot($1::text::regclass::oid)"
}

fn cost_tuning_snapshot_sql() -> &'static str {
    "SELECT storage_format, effective_rerank_width,
            cost_routing_dimension_scale, cost_leaf_dimension_scale,
            cost_index_page_scale, cost_local_store_page_fanout_scale,
            cost_storage_scoring_multiplier, effective_storage_scoring_multiplier,
            cost_rerank_multiplier, effective_rerank_multiplier
     FROM ec_spire_index_cost_tuning_snapshot($1::text::regclass::oid)"
}

#[derive(Debug)]
struct QueryVector {
    #[allow(dead_code)]
    id: i64,
    source: Vec<f32>,
}

#[derive(Debug)]
struct RoutingRow {
    effective_nprobe: i64,
    effective_nprobe_source: String,
    adaptive_nprobe_decision: String,
    recursive_beam_width: i64,
    max_leaf_routes: i64,
    max_routing_expansions: i64,
    routing_level: i64,
    input_frontier_width: i64,
    expanded_parent_count: i64,
    selected_child_count: i64,
    deduped_route_count: i64,
    truncation_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RemotePlacementGate {
    total_placement_count: i64,
    remote_placement_count: i64,
    local_placement_count: i64,
    remote_node_count: i64,
}

impl From<Row> for RemotePlacementGate {
    fn from(row: Row) -> Self {
        Self {
            total_placement_count: row.get("total_placement_count"),
            remote_placement_count: row.get("remote_placement_count"),
            local_placement_count: row.get("local_placement_count"),
            remote_node_count: row.get("remote_node_count"),
        }
    }
}

fn enforce_remote_placement_gate(index: &str, row: &RemotePlacementGate) -> Result<()> {
    if row.remote_placement_count <= 0 {
        return Err(eyre!(
            "distributed SPIRE placement gate failed for index {:?}: remote placement count is 0 (total placements {}, local placements {}, remote nodes {}). Run distributed placement/materialization before AWS distributed read verification.",
            index,
            row.total_placement_count,
            row.local_placement_count,
            row.remote_node_count
        ));
    }
    Ok(())
}

impl From<Row> for RoutingRow {
    fn from(row: Row) -> Self {
        Self {
            effective_nprobe: row.get(1),
            effective_nprobe_source: row.get(2),
            adaptive_nprobe_decision: row.get(3),
            recursive_beam_width: row.get(4),
            max_leaf_routes: row.get(5),
            max_routing_expansions: row.get(6),
            routing_level: row.get(7),
            input_frontier_width: row.get(8),
            expanded_parent_count: row.get(9),
            selected_child_count: row.get(10),
            deduped_route_count: row.get(11),
            truncation_reason: row.get(12),
        }
    }
}

#[derive(Debug, Clone)]
struct LocalPipelineRow {
    step_ordinal: i64,
    step_name: String,
    active_epoch: i64,
    status: String,
    item_count: i64,
    ready_count: i64,
    blocked_count: i64,
    route_count: i64,
    candidate_count: i64,
    heap_rerank_row_count: i64,
    remote_fanout_count: i64,
    next_blocker: String,
    recommendation: String,
}

impl From<Row> for LocalPipelineRow {
    fn from(row: Row) -> Self {
        Self {
            step_ordinal: row.get(0),
            step_name: row.get(1),
            active_epoch: row.get(2),
            status: row.get(3),
            item_count: row.get(4),
            ready_count: row.get(5),
            blocked_count: row.get(6),
            route_count: row.get(7),
            candidate_count: row.get(8),
            heap_rerank_row_count: row.get(9),
            remote_fanout_count: row.get(10),
            next_blocker: row.get(11),
            recommendation: row.get(12),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct FunnelPipelineStageRecord {
    step_ordinal: i64,
    step_name: String,
    active_epoch: i64,
    status: String,
    item_count: i64,
    ready_count: i64,
    blocked_count: i64,
    route_count: i64,
    candidate_count: i64,
    heap_rerank_row_count: i64,
    remote_fanout_count: i64,
    next_blocker: String,
    recommendation: String,
}

impl From<&LocalPipelineRow> for FunnelPipelineStageRecord {
    fn from(row: &LocalPipelineRow) -> Self {
        Self {
            step_ordinal: row.step_ordinal,
            step_name: row.step_name.clone(),
            active_epoch: row.active_epoch,
            status: row.status.clone(),
            item_count: row.item_count,
            ready_count: row.ready_count,
            blocked_count: row.blocked_count,
            route_count: row.route_count,
            candidate_count: row.candidate_count,
            heap_rerank_row_count: row.heap_rerank_row_count,
            remote_fanout_count: row.remote_fanout_count,
            next_blocker: row.next_blocker.clone(),
            recommendation: row.recommendation.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct LeafCandidateRow {
    #[allow(dead_code)]
    pid: i64,
    #[allow(dead_code)]
    node_id: i64,
    #[allow(dead_code)]
    local_store_id: i64,
    object_bytes: i64,
    route_count: i64,
    scanned_count: i64,
    candidate_row_count: i64,
    leaf_block_available_count: i64,
    leaf_block_selected_count: i64,
    leaf_block_skipped_count: i64,
    leaf_summary_object_bytes: i64,
    leaf_row_object_bytes: i64,
    leaf_row_segment_read_count: i64,
    leaf_row_segment_read_bytes: i64,
    primary_candidate_row_count: i64,
    boundary_replica_candidate_row_count: i64,
    deduped_candidate_row_count: i64,
    truncated_candidate_row_count: i64,
    candidate_winner_count: i64,
    leaf_object_read_nanos: i64,
    leaf_summary_score_nanos: i64,
    leaf_row_score_nanos: i64,
    candidate_score_nanos: i64,
    candidate_materialize_nanos: i64,
    candidate_heap_append_nanos: i64,
}

impl From<Row> for LeafCandidateRow {
    fn from(row: Row) -> Self {
        Self {
            pid: row.get(0),
            node_id: row.get(1),
            local_store_id: row.get(2),
            object_bytes: row.get(3),
            route_count: row.get(4),
            scanned_count: row.get(5),
            candidate_row_count: row.get(6),
            leaf_block_available_count: row.get(7),
            leaf_block_selected_count: row.get(8),
            leaf_block_skipped_count: row.get(9),
            leaf_summary_object_bytes: row.get(10),
            leaf_row_object_bytes: row.get(11),
            primary_candidate_row_count: row.get(12),
            boundary_replica_candidate_row_count: row.get(13),
            deduped_candidate_row_count: row.get(14),
            truncated_candidate_row_count: row.get(15),
            candidate_winner_count: row.get(16),
            leaf_object_read_nanos: row.get(17),
            leaf_summary_score_nanos: row.get(18),
            leaf_row_score_nanos: row.get(19),
            candidate_score_nanos: row.get(20),
            candidate_materialize_nanos: row.get(21),
            candidate_heap_append_nanos: row.get(22),
            leaf_row_segment_read_count: row.get(23),
            leaf_row_segment_read_bytes: row.get(24),
        }
    }
}

impl LeafCandidateRow {
    fn from_legacy(row: Row) -> Self {
        Self {
            pid: row.get(0),
            node_id: row.get(1),
            local_store_id: row.get(2),
            object_bytes: row.get(3),
            route_count: row.get(4),
            scanned_count: row.get(5),
            candidate_row_count: row.get(6),
            leaf_block_available_count: row.get(7),
            leaf_block_selected_count: row.get(8),
            leaf_block_skipped_count: row.get(9),
            leaf_summary_object_bytes: row.get(10),
            leaf_row_object_bytes: row.get(11),
            leaf_row_segment_read_count: 0,
            leaf_row_segment_read_bytes: 0,
            primary_candidate_row_count: row.get(12),
            boundary_replica_candidate_row_count: row.get(13),
            deduped_candidate_row_count: row.get(14),
            truncated_candidate_row_count: row.get(15),
            candidate_winner_count: row.get(16),
            leaf_object_read_nanos: row.get(17),
            leaf_summary_score_nanos: row.get(18),
            leaf_row_score_nanos: row.get(19),
            candidate_score_nanos: row.get(20),
            candidate_materialize_nanos: row.get(21),
            candidate_heap_append_nanos: row.get(22),
        }
    }
}

#[derive(Debug, Clone)]
struct RerankLocalityRow {
    candidate_count: i64,
    rerank_prefix_count: i64,
    unique_heap_block_count: i64,
    heap_block_transition_count: i64,
    heap_block_span: i64,
    heap_block_jump_sum: i64,
    heap_block_jump_max: i64,
}

impl From<Row> for RerankLocalityRow {
    fn from(row: Row) -> Self {
        Self {
            candidate_count: row.get(0),
            rerank_prefix_count: row.get(1),
            unique_heap_block_count: row.get(2),
            heap_block_transition_count: row.get(3),
            heap_block_span: row.get(4),
            heap_block_jump_sum: row.get(5),
            heap_block_jump_max: row.get(6),
        }
    }
}

#[derive(Debug, Clone)]
struct LeafTargetAssignmentRow {
    target_ordinal: i64,
    #[allow(dead_code)]
    target_local_sequence: i64,
    status: String,
    leaf_pid: Option<i64>,
    #[allow(dead_code)]
    parent_pid: Option<i64>,
    #[allow(dead_code)]
    object_version: Option<i64>,
    row_index: Option<i64>,
    #[allow(dead_code)]
    assignment_flags: Option<i32>,
}

impl From<Row> for LeafTargetAssignmentRow {
    fn from(row: Row) -> Self {
        Self {
            target_ordinal: row.get(0),
            target_local_sequence: row.get(1),
            status: row.get(2),
            leaf_pid: row.get(3),
            parent_pid: row.get(4),
            object_version: row.get(5),
            row_index: row.get(6),
            assignment_flags: row.get(7),
        }
    }
}

#[derive(Debug, Clone)]
struct LeafBlockRankRow {
    target_ordinal: i64,
    target_local_sequence: i64,
    status: String,
    max_global_blocks: i64,
    radius_weight: f64,
    scored_block_count: i64,
    block_rank: Option<i64>,
    selected_by_global_cap: Option<bool>,
    pid: Option<i64>,
    node_id: Option<i64>,
    local_store_id: Option<i64>,
    object_version: Option<i64>,
    row_index: Option<i64>,
    row_base: Option<i64>,
    row_end: Option<i64>,
    row_count: Option<i64>,
    block_ip: Option<f32>,
    cap_block_ip: Option<f32>,
    block_ip_margin_to_cap: Option<f32>,
    route_rank: Option<i64>,
    route_score: Option<f32>,
    assignment_flags: Option<i64>,
}

impl From<Row> for LeafBlockRankRow {
    fn from(row: Row) -> Self {
        Self {
            target_ordinal: row.get(0),
            target_local_sequence: row.get(1),
            status: row.get(2),
            max_global_blocks: row.get(3),
            radius_weight: row.get(4),
            scored_block_count: row.get(5),
            block_rank: row.get(6),
            selected_by_global_cap: row.get(7),
            pid: row.get(8),
            node_id: row.get(9),
            local_store_id: row.get(10),
            object_version: row.get(11),
            row_index: row.get(12),
            row_base: row.get(13),
            row_end: row.get(14),
            row_count: row.get(15),
            block_ip: row.get(16),
            cap_block_ip: row.get(17),
            block_ip_margin_to_cap: row.get(18),
            route_rank: row.get(19),
            route_score: row.get(20),
            assignment_flags: row.get(21),
        }
    }
}

#[derive(Debug, Serialize)]
struct LeafBlockRankRecord {
    kind: &'static str,
    nprobe: i32,
    query_ordinal: usize,
    query_id: i64,
    truth_rank: i64,
    truth_id: i64,
    target_local_sequence: i64,
    status: String,
    max_global_blocks: i64,
    radius_weight: f64,
    scored_block_count: i64,
    block_rank: Option<i64>,
    selected_by_global_cap: Option<bool>,
    pid: Option<i64>,
    node_id: Option<i64>,
    local_store_id: Option<i64>,
    object_version: Option<i64>,
    row_index: Option<i64>,
    row_base: Option<i64>,
    row_end: Option<i64>,
    row_count: Option<i64>,
    block_ip: Option<f32>,
    cap_block_ip: Option<f32>,
    block_ip_margin_to_cap: Option<f32>,
    route_rank: Option<i64>,
    route_score: Option<f32>,
    assignment_flags: Option<i64>,
}

impl LeafBlockRankRecord {
    fn from_row(
        nprobe: i32,
        query_ordinal: usize,
        query_id: i64,
        truth_id: i64,
        row: LeafBlockRankRow,
    ) -> Self {
        Self::from_row_with_kind(
            "spire_leaf_block_rank",
            nprobe,
            query_ordinal,
            query_id,
            truth_id,
            row,
        )
    }

    fn from_row_with_kind(
        kind: &'static str,
        nprobe: i32,
        query_ordinal: usize,
        query_id: i64,
        truth_id: i64,
        row: LeafBlockRankRow,
    ) -> Self {
        Self {
            kind,
            nprobe,
            query_ordinal,
            query_id,
            truth_rank: row.target_ordinal + 1,
            truth_id,
            target_local_sequence: row.target_local_sequence,
            status: row.status,
            max_global_blocks: row.max_global_blocks,
            radius_weight: row.radius_weight,
            scored_block_count: row.scored_block_count,
            block_rank: row.block_rank,
            selected_by_global_cap: row.selected_by_global_cap,
            pid: row.pid,
            node_id: row.node_id,
            local_store_id: row.local_store_id,
            object_version: row.object_version,
            row_index: row.row_index,
            row_base: row.row_base,
            row_end: row.row_end,
            row_count: row.row_count,
            block_ip: row.block_ip,
            cap_block_ip: row.cap_block_ip,
            block_ip_margin_to_cap: row.block_ip_margin_to_cap,
            route_rank: row.route_rank,
            route_score: row.route_score,
            assignment_flags: row.assignment_flags,
        }
    }
}

#[derive(Debug, Serialize)]
struct MissAttributionRecord {
    kind: &'static str,
    nprobe: i32,
    query_ordinal: usize,
    query_id: i64,
    truth_rank: usize,
    truth_id: i64,
    target_local_sequence: i64,
    hit: bool,
    miss_stage: &'static str,
    block_status: String,
    block_rank: Option<i64>,
    selected_by_global_cap: Option<bool>,
    scored_block_count: i64,
    max_global_blocks: i64,
    pid: Option<i64>,
    node_id: Option<i64>,
    local_store_id: Option<i64>,
    row_index: Option<i64>,
    assignment_status: Option<String>,
    assignment_leaf_count: usize,
    selected_assignment_leaf_count: usize,
    leaf_candidate_row_count: Option<i64>,
    leaf_block_available_count: Option<i64>,
    leaf_block_selected_count: Option<i64>,
    leaf_block_skipped_count: Option<i64>,
    truncated_candidate_row_count: Option<i64>,
    returned_rank: Option<usize>,
    returned_count: usize,
}

impl MissAttributionRecord {
    fn from_query(
        nprobe: i32,
        query_ordinal: usize,
        query_id: i64,
        truth_ids: &[i64],
        target_local_sequences: &[i64],
        predicted_ids: &[i64],
        rank_rows: &[LeafBlockRankRow],
    ) -> Result<Vec<Self>> {
        let predicted_id_set = predicted_ids.iter().copied().collect::<HashSet<_>>();
        let returned_rank_by_id = predicted_ids
            .iter()
            .enumerate()
            .map(|(rank_index, predicted_id)| (*predicted_id, rank_index + 1))
            .collect::<HashMap<_, _>>();
        let rank_row_by_ordinal = rank_rows
            .iter()
            .map(|row| (row.target_ordinal, row))
            .collect::<HashMap<_, _>>();
        let mut rows = Vec::with_capacity(truth_ids.len());
        for (truth_index, truth_id) in truth_ids.iter().enumerate() {
            let target_local_sequence =
                *target_local_sequences.get(truth_index).ok_or_else(|| {
                    eyre!(
                        "target local sequence missing for truth ordinal {}",
                        truth_index + 1
                    )
                })?;
            let target_ordinal = i64::try_from(truth_index)
                .map_err(|_| eyre!("truth ordinal {} exceeds i64", truth_index + 1))?;
            let rank_row = rank_row_by_ordinal.get(&target_ordinal).copied();
            let hit = predicted_id_set.contains(truth_id);
            let miss_stage = if hit {
                "hit"
            } else {
                classify_miss_stage(rank_row)
            };
            rows.push(Self {
                kind: "spire_recall_miss_attribution",
                nprobe,
                query_ordinal,
                query_id,
                truth_rank: truth_index + 1,
                truth_id: *truth_id,
                target_local_sequence,
                hit,
                miss_stage,
                block_status: rank_row
                    .map(|row| row.status.clone())
                    .unwrap_or_else(|| "missing_rank_row".to_owned()),
                block_rank: rank_row.and_then(|row| row.block_rank),
                selected_by_global_cap: rank_row.and_then(|row| row.selected_by_global_cap),
                scored_block_count: rank_row.map(|row| row.scored_block_count).unwrap_or(0),
                max_global_blocks: rank_row.map(|row| row.max_global_blocks).unwrap_or(0),
                pid: rank_row.and_then(|row| row.pid),
                node_id: rank_row.and_then(|row| row.node_id),
                local_store_id: rank_row.and_then(|row| row.local_store_id),
                row_index: rank_row.and_then(|row| row.row_index),
                assignment_status: rank_row.map(|row| row.status.clone()),
                assignment_leaf_count: usize::from(rank_row.is_some()),
                selected_assignment_leaf_count: usize::from(
                    rank_row.is_some_and(|row| row.pid.is_some()),
                ),
                leaf_candidate_row_count: None,
                leaf_block_available_count: None,
                leaf_block_selected_count: None,
                leaf_block_skipped_count: None,
                truncated_candidate_row_count: None,
                returned_rank: returned_rank_by_id.get(truth_id).copied(),
                returned_count: predicted_ids.len(),
            });
        }
        Ok(rows)
    }

    fn from_target_assignments(
        nprobe: i32,
        query_ordinal: usize,
        query_id: i64,
        truth_ids: &[i64],
        local_sequence_offset: i64,
        predicted_ids: &[i64],
        leaf_candidate_rows: &[LeafCandidateRow],
        target_assignments_by_ordinal: &HashMap<i64, Vec<LeafTargetAssignmentRow>>,
    ) -> Result<Vec<Self>> {
        let predicted_id_set = predicted_ids.iter().copied().collect::<HashSet<_>>();
        let returned_rank_by_id = predicted_ids
            .iter()
            .enumerate()
            .map(|(rank_index, predicted_id)| (*predicted_id, rank_index + 1))
            .collect::<HashMap<_, _>>();
        let candidate_by_pid = leaf_candidate_rows
            .iter()
            .map(|row| (row.pid, row))
            .collect::<HashMap<_, _>>();
        let truth_count = i64::try_from(truth_ids.len())
            .map_err(|_| eyre!("truth id count {} exceeds i64", truth_ids.len()))?;
        let query_base_ordinal = i64::try_from(query_ordinal - 1)
            .map_err(|_| eyre!("query ordinal {query_ordinal} exceeds i64"))?
            .checked_mul(truth_count)
            .ok_or_else(|| eyre!("query target ordinal overflow"))?;

        let mut rows = Vec::with_capacity(truth_ids.len());
        for (truth_index, truth_id) in truth_ids.iter().enumerate() {
            let target_ordinal = query_base_ordinal
                .checked_add(
                    i64::try_from(truth_index)
                        .map_err(|_| eyre!("truth ordinal {} exceeds i64", truth_index + 1))?,
                )
                .ok_or_else(|| eyre!("target ordinal overflow"))?;
            let target_local_sequence =
                truth_id.checked_add(local_sequence_offset).ok_or_else(|| {
                    eyre!("target assignment local sequence overflow for truth id {truth_id}")
                })?;
            let assignments = target_assignments_by_ordinal
                .get(&target_ordinal)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let selected_assignment_rows = assignments
                .iter()
                .filter_map(|assignment| {
                    assignment
                        .leaf_pid
                        .and_then(|pid| candidate_by_pid.get(&pid).copied())
                })
                .collect::<Vec<_>>();
            let selected_leaf = selected_assignment_rows.first().copied();
            let hit = predicted_id_set.contains(truth_id);
            let miss_stage = if hit {
                "hit"
            } else if assignments.is_empty()
                || assignments
                    .iter()
                    .all(|assignment| assignment.leaf_pid.is_none())
            {
                "assignment_missing"
            } else if selected_assignment_rows.is_empty() {
                "routing_miss"
            } else if selected_assignment_rows
                .iter()
                .any(|row| row.leaf_block_skipped_count > 0)
            {
                "selected_leaf_block_pruning_or_candidate_cap"
            } else {
                "candidate_or_rerank_cap"
            };
            let selected_assignment = assignments
                .iter()
                .find(|assignment| {
                    assignment
                        .leaf_pid
                        .is_some_and(|pid| candidate_by_pid.contains_key(&pid))
                })
                .or_else(|| assignments.first());

            rows.push(Self {
                kind: "spire_recall_miss_attribution",
                nprobe,
                query_ordinal,
                query_id,
                truth_rank: truth_index + 1,
                truth_id: *truth_id,
                target_local_sequence,
                hit,
                miss_stage,
                block_status: if selected_leaf.is_some() {
                    "selected_leaf_candidate_snapshot".to_owned()
                } else {
                    selected_assignment
                        .map(|assignment| assignment.status.clone())
                        .unwrap_or_else(|| "missing_assignment_row".to_owned())
                },
                block_rank: None,
                selected_by_global_cap: None,
                scored_block_count: 0,
                max_global_blocks: 0,
                pid: selected_assignment.and_then(|assignment| assignment.leaf_pid),
                node_id: selected_leaf.map(|row| row.node_id),
                local_store_id: selected_leaf.map(|row| row.local_store_id),
                row_index: selected_assignment.and_then(|assignment| assignment.row_index),
                assignment_status: selected_assignment.map(|assignment| assignment.status.clone()),
                assignment_leaf_count: assignments
                    .iter()
                    .filter(|assignment| assignment.leaf_pid.is_some())
                    .count(),
                selected_assignment_leaf_count: selected_assignment_rows.len(),
                leaf_candidate_row_count: selected_leaf.map(|row| row.candidate_row_count),
                leaf_block_available_count: selected_leaf.map(|row| row.leaf_block_available_count),
                leaf_block_selected_count: selected_leaf.map(|row| row.leaf_block_selected_count),
                leaf_block_skipped_count: selected_leaf.map(|row| row.leaf_block_skipped_count),
                truncated_candidate_row_count: selected_leaf
                    .map(|row| row.truncated_candidate_row_count),
                returned_rank: returned_rank_by_id.get(truth_id).copied(),
                returned_count: predicted_ids.len(),
            });
        }
        Ok(rows)
    }
}

fn classify_miss_stage(rank_row: Option<&LeafBlockRankRow>) -> &'static str {
    let Some(row) = rank_row else {
        return "attribution_missing";
    };
    match row.status.as_str() {
        "not_found_in_routed_leaves" | "nprobe_zero" => "routing_miss",
        "block_ranked" => match row.selected_by_global_cap {
            Some(false) => "block_pruned_global_cap",
            Some(true) => "candidate_or_rerank_cap",
            None => "candidate_or_rerank_cap",
        },
        _ => "attribution_unknown",
    }
}

#[derive(Debug, Serialize)]
struct FunnelRecord {
    kind: &'static str,
    nprobe: i32,
    query_ordinal: usize,
    query_id: i64,
    pipeline_stages: Vec<FunnelPipelineStageRecord>,
    leaf_route_count: i64,
    scanned_leaf_count: i64,
    candidate_count: i64,
    leaf_candidate_count: i64,
    primary_candidate_count: i64,
    boundary_replica_candidate_count: i64,
    retained_after_rerank_count: i64,
    returned_to_k_count: Option<usize>,
    deduped_candidate_count: i64,
    truncated_candidate_count: i64,
    candidate_winner_count: i64,
    leaf_candidate_mean: f64,
    leaf_candidate_p95: i64,
    leaf_candidate_max: i64,
    leaf_object_bytes: i64,
    leaf_summary_object_bytes: i64,
    leaf_row_object_bytes: i64,
    leaf_row_segment_read_count: i64,
    leaf_row_segment_read_bytes: i64,
    leaf_block_available_count: i64,
    leaf_block_selected_count: i64,
    leaf_block_skipped_count: i64,
    leaf_object_read_nanos: i64,
    leaf_summary_score_nanos: i64,
    leaf_row_score_nanos: i64,
    candidate_score_nanos: i64,
    candidate_materialize_nanos: i64,
    candidate_heap_append_nanos: i64,
    rerank_locality_candidate_count: i64,
    rerank_prefix_count: i64,
    rerank_unique_heap_block_count: i64,
    rerank_heap_block_transition_count: i64,
    rerank_heap_block_span: i64,
    rerank_heap_block_jump_sum: i64,
    rerank_heap_block_jump_max: i64,
}

impl FunnelRecord {
    fn from_query(
        nprobe: i32,
        query_ordinal: usize,
        query_id: i64,
        local_rows: &[LocalPipelineRow],
        leaf_rows: &[LeafCandidateRow],
        rerank_locality: Option<&RerankLocalityRow>,
        returned_to_k_count: Option<usize>,
    ) -> Result<Self> {
        let pipeline_stages = local_rows
            .iter()
            .map(FunnelPipelineStageRecord::from)
            .collect();
        let candidate_count = local_step_value(local_rows, "candidates", |row| row.candidate_count)
            .unwrap_or_else(|| leaf_rows.iter().map(|row| row.candidate_row_count).sum());
        let retained_after_rerank_count =
            local_step_value(local_rows, "heap_rerank", |row| row.heap_rerank_row_count)
                .unwrap_or(0);
        let leaf_route_count = leaf_rows.iter().map(|row| row.route_count).sum();
        let scanned_leaf_count = leaf_rows.iter().map(|row| row.scanned_count).sum();
        let leaf_candidate_count = leaf_rows.iter().map(|row| row.candidate_row_count).sum();
        let primary_candidate_count = leaf_rows
            .iter()
            .map(|row| row.primary_candidate_row_count)
            .sum();
        let boundary_replica_candidate_count = leaf_rows
            .iter()
            .map(|row| row.boundary_replica_candidate_row_count)
            .sum();
        let deduped_candidate_count = leaf_rows
            .iter()
            .map(|row| row.deduped_candidate_row_count)
            .sum();
        let truncated_candidate_count = leaf_rows
            .iter()
            .map(|row| row.truncated_candidate_row_count)
            .sum();
        let candidate_winner_count = leaf_rows.iter().map(|row| row.candidate_winner_count).sum();
        let leaf_object_bytes = leaf_rows.iter().map(|row| row.object_bytes).sum();
        let leaf_summary_object_bytes = leaf_rows
            .iter()
            .map(|row| row.leaf_summary_object_bytes)
            .sum();
        let leaf_row_object_bytes = leaf_rows.iter().map(|row| row.leaf_row_object_bytes).sum();
        let leaf_row_segment_read_count = leaf_rows
            .iter()
            .map(|row| row.leaf_row_segment_read_count)
            .sum();
        let leaf_row_segment_read_bytes = leaf_rows
            .iter()
            .map(|row| row.leaf_row_segment_read_bytes)
            .sum();
        let leaf_block_available_count = leaf_rows
            .iter()
            .map(|row| row.leaf_block_available_count)
            .sum();
        let leaf_block_selected_count = leaf_rows
            .iter()
            .map(|row| row.leaf_block_selected_count)
            .sum();
        let leaf_block_skipped_count = leaf_rows
            .iter()
            .map(|row| row.leaf_block_skipped_count)
            .sum();
        let leaf_object_read_nanos = leaf_rows.iter().map(|row| row.leaf_object_read_nanos).sum();
        let leaf_summary_score_nanos = leaf_rows
            .iter()
            .map(|row| row.leaf_summary_score_nanos)
            .sum();
        let leaf_row_score_nanos = leaf_rows.iter().map(|row| row.leaf_row_score_nanos).sum();
        let candidate_score_nanos = leaf_rows.iter().map(|row| row.candidate_score_nanos).sum();
        let candidate_materialize_nanos = leaf_rows
            .iter()
            .map(|row| row.candidate_materialize_nanos)
            .sum();
        let candidate_heap_append_nanos = leaf_rows
            .iter()
            .map(|row| row.candidate_heap_append_nanos)
            .sum();
        let mut per_leaf_candidates = leaf_rows
            .iter()
            .map(|row| row.candidate_row_count)
            .collect::<Vec<_>>();
        let leaf_candidate_mean = if per_leaf_candidates.is_empty() {
            0.0
        } else {
            per_leaf_candidates.iter().sum::<i64>() as f64 / per_leaf_candidates.len() as f64
        };
        let leaf_candidate_p95 = percentile_nearest_rank(&mut per_leaf_candidates, 95);
        let leaf_candidate_max = per_leaf_candidates.into_iter().max().unwrap_or(0);
        let rerank_locality_candidate_count = rerank_locality.map_or(0, |row| row.candidate_count);
        let rerank_prefix_count = rerank_locality.map_or(0, |row| row.rerank_prefix_count);
        let rerank_unique_heap_block_count =
            rerank_locality.map_or(0, |row| row.unique_heap_block_count);
        let rerank_heap_block_transition_count =
            rerank_locality.map_or(0, |row| row.heap_block_transition_count);
        let rerank_heap_block_span = rerank_locality.map_or(0, |row| row.heap_block_span);
        let rerank_heap_block_jump_sum = rerank_locality.map_or(0, |row| row.heap_block_jump_sum);
        let rerank_heap_block_jump_max = rerank_locality.map_or(0, |row| row.heap_block_jump_max);

        Ok(Self {
            kind: "spire_candidate_funnel",
            nprobe,
            query_ordinal,
            query_id,
            pipeline_stages,
            leaf_route_count,
            scanned_leaf_count,
            candidate_count,
            leaf_candidate_count,
            primary_candidate_count,
            boundary_replica_candidate_count,
            retained_after_rerank_count,
            returned_to_k_count,
            deduped_candidate_count,
            truncated_candidate_count,
            candidate_winner_count,
            leaf_candidate_mean,
            leaf_candidate_p95,
            leaf_candidate_max,
            leaf_object_bytes,
            leaf_summary_object_bytes,
            leaf_row_object_bytes,
            leaf_row_segment_read_count,
            leaf_row_segment_read_bytes,
            leaf_block_available_count,
            leaf_block_selected_count,
            leaf_block_skipped_count,
            leaf_object_read_nanos,
            leaf_summary_score_nanos,
            leaf_row_score_nanos,
            candidate_score_nanos,
            candidate_materialize_nanos,
            candidate_heap_append_nanos,
            rerank_locality_candidate_count,
            rerank_prefix_count,
            rerank_unique_heap_block_count,
            rerank_heap_block_transition_count,
            rerank_heap_block_span,
            rerank_heap_block_jump_sum,
            rerank_heap_block_jump_max,
        })
    }
}

#[derive(Debug, Default)]
struct LeafStageTotals {
    route_count: i64,
    scanned_count: i64,
    candidate_row_count: i64,
    leaf_block_available_count: i64,
    leaf_block_selected_count: i64,
    leaf_block_skipped_count: i64,
    leaf_object_bytes: i64,
    leaf_summary_object_bytes: i64,
    leaf_row_object_bytes: i64,
    leaf_row_segment_read_bytes: i64,
    leaf_object_read_nanos: i64,
    leaf_summary_score_nanos: i64,
    leaf_row_score_nanos: i64,
    candidate_materialize_nanos: i64,
    candidate_heap_append_nanos: i64,
}

impl LeafStageTotals {
    fn from_rows(rows: &[LeafCandidateRow]) -> Self {
        rows.iter().fold(Self::default(), |mut totals, row| {
            totals.route_count += row.route_count;
            totals.scanned_count += row.scanned_count;
            totals.candidate_row_count += row.candidate_row_count;
            totals.leaf_block_available_count += row.leaf_block_available_count;
            totals.leaf_block_selected_count += row.leaf_block_selected_count;
            totals.leaf_block_skipped_count += row.leaf_block_skipped_count;
            totals.leaf_object_bytes += row.object_bytes;
            totals.leaf_summary_object_bytes += row.leaf_summary_object_bytes;
            totals.leaf_row_object_bytes += row.leaf_row_object_bytes;
            totals.leaf_row_segment_read_bytes += row.leaf_row_segment_read_bytes;
            totals.leaf_object_read_nanos += row.leaf_object_read_nanos;
            totals.leaf_summary_score_nanos += row.leaf_summary_score_nanos;
            totals.leaf_row_score_nanos += row.leaf_row_score_nanos;
            totals.candidate_materialize_nanos += row.candidate_materialize_nanos;
            totals.candidate_heap_append_nanos += row.candidate_heap_append_nanos;
            totals
        })
    }
}

#[derive(Debug, Serialize)]
struct StageContainmentRecord {
    kind: &'static str,
    nprobe: i32,
    query_ordinal: usize,
    query_id: i64,
    stage_ordinal: u8,
    stage_name: &'static str,
    containment_basis: &'static str,
    truth_top_k_count: usize,
    contained_truth_count: usize,
    missing_truth_count: usize,
    contained_truth_ranks: Vec<usize>,
    missing_truth_ranks: Vec<usize>,
    missing_reason_counts: BTreeMap<String, usize>,
    candidate_or_object_count: i64,
    bytes_read_or_shipped: i64,
    leaf_object_bytes: i64,
    leaf_summary_object_bytes: i64,
    leaf_row_object_bytes: i64,
    leaf_row_segment_read_bytes: i64,
    leaf_block_available_count: i64,
    leaf_block_selected_count: i64,
    leaf_block_skipped_count: i64,
    latency_nanos: Option<i64>,
    route_count: i64,
    candidate_count: i64,
    heap_rerank_row_count: i64,
    remote_fanout_count: i64,
    budget_count: Option<i64>,
    blocked_count: i64,
    next_blocker: String,
    recommendation: String,
}

impl StageContainmentRecord {
    #[allow(clippy::too_many_arguments)]
    fn from_query(
        nprobe: i32,
        query_ordinal: usize,
        query_id: i64,
        truth_ids: &[i64],
        predicted_ids: &[i64],
        local_rows: &[LocalPipelineRow],
        leaf_rows: &[LeafCandidateRow],
        rerank_locality: Option<&RerankLocalityRow>,
        rank_rows: &[LeafBlockRankRow],
    ) -> Result<Vec<Self>> {
        let rank_by_ordinal = rank_rows
            .iter()
            .map(|row| {
                let ordinal = usize::try_from(row.target_ordinal).map_err(|_| {
                    eyre!(
                        "stage containment target ordinal {} exceeds usize",
                        row.target_ordinal
                    )
                })?;
                Ok((ordinal, row))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        let predicted_id_set = predicted_ids.iter().copied().collect::<HashSet<_>>();
        let totals = LeafStageTotals::from_rows(leaf_rows);
        let routing_step = local_step(local_rows, "routing");
        let placement_step = local_step(local_rows, "placement");
        let prefetch_step = local_step(local_rows, "prefetch");
        let candidate_step = local_step(local_rows, "candidates");
        let rerank_step = local_step(local_rows, "heap_rerank");

        let mut rows = Vec::with_capacity(6);
        rows.push(Self::build(
            nprobe,
            query_ordinal,
            query_id,
            1,
            "topology_route_set",
            "target_block_rank_routed_leaf_lookup",
            truth_ids,
            |truth_index, _truth_id| {
                target_rank_row_is_routed(rank_by_ordinal.get(&truth_index).copied())
            },
            |truth_index, _truth_id| {
                stage_missing_reason_for_route(rank_by_ordinal.get(&truth_index).copied())
            },
            routing_step,
            routing_step.map_or(0, |row| row.route_count),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            None,
        ));
        rows.push(Self::build(
            nprobe,
            query_ordinal,
            query_id,
            2,
            "selected_leaves",
            "target_block_rank_routed_leaf_lookup",
            truth_ids,
            |truth_index, _truth_id| {
                target_rank_row_is_routed(rank_by_ordinal.get(&truth_index).copied())
            },
            |truth_index, _truth_id| {
                stage_missing_reason_for_route(rank_by_ordinal.get(&truth_index).copied())
            },
            placement_step.or(prefetch_step),
            totals.scanned_count,
            totals.leaf_object_bytes,
            totals.leaf_object_bytes,
            0,
            0,
            0,
            0,
            0,
            0,
            Some(totals.leaf_object_read_nanos),
        ));
        rows.push(Self::build(
            nprobe,
            query_ordinal,
            query_id,
            3,
            "selected_leaf_blocks",
            "target_block_rank_selected_by_global_cap",
            truth_ids,
            |truth_index, _truth_id| {
                target_rank_row_block_selected(rank_by_ordinal.get(&truth_index).copied())
            },
            |truth_index, _truth_id| {
                stage_missing_reason_for_block(rank_by_ordinal.get(&truth_index).copied())
            },
            candidate_step,
            totals.leaf_block_selected_count,
            totals.leaf_summary_object_bytes,
            totals.leaf_object_bytes,
            totals.leaf_summary_object_bytes,
            totals.leaf_row_object_bytes,
            totals.leaf_row_segment_read_bytes,
            totals.leaf_block_available_count,
            totals.leaf_block_selected_count,
            totals.leaf_block_skipped_count,
            Some(totals.leaf_summary_score_nanos),
        ));
        rows.push(Self::build(
            nprobe,
            query_ordinal,
            query_id,
            4,
            "local_candidate_frontier",
            "final_hits_lower_bound_until_target_candidate_rank_snapshot",
            truth_ids,
            |_truth_index, truth_id| predicted_id_set.contains(truth_id),
            |truth_index, truth_id| {
                stage_missing_reason_for_candidate_frontier(
                    rank_by_ordinal.get(&truth_index).copied(),
                    predicted_id_set.contains(truth_id),
                )
            },
            candidate_step,
            candidate_step.map_or(totals.candidate_row_count, |row| row.candidate_count),
            totals.leaf_row_segment_read_bytes,
            totals.leaf_object_bytes,
            totals.leaf_summary_object_bytes,
            totals.leaf_row_object_bytes,
            totals.leaf_row_segment_read_bytes,
            totals.leaf_block_available_count,
            totals.leaf_block_selected_count,
            totals.leaf_block_skipped_count,
            Some(
                totals.leaf_row_score_nanos
                    + totals.candidate_materialize_nanos
                    + totals.candidate_heap_append_nanos,
            ),
        ));
        rows.push(Self::build(
            nprobe,
            query_ordinal,
            query_id,
            5,
            "exact_source_rerank_frontier",
            "final_hits_lower_bound_until_target_candidate_rank_snapshot",
            truth_ids,
            |_truth_index, truth_id| predicted_id_set.contains(truth_id),
            |truth_index, truth_id| {
                stage_missing_reason_for_rerank_frontier(
                    rank_by_ordinal.get(&truth_index).copied(),
                    predicted_id_set.contains(truth_id),
                )
            },
            rerank_step,
            rerank_locality.map_or(0, |row| row.rerank_prefix_count),
            0,
            totals.leaf_object_bytes,
            totals.leaf_summary_object_bytes,
            totals.leaf_row_object_bytes,
            totals.leaf_row_segment_read_bytes,
            totals.leaf_block_available_count,
            totals.leaf_block_selected_count,
            totals.leaf_block_skipped_count,
            None,
        ));
        rows.push(Self::build(
            nprobe,
            query_ordinal,
            query_id,
            6,
            "final_top_k",
            "query_metric_returned_ids",
            truth_ids,
            |_truth_index, truth_id| predicted_id_set.contains(truth_id),
            |truth_index, truth_id| {
                stage_missing_reason_for_final(
                    rank_by_ordinal.get(&truth_index).copied(),
                    predicted_id_set.contains(truth_id),
                )
            },
            rerank_step,
            i64::try_from(predicted_ids.len())
                .map_err(|_| eyre!("predicted id count exceeds i64"))?,
            0,
            totals.leaf_object_bytes,
            totals.leaf_summary_object_bytes,
            totals.leaf_row_object_bytes,
            totals.leaf_row_segment_read_bytes,
            totals.leaf_block_available_count,
            totals.leaf_block_selected_count,
            totals.leaf_block_skipped_count,
            None,
        ));
        Ok(rows)
    }

    #[allow(clippy::too_many_arguments)]
    fn build<C, R>(
        nprobe: i32,
        query_ordinal: usize,
        query_id: i64,
        stage_ordinal: u8,
        stage_name: &'static str,
        containment_basis: &'static str,
        truth_ids: &[i64],
        contains_truth: C,
        missing_reason: R,
        pipeline_stage: Option<&LocalPipelineRow>,
        candidate_or_object_count: i64,
        bytes_read_or_shipped: i64,
        leaf_object_bytes: i64,
        leaf_summary_object_bytes: i64,
        leaf_row_object_bytes: i64,
        leaf_row_segment_read_bytes: i64,
        leaf_block_available_count: i64,
        leaf_block_selected_count: i64,
        leaf_block_skipped_count: i64,
        latency_nanos: Option<i64>,
    ) -> Self
    where
        C: Fn(usize, &i64) -> bool,
        R: Fn(usize, &i64) -> &'static str,
    {
        let mut contained_truth_ranks = Vec::new();
        let mut missing_truth_ranks = Vec::new();
        let mut missing_reason_counts = BTreeMap::<String, usize>::new();
        for (truth_index, truth_id) in truth_ids.iter().enumerate() {
            let truth_rank = truth_index + 1;
            if contains_truth(truth_index, truth_id) {
                contained_truth_ranks.push(truth_rank);
            } else {
                missing_truth_ranks.push(truth_rank);
                *missing_reason_counts
                    .entry(missing_reason(truth_index, truth_id).to_owned())
                    .or_default() += 1;
            }
        }

        Self {
            kind: "spire_stage_containment",
            nprobe,
            query_ordinal,
            query_id,
            stage_ordinal,
            stage_name,
            containment_basis,
            truth_top_k_count: truth_ids.len(),
            contained_truth_count: contained_truth_ranks.len(),
            missing_truth_count: missing_truth_ranks.len(),
            contained_truth_ranks,
            missing_truth_ranks,
            missing_reason_counts,
            candidate_or_object_count,
            bytes_read_or_shipped,
            leaf_object_bytes,
            leaf_summary_object_bytes,
            leaf_row_object_bytes,
            leaf_row_segment_read_bytes,
            leaf_block_available_count,
            leaf_block_selected_count,
            leaf_block_skipped_count,
            latency_nanos,
            route_count: pipeline_stage.map_or(0, |row| row.route_count),
            candidate_count: pipeline_stage.map_or(0, |row| row.candidate_count),
            heap_rerank_row_count: pipeline_stage.map_or(0, |row| row.heap_rerank_row_count),
            remote_fanout_count: pipeline_stage.map_or(0, |row| row.remote_fanout_count),
            budget_count: pipeline_stage.map(|row| row.item_count),
            blocked_count: pipeline_stage.map_or(0, |row| row.blocked_count),
            next_blocker: pipeline_stage
                .map(|row| row.next_blocker.clone())
                .unwrap_or_else(|| "none".to_owned()),
            recommendation: pipeline_stage
                .map(|row| row.recommendation.clone())
                .unwrap_or_else(|| "none".to_owned()),
        }
    }
}

fn local_step<'a>(rows: &'a [LocalPipelineRow], step_name: &str) -> Option<&'a LocalPipelineRow> {
    rows.iter().find(|row| row.step_name == step_name)
}

fn target_rank_row_is_routed(row: Option<&LeafBlockRankRow>) -> bool {
    row.is_some_and(|row| {
        !matches!(
            row.status.as_str(),
            "not_found_in_routed_leaves" | "nprobe_zero"
        )
    })
}

fn target_rank_row_block_selected(row: Option<&LeafBlockRankRow>) -> bool {
    row.is_some_and(|row| {
        matches!(row.status.as_str(), "block_ranked" | "target_block_ranked")
            && row.selected_by_global_cap != Some(false)
    })
}

fn stage_missing_reason_for_route(row: Option<&LeafBlockRankRow>) -> &'static str {
    match row.map(|row| row.status.as_str()) {
        Some("nprobe_zero") => "nprobe_zero",
        Some("not_found_in_routed_leaves") => "routing_miss",
        Some(_) => "contained",
        None => "rank_row_missing",
    }
}

fn stage_missing_reason_for_block(row: Option<&LeafBlockRankRow>) -> &'static str {
    if !target_rank_row_is_routed(row) {
        return stage_missing_reason_for_route(row);
    }
    match row.and_then(|row| row.selected_by_global_cap) {
        Some(false) => "block_pruned_global_cap",
        _ if target_rank_row_block_selected(row) => "contained",
        _ => "block_rank_missing",
    }
}

fn stage_missing_reason_for_candidate_frontier(
    row: Option<&LeafBlockRankRow>,
    final_hit: bool,
) -> &'static str {
    if final_hit {
        return "contained";
    }
    if !target_rank_row_block_selected(row) {
        return stage_missing_reason_for_block(row);
    }
    "candidate_or_later_cap"
}

fn stage_missing_reason_for_rerank_frontier(
    row: Option<&LeafBlockRankRow>,
    final_hit: bool,
) -> &'static str {
    if final_hit {
        return "contained";
    }
    if !target_rank_row_block_selected(row) {
        return stage_missing_reason_for_block(row);
    }
    "rerank_or_final_topk_cap"
}

fn stage_missing_reason_for_final(row: Option<&LeafBlockRankRow>, final_hit: bool) -> &'static str {
    if final_hit {
        return "contained";
    }
    if !target_rank_row_block_selected(row) {
        return stage_missing_reason_for_block(row);
    }
    "not_returned_in_topk"
}

fn local_step_value<F>(rows: &[LocalPipelineRow], step_name: &str, value: F) -> Option<i64>
where
    F: Fn(&LocalPipelineRow) -> i64,
{
    rows.iter()
        .find(|row| row.step_name == step_name)
        .map(value)
}

fn percentile_nearest_rank(values: &mut [i64], percentile: usize) -> i64 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    let numerator = percentile.saturating_mul(values.len()).saturating_add(99);
    let rank = numerator / 100;
    values[rank.saturating_sub(1).min(values.len() - 1)]
}

async fn write_funnel_jsonl(path: &PathBuf, rows: &[FunnelRecord]) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .wrap_err_with(|| format!("creating {}", parent.display()))?;
    }
    let mut output = String::new();
    for row in rows {
        output.push_str(&serde_json::to_string(row).wrap_err("serializing funnel row")?);
        output.push('\n');
    }
    tokio::fs::write(path, output)
        .await
        .wrap_err_with(|| format!("writing {}", path.display()))
}

async fn write_stage_containment_jsonl(
    path: &PathBuf,
    rows: &[StageContainmentRecord],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .wrap_err_with(|| format!("creating {}", parent.display()))?;
    }
    let mut output = String::new();
    for row in rows {
        output.push_str(&serde_json::to_string(row).wrap_err("serializing containment row")?);
        output.push('\n');
    }
    tokio::fs::write(path, output)
        .await
        .wrap_err_with(|| format!("writing {}", path.display()))
}

async fn write_leaf_block_rank_jsonl(path: &PathBuf, rows: &[LeafBlockRankRecord]) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .wrap_err_with(|| format!("creating {}", parent.display()))?;
    }
    let mut output = String::new();
    for row in rows {
        output.push_str(&serde_json::to_string(row).wrap_err("serializing leaf block rank row")?);
        output.push('\n');
    }
    tokio::fs::write(path, output)
        .await
        .wrap_err_with(|| format!("writing {}", path.display()))
}

async fn write_miss_attribution_jsonl(
    path: &PathBuf,
    rows: &[MissAttributionRecord],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .wrap_err_with(|| format!("creating {}", parent.display()))?;
    }
    let mut output = String::new();
    for row in rows {
        output.push_str(&serde_json::to_string(row).wrap_err("serializing miss attribution row")?);
        output.push('\n');
    }
    tokio::fs::write(path, output)
        .await
        .wrap_err_with(|| format!("writing {}", path.display()))
}

#[derive(Debug)]
struct RemotePipelineRow {
    step_ordinal: i64,
    step_name: String,
    status: String,
    item_count: i64,
    ready_count: i64,
    blocked_count: i64,
    remote_pid_count: i64,
    next_blocker: String,
}

impl From<Row> for RemotePipelineRow {
    fn from(row: Row) -> Self {
        Self {
            step_ordinal: row.get(0),
            step_name: row.get(1),
            status: row.get(3),
            item_count: row.get(4),
            ready_count: row.get(5),
            blocked_count: row.get(6),
            remote_pid_count: row.get(7),
            next_blocker: row.get(8),
        }
    }
}

#[derive(Debug)]
struct DegradedSkipRow {
    requested_epoch: i64,
    node_id: i64,
    skipped_pid_count: i64,
    first_skip_category: String,
    status: String,
}

impl From<Row> for DegradedSkipRow {
    fn from(row: Row) -> Self {
        Self {
            requested_epoch: row.get(0),
            node_id: row.get(1),
            skipped_pid_count: row.get(2),
            first_skip_category: row.get(3),
            status: row.get(4),
        }
    }
}

#[derive(Debug, Default)]
struct ProductionReadProfileRow {
    values: BTreeMap<String, String>,
}

impl ProductionReadProfileRow {
    fn from_metric_rows(rows: Vec<Row>) -> Self {
        let values = rows
            .into_iter()
            .map(|row| (row.get::<_, String>(0), row.get::<_, String>(1)))
            .collect();
        Self { values }
    }

    fn string_metric(&self, metric: &str) -> String {
        self.values
            .get(metric)
            .cloned()
            .unwrap_or_else(|| "missing".to_owned())
    }

    fn i64_metric(&self, metric: &str) -> i64 {
        self.values
            .get(metric)
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(0)
    }

    fn duration_metric(&self, metric: &str) -> Duration {
        let millis = self.i64_metric(metric).max(0) as u64;
        Duration::from_millis(millis)
    }
}

#[derive(Debug)]
struct EndpointIdentityRow {
    tuple_transport_capabilities: Vec<String>,
    tuple_transport_default: String,
    tuple_transport_status: String,
    status: String,
    recommendation: String,
}

#[derive(Debug)]
struct CostTuningRow {
    storage_format: String,
    effective_rerank_width: i32,
    cost_routing_dimension_scale: f64,
    cost_leaf_dimension_scale: f64,
    cost_index_page_scale: f64,
    cost_local_store_page_fanout_scale: f64,
    cost_storage_scoring_multiplier: f64,
    effective_storage_scoring_multiplier: f64,
    cost_rerank_multiplier: f64,
    effective_rerank_multiplier: f64,
}

impl From<Row> for CostTuningRow {
    fn from(row: Row) -> Self {
        Self {
            storage_format: row.get(0),
            effective_rerank_width: row.get(1),
            cost_routing_dimension_scale: row.get(2),
            cost_leaf_dimension_scale: row.get(3),
            cost_index_page_scale: row.get(4),
            cost_local_store_page_fanout_scale: row.get(5),
            cost_storage_scoring_multiplier: row.get(6),
            effective_storage_scoring_multiplier: row.get(7),
            cost_rerank_multiplier: row.get(8),
            effective_rerank_multiplier: row.get(9),
        }
    }
}

impl EndpointIdentityRow {
    fn pg_binary_attr_v1_ready(&self) -> bool {
        self.tuple_transport_capabilities
            .iter()
            .any(|capability| capability == "pg_binary_attr_v1")
            && self.tuple_transport_default == "pg_binary_attr_v1"
            && self.tuple_transport_status == "ready"
    }
}

impl From<Row> for EndpointIdentityRow {
    fn from(row: Row) -> Self {
        Self {
            tuple_transport_capabilities: row.get(0),
            tuple_transport_default: row.get(1),
            tuple_transport_status: row.get(2),
            status: row.get(3),
            recommendation: row.get(4),
        }
    }
}

#[derive(Debug)]
struct LocalStoreOverlapRow {
    node_id: i64,
    local_store_id: i64,
    route_count: i64,
    leaf_route_count: i64,
    delta_route_count: i64,
    candidate_row_count: i64,
    prefetched_object_bytes: i64,
    read_batch_count: i64,
    delta_decode_count: i64,
}

impl From<Row> for LocalStoreOverlapRow {
    fn from(row: Row) -> Self {
        Self {
            node_id: row.get(0),
            local_store_id: row.get(1),
            route_count: row.get(2),
            leaf_route_count: row.get(3),
            delta_route_count: row.get(4),
            candidate_row_count: row.get(5),
            prefetched_object_bytes: row.get(6),
            read_batch_count: row.get(7),
            delta_decode_count: row.get(8),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RoutingKey {
    nprobe: i32,
    routing_level: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StepKey {
    nprobe: i32,
    step_ordinal: i64,
    step_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct LocalStoreOverlapKey {
    nprobe: i32,
    node_id: i64,
    local_store_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DegradedSkipKey {
    nprobe: i32,
    node_id: i64,
}

#[derive(Debug, Default)]
struct RoutingAggregate {
    queries: usize,
    effective_nprobe: MixedValue,
    effective_nprobe_source: MixedValue,
    adaptive_nprobe_decision: MixedValue,
    recursive_beam_width: MixedValue,
    max_leaf_routes: MixedValue,
    max_routing_expansions: MixedValue,
    input_frontier_width_sum: i64,
    expanded_parent_count_sum: i64,
    selected_child_count_sum: i64,
    deduped_route_count_sum: i64,
    truncation_reason: MixedValue,
}

impl RoutingAggregate {
    fn record(&mut self, row: RoutingRow) {
        self.queries += 1;
        self.effective_nprobe
            .record(row.effective_nprobe.to_string());
        self.effective_nprobe_source
            .record(row.effective_nprobe_source);
        self.adaptive_nprobe_decision
            .record(row.adaptive_nprobe_decision);
        self.recursive_beam_width
            .record(row.recursive_beam_width.to_string());
        self.max_leaf_routes.record(row.max_leaf_routes.to_string());
        self.max_routing_expansions
            .record(row.max_routing_expansions.to_string());
        self.input_frontier_width_sum += row.input_frontier_width;
        self.expanded_parent_count_sum += row.expanded_parent_count;
        self.selected_child_count_sum += row.selected_child_count;
        self.deduped_route_count_sum += row.deduped_route_count;
        self.truncation_reason.record(row.truncation_reason);
    }
}

#[derive(Debug, Default)]
struct LocalStepAggregate {
    queries: usize,
    status: MixedValue,
    item_count_sum: i64,
    ready_count_sum: i64,
    blocked_count_sum: i64,
    route_count_sum: i64,
    candidate_count_sum: i64,
    heap_rerank_row_count_sum: i64,
    remote_fanout_count_sum: i64,
    next_blocker: MixedValue,
}

impl LocalStepAggregate {
    fn record(&mut self, row: LocalPipelineRow) {
        self.queries += 1;
        self.status.record(row.status);
        self.item_count_sum += row.item_count;
        self.ready_count_sum += row.ready_count;
        self.blocked_count_sum += row.blocked_count;
        self.route_count_sum += row.route_count;
        self.candidate_count_sum += row.candidate_count;
        self.heap_rerank_row_count_sum += row.heap_rerank_row_count;
        self.remote_fanout_count_sum += row.remote_fanout_count;
        self.next_blocker.record(row.next_blocker);
    }
}

#[derive(Debug, Default)]
struct RemoteStepAggregate {
    queries: usize,
    status: MixedValue,
    item_count_sum: i64,
    ready_count_sum: i64,
    blocked_count_sum: i64,
    remote_pid_count_sum: i64,
    next_blocker: MixedValue,
}

impl RemoteStepAggregate {
    fn record(&mut self, row: RemotePipelineRow) {
        self.queries += 1;
        self.status.record(row.status);
        self.item_count_sum += row.item_count;
        self.ready_count_sum += row.ready_count;
        self.blocked_count_sum += row.blocked_count;
        self.remote_pid_count_sum += row.remote_pid_count;
        self.next_blocker.record(row.next_blocker);
    }
}

#[derive(Debug, Default)]
struct DegradedSkipAggregate {
    reports: usize,
    requested_epoch: MixedValue,
    skipped_pid_count_sum: i64,
    first_skip_category: MixedValue,
    status: MixedValue,
}

impl DegradedSkipAggregate {
    fn record(&mut self, row: DegradedSkipRow) {
        self.reports += 1;
        self.requested_epoch.record(row.requested_epoch.to_string());
        self.skipped_pid_count_sum += row.skipped_pid_count;
        self.first_skip_category.record(row.first_skip_category);
        self.status.record(row.status);
    }
}

#[derive(Debug, Default)]
struct LocalStoreOverlapAggregate {
    queries: usize,
    route_count_sum: i64,
    leaf_route_count_sum: i64,
    delta_route_count_sum: i64,
    candidate_row_count_sum: i64,
    prefetched_object_bytes_sum: i64,
    read_batch_count_sum: i64,
    delta_decode_count_sum: i64,
}

impl LocalStoreOverlapAggregate {
    fn record(&mut self, row: LocalStoreOverlapRow) {
        self.queries += 1;
        self.route_count_sum += row.route_count;
        self.leaf_route_count_sum += row.leaf_route_count;
        self.delta_route_count_sum += row.delta_route_count;
        self.candidate_row_count_sum += row.candidate_row_count;
        self.prefetched_object_bytes_sum += row.prefetched_object_bytes;
        self.read_batch_count_sum += row.read_batch_count;
        self.delta_decode_count_sum += row.delta_decode_count;
    }
}

#[derive(Debug, Default)]
struct QueryMetricAggregate {
    durations: Vec<Duration>,
    predicted_ids: Vec<Vec<i64>>,
    recall_at_k: Option<f64>,
}

impl QueryMetricAggregate {
    fn record(&mut self, duration: Duration, predicted_ids: Vec<i64>) {
        self.durations.push(duration);
        self.predicted_ids.push(predicted_ids);
    }

    fn record_recall(&mut self, truth_ids: &[Vec<i64>], k: usize) {
        self.recall_at_k = Some(super::recall::recall_at_k(
            truth_ids,
            &self.predicted_ids,
            k,
        ));
    }

    fn latency_stats(&self) -> DurationStats {
        summarize_durations(&self.durations)
    }
}

#[derive(Debug, Default)]
struct ProductionReadProfileAggregate {
    profiles: usize,
    status: MixedValue,
    result_source: MixedValue,
    selected_pid_count_sum: i64,
    remote_pid_count_sum: i64,
    dispatch_count_sum: i64,
    socket_open_count_sum: i64,
    candidate_receive_query_count_sum: i64,
    heap_receive_query_count_sum: i64,
    endpoint_identity_query_count_sum: i64,
    payload_decode_bytes_sum: i64,
    remote_timeout_count_sum: i64,
    remote_cancel_count_sum: i64,
    degraded_skipped_dispatch_count_sum: i64,
    returned_candidate_count_sum: i64,
    connect_elapsed: Vec<Duration>,
    endpoint_identity_elapsed: Vec<Duration>,
    candidate_receive_elapsed: Vec<Duration>,
    heap_receive_elapsed: Vec<Duration>,
    merge_elapsed: Vec<Duration>,
    total_elapsed: Vec<Duration>,
}

impl ProductionReadProfileAggregate {
    fn record(&mut self, row: ProductionReadProfileRow) {
        self.profiles += 1;
        self.status.record(row.string_metric("status"));
        self.result_source
            .record(row.string_metric("result_source"));
        self.selected_pid_count_sum += row.i64_metric("selected_pid_count");
        self.remote_pid_count_sum += row.i64_metric("remote_pid_count");
        self.dispatch_count_sum += row.i64_metric("dispatch_count");
        self.socket_open_count_sum += row.i64_metric("socket_open_count");
        self.candidate_receive_query_count_sum += row.i64_metric("candidate_receive_query_count");
        self.heap_receive_query_count_sum += row.i64_metric("heap_receive_query_count");
        self.endpoint_identity_query_count_sum += row.i64_metric("endpoint_identity_query_count");
        self.payload_decode_bytes_sum += row.i64_metric("payload_decode_bytes");
        self.remote_timeout_count_sum += row.i64_metric("remote_timeout_count");
        self.remote_cancel_count_sum += row.i64_metric("remote_cancel_count");
        self.degraded_skipped_dispatch_count_sum +=
            row.i64_metric("degraded_skipped_dispatch_count");
        self.returned_candidate_count_sum += row.i64_metric("returned_candidate_count");
        self.connect_elapsed
            .push(row.duration_metric("connect_elapsed_ms"));
        self.endpoint_identity_elapsed
            .push(row.duration_metric("endpoint_identity_elapsed_ms"));
        self.candidate_receive_elapsed
            .push(row.duration_metric("candidate_receive_elapsed_ms"));
        self.heap_receive_elapsed
            .push(row.duration_metric("heap_receive_elapsed_ms"));
        self.merge_elapsed
            .push(row.duration_metric("merge_elapsed_ms"));
        self.total_elapsed
            .push(row.duration_metric("total_elapsed_ms"));
    }
}

#[derive(Debug, Clone, Copy)]
struct DurationStats {
    count: usize,
    min: Duration,
    p50: Duration,
    p95: Duration,
    p99: Duration,
    max: Duration,
}

fn summarize_durations(durations: &[Duration]) -> DurationStats {
    if durations.is_empty() {
        return DurationStats {
            count: 0,
            min: Duration::ZERO,
            p50: Duration::ZERO,
            p95: Duration::ZERO,
            p99: Duration::ZERO,
            max: Duration::ZERO,
        };
    }
    let mut sorted = durations.to_vec();
    sorted.sort_unstable();
    DurationStats {
        count: sorted.len(),
        min: sorted[0],
        p50: percentile_duration(&sorted, 0.50),
        p95: percentile_duration(&sorted, 0.95),
        p99: percentile_duration(&sorted, 0.99),
        max: sorted[sorted.len() - 1],
    }
}

fn percentile_duration(sorted: &[Duration], percentile: f64) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let idx = ((sorted.len() - 1) as f64 * percentile).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn format_duration_ms(duration: Duration) -> String {
    format!("{:.3} ms", duration.as_secs_f64() * 1000.0)
}

#[derive(Debug, Default)]
struct MixedValue {
    value: Option<String>,
    mixed: bool,
}

impl MixedValue {
    fn record(&mut self, value: String) {
        if let Some(existing) = &self.value {
            if existing != &value {
                self.mixed = true;
            }
        } else {
            self.value = Some(value);
        }
    }

    fn label(&self) -> String {
        if self.mixed {
            "mixed".to_owned()
        } else {
            self.value.clone().unwrap_or_else(|| "none".to_owned())
        }
    }
}

struct ReportInput<'a> {
    prefix: &'a str,
    index: &'a str,
    queries: usize,
    sweep_values: &'a [i32],
    rerank_width: Option<i32>,
    max_candidate_rows: Option<i32>,
    max_routed_candidate_rows: Option<i32>,
    remote_tuple_transport: Option<SpireRemoteTupleTransportMode>,
    endpoint_identity: &'a EndpointIdentityRow,
    adaptive_nprobe_options: super::AdaptiveNprobeBenchOptions,
    cost_snapshot_enabled: bool,
    cost_tuning: &'a BTreeMap<i32, CostTuningRow>,
    remote_enabled: bool,
    remote_selected_pids: &'a [i64],
    remote_epoch: Option<i64>,
    query_metrics_enabled: bool,
    include_recall: bool,
    query_metric_k: usize,
    query_metric_projection_columns: &'a [String],
    production_read_profile_enabled: bool,
    production_read_only: bool,
    local_store_overlap_enabled: bool,
    routing: &'a BTreeMap<RoutingKey, RoutingAggregate>,
    local: &'a BTreeMap<StepKey, LocalStepAggregate>,
    remote: &'a BTreeMap<StepKey, RemoteStepAggregate>,
    local_store_overlap: &'a BTreeMap<LocalStoreOverlapKey, LocalStoreOverlapAggregate>,
    degraded_skip: &'a BTreeMap<DegradedSkipKey, DegradedSkipAggregate>,
    query_metrics: &'a BTreeMap<i32, QueryMetricAggregate>,
    production_read_profile: &'a BTreeMap<i32, ProductionReadProfileAggregate>,
}

fn render_report(input: ReportInput<'_>) -> String {
    let mut sections = vec![render_header(&input)];
    sections.push(render_endpoint_identity_table(input.endpoint_identity));
    if input.cost_snapshot_enabled {
        sections.push(render_cost_tuning_table(input.cost_tuning));
    }
    sections.push(render_routing_table(input.routing));
    sections.push(render_local_table(input.local));
    if input.remote_enabled {
        sections.push(render_remote_table(input.remote));
        sections.push(render_degraded_skip_table(input.degraded_skip));
    }
    if input.local_store_overlap_enabled {
        sections.push(render_local_store_overlap_table(input.local_store_overlap));
    }
    if input.query_metrics_enabled {
        sections.push(render_query_metrics_table(
            input.query_metrics,
            input.include_recall,
        ));
    }
    if input.production_read_profile_enabled {
        sections.push(render_production_read_profile_table(
            input.production_read_profile,
        ));
    }
    sections.join("\n\n")
}

fn render_header(input: &ReportInput<'_>) -> String {
    let adaptive = if input.adaptive_nprobe_options.enabled {
        match input.adaptive_nprobe_options.score_gap_micros {
            Some(value) => format!("on gap_micros={value}"),
            None => "on".to_owned(),
        }
    } else {
        "off".to_owned()
    };
    format!(
        "SPIRE pipeline benchmark\nprefix: {prefix}\nindex: {index}\nqueries: {queries}\nsweep: {sweep:?}\nrerank_width: {rerank_width}\nmax_candidate_rows: {max_candidate_rows}\nmax_routed_candidate_rows: {max_routed_candidate_rows}\nremote_tuple_transport: {remote_tuple_transport}\nadaptive_nprobe: {adaptive}\ncost_snapshot: {cost_snapshot}\nremote: {remote}\nremote_selected_pids: {remote_selected_pids:?}\nremote_requested_epoch: {remote_epoch}\nlocal_store_overlap: {local_store_overlap}\nquery_metrics: {query_metrics}\nquery_metric_k: {query_metric_k}\nquery_metric_projection_columns: {query_metric_projection_columns}\nquery_recall: {query_recall}\nproduction_read_profile: {production_read_profile}\nproduction_read_only: {production_read_only}",
        prefix = input.prefix,
        index = input.index,
        queries = input.queries,
        sweep = input.sweep_values,
        rerank_width = option_label(input.rerank_width),
        max_candidate_rows = option_label(input.max_candidate_rows),
        max_routed_candidate_rows = option_label(input.max_routed_candidate_rows),
        remote_tuple_transport = option_label(input.remote_tuple_transport),
        cost_snapshot = input.cost_snapshot_enabled,
        remote = input.remote_enabled,
        remote_selected_pids = input.remote_selected_pids,
        remote_epoch = option_label(input.remote_epoch),
        local_store_overlap = input.local_store_overlap_enabled,
        query_metrics = input.query_metrics_enabled,
        query_metric_k = input.query_metric_k,
        query_metric_projection_columns = if input.query_metric_projection_columns.is_empty() {
            "id".to_owned()
        } else {
            format!("id,{}", input.query_metric_projection_columns.join(","))
        },
        query_recall = input.include_recall,
        production_read_profile = input.production_read_profile_enabled,
        production_read_only = input.production_read_only,
    )
}

fn render_cost_tuning_table(rows: &BTreeMap<i32, CostTuningRow>) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "nprobe",
        "storage",
        "rerank_width",
        "routing_dim",
        "leaf_dim",
        "page",
        "store_fanout",
        "storage_guc",
        "storage_effective",
        "rerank_guc",
        "rerank_effective",
    ]);
    for (nprobe, row) in rows {
        table.add_row(vec![
            Cell::new(nprobe),
            Cell::new(&row.storage_format),
            Cell::new(row.effective_rerank_width),
            Cell::new(format_cost_scale(row.cost_routing_dimension_scale)),
            Cell::new(format_cost_scale(row.cost_leaf_dimension_scale)),
            Cell::new(format_cost_scale(row.cost_index_page_scale)),
            Cell::new(format_cost_scale(row.cost_local_store_page_fanout_scale)),
            Cell::new(format_cost_scale(row.cost_storage_scoring_multiplier)),
            Cell::new(format_cost_scale(row.effective_storage_scoring_multiplier)),
            Cell::new(format_cost_scale(row.cost_rerank_multiplier)),
            Cell::new(format_cost_scale(row.effective_rerank_multiplier)),
        ]);
    }
    format!("Cost tuning snapshot\n{table}")
}

fn format_cost_scale(value: f64) -> String {
    format!("{value:.6}")
}

fn render_endpoint_identity_table(row: &EndpointIdentityRow) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["field", "value"]);
    let capabilities = if row.tuple_transport_capabilities.is_empty() {
        "none".to_owned()
    } else {
        row.tuple_transport_capabilities.join(",")
    };
    table.add_row(vec![
        Cell::new("tuple_transport_capabilities"),
        Cell::new(capabilities),
    ]);
    table.add_row(vec![
        Cell::new("tuple_transport_default"),
        Cell::new(&row.tuple_transport_default),
    ]);
    table.add_row(vec![
        Cell::new("tuple_transport_status"),
        Cell::new(&row.tuple_transport_status),
    ]);
    table.add_row(vec![
        Cell::new("pg_binary_attr_v1_ready"),
        Cell::new(row.pg_binary_attr_v1_ready()),
    ]);
    table.add_row(vec![Cell::new("status"), Cell::new(&row.status)]);
    table.add_row(vec![
        Cell::new("recommendation"),
        Cell::new(&row.recommendation),
    ]);
    format!("Endpoint tuple transport identity\n{table}")
}

fn render_routing_table(rows: &BTreeMap<RoutingKey, RoutingAggregate>) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "nprobe",
        "level",
        "queries",
        "effective_nprobe",
        "source",
        "adaptive",
        "beam_width",
        "max_leaf_routes",
        "max_routing_expansions",
        "input_frontier_sum",
        "expanded_parent_sum",
        "selected_child_sum",
        "deduped_route_sum",
        "truncation",
    ]);
    for (key, aggregate) in rows {
        table.add_row(vec![
            Cell::new(key.nprobe),
            Cell::new(key.routing_level),
            Cell::new(aggregate.queries),
            Cell::new(aggregate.effective_nprobe.label()),
            Cell::new(aggregate.effective_nprobe_source.label()),
            Cell::new(aggregate.adaptive_nprobe_decision.label()),
            Cell::new(aggregate.recursive_beam_width.label()),
            Cell::new(aggregate.max_leaf_routes.label()),
            Cell::new(aggregate.max_routing_expansions.label()),
            Cell::new(aggregate.input_frontier_width_sum),
            Cell::new(aggregate.expanded_parent_count_sum),
            Cell::new(aggregate.selected_child_count_sum),
            Cell::new(aggregate.deduped_route_count_sum),
            Cell::new(aggregate.truncation_reason.label()),
        ]);
    }
    format!("Routing budget counters\n{table}")
}

fn render_local_table(rows: &BTreeMap<StepKey, LocalStepAggregate>) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "nprobe",
        "step",
        "queries",
        "status",
        "item_sum",
        "ready_sum",
        "blocked_sum",
        "route_sum",
        "candidate_sum",
        "heap_rerank_sum",
        "remote_fanout_sum",
        "next_blocker",
    ]);
    for (key, aggregate) in rows {
        table.add_row(vec![
            Cell::new(key.nprobe),
            Cell::new(&key.step_name),
            Cell::new(aggregate.queries),
            Cell::new(aggregate.status.label()),
            Cell::new(aggregate.item_count_sum),
            Cell::new(aggregate.ready_count_sum),
            Cell::new(aggregate.blocked_count_sum),
            Cell::new(aggregate.route_count_sum),
            Cell::new(aggregate.candidate_count_sum),
            Cell::new(aggregate.heap_rerank_row_count_sum),
            Cell::new(aggregate.remote_fanout_count_sum),
            Cell::new(aggregate.next_blocker.label()),
        ]);
    }
    format!("Local pipeline counters\n{table}")
}

fn render_remote_table(rows: &BTreeMap<StepKey, RemoteStepAggregate>) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "nprobe",
        "step",
        "queries",
        "status",
        "item_sum",
        "ready_sum",
        "blocked_sum",
        "remote_pid_sum",
        "next_blocker",
    ]);
    for (key, aggregate) in rows {
        table.add_row(vec![
            Cell::new(key.nprobe),
            Cell::new(&key.step_name),
            Cell::new(aggregate.queries),
            Cell::new(aggregate.status.label()),
            Cell::new(aggregate.item_count_sum),
            Cell::new(aggregate.ready_count_sum),
            Cell::new(aggregate.blocked_count_sum),
            Cell::new(aggregate.remote_pid_count_sum),
            Cell::new(aggregate.next_blocker.label()),
        ]);
    }
    format!("Remote pipeline counters\n{table}")
}

fn render_degraded_skip_table(rows: &BTreeMap<DegradedSkipKey, DegradedSkipAggregate>) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "nprobe",
        "node_id",
        "reports",
        "requested_epoch",
        "skipped_pid_sum",
        "first_skip_category",
        "status",
    ]);
    for (key, aggregate) in rows {
        table.add_row(vec![
            Cell::new(key.nprobe),
            Cell::new(key.node_id),
            Cell::new(aggregate.reports),
            Cell::new(aggregate.requested_epoch.label()),
            Cell::new(aggregate.skipped_pid_count_sum),
            Cell::new(aggregate.first_skip_category.label()),
            Cell::new(aggregate.status.label()),
        ]);
    }
    format!("Remote degraded skip counters\n{table}")
}

fn render_local_store_overlap_table(
    rows: &BTreeMap<LocalStoreOverlapKey, LocalStoreOverlapAggregate>,
) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "nprobe",
        "node_id",
        "local_store_id",
        "queries",
        "route_sum",
        "leaf_route_sum",
        "delta_route_sum",
        "candidate_sum",
        "object_bytes_sum",
        "read_batch_sum",
        "delta_decode_sum",
    ]);
    for (key, aggregate) in rows {
        table.add_row(vec![
            Cell::new(key.nprobe),
            Cell::new(key.node_id),
            Cell::new(key.local_store_id),
            Cell::new(aggregate.queries),
            Cell::new(aggregate.route_count_sum),
            Cell::new(aggregate.leaf_route_count_sum),
            Cell::new(aggregate.delta_route_count_sum),
            Cell::new(aggregate.candidate_row_count_sum),
            Cell::new(aggregate.prefetched_object_bytes_sum),
            Cell::new(aggregate.read_batch_count_sum),
            Cell::new(aggregate.delta_decode_count_sum),
        ]);
    }
    format!("Local store overlap counters\n{table}")
}

fn render_query_metrics_table(
    rows: &BTreeMap<i32, QueryMetricAggregate>,
    include_recall: bool,
) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    let mut header = vec![
        "nprobe",
        "queries",
        "latency_min",
        "latency_p50",
        "latency_p95",
        "latency_p99",
        "latency_max",
    ];
    if include_recall {
        header.push("recall@k");
    }
    table.set_header(header);
    for (nprobe, aggregate) in rows {
        let stats = aggregate.latency_stats();
        let mut row = vec![
            Cell::new(nprobe),
            Cell::new(stats.count),
            Cell::new(format_duration_ms(stats.min)),
            Cell::new(format_duration_ms(stats.p50)),
            Cell::new(format_duration_ms(stats.p95)),
            Cell::new(format_duration_ms(stats.p99)),
            Cell::new(format_duration_ms(stats.max)),
        ];
        if include_recall {
            row.push(Cell::new(
                aggregate
                    .recall_at_k
                    .map(|value| format!("{value:.4}"))
                    .unwrap_or_else(|| "not_computed".to_owned()),
            ));
        }
        table.add_row(row);
    }
    format!("Coordinator query metrics\n{table}")
}

fn render_production_read_profile_table(
    rows: &BTreeMap<i32, ProductionReadProfileAggregate>,
) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "nprobe",
        "profiles",
        "status",
        "result_source",
        "selected_pid_sum",
        "remote_pid_sum",
        "dispatch_sum",
        "socket_open_sum",
        "connect_p50",
        "connect_p95",
        "endpoint_identity_p50",
        "endpoint_identity_p95",
        "candidate_p50",
        "candidate_p95",
        "heap_p50",
        "heap_p95",
        "merge_p50",
        "merge_p95",
        "total_p50",
        "total_p95",
        "candidate_query_sum",
        "heap_query_sum",
        "endpoint_identity_query_sum",
        "payload_bytes_sum",
        "timeout_sum",
        "cancel_sum",
        "degraded_skip_sum",
        "returned_sum",
    ]);
    for (nprobe, aggregate) in rows {
        let connect = summarize_durations(&aggregate.connect_elapsed);
        let endpoint_identity = summarize_durations(&aggregate.endpoint_identity_elapsed);
        let candidate = summarize_durations(&aggregate.candidate_receive_elapsed);
        let heap = summarize_durations(&aggregate.heap_receive_elapsed);
        let merge = summarize_durations(&aggregate.merge_elapsed);
        let total = summarize_durations(&aggregate.total_elapsed);
        table.add_row(vec![
            Cell::new(nprobe),
            Cell::new(aggregate.profiles),
            Cell::new(aggregate.status.label()),
            Cell::new(aggregate.result_source.label()),
            Cell::new(aggregate.selected_pid_count_sum),
            Cell::new(aggregate.remote_pid_count_sum),
            Cell::new(aggregate.dispatch_count_sum),
            Cell::new(aggregate.socket_open_count_sum),
            Cell::new(format_duration_ms(connect.p50)),
            Cell::new(format_duration_ms(connect.p95)),
            Cell::new(format_duration_ms(endpoint_identity.p50)),
            Cell::new(format_duration_ms(endpoint_identity.p95)),
            Cell::new(format_duration_ms(candidate.p50)),
            Cell::new(format_duration_ms(candidate.p95)),
            Cell::new(format_duration_ms(heap.p50)),
            Cell::new(format_duration_ms(heap.p95)),
            Cell::new(format_duration_ms(merge.p50)),
            Cell::new(format_duration_ms(merge.p95)),
            Cell::new(format_duration_ms(total.p50)),
            Cell::new(format_duration_ms(total.p95)),
            Cell::new(aggregate.candidate_receive_query_count_sum),
            Cell::new(aggregate.heap_receive_query_count_sum),
            Cell::new(aggregate.endpoint_identity_query_count_sum),
            Cell::new(aggregate.payload_decode_bytes_sum),
            Cell::new(aggregate.remote_timeout_count_sum),
            Cell::new(aggregate.remote_cancel_count_sum),
            Cell::new(aggregate.degraded_skipped_dispatch_count_sum),
            Cell::new(aggregate.returned_candidate_count_sum),
        ]);
    }
    format!("Production read profile\n{table}")
}

fn option_label<T: std::fmt::Display>(value: Option<T>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "default".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_args() -> SpirePipelineArgs {
        SpirePipelineArgs {
            prefix: "pfx".to_owned(),
            index: None,
            queries_limit: 1,
            sweep: vec![],
            rerank_width: None,
            max_candidate_rows: None,
            max_routed_candidate_rows: None,
            remote_tuple_transport: None,
            include_cost_snapshot: false,
            cost_routing_dimension_scale: None,
            cost_leaf_dimension_scale: None,
            cost_index_page_scale: None,
            cost_local_store_page_fanout_scale: None,
            cost_storage_scoring_multiplier: None,
            cost_rerank_multiplier: None,
            include_query_metrics: false,
            include_recall: false,
            truth_corpus_file: None,
            truth_cache_file: None,
            leaf_block_rank_output: None,
            target_block_rank_output: None,
            miss_attribution_output: None,
            leaf_block_rank_local_sequence_offset: 0,
            include_production_read_profile: false,
            production_read_only: false,
            query_metric_k: 10,
            query_metric_projection_columns: vec![],
            session_gucs: vec![],
            task87_candidate_batch_counters: false,
            adaptive_nprobe: false,
            adaptive_nprobe_score_gap_micros: None,
            include_remote: false,
            require_remote_placements: false,
            include_local_store_overlap: false,
            remote_selected_pids: vec![],
            remote_requested_epoch: None,
            top_k: 10,
            consistency_mode: "epoch".to_owned(),
            log_output: None,
            funnel_output: None,
            stage_containment_output: None,
        }
    }

    fn ranked_row(
        target_ordinal: i64,
        status: &str,
        selected_by_global_cap: Option<bool>,
    ) -> LeafBlockRankRow {
        LeafBlockRankRow {
            target_ordinal,
            target_local_sequence: 100 + target_ordinal,
            status: status.to_owned(),
            max_global_blocks: 1152,
            radius_weight: 0.25,
            scored_block_count: 2048,
            block_rank: Some(target_ordinal + 1),
            selected_by_global_cap,
            pid: Some(10 + target_ordinal),
            node_id: Some(1),
            local_store_id: Some(1),
            object_version: Some(1),
            row_index: Some(target_ordinal),
            row_base: Some(0),
            row_end: Some(16),
            row_count: Some(16),
            block_ip: Some(0.1),
            cap_block_ip: Some(0.0),
            block_ip_margin_to_cap: Some(0.1),
            route_rank: Some(target_ordinal + 1),
            route_score: Some(0.2),
            assignment_flags: Some(0),
        }
    }

    fn ready_endpoint_identity() -> EndpointIdentityRow {
        EndpointIdentityRow {
            tuple_transport_capabilities: vec!["pg_binary_attr_v1".to_owned()],
            tuple_transport_default: "pg_binary_attr_v1".to_owned(),
            tuple_transport_status: "ready".to_owned(),
            status: "ready".to_owned(),
            recommendation: "none".to_owned(),
        }
    }

    #[test]
    fn spire_pipeline_defaults_to_spire_sweep_values() {
        let args = default_args();
        assert_eq!(sweep_values(&args).unwrap(), EC_SPIRE.default_sweep);
    }

    #[test]
    fn spire_pipeline_rejects_invalid_limits() {
        let mut args = default_args();
        args.queries_limit = 0;
        assert!(validate_args(&args)
            .unwrap_err()
            .to_string()
            .contains("--queries-limit"));

        let mut args = default_args();
        args.top_k = -1;
        assert!(validate_args(&args)
            .unwrap_err()
            .to_string()
            .contains("--top-k"));

        let mut args = default_args();
        args.remote_selected_pids = vec![-1];
        assert!(validate_args(&args)
            .unwrap_err()
            .to_string()
            .contains("--remote-selected-pids"));

        let mut args = default_args();
        args.query_metric_k = 0;
        assert!(validate_args(&args)
            .unwrap_err()
            .to_string()
            .contains("--query-metric-k"));

        let mut args = default_args();
        args.query_metric_projection_columns = vec!["title;drop".to_owned()];
        assert!(validate_args(&args)
            .unwrap_err()
            .to_string()
            .contains("--query-metric-projection-columns"));

        let mut args = default_args();
        args.cost_index_page_scale = Some(f64::INFINITY);
        assert!(validate_args(&args)
            .unwrap_err()
            .to_string()
            .contains("--cost-index-page-scale"));

        let mut args = default_args();
        args.stage_containment_output = Some("stage.jsonl".into());
        args.include_recall = false;
        args.include_query_metrics = true;
        assert!(validate_args(&args)
            .unwrap_err()
            .to_string()
            .contains("--stage-containment-output requires --include-recall"));

        let mut args = default_args();
        args.stage_containment_output = Some("stage.jsonl".into());
        args.include_recall = true;
        args.include_query_metrics = false;
        assert!(validate_args(&args)
            .unwrap_err()
            .to_string()
            .contains("--stage-containment-output requires --include-query-metrics"));
    }

    #[test]
    fn miss_attribution_classifies_hit_routing_block_and_cap_misses() {
        let rows = MissAttributionRecord::from_query(
            96,
            1,
            990000,
            &[1, 2, 3, 4],
            &[101, 102, 103, 104],
            &[1],
            &[
                ranked_row(0, "block_ranked", Some(true)),
                ranked_row(1, "not_found_in_routed_leaves", None),
                ranked_row(2, "block_ranked", Some(false)),
                ranked_row(3, "block_ranked", Some(true)),
            ],
        )
        .expect("attribution rows");

        assert_eq!(rows[0].miss_stage, "hit");
        assert_eq!(rows[1].miss_stage, "routing_miss");
        assert_eq!(rows[2].miss_stage, "block_pruned_global_cap");
        assert_eq!(rows[3].miss_stage, "candidate_or_rerank_cap");
    }

    #[test]
    fn stage_containment_records_per_stage_truth_retention() {
        let local_rows = vec![
            LocalPipelineRow {
                step_ordinal: 1,
                step_name: "routing".to_owned(),
                active_epoch: 1,
                status: "ready".to_owned(),
                item_count: 1,
                ready_count: 1,
                blocked_count: 0,
                route_count: 3,
                candidate_count: 0,
                heap_rerank_row_count: 0,
                remote_fanout_count: 0,
                next_blocker: "none".to_owned(),
                recommendation: "none".to_owned(),
            },
            LocalPipelineRow {
                step_ordinal: 4,
                step_name: "candidates".to_owned(),
                active_epoch: 1,
                status: "truncated".to_owned(),
                item_count: 300,
                ready_count: 25,
                blocked_count: 275,
                route_count: 3,
                candidate_count: 300,
                heap_rerank_row_count: 0,
                remote_fanout_count: 0,
                next_blocker: "candidate_budget".to_owned(),
                recommendation: "increase max_candidate_rows".to_owned(),
            },
            LocalPipelineRow {
                step_ordinal: 5,
                step_name: "heap_rerank".to_owned(),
                active_epoch: 1,
                status: "ready".to_owned(),
                item_count: 25,
                ready_count: 25,
                blocked_count: 0,
                route_count: 0,
                candidate_count: 25,
                heap_rerank_row_count: 25,
                remote_fanout_count: 0,
                next_blocker: "none".to_owned(),
                recommendation: "none".to_owned(),
            },
        ];
        let leaf_rows = vec![LeafCandidateRow {
            pid: 10,
            node_id: 1,
            local_store_id: 1,
            object_bytes: 1024,
            route_count: 3,
            scanned_count: 3,
            candidate_row_count: 300,
            leaf_block_available_count: 12,
            leaf_block_selected_count: 6,
            leaf_block_skipped_count: 6,
            leaf_summary_object_bytes: 384,
            leaf_row_object_bytes: 2048,
            leaf_row_segment_read_count: 4,
            leaf_row_segment_read_bytes: 512,
            primary_candidate_row_count: 300,
            boundary_replica_candidate_row_count: 0,
            deduped_candidate_row_count: 0,
            truncated_candidate_row_count: 275,
            candidate_winner_count: 25,
            leaf_object_read_nanos: 1000,
            leaf_summary_score_nanos: 200,
            leaf_row_score_nanos: 300,
            candidate_score_nanos: 500,
            candidate_materialize_nanos: 50,
            candidate_heap_append_nanos: 25,
        }];
        let rerank_locality = RerankLocalityRow {
            candidate_count: 300,
            rerank_prefix_count: 25,
            unique_heap_block_count: 20,
            heap_block_transition_count: 24,
            heap_block_span: 1024,
            heap_block_jump_sum: 4096,
            heap_block_jump_max: 512,
        };
        let rows = StageContainmentRecord::from_query(
            96,
            1,
            42,
            &[1, 2, 3, 4],
            &[1],
            &local_rows,
            &leaf_rows,
            Some(&rerank_locality),
            &[
                ranked_row(0, "target_block_ranked", Some(true)),
                ranked_row(1, "not_found_in_routed_leaves", None),
                ranked_row(2, "target_block_ranked", Some(false)),
                ranked_row(3, "target_block_ranked", Some(true)),
            ],
        )
        .expect("stage containment rows");

        assert_eq!(rows.len(), 6);
        let route = rows
            .iter()
            .find(|row| row.stage_name == "topology_route_set")
            .expect("route stage");
        assert_eq!(route.contained_truth_count, 3);
        assert_eq!(route.missing_reason_counts["routing_miss"], 1);

        let blocks = rows
            .iter()
            .find(|row| row.stage_name == "selected_leaf_blocks")
            .expect("block stage");
        assert_eq!(blocks.contained_truth_count, 2);
        assert_eq!(blocks.missing_reason_counts["block_pruned_global_cap"], 1);
        assert_eq!(blocks.leaf_block_selected_count, 6);

        let candidates = rows
            .iter()
            .find(|row| row.stage_name == "local_candidate_frontier")
            .expect("candidate stage");
        assert_eq!(
            candidates.containment_basis,
            "final_hits_lower_bound_until_target_candidate_rank_snapshot"
        );
        assert_eq!(candidates.contained_truth_count, 1);
        assert_eq!(candidates.candidate_or_object_count, 300);
        assert_eq!(candidates.blocked_count, 275);
        assert_eq!(
            candidates.missing_reason_counts["candidate_or_later_cap"],
            1
        );

        let final_topk = rows
            .iter()
            .find(|row| row.stage_name == "final_top_k")
            .expect("final stage");
        assert_eq!(final_topk.contained_truth_ranks, vec![1]);
        assert_eq!(final_topk.missing_reason_counts["not_returned_in_topk"], 1);
    }

    #[test]
    fn funnel_record_carries_task85_read_and_score_breakdown() {
        let local_rows = vec![LocalPipelineRow {
            step_ordinal: 4,
            step_name: "candidates".to_owned(),
            active_epoch: 1,
            status: "truncated".to_owned(),
            item_count: 300,
            ready_count: 25,
            blocked_count: 275,
            route_count: 2,
            candidate_count: 300,
            heap_rerank_row_count: 0,
            remote_fanout_count: 0,
            next_blocker: "candidate_budget".to_owned(),
            recommendation: "increase max_candidate_rows or inspect candidate diagnostics"
                .to_owned(),
        }];
        let leaf_rows = vec![
            LeafCandidateRow {
                pid: 10,
                node_id: 1,
                local_store_id: 1,
                object_bytes: 1000,
                route_count: 1,
                scanned_count: 1,
                candidate_row_count: 100,
                leaf_block_available_count: 8,
                leaf_block_selected_count: 3,
                leaf_block_skipped_count: 5,
                leaf_summary_object_bytes: 128,
                leaf_row_object_bytes: 512,
                leaf_row_segment_read_count: 2,
                leaf_row_segment_read_bytes: 192,
                primary_candidate_row_count: 90,
                boundary_replica_candidate_row_count: 10,
                deduped_candidate_row_count: 1,
                truncated_candidate_row_count: 75,
                candidate_winner_count: 25,
                leaf_object_read_nanos: 1000,
                leaf_summary_score_nanos: 200,
                leaf_row_score_nanos: 300,
                candidate_score_nanos: 500,
                candidate_materialize_nanos: 50,
                candidate_heap_append_nanos: 25,
            },
            LeafCandidateRow {
                pid: 11,
                node_id: 1,
                local_store_id: 1,
                object_bytes: 2000,
                route_count: 1,
                scanned_count: 1,
                candidate_row_count: 200,
                leaf_block_available_count: 16,
                leaf_block_selected_count: 4,
                leaf_block_skipped_count: 12,
                leaf_summary_object_bytes: 256,
                leaf_row_object_bytes: 1024,
                leaf_row_segment_read_count: 3,
                leaf_row_segment_read_bytes: 256,
                primary_candidate_row_count: 190,
                boundary_replica_candidate_row_count: 10,
                deduped_candidate_row_count: 2,
                truncated_candidate_row_count: 175,
                candidate_winner_count: 25,
                leaf_object_read_nanos: 3000,
                leaf_summary_score_nanos: 400,
                leaf_row_score_nanos: 500,
                candidate_score_nanos: 900,
                candidate_materialize_nanos: 75,
                candidate_heap_append_nanos: 35,
            },
        ];

        let rerank_locality = RerankLocalityRow {
            candidate_count: 300,
            rerank_prefix_count: 25,
            unique_heap_block_count: 20,
            heap_block_transition_count: 24,
            heap_block_span: 1024,
            heap_block_jump_sum: 4096,
            heap_block_jump_max: 512,
        };
        let record = FunnelRecord::from_query(
            96,
            0,
            42,
            &local_rows,
            &leaf_rows,
            Some(&rerank_locality),
            Some(10),
        )
        .expect("funnel record");

        assert_eq!(record.candidate_count, 300);
        assert_eq!(record.pipeline_stages.len(), 1);
        assert_eq!(record.pipeline_stages[0].step_name, "candidates");
        assert_eq!(record.pipeline_stages[0].status, "truncated");
        assert_eq!(record.pipeline_stages[0].blocked_count, 275);
        assert_eq!(record.pipeline_stages[0].next_blocker, "candidate_budget");
        assert_eq!(
            record.pipeline_stages[0].recommendation,
            "increase max_candidate_rows or inspect candidate diagnostics"
        );
        assert_eq!(record.leaf_object_bytes, 3000);
        assert_eq!(record.leaf_summary_object_bytes, 384);
        assert_eq!(record.leaf_row_object_bytes, 1536);
        assert_eq!(record.leaf_row_segment_read_count, 5);
        assert_eq!(record.leaf_row_segment_read_bytes, 448);
        assert_eq!(record.leaf_block_available_count, 24);
        assert_eq!(record.leaf_block_selected_count, 7);
        assert_eq!(record.leaf_block_skipped_count, 17);
        assert_eq!(record.leaf_object_read_nanos, 4000);
        assert_eq!(record.leaf_summary_score_nanos, 600);
        assert_eq!(record.leaf_row_score_nanos, 800);
        assert_eq!(record.candidate_score_nanos, 1400);
        assert_eq!(record.rerank_prefix_count, 25);
        assert_eq!(record.rerank_unique_heap_block_count, 20);
        assert_eq!(record.rerank_heap_block_transition_count, 24);
        assert_eq!(record.rerank_heap_block_jump_sum, 4096);
    }

    #[test]
    fn spire_pipeline_rejects_out_of_range_sweep_values() {
        let mut args = default_args();
        args.sweep = vec![EC_SPIRE_MAX_NPROBE + 1];
        assert!(sweep_values(&args)
            .unwrap_err()
            .to_string()
            .contains("--sweep values"));
    }

    #[test]
    fn spire_pipeline_sql_uses_public_snapshot_contracts() {
        assert!(routing_snapshot_sql().contains("ec_spire_index_scan_routing_snapshot"));
        assert!(routing_snapshot_sql().contains("$1::text::regclass::oid"));
        assert!(local_pipeline_snapshot_sql().contains("ec_spire_index_scan_pipeline_snapshot"));
        assert!(leaf_candidate_snapshot_sql().contains("leaf_summary_object_bytes"));
        assert!(leaf_candidate_snapshot_sql().contains("leaf_row_object_bytes"));
        assert!(leaf_candidate_snapshot_sql().contains("leaf_row_segment_read_count"));
        assert!(leaf_candidate_snapshot_sql().contains("leaf_row_segment_read_bytes"));
        assert!(leaf_candidate_snapshot_sql().contains("leaf_summary_score_nanos"));
        assert!(
            rerank_locality_snapshot_sql().contains("ec_spire_index_scan_rerank_locality_snapshot")
        );
        assert!(legacy_leaf_candidate_snapshot_sql().contains("leaf_summary_object_bytes"));
        assert!(legacy_leaf_candidate_snapshot_sql().contains("leaf_row_object_bytes"));
        assert!(!legacy_leaf_candidate_snapshot_sql().contains("leaf_row_segment_read_count"));
        assert!(!legacy_leaf_candidate_snapshot_sql().contains("leaf_row_segment_read_bytes"));
        assert!(local_store_overlap_sql()
            .contains("ec_spire_index_scan_local_store_read_overlap_harness"));
        assert!(remote_pipeline_steps_sql().contains("ec_spire_remote_pipeline_steps"));
        assert!(remote_pipeline_steps_sql().contains("$4::bigint[]"));
        assert!(degraded_skip_report_sql().contains("ec_spire_remote_search_degraded_skip_report"));
        assert!(degraded_skip_report_sql().contains("$4::bigint[]"));
        assert!(production_read_profile_sql()
            .contains("ec_spire_remote_search_production_read_profile"));
        assert!(production_read_profile_sql().contains("$2::real[]"));
        assert!(endpoint_identity_sql().contains("ec_spire_remote_search_endpoint_identity"));
        assert!(endpoint_identity_sql().contains("tuple_transport_capabilities"));
        assert!(remote_placement_gate_sql().contains("ec_spire_remote_node_snapshot"));
        assert!(remote_placement_gate_sql().contains("node_id > 1"));
        assert!(cost_tuning_snapshot_sql().contains("ec_spire_index_cost_tuning_snapshot"));
        assert!(cost_tuning_snapshot_sql().contains("cost_routing_dimension_scale"));
    }

    #[test]
    fn spire_pipeline_remote_placement_gate_rejects_empty_or_local_only() {
        let empty = RemotePlacementGate {
            total_placement_count: 0,
            remote_placement_count: 0,
            local_placement_count: 0,
            remote_node_count: 0,
        };
        let err = enforce_remote_placement_gate("pfx_idx", &empty)
            .unwrap_err()
            .to_string();
        assert!(err.contains("remote placement count is 0"), "err: {err}");

        let local_only = RemotePlacementGate {
            total_placement_count: 7,
            remote_placement_count: 0,
            local_placement_count: 7,
            remote_node_count: 0,
        };
        let err = enforce_remote_placement_gate("pfx_idx", &local_only)
            .unwrap_err()
            .to_string();
        assert!(err.contains("local placements 7"), "err: {err}");
    }

    #[test]
    fn spire_pipeline_remote_placement_gate_accepts_remote_placements() {
        let row = RemotePlacementGate {
            total_placement_count: 11,
            remote_placement_count: 4,
            local_placement_count: 7,
            remote_node_count: 2,
        };
        enforce_remote_placement_gate("pfx_idx", &row).unwrap();
    }

    #[test]
    fn spire_pipeline_query_metric_sql_projects_payload_columns() {
        assert_eq!(
            build_query_metric_sql("corpus", &[]),
            "SELECT id FROM corpus ORDER BY embedding <#> $1::real[] LIMIT $2"
        );
        assert_eq!(
            build_query_metric_sql(
                "corpus",
                &["title".to_owned(), "body".to_owned(), "id".to_owned()]
            ),
            "SELECT id, title, body FROM corpus ORDER BY embedding <#> $1::real[] LIMIT $2"
        );
    }

    #[test]
    fn spire_pipeline_reports_remote_tuple_transport_override() {
        let routing = BTreeMap::new();
        let local = BTreeMap::new();
        let remote = BTreeMap::new();
        let header = render_header(&ReportInput {
            prefix: "pfx",
            index: "pfx_idx",
            queries: 1,
            sweep_values: &[8],
            rerank_width: None,
            max_candidate_rows: None,
            max_routed_candidate_rows: None,
            remote_tuple_transport: Some(SpireRemoteTupleTransportMode::PgBinaryAttrV1),
            endpoint_identity: &ready_endpoint_identity(),
            adaptive_nprobe_options: super::super::AdaptiveNprobeBenchOptions {
                enabled: false,
                score_gap_micros: None,
                score_margin_ratio_bps: None,
            },
            cost_snapshot_enabled: true,
            cost_tuning: &BTreeMap::new(),
            remote_enabled: true,
            remote_selected_pids: &[2, 3],
            remote_epoch: Some(1),
            query_metrics_enabled: true,
            include_recall: false,
            query_metric_k: 10,
            query_metric_projection_columns: &["title".to_owned(), "body".to_owned()],
            production_read_profile_enabled: true,
            production_read_only: true,
            local_store_overlap_enabled: true,
            routing: &routing,
            local: &local,
            remote: &remote,
            local_store_overlap: &BTreeMap::new(),
            degraded_skip: &BTreeMap::new(),
            query_metrics: &BTreeMap::new(),
            production_read_profile: &BTreeMap::new(),
        });
        assert!(header.contains("remote_tuple_transport: pg_binary_attr_v1"));
        assert!(header.contains("cost_snapshot: true"));
        assert!(header.contains("local_store_overlap: true"));
        assert!(header.contains("query_metrics: true"));
        assert!(header.contains("query_metric_k: 10"));
        assert!(header.contains("query_metric_projection_columns: id,title,body"));
        assert!(header.contains("query_recall: false"));
        assert!(header.contains("production_read_profile: true"));
        assert!(header.contains("production_read_only: true"));
    }

    #[test]
    fn spire_pipeline_renders_endpoint_identity_readiness() {
        let rendered = render_endpoint_identity_table(&ready_endpoint_identity());
        assert!(rendered.contains("Endpoint tuple transport identity"));
        assert!(rendered.contains("tuple_transport_capabilities"));
        assert!(rendered.contains("pg_binary_attr_v1"));
        assert!(rendered.contains("pg_binary_attr_v1_ready"));
        assert!(rendered.contains("true"));
    }

    #[test]
    fn spire_pipeline_renders_cost_tuning_snapshot() {
        let mut rows = BTreeMap::new();
        rows.insert(
            8,
            CostTuningRow {
                storage_format: "rabitq".to_owned(),
                effective_rerank_width: 0,
                cost_routing_dimension_scale: 0.02,
                cost_leaf_dimension_scale: 0.03,
                cost_index_page_scale: 2.0,
                cost_local_store_page_fanout_scale: 0.10,
                cost_storage_scoring_multiplier: 1.5,
                effective_storage_scoring_multiplier: 0.675,
                cost_rerank_multiplier: 2.0,
                effective_rerank_multiplier: 2.0,
            },
        );

        let rendered = render_cost_tuning_table(&rows);
        assert!(rendered.contains("Cost tuning snapshot"));
        assert!(rendered.contains("routing_dim"));
        assert!(rendered.contains("0.020000"));
        assert!(rendered.contains("0.675000"));
    }

    #[test]
    fn spire_pipeline_renders_degraded_skip_counters() {
        let mut aggregate = DegradedSkipAggregate::default();
        aggregate.record(DegradedSkipRow {
            requested_epoch: 7,
            node_id: 3,
            skipped_pid_count: 2,
            first_skip_category: "remote_index_unavailable".to_owned(),
            status: "degraded_skipped".to_owned(),
        });
        let mut rows = BTreeMap::new();
        rows.insert(
            DegradedSkipKey {
                nprobe: 8,
                node_id: 3,
            },
            aggregate,
        );

        let rendered = render_degraded_skip_table(&rows);
        assert!(rendered.contains("Remote degraded skip counters"));
        assert!(rendered.contains("skipped_pid_sum"));
        assert!(rendered.contains("remote_index_unavailable"));
        assert!(rendered.contains("degraded_skipped"));
    }

    #[test]
    fn spire_pipeline_renders_local_store_overlap_counters() {
        let mut aggregate = LocalStoreOverlapAggregate::default();
        aggregate.record(LocalStoreOverlapRow {
            node_id: 2,
            local_store_id: 1,
            route_count: 3,
            leaf_route_count: 2,
            delta_route_count: 1,
            candidate_row_count: 4,
            prefetched_object_bytes: 4096,
            read_batch_count: 1,
            delta_decode_count: 1,
        });
        let mut rows = BTreeMap::new();
        rows.insert(
            LocalStoreOverlapKey {
                nprobe: 8,
                node_id: 2,
                local_store_id: 1,
            },
            aggregate,
        );

        let rendered = render_local_store_overlap_table(&rows);
        assert!(rendered.contains("Local store overlap counters"));
        assert!(rendered.contains("object_bytes_sum"));
        assert!(rendered.contains("delta_decode_sum"));
        assert!(rendered.contains("4096"));
    }

    #[test]
    fn spire_pipeline_renders_query_metrics_with_recall() {
        let mut aggregate = QueryMetricAggregate::default();
        aggregate.record(Duration::from_millis(1), vec![10, 20]);
        aggregate.record(Duration::from_millis(3), vec![20, 30]);
        aggregate.record_recall(&[vec![10, 20], vec![20, 40]], 2);
        let mut rows = BTreeMap::new();
        rows.insert(8, aggregate);

        let rendered = render_query_metrics_table(&rows, true);
        assert!(rendered.contains("Coordinator query metrics"));
        assert!(rendered.contains("latency_p50"));
        assert!(rendered.contains("recall@k"));
        assert!(rendered.contains("0.7500"));
    }

    #[test]
    fn spire_pipeline_renders_production_read_profile() {
        let mut aggregate = ProductionReadProfileAggregate::default();
        aggregate.record(ProductionReadProfileRow {
            values: BTreeMap::from([
                ("status".into(), "ready".into()),
                ("result_source".into(), "remote_heap_candidates".into()),
                ("selected_pid_count".into(), "3".into()),
                ("remote_pid_count".into(), "3".into()),
                ("dispatch_count".into(), "2".into()),
                ("socket_open_count".into(), "2".into()),
                ("connect_elapsed_ms".into(), "1".into()),
                ("endpoint_identity_elapsed_ms".into(), "2".into()),
                ("candidate_receive_elapsed_ms".into(), "5".into()),
                ("heap_receive_elapsed_ms".into(), "7".into()),
                ("merge_elapsed_ms".into(), "1".into()),
                ("total_elapsed_ms".into(), "10".into()),
                ("candidate_receive_query_count".into(), "2".into()),
                ("heap_receive_query_count".into(), "2".into()),
                ("endpoint_identity_query_count".into(), "2".into()),
                ("payload_decode_bytes".into(), "256".into()),
                ("remote_timeout_count".into(), "0".into()),
                ("remote_cancel_count".into(), "0".into()),
                ("degraded_skipped_dispatch_count".into(), "0".into()),
                ("returned_candidate_count".into(), "6".into()),
            ]),
        });
        let mut rows = BTreeMap::new();
        rows.insert(3, aggregate);

        let rendered = render_production_read_profile_table(&rows);
        assert!(rendered.contains("Production read profile"));
        assert!(rendered.contains("connect_p95"));
        assert!(rendered.contains("endpoint_identity_query_sum"));
        assert!(rendered.contains("remote_heap_candidates"));
        assert!(rendered.contains("payload_bytes_sum"));
        assert!(rendered.contains("256"));
    }

    #[test]
    fn spire_pipeline_query_matrix_requires_fixed_dimensions() {
        let rows = vec![
            QueryVector {
                id: 1,
                source: vec![1.0, 0.0],
            },
            QueryVector {
                id: 2,
                source: vec![1.0],
            },
        ];
        assert!(query_matrix(&rows)
            .unwrap_err()
            .to_string()
            .contains("fixed dimensions"));
    }

    #[test]
    fn mixed_value_reports_stable_or_mixed_values() {
        let mut value = MixedValue::default();
        value.record("ready".to_owned());
        value.record("ready".to_owned());
        assert_eq!(value.label(), "ready");
        value.record("blocked".to_owned());
        assert_eq!(value.label(), "mixed");
    }
}
