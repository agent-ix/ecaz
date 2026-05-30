//! `ecaz corpus load` — canonical real-corpus loader for Postgres fixtures.
//!
//! See the module-level doc in `super` for the corpus model. This command
//! is the only way new data enters Postgres; everything downstream assumes
//! the `<prefix>_corpus` / `<prefix>_queries` contract it establishes.
//!
//! The flow is idempotent: an existing non-empty corpus/query table is
//! left alone, and an index whose reloptions already match the requested
//! set is kept as-is. This preserves the "load once, rerun forever"
//! discipline that makes the real-corpus benchmarks cheap to iterate.

use bytes::{BufMut, BytesMut};
use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use futures::SinkExt;
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio_postgres::{Client, Transaction};

use crate::manifest;
use crate::profiles::{self, IndexProfile};
use crate::psql::{self, ConnectionOptions};
use crate::reloptions;
use crate::tsv;

const DEFAULT_HNSW_BUILD_SOURCE_COLUMN: &str = "source";
const DEFAULT_HNSW_EF_CONSTRUCTION: i32 = 128;
const DEFAULT_HNSW_M_SWEEP: &[i32] = &[8, 16];
const HNSW_ONLY_RELOPTIONS: &[&str] = &["m", "ef_construction", "build_source_column"];
/// Flush the COPY sink at roughly this size. Large enough to amortise the
/// async send overhead, small enough that a 10M-row corpus still surfaces
/// progress before finishing.
const COPY_CHUNK_BYTES: usize = 1 << 20;

#[derive(Args, Debug)]
pub struct LoadArgs {
    /// Fixture prefix used for table and index names. Must match
    /// [a-zA-Z_][a-zA-Z0-9_]*.
    #[arg(long)]
    pub prefix: String,

    /// Path to <basename>_corpus.tsv (one `id\t<json_array>` per line).
    #[arg(long)]
    pub corpus_file: Option<PathBuf>,

    /// Path to <basename>_queries.tsv (one `id\t<json_array>` per line).
    #[arg(long)]
    pub queries_file: Option<PathBuf>,

    /// Vector dimensionality.
    #[arg(long, default_value_t = 1536)]
    pub dim: usize,

    /// Access-method profile (drives embedding type, encoder, opclass).
    #[arg(long, default_value = "ec_hnsw")]
    pub profile: String,

    /// Quantization bits passed to the profile's encoder.
    #[arg(long, default_value_t = 4)]
    pub bits: i32,

    /// Quantizer seed passed to the profile's encoder.
    #[arg(long, default_value_t = 42)]
    pub seed: i64,

    /// HNSW-only: m values to sweep. Accepts `--m 8,16` or repeated `--m 8 --m 16`.
    #[arg(long, value_delimiter = ',')]
    pub m: Vec<i32>,

    /// HNSW-only: ef_construction passed to CREATE INDEX.
    #[arg(long)]
    pub ef_construction: Option<i32>,

    /// Optional storage format (turboquant / pq_fastscan).
    #[arg(long)]
    pub storage_format: Option<String>,

    /// Optional exact index name for single-index profiles.
    ///
    /// This is primarily used by SPIRE remote shard materialization, where the
    /// coordinator descriptor already names the remote index regclass.
    #[arg(long)]
    pub index_name: Option<String>,

    /// AM-specific reloption passthrough. Repeatable.
    /// Example: `--reloption graph_degree=48 --reloption alpha=1.2`.
    #[arg(long = "reloption", value_parser = crate::reloptions::parse_cli)]
    pub reloptions: Vec<(String, String)>,

    /// Optional manifest file path (auto-discovered when corpus/queries files
    /// follow the `<basename>_{corpus,queries}.tsv` convention).
    #[arg(long)]
    pub manifest_file: Option<PathBuf>,

    /// Continue past manifest verification failures with a warning.
    #[arg(long)]
    pub allow_manifest_mismatch: bool,

    /// Force chunked-manifest loading via `--manifest-file`.
    #[arg(long)]
    pub chunked: bool,

    /// Load and index the corpus table only. Query tables are intentionally
    /// skipped for remote SPIRE shard materialization.
    #[arg(long)]
    pub corpus_only: bool,

    /// Static SPIRE remote placement config for distributed corpus loading.
    ///
    /// This is accepted only with `--profile ec_spire`. Until Task 30 Phase 13e.1
    /// materializes remote shards, a valid config fails closed instead of
    /// silently producing a local-only fixture.
    #[arg(long)]
    pub distributed_placement_config: Option<PathBuf>,

    /// Directory for static SPIRE distributed-load artifacts.
    ///
    /// When paired with `--distributed-placement-config`, the loader splits
    /// corpus rows into per-remote TSV files and writes a placement plan JSON,
    /// then exits before any local-only coordinator load can occur.
    #[arg(long)]
    pub distributed_placement_output_dir: Option<PathBuf>,
}

#[derive(Debug)]
struct IndexJob {
    name: String,
    reloptions: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoadKind {
    Corpus,
    Queries,
}

impl LoadKind {
    fn as_str(self) -> &'static str {
        match self {
            LoadKind::Corpus => "corpus",
            LoadKind::Queries => "queries",
        }
    }
}

#[derive(Debug, Clone)]
struct ChunkStateRow {
    chunk_path: String,
    chunk_sha256: String,
    row_count: i64,
}

struct LoadedChunkedManifest {
    manifest: manifest::ChunkedManifest,
    base_dir: PathBuf,
}

