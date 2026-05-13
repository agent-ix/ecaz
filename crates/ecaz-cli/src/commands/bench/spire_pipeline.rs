//! `ecaz bench spire-pipeline` — SPIRE routing and pipeline counters.
//!
//! The recall, latency, and storage commands own the scalar performance
//! measurements. This command owns the structural counters Phase 9/10 need:
//! routing budgets, local scan pipeline counts, and optional remote fanout
//! diagnostics from the SQL-visible operator surfaces.

use clap::{Args, ValueEnum};
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use tokio_postgres::{Client, Row};

use crate::profiles::{self, EC_SPIRE};
use crate::psql::{self, ConnectionOptions};

const EC_SPIRE_MAX_NPROBE: i32 = 1_000_000;
const EC_SPIRE_MAX_RERANK_WIDTH: i32 = 10_000_000;
const EC_SPIRE_MAX_CANDIDATE_ROWS: i32 = 10_000_000;

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
    /// Write the pipeline report to this path in addition to stdout.
    #[arg(long)]
    pub log_output: Option<PathBuf>,
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
    let adaptive_nprobe_options = super::SpireAdaptiveNprobeBenchOptions {
        enabled: args.adaptive_nprobe,
        score_gap_micros: args.adaptive_nprobe_score_gap_micros,
    };
    super::validate_spire_adaptive_nprobe_options(&EC_SPIRE, adaptive_nprobe_options)?;

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
    let queries = fetch_queries(&client, &queries_table, args.queries_limit).await?;
    if queries.is_empty() {
        return Err(eyre!("queries table {queries_table:?} is empty"));
    }

    let mut routing = BTreeMap::<RoutingKey, RoutingAggregate>::new();
    let mut local = BTreeMap::<StepKey, LocalStepAggregate>::new();
    let mut remote = BTreeMap::<StepKey, RemoteStepAggregate>::new();
    let mut remote_epoch = args.remote_requested_epoch;

    for nprobe in &sweep_values {
        apply_session_options(
            &client,
            *nprobe,
            args.rerank_width,
            args.max_candidate_rows,
            args.remote_tuple_transport,
            adaptive_nprobe_options,
        )
        .await?;

        for query in &queries {
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
            }
        }
    }

    let output = render_report(ReportInput {
        prefix: &args.prefix,
        index: &index,
        queries: queries.len(),
        sweep_values: &sweep_values,
        rerank_width: args.rerank_width,
        max_candidate_rows: args.max_candidate_rows,
        remote_tuple_transport: args.remote_tuple_transport,
        adaptive_nprobe_options,
        remote_enabled,
        remote_selected_pids: &args.remote_selected_pids,
        remote_epoch,
        routing: &routing,
        local: &local,
        remote: &remote,
    });
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
    Ok(())
}

fn validate_args(args: &SpirePipelineArgs) -> Result<()> {
    if args.queries_limit == 0 {
        return Err(eyre!("--queries-limit must be >= 1"));
    }
    if args.top_k < 0 {
        return Err(eyre!("--top-k must be >= 0"));
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
    Ok(())
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
    let sql = format!("SELECT id, source FROM {queries_table} ORDER BY id LIMIT {queries_limit}");
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
    remote_tuple_transport: Option<SpireRemoteTupleTransportMode>,
    adaptive_nprobe_options: super::SpireAdaptiveNprobeBenchOptions,
) -> Result<()> {
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
    super::apply_spire_adaptive_nprobe_options(client, adaptive_nprobe_options).await?;
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

fn remote_pipeline_steps_sql() -> &'static str {
    "SELECT step_ordinal, step_name, requested_epoch, status, item_count,
            ready_count, blocked_count, remote_pid_count, next_blocker,
            recommendation
     FROM ec_spire_remote_pipeline_steps(
            $1::text::regclass::oid, $2::bigint, $3::real[], $4::bigint[],
            $5::integer, $6::text)
     ORDER BY step_ordinal"
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

#[derive(Debug)]
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
        }
    }
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
    remote_tuple_transport: Option<SpireRemoteTupleTransportMode>,
    adaptive_nprobe_options: super::SpireAdaptiveNprobeBenchOptions,
    remote_enabled: bool,
    remote_selected_pids: &'a [i64],
    remote_epoch: Option<i64>,
    routing: &'a BTreeMap<RoutingKey, RoutingAggregate>,
    local: &'a BTreeMap<StepKey, LocalStepAggregate>,
    remote: &'a BTreeMap<StepKey, RemoteStepAggregate>,
}

fn render_report(input: ReportInput<'_>) -> String {
    let mut sections = vec![render_header(&input)];
    sections.push(render_routing_table(input.routing));
    sections.push(render_local_table(input.local));
    if input.remote_enabled {
        sections.push(render_remote_table(input.remote));
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
        "SPIRE pipeline benchmark\nprefix: {prefix}\nindex: {index}\nqueries: {queries}\nsweep: {sweep:?}\nrerank_width: {rerank_width}\nmax_candidate_rows: {max_candidate_rows}\nremote_tuple_transport: {remote_tuple_transport}\nadaptive_nprobe: {adaptive}\nremote: {remote}\nremote_selected_pids: {remote_selected_pids:?}\nremote_requested_epoch: {remote_epoch}",
        prefix = input.prefix,
        index = input.index,
        queries = input.queries,
        sweep = input.sweep_values,
        rerank_width = option_label(input.rerank_width),
        max_candidate_rows = option_label(input.max_candidate_rows),
        remote_tuple_transport = option_label(input.remote_tuple_transport),
        remote = input.remote_enabled,
        remote_selected_pids = input.remote_selected_pids,
        remote_epoch = option_label(input.remote_epoch),
    )
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
            remote_tuple_transport: None,
            adaptive_nprobe: false,
            adaptive_nprobe_score_gap_micros: None,
            include_remote: false,
            remote_selected_pids: vec![],
            remote_requested_epoch: None,
            top_k: 10,
            consistency_mode: "epoch".to_owned(),
            log_output: None,
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
        assert!(remote_pipeline_steps_sql().contains("ec_spire_remote_pipeline_steps"));
        assert!(remote_pipeline_steps_sql().contains("$4::bigint[]"));
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
            remote_tuple_transport: Some(SpireRemoteTupleTransportMode::PgBinaryAttrV1),
            adaptive_nprobe_options: super::super::SpireAdaptiveNprobeBenchOptions {
                enabled: false,
                score_gap_micros: None,
            },
            remote_enabled: true,
            remote_selected_pids: &[2, 3],
            remote_epoch: Some(1),
            routing: &routing,
            local: &local,
            remote: &remote,
        });
        assert!(header.contains("remote_tuple_transport: pg_binary_attr_v1"));
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
