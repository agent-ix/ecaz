//! `ecaz bench suite` — configured benchmark suite runner.
//!
//! Suites are JSON plans that expand into ordinary `ecaz` commands. The runner
//! keeps the expansion visible in a manifest, then optionally executes each
//! selected step in sequence.

use clap::{Args, Subcommand};
use color_eyre::eyre::{bail, Context, ContextCompat, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::process::Command;

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
}

#[derive(Debug)]
struct SuiteRunOptions {
    config: PathBuf,
    dry_run: bool,
    continue_on_error: bool,
    only: Vec<String>,
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
    steps: Vec<SuiteStep>,
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
    Load(LoadStep),
    Recall(RecallStep),
    Latency(LatencyStep),
    Storage(StorageStep),
    Explain(ExplainStep),
    Raw(RawStep),
}

#[derive(Debug, Deserialize)]
struct LoadStep {
    name: String,
    prefix: String,
    corpus_file: PathBuf,
    queries_file: PathBuf,
    #[serde(default)]
    manifest_file: Option<PathBuf>,
    #[serde(default)]
    allow_manifest_mismatch: bool,
    #[serde(default)]
    dim: Option<usize>,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    bits: Option<i32>,
    #[serde(default)]
    seed: Option<i64>,
    #[serde(default)]
    reloptions: Vec<String>,
    #[serde(default)]
    log_file: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct RecallStep {
    name: String,
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
    prefix: String,
    #[serde(default)]
    log_file: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct ExplainStep {
    name: String,
    prefix: String,
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
struct RawStep {
    name: String,
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
        Some(SuiteCommand::Report(report_args)) => report_manifest(&report_args.manifest).await,
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
            manifest_output: args.manifest_output,
        }
    }
}

async fn run_suite(conn: &ConnectionOptions, args: SuiteRunOptions) -> Result<()> {
    let (raw, config) = load_config(&args.config).await?;
    validate_config(&config)?;

    let mut manifest = build_manifest(conn, &args, &raw, &config)?;
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
        prepare_step(&config.steps[idx]).await?;
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

    Ok(())
}

async fn audit_suite(config_path: &Path) -> Result<()> {
    let (_raw, config) = load_config(config_path).await?;
    let mut findings = Vec::new();
    if let Err(err) = validate_config(&config) {
        findings.push(err.to_string());
    }
    for step in &config.steps {
        for input in step.input_paths() {
            if tokio::fs::metadata(&input).await.is_err() {
                findings.push(format!(
                    "step {:?} references missing input {}",
                    step.name(),
                    input.display()
                ));
            }
        }
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

async fn report_manifest(path: &Path) -> Result<()> {
    let manifest = load_manifest(path).await?;
    let summary = summarize_manifest(&manifest).await;
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
    };

    for step in &config.steps {
        let selected = args.only.is_empty() || args.only.iter().any(|only| only == step.name());
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
            SuiteStep::Load(step) => &step.name,
            SuiteStep::Recall(step) => &step.name,
            SuiteStep::Latency(step) => &step.name,
            SuiteStep::Storage(step) => &step.name,
            SuiteStep::Explain(step) => &step.name,
            SuiteStep::Raw(step) => &step.name,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            SuiteStep::Load(_) => "load",
            SuiteStep::Recall(_) => "recall",
            SuiteStep::Latency(_) => "latency",
            SuiteStep::Storage(_) => "storage",
            SuiteStep::Explain(_) => "explain",
            SuiteStep::Raw(_) => "raw",
        }
    }

    fn validate(&self) -> Result<()> {
        match self {
            SuiteStep::Recall(step) if step.sweep.is_empty() => {
                bail!(
                    "recall step {:?} must include at least one sweep value",
                    step.name
                )
            }
            SuiteStep::Recall(step)
                if step.truth_cache_file.is_some() && step.truth_cache_dir.is_some() =>
            {
                bail!(
                    "recall step {:?} cannot set both truth_cache_file and truth_cache_dir",
                    step.name
                )
            }
            SuiteStep::Latency(step) if step.sweep.is_empty() => {
                bail!(
                    "latency step {:?} must include at least one sweep value",
                    step.name
                )
            }
            SuiteStep::Raw(step) if step.args.is_empty() => {
                bail!("raw step {:?} must include args", step.name)
            }
            _ => Ok(()),
        }
    }

    fn expand(&self, defaults: &SuiteDefaults, conn: &ConnectionOptions) -> Result<Vec<String>> {
        match self {
            SuiteStep::Load(step) => Ok(expand_load(step, defaults)),
            SuiteStep::Recall(step) => Ok(expand_recall(step, defaults)),
            SuiteStep::Latency(step) => Ok(expand_latency(step, defaults)),
            SuiteStep::Storage(step) => Ok(expand_storage(step)),
            SuiteStep::Explain(step) => Ok(expand_explain(step, defaults, conn)),
            SuiteStep::Raw(step) => Ok(step.args.clone()),
        }
    }

    fn expected_artifacts(&self) -> Vec<PathBuf> {
        match self {
            SuiteStep::Load(step) => step.log_file.iter().cloned().collect(),
            SuiteStep::Recall(step) => step.log_output.iter().cloned().collect(),
            SuiteStep::Latency(step) => step.log_output.iter().cloned().collect(),
            SuiteStep::Storage(step) => step.log_file.iter().cloned().collect(),
            SuiteStep::Explain(step) => vec![step.sql_file.clone(), step.log_output.clone()],
            SuiteStep::Raw(step) => step.expected_artifacts.clone(),
        }
    }

    fn input_paths(&self) -> Vec<PathBuf> {
        match self {
            SuiteStep::Load(step) => {
                let mut paths = vec![step.corpus_file.clone(), step.queries_file.clone()];
                if let Some(path) = &step.manifest_file {
                    paths.push(path.clone());
                }
                paths
            }
            _ => Vec::new(),
        }
    }
}

async fn prepare_step(step: &SuiteStep) -> Result<()> {
    if let SuiteStep::Explain(step) = step {
        if let Some(parent) = step.sql_file.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .wrap_err_with(|| format!("creating {}", parent.display()))?;
        }
        tokio::fs::write(&step.sql_file, explain_sql(step))
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
    push_arg_path(&mut args, "--corpus-file", &step.corpus_file);
    push_arg_path(&mut args, "--queries-file", &step.queries_file);
    push_opt_path(&mut args, "--manifest-file", step.manifest_file.as_deref());
    if step.allow_manifest_mismatch {
        args.push("--allow-manifest-mismatch".into());
    }
    if let Some(dim) = step.dim {
        push_arg(&mut args, "--dim", &dim.to_string());
    }
    push_arg(&mut args, "--bits", &bits(defaults, step.bits).to_string());
    push_arg(&mut args, "--seed", &seed(defaults, step.seed).to_string());
    for reloption in &step.reloptions {
        push_arg(&mut args, "--reloption", reloption);
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

fn explain_sql(step: &ExplainStep) -> String {
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
    format!(
        "\\pset pager off\n\
         \\timing on\n\n\
         SET enable_seqscan = off;\n\
         SET ec_ivf.nprobe = {nprobe};\n\
         SET ec_ivf.rerank_width = {rerank_width};\n\n\
         SELECT\n\
           current_setting('server_version') AS server_version,\n\
           current_setting('ec_ivf.nprobe') AS nprobe,\n\
           current_setting('ec_ivf.rerank_width') AS rerank_width;\n\n\
         SELECT\n\
           '{index}' AS index_name,\n\
           pg_relation_size('{index}'::regclass) AS index_bytes,\n\
           pg_size_pretty(pg_relation_size('{index}'::regclass)) AS index_size;\n\n\
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
         RESET ec_ivf.nprobe;\n\
         RESET ec_ivf.rerank_width;\n",
        nprobe = step.nprobe,
        rerank_width = step.rerank_width,
        index = index,
        corpus_table = corpus_table,
        query_table = query_table
    )
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
        ])
        .expect("suite parses");
        match cli.args.command {
            Some(SuiteCommand::Run(args)) => {
                assert_eq!(args.config, PathBuf::from("suite.json"));
                assert!(args.dry_run);
                assert_eq!(args.only, vec!["r10"]);
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
    fn expands_recall_with_defaults() {
        let defaults = SuiteDefaults {
            profile: Some("ec_ivf".into()),
            queries_limit: Some(100),
            force_index: Some(true),
            ..SuiteDefaults::default()
        };
        let step = RecallStep {
            name: "recall".into(),
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
            prefix: "pfx".into(),
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
            prefix: "pfx".into(),
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
        let sql = explain_sql(&step);
        assert!(sql.contains("SET ec_ivf.nprobe = 96;"));
        assert!(sql.contains("SET ec_ivf.rerank_width = 1000;"));
        assert!(sql.contains("FROM pfx_corpus"));
        assert!(sql.contains("FROM pfx_queries"));
        assert!(sql.contains("'pfx_idx'::regclass"));
    }
}