#[derive(Debug)]
struct LoadedDistributedPlacementConfig {
    path: PathBuf,
    config: DistributedPlacementConfig,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DistributedPlacementConfig {
    version: u32,
    coordinator: DistributedPlacementCoordinator,
    remotes: Vec<DistributedPlacementRemote>,
    shard_policy: DistributedShardPolicy,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DistributedPlacementCoordinator {
    index_name: String,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DistributedPlacementRemote {
    node_id: u32,
    conninfo_secret_name: String,
    remote_index_regclass: String,
    shard_ids: Vec<u32>,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DistributedShardPolicy {
    kind: String,
    shard_count: u32,
    source_identity_column: String,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
struct DistributedPlacementOutputPlan {
    version: u32,
    prefix: String,
    profile: String,
    dimension: usize,
    bits: i32,
    seed: i64,
    storage_format: Option<String>,
    reloptions: Vec<String>,
    coordinator_index_name: String,
    source_identity_column: String,
    shard_policy: String,
    shard_count: u32,
    total_rows: usize,
    remotes: Vec<DistributedRemoteOutputPlan>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
struct DistributedRemoteOutputPlan {
    node_id: u32,
    conninfo_secret_name: String,
    conninfo_provider_lookup_key: String,
    remote_index_regclass: String,
    remote_prefix: String,
    shard_ids: Vec<u32>,
    corpus_file: String,
    remote_load_args: Vec<String>,
    remote_identity_query_sql: String,
    coordinator_register_descriptor_sql_template: String,
    row_count: usize,
    shard_row_counts: Vec<DistributedShardRowCount>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
struct DistributedShardRowCount {
    shard_id: u32,
    row_count: usize,
}

struct DistributedRemoteWriter {
    node_id: u32,
    conninfo_secret_name: String,
    conninfo_provider_lookup_key: String,
    remote_index_regclass: String,
    remote_prefix: String,
    shard_ids: Vec<u32>,
    corpus_file: PathBuf,
    writer: std::io::BufWriter<std::fs::File>,
    row_count: usize,
    shard_row_counts: BTreeMap<u32, usize>,
}

pub async fn run(conn: &ConnectionOptions, args: LoadArgs) -> Result<()> {
    let total_started = Instant::now();
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    let profile = profiles::resolve(&args.profile).ok_or_else(|| {
        eyre!(
            "unknown profile {:?}; try {}",
            args.profile,
            profiles::names().join(", ")
        )
    })?;

    if !profile.sweep_axis_is_m() && !args.m.is_empty() {
        return Err(eyre!(unsupported_m_error(profile)));
    }
    if !profile.sweep_axis_is_m() && args.ef_construction.is_some() {
        return Err(eyre!(unsupported_ef_construction_error(profile)));
    }
    if let Some(index_name) = args.index_name.as_deref() {
        validate_qualified_ident(index_name)
            .wrap_err_with(|| format!("invalid --index-name {:?}", index_name))?;
    }
    let hnsw_only_reloptions = foreign_hnsw_reloption_keys(profile, &args.reloptions);
    if !hnsw_only_reloptions.is_empty() {
        return Err(eyre!(unsupported_hnsw_reloption_error(
            profile,
            &hnsw_only_reloptions
        )));
    }

    let unknown = profile.unknown_reloption_keys(&args.reloptions);
    if !unknown.is_empty() {
        crate::ecaz_eprintln!(
            "[loader] warning: profile {:?} does not list {} as known reloption{}; \
             passing through verbatim. Known reloptions: {}",
            profile.name,
            unknown.join(", "),
            if unknown.len() == 1 { "" } else { "s" },
            profile.known_reloptions.join(", ")
        );
    }

    let collisions =
        reloption_flag_collisions(profile, &args.reloptions, args.storage_format.as_deref());
    if !collisions.is_empty() {
        let formatted = collisions
            .iter()
            .map(|c| format!("--reloption {}=... conflicts with {}", c.key, c.flag))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(eyre!(
            "{formatted}. Use the native CLI flag or drop the --reloption, not both"
        ));
    }

    let distributed_placement_config = load_distributed_placement_config_if_requested(
        args.distributed_placement_config.as_deref(),
        profile,
    )?;
    if distributed_placement_config.is_some() && args.distributed_placement_output_dir.is_none() {
        return Err(eyre!(
            "--distributed-placement-config requires --distributed-placement-output-dir so distributed corpus rows are materialized explicitly instead of falling back to local-only load"
        ));
    }

    let corpus_table = format!("{}_corpus", args.prefix);
    let queries_table = format!("{}_queries", args.prefix);
    let index_prefix = match args.storage_format.as_deref() {
        Some(sf) => format!("{}_{sf}", args.prefix),
        None => args.prefix.clone(),
    };
    let index_jobs = plan_index_jobs_with_optional_name(
        profile,
        &index_prefix,
        &args.m,
        args.ef_construction.unwrap_or(DEFAULT_HNSW_EF_CONSTRUCTION),
        args.storage_format.as_deref(),
        &args.reloptions,
        args.index_name.as_deref(),
    )?;
    let chunked_manifest = load_chunked_manifest_if_requested(
        args.manifest_file.as_deref(),
        args.chunked,
        &args.prefix,
        args.dim,
    )?;
    if args.chunked && chunked_manifest.is_none() {
        return Err(eyre!(
            "--chunked requires a chunked manifest passed via --manifest-file"
        ));
    }

    if let Some(chunked_manifest) = chunked_manifest {
        if let Some(config) = &distributed_placement_config {
            let output_dir = args
                .distributed_placement_output_dir
                .as_deref()
                .expect("validated distributed placement output dir should exist");
            let corpus_paths = chunked_manifest
                .manifest
                .corpus
                .chunks
                .iter()
                .map(|chunk| chunked_manifest.base_dir.join(&chunk.path))
                .collect::<Vec<_>>();
            let plan = write_distributed_placement_outputs(
                output_dir,
                config,
                &args.prefix,
                profile,
                &corpus_paths,
                args.dim,
                args.bits,
                args.seed,
                args.storage_format.as_deref(),
                &args.reloptions,
            )?;
            print_distributed_placement_output_summary(output_dir, &plan);
            return Ok(());
        }
        let mut client = psql::connect(conn).await?;
        let corpus_table = format!("{}_corpus", args.prefix);
        let queries_table = format!("{}_queries", args.prefix);
        let corpus_loaded = ensure_chunked_corpus_table(
            &mut client,
            &corpus_table,
            &chunked_manifest,
            args.bits,
            args.seed,
            profile,
        )
        .await?;
        let queries_loaded = if args.corpus_only {
            None
        } else {
            Some(
                ensure_chunked_queries_table(&mut client, &queries_table, &chunked_manifest)
                    .await?,
            )
        };
        for job in &index_jobs {
            ensure_index(&client, &corpus_table, job, profile).await?;
        }
        print_summary(
            profile,
            &corpus_table,
            corpus_loaded,
            if args.corpus_only {
                None
            } else {
                Some(queries_table.as_str())
            },
            queries_loaded,
            &index_jobs,
        );
        crate::ecaz_eprintln!(
            "[loader] completed prefix {} in {:.2?}",
            args.prefix,
            total_started.elapsed()
        );
        return Ok(());
    }

    let corpus_file = args.corpus_file.as_deref().ok_or_else(|| {
        eyre!("--corpus-file is required unless --manifest-file points to a chunked manifest")
    })?;
    let queries_file = if args.corpus_only {
        None
    } else {
        Some(args.queries_file.as_deref().ok_or_else(|| {
            eyre!("--queries-file is required unless --manifest-file points to a chunked manifest or --corpus-only is set")
        })?)
    };

    // Inspect inputs first: row counts drive progress bars and manifest
    // verification, and we want to fail fast on malformed files before we
    // open any transactions.
    crate::ecaz_eprintln!("[loader] inspecting {}", corpus_file.display());
    let corpus_stats = tsv::inspect(corpus_file, args.dim)?;
    let query_stats = if let Some(queries_file) = queries_file {
        crate::ecaz_eprintln!("[loader] inspecting {}", queries_file.display());
        Some(tsv::inspect(queries_file, args.dim)?)
    } else {
        None
    };

    if let Some(query_stats) = &query_stats {
        crate::ecaz_eprintln!(
            "[loader] corpus: {} rows, sha256={}  queries: {} rows, sha256={}",
            corpus_stats.rows,
            corpus_stats.sha256_hex,
            query_stats.rows,
            query_stats.sha256_hex
        );
    } else {
        crate::ecaz_eprintln!(
            "[loader] corpus: {} rows, sha256={}  queries: skipped (--corpus-only)",
            corpus_stats.rows,
            corpus_stats.sha256_hex
        );
    }

    if let (Some(queries_file), Some(query_stats)) = (queries_file, query_stats.as_ref()) {
        verify_manifest_if_present(
            args.manifest_file.as_deref(),
            corpus_file,
            queries_file,
            &args.prefix,
            args.dim,
            &corpus_stats,
            query_stats,
            args.allow_manifest_mismatch,
        )?;
    } else if args.manifest_file.is_some() {
        return Err(eyre!(
            "--manifest-file verification requires --queries-file unless --manifest-file points to a chunked manifest; omit --manifest-file for --corpus-only remote shard loads"
        ));
    }

    if let Some(config) = &distributed_placement_config {
        let output_dir = args
            .distributed_placement_output_dir
            .as_deref()
            .expect("validated distributed placement output dir should exist");
        let plan = write_distributed_placement_outputs(
            output_dir,
            config,
            &args.prefix,
            profile,
            &[corpus_file.to_path_buf()],
            args.dim,
            args.bits,
            args.seed,
            args.storage_format.as_deref(),
            &args.reloptions,
        )?;
        print_distributed_placement_output_summary(output_dir, &plan);
        return Ok(());
    }

    let mut client = psql::connect(conn).await?;
    client
        .batch_execute("CREATE EXTENSION IF NOT EXISTS ecaz")
        .await
        .wrap_err("ensuring ecaz extension")?;

    let corpus_loaded = ensure_corpus_table(
        &mut client,
        &corpus_table,
        corpus_file,
        args.dim,
        args.bits,
        args.seed,
        profile,
        corpus_stats.rows,
    )
    .await?;
    let queries_loaded =
        if let (Some(queries_file), Some(query_stats)) = (queries_file, query_stats) {
            Some(
                ensure_queries_table(
                    &client,
                    &queries_table,
                    queries_file,
                    args.dim,
                    query_stats.rows,
                )
                .await?,
            )
        } else {
            None
        };

    for job in &index_jobs {
        ensure_index(&client, &corpus_table, job, profile).await?;
    }

    print_summary(
        profile,
        &corpus_table,
        corpus_loaded,
        if args.corpus_only {
            None
        } else {
            Some(queries_table.as_str())
        },
        queries_loaded,
        &index_jobs,
    );
    crate::ecaz_eprintln!(
        "[loader] completed prefix {} in {:.2?}",
        args.prefix,
        total_started.elapsed()
    );
    Ok(())
}

fn load_chunked_manifest_if_requested(
    path: Option<&Path>,
    force_chunked: bool,
    prefix: &str,
    dim: usize,
) -> Result<Option<LoadedChunkedManifest>> {
    let Some(path) = path else {
        return Ok(None);
    };
    let raw = std::fs::read_to_string(path)
        .wrap_err_with(|| format!("reading manifest {}", path.display()))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .wrap_err_with(|| format!("parsing manifest {}", path.display()))?;
    if !force_chunked && !manifest::is_chunked_manifest(&parsed) {
        return Ok(None);
    }
    let chunked = manifest::parse_chunked_manifest(&parsed)?;
    if chunked.prefix != prefix {
        return Err(eyre!(
            "manifest prefix {:?} does not match --prefix {:?}",
            chunked.prefix, prefix
        ));
    }
    if chunked.dimension != dim {
        return Err(eyre!(
            "manifest dimension {} does not match --dim {}",
            chunked.dimension, dim
        ));
    }
    Ok(Some(LoadedChunkedManifest {
        manifest: chunked,
        base_dir: path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(".")),
    }))
}

fn load_distributed_placement_config_if_requested(
    path: Option<&Path>,
    profile: &IndexProfile,
) -> Result<Option<LoadedDistributedPlacementConfig>> {
    let Some(path) = path else {
        return Ok(None);
    };
    if profile.name != "ec_spire" {
        return Err(eyre!(
            "--distributed-placement-config is only supported with --profile ec_spire, not {:?}",
            profile.name
        ));
    }
    let raw = std::fs::read_to_string(path)
        .wrap_err_with(|| format!("reading distributed placement config {}", path.display()))?;
    let config: DistributedPlacementConfig = serde_json::from_str(&raw)
        .wrap_err_with(|| format!("parsing distributed placement config {}", path.display()))?;
    validate_distributed_placement_config(&config)?;
    Ok(Some(LoadedDistributedPlacementConfig {
        path: path.to_path_buf(),
        config,
    }))
}

fn validate_distributed_placement_config(config: &DistributedPlacementConfig) -> Result<()> {
    if config.version != 1 {
        return Err(eyre!(
            "distributed placement config version {} is not supported; expected 1",
            config.version
        ));
    }
    if config.coordinator.index_name.trim().is_empty() {
        return Err(eyre!(
            "distributed placement config coordinator.index_name must not be empty"
        ));
    }
    if config.remotes.is_empty() {
        return Err(eyre!(
            "distributed placement config must define at least one remote"
        ));
    }
    if config.shard_policy.kind != "hash_source_identity" {
        return Err(eyre!(
            "distributed placement config shard_policy.kind {:?} is not supported; expected \"hash_source_identity\"",
            config.shard_policy.kind
        ));
    }
    if config.shard_policy.shard_count == 0 {
        return Err(eyre!(
            "distributed placement config shard_policy.shard_count must be greater than zero"
        ));
    }
    profiles::validate_ident(&config.shard_policy.source_identity_column).wrap_err_with(|| {
        format!(
            "invalid distributed placement source identity column {:?}",
            config.shard_policy.source_identity_column
        )
    })?;

    let mut node_ids = HashSet::new();
    let mut seen_shards = HashSet::new();
    for remote in &config.remotes {
        if remote.node_id <= 1 {
            return Err(eyre!(
                "distributed placement remote node_id {} is invalid; remote node ids must be greater than 1",
                remote.node_id
            ));
        }
        if !node_ids.insert(remote.node_id) {
            return Err(eyre!(
                "distributed placement config repeats remote node_id {}",
                remote.node_id
            ));
        }
        if remote.conninfo_secret_name.trim().is_empty() {
            return Err(eyre!(
                "distributed placement remote node_id {} has empty conninfo_secret_name",
                remote.node_id
            ));
        }
        if remote.remote_index_regclass.trim().is_empty() {
            return Err(eyre!(
                "distributed placement remote node_id {} has empty remote_index_regclass",
                remote.node_id
            ));
        }
        validate_qualified_ident(&remote.remote_index_regclass).wrap_err_with(|| {
            format!(
                "invalid distributed placement remote_index_regclass {:?} for node_id {}",
                remote.remote_index_regclass, remote.node_id
            )
        })?;
        if remote.shard_ids.is_empty() {
            return Err(eyre!(
                "distributed placement remote node_id {} owns no shards",
                remote.node_id
            ));
        }
        for shard_id in &remote.shard_ids {
            if *shard_id >= config.shard_policy.shard_count {
                return Err(eyre!(
                    "distributed placement shard_id {} is outside shard_count {}",
                    shard_id,
                    config.shard_policy.shard_count
                ));
            }
            if !seen_shards.insert(*shard_id) {
                return Err(eyre!(
                    "distributed placement shard_id {} is assigned more than once",
                    shard_id
                ));
            }
        }
    }

    let missing: Vec<u32> = (0..config.shard_policy.shard_count)
        .filter(|shard_id| !seen_shards.contains(shard_id))
        .take(16)
        .collect();
    if !missing.is_empty() {
        let suffix = if seen_shards.len() + missing.len() < config.shard_policy.shard_count as usize
        {
            ", ..."
        } else {
            ""
        };
        return Err(eyre!(
            "distributed placement config is missing shard ids: {}{}",
            missing
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(", "),
            suffix
        ));
    }

    Ok(())
}

fn write_distributed_placement_outputs(
    output_dir: &Path,
    input: &LoadedDistributedPlacementConfig,
    prefix: &str,
    profile: &IndexProfile,
    corpus_paths: &[PathBuf],
    dim: usize,
    bits: i32,
    seed: i64,
    storage_format: Option<&str>,
    reloptions: &[(String, String)],
) -> Result<DistributedPlacementOutputPlan> {
    if input.config.shard_policy.source_identity_column != "id" {
        return Err(eyre!(
            "distributed placement source_identity_column {:?} is not supported by this loader slice yet; use \"id\"",
            input.config.shard_policy.source_identity_column
        ));
    }
    if corpus_paths.is_empty() {
        return Err(eyre!(
            "distributed placement output requires at least one corpus input path"
        ));
    }

    std::fs::create_dir_all(output_dir).wrap_err_with(|| {
        format!(
            "creating distributed placement output dir {}",
            output_dir.display()
        )
    })?;

    let mut shard_to_node = HashMap::new();
    for remote in &input.config.remotes {
        for shard_id in &remote.shard_ids {
            shard_to_node.insert(*shard_id, remote.node_id);
        }
    }

    let mut writers = BTreeMap::new();
    for remote in &input.config.remotes {
        let remote_dir = output_dir.join(format!("node-{}", remote.node_id));
        std::fs::create_dir_all(&remote_dir).wrap_err_with(|| {
            format!(
                "creating distributed placement remote output dir {}",
                remote_dir.display()
            )
        })?;
        let corpus_file = remote_dir.join(format!("{prefix}_node_{}_corpus.tsv", remote.node_id));
        let remote_prefix = format!("{prefix}_node_{}", remote.node_id);
        let file = std::fs::File::create(&corpus_file).wrap_err_with(|| {
            format!(
                "creating distributed placement corpus shard {}",
                corpus_file.display()
            )
        })?;
        writers.insert(
            remote.node_id,
            DistributedRemoteWriter {
                node_id: remote.node_id,
                conninfo_secret_name: remote.conninfo_secret_name.clone(),
                conninfo_provider_lookup_key: spire_remote_conninfo_provider_lookup_key(
                    &remote.conninfo_secret_name,
                )?,
                remote_index_regclass: remote.remote_index_regclass.clone(),
                remote_prefix,
                shard_ids: remote.shard_ids.clone(),
                corpus_file,
                writer: std::io::BufWriter::new(file),
                row_count: 0,
                shard_row_counts: BTreeMap::new(),
            },
        );
    }

    let mut total_rows = 0usize;
    for corpus_path in corpus_paths {
        for row in tsv::iter_rows(corpus_path, dim)? {
            let row = row?;
            let source_identity = spire_static_source_identity_from_i64(row.id);
            let shard_id =
                spire_static_shard_id(&source_identity, input.config.shard_policy.shard_count);
            let node_id = shard_to_node.get(&shard_id).copied().ok_or_else(|| {
                eyre!(
                    "distributed placement config has no remote owner for shard_id {}",
                    shard_id
                )
            })?;
            let writer = writers.get_mut(&node_id).ok_or_else(|| {
                eyre!(
                    "distributed placement config maps shard_id {} to missing node_id {}",
                    shard_id,
                    node_id
                )
            })?;
            writeln!(
                writer.writer,
                "{}\t{}",
                row.id,
                serde_json::to_string(&row.values)
                    .expect("serializing parsed f32 vector should not fail")
            )
            .wrap_err_with(|| {
                format!(
                    "writing distributed placement row {} to {}",
                    row.id,
                    writer.corpus_file.display()
                )
            })?;
            writer.row_count += 1;
            *writer.shard_row_counts.entry(shard_id).or_default() += 1;
            total_rows += 1;
        }
    }

    let mut remotes = Vec::with_capacity(writers.len());
    for (_, mut writer) in writers {
        writer.writer.flush().wrap_err_with(|| {
            format!(
                "flushing distributed placement corpus shard {}",
                writer.corpus_file.display()
            )
        })?;
        remotes.push(DistributedRemoteOutputPlan {
            node_id: writer.node_id,
            conninfo_secret_name: writer.conninfo_secret_name.clone(),
            conninfo_provider_lookup_key: writer.conninfo_provider_lookup_key,
            remote_index_regclass: writer.remote_index_regclass.clone(),
            remote_prefix: writer.remote_prefix.clone(),
            shard_ids: writer.shard_ids,
            corpus_file: writer.corpus_file.display().to_string(),
            remote_load_args: distributed_remote_load_args(
                &writer.remote_prefix,
                &writer.corpus_file,
                &writer.remote_index_regclass,
                dim,
                bits,
                seed,
                storage_format,
                reloptions,
            ),
            remote_identity_query_sql: distributed_remote_identity_query_sql(
                &writer.remote_index_regclass,
            ),
            coordinator_register_descriptor_sql_template:
                distributed_coordinator_register_descriptor_sql_template(
                    &plan_coordinator_index_name(&input.config),
                    writer.node_id,
                    &writer.conninfo_secret_name,
                    &writer.remote_index_regclass,
                ),
            row_count: writer.row_count,
            shard_row_counts: writer
                .shard_row_counts
                .into_iter()
                .map(|(shard_id, row_count)| DistributedShardRowCount {
                    shard_id,
                    row_count,
                })
                .collect(),
        });
    }

    let plan = DistributedPlacementOutputPlan {
        version: 1,
        prefix: prefix.to_owned(),
        profile: profile.name.to_owned(),
        dimension: dim,
        bits,
        seed,
        storage_format: storage_format.map(str::to_owned),
        reloptions: reloptions
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect(),
        coordinator_index_name: input.config.coordinator.index_name.clone(),
        source_identity_column: input.config.shard_policy.source_identity_column.clone(),
        shard_policy: input.config.shard_policy.kind.clone(),
        shard_count: input.config.shard_policy.shard_count,
        total_rows,
        remotes,
    };
    let plan_path = output_dir.join("distributed-placement-plan.json");
    let plan_json =
        serde_json::to_string_pretty(&plan).expect("serializing distributed placement plan");
    std::fs::write(&plan_path, format!("{plan_json}\n"))
        .wrap_err_with(|| format!("writing distributed placement plan {}", plan_path.display()))?;

    Ok(plan)
}

fn spire_static_source_identity_from_i64(id: i64) -> [u8; 16] {
    let digest = Sha256::digest(id.to_be_bytes());
    let mut identity = [0_u8; 16];
    identity.copy_from_slice(&digest[..16]);
    identity
}

fn spire_static_shard_id(source_identity: &[u8; 16], shard_count: u32) -> u32 {
    let mut shard_bytes = [0_u8; 8];
    shard_bytes.copy_from_slice(&source_identity[..8]);
    (u64::from_be_bytes(shard_bytes) % u64::from(shard_count)) as u32
}

fn spire_remote_conninfo_provider_lookup_key(conninfo_secret_name: &str) -> Result<String> {
    if conninfo_secret_name.is_empty() {
        return Err(eyre!("conninfo_secret_name must be nonempty"));
    }
    let mut key = String::from("EC_SPIRE_REMOTE_CONNINFO_");
    for byte in conninfo_secret_name.bytes() {
        if byte.is_ascii_alphanumeric() {
            key.push(char::from(byte).to_ascii_uppercase());
        } else {
            key.push('_');
        }
    }
    Ok(key)
}

fn distributed_remote_load_args(
    remote_prefix: &str,
    corpus_file: &Path,
    remote_index_regclass: &str,
    dim: usize,
    bits: i32,
    seed: i64,
    storage_format: Option<&str>,
    reloptions: &[(String, String)],
) -> Vec<String> {
    let mut args = vec![
        "ecaz".to_owned(),
        "corpus".to_owned(),
        "load".to_owned(),
        "--profile".to_owned(),
        "ec_spire".to_owned(),
        "--prefix".to_owned(),
        remote_prefix.to_owned(),
        "--dim".to_owned(),
        dim.to_string(),
        "--bits".to_owned(),
        bits.to_string(),
        "--seed".to_owned(),
        seed.to_string(),
        "--corpus-file".to_owned(),
        corpus_file.display().to_string(),
        "--corpus-only".to_owned(),
        "--index-name".to_owned(),
        remote_index_regclass.to_owned(),
    ];
    if let Some(storage_format) = storage_format {
        args.push("--storage-format".to_owned());
        args.push(storage_format.to_owned());
    }
    for (key, value) in reloptions {
        args.push("--reloption".to_owned());
        args.push(format!("{key}={value}"));
    }
    args
}

fn plan_coordinator_index_name(config: &DistributedPlacementConfig) -> String {
    config.coordinator.index_name.clone()
}

fn distributed_remote_identity_query_sql(remote_index_regclass: &str) -> String {
    format!(
        "SELECT jsonb_build_object(\
            'remote_index_regclass', {remote_index_literal}, \
            'last_served_epoch', a.active_epoch, \
            'min_retained_epoch', a.active_epoch, \
            'extension_version', e.extension_version, \
            'remote_index_identity_hex', e.profile_fingerprint, \
            'endpoint_status', e.status, \
            'tuple_transport_status', e.tuple_transport_status\
        )::text \
         FROM ec_spire_remote_search_endpoint_identity({remote_index_literal}::regclass::oid) e \
         CROSS JOIN ec_spire_index_active_snapshot_diagnostics({remote_index_literal}::regclass::oid) a",
        remote_index_literal = sql_string_literal(remote_index_regclass)
    )
}

fn distributed_coordinator_register_descriptor_sql_template(
    coordinator_index_name: &str,
    node_id: u32,
    conninfo_secret_name: &str,
    remote_index_regclass: &str,
) -> String {
    format!(
        "SELECT ec_spire_register_remote_node_descriptor(\
            {coordinator_index}::regclass::oid, \
            {node_id}, \
            1, \
            {conninfo_secret}, \
            decode('{{remote_index_identity_hex}}', 'hex'), \
            {remote_index}, \
            'active', \
            {{last_served_epoch}}, \
            {{min_retained_epoch}}, \
            '{{extension_version}}', \
            'none'\
        ) AS registered",
        coordinator_index = sql_string_literal(coordinator_index_name),
        conninfo_secret = sql_string_literal(conninfo_secret_name),
        remote_index = sql_string_literal(remote_index_regclass)
    )
}

fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn print_distributed_placement_output_summary(
    output_dir: &Path,
    plan: &DistributedPlacementOutputPlan,
) {
    let mut t = Table::new();
    t.load_preset(UTF8_FULL);
    t.set_header(vec!["field", "value"]);
    t.add_row(vec![
        "mode".into(),
        Cell::new("distributed-placement-output"),
    ]);
    t.add_row(vec!["output_dir".into(), Cell::new(output_dir.display())]);
    t.add_row(vec!["prefix".into(), Cell::new(&plan.prefix)]);
    t.add_row(vec![
        "coordinator_index".into(),
        Cell::new(&plan.coordinator_index_name),
    ]);
    t.add_row(vec!["total_rows".into(), Cell::new(plan.total_rows)]);
    let remotes = plan
        .remotes
        .iter()
        .map(|remote| {
            format!(
                "node {}: {} rows, shards {:?}, env {}",
                remote.node_id,
                remote.row_count,
                remote.shard_ids,
                remote.conninfo_provider_lookup_key
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    t.add_row(vec!["remotes".into(), Cell::new(remotes)]);
    crate::ecaz_println!("{t}");
}

/// Verify a sibling manifest if one was requested or auto-discovered.
///
/// Three paths:
/// - `--manifest-file` passed: the path must exist, or we fail.
/// - No flag, sibling auto-discovered and present: verify it.
/// - No flag, no sibling on disk: log once, continue without verification.
///
/// When problems are found and `allow_mismatch` is false, bail with the
/// full diff. With `allow_mismatch`, log a warning and continue so a
/// reviewer can poke at an inconsistent fixture without rebuilding it.
fn verify_manifest_if_present(
    explicit: Option<&Path>,
    corpus_file: &Path,
    queries_file: &Path,
    prefix: &str,
    dim: usize,
    corpus_stats: &tsv::VectorFileStats,
    query_stats: &tsv::VectorFileStats,
    allow_mismatch: bool,
) -> Result<()> {
    let derived = manifest::derive_manifest_path(corpus_file, queries_file);
    let (path, explicit_request): (PathBuf, bool) = match (explicit, derived) {
        (Some(p), _) => (p.to_path_buf(), true),
        (None, Some(p)) if p.exists() => (p, false),
        (None, Some(p)) => {
            crate::ecaz_eprintln!(
                "[loader] no sibling manifest at {}; continuing without verification",
                p.display()
            );
            return Ok(());
        }
        (None, None) => return Ok(()),
    };
    if explicit_request && !path.exists() {
        return Err(eyre!("manifest file {:?} does not exist", path));
    }
    let raw = std::fs::read_to_string(&path)
        .wrap_err_with(|| format!("reading manifest {}", path.display()))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .wrap_err_with(|| format!("parsing manifest {}", path.display()))?;
    let problems = manifest::verify(
        &parsed,
        prefix,
        corpus_file,
        queries_file,
        dim,
        corpus_stats,
        query_stats,
    );
    if problems.is_empty() {
        crate::ecaz_eprintln!(
            "[loader] verified manifest {} for prefix {prefix}",
            path.display()
        );
        return Ok(());
    }
    let joined = problems
        .iter()
        .map(|p| p.0.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    let msg = format!(
        "manifest verification failed for {}: {joined}",
        path.display()
    );
    if allow_mismatch {
        crate::ecaz_eprintln!("[loader] warning: {msg}");
        Ok(())
    } else {
        Err(eyre!(msg))
    }
}

/// Pair describing a `--reloption key=...` that duplicates a native CLI flag.
struct FlagCollision {
    key: &'static str,
    flag: &'static str,
}

fn unsupported_m_error(profile: &IndexProfile) -> String {
    format!(
        "--m is not supported by profile {:?}; use --reloption for {} tuning instead (known keys: {}). Example: `ecaz corpus load --profile {} --reloption graph_degree=48 --reloption alpha=1.2 ...`",
        profile.name,
        profile.name,
        profile.known_reloptions.join(", "),
        profile.name
    )
}

fn unsupported_ef_construction_error(profile: &IndexProfile) -> String {
    format!(
        "--ef-construction is not supported by profile {:?}; use --reloption for {} tuning instead (known keys: {}). Example: `ecaz corpus load --profile {} --reloption graph_degree=48 --reloption build_list_size=128 ...`",
        profile.name,
        profile.name,
        profile.known_reloptions.join(", "),
        profile.name
    )
}

fn foreign_hnsw_reloption_keys(
    profile: &IndexProfile,
    reloptions: &[(String, String)],
) -> Vec<String> {
    if profile.sweep_axis_is_m() {
        return Vec::new();
    }
    let keys: std::collections::BTreeSet<String> = reloptions
        .iter()
        .filter_map(|(key, _)| {
            HNSW_ONLY_RELOPTIONS
                .iter()
                .any(|hnsw_key| hnsw_key == &key.as_str())
                .then(|| key.clone())
        })
        .collect();
    keys.into_iter().collect()
}

fn unsupported_hnsw_reloption_error(profile: &IndexProfile, keys: &[String]) -> String {
    format!(
        "profile {:?} does not support HNSW-only reloption{} {}; use DiskANN reloptions instead (known keys: {}). Example: `ecaz corpus load --profile {} --reloption graph_degree=48 --reloption build_list_size=128 ...`",
        profile.name,
        if keys.len() == 1 { "" } else { "s" },
        keys.join(", "),
        profile.known_reloptions.join(", "),
        profile.name
    )
}

fn existing_single_index_conflict_error(
    profile: &IndexProfile,
    index: &str,
    reloptions: &[(String, String)],
) -> String {
    let requested = if reloptions.is_empty() {
        "<default>".to_owned()
    } else {
        reloptions::normalize_list(reloptions).join(", ")
    };
    format!(
        "index {:?} already exists for profile {:?}; {} keeps one index name per prefix, so `ecaz corpus load` will not rebuild it in place. Drop it first (for example: `DROP INDEX {index}`) or change --prefix / --storage-format. Requested reloptions: {requested}",
        index,
        profile.name,
        profile.name,
    )
}

/// Reject `--reloption` keys that a native CLI flag already sets. Postgres
/// rejects duplicate reloption keys at `CREATE INDEX`, and even when it
/// doesn't, letting `--reloption` silently override a native flag is worse
/// UX than a clear up-front error pointing at the redundant flag.
fn reloption_flag_collisions(
    profile: &IndexProfile,
    reloptions: &[(String, String)],
    storage_format: Option<&str>,
) -> Vec<FlagCollision> {
    let mut managed: Vec<FlagCollision> = Vec::new();
    if profile.sweep_axis_is_m() {
        managed.push(FlagCollision {
            key: "m",
            flag: "--m",
        });
        managed.push(FlagCollision {
            key: "ef_construction",
            flag: "--ef-construction",
        });
        managed.push(FlagCollision {
            key: "build_source_column",
            flag: "(HNSW built-in)",
        });
    }
    if storage_format.is_some() {
        managed.push(FlagCollision {
            key: "storage_format",
            flag: "--storage-format",
        });
    }
    managed
        .into_iter()
        .filter(|c| reloptions.iter().any(|(k, _)| k == c.key))
        .collect()
}

fn plan_index_jobs(
    profile: &IndexProfile,
    index_prefix: &str,
    m_values: &[i32],
    ef_construction: i32,
    storage_format: Option<&str>,
    extra: &[(String, String)],
) -> Vec<IndexJob> {
    if profile.sweep_axis_is_m() {
        let sweep = dedup_preserve_order(if m_values.is_empty() {
            DEFAULT_HNSW_M_SWEEP.to_vec()
        } else {
            m_values.to_vec()
        });
        sweep
            .into_iter()
            .map(|m| {
                let mut opts: Vec<(String, String)> = vec![
                    ("m".into(), m.to_string()),
                    ("ef_construction".into(), ef_construction.to_string()),
                    (
                        "build_source_column".into(),
                        DEFAULT_HNSW_BUILD_SOURCE_COLUMN.into(),
                    ),
                ];
                if let Some(sf) = storage_format {
                    opts.push(("storage_format".into(), sf.into()));
                }
                opts.extend(extra.iter().cloned());
                IndexJob {
                    name: format!("{index_prefix}_m{m}_idx"),
                    reloptions: opts,
                }
            })
            .collect()
    } else {
        let mut opts: Vec<(String, String)> = extra.to_vec();
        if let Some(sf) = storage_format {
            opts.push(("storage_format".into(), sf.into()));
        }
        vec![IndexJob {
            name: format!("{index_prefix}_idx"),
            reloptions: opts,
        }]
    }
}

fn plan_index_jobs_with_optional_name(
    profile: &IndexProfile,
    index_prefix: &str,
    m_values: &[i32],
    ef_construction: i32,
    storage_format: Option<&str>,
    extra: &[(String, String)],
    index_name: Option<&str>,
) -> Result<Vec<IndexJob>> {
    let mut jobs = plan_index_jobs(
        profile,
        index_prefix,
        m_values,
        ef_construction,
        storage_format,
        extra,
    );
    if let Some(index_name) = index_name {
        if jobs.len() != 1 {
            return Err(eyre!(
                "--index-name is only supported for single-index profiles; profile {:?} planned {} indexes",
                profile.name,
                jobs.len()
            ));
        }
        jobs[0].name = index_name.to_owned();
    }
    Ok(jobs)
}

fn validate_qualified_ident(value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(eyre!("identifier must not be empty"));
    }
    for part in value.split('.') {
        profiles::validate_ident(part)?;
    }
    Ok(())
}

fn dedup_preserve_order(values: Vec<i32>) -> Vec<i32> {
    let mut seen = std::collections::HashSet::new();
    values.into_iter().filter(|v| seen.insert(*v)).collect()
}

async fn ensure_chunked_corpus_table(
    client: &mut Client,
    table: &str,
    input: &LoadedChunkedManifest,
    bits: i32,
    seed: i64,
    profile: &IndexProfile,
) -> Result<usize> {
    ensure_chunked_state_table(client).await?;
    ensure_chunked_target_table(client, table, true, profile).await?;
    load_chunk_set(
        client,
        &input.manifest.prefix,
        table,
        LoadKind::Corpus,
        &input.manifest.corpus,
        input,
        Some((profile, bits, seed)),
    )
    .await
}

async fn ensure_chunked_queries_table(
    client: &mut Client,
    table: &str,
    input: &LoadedChunkedManifest,
) -> Result<usize> {
    ensure_chunked_state_table(client).await?;
    ensure_chunked_target_table(client, table, false, &profiles::EC_HNSW).await?;
    load_chunk_set(
        client,
        &input.manifest.prefix,
        table,
        LoadKind::Queries,
        &input.manifest.queries,
        input,
        None,
    )
    .await
}

async fn ensure_chunked_state_table(client: &Client) -> Result<()> {
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS ecaz_corpus_load_state (
                prefix text NOT NULL,
                chunk_kind text NOT NULL,
                chunk_path text NOT NULL,
                chunk_sha256 text NOT NULL,
                row_count bigint NOT NULL,
                loaded_at timestamptz NOT NULL DEFAULT now(),
                PRIMARY KEY (prefix, chunk_kind, chunk_path)
            )",
        )
        .await
        .wrap_err("creating ecaz_corpus_load_state")?;
    Ok(())
}

async fn ensure_chunked_target_table(
    client: &Client,
    table: &str,
    is_corpus: bool,
    profile: &IndexProfile,
) -> Result<()> {
    if psql::relation_exists(client, table, 'r').await? {
        return Ok(());
    }
    let ddl = if is_corpus {
        format!(
            "CREATE TABLE {table} (
                id        bigint PRIMARY KEY,
                source    real[] NOT NULL,
                embedding {embedding} NOT NULL
            )",
            embedding = profile.embedding_type
        )
    } else {
        format!(
            "CREATE TABLE {table} (
                id     bigint PRIMARY KEY,
                source real[] NOT NULL
            )"
        )
    };
    client
        .batch_execute(&ddl)
        .await
        .wrap_err_with(|| format!("creating table {table}"))?;
    Ok(())
}

async fn load_chunk_set(
    client: &mut Client,
    prefix: &str,
    table: &str,
    kind: LoadKind,
    section: &manifest::ChunkedFileManifest,
    input: &LoadedChunkedManifest,
    encode: Option<(&IndexProfile, i32, i64)>,
) -> Result<usize> {
    let existing_rows = if psql::relation_exists(client, table, 'r').await? {
        psql::row_count(client, table).await? as usize
    } else {
        0
    };
    let state_rows = fetch_chunk_state_rows(client, prefix, kind).await?;
    validate_existing_chunk_state(table, kind, section, existing_rows, &state_rows)?;

    let state_map: HashMap<String, ChunkStateRow> = state_rows
        .into_iter()
        .map(|row| (row.chunk_path.clone(), row))
        .collect();

    for chunk in &section.chunks {
        let chunk_path = input.base_dir.join(&chunk.path);
        verify_chunk_file(&chunk_path, chunk, input.manifest.dimension)?;
        if let Some(existing) = state_map.get(&chunk.path) {
            if existing.chunk_sha256 != chunk.sha256 || existing.row_count != chunk.rows as i64 {
                return Err(eyre!(
                    "{table}: state row for {} does not match manifest",
                    chunk.path
                ));
            }
            crate::ecaz_eprintln!(
                "[loader] skipping {} chunk {} (already loaded)",
                kind.as_str(),
                chunk.path
            );
            continue;
        }
        load_one_chunk(
            client,
            prefix,
            table,
            kind,
            chunk,
            &chunk_path,
            input.manifest.dimension,
            encode,
        )
        .await?;
    }

    let final_rows = psql::row_count(client, table).await? as usize;
    if final_rows != section.rows {
        return Err(eyre!(
            "{table}: loaded {final_rows} rows but manifest expects {}",
            section.rows
        ));
    }
    Ok(final_rows)
}

async fn fetch_chunk_state_rows(
    client: &Client,
    prefix: &str,
    kind: LoadKind,
) -> Result<Vec<ChunkStateRow>> {
    let rows = client
        .query(
            "SELECT chunk_path, chunk_sha256, row_count
             FROM ecaz_corpus_load_state
             WHERE prefix = $1 AND chunk_kind = $2
             ORDER BY chunk_path",
            &[&prefix, &kind.as_str()],
        )
        .await
        .wrap_err("reading ecaz_corpus_load_state")?;
    Ok(rows
        .into_iter()
        .map(|row| ChunkStateRow {
            chunk_path: row.get(0),
            chunk_sha256: row.get(1),
            row_count: row.get(2),
        })
        .collect())
}

fn validate_existing_chunk_state(
    table: &str,
    kind: LoadKind,
    section: &manifest::ChunkedFileManifest,
    existing_rows: usize,
    state_rows: &[ChunkStateRow],
) -> Result<()> {
    if existing_rows == 0 && state_rows.is_empty() {
        return Ok(());
    }
    if existing_rows == 0 && !state_rows.is_empty() {
        return Err(eyre!(
            "{table}: found {} {} state row(s) but table is empty; cleanup required",
            state_rows.len(),
            kind.as_str()
        ));
    }
    if existing_rows > 0 && state_rows.is_empty() {
        return Err(eyre!(
            "{table}: table has {existing_rows} rows but no {} state rows; cleanup required",
            kind.as_str()
        ));
    }
    let expected: HashMap<&str, (&str, i64)> = section
        .chunks
        .iter()
        .map(|chunk| {
            (
                chunk.path.as_str(),
                (chunk.sha256.as_str(), chunk.rows as i64),
            )
        })
        .collect();
    let mut state_sum = 0usize;
    for row in state_rows {
        let Some((sha, rows)) = expected.get(row.chunk_path.as_str()) else {
            return Err(eyre!(
                "{table}: unexpected {} state row for {}",
                kind.as_str(),
                row.chunk_path
            ));
        };
        if row.chunk_sha256 != *sha || row.row_count != *rows {
            return Err(eyre!(
                "{table}: {} state row for {} does not match manifest",
                kind.as_str(),
                row.chunk_path
            ));
        }
        state_sum += row.row_count as usize;
    }
    if existing_rows != state_sum {
        return Err(eyre!(
            "{table}: table has {existing_rows} rows but {} state rows sum to {state_sum}",
            kind.as_str()
        ));
    }
    Ok(())
}

fn verify_chunk_file(path: &Path, chunk: &manifest::ChunkManifest, dim: usize) -> Result<()> {
    let stats = tsv::inspect(path, dim)?;
    let byte_length = std::fs::metadata(path)
        .wrap_err_with(|| format!("stat {}", path.display()))?
        .len();
    if stats.rows != chunk.rows {
        return Err(eyre!(
            "{}: manifest rows={} but file has {}",
            path.display(),
            chunk.rows,
            stats.rows
        ));
    }
    if stats.sha256_hex != chunk.sha256 {
        return Err(eyre!(
            "{}: manifest sha256={} but file has {}",
            path.display(),
            chunk.sha256,
            stats.sha256_hex
        ));
    }
    if byte_length != chunk.byte_length {
        return Err(eyre!(
            "{}: manifest byte_length={} but file has {}",
            path.display(),
            chunk.byte_length,
            byte_length
        ));
    }
    Ok(())
}

async fn load_one_chunk(
    client: &mut Client,
    prefix: &str,
    table: &str,
    kind: LoadKind,
    chunk: &manifest::ChunkManifest,
    chunk_path: &Path,
    dim: usize,
    encode: Option<(&IndexProfile, i32, i64)>,
) -> Result<()> {
    let tx = client.transaction().await?;
    tx.batch_execute(
        "CREATE TEMP TABLE ecaz_chunk_stage (
            id bigint NOT NULL,
            source real[] NOT NULL
        ) ON COMMIT DROP",
    )
    .await?;
    copy_rows_to_stage(&tx, chunk_path, dim, chunk.rows, kind.as_str()).await?;
    match encode {
        Some((profile, bits, seed)) => {
            tx.batch_execute(&format!(
                "INSERT INTO {table} (id, source, embedding)
                 SELECT id, source, {fn_name}(source, {bits}, {seed})
                 FROM ecaz_chunk_stage
                 ORDER BY id",
                fn_name = profile.encoder_function
            ))
            .await
            .wrap_err_with(|| format!("inserting corpus chunk {}", chunk.path))?;
        }
        None => {
            tx.batch_execute(&format!(
                "INSERT INTO {table} (id, source)
                 SELECT id, source
                 FROM ecaz_chunk_stage
                 ORDER BY id"
            ))
            .await
            .wrap_err_with(|| format!("inserting query chunk {}", chunk.path))?;
        }
    }
    tx.execute(
        "INSERT INTO ecaz_corpus_load_state
         (prefix, chunk_kind, chunk_path, chunk_sha256, row_count)
         VALUES ($1, $2, $3, $4, $5)",
        &[
            &prefix,
            &kind.as_str(),
            &chunk.path,
            &chunk.sha256,
            &(chunk.rows as i64),
        ],
    )
    .await
    .wrap_err_with(|| format!("recording chunk state for {}", chunk.path))?;
    tx.commit().await?;
    crate::ecaz_eprintln!("[loader] loaded {} chunk {}", kind.as_str(), chunk.path);
    Ok(())
}

async fn ensure_corpus_table(
    client: &mut Client,
    table: &str,
    path: &Path,
    dim: usize,
    bits: i32,
    seed: i64,
    profile: &IndexProfile,
    expected_rows: usize,
) -> Result<usize> {
    if psql::relation_exists(client, table, 'r').await? {
        let existing = psql::row_count(client, table).await? as usize;
        if existing > 0 {
            crate::ecaz_eprintln!("[loader] {table} already has {existing} rows; skipping reload");
            return Ok(existing);
        }
        crate::ecaz_eprintln!("[loader] {table} exists but is empty; dropping and reloading");
        client
            .batch_execute(&format!("DROP TABLE IF EXISTS {table} CASCADE"))
            .await?;
    }
    client
        .batch_execute(&format!(
            "CREATE TABLE {table} (
                id        bigint PRIMARY KEY,
                source    real[] NOT NULL,
                embedding {embedding} NOT NULL
            )",
            embedding = profile.embedding_type
        ))
        .await
        .wrap_err_with(|| format!("creating table {table}"))?;

    let copy_started = Instant::now();
    let tx = client.transaction().await?;
    tx.batch_execute(
        "CREATE TEMP TABLE ecaz_corpus_stage (
            id bigint NOT NULL,
            source real[] NOT NULL
        ) ON COMMIT DROP",
    )
    .await
    .wrap_err("creating ecaz_corpus_stage")?;
    copy_corpus_rows_to_stage(&tx, path, dim, expected_rows).await?;
    crate::ecaz_eprintln!(
        "[loader] copied corpus table {table} in {:.2?}",
        copy_started.elapsed()
    );

    crate::ecaz_eprintln!(
        "[loader] encoding {embedding_type} embeddings via {fn_name}(source, {bits}, {seed}) ...",
        embedding_type = profile.embedding_type,
        fn_name = profile.encoder_function
    );
    let encode_started = Instant::now();
    tx.batch_execute(&format!(
        "INSERT INTO {table} (id, source, embedding)
             SELECT id, source, {fn_name}(source, {bits}, {seed})
             FROM ecaz_corpus_stage
             ORDER BY id",
        fn_name = profile.encoder_function
    ))
    .await
    .wrap_err_with(|| format!("encoding embeddings for {table}"))?;
    tx.commit()
        .await
        .wrap_err_with(|| format!("committing corpus load for {table}"))?;
    crate::ecaz_eprintln!(
        "[loader] encoded corpus table {table} in {:.2?}",
        encode_started.elapsed()
    );
    psql::row_count(client, table).await.map(|n| n as usize)
}

async fn copy_corpus_rows_to_stage(
    tx: &Transaction<'_>,
    path: &Path,
    dim: usize,
    expected_rows: usize,
) -> Result<()> {
    let sink = tx
        .copy_in::<_, bytes::Bytes>(
            "COPY ecaz_corpus_stage (id, source) FROM STDIN WITH (FORMAT text, DELIMITER E'\\t')",
        )
        .await
        .wrap_err("opening COPY stream for ecaz_corpus_stage")?;
    futures::pin_mut!(sink);

    let bar = ProgressBar::new(expected_rows as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[loader] {msg} {wide_bar} {human_pos}/{human_len} ({per_sec}, eta {eta})",
        )
        .unwrap(),
    );
    bar.set_message("loading corpus into ecaz_corpus_stage");
    bar.enable_steady_tick(Duration::from_millis(250));

    let mut buf = BytesMut::with_capacity(COPY_CHUNK_BYTES + 4096);
    let mut sent = 0u64;
    for row in tsv::iter_rows(path, dim)? {
        let row = row?;
        use std::io::Write as _;
        let mut w = (&mut buf).writer();
        write!(w, "{}\t", row.id).expect("bytesmut writer is infallible");
        // Reuse the shared array-literal formatter so the COPY payload and
        // any other place we render vectors agree on float repr.
        let lit = tsv::format_real_array_literal(&row.values);
        buf.put_slice(lit.as_bytes());
        buf.put_u8(b'\n');
        sent += 1;
        if buf.len() >= COPY_CHUNK_BYTES {
            sink.send(buf.split().freeze())
                .await
                .wrap_err("COPY send failed for ecaz_corpus_stage")?;
            bar.set_position(sent);
        }
    }
    if !buf.is_empty() {
        sink.send(buf.split().freeze())
            .await
            .wrap_err("COPY send failed for ecaz_corpus_stage")?;
    }
    let finished = sink
        .finish()
        .await
        .wrap_err("COPY finish failed for ecaz_corpus_stage")?;
    bar.finish_with_message(format!(
        "staged {finished} corpus rows into ecaz_corpus_stage"
    ));
    Ok(())
}

async fn ensure_queries_table(
    client: &Client,
    table: &str,
    path: &Path,
    dim: usize,
    expected_rows: usize,
) -> Result<usize> {
    if psql::relation_exists(client, table, 'r').await? {
        let existing = psql::row_count(client, table).await? as usize;
        if existing > 0 {
            crate::ecaz_eprintln!("[loader] {table} already has {existing} rows; skipping reload");
            return Ok(existing);
        }
        crate::ecaz_eprintln!("[loader] {table} exists but is empty; dropping and reloading");
        client
            .batch_execute(&format!("DROP TABLE IF EXISTS {table} CASCADE"))
            .await?;
    }
    client
        .batch_execute(&format!(
            "CREATE TABLE {table} (
                id     bigint PRIMARY KEY,
                source real[] NOT NULL
            )"
        ))
        .await
        .wrap_err_with(|| format!("creating table {table}"))?;
    let copy_started = Instant::now();
    copy_rows_from_tsv(client, table, path, dim, expected_rows, "queries").await?;
    crate::ecaz_eprintln!(
        "[loader] copied queries table {table} in {:.2?}",
        copy_started.elapsed()
    );
    psql::row_count(client, table).await.map(|n| n as usize)
}

async fn copy_rows_from_tsv(
    client: &Client,
    table: &str,
    path: &Path,
    dim: usize,
    expected_rows: usize,
    label: &str,
) -> Result<()> {
    let sink = client
        .copy_in::<_, bytes::Bytes>(&format!(
            "COPY {table} (id, source) FROM STDIN WITH (FORMAT text, DELIMITER E'\\t')"
        ))
        .await
        .wrap_err_with(|| format!("opening COPY stream for {table}"))?;
    futures::pin_mut!(sink);

    let bar = ProgressBar::new(expected_rows as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[loader] {msg} {wide_bar} {human_pos}/{human_len} ({per_sec}, eta {eta})",
        )
        .unwrap(),
    );
    bar.set_message(format!("loading {label} into {table}"));
    bar.enable_steady_tick(Duration::from_millis(250));

