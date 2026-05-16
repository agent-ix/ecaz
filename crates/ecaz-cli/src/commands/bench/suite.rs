//! `ecaz bench suite` — configured benchmark suite runner.
//!
//! Suites are JSON plans that expand into ordinary `ecaz` commands. The runner
//! keeps the expansion visible in a manifest, then optionally executes each
//! selected step in sequence.

use clap::{Args, Subcommand};
use color_eyre::eyre::{bail, Context, ContextCompat, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::process::Command;

use crate::profiles::{self, IndexProfile};
use crate::psql::ConnectionOptions;

#[derive(Args, Debug)]
pub struct SuiteArgs {
    #[command(subcommand)]
    command: Option<SuiteCommand>,

    /// JSON suite configuration file. Legacy alias for `bench suite run --dry-run`.
    #[arg(long)]
    config: Option<PathBuf>,

    /// Print expanded commands without executing suite steps. Legacy alias for
    /// `bench suite run --dry-run`.
    #[arg(long)]
    dry_run: bool,

    /// Expand only steps with this name. Repeatable. Legacy alias for
    /// `bench suite run --only`.
    #[arg(long = "only")]
    only: Vec<String>,

    /// Write the suite manifest to this path. Legacy alias for
    /// `bench suite run --manifest-output`.
    #[arg(long)]
    manifest_output: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum SuiteCommand {
    /// Execute or dry-run a configured benchmark suite.
    Run(RunArgs),
    /// Validate suite shape and required input files before a long run.
    Audit(AuditArgs),
    /// Summarize completion state from a suite manifest.
    Status(StatusArgs),
    /// Emit a minimal markdown report from a suite manifest.
    Report(ReportArgs),
}

#[derive(Args, Debug)]
struct RunArgs {
    /// JSON suite configuration file.
    #[arg(long)]
    config: PathBuf,

    /// Print expanded commands without executing suite steps.
    #[arg(long)]
    dry_run: bool,

    /// Execute remaining selected steps after a failure.
    #[arg(long)]
    continue_on_error: bool,

    /// Run only steps with this name. Repeatable.
    #[arg(long = "only")]
    only: Vec<String>,

    /// Run only steps with this tag. Repeatable.
    #[arg(long = "only-tag")]
    only_tag: Vec<String>,

    /// Reuse successful step records from an earlier manifest.
    #[arg(long)]
    resume_from: Option<PathBuf>,

    /// Write normalized result rows. Defaults to `<artifact_dir>/results.jsonl`
    /// when the config has `artifact_dir`.
    #[arg(long)]
    results_output: Option<PathBuf>,

    /// Write the suite manifest to this path. Defaults to
    /// `<artifact_dir>/suite-manifest.json` when the config has `artifact_dir`.
    #[arg(long)]
    manifest_output: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct AuditArgs {
    /// JSON suite configuration file.
    #[arg(long)]
    config: PathBuf,
}

#[derive(Args, Debug)]
struct StatusArgs {
    /// Suite manifest produced by `ecaz bench suite run`.
    #[arg(long)]
    manifest: PathBuf,
}

#[derive(Args, Debug)]
struct ReportArgs {
    /// Suite manifest produced by `ecaz bench suite run`.
    #[arg(long)]
    manifest: PathBuf,

    /// Write normalized result rows parsed from manifest artifacts.
    #[arg(long)]
    results_output: Option<PathBuf>,
}

#[derive(Debug)]
struct SuiteRunOptions {
    config: PathBuf,
    dry_run: bool,
    continue_on_error: bool,
    only: Vec<String>,
    only_tag: Vec<String>,
    resume_from: Option<PathBuf>,
    results_output: Option<PathBuf>,
    manifest_output: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct SuiteConfig {
    name: String,
    schema_version: u32,
    #[serde(default)]
    artifact_dir: Option<PathBuf>,
    #[serde(default)]
    defaults: SuiteDefaults,
    #[serde(default)]
    thresholds: Vec<ThresholdConfig>,
    steps: Vec<SuiteStep>,
}

#[derive(Debug, Clone, Deserialize)]
struct ThresholdConfig {
    name: String,
    step: String,
    metric: String,
    #[serde(default)]
    filters: BTreeMap<String, String>,
    field: String,
    op: ThresholdOp,
    value: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ThresholdOp {
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
}

#[derive(Debug, Default, Deserialize)]
struct SuiteDefaults {
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    bits: Option<i32>,
    #[serde(default)]
    seed: Option<i64>,
    #[serde(default)]
    queries_limit: Option<usize>,
    #[serde(default)]
    iterations: Option<usize>,
    #[serde(default)]
    force_index: Option<bool>,
    #[serde(default)]
    sample_backend_memory: Option<bool>,
    #[serde(default)]
    memory_sample_interval_ms: Option<u64>,
    #[serde(default)]
    pg: Option<u16>,
    #[serde(default)]
    socket_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum SuiteStep {
    CorpusFetch(CorpusFetchStep),
    CorpusPrepare(CorpusPrepareStep),
    Load(LoadStep),
    Recall(RecallStep),
    Latency(LatencyStep),
    Storage(StorageStep),
    Explain(ExplainStep),
    ComparePgvector(ComparePgvectorStep),
    CompareVectorscale(CompareVectorscaleStep),
    Raw(RawStep),
}

#[derive(Debug, Deserialize)]
struct CorpusFetchStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    dataset: String,
    output_dir: PathBuf,
    #[serde(default)]
    revision: Option<String>,
    #[serde(default)]
    force: bool,
}

#[derive(Debug, Deserialize)]
struct CorpusPrepareStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    profile: String,
    parquet: PathBuf,
    output_dir: PathBuf,
    #[serde(default)]
    id_column: Option<String>,
    #[serde(default)]
    vector_column: Option<String>,
    #[serde(default)]
    dim: Option<usize>,
    #[serde(default)]
    source_dataset: Option<String>,
    #[serde(default)]
    chunk_rows: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct LoadStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    prefix: String,
    #[serde(default)]
    corpus_file: Option<PathBuf>,
    #[serde(default)]
    queries_file: Option<PathBuf>,
    #[serde(default)]
    manifest_file: Option<PathBuf>,
    #[serde(default)]
    allow_manifest_mismatch: bool,
    #[serde(default)]
    chunked: bool,
    #[serde(default)]
    dim: Option<usize>,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    bits: Option<i32>,
    #[serde(default)]
    seed: Option<i64>,
    #[serde(default)]
    m: Vec<i32>,
    #[serde(default)]
    ef_construction: Option<i32>,
    #[serde(default)]
    reloptions: Vec<String>,
    #[serde(default)]
    log_file: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct RecallStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    prefix: String,
    k: usize,
    sweep: Vec<i32>,
    #[serde(default)]
    rerank_width: Option<i32>,
    #[serde(default)]
    queries_limit: Option<usize>,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    bits: Option<i32>,
    #[serde(default)]
    seed: Option<i64>,
    #[serde(default)]
    force_index: Option<bool>,
    #[serde(default)]
    truth_cache_file: Option<PathBuf>,
    #[serde(default)]
    truth_cache_dir: Option<PathBuf>,
    #[serde(default)]
    log_output: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LatencyStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    prefix: String,
    sweep: Vec<i32>,
    #[serde(default)]
    k: Option<usize>,
    #[serde(default)]
    concurrency: Option<usize>,
    #[serde(default)]
    iterations: Option<usize>,
    #[serde(default)]
    rerank_width: Option<i32>,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    bits: Option<i32>,
    #[serde(default)]
    seed: Option<i64>,
    #[serde(default)]
    force_index: Option<bool>,
    #[serde(default)]
    sample_backend_memory: Option<bool>,
    #[serde(default)]
    memory_sample_interval_ms: Option<u64>,
    #[serde(default)]
    log_output: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct StorageStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    prefix: String,
    #[serde(default)]
    log_file: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct ExplainStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    prefix: String,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    index_name: Option<String>,
    #[serde(default)]
    query_table: Option<String>,
    #[serde(default)]
    corpus_table: Option<String>,
    nprobe: i32,
    rerank_width: i32,
    #[serde(default)]
    pg: Option<u16>,
    #[serde(default)]
    db: Option<String>,
    #[serde(default)]
    socket_dir: Option<PathBuf>,
    #[serde(default)]
    port: Option<u16>,
    sql_file: PathBuf,
    log_output: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ComparePgvectorStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    prefix: String,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    k: Option<usize>,
    #[serde(default)]
    sweep: Vec<i32>,
    #[serde(default)]
    ecaz_sweep: Option<i32>,
    #[serde(default)]
    pgvector_am: Option<String>,
    #[serde(default)]
    pgvector_ef_search: Option<i32>,
    #[serde(default)]
    pgvector_m: Option<i32>,
    #[serde(default)]
    pgvector_ef_construction: Option<i32>,
    #[serde(default)]
    pgvector_lists: Option<i32>,
    #[serde(default)]
    pgvector_probes: Option<i32>,
    #[serde(default)]
    pgvector_maintenance_work_mem: Option<String>,
    #[serde(default)]
    rerank_width: Option<i32>,
    #[serde(default)]
    queries_limit: Option<usize>,
    #[serde(default)]
    rebuild: bool,
    #[serde(default)]
    log_file: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct CompareVectorscaleStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    prefix: String,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    k: Option<usize>,
    #[serde(default)]
    sweep: Vec<i32>,
    #[serde(default)]
    ecaz_sweep: Option<i32>,
    #[serde(default)]
    vectorscale_num_neighbors: Option<i32>,
    #[serde(default)]
    vectorscale_build_search_list_size: Option<i32>,
    #[serde(default)]
    vectorscale_max_alpha: Option<f32>,
    #[serde(default)]
    vectorscale_storage_layout: Option<String>,
    #[serde(default)]
    vectorscale_query_rescore: Option<i32>,
    #[serde(default)]
    queries_limit: Option<usize>,
    #[serde(default)]
    rebuild: bool,
    #[serde(default)]
    log_file: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct RawStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    args: Vec<String>,
    #[serde(default)]
    expected_artifacts: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SuiteManifest {
    suite: String,
    schema_version: u32,
    config: String,
    config_sha256: String,
    dry_run: bool,
    generated_at_unix_ms: u128,
    connection: ManifestConnection,
    steps: Vec<StepRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    threshold_results: Vec<ThresholdResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestConnection {
    database: String,
    host: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    password_configured: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct StepRecord {
    name: String,
    kind: String,
    command: Vec<String>,
    selected: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    expected_artifacts: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    status: Option<StepStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    started_at_unix_ms: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    finished_at_unix_ms: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    duration_ms: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    exit_code: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThresholdResult {
    name: String,
    step: String,
    metric: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    filters: BTreeMap<String, String>,
    field: String,
    op: ThresholdOp,
    expected: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    actual: Option<f64>,
    passed: bool,
    message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum StepStatus {
    DryRun,
    Pending,
    Skipped,
    Succeeded,
    Failed,
}

pub async fn run(conn: &ConnectionOptions, args: SuiteArgs) -> Result<()> {
    match args.command {
        Some(SuiteCommand::Run(run_args)) => run_suite(conn, run_args.into()).await,
        Some(SuiteCommand::Audit(audit_args)) => audit_suite(&audit_args.config).await,
        Some(SuiteCommand::Status(status_args)) => status_manifest(&status_args.manifest).await,
        Some(SuiteCommand::Report(report_args)) => {
            report_manifest(&report_args.manifest, report_args.results_output.as_deref()).await
        }
        None => {
            let config = args.config.context(
                "missing --config; use `ecaz bench suite run --config <path>` or the legacy `ecaz bench suite --config <path> --dry-run` alias",
            )?;
            if !args.dry_run {
                bail!(
                    "legacy `ecaz bench suite --config` only supports --dry-run; use `ecaz bench suite run --config {}` to execute",
                    config.display()
                );
            }
            run_suite(
                conn,
                SuiteRunOptions {
                    config,
                    dry_run: true,
                    continue_on_error: false,
                    only: args.only,
                    only_tag: Vec::new(),
                    resume_from: None,
                    results_output: None,
                    manifest_output: args.manifest_output,
                },
            )
            .await
        }
    }
}

impl From<RunArgs> for SuiteRunOptions {
    fn from(args: RunArgs) -> Self {
        Self {
            config: args.config,
            dry_run: args.dry_run,
            continue_on_error: args.continue_on_error,
            only: args.only,
            only_tag: args.only_tag,
            resume_from: args.resume_from,
            results_output: args.results_output,
            manifest_output: args.manifest_output,
        }
    }
}

async fn run_suite(conn: &ConnectionOptions, args: SuiteRunOptions) -> Result<()> {
    let (raw, config) = load_config(&args.config).await?;
    validate_config(&config)?;

    let mut manifest = build_manifest(conn, &args, &raw, &config)?;
    if let Some(resume_from) = &args.resume_from {
        apply_resume(&mut manifest, resume_from).await?;
    }
    write_manifest_if_requested(&args, &config, &manifest).await?;

    if args.dry_run {
        for record in &manifest.steps {
            if record.selected {
                crate::ecaz_println!(
                    "[suite:{}] {} -> {}",
                    config.name,
                    record.name,
                    shell_join(&record.command)
                );
            }
        }
        return Ok(());
    }

    let exe = std::env::current_exe().context("resolving current ecaz executable")?;
    for idx in 0..manifest.steps.len() {
        if !manifest.steps[idx].selected {
            continue;
        }
        if matches!(manifest.steps[idx].status, Some(StepStatus::Succeeded)) {
            crate::ecaz_println!(
                "[suite:{}] {} already succeeded in resume manifest",
                config.name,
                manifest.steps[idx].name
            );
            continue;
        }
        prepare_step(&config.steps[idx], &config.defaults).await?;
        let command = manifest.steps[idx].command.clone();
        crate::ecaz_println!(
            "[suite:{}] {} -> {}",
            config.name,
            manifest.steps[idx].name,
            shell_join(&command)
        );
        manifest.steps[idx].status = Some(StepStatus::Pending);
        manifest.steps[idx].started_at_unix_ms = Some(now_ms());
        write_manifest_if_requested(&args, &config, &manifest).await?;

        let started = Instant::now();
        let status = spawn_step(&exe, &command, conn).await.wrap_err_with(|| {
            format!(
                "running suite step {:?}: {}",
                manifest.steps[idx].name,
                shell_join(&command)
            )
        })?;
        manifest.steps[idx].finished_at_unix_ms = Some(now_ms());
        manifest.steps[idx].duration_ms = Some(started.elapsed().as_millis());
        manifest.steps[idx].exit_code = status.code();
        manifest.steps[idx].status = Some(if status.success() {
            StepStatus::Succeeded
        } else {
            StepStatus::Failed
        });
        write_manifest_if_requested(&args, &config, &manifest).await?;

        if !status.success() && !args.continue_on_error {
            bail!(
                "suite step {:?} failed with {}; rerun with --continue-on-error to keep going",
                manifest.steps[idx].name,
                format_exit_status(status)
            );
        }
    }

    let rows = write_results_if_requested(&args, &config, &manifest).await?;
    let selected_steps = selected_step_names(&manifest);
    manifest.threshold_results =
        evaluate_thresholds_for_steps(&config.thresholds, &rows, &selected_steps);
    write_manifest_if_requested(&args, &config, &manifest).await?;
    let failures = manifest
        .threshold_results
        .iter()
        .filter(|result| !result.passed)
        .count();
    if failures > 0 {
        bail!("suite thresholds failed: {failures}");
    }
    Ok(())
}

async fn audit_suite(config_path: &Path) -> Result<()> {
    let (_raw, config) = load_config(config_path).await?;
    let mut findings = Vec::new();
    let mut produced = HashSet::new();
    if let Err(err) = validate_config(&config) {
        findings.push(err.to_string());
    }
    for step in &config.steps {
        for input in step.input_paths() {
            if produced.contains(&input) {
                continue;
            }
            if tokio::fs::metadata(&input).await.is_err() {
                findings.push(format!(
                    "step {:?} references missing input {}",
                    step.name(),
                    input.display()
                ));
            }
        }
        produced.extend(step.produced_paths());
        if step.expected_artifacts().is_empty() {
            findings.push(format!(
                "step {:?} does not declare an artifact path",
                step.name()
            ));
        }
    }

    if findings.is_empty() {
        crate::ecaz_println!(
            "[suite:{}] audit passed: {} steps",
            config.name,
            config.steps.len()
        );
        Ok(())
    } else {
        for finding in &findings {
            crate::ecaz_eprintln!("[suite:{}] audit: {finding}", config.name);
        }
        bail!("suite audit found {} issue(s)", findings.len())
    }
}

async fn status_manifest(path: &Path) -> Result<()> {
    let manifest = load_manifest(path).await?;
    let summary = summarize_manifest(&manifest).await;
    crate::ecaz_println!(
        "[suite:{}] completed={} failed={} skipped={} dry_run={} missing_artifacts={} stale={}",
        manifest.suite,
        summary.completed,
        summary.failed,
        summary.skipped,
        summary.dry_run,
        summary.missing_artifacts,
        summary.stale
    );
    for step in &manifest.steps {
        let status = step.status.unwrap_or(if step.selected {
            StepStatus::Pending
        } else {
            StepStatus::Skipped
        });
        crate::ecaz_println!(
            "{:<12} {:<36} {}",
            format!("{status:?}"),
            step.name,
            shell_join(&step.command)
        );
    }
    Ok(())
}

async fn report_manifest(path: &Path, results_output: Option<&Path>) -> Result<()> {
    let manifest = load_manifest(path).await?;
    let summary = summarize_manifest(&manifest).await;
    let rows = extract_result_rows(&manifest).await?;
    crate::ecaz_println!("# Suite Report: {}", manifest.suite);
    crate::ecaz_println!("");
    crate::ecaz_println!("- config: `{}`", manifest.config);
    crate::ecaz_println!("- config_sha256: `{}`", manifest.config_sha256);
    crate::ecaz_println!("- dry_run: `{}`", manifest.dry_run);
    crate::ecaz_println!(
        "- steps: completed {}, failed {}, skipped {}, dry-run {}, missing artifacts {}, stale {}",
        summary.completed,
        summary.failed,
        summary.skipped,
        summary.dry_run,
        summary.missing_artifacts,
        summary.stale
    );
    crate::ecaz_println!("");
    crate::ecaz_println!("| Step | Kind | Status | Duration ms | Artifacts |");
    crate::ecaz_println!("| --- | --- | --- | ---: | --- |");
    for step in &manifest.steps {
        let status = step.status.unwrap_or(if step.selected {
            StepStatus::Pending
        } else {
            StepStatus::Skipped
        });
        crate::ecaz_println!(
            "| {} | {} | {:?} | {} | {} |",
            step.name,
            step.kind,
            status,
            step.duration_ms
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".into()),
            if step.expected_artifacts.is_empty() {
                "-".into()
            } else {
                step.expected_artifacts
                    .iter()
                    .map(|path| format!("`{path}`"))
                    .collect::<Vec<_>>()
                    .join("<br>")
            }
        );
    }
    if !rows.is_empty() {
        crate::ecaz_println!("");
        crate::ecaz_println!("## Parsed Results");
        crate::ecaz_println!("");
        crate::ecaz_println!("| Step | Kind | Metric | Values |");
        crate::ecaz_println!("| --- | --- | --- | --- |");
        for row in &rows {
            crate::ecaz_println!(
                "| {} | {} | {} | {} |",
                row.step,
                row.kind,
                row.metric,
                format_metric_values(&row.values)
            );
        }
    }
    if !manifest.threshold_results.is_empty() {
        crate::ecaz_println!("");
        crate::ecaz_println!("## Thresholds");
        crate::ecaz_println!("");
        crate::ecaz_println!("| Name | Status | Actual | Expected |");
        crate::ecaz_println!("| --- | --- | ---: | ---: |");
        for result in &manifest.threshold_results {
            crate::ecaz_println!(
                "| {} | {} | {} | {:?} {} |",
                result.name,
                if result.passed { "pass" } else { "fail" },
                result
                    .actual
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "-".into()),
                result.op,
                result.expected
            );
        }
    }
    if let Some(path) = results_output {
        write_results_jsonl(path, &rows).await?;
        crate::ecaz_eprintln!("wrote {}", path.display());
    }
    Ok(())
}

async fn load_config(path: &Path) -> Result<(String, SuiteConfig)> {
    let raw = tokio::fs::read_to_string(path)
        .await
        .wrap_err_with(|| format!("reading {}", path.display()))?;
    let config: SuiteConfig =
        serde_json::from_str(&raw).wrap_err_with(|| format!("parsing {}", path.display()))?;
    Ok((raw, config))
}

async fn load_manifest(path: &Path) -> Result<SuiteManifest> {
    let raw = tokio::fs::read_to_string(path)
        .await
        .wrap_err_with(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&raw).wrap_err_with(|| format!("parsing {}", path.display()))
}

fn build_manifest(
    conn: &ConnectionOptions,
    args: &SuiteRunOptions,
    raw: &str,
    config: &SuiteConfig,
) -> Result<SuiteManifest> {
    let mut manifest = SuiteManifest {
        suite: config.name.clone(),
        schema_version: config.schema_version,
        config: args.config.display().to_string(),
        config_sha256: sha256_hex(raw.as_bytes()),
        dry_run: args.dry_run,
        generated_at_unix_ms: now_ms(),
        connection: ManifestConnection {
            database: conn.database.clone(),
            host: conn.host.clone(),
            port: conn.port,
            user: conn.user.clone(),
            password_configured: conn.password.is_some(),
        },
        steps: Vec::with_capacity(config.steps.len()),
        threshold_results: Vec::new(),
    };

    for step in &config.steps {
        let selected = step_selected(step, args);
        let command = if selected {
            child_command_args(conn, step.expand(&config.defaults, conn)?)
        } else {
            Vec::new()
        };
        manifest.steps.push(StepRecord {
            name: step.name().to_string(),
            kind: step.kind().to_string(),
            command,
            selected,
            tags: step.tags().to_vec(),
            expected_artifacts: step
                .expected_artifacts()
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            status: Some(if selected {
                if args.dry_run {
                    StepStatus::DryRun
                } else {
                    StepStatus::Pending
                }
            } else {
                StepStatus::Skipped
            }),
            started_at_unix_ms: None,
            finished_at_unix_ms: None,
            duration_ms: None,
            exit_code: None,
        });
    }
    Ok(manifest)
}

fn step_selected(step: &SuiteStep, args: &SuiteRunOptions) -> bool {
    let name_matches = args.only.is_empty() || args.only.iter().any(|only| only == step.name());
    let tag_matches = args.only_tag.is_empty()
        || args
            .only_tag
            .iter()
            .any(|only| step.tags().iter().any(|tag| tag == only));
    name_matches && tag_matches
}

async fn apply_resume(manifest: &mut SuiteManifest, resume_from: &Path) -> Result<()> {
    let previous = load_manifest(resume_from).await?;
    if previous.config_sha256 != manifest.config_sha256 {
        bail!(
            "resume manifest config hash {} does not match current config hash {}",
            previous.config_sha256,
            manifest.config_sha256
        );
    }
    let previous_by_name: HashMap<_, _> = previous
        .steps
        .into_iter()
        .map(|step| (step.name.clone(), step))
        .collect();
    for step in &mut manifest.steps {
        if !step.selected {
            continue;
        }
        if let Some(previous) = previous_by_name.get(&step.name) {
            if matches!(previous.status, Some(StepStatus::Succeeded)) {
                if previous.command != step.command {
                    bail!(
                        "resume step {:?} command differs from current expanded command",
                        step.name
                    );
                }
                step.status = previous.status;
                step.started_at_unix_ms = previous.started_at_unix_ms;
                step.finished_at_unix_ms = previous.finished_at_unix_ms;
                step.duration_ms = previous.duration_ms;
                step.exit_code = previous.exit_code;
            }
        }
    }
    Ok(())
}

async fn write_manifest_if_requested(
    args: &SuiteRunOptions,
    config: &SuiteConfig,
    manifest: &SuiteManifest,
) -> Result<()> {
    if let Some(path) = manifest_path(args, config) {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .wrap_err_with(|| format!("creating {}", parent.display()))?;
        }
        let body = serde_json::to_string_pretty(manifest)?;
        tokio::fs::write(&path, format!("{body}\n"))
            .await
            .wrap_err_with(|| format!("writing {}", path.display()))?;
        crate::ecaz_eprintln!("[suite:{}] wrote {}", config.name, path.display());
    }
    Ok(())
}

async fn write_results_if_requested(
    args: &SuiteRunOptions,
    config: &SuiteConfig,
    manifest: &SuiteManifest,
) -> Result<Vec<ResultRow>> {
    let rows = extract_result_rows(manifest).await?;
    let path = args.results_output.clone().or_else(|| {
        config
            .artifact_dir
            .as_ref()
            .map(|dir| dir.join("results.jsonl"))
    });
    if let Some(path) = path {
        write_results_jsonl(&path, &rows).await?;
        crate::ecaz_eprintln!("[suite:{}] wrote {}", config.name, path.display());
    }
    Ok(rows)
}

#[derive(Debug, Serialize)]
struct ResultRow {
    suite: String,
    step: String,
    kind: String,
    metric: String,
    artifact: String,
    values: BTreeMap<String, String>,
}

async fn extract_result_rows(manifest: &SuiteManifest) -> Result<Vec<ResultRow>> {
    let mut rows = Vec::new();
    for step in &manifest.steps {
        if !matches!(step.status, Some(StepStatus::Succeeded)) {
            continue;
        }
        for artifact in &step.expected_artifacts {
            let path = Path::new(artifact);
            let Ok(raw) = tokio::fs::read_to_string(path).await else {
                continue;
            };
            rows.extend(parse_result_rows(manifest, step, artifact, &raw));
        }
    }
    Ok(rows)
}

fn parse_result_rows(
    manifest: &SuiteManifest,
    step: &StepRecord,
    artifact: &str,
    raw: &str,
) -> Vec<ResultRow> {
    match step.kind.as_str() {
        "recall" | "latency" => parse_table_rows(raw)
            .into_iter()
            .map(|values| ResultRow {
                suite: manifest.suite.clone(),
                step: step.name.clone(),
                kind: step.kind.clone(),
                metric: step.kind.clone(),
                artifact: artifact.into(),
                values,
            })
            .collect(),
        "storage" => parse_storage_rows(raw)
            .into_iter()
            .map(|(metric, values)| ResultRow {
                suite: manifest.suite.clone(),
                step: step.name.clone(),
                kind: step.kind.clone(),
                metric,
                artifact: artifact.into(),
                values,
            })
            .collect(),
        "load" => parse_load_rows(raw)
            .into_iter()
            .map(|(metric, values)| ResultRow {
                suite: manifest.suite.clone(),
                step: step.name.clone(),
                kind: step.kind.clone(),
                metric,
                artifact: artifact.into(),
                values,
            })
            .collect(),
        "compare-pgvector" | "compare-vectorscale" => {
            let mut rows: Vec<ResultRow> = parse_compare_table_rows(raw)
                .into_iter()
                .map(|values| ResultRow {
                    suite: manifest.suite.clone(),
                    step: step.name.clone(),
                    kind: step.kind.clone(),
                    metric: "compare".into(),
                    artifact: artifact.into(),
                    values,
                })
                .collect();
            rows.extend(
                parse_compare_summary_rows(raw)
                    .into_iter()
                    .map(|(metric, values)| ResultRow {
                        suite: manifest.suite.clone(),
                        step: step.name.clone(),
                        kind: step.kind.clone(),
                        metric,
                        artifact: artifact.into(),
                        values,
                    }),
            );
            rows
        }
        "explain" => parse_explain_rows(raw)
            .into_iter()
            .map(|(metric, values)| ResultRow {
                suite: manifest.suite.clone(),
                step: step.name.clone(),
                kind: step.kind.clone(),
                metric,
                artifact: artifact.into(),
                values,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn parse_table_rows(raw: &str) -> Vec<BTreeMap<String, String>> {
    let mut header: Option<Vec<String>> = None;
    let mut rows = Vec::new();
    for cells in table_lines(raw) {
        if cells.iter().any(|cell| cell.chars().all(|ch| ch == '═')) {
            continue;
        }
        if header.as_ref().map(|h| h.len()) != Some(cells.len()) {
            header = Some(cells);
            continue;
        }
        if let Some(header) = &header {
            rows.push(
                header
                    .iter()
                    .cloned()
                    .zip(cells.into_iter())
                    .collect::<BTreeMap<_, _>>(),
            );
        }
    }
    rows
}

fn parse_storage_rows(raw: &str) -> Vec<(String, BTreeMap<String, String>)> {
    let mut rows = Vec::new();
    for table_row in parse_table_rows(raw) {
        if let (Some(field), Some(value)) = (table_row.get("field"), table_row.get("value")) {
            rows.push((
                "storage_field".into(),
                BTreeMap::from([
                    ("field".into(), field.clone()),
                    ("value".into(), value.clone()),
                ]),
            ));
        } else if table_row.contains_key("index") {
            rows.push(("storage_index".into(), table_row));
        }
    }
    rows
}

fn parse_compare_table_rows(raw: &str) -> Vec<BTreeMap<String, String>> {
    parse_table_rows(raw)
        .into_iter()
        .filter(|row| {
            row.get("engine")
                .map(|engine| !engine.starts_with('Δ'))
                .unwrap_or(false)
        })
        .collect()
}

fn parse_compare_summary_rows(raw: &str) -> Vec<(String, BTreeMap<String, String>)> {
    let mut rows = Vec::new();
    for line in raw.lines() {
        if let Some((name, seconds)) = parse_compare_timed_line(line, "built ") {
            rows.push((
                "compare_build".into(),
                BTreeMap::from([("subject".into(), name), ("seconds".into(), seconds)]),
            ));
        } else if let Some((name, bytes)) = parse_compare_size_line(line) {
            rows.push((
                "compare_index_size".into(),
                BTreeMap::from([("subject".into(), name), ("bytes".into(), bytes)]),
            ));
        }
    }
    rows
}

fn parse_load_rows(raw: &str) -> Vec<(String, BTreeMap<String, String>)> {
    let mut rows = Vec::new();
    for line in raw.lines() {
        if let Some((name, seconds)) = parse_timed_loader_line(line, "copied corpus table ") {
            rows.push((
                "load_timing".into(),
                timed_values("copy_corpus", &name, seconds),
            ));
        } else if let Some((name, seconds)) = parse_timed_loader_line(line, "encoded corpus table ")
        {
            rows.push((
                "load_timing".into(),
                timed_values("encode_corpus", &name, seconds),
            ));
        } else if let Some((name, seconds)) = parse_timed_loader_line(line, "copied queries table ")
        {
            rows.push((
                "load_timing".into(),
                timed_values("copy_queries", &name, seconds),
            ));
        } else if let Some((name, seconds)) = parse_timed_loader_line(line, "built ") {
            rows.push((
                "load_timing".into(),
                timed_values("build_index", &name, seconds),
            ));
        } else if let Some((name, seconds)) = parse_timed_loader_line(line, "completed prefix ") {
            rows.push(("load_timing".into(), timed_values("total", &name, seconds)));
        }
    }
    rows
}

fn parse_compare_timed_line(line: &str, prefix: &str) -> Option<(String, String)> {
    let rest = line
        .trim_start()
        .strip_prefix("[compare] ")?
        .strip_prefix(prefix)?;
    let (name, duration) = rest.rsplit_once(" in ")?;
    Some((name.trim().into(), duration_seconds(duration.trim())?))
}

fn parse_compare_size_line(line: &str) -> Option<(String, String)> {
    let rest = line.trim_start().strip_prefix("[compare] ")?;
    let (name, bytes) = rest.rsplit_once(" pg_relation_size=")?;
    let bytes = bytes.strip_suffix(" bytes")?.trim();
    bytes.parse::<u64>().ok()?;
    Some((name.trim().into(), bytes.into()))
}

fn parse_explain_rows(raw: &str) -> Vec<(String, BTreeMap<String, String>)> {
    parse_table_rows(raw)
        .into_iter()
        .filter(|row| row.contains_key("modeled_total_cost"))
        .map(|row| ("planner_cost".into(), row))
        .collect()
}

fn parse_timed_loader_line(line: &str, prefix: &str) -> Option<(String, String)> {
    let rest = line
        .trim_start()
        .strip_prefix("[loader] ")?
        .strip_prefix(prefix)?;
    let (name, duration) = rest.rsplit_once(" in ")?;
    Some((name.trim().into(), duration_seconds(duration.trim())?))
}

fn timed_values(phase: &str, subject: &str, seconds: String) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("phase".into(), phase.into()),
        ("subject".into(), subject.into()),
        ("seconds".into(), seconds),
    ])
}

fn duration_seconds(value: &str) -> Option<String> {
    let value = value.trim();
    let split_at = value
        .find(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .unwrap_or(value.len());
    let amount = value[..split_at].parse::<f64>().ok()?;
    let unit = value[split_at..].trim();
    let seconds = match unit {
        "ms" => amount / 1000.0,
        "" | "s" => amount,
        "m" | "min" => amount * 60.0,
        _ => return None,
    };
    Some(format!("{seconds:.6}"))
}

fn table_lines(raw: &str) -> Vec<Vec<String>> {
    raw.lines()
        .filter(|line| line.trim_start().starts_with('│'))
        .map(|line| {
            line.trim_matches('│')
                .split('┆')
                .flat_map(|part| part.split('│'))
                .map(|cell| cell.trim().to_string())
                .filter(|cell| !cell.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|cells| !cells.is_empty())
        .collect()
}

async fn write_results_jsonl(path: &Path, rows: &[ResultRow]) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .wrap_err_with(|| format!("creating {}", parent.display()))?;
    }
    let mut body = String::new();
    for row in rows {
        body.push_str(&serde_json::to_string(row)?);
        body.push('\n');
    }
    tokio::fs::write(path, body)
        .await
        .wrap_err_with(|| format!("writing {}", path.display()))
}

fn format_metric_values(values: &BTreeMap<String, String>) -> String {
    values
        .iter()
        .map(|(key, value)| format!("`{key}={value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn selected_step_names(manifest: &SuiteManifest) -> HashSet<&str> {
    manifest
        .steps
        .iter()
        .filter(|step| step.selected)
        .map(|step| step.name.as_str())
        .collect()
}

#[cfg(test)]
fn evaluate_thresholds(thresholds: &[ThresholdConfig], rows: &[ResultRow]) -> Vec<ThresholdResult> {
    let selected_steps: HashSet<&str> = thresholds
        .iter()
        .map(|threshold| threshold.step.as_str())
        .collect();
    evaluate_thresholds_for_steps(thresholds, rows, &selected_steps)
}

fn evaluate_thresholds_for_steps(
    thresholds: &[ThresholdConfig],
    rows: &[ResultRow],
    selected_steps: &HashSet<&str>,
) -> Vec<ThresholdResult> {
    thresholds
        .iter()
        .filter(|threshold| selected_steps.contains(threshold.step.as_str()))
        .map(|threshold| evaluate_threshold(threshold, rows))
        .collect()
}

fn evaluate_threshold(threshold: &ThresholdConfig, rows: &[ResultRow]) -> ThresholdResult {
    let actual = rows
        .iter()
        .filter(|row| row.step == threshold.step && row.metric == threshold.metric)
        .filter(|row| {
            threshold.filters.iter().all(|(key, value)| {
                row.values
                    .get(key)
                    .map(|actual| actual == value)
                    .unwrap_or(false)
            })
        })
        .filter_map(|row| row.values.get(&threshold.field))
        .filter_map(|value| parse_numeric_prefix(value))
        .next();
    let passed = actual
        .map(|actual| compare_threshold(actual, threshold.op, threshold.value))
        .unwrap_or(false);
    ThresholdResult {
        name: threshold.name.clone(),
        step: threshold.step.clone(),
        metric: threshold.metric.clone(),
        filters: threshold.filters.clone(),
        field: threshold.field.clone(),
        op: threshold.op,
        expected: threshold.value,
        actual,
        passed,
        message: match actual {
            Some(actual) => format!(
                "{} {} {:?} {} -> {}",
                threshold.field, actual, threshold.op, threshold.value, passed
            ),
            None => format!(
                "no result row for step={}, metric={}, filters={:?}, field={}",
                threshold.step, threshold.metric, threshold.filters, threshold.field
            ),
        },
    }
}

fn compare_threshold(actual: f64, op: ThresholdOp, expected: f64) -> bool {
    match op {
        ThresholdOp::Gt => actual > expected,
        ThresholdOp::Gte => actual >= expected,
        ThresholdOp::Lt => actual < expected,
        ThresholdOp::Lte => actual <= expected,
        ThresholdOp::Eq => (actual - expected).abs() < f64::EPSILON,
    }
}

fn parse_numeric_prefix(value: &str) -> Option<f64> {
    let value = value.trim();
    let split_at = value
        .find(|ch: char| !(ch.is_ascii_digit() || ch == '.' || ch == '-'))
        .unwrap_or(value.len());
    value[..split_at].parse().ok()
}

fn validate_config(config: &SuiteConfig) -> Result<()> {
    if config.schema_version != 1 {
        bail!(
            "unsupported suite schema_version {}; supported: 1",
            config.schema_version
        );
    }
    if config.steps.is_empty() {
        bail!("suite {:?} has no steps", config.name);
    }
    validate_profile_name("suite defaults profile", config.defaults.profile.as_deref())?;
    let mut names = HashSet::new();
    for step in &config.steps {
        if !names.insert(step.name()) {
            bail!("duplicate suite step name {:?}", step.name());
        }
        step.validate()?;
    }
    Ok(())
}

impl SuiteStep {
    fn name(&self) -> &str {
        match self {
            SuiteStep::CorpusFetch(step) => &step.name,
            SuiteStep::CorpusPrepare(step) => &step.name,
            SuiteStep::Load(step) => &step.name,
            SuiteStep::Recall(step) => &step.name,
            SuiteStep::Latency(step) => &step.name,
            SuiteStep::Storage(step) => &step.name,
            SuiteStep::Explain(step) => &step.name,
            SuiteStep::ComparePgvector(step) => &step.name,
            SuiteStep::CompareVectorscale(step) => &step.name,
            SuiteStep::Raw(step) => &step.name,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            SuiteStep::CorpusFetch(_) => "corpus-fetch",
            SuiteStep::CorpusPrepare(_) => "corpus-prepare",
            SuiteStep::Load(_) => "load",
            SuiteStep::Recall(_) => "recall",
            SuiteStep::Latency(_) => "latency",
            SuiteStep::Storage(_) => "storage",
            SuiteStep::Explain(_) => "explain",
            SuiteStep::ComparePgvector(_) => "compare-pgvector",
            SuiteStep::CompareVectorscale(_) => "compare-vectorscale",
            SuiteStep::Raw(_) => "raw",
        }
    }

    fn tags(&self) -> &[String] {
        match self {
            SuiteStep::CorpusFetch(step) => &step.tags,
            SuiteStep::CorpusPrepare(step) => &step.tags,
            SuiteStep::Load(step) => &step.tags,
            SuiteStep::Recall(step) => &step.tags,
            SuiteStep::Latency(step) => &step.tags,
            SuiteStep::Storage(step) => &step.tags,
            SuiteStep::Explain(step) => &step.tags,
            SuiteStep::ComparePgvector(step) => &step.tags,
            SuiteStep::CompareVectorscale(step) => &step.tags,
            SuiteStep::Raw(step) => &step.tags,
        }
    }

    fn validate(&self) -> Result<()> {
        match self {
            SuiteStep::CorpusPrepare(step) => {
                if step.dim == Some(0) {
                    bail!("corpus-prepare step {:?} must set dim >= 1", step.name)
                }
                if step.chunk_rows == Some(0) {
                    bail!(
                        "corpus-prepare step {:?} must set chunk_rows >= 1",
                        step.name
                    )
                }
                Ok(())
            }
            SuiteStep::Load(step) => {
                validate_profile_name("load profile", step.profile.as_deref())?;
                if step.corpus_file.is_none()
                    && step.queries_file.is_none()
                    && step.manifest_file.is_none()
                {
                    bail!(
                        "load step {:?} must include corpus/queries files or a manifest_file",
                        step.name
                    )
                }
                if step.chunked && (step.corpus_file.is_some() || step.queries_file.is_some()) {
                    bail!(
                        "load step {:?} cannot mix chunked manifest loading with corpus/queries files",
                        step.name
                    )
                }
                if step.chunked && step.manifest_file.is_none() {
                    bail!(
                        "load step {:?} requires manifest_file when chunked=true",
                        step.name
                    )
                }
                if !step.chunked && (step.corpus_file.is_none() || step.queries_file.is_none()) {
                    bail!(
                        "load step {:?} requires corpus_file and queries_file unless chunked=true",
                        step.name
                    )
                }
                Ok(())
            }
            SuiteStep::Recall(step) => {
                validate_profile_name("recall profile", step.profile.as_deref())?;
                if step.sweep.is_empty() {
                    bail!(
                        "recall step {:?} must include at least one sweep value",
                        step.name
                    )
                }
                if step.truth_cache_file.is_some() && step.truth_cache_dir.is_some() {
                    bail!(
                        "recall step {:?} cannot set both truth_cache_file and truth_cache_dir",
                        step.name
                    )
                }
                Ok(())
            }
            SuiteStep::Latency(step) => {
                validate_profile_name("latency profile", step.profile.as_deref())?;
                if step.sweep.is_empty() {
                    bail!(
                        "latency step {:?} must include at least one sweep value",
                        step.name
                    )
                }
                Ok(())
            }
            SuiteStep::Explain(step) => {
                validate_profile_name("explain profile", step.profile.as_deref())
            }
            SuiteStep::ComparePgvector(step) => {
                validate_profile_name("compare-pgvector profile", step.profile.as_deref())?;
                if step.sweep.is_empty() && step.ecaz_sweep.is_none() {
                    bail!(
                        "compare-pgvector step {:?} must include sweep or ecaz_sweep",
                        step.name
                    )
                }
                Ok(())
            }
            SuiteStep::CompareVectorscale(step) => {
                validate_profile_name("compare-vectorscale profile", step.profile.as_deref())?;
                if step.sweep.is_empty() && step.ecaz_sweep.is_none() {
                    bail!(
                        "compare-vectorscale step {:?} must include sweep or ecaz_sweep",
                        step.name
                    )
                }
                Ok(())
            }
            SuiteStep::Raw(step) if step.args.is_empty() => {
                bail!("raw step {:?} must include args", step.name)
            }
            _ => Ok(()),
        }
    }

    fn expand(&self, defaults: &SuiteDefaults, conn: &ConnectionOptions) -> Result<Vec<String>> {
        match self {
            SuiteStep::CorpusFetch(step) => Ok(expand_corpus_fetch(step)),
            SuiteStep::CorpusPrepare(step) => Ok(expand_corpus_prepare(step)),
            SuiteStep::Load(step) => Ok(expand_load(step, defaults)),
            SuiteStep::Recall(step) => Ok(expand_recall(step, defaults)),
            SuiteStep::Latency(step) => Ok(expand_latency(step, defaults)),
            SuiteStep::Storage(step) => Ok(expand_storage(step)),
            SuiteStep::Explain(step) => Ok(expand_explain(step, defaults, conn)),
            SuiteStep::ComparePgvector(step) => Ok(expand_compare_pgvector(step, defaults)),
            SuiteStep::CompareVectorscale(step) => Ok(expand_compare_vectorscale(step, defaults)),
            SuiteStep::Raw(step) => Ok(step.args.clone()),
        }
    }

    fn expected_artifacts(&self) -> Vec<PathBuf> {
        match self {
            SuiteStep::CorpusFetch(step) => vec![step.output_dir.join("ecaz_fetch_manifest.json")],
            SuiteStep::CorpusPrepare(step) => {
                let manifest = step
                    .output_dir
                    .join(format!("{}_manifest.json", step.profile));
                if step.chunk_rows.is_some() {
                    vec![manifest]
                } else {
                    vec![
                        step.output_dir.join(format!("{}_corpus.tsv", step.profile)),
                        step.output_dir
                            .join(format!("{}_queries.tsv", step.profile)),
                        manifest,
                    ]
                }
            }
            SuiteStep::Load(step) => step.log_file.iter().cloned().collect(),
            SuiteStep::Recall(step) => step.log_output.iter().cloned().collect(),
            SuiteStep::Latency(step) => step.log_output.iter().cloned().collect(),
            SuiteStep::Storage(step) => step.log_file.iter().cloned().collect(),
            SuiteStep::Explain(step) => vec![step.sql_file.clone(), step.log_output.clone()],
            SuiteStep::ComparePgvector(step) => step.log_file.iter().cloned().collect(),
            SuiteStep::CompareVectorscale(step) => step.log_file.iter().cloned().collect(),
            SuiteStep::Raw(step) => step.expected_artifacts.clone(),
        }
    }

    fn input_paths(&self) -> Vec<PathBuf> {
        match self {
            SuiteStep::CorpusPrepare(step) => vec![step.parquet.clone()],
            SuiteStep::Load(step) => {
                let mut paths = Vec::new();
                if let Some(path) = &step.corpus_file {
                    paths.push(path.clone());
                }
                if let Some(path) = &step.queries_file {
                    paths.push(path.clone());
                }
                if let Some(path) = &step.manifest_file {
                    paths.push(path.clone());
                }
                paths
            }
            _ => Vec::new(),
        }
    }

    fn produced_paths(&self) -> Vec<PathBuf> {
        match self {
            SuiteStep::CorpusFetch(step) => vec![
                step.output_dir.clone(),
                step.output_dir.join("data"),
                step.output_dir.join("ecaz_fetch_manifest.json"),
            ],
            SuiteStep::CorpusPrepare(step) => {
                let mut paths = vec![
                    step.output_dir.clone(),
                    step.output_dir
                        .join(format!("{}_manifest.json", step.profile)),
                ];
                if step.chunk_rows.is_some() {
                    paths.push(step.output_dir.join(format!("{}_corpus", step.profile)));
                    paths.push(step.output_dir.join(format!("{}_queries", step.profile)));
                } else {
                    paths.push(step.output_dir.join(format!("{}_corpus.tsv", step.profile)));
                    paths.push(
                        step.output_dir
                            .join(format!("{}_queries.tsv", step.profile)),
                    );
                }
                paths
            }
            SuiteStep::Explain(step) => vec![step.sql_file.clone()],
            _ => Vec::new(),
        }
    }
}

fn validate_profile_name(label: &str, profile_name: Option<&str>) -> Result<()> {
    if let Some(profile_name) = profile_name {
        if profiles::resolve(profile_name).is_none() {
            bail!(
                "{label} {:?} is not registered; known profiles: {}",
                profile_name,
                profiles::names().join(", ")
            );
        }
    }
    Ok(())
}

async fn prepare_step(step: &SuiteStep, defaults: &SuiteDefaults) -> Result<()> {
    if let SuiteStep::Explain(step) = step {
        if let Some(parent) = step.sql_file.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .wrap_err_with(|| format!("creating {}", parent.display()))?;
        }
        tokio::fs::write(&step.sql_file, explain_sql(step, defaults))
            .await
            .wrap_err_with(|| format!("writing {}", step.sql_file.display()))?;
    }
    Ok(())
}

async fn spawn_step(exe: &Path, args: &[String], conn: &ConnectionOptions) -> Result<ExitStatus> {
    let mut command = Command::new(exe);
    command.args(args);
    if let Some(password) = &conn.password {
        command.env("PGPASSWORD", password);
    }
    command
        .status()
        .await
        .wrap_err_with(|| format!("spawning {}", exe.display()))
}

fn child_command_args(conn: &ConnectionOptions, mut step_args: Vec<String>) -> Vec<String> {
    let mut args = Vec::new();
    push_arg(&mut args, "--database", &conn.database);
    if let Some(host) = &conn.host {
        push_arg(&mut args, "--host", host);
    }
    if let Some(port) = conn.port {
        push_arg(&mut args, "--port", &port.to_string());
    }
    if let Some(user) = &conn.user {
        push_arg(&mut args, "--user", user);
    }
    args.append(&mut step_args);
    args
}

fn expand_load(step: &LoadStep, defaults: &SuiteDefaults) -> Vec<String> {
    let mut args = Vec::new();
    push_opt_path(&mut args, "--log-file", step.log_file.as_deref());
    args.extend(["corpus".into(), "load".into()]);
    push_arg(&mut args, "--prefix", &step.prefix);
    push_arg(
        &mut args,
        "--profile",
        &profile(defaults, step.profile.as_deref()),
    );
    push_opt_path(&mut args, "--corpus-file", step.corpus_file.as_deref());
    push_opt_path(&mut args, "--queries-file", step.queries_file.as_deref());
    push_opt_path(&mut args, "--manifest-file", step.manifest_file.as_deref());
    if step.allow_manifest_mismatch {
        args.push("--allow-manifest-mismatch".into());
    }
    if step.chunked {
        args.push("--chunked".into());
    }
    if let Some(dim) = step.dim {
        push_arg(&mut args, "--dim", &dim.to_string());
    }
    push_arg(&mut args, "--bits", &bits(defaults, step.bits).to_string());
    push_arg(&mut args, "--seed", &seed(defaults, step.seed).to_string());
    if !step.m.is_empty() {
        push_arg(&mut args, "--m", &join_i32(&step.m));
    }
    if let Some(ef_construction) = step.ef_construction {
        push_arg(&mut args, "--ef-construction", &ef_construction.to_string());
    }
    for reloption in &step.reloptions {
        push_arg(&mut args, "--reloption", reloption);
    }
    args
}

fn expand_corpus_fetch(step: &CorpusFetchStep) -> Vec<String> {
    let mut args = vec!["corpus".into(), "fetch".into()];
    push_arg(&mut args, "--dataset", &step.dataset);
    push_arg_path(&mut args, "--output-dir", &step.output_dir);
    if let Some(revision) = step.revision.as_deref() {
        push_arg(&mut args, "--revision", revision);
    }
    if step.force {
        args.push("--force".into());
    }
    args
}

fn expand_corpus_prepare(step: &CorpusPrepareStep) -> Vec<String> {
    let mut args = vec!["corpus".into(), "prepare".into()];
    push_arg(&mut args, "--profile", &step.profile);
    push_arg_path(&mut args, "--parquet", &step.parquet);
    push_arg_path(&mut args, "--output-dir", &step.output_dir);
    if let Some(id_column) = step.id_column.as_deref() {
        push_arg(&mut args, "--id-column", id_column);
    }
    if let Some(vector_column) = step.vector_column.as_deref() {
        push_arg(&mut args, "--vector-column", vector_column);
    }
    if let Some(dim) = step.dim {
        push_arg(&mut args, "--dim", &dim.to_string());
    }
    if let Some(source_dataset) = step.source_dataset.as_deref() {
        push_arg(&mut args, "--source-dataset", source_dataset);
    }
    if let Some(chunk_rows) = step.chunk_rows {
        push_arg(&mut args, "--chunk-rows", &chunk_rows.to_string());
    }
    args
}

fn expand_recall(step: &RecallStep, defaults: &SuiteDefaults) -> Vec<String> {
    let mut args = vec!["bench".into(), "recall".into()];
    push_arg(&mut args, "--prefix", &step.prefix);
    push_arg(
        &mut args,
        "--profile",
        &profile(defaults, step.profile.as_deref()),
    );
    push_arg(&mut args, "--k", &step.k.to_string());
    push_arg(&mut args, "--sweep", &join_i32(&step.sweep));
    if let Some(width) = step.rerank_width {
        push_arg(&mut args, "--rerank-width", &width.to_string());
    }
    if let Some(limit) = step.queries_limit.or(defaults.queries_limit) {
        push_arg(&mut args, "--queries-limit", &limit.to_string());
    }
    push_arg(&mut args, "--bits", &bits(defaults, step.bits).to_string());
    push_arg(&mut args, "--seed", &seed(defaults, step.seed).to_string());
    if step.force_index.or(defaults.force_index).unwrap_or(false) {
        args.push("--force-index".into());
    }
    push_opt_path(
        &mut args,
        "--truth-cache-file",
        step.truth_cache_file.as_deref(),
    );
    push_opt_path(
        &mut args,
        "--truth-cache-dir",
        step.truth_cache_dir.as_deref(),
    );
    push_opt_path(&mut args, "--log-output", step.log_output.as_deref());
    args
}

fn expand_latency(step: &LatencyStep, defaults: &SuiteDefaults) -> Vec<String> {
    let mut args = vec!["bench".into(), "latency".into()];
    push_arg(&mut args, "--prefix", &step.prefix);
    push_arg(
        &mut args,
        "--profile",
        &profile(defaults, step.profile.as_deref()),
    );
    push_arg(&mut args, "--k", &step.k.unwrap_or(10).to_string());
    push_arg(
        &mut args,
        "--concurrency",
        &step.concurrency.unwrap_or(1).to_string(),
    );
    push_arg(
        &mut args,
        "--iterations",
        &step
            .iterations
            .or(defaults.iterations)
            .unwrap_or(1000)
            .to_string(),
    );
    push_arg(&mut args, "--sweep", &join_i32(&step.sweep));
    if let Some(width) = step.rerank_width {
        push_arg(&mut args, "--rerank-width", &width.to_string());
    }
    push_arg(&mut args, "--bits", &bits(defaults, step.bits).to_string());
    push_arg(&mut args, "--seed", &seed(defaults, step.seed).to_string());
    if step.force_index.or(defaults.force_index).unwrap_or(false) {
        args.push("--force-index".into());
    }
    if step
        .sample_backend_memory
        .or(defaults.sample_backend_memory)
        .unwrap_or(false)
    {
        args.push("--sample-backend-memory".into());
    }
    push_arg(
        &mut args,
        "--memory-sample-interval-ms",
        &step
            .memory_sample_interval_ms
            .or(defaults.memory_sample_interval_ms)
            .unwrap_or(25)
            .to_string(),
    );
    push_opt_path(&mut args, "--log-output", step.log_output.as_deref());
    args
}

fn expand_storage(step: &StorageStep) -> Vec<String> {
    let mut args = Vec::new();
    push_opt_path(&mut args, "--log-file", step.log_file.as_deref());
    args.extend(["bench".into(), "storage".into()]);
    push_arg(&mut args, "--prefix", &step.prefix);
    args
}

fn expand_explain(
    step: &ExplainStep,
    defaults: &SuiteDefaults,
    conn: &ConnectionOptions,
) -> Vec<String> {
    let mut args = vec!["dev".into(), "sql".into()];
    push_arg(
        &mut args,
        "--pg",
        &step.pg.or(defaults.pg).unwrap_or(18).to_string(),
    );
    push_arg(
        &mut args,
        "--db",
        step.db.as_deref().unwrap_or(&conn.database),
    );
    push_opt_path(
        &mut args,
        "--socket-dir",
        step.socket_dir
            .as_deref()
            .or(defaults.socket_dir.as_deref())
            .or(conn.host.as_deref().map(Path::new)),
    );
    if let Some(port) = step.port.or(conn.port) {
        push_arg(&mut args, "--port", &port.to_string());
    }
    args.push("--raw".into());
    push_arg_path(&mut args, "--file", &step.sql_file);
    push_arg_path(&mut args, "--log-output", &step.log_output);
    args
}

fn expand_compare_pgvector(step: &ComparePgvectorStep, defaults: &SuiteDefaults) -> Vec<String> {
    let mut args = Vec::new();
    push_opt_path(&mut args, "--log-file", step.log_file.as_deref());
    args.extend(["compare".into(), "pgvector".into()]);
    push_arg(&mut args, "--prefix", &step.prefix);
    push_arg(
        &mut args,
        "--profile",
        &profile(defaults, step.profile.as_deref()),
    );
    push_arg(&mut args, "--k", &step.k.unwrap_or(10).to_string());
    if let Some(pgvector_am) = step.pgvector_am.as_deref() {
        push_arg(&mut args, "--pgvector-am", pgvector_am);
    }
    if !step.sweep.is_empty() {
        push_arg(&mut args, "--sweep", &join_i32(&step.sweep));
    } else if let Some(ecaz_sweep) = step.ecaz_sweep {
        push_arg(&mut args, "--ecaz-sweep", &ecaz_sweep.to_string());
        push_arg(
            &mut args,
            "--pgvector-ef-search",
            &step.pgvector_ef_search.unwrap_or(ecaz_sweep).to_string(),
        );
    }
    if let Some(pgvector_probes) = step.pgvector_probes {
        push_arg(&mut args, "--pgvector-probes", &pgvector_probes.to_string());
    }
    if let Some(memory) = step.pgvector_maintenance_work_mem.as_deref() {
        push_arg(&mut args, "--pgvector-maintenance-work-mem", memory);
    }
    if let Some(pgvector_m) = step.pgvector_m {
        push_arg(&mut args, "--pgvector-m", &pgvector_m.to_string());
    }
    if let Some(pgvector_ef_construction) = step.pgvector_ef_construction {
        push_arg(
            &mut args,
            "--pgvector-ef-construction",
            &pgvector_ef_construction.to_string(),
        );
    }
    if let Some(pgvector_lists) = step.pgvector_lists {
        push_arg(&mut args, "--pgvector-lists", &pgvector_lists.to_string());
    }
    if let Some(width) = step.rerank_width {
        push_arg(&mut args, "--rerank-width", &width.to_string());
    }
    if let Some(limit) = step.queries_limit.or(defaults.queries_limit) {
        push_arg(&mut args, "--queries-limit", &limit.to_string());
    }
    if step.rebuild {
        args.push("--rebuild".into());
    }
    args
}

fn expand_compare_vectorscale(
    step: &CompareVectorscaleStep,
    defaults: &SuiteDefaults,
) -> Vec<String> {
    let mut args = Vec::new();
    push_opt_path(&mut args, "--log-file", step.log_file.as_deref());
    args.extend(["compare".into(), "vectorscale".into()]);
    push_arg(&mut args, "--prefix", &step.prefix);
    push_arg(
        &mut args,
        "--profile",
        &profile(defaults, step.profile.as_deref()),
    );
    push_arg(&mut args, "--k", &step.k.unwrap_or(10).to_string());
    if !step.sweep.is_empty() {
        push_arg(&mut args, "--sweep", &join_i32(&step.sweep));
    } else if let Some(ecaz_sweep) = step.ecaz_sweep {
        push_arg(&mut args, "--ecaz-sweep", &ecaz_sweep.to_string());
    }
    if let Some(num_neighbors) = step.vectorscale_num_neighbors {
        push_arg(
            &mut args,
            "--vectorscale-num-neighbors",
            &num_neighbors.to_string(),
        );
    }
    if let Some(search_list_size) = step.vectorscale_build_search_list_size {
        push_arg(
            &mut args,
            "--vectorscale-build-search-list-size",
            &search_list_size.to_string(),
        );
    }
    if let Some(max_alpha) = step.vectorscale_max_alpha {
        push_arg(&mut args, "--vectorscale-max-alpha", &max_alpha.to_string());
    }
    if let Some(storage_layout) = step.vectorscale_storage_layout.as_deref() {
        push_arg(&mut args, "--vectorscale-storage-layout", storage_layout);
    }
    if let Some(query_rescore) = step.vectorscale_query_rescore {
        push_arg(
            &mut args,
            "--vectorscale-query-rescore",
            &query_rescore.to_string(),
        );
    }
    if let Some(limit) = step.queries_limit.or(defaults.queries_limit) {
        push_arg(&mut args, "--queries-limit", &limit.to_string());
    }
    if step.rebuild {
        args.push("--rebuild".into());
    }
    args
}

fn explain_sql(step: &ExplainStep, defaults: &SuiteDefaults) -> String {
    let corpus_table = step
        .corpus_table
        .clone()
        .unwrap_or_else(|| format!("{}_corpus", step.prefix));
    let query_table = step
        .query_table
        .clone()
        .unwrap_or_else(|| format!("{}_queries", step.prefix));
    let index = step
        .index_name
        .clone()
        .unwrap_or_else(|| format!("{}_idx", step.prefix));
    let profile = explain_step_profile(step, defaults);
    let scan_guc = profile.ef_search_guc.unwrap_or("ec_ivf.nprobe");
    let rerank_guc = rerank_width_guc(profile);
    let set_rerank_sql = rerank_guc
        .map(|guc| {
            format!(
                "SET {guc} = {rerank_width};\n",
                rerank_width = step.rerank_width
            )
        })
        .unwrap_or_default();
    let current_rerank_sql = rerank_guc
        .map(|guc| format!("current_setting('{guc}') AS rerank_width,\n           "))
        .unwrap_or_default();
    let reset_rerank_sql = rerank_guc
        .map(|guc| format!("RESET {guc};\n"))
        .unwrap_or_default();
    let cost_snapshot_sql = cost_snapshot_function(profile)
        .map(|function| {
            format!(
                "SELECT *\n\
                 FROM {function}('{index}'::regclass);\n\n"
            )
        })
        .unwrap_or_default();
    let cost_tuning_snapshot_sql = cost_tuning_snapshot_function(profile)
        .map(|function| {
            format!(
                "SELECT *\n\
                 FROM {function}('{index}'::regclass);\n\n"
            )
        })
        .unwrap_or_default();
    format!(
        "\\pset pager off\n\
         \\timing on\n\n\
         SET enable_seqscan = off;\n\
         SET {scan_guc} = {nprobe};\n\
         {set_rerank_sql}\n\
         SELECT\n\
           current_setting('server_version') AS server_version,\n\
           current_setting('{scan_guc}') AS sweep_value,\n\
           {current_rerank_sql}'{profile_name}' AS profile;\n\n\
         SELECT\n\
           '{index}' AS index_name,\n\
           pg_relation_size('{index}'::regclass) AS index_bytes,\n\
           pg_size_pretty(pg_relation_size('{index}'::regclass)) AS index_size;\n\n\
         {cost_snapshot_sql}\
         {cost_tuning_snapshot_sql}\
         EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)\n\
         SELECT id\n\
         FROM {corpus_table}\n\
         ORDER BY embedding <#> (\n\
           SELECT source\n\
           FROM {query_table}\n\
           ORDER BY id\n\
           LIMIT 1\n\
         )::real[]\n\
         LIMIT 10;\n\n\
         RESET enable_seqscan;\n\
         RESET {scan_guc};\n\
         {reset_rerank_sql}",
        nprobe = step.nprobe,
        scan_guc = scan_guc,
        set_rerank_sql = set_rerank_sql,
        current_rerank_sql = current_rerank_sql,
        profile_name = profile.name,
        index = index,
        cost_snapshot_sql = cost_snapshot_sql,
        cost_tuning_snapshot_sql = cost_tuning_snapshot_sql,
        corpus_table = corpus_table,
        query_table = query_table,
        reset_rerank_sql = reset_rerank_sql
    )
}

fn explain_step_profile<'a>(
    step: &'a ExplainStep,
    defaults: &'a SuiteDefaults,
) -> &'static IndexProfile {
    let profile_name = step
        .profile
        .as_deref()
        .or(defaults.profile.as_deref())
        .unwrap_or("ec_ivf");
    profiles::resolve(profile_name).unwrap_or(&profiles::EC_IVF)
}

fn rerank_width_guc(profile: &IndexProfile) -> Option<&'static str> {
    match profile.name {
        "ec_ivf" => Some("ec_ivf.rerank_width"),
        "ec_spire" => Some("ec_spire.rerank_width"),
        _ => None,
    }
}

fn cost_snapshot_function(profile: &IndexProfile) -> Option<&'static str> {
    match profile.name {
        "ec_hnsw" => Some("ec_hnsw_index_cost_snapshot"),
        "ec_ivf" => Some("ec_ivf_index_cost_snapshot"),
        "ec_spire" => Some("ec_spire_index_cost_snapshot"),
        _ => None,
    }
}

fn cost_tuning_snapshot_function(profile: &IndexProfile) -> Option<&'static str> {
    match profile.name {
        "ec_spire" => Some("ec_spire_index_cost_tuning_snapshot"),
        _ => None,
    }
}

fn manifest_path(args: &SuiteRunOptions, config: &SuiteConfig) -> Option<PathBuf> {
    args.manifest_output.clone().or_else(|| {
        config
            .artifact_dir
            .as_ref()
            .map(|dir| dir.join("suite-manifest.json"))
    })
}

fn profile(defaults: &SuiteDefaults, step_profile: Option<&str>) -> String {
    step_profile
        .or(defaults.profile.as_deref())
        .unwrap_or("ec_hnsw")
        .to_string()
}

fn bits(defaults: &SuiteDefaults, step_bits: Option<i32>) -> i32 {
    step_bits.or(defaults.bits).unwrap_or(4)
}

fn seed(defaults: &SuiteDefaults, step_seed: Option<i64>) -> i64 {
    step_seed.or(defaults.seed).unwrap_or(42)
}

fn join_i32(values: &[i32]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn push_arg(args: &mut Vec<String>, flag: &str, value: &str) {
    args.push(flag.into());
    args.push(value.into());
}

fn push_arg_path(args: &mut Vec<String>, flag: &str, value: &Path) {
    push_arg(args, flag, &value.display().to_string());
}

fn push_opt_path(args: &mut Vec<String>, flag: &str, value: Option<&Path>) {
    if let Some(value) = value {
        push_arg_path(args, flag, value);
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex::encode(digest)
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn shell_join(args: &[String]) -> String {
    args.iter()
        .map(|arg| {
            if arg.chars().all(|c| {
                c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ':' | '=')
            }) {
                arg.clone()
            } else {
                format!("{arg:?}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_exit_status(status: ExitStatus) -> String {
    status
        .code()
        .map(|code| format!("exit code {code}"))
        .unwrap_or_else(|| "signal termination".into())
}

#[derive(Default)]
struct ManifestSummary {
    completed: usize,
    failed: usize,
    skipped: usize,
    dry_run: usize,
    missing_artifacts: usize,
    stale: usize,
}

async fn summarize_manifest(manifest: &SuiteManifest) -> ManifestSummary {
    let mut summary = ManifestSummary::default();
    for step in &manifest.steps {
        match step.status.unwrap_or(if step.selected {
            StepStatus::Pending
        } else {
            StepStatus::Skipped
        }) {
            StepStatus::Succeeded => summary.completed += 1,
            StepStatus::Failed => summary.failed += 1,
            StepStatus::Skipped => summary.skipped += 1,
            StepStatus::DryRun => summary.dry_run += 1,
            StepStatus::Pending => summary.stale += 1,
        }
        if step.selected
            && matches!(step.status, Some(StepStatus::Succeeded))
            && has_missing_artifact(step).await
        {
            summary.missing_artifacts += 1;
        }
    }
    summary
}

async fn has_missing_artifact(step: &StepRecord) -> bool {
    for artifact in &step.expected_artifacts {
        if tokio::fs::metadata(artifact).await.is_err() {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    struct SuiteOnly {
        #[command(flatten)]
        args: SuiteArgs,
    }

    fn conn() -> ConnectionOptions {
        ConnectionOptions {
            database: "postgres".into(),
            host: Some("/tmp/pg".into()),
            port: Some(28818),
            user: None,
            password: Some("secret".into()),
        }
    }

    #[test]
    fn parses_nested_run_command() {
        let cli = SuiteOnly::try_parse_from([
            "suite",
            "run",
            "--config",
            "suite.json",
            "--dry-run",
            "--only",
            "r10",
            "--only-tag",
            "recall",
            "--resume-from",
            "old-manifest.json",
            "--results-output",
            "results.jsonl",
        ])
        .expect("suite parses");
        match cli.args.command {
            Some(SuiteCommand::Run(args)) => {
                assert_eq!(args.config, PathBuf::from("suite.json"));
                assert!(args.dry_run);
                assert_eq!(args.only, vec!["r10"]);
                assert_eq!(args.only_tag, vec!["recall"]);
                assert_eq!(args.resume_from, Some(PathBuf::from("old-manifest.json")));
                assert_eq!(args.results_output, Some(PathBuf::from("results.jsonl")));
            }
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn parses_legacy_dry_run_alias() {
        let cli = SuiteOnly::try_parse_from([
            "suite",
            "--config",
            "suite.json",
            "--dry-run",
            "--manifest-output",
            "manifest.json",
        ])
        .expect("suite parses");
        assert!(cli.args.command.is_none());
        assert_eq!(cli.args.config, Some(PathBuf::from("suite.json")));
        assert!(cli.args.dry_run);
        assert_eq!(
            cli.args.manifest_output,
            Some(PathBuf::from("manifest.json"))
        );
    }

    #[test]
    fn parses_minimal_suite_config() {
        let cfg: SuiteConfig = serde_json::from_str(
            r#"{
              "name": "smoke",
              "schema_version": 1,
              "steps": [
                {
                  "kind": "recall",
                  "name": "r10",
                  "prefix": "p",
                  "k": 10,
                  "sweep": [48]
                }
              ]
            }"#,
        )
        .unwrap();
        assert_eq!(cfg.name, "smoke");
        assert_eq!(cfg.steps.len(), 1);
        assert_eq!(cfg.steps[0].name(), "r10");
        validate_config(&cfg).unwrap();
    }

    #[test]
    fn rejects_duplicate_step_names() {
        let cfg: SuiteConfig = serde_json::from_str(
            r#"{
              "name": "smoke",
              "schema_version": 1,
              "steps": [
                {"kind": "storage", "name": "same", "prefix": "p"},
                {"kind": "storage", "name": "same", "prefix": "p"}
              ]
            }"#,
        )
        .unwrap();
        assert!(validate_config(&cfg)
            .unwrap_err()
            .to_string()
            .contains("duplicate suite step name"));
    }

    #[test]
    fn rejects_unknown_profile_names() {
        let cfg: SuiteConfig = serde_json::from_str(
            r#"{
              "name": "smoke",
              "schema_version": 1,
              "defaults": {"profile": "missing_am"},
              "steps": [
                {"kind": "storage", "name": "storage", "prefix": "p"}
              ]
            }"#,
        )
        .unwrap();

        assert!(validate_config(&cfg)
            .unwrap_err()
            .to_string()
            .contains("known profiles"));
    }

    #[test]
    fn expands_recall_with_defaults() {
        let defaults = SuiteDefaults {
            profile: Some("ec_ivf".into()),
            queries_limit: Some(100),
            force_index: Some(true),
            ..SuiteDefaults::default()
        };
        let step = RecallStep {
            name: "recall".into(),
            tags: vec!["sweep".into()],
            prefix: "surface".into(),
            k: 10,
            sweep: vec![48, 96],
            rerank_width: Some(500),
            queries_limit: None,
            profile: None,
            bits: None,
            seed: None,
            force_index: None,
            truth_cache_file: Some("truth.json".into()),
            truth_cache_dir: None,
            log_output: Some("recall.log".into()),
        };
        let args = expand_recall(&step, &defaults);
        assert!(args.windows(2).any(|w| w == ["--profile", "ec_ivf"]));
        assert!(args.windows(2).any(|w| w == ["--queries-limit", "100"]));
        assert!(args.contains(&"--force-index".into()));
        assert!(args.windows(2).any(|w| w == ["--sweep", "48,96"]));
    }

    #[test]
    fn expands_chunked_load_without_corpus_query_paths() {
        let defaults = SuiteDefaults {
            profile: Some("ec_ivf".into()),
            bits: Some(4),
            seed: Some(42),
            ..SuiteDefaults::default()
        };
        let step = LoadStep {
            name: "load".into(),
            tags: vec!["load".into()],
            prefix: "surface".into(),
            corpus_file: None,
            queries_file: None,
            manifest_file: Some("stage/anchor_manifest.json".into()),
            allow_manifest_mismatch: false,
            chunked: true,
            dim: None,
            profile: Some("ec_ivf".into()),
            bits: None,
            seed: None,
            m: Vec::new(),
            ef_construction: None,
            reloptions: vec!["nlists=1024".into()],
            log_file: Some("load.log".into()),
        };
        let args = expand_load(&step, &defaults);
        assert!(args.contains(&"--chunked".into()));
        assert!(args
            .windows(2)
            .any(|w| w == ["--manifest-file", "stage/anchor_manifest.json"]));
        assert!(!args.iter().any(|arg| arg == "--corpus-file"));
        assert!(!args.iter().any(|arg| arg == "--queries-file"));
    }

    #[test]
    fn parses_fetch_prepare_suite_config() {
        let cfg: SuiteConfig = serde_json::from_str(
            r#"{
              "name": "scale",
              "schema_version": 1,
              "steps": [
                {
                  "kind": "corpus-fetch",
                  "name": "fetch",
                  "dataset": "dbpedia-openai3-large-1536-1m",
                  "output_dir": "data/fetch"
                },
                {
                  "kind": "corpus-prepare",
                  "name": "prepare",
                  "profile": "ec_hnsw_real_ann_benchmarks_anchor",
                  "parquet": "data/fetch/data",
                  "output_dir": "data/staged",
                  "chunk_rows": 25000
                },
                {
                  "kind": "load",
                  "name": "load",
                  "prefix": "profile_real1m",
                  "manifest_file": "data/staged/ec_hnsw_real_ann_benchmarks_anchor_manifest.json",
                  "chunked": true
                }
              ]
            }"#,
        )
        .unwrap();
        validate_config(&cfg).unwrap();
        assert_eq!(cfg.steps[0].kind(), "corpus-fetch");
        assert_eq!(cfg.steps[1].kind(), "corpus-prepare");
        assert_eq!(cfg.steps[2].kind(), "load");
    }

    #[test]
    fn step_selection_requires_name_and_tag_matches() {
        let step = SuiteStep::Recall(RecallStep {
            name: "recall".into(),
            tags: vec!["recall".into(), "sweep".into()],
            prefix: "surface".into(),
            k: 10,
            sweep: vec![48],
            rerank_width: None,
            queries_limit: None,
            profile: None,
            bits: None,
            seed: None,
            force_index: None,
            truth_cache_file: None,
            truth_cache_dir: None,
            log_output: None,
        });
        let args = SuiteRunOptions {
            config: "suite.json".into(),
            dry_run: true,
            continue_on_error: false,
            only: vec!["recall".into()],
            only_tag: vec!["sweep".into()],
            resume_from: None,
            results_output: None,
            manifest_output: None,
        };
        assert!(step_selected(&step, &args));

        let args = SuiteRunOptions {
            only_tag: vec!["latency".into()],
            ..args
        };
        assert!(!step_selected(&step, &args));
    }

    #[test]
    fn parses_recall_result_table() {
        let rows = parse_table_rows(
            "┌────────┬──────────┬────────┬─────────────┐\n\
             │ nprobe ┆ recall@k ┆ ndcg@k ┆ mean q-time │\n\
             ╞════════╪══════════╪════════╪═════════════╡\n\
             │ 96     ┆ 0.9980   ┆ 0.9997 ┆ 11.00 ms    │\n\
             └────────┴──────────┴────────┴─────────────┘\n",
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("nprobe").map(String::as_str), Some("96"));
        assert_eq!(rows[0].get("recall@k").map(String::as_str), Some("0.9980"));
    }

    #[test]
    fn parses_loader_timing_rows() {
        let rows = parse_load_rows(
            "[loader] copied corpus table p_corpus in 15.02s\n\
             [loader] copied queries table p_queries in 183.48ms\n\
             [loader] completed prefix p in 45.76s\n",
        );
        assert_eq!(rows.len(), 3);
        assert_eq!(
            rows[0].1.get("phase").map(String::as_str),
            Some("copy_corpus")
        );
        assert_eq!(
            rows[1].1.get("seconds").map(String::as_str),
            Some("0.183480")
        );
    }

    #[test]
    fn parses_explain_planner_cost_rows() {
        let rows = parse_explain_rows(
            "┌──────────────────────┬──────────────────────┬────────────────────┐\n\
             │ planner_scan_enabled ┆ modeled_startup_cost ┆ modeled_total_cost │\n\
             ╞══════════════════════╪══════════════════════╪════════════════════╡\n\
             │ t                    ┆ 12.5                 ┆ 37.25              │\n\
             └──────────────────────┴──────────────────────┴────────────────────┘\n",
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "planner_cost");
        assert_eq!(
            rows[0].1.get("modeled_total_cost").map(String::as_str),
            Some("37.25")
        );
    }

    #[test]
    fn evaluates_thresholds_against_result_rows() {
        let rows = vec![ResultRow {
            suite: "suite".into(),
            step: "recall".into(),
            kind: "recall".into(),
            metric: "recall".into(),
            artifact: "recall.log".into(),
            values: BTreeMap::from([("recall@k".into(), "0.9980".into())]),
        }];
        let thresholds = vec![ThresholdConfig {
            name: "recall-floor".into(),
            step: "recall".into(),
            metric: "recall".into(),
            filters: BTreeMap::new(),
            field: "recall@k".into(),
            op: ThresholdOp::Gte,
            value: 0.995,
        }];
        let results = evaluate_thresholds(&thresholds, &rows);
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
        assert_eq!(results[0].actual, Some(0.9980));
    }

    #[test]
    fn threshold_filters_select_matching_sweep_row() {
        let rows = vec![
            ResultRow {
                suite: "suite".into(),
                step: "recall".into(),
                kind: "recall".into(),
                metric: "recall".into(),
                artifact: "recall.log".into(),
                values: BTreeMap::from([
                    ("nprobe".into(), "48".into()),
                    ("recall@k".into(), "0.9820".into()),
                ]),
            },
            ResultRow {
                suite: "suite".into(),
                step: "recall".into(),
                kind: "recall".into(),
                metric: "recall".into(),
                artifact: "recall.log".into(),
                values: BTreeMap::from([
                    ("nprobe".into(), "96".into()),
                    ("recall@k".into(), "0.9980".into()),
                ]),
            },
        ];
        let threshold = ThresholdConfig {
            name: "recall-p96-floor".into(),
            step: "recall".into(),
            metric: "recall".into(),
            filters: BTreeMap::from([("nprobe".into(), "96".into())]),
            field: "recall@k".into(),
            op: ThresholdOp::Gte,
            value: 0.995,
        };
        let result = evaluate_threshold(&threshold, &rows);
        assert!(result.passed);
        assert_eq!(result.actual, Some(0.9980));
    }

    #[test]
    fn skips_thresholds_for_unselected_steps() {
        let rows = vec![ResultRow {
            suite: "suite".into(),
            step: "selected".into(),
            kind: "recall".into(),
            metric: "recall".into(),
            artifact: "selected.log".into(),
            values: BTreeMap::from([("recall@k".into(), "0.9980".into())]),
        }];
        let thresholds = vec![
            ThresholdConfig {
                name: "selected-floor".into(),
                step: "selected".into(),
                metric: "recall".into(),
                filters: BTreeMap::new(),
                field: "recall@k".into(),
                op: ThresholdOp::Gte,
                value: 0.995,
            },
            ThresholdConfig {
                name: "unselected-floor".into(),
                step: "unselected".into(),
                metric: "recall".into(),
                filters: BTreeMap::new(),
                field: "recall@k".into(),
                op: ThresholdOp::Gte,
                value: 0.995,
            },
        ];
        let selected_steps = HashSet::from(["selected"]);
        let results = evaluate_thresholds_for_steps(&thresholds, &rows, &selected_steps);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "selected-floor");
        assert!(results[0].passed);
    }

    #[test]
    fn prefixes_child_commands_with_connection_flags() {
        let args = child_command_args(&conn(), vec!["bench".into(), "storage".into()]);
        assert!(args.windows(2).any(|w| w == ["--database", "postgres"]));
        assert!(args.windows(2).any(|w| w == ["--host", "/tmp/pg"]));
        assert!(args.windows(2).any(|w| w == ["--port", "28818"]));
        assert!(!args.contains(&"--password".into()));
        assert!(args.ends_with(&["bench".into(), "storage".into()]));
    }

    #[test]
    fn expands_explain_with_connection_defaults() {
        let defaults = SuiteDefaults::default();
        let step = ExplainStep {
            name: "explain".into(),
            tags: Vec::new(),
            prefix: "pfx".into(),
            profile: None,
            index_name: None,
            query_table: None,
            corpus_table: None,
            nprobe: 96,
            rerank_width: 1000,
            pg: None,
            db: None,
            socket_dir: None,
            port: None,
            sql_file: "explain.sql".into(),
            log_output: "explain.log".into(),
        };
        let args = expand_explain(&step, &defaults, &conn());
        assert!(args.windows(2).any(|w| w == ["--db", "postgres"]));
        assert!(args.windows(2).any(|w| w == ["--socket-dir", "/tmp/pg"]));
        assert!(args.windows(2).any(|w| w == ["--port", "28818"]));
        assert!(args.windows(2).any(|w| w == ["--file", "explain.sql"]));
    }

    #[test]
    fn explain_sql_uses_suite_fields() {
        let step = ExplainStep {
            name: "explain".into(),
            tags: Vec::new(),
            prefix: "pfx".into(),
            profile: None,
            index_name: None,
            query_table: None,
            corpus_table: None,
            nprobe: 96,
            rerank_width: 1000,
            pg: None,
            db: None,
            socket_dir: None,
            port: None,
            sql_file: "explain.sql".into(),
            log_output: "explain.log".into(),
        };
        let sql = explain_sql(&step, &SuiteDefaults::default());
        assert!(sql.contains("SET ec_ivf.nprobe = 96;"));
        assert!(sql.contains("SET ec_ivf.rerank_width = 1000;"));
        assert!(sql.contains("FROM ec_ivf_index_cost_snapshot('pfx_idx'::regclass);"));
        assert!(sql.contains("FROM pfx_corpus"));
        assert!(sql.contains("FROM pfx_queries"));
        assert!(sql.contains("'pfx_idx'::regclass"));
    }

    #[test]
    fn explain_sql_uses_spire_profile_gucs_and_cost_snapshot() {
        let step = ExplainStep {
            name: "explain".into(),
            tags: Vec::new(),
            prefix: "spire_pfx".into(),
            profile: Some("ec_spire".into()),
            index_name: None,
            query_table: None,
            corpus_table: None,
            nprobe: 32,
            rerank_width: 500,
            pg: None,
            db: None,
            socket_dir: None,
            port: None,
            sql_file: "explain.sql".into(),
            log_output: "explain.log".into(),
        };
        let sql = explain_sql(&step, &SuiteDefaults::default());

        assert!(sql.contains("SET ec_spire.nprobe = 32;"));
        assert!(sql.contains("SET ec_spire.rerank_width = 500;"));
        assert!(sql.contains("FROM ec_spire_index_cost_snapshot('spire_pfx_idx'::regclass);"));
        assert!(
            sql.contains("FROM ec_spire_index_cost_tuning_snapshot('spire_pfx_idx'::regclass);")
        );
        assert!(sql.contains("'ec_spire' AS profile"));
        assert!(sql.contains("RESET ec_spire.nprobe;"));
        assert!(sql.contains("RESET ec_spire.rerank_width;"));
    }
}