    let mut buf = BytesMut::with_capacity(COPY_CHUNK_BYTES + 4096);
    let mut sent = 0u64;
    for row in tsv::iter_rows(path, dim)? {
        let row = row?;
        use std::io::Write as _;
        let mut w = (&mut buf).writer();
        write!(w, "{}\t", row.id).expect("bytesmut writer is infallible");
        // Reuse the shared array-literal formatter so the COPY payload and
        // any other place we render vectors agree on float repr.
        let lit = tsv::format_real_array_literal(&row.values);
        buf.put_slice(lit.as_bytes());
        buf.put_u8(b'\n');
        sent += 1;
        if buf.len() >= COPY_CHUNK_BYTES {
            sink.send(buf.split().freeze())
                .await
                .wrap_err_with(|| format!("COPY send failed for {table}"))?;
            bar.set_position(sent);
        }
    }
    if !buf.is_empty() {
        sink.send(buf.split().freeze())
            .await
            .wrap_err_with(|| format!("COPY send failed for {table}"))?;
    }
    let finished = sink
        .finish()
        .await
        .wrap_err_with(|| format!("COPY finish failed for {table}"))?;
    bar.finish_with_message(format!("loaded {finished} {label} rows into {table}"));
    Ok(())
}

async fn copy_rows_to_stage(
    tx: &Transaction<'_>,
    path: &Path,
    dim: usize,
    expected_rows: usize,
    label: &str,
) -> Result<()> {
    let sink = tx
        .copy_in::<_, bytes::Bytes>(
            "COPY ecaz_chunk_stage (id, source) FROM STDIN WITH (FORMAT text, DELIMITER E'\\t')",
        )
        .await
        .wrap_err("opening COPY stream for ecaz_chunk_stage")?;
    futures::pin_mut!(sink);

    let bar = ProgressBar::new(expected_rows as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[loader] {msg} {wide_bar} {human_pos}/{human_len} ({per_sec}, eta {eta})",
        )
        .unwrap(),
    );
    bar.set_message(format!("staging {label} chunk {}", path.display()));
    bar.enable_steady_tick(Duration::from_millis(250));

    let mut buf = BytesMut::with_capacity(COPY_CHUNK_BYTES + 4096);
    let mut sent = 0u64;
    for row in tsv::iter_rows(path, dim)? {
        let row = row?;
        use std::io::Write as _;
        let mut w = (&mut buf).writer();
        write!(w, "{}\t", row.id).expect("bytesmut writer is infallible");
        let lit = tsv::format_real_array_literal(&row.values);
        buf.put_slice(lit.as_bytes());
        buf.put_u8(b'\n');
        sent += 1;
        if buf.len() >= COPY_CHUNK_BYTES {
            sink.send(buf.split().freeze())
                .await
                .wrap_err("COPY send failed for ecaz_chunk_stage")?;
            bar.set_position(sent);
        }
    }
    if !buf.is_empty() {
        sink.send(buf.split().freeze())
            .await
            .wrap_err("COPY send failed for ecaz_chunk_stage")?;
    }
    let finished = sink
        .finish()
        .await
        .wrap_err("COPY finish failed for ecaz_chunk_stage")?;
    bar.finish_with_message(format!("staged {finished} rows from {}", path.display()));
    Ok(())
}

async fn ensure_index(
    client: &Client,
    corpus_table: &str,
    job: &IndexJob,
    profile: &IndexProfile,
) -> Result<()> {
    let summary = if job.reloptions.is_empty() {
        "<none>".to_owned()
    } else {
        reloptions::normalize_list(&job.reloptions).join(", ")
    };
    if psql::index_exists_with_reloptions(client, &job.name, &job.reloptions).await? {
        crate::ecaz_eprintln!(
            "[loader] {index} already exists with reloptions=[{summary}]; skipping rebuild",
            index = job.name
        );
        return Ok(());
    }
    if !profile.sweep_axis_is_m() && psql::relation_exists(client, &job.name, 'i').await? {
        return Err(eyre!(existing_single_index_conflict_error(
            profile,
            &job.name,
            &job.reloptions
        )));
    }
    crate::ecaz_eprintln!(
        "[loader] building {index} using {am} (reloptions=[{summary}]) ...",
        index = job.name,
        am = profile.access_method,
    );
    let sql = psql::build_create_index_sql(corpus_table, &job.name, profile, &job.reloptions);
    let build_started = Instant::now();
    client
        .batch_execute(&sql)
        .await
        .wrap_err_with(|| format!("building index {}", job.name))?;
    crate::ecaz_eprintln!(
        "[loader] built {index} in {:.2?}",
        build_started.elapsed(),
        index = job.name
    );
    Ok(())
}

fn print_summary(
    profile: &IndexProfile,
    corpus_table: &str,
    corpus_rows: usize,
    queries_table: Option<&str>,
    queries_rows: Option<usize>,
    jobs: &[IndexJob],
) {
    let mut t = Table::new();
    t.load_preset(UTF8_FULL);
    t.set_header(vec!["field", "value"]);
    t.add_row(vec!["profile".into(), Cell::new(profile.name)]);
    t.add_row(vec![
        "corpus".into(),
        Cell::new(format!("{corpus_table} ({corpus_rows} rows)")),
    ]);
    let queries = match (queries_table, queries_rows) {
        (Some(table), Some(rows)) => format!("{table} ({rows} rows)"),
        _ => "skipped (--corpus-only)".to_owned(),
    };
    t.add_row(vec!["queries".into(), Cell::new(queries)]);
    let indexes = jobs
        .iter()
        .map(|j| {
            let opts = if j.reloptions.is_empty() {
                "<default>".to_owned()
            } else {
                reloptions::normalize_list(&j.reloptions).join(", ")
            };
            format!("{} [{}]", j.name, opts)
        })
        .collect::<Vec<_>>()
        .join("\n");
    t.add_row(vec!["indexes".into(), Cell::new(indexes)]);
    crate::ecaz_println!("{t}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::{EC_DISKANN, EC_HNSW, EC_IVF, EC_SPIRE};
    use crate::tsv::VectorFileStats;
    use tempfile::TempDir;

    fn opt(k: &str, v: &str) -> (String, String) {
        (k.to_owned(), v.to_owned())
    }

    fn stats(rows: usize, sha: &str) -> VectorFileStats {
        VectorFileStats {
            rows,
            sha256_hex: sha.to_owned(),
            first_id: Some(0),
            last_id: Some(rows.saturating_sub(1) as i64),
        }
    }

    #[test]
    fn hnsw_plan_defaults_to_8_16_sweep_with_ef_and_build_source() {
        let jobs = plan_index_jobs(&EC_HNSW, "dbpedia_10k", &[], 128, None, &[]);
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].name, "dbpedia_10k_m8_idx");
        assert_eq!(jobs[1].name, "dbpedia_10k_m16_idx");
        assert!(jobs[0].reloptions.contains(&opt("m", "8")));
        assert!(jobs[0].reloptions.contains(&opt("ef_construction", "128")));
        assert!(jobs[0]
            .reloptions
            .contains(&opt("build_source_column", "source")));
    }

    #[test]
    fn hnsw_plan_honors_explicit_m_and_dedup() {
        let jobs = plan_index_jobs(
            &EC_HNSW,
            "foo_pq_fastscan",
            &[8, 16, 8],
            96,
            Some("pq_fastscan"),
            &[],
        );
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].name, "foo_pq_fastscan_m8_idx");
        assert!(jobs[0].reloptions.contains(&opt("ef_construction", "96")));
        assert!(jobs[0]
            .reloptions
            .contains(&opt("storage_format", "pq_fastscan")));
    }

    #[test]
    fn hnsw_plan_passes_extras_through_and_orders_after_built_ins() {
        let extras = vec![opt("storage_format", "turboquant"), opt("custom", "x")];
        let jobs = plan_index_jobs(&EC_HNSW, "p", &[8], 128, None, &extras);
        // built-ins come first so duplicates from --reloption would override
        let keys: Vec<&str> = jobs[0].reloptions.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(
            keys,
            vec![
                "m",
                "ef_construction",
                "build_source_column",
                "storage_format",
                "custom"
            ]
        );
    }

    #[test]
    fn dedup_preserve_order_keeps_first_occurrence() {
        assert_eq!(
            dedup_preserve_order(vec![16, 8, 16, 32, 8]),
            vec![16, 8, 32]
        );
        assert_eq!(dedup_preserve_order(vec![]), Vec::<i32>::new());
        assert_eq!(dedup_preserve_order(vec![8]), vec![8]);
    }

    #[test]
    fn diskann_plan_is_single_index_with_no_hnsw_defaults() {
        let extras = vec![opt("graph_degree", "48")];
        let jobs = plan_index_jobs(&EC_DISKANN, "foo", &[], 128, None, &extras);
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "foo_idx");
        assert!(jobs[0].reloptions.contains(&opt("graph_degree", "48")));
        assert!(!jobs[0].reloptions.iter().any(|(k, _)| k == "m"));
        assert!(!jobs[0]
            .reloptions
            .iter()
            .any(|(k, _)| k == "build_source_column"));
    }

    #[test]
    fn diskann_plan_appends_storage_format_to_extras() {
        let jobs = plan_index_jobs(
            &EC_DISKANN,
            "foo_pq_fastscan",
            &[],
            128,
            Some("pq_fastscan"),
            &[],
        );
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "foo_pq_fastscan_idx");
        assert!(jobs[0]
            .reloptions
            .contains(&opt("storage_format", "pq_fastscan")));
    }

    #[test]
    fn ivf_plan_is_single_index_with_ivf_reloptions_only() {
        let extras = vec![opt("nlists", "128"), opt("nprobe", "8")];
        let jobs = plan_index_jobs(
            &EC_IVF,
            "foo_turboquant",
            &[],
            128,
            Some("turboquant"),
            &extras,
        );
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "foo_turboquant_idx");
        assert!(jobs[0].reloptions.contains(&opt("nlists", "128")));
        assert!(jobs[0].reloptions.contains(&opt("nprobe", "8")));
        assert!(jobs[0]
            .reloptions
            .contains(&opt("storage_format", "turboquant")));
        assert!(!jobs[0].reloptions.iter().any(|(k, _)| k == "m"));
        assert!(!jobs[0]
            .reloptions
            .iter()
            .any(|(k, _)| k == "build_source_column"));
    }

    #[test]
    fn single_index_plan_honors_explicit_index_name() {
        let jobs = plan_index_jobs_with_optional_name(
            &EC_SPIRE,
            "aws_spire_node_2",
            &[],
            128,
            Some("rabitq"),
            &[opt("nlists", "8")],
            Some("public.aws_spire_remote_a_idx"),
        )
        .unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "public.aws_spire_remote_a_idx");
        assert!(jobs[0]
            .reloptions
            .contains(&opt("storage_format", "rabitq")));
        assert!(jobs[0].reloptions.contains(&opt("nlists", "8")));
    }

    #[test]
    fn index_name_override_rejects_sweep_profiles() {
        let err = plan_index_jobs_with_optional_name(
            &EC_HNSW,
            "pfx",
            &[],
            128,
            None,
            &[],
            Some("custom_idx"),
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("--index-name is only supported"), "err: {err}");
    }

    #[test]
    fn qualified_identifier_validation_rejects_injection_shape() {
        validate_qualified_ident("public.valid_idx").unwrap();
        let err = validate_qualified_ident("public.bad;drop")
            .unwrap_err()
            .to_string();
        assert!(err.contains("must match"), "err: {err}");
    }

    // --- reloption / CLI flag collisions ---

    #[test]
    fn collision_hnsw_m_reloption_flagged() {
        let opts = vec![opt("m", "32")];
        let c = reloption_flag_collisions(&EC_HNSW, &opts, None);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].key, "m");
    }

    #[test]
    fn collision_hnsw_ef_construction_and_build_source_flagged() {
        let opts = vec![
            opt("ef_construction", "96"),
            opt("build_source_column", "x"),
        ];
        let c = reloption_flag_collisions(&EC_HNSW, &opts, None);
        let keys: Vec<&str> = c.iter().map(|c| c.key).collect();
        assert!(keys.contains(&"ef_construction"));
        assert!(keys.contains(&"build_source_column"));
    }

    #[test]
    fn collision_storage_format_flagged_only_when_cli_flag_set() {
        let opts = vec![opt("storage_format", "pq_fastscan")];
        assert!(reloption_flag_collisions(&EC_DISKANN, &opts, None).is_empty());
        let c = reloption_flag_collisions(&EC_DISKANN, &opts, Some("turboquant"));
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].key, "storage_format");
    }

    #[test]
    fn collision_diskann_m_reloption_not_flagged() {
        // DiskANN has no --m flag; an `m=` reloption here is just pass-through
        // (and independently flagged as unknown by profile.unknown_reloption_keys).
        let opts = vec![opt("m", "32")];
        assert!(reloption_flag_collisions(&EC_DISKANN, &opts, None).is_empty());
    }

    #[test]
    fn collision_empty_when_no_overlap() {
        let opts = vec![opt("graph_degree", "48"), opt("alpha", "1.2")];
        assert!(reloption_flag_collisions(&EC_DISKANN, &opts, None).is_empty());
        assert!(reloption_flag_collisions(&EC_HNSW, &[], None).is_empty());
    }

    #[test]
    fn unsupported_m_error_points_diskann_operators_at_reloptions() {
        let err = unsupported_m_error(&EC_DISKANN);
        assert!(err.contains("--m is not supported by profile \"ec_diskann\""));
        assert!(err.contains("known keys: graph_degree, build_list_size, list_size"));
        assert!(err.contains("--profile ec_diskann --reloption graph_degree=48"));
    }

    #[test]
    fn unsupported_ef_construction_error_points_diskann_operators_at_reloptions() {
        let err = unsupported_ef_construction_error(&EC_DISKANN);
        assert!(err.contains("--ef-construction is not supported by profile \"ec_diskann\""));
        assert!(err.contains("known keys: graph_degree, build_list_size, list_size"));
        assert!(err.contains(
            "--profile ec_diskann --reloption graph_degree=48 --reloption build_list_size=128"
        ));
    }

    #[test]
    fn foreign_hnsw_reloption_keys_find_hnsw_only_keys_once_for_diskann() {
        let keys = foreign_hnsw_reloption_keys(
            &EC_DISKANN,
            &[
                opt("m", "16"),
                opt("custom", "x"),
                opt("ef_construction", "128"),
                opt("m", "32"),
            ],
        );
        assert_eq!(keys, vec!["ef_construction".to_string(), "m".to_string()]);
    }

    #[test]
    fn foreign_hnsw_reloption_keys_are_ignored_for_hnsw_profile() {
        let keys = foreign_hnsw_reloption_keys(&EC_HNSW, &[opt("m", "16")]);
        assert!(keys.is_empty());
    }

    #[test]
    fn unsupported_hnsw_reloption_error_points_diskann_operator_at_known_keys() {
        let err = unsupported_hnsw_reloption_error(
            &EC_DISKANN,
            &["m".to_string(), "ef_construction".to_string()],
        );
        assert!(err.contains("profile \"ec_diskann\" does not support HNSW-only reloptions"));
        assert!(err.contains("m, ef_construction"));
        assert!(err.contains("known keys: graph_degree, build_list_size, list_size"));
        assert!(err.contains(
            "--profile ec_diskann --reloption graph_degree=48 --reloption build_list_size=128"
        ));
    }

    #[test]
    fn existing_single_index_conflict_error_points_diskann_operator_at_drop_index() {
        let err = existing_single_index_conflict_error(
            &EC_DISKANN,
            "dbpedia_10k_idx",
            &[opt("graph_degree", "48"), opt("build_list_size", "128")],
        );
        assert!(err.contains("index \"dbpedia_10k_idx\" already exists"));
        assert!(err.contains("profile \"ec_diskann\""));
        assert!(err.contains("DROP INDEX dbpedia_10k_idx"));
        assert!(err.contains("graph_degree=48"));
        assert!(err.contains("build_list_size=128"));
    }

    // --- distributed SPIRE placement config ---

    fn distributed_placement_config_json() -> String {
        serde_json::json!({
            "version": 1,
            "coordinator": {
                "index_name": "aws_spire_idx"
            },
            "remotes": [
                {
                    "node_id": 2,
                    "conninfo_secret_name": "spire/remote/a",
                    "remote_index_regclass": "public.aws_spire_remote_a_idx",
                    "shard_ids": [0, 2]
                },
                {
                    "node_id": 3,
                    "conninfo_secret_name": "spire/remote/b",
                    "remote_index_regclass": "public.aws_spire_remote_b_idx",
                    "shard_ids": [1, 3]
                }
            ],
            "shard_policy": {
                "kind": "hash_source_identity",
                "shard_count": 4,
                "source_identity_column": "id"
            }
        })
        .to_string()
    }

    #[test]
    fn distributed_placement_config_accepts_complete_static_shard_map() {
        let td = TempDir::new().unwrap();
        let path = write(&td, "placement.json", &distributed_placement_config_json());
        let loaded = load_distributed_placement_config_if_requested(Some(&path), &EC_SPIRE)
            .unwrap()
            .unwrap();
        assert_eq!(loaded.path, path);
        assert_eq!(loaded.config.remotes.len(), 2);
        assert_eq!(loaded.config.shard_policy.shard_count, 4);
    }

    #[test]
    fn distributed_placement_config_rejects_non_spire_profile() {
        let td = TempDir::new().unwrap();
        let path = write(&td, "placement.json", &distributed_placement_config_json());
        let err = load_distributed_placement_config_if_requested(Some(&path), &EC_DISKANN)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("only supported with --profile ec_spire"),
            "err: {err}"
        );
    }

    #[test]
    fn distributed_placement_config_rejects_local_node_id() {
        let config: DistributedPlacementConfig = serde_json::from_value(serde_json::json!({
            "version": 1,
            "coordinator": {"index_name": "aws_spire_idx"},
            "remotes": [{
                "node_id": 1,
                "conninfo_secret_name": "spire/remote/a",
                "remote_index_regclass": "public.aws_spire_remote_a_idx",
                "shard_ids": [0]
            }],
            "shard_policy": {
                "kind": "hash_source_identity",
                "shard_count": 1,
                "source_identity_column": "id"
            }
        }))
        .unwrap();
        let err = validate_distributed_placement_config(&config)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("remote node ids must be greater than 1"),
            "err: {err}"
        );
    }

    #[test]
    fn distributed_placement_config_rejects_duplicate_shard_assignment() {
        let config: DistributedPlacementConfig = serde_json::from_value(serde_json::json!({
            "version": 1,
            "coordinator": {"index_name": "aws_spire_idx"},
            "remotes": [
                {
                    "node_id": 2,
                    "conninfo_secret_name": "spire/remote/a",
                    "remote_index_regclass": "public.aws_spire_remote_a_idx",
                    "shard_ids": [0]
                },
                {
                    "node_id": 3,
                    "conninfo_secret_name": "spire/remote/b",
                    "remote_index_regclass": "public.aws_spire_remote_b_idx",
                    "shard_ids": [0]
                }
            ],
            "shard_policy": {
                "kind": "hash_source_identity",
                "shard_count": 1,
                "source_identity_column": "id"
            }
        }))
        .unwrap();
        let err = validate_distributed_placement_config(&config)
            .unwrap_err()
            .to_string();
        assert!(err.contains("assigned more than once"), "err: {err}");
    }

    #[test]
    fn distributed_placement_config_rejects_missing_shards() {
        let config: DistributedPlacementConfig = serde_json::from_value(serde_json::json!({
            "version": 1,
            "coordinator": {"index_name": "aws_spire_idx"},
            "remotes": [{
                "node_id": 2,
                "conninfo_secret_name": "spire/remote/a",
                "remote_index_regclass": "public.aws_spire_remote_a_idx",
                "shard_ids": [0, 2]
            }],
            "shard_policy": {
                "kind": "hash_source_identity",
                "shard_count": 4,
                "source_identity_column": "id"
            }
        }))
        .unwrap();
        let err = validate_distributed_placement_config(&config)
            .unwrap_err()
            .to_string();
        assert!(err.contains("missing shard ids: 1, 3"), "err: {err}");
    }

    #[test]
    fn distributed_placement_output_splits_corpus_by_static_shard() {
        let td = TempDir::new().unwrap();
        let config_path = write(&td, "placement.json", &distributed_placement_config_json());
        let corpus_path = write(
            &td,
            "corpus.tsv",
            "1\t[0.1, 0.2]\n2\t[0.3, 0.4]\n3\t[0.5, 0.6]\n4\t[0.7, 0.8]\n5\t[0.9, 1.0]\n",
        );
        let output_dir = td.path().join("distributed");
        let loaded = load_distributed_placement_config_if_requested(Some(&config_path), &EC_SPIRE)
            .unwrap()
            .unwrap();

        let plan = write_distributed_placement_outputs(
            &output_dir,
            &loaded,
            "aws_spire",
            &EC_SPIRE,
            &[corpus_path],
            2,
            6,
            99,
            Some("rabitq"),
            &[opt("nlists", "8")],
        )
        .unwrap();

        assert_eq!(plan.total_rows, 5);
        assert_eq!(plan.dimension, 2);
        assert_eq!(plan.bits, 6);
        assert_eq!(plan.seed, 99);
        assert_eq!(plan.storage_format.as_deref(), Some("rabitq"));
        assert_eq!(plan.reloptions, vec!["nlists=8".to_owned()]);
        assert_eq!(plan.remotes.len(), 2);
        assert_eq!(
            plan.remotes
                .iter()
                .map(|remote| remote.row_count)
                .sum::<usize>(),
            5
        );
        assert!(output_dir.join("distributed-placement-plan.json").exists());
        for remote in &plan.remotes {
            assert!(remote
                .conninfo_provider_lookup_key
                .starts_with("EC_SPIRE_REMOTE_CONNINFO_"));
            let body = std::fs::read_to_string(&remote.corpus_file).unwrap();
            assert_eq!(body.lines().count(), remote.row_count);
            assert_eq!(
                remote.remote_prefix,
                format!("aws_spire_node_{}", remote.node_id)
            );
            assert!(remote
                .remote_load_args
                .contains(&"--corpus-only".to_owned()));
            assert!(remote
                .remote_load_args
                .contains(&remote.remote_index_regclass));
            assert!(remote.remote_load_args.contains(&"--dim".to_owned()));
            assert!(remote.remote_load_args.contains(&"2".to_owned()));
            assert!(remote
                .remote_load_args
                .contains(&"--storage-format".to_owned()));
            assert!(remote.remote_load_args.contains(&"rabitq".to_owned()));
            assert!(remote.remote_load_args.contains(&"--reloption".to_owned()));
            assert!(remote.remote_load_args.contains(&"nlists=8".to_owned()));
            assert!(remote
                .remote_identity_query_sql
                .contains("ec_spire_remote_search_endpoint_identity"));
            assert!(remote
                .remote_identity_query_sql
                .contains("ec_spire_index_active_snapshot_diagnostics"));
            assert!(remote
                .coordinator_register_descriptor_sql_template
                .contains("ec_spire_register_remote_node_descriptor"));
            assert!(remote
                .coordinator_register_descriptor_sql_template
                .contains("decode('{remote_index_identity_hex}', 'hex')"));
            assert!(remote
                .coordinator_register_descriptor_sql_template
                .contains("'{extension_version}'"));
        }

        let plan_json: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(output_dir.join("distributed-placement-plan.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(plan_json["total_rows"], 5);
        assert_eq!(plan_json["dimension"], 2);
        assert_eq!(plan_json["shard_count"], 4);
    }

    #[test]
    fn distributed_descriptor_registration_sql_uses_remote_endpoint_identity() {
        let identity_sql = distributed_remote_identity_query_sql("public.remote_idx");
        assert!(identity_sql.contains("'public.remote_idx'::regclass::oid"));
        assert!(identity_sql.contains("'remote_index_identity_hex', e.profile_fingerprint"));
        assert!(identity_sql.contains("'last_served_epoch', a.active_epoch"));
        assert!(identity_sql.contains("'min_retained_epoch', a.active_epoch"));
        assert!(!identity_sql.contains("'active_epoch'"));

        let register_sql = distributed_coordinator_register_descriptor_sql_template(
            "public.coord_idx",
            2,
            "spire/remote/a",
            "public.remote_idx",
        );
        assert!(register_sql.contains("'public.coord_idx'::regclass::oid"));
        assert!(register_sql.contains("2"));
        assert!(register_sql.contains("'spire/remote/a'"));
        assert!(register_sql.contains("decode('{remote_index_identity_hex}', 'hex')"));
        assert!(register_sql.contains("'{extension_version}'"));
    }

    #[test]
    fn sql_string_literal_escapes_single_quotes() {
        assert_eq!(sql_string_literal("a'b"), "'a''b'");
    }

    #[test]
    fn distributed_placement_output_rejects_non_id_source_identity_column() {
        let config: DistributedPlacementConfig = serde_json::from_value(serde_json::json!({
            "version": 1,
            "coordinator": {"index_name": "aws_spire_idx"},
            "remotes": [{
                "node_id": 2,
                "conninfo_secret_name": "spire/remote/a",
                "remote_index_regclass": "public.aws_spire_remote_a_idx",
                "shard_ids": [0]
            }],
            "shard_policy": {
                "kind": "hash_source_identity",
                "shard_count": 1,
                "source_identity_column": "external_id"
            }
        }))
        .unwrap();
        let loaded = LoadedDistributedPlacementConfig {
            path: PathBuf::from("placement.json"),
            config,
        };
        let td = TempDir::new().unwrap();
        let corpus_path = write(&td, "corpus.tsv", "1\t[0.1]\n");

        let err = write_distributed_placement_outputs(
            td.path(),
            &loaded,
            "aws_spire",
            &EC_SPIRE,
            &[corpus_path],
            1,
            4,
            42,
            None,
            &[],
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("source_identity_column"), "err: {err}");
        assert!(err.contains("use \"id\""), "err: {err}");
    }

    // --- manifest orchestration ---

    fn write(dir: &TempDir, name: &str, body: &str) -> PathBuf {
        let p = dir.path().join(name);
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        p
    }

    #[test]
    fn manifest_orchestration_no_derived_no_explicit_is_ok() {
        let td = TempDir::new().unwrap();
        let corpus = write(&td, "odd_name.txt", "");
        let queries = write(&td, "other.txt", "");
        let res = verify_manifest_if_present(
            None,
            &corpus,
            &queries,
            "p",
            4,
            &stats(1, &"a".repeat(64)),
            &stats(1, &"b".repeat(64)),
            false,
        );
        assert!(res.is_ok());
    }

    #[test]
    fn manifest_orchestration_explicit_missing_errs() {
        let td = TempDir::new().unwrap();
        let corpus = write(&td, "x_corpus.tsv", "");
        let queries = write(&td, "x_queries.tsv", "");
        let missing = td.path().join("nope.json");
        let err = verify_manifest_if_present(
            Some(&missing),
            &corpus,
            &queries,
            "x",
            4,
            &stats(1, &"a".repeat(64)),
            &stats(1, &"b".repeat(64)),
            false,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("does not exist"), "err: {err}");
    }

    #[test]
    fn manifest_orchestration_sibling_auto_discovered_verified() {
        let td = TempDir::new().unwrap();
        let corpus = write(&td, "x_corpus.tsv", "");
        let queries = write(&td, "x_queries.tsv", "");
        let manifest_path = td.path().join("x_manifest.json");
        let body = serde_json::json!({
            "manifest_version": 1,
            "prefix": "x",
            "dimension": 4,
            "corpus": {
                "file": "x_corpus.tsv", "rows": 1,
                "sha256": "a".repeat(64), "first_id": 0, "last_id": 0
            },
            "queries": {
                "file": "x_queries.tsv", "rows": 1,
                "sha256": "b".repeat(64), "first_id": 0, "last_id": 0
            }
        })
        .to_string();
        std::fs::write(&manifest_path, body).unwrap();
        verify_manifest_if_present(
            None,
            &corpus,
            &queries,
            "x",
            4,
            &stats(1, &"a".repeat(64)),
            &stats(1, &"b".repeat(64)),
            false,
        )
        .unwrap();
    }

    #[test]
    fn manifest_orchestration_mismatch_errs_unless_allowed() {
        let td = TempDir::new().unwrap();
        let corpus = write(&td, "x_corpus.tsv", "");
        let queries = write(&td, "x_queries.tsv", "");
        let manifest_path = td.path().join("x_manifest.json");
        let body = serde_json::json!({
            "manifest_version": 1, "prefix": "x", "dimension": 4,
            "corpus": { "file": "x_corpus.tsv", "rows": 99,
                        "sha256": "a".repeat(64), "first_id": 0, "last_id": 0 },
            "queries": { "file": "x_queries.tsv", "rows": 1,
                         "sha256": "b".repeat(64), "first_id": 0, "last_id": 0 },
        })
        .to_string();
        std::fs::write(&manifest_path, body).unwrap();

        let strict = verify_manifest_if_present(
            None,
            &corpus,
            &queries,
            "x",
            4,
            &stats(1, &"a".repeat(64)),
            &stats(1, &"b".repeat(64)),
            false,
        );
        assert!(strict.is_err());
        let lenient = verify_manifest_if_present(
            None,
            &corpus,
            &queries,
            "x",
            4,
            &stats(1, &"a".repeat(64)),
            &stats(1, &"b".repeat(64)),
            true,
        );
        assert!(lenient.is_ok());
    }

    fn chunk(path: &str, kind: &str, start_row: i64, rows: usize) -> manifest::ChunkManifest {
        manifest::ChunkManifest {
            path: path.to_owned(),
            kind: kind.to_owned(),
            start_row,
            end_row: start_row + rows as i64 - 1,
            rows,
            byte_length: 10,
            sha256: format!("{kind}-{start_row}"),
        }
    }

    fn chunked_section(kind: &str) -> manifest::ChunkedFileManifest {
        let chunks = if kind == "corpus" {
            vec![
                chunk("corpus/corpus-00000.tsv", kind, 0, 2),
                chunk("corpus/corpus-00001.tsv", kind, 2, 1),
            ]
        } else {
            vec![chunk("queries/queries-00000.tsv", kind, 3, 1)]
        };
        manifest::ChunkedFileManifest {
            rows: chunks.iter().map(|c| c.rows).sum(),
            first_id: chunks.first().map(|c| c.start_row),
            last_id: chunks.last().map(|c| c.end_row),
            first_source_id: Some("a".into()),
            last_source_id: Some("z".into()),
            chunks,
        }
    }

    #[test]
    fn chunk_state_validation_accepts_matching_partial_resume() {
        let section = chunked_section("corpus");
        let state = vec![ChunkStateRow {
            chunk_path: "corpus/corpus-00000.tsv".into(),
            chunk_sha256: "corpus-0".into(),
            row_count: 2,
        }];
        validate_existing_chunk_state("t_corpus", LoadKind::Corpus, &section, 2, &state).unwrap();
    }

    #[test]
    fn chunk_state_validation_rejects_rows_without_state() {
        let section = chunked_section("corpus");
        let err = validate_existing_chunk_state("t_corpus", LoadKind::Corpus, &section, 2, &[])
            .unwrap_err()
            .to_string();
        assert!(err.contains("no corpus state rows"), "err: {err}");
    }

    #[test]
    fn load_chunked_manifest_detects_chunked_layout() {
        let td = TempDir::new().unwrap();
        let manifest_path = td.path().join("x_manifest.json");
        std::fs::write(
            &manifest_path,
            serde_json::json!({
                "manifest_version": 1,
                "artifact_layout": "chunked",
                "prefix": "x",
                "source_dataset": "dbpedia",
                "source_parquet": "/tmp/dbpedia",
                "source_parquet_basename": "dbpedia",
                "source_parquet_shard_basenames": ["part-0.parquet"],
                "id_column": "_id",
                "vector_column": "embedding",
                "dimension": 4,
                "chunk_rows": 2,
                "selection_rule": {},
                "corpus": chunked_section("corpus"),
                "queries": chunked_section("queries"),
                "generated_at_utc": "2026-04-26T00:00:00Z",
                "generated_by": "ecaz corpus prepare"
            })
            .to_string(),
        )
        .unwrap();
        let loaded = load_chunked_manifest_if_requested(Some(&manifest_path), false, "x", 4)
            .unwrap()
            .unwrap();
        assert_eq!(loaded.base_dir, td.path());
        assert_eq!(loaded.manifest.corpus.chunks.len(), 2);
    }
}
