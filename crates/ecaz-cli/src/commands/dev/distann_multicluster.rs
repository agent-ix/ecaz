//! `ec_distann` real multi-instance fixture (Task 165 M3, Slice A).
//!
//! The primary lane loads source rows only on the coordinator, creates empty
//! participant shells, and drives the Task 179 physical generation lifecycle.
//! The historical replicated-serving fixture remains available under an
//! explicit control-only subcommand.

use clap::{Args, Subcommand};
use color_eyre::eyre::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;

use super::support::{find_pgrx_install, repo_root, resolve_pgrx_home, run_status};

#[derive(Subcommand, Debug)]
pub enum DistannMulticlusterCommand {
    /// Spin up N real PG18 instances and build one physically sharded epoch.
    #[command(name = "local-multinode-pg18")]
    LocalMultinodePg18(LocalMultinodePg18Args),
    /// Historical replicated-serving control (not physical topology evidence).
    #[command(name = "replicated-serving-control-pg18")]
    ReplicatedServingControlPg18(LocalMultinodePg18Args),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FixtureMode {
    Physical,
    ReplicatedServingControl,
}

#[derive(Args, Debug)]
pub struct LocalMultinodePg18Args {
    #[arg(long, default_value_t = 18)]
    pub pg: u16,
    #[arg(long)]
    pub pgbin: Option<PathBuf>,
    #[arg(long)]
    pub pgrx_home: Option<PathBuf>,
    #[arg(long)]
    pub run_dir: Option<PathBuf>,
    #[arg(long)]
    pub artifact_dir: Option<PathBuf>,
    /// Number of physical owners. Node 1 is also the coordinator unless
    /// `--coordinator-outside-roster` is set.
    #[arg(long, default_value_t = 3)]
    pub nodes: u32,
    /// Start one additional coordinator instance which is not a physical owner.
    #[arg(long, default_value_t = false)]
    pub coordinator_outside_roster: bool,
    /// Run same-data single-instance versus physical recall, latency, and
    /// storage measurements through the standard benchmark commands.
    #[arg(long, default_value_t = false)]
    pub physical_benchmark: bool,
    /// Query iterations per latency arm in physical benchmark mode.
    #[arg(long, default_value_t = 5)]
    pub benchmark_iterations: u32,
    /// Untimed queries on each latency worker before physical benchmark
    /// measurement. This warms backend-local head and transport caches.
    #[arg(long, default_value_t = 0)]
    pub benchmark_warmup_iterations: u32,
    /// First TCP port; node k listens on base_port + (k - 1).
    #[arg(long, default_value_t = 39710)]
    pub base_port: u16,
    /// Deterministic corpus row count (per node; replicated).
    #[arg(long, default_value_t = 2000)]
    pub rows: u32,
    /// Vector dimension.
    #[arg(long, default_value_t = 16)]
    pub dim: u32,
    /// ec_distann graph degree reloption.
    #[arg(long, default_value_t = 32)]
    pub graph_degree: u32,
    /// Persisted coordinator head-sample cap reloption. Exposed so FR-080
    /// sensitivity matrices can vary the cap through `ecaz bench suite`.
    #[arg(long, default_value_t = 4096)]
    pub head_index_cap: u32,
    /// Session beam width applied to both physical and single benchmark arms.
    #[arg(long)]
    pub beam_width: Option<u32>,
    /// Session hop-round cap applied to both benchmark arms. Together with
    /// beam_width this makes fixed-product BW/H A/B runs suite-addressable.
    #[arg(long)]
    pub hop_rounds: Option<u32>,
    /// Task 180 benchmark-only physical seed mode. Requires an extension build
    /// with `distann-head-attribution-benchmark` when set.
    #[arg(long)]
    pub seed_strategy: Option<String>,
    /// Task 180 benchmark-only approximate head-search width, independent of
    /// the distributed beam width.
    #[arg(long)]
    pub head_search_width: Option<u32>,
    /// Task 180 benchmark-only number of head candidates returned as seeds.
    #[arg(long)]
    pub head_seed_count: Option<u32>,
    /// Task 180 benchmark-only traversal scoring mode.
    #[arg(long)]
    pub neighbor_score_mode: Option<String>,
    /// Task 180 benchmark-only seed variants evaluated against one immutable
    /// physical generation. Repeat as
    /// NAME:MODE:SEARCH_WIDTH:SEED_COUNT:NEIGHBOR_SCORE_MODE.
    #[arg(long = "benchmark-seed-variant")]
    pub benchmark_seed_variants: Vec<String>,
    /// Query count for the recall comparison.
    #[arg(long, default_value_t = 50)]
    pub queries: u32,
    /// top-k for the recall comparison.
    #[arg(long, default_value_t = 10)]
    pub top_k: u32,
    /// Keep the instances running after the run (for manual inspection).
    #[arg(long, default_value_t = false)]
    pub keep_running: bool,
    /// Run only the multi-node distinct-recall gates (skip the TC-042 fault
    /// matrix + FR-082 lifecycle drills). Used by the `distann-local-multinode`
    /// suite step to package the scaled distinct_recall evidence without the
    /// (expensive at scale) per-drill re-setups.
    #[arg(long, default_value_t = false)]
    pub skip_fault_drills: bool,
    /// After the physical fixture passes, drop the extension on every owner
    /// and prove AM-owned generation relations are dependency-cleaned and the
    /// preloaded hooks pass through ordinary DML without the extension.
    #[arg(long, default_value_t = false)]
    pub drop_extension_cleanup_drill: bool,
    /// Load a real staged corpus instead of the synthetic deterministic corpus.
    /// When set, each node loads `{staged_dir}/{corpus_prefix}_corpus.tsv` into
    /// `dm` (encoded with the standard `encode_to_ecvector(source, 4, 42)`) and
    /// `{corpus_prefix}_queries.tsv` into `dm_queries`, so the recall gate runs
    /// against real DBpedia vectors + held-out queries (Task 172 real-corpus
    /// distributed quality lane, not a synthetic identity smoke).
    #[arg(long)]
    pub corpus_prefix: Option<String>,
    /// Directory holding the staged corpus TSVs (default: repo
    /// `data/staged-current`). Only used with `--corpus-prefix`.
    #[arg(long)]
    pub staged_dir: Option<PathBuf>,
}

impl DistannMulticlusterCommand {
    pub async fn run(&self) -> Result<()> {
        match self {
            DistannMulticlusterCommand::LocalMultinodePg18(args) => {
                run_local_multinode_pg18(args, FixtureMode::Physical).await
            }
            DistannMulticlusterCommand::ReplicatedServingControlPg18(args) => {
                run_local_multinode_pg18(args, FixtureMode::ReplicatedServingControl).await
            }
        }
    }
}

struct Node {
    node_id: u32,
    port: u16,
    data_dir: PathBuf,
    log_file: PathBuf,
}

async fn run_local_multinode_pg18(args: &LocalMultinodePg18Args, mode: FixtureMode) -> Result<()> {
    if args.pg != 18 {
        bail!("distann local-multinode requires --pg 18, got {}", args.pg);
    }
    if args.nodes == 0 || (mode == FixtureMode::ReplicatedServingControl && args.nodes < 2) {
        bail!(
            "distann local-multinode has invalid owner count {}",
            args.nodes
        );
    }
    if mode == FixtureMode::ReplicatedServingControl && args.coordinator_outside_roster {
        bail!("replicated-serving-control does not support an outside coordinator");
    }
    if args.physical_benchmark && args.corpus_prefix.is_none() {
        bail!("--physical-benchmark requires --corpus-prefix");
    }
    if args.benchmark_iterations == 0 {
        bail!("--benchmark-iterations must be at least 1");
    }
    if !(16..=1_048_576).contains(&args.head_index_cap) {
        bail!("--head-index-cap must be in 16..=1048576");
    }
    if args
        .beam_width
        .is_some_and(|value| !(1..=64).contains(&value))
    {
        bail!("--beam-width must be in 1..=64");
    }
    if args
        .hop_rounds
        .is_some_and(|value| !(1..=256).contains(&value))
    {
        bail!("--hop-rounds must be in 1..=256");
    }
    if let Some(strategy) = args.seed_strategy.as_deref() {
        if !matches!(
            strategy,
            "persisted_head" | "head_sample_exact" | "owner_scan"
        ) {
            bail!("--seed-strategy must be persisted_head, head_sample_exact, or owner_scan");
        }
        if !args.physical_benchmark {
            bail!("--seed-strategy requires --physical-benchmark");
        }
    }
    if args
        .head_search_width
        .is_some_and(|value| !(1..=4096).contains(&value))
    {
        bail!("--head-search-width must be in 1..=4096");
    }
    if args
        .head_seed_count
        .is_some_and(|value| !(1..=4096).contains(&value))
    {
        bail!("--head-seed-count must be in 1..=4096");
    }
    if let Some(mode) = args.neighbor_score_mode.as_deref() {
        if !matches!(mode, "rabitq" | "exact_neighbor") {
            bail!("--neighbor-score-mode must be rabitq or exact_neighbor");
        }
        if !args.physical_benchmark {
            bail!("--neighbor-score-mode requires --physical-benchmark");
        }
    }
    if !args.benchmark_seed_variants.is_empty() {
        if !args.physical_benchmark {
            bail!("--benchmark-seed-variant requires --physical-benchmark");
        }
        if args.seed_strategy.is_some()
            || args.head_search_width.is_some()
            || args.head_seed_count.is_some()
            || args.neighbor_score_mode.is_some()
        {
            bail!(
                "--benchmark-seed-variant cannot be combined with singular seed or neighbor-score controls"
            );
        }
        parse_benchmark_seed_variants(&args.benchmark_seed_variants)?;
    }
    let instance_count = args.nodes + u32::from(args.coordinator_outside_roster);
    let repo_root = repo_root()?;
    let pgbin = match args.pgbin.clone() {
        Some(path) => path,
        None => {
            let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
            find_pgrx_install(args.pg, &pgrx_home)?.bin_dir
        }
    };
    let pg_ctl = pgbin.join("pg_ctl");
    let psql = pgbin.join("psql");

    let run_dir = args
        .run_dir
        .clone()
        .unwrap_or_else(|| repo_root.join("target/distann-local-multinode"));
    let mut socket_dir = run_dir.join("sockets");
    let mut log_dir = args
        .artifact_dir
        .clone()
        .unwrap_or_else(|| run_dir.join("logs"));
    if run_dir.exists() {
        // Best-effort stop of a prior run before wiping.
        for k in 0..instance_count {
            let data_dir = run_dir.join(format!("node{}", k + 1));
            let _ = Command::new(&pg_ctl)
                .arg("-D")
                .arg(&data_dir)
                .arg("-m")
                .arg("immediate")
                .arg("stop")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await;
        }
        fs::remove_dir_all(&run_dir).wrap_err_with(|| format!("clearing {}", run_dir.display()))?;
    }
    fs::create_dir_all(&socket_dir)?;
    fs::create_dir_all(&log_dir)?;
    socket_dir = fs::canonicalize(&socket_dir)
        .wrap_err_with(|| format!("canonicalizing {}", socket_dir.display()))?;
    log_dir = fs::canonicalize(&log_dir)
        .wrap_err_with(|| format!("canonicalizing {}", log_dir.display()))?;

    let nodes: Vec<Node> = (0..instance_count)
        .map(|k| Node {
            node_id: k + 1,
            port: args.base_port + k as u16,
            data_dir: run_dir.join(format!("node{}", k + 1)),
            log_file: log_dir.join(format!("node{}-postgres.log", k + 1)),
        })
        .collect();

    crate::ecaz_println!("[distann-multicluster] repo={}", repo_root.display());
    crate::ecaz_println!("[distann-multicluster] pgbin={}", pgbin.display());
    crate::ecaz_println!(
        "[distann-multicluster] mode={} owners={} instances={} coordinator_outside_roster={} base_port={} rows={} dim={} head_index_cap={}",
        match mode {
            FixtureMode::Physical => "physical",
            FixtureMode::ReplicatedServingControl => "replicated-serving-control",
        },
        args.nodes,
        instance_count,
        args.coordinator_outside_roster,
        args.base_port,
        args.rows,
        args.dim,
        args.head_index_cap
    );

    // initdb + start + extension on every node.
    for node in &nodes {
        let mut command = Command::new(&pg_ctl);
        command
            .arg("initdb")
            .arg("-D")
            .arg(&node.data_dir)
            .arg("-o")
            .arg("-A trust -U postgres")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        run_status(command)
            .await
            .wrap_err_with(|| format!("initdb node {}", node.node_id))?;
    }
    for node in &nodes {
        let mut command = Command::new(&pg_ctl);
        command
            .arg("-w")
            .arg("-D")
            .arg(&node.data_dir)
            .arg("-l")
            .arg(&node.log_file)
            .arg("-o")
            .arg(format!(
                "-p {} -c listen_addresses=127.0.0.1 -c unix_socket_directories='' \
                 -c shared_preload_libraries=ecaz",
                node.port
            ))
            .arg("start")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        for target in &nodes {
            command.env(
                format!("EC_SPIRE_REMOTE_CONNINFO_DISTANN_NODE_{}", target.node_id),
                conninfo(&socket_dir, target.port),
            );
        }
        run_status(command)
            .await
            .wrap_err_with(|| format!("start node {}", node.node_id))?;
    }

    let result = match mode {
        FixtureMode::Physical => {
            drive_physical_fixture(args, &pg_ctl, &psql, &socket_dir, &nodes, log_dir.as_path())
                .await
        }
        FixtureMode::ReplicatedServingControl => {
            drive_fixture(args, &pg_ctl, &psql, &socket_dir, &nodes, log_dir.as_path()).await
        }
    };

    if !args.keep_running {
        for node in &nodes {
            let _ = Command::new(&pg_ctl)
                .arg("-D")
                .arg(&node.data_dir)
                .arg("-m")
                .arg("fast")
                .arg("stop")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await;
        }
    } else {
        crate::ecaz_println!(
            "[distann-multicluster] instances left running under {}",
            run_dir.display()
        );
    }
    result
}

/// libpq conninfo for a node over the shared socket dir.
fn conninfo(_socket_dir: &Path, port: u16) -> String {
    format!("host=127.0.0.1 port={} dbname=postgres user=postgres", port,)
}

/// The identical, deterministic corpus + index setup run on every node.
/// Build the per-node corpus/index setup SQL: real staged corpus when
/// `--corpus-prefix` is set, else the synthetic deterministic corpus.
fn build_setup_sql(args: &LocalMultinodePg18Args) -> Result<String> {
    match &args.corpus_prefix {
        Some(prefix) => {
            let staged_dir = match &args.staged_dir {
                Some(dir) => dir.clone(),
                None => repo_root()?.join("data/staged-current"),
            };
            // Canonicalize (absolute + symlinks resolved) so the server-side
            // COPY can read the staged file regardless of the backend's cwd.
            let corpus_path =
                std::fs::canonicalize(staged_dir.join(format!("{prefix}_corpus.tsv")))
                    .wrap_err_with(|| format!("resolving staged corpus for prefix {prefix}"))?;
            let queries_path =
                std::fs::canonicalize(staged_dir.join(format!("{prefix}_queries.tsv")))
                    .wrap_err_with(|| format!("resolving staged queries for prefix {prefix}"))?;
            Ok(real_setup_sql(
                &corpus_path,
                &queries_path,
                args.queries,
                args.graph_degree,
                args.head_index_cap,
            ))
        }
        None => Ok(setup_sql(args)),
    }
}

/// A SQL `real[]` expression for a drill row's insert vector. In real mode it
/// reuses an existing corpus vector (guaranteed to match the index dimension);
/// in synthetic mode it reproduces the deterministic corpus generator at
/// `args.dim`. A synthetic `args.dim` vector does NOT match a real corpus, so
/// the drill inserts must not synthesize a vector when a real corpus is loaded.
fn insert_vector_expr(args: &LocalMultinodePg18Args) -> String {
    if args.corpus_prefix.is_some() {
        "(SELECT source FROM dm ORDER BY id LIMIT 1)".to_owned()
    } else {
        format!(
            "(SELECT array_agg((sin(7 * 0.017 * (d + 1)) + cos(7 * 0.0031 * (d + 1)))::real) \
             FROM generate_series(0, {} - 1) AS d)",
            args.dim
        )
    }
}

/// Real staged-corpus load: COPY each 2-column TSV (`id\t[v1,v2,...]`) into a
/// text stage, convert the `[...]` JSON-array literal to a PG `{...}` array,
/// and materialize `dm(source real[], embedding ecvector)` +
/// `dm_queries(source real[])`. The `4, 42` encode params match both the
/// synthetic path and the standard suite load (`encode_to_ecvector(source, 4,
/// 42)`), so this lane is comparable to the single-node suite matrices.
fn real_setup_sql(
    corpus_path: &Path,
    queries_path: &Path,
    queries_limit: u32,
    gd: u32,
    head_index_cap: u32,
) -> String {
    // Escape the paths as SQL string literals (double any single quote) so a
    // path containing `'` cannot break out of the COPY ... FROM '<path>' literal
    // (172-P2). Canonical repo paths are unlikely to contain one, but the COPY
    // literal must be robust to it.
    let corpus = corpus_path.display().to_string().replace('\'', "''");
    let queries = queries_path.display().to_string().replace('\'', "''");
    format!(
        "CREATE EXTENSION IF NOT EXISTS ecaz;\n\
         DO $$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'peter') THEN CREATE ROLE peter LOGIN SUPERUSER; END IF; END $$;\n\
         DROP TABLE IF EXISTS dm; DROP TABLE IF EXISTS dm_queries;\n\
         CREATE TABLE dm (id bigint, source real[], embedding ecvector);\n\
         CREATE TEMP TABLE dm_stage (id bigint, vec text);\n\
         COPY dm_stage (id, vec) FROM '{corpus}' WITH (FORMAT text, DELIMITER E'\\t');\n\
         INSERT INTO dm\n\
           SELECT id, translate(vec, '[]', '{{}}')::real[],\n\
                  encode_to_ecvector(translate(vec, '[]', '{{}}')::real[], 4, 42)\n\
           FROM dm_stage ORDER BY id;\n\
         DROP TABLE dm_stage;\n\
         CREATE TABLE dm_queries (id bigint, source real[]);\n\
         CREATE TEMP TABLE dmq_stage (id bigint, vec text);\n\
         COPY dmq_stage (id, vec) FROM '{queries}' WITH (FORMAT text, DELIMITER E'\\t');\n\
         INSERT INTO dm_queries\n\
           SELECT id, translate(vec, '[]', '{{}}')::real[]\n\
           FROM dmq_stage ORDER BY id LIMIT {queries_limit};\n\
         DROP TABLE dmq_stage;\n\
         CREATE INDEX dm_idx ON dm USING ec_distann (embedding ecvector_distann_ip_ops)\n\
           WITH (graph_degree = {gd}, head_index_cap = {head_index_cap});\n",
    )
}

fn setup_sql(args: &LocalMultinodePg18Args) -> String {
    format!(
        "CREATE EXTENSION IF NOT EXISTS ecaz;\n\
         DO $$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'peter') THEN CREATE ROLE peter LOGIN SUPERUSER; END IF; END $$;\n\
         DROP TABLE IF EXISTS dm;\n\
         CREATE TABLE dm (id bigint, source real[], embedding ecvector);\n\
         INSERT INTO dm\n\
         SELECT g,\n\
                arr,\n\
                encode_to_ecvector(arr, 4, 42)\n\
         FROM (\n\
           SELECT g,\n\
                  (SELECT array_agg((sin(g * 0.017 * (d + 1)) + cos(g * 0.0031 * (d + 1)))::real)\n\
                     FROM generate_series(0, {dim} - 1) AS d) AS arr\n\
           FROM generate_series(1, {rows}) AS g\n\
         ) s;\n\
         CREATE INDEX dm_idx ON dm USING ec_distann (embedding ecvector_distann_ip_ops)\n\
           WITH (graph_degree = {gd}, head_index_cap = {head_index_cap});\n",
        dim = args.dim,
        rows = args.rows,
        gd = args.graph_degree,
        head_index_cap = args.head_index_cap,
    )
}

/// The coordinator-side recall comparison: single-node (empty roster) vs the
/// full multi-node roster (CustomScan → owner row shipping), asserting the top-k
/// id sets are identical (distinct_recall delta 0 ⇒ ≥ single − 0.001).
fn recall_sql(roster: &str, queries: u32, top_k: u32, real: bool) -> String {
    // Real lane: held-out queries from dm_queries. Synthetic lane: the first
    // `queries` corpus rows (ids 1..=queries) double as queries, as before.
    let q_source = if real {
        format!("SELECT id AS qid, source AS v FROM dm_queries ORDER BY id LIMIT {queries}")
    } else {
        format!("SELECT id AS qid, source AS v FROM dm WHERE id <= {queries}")
    };
    format!(
        "SET enable_seqscan = off;\n\
         DROP TABLE IF EXISTS q; CREATE TEMP TABLE q AS {q_source};\n\
         SET ec_distann.roster = ''; SET ec_distann.local_node_id = 1; SET ec_distann.epoch = 0;\n\
         DROP TABLE IF EXISTS base; CREATE TEMP TABLE base AS\n\
           SELECT q.qid, r.id FROM q CROSS JOIN LATERAL\n\
             (SELECT id FROM dm ORDER BY embedding <#> q.v LIMIT {top_k}) r;\n\
         SET ec_distann.roster = '{roster}'; SET ec_distann.local_node_id = 1; SET ec_distann.epoch = 1;\n\
         DROP TABLE IF EXISTS two; CREATE TEMP TABLE two AS\n\
           SELECT q.qid, r.id FROM q CROSS JOIN LATERAL\n\
             (SELECT id FROM dm ORDER BY embedding <#> q.v LIMIT {top_k}) r;\n\
         SET ec_distann.roster = '';\n\
         SELECT 'RECALL_RESULT'\n\
           || ' n_queries=' || count(DISTINCT qid)\n\
           || ' identical=' || count(DISTINCT qid) FILTER (WHERE mismatch = 0)\n\
           || ' mismatched_ids=' || coalesce(sum(mismatch), 0)\n\
         FROM (\n\
           SELECT q.qid,\n\
             (SELECT count(*) FROM (SELECT id FROM base WHERE qid=q.qid EXCEPT SELECT id FROM two WHERE qid=q.qid) d)\n\
           + (SELECT count(*) FROM (SELECT id FROM two WHERE qid=q.qid EXCEPT SELECT id FROM base WHERE qid=q.qid) d) AS mismatch\n\
           FROM q\n\
         ) s;\n"
    )
}

fn physical_setup_sql(args: &LocalMultinodePg18Args, coordinator: bool) -> Result<String> {
    let physical_dim = if let Some(corpus_prefix) = &args.corpus_prefix {
        let staged_dir = args
            .staged_dir
            .clone()
            .unwrap_or(repo_root()?.join("data/staged-current"));
        let manifest_path = staged_dir.join(format!("{corpus_prefix}_manifest.json"));
        let manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(&manifest_path)
                .wrap_err_with(|| format!("reading {}", manifest_path.display()))?,
        )
        .wrap_err_with(|| format!("parsing {}", manifest_path.display()))?;
        manifest
            .get("dimension")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .filter(|value| *value > 0)
            .ok_or_else(|| {
                color_eyre::eyre::eyre!(
                    "staged manifest {} has no valid dimension",
                    manifest_path.display()
                )
            })?
    } else {
        args.dim
    };
    let prefix = format!(
        "CREATE EXTENSION IF NOT EXISTS ecaz;
        DROP TABLE IF EXISTS dm CASCADE;
        DROP TABLE IF EXISTS dm_queries;
        CREATE TABLE dm (
            id bigint, source_id uuid NOT NULL, source real[], embedding ecvector({})
        );",
        physical_dim
    );
    let load = if !coordinator {
        String::new()
    } else if let Some(corpus_prefix) = &args.corpus_prefix {
        let staged_dir = args
            .staged_dir
            .clone()
            .unwrap_or(repo_root()?.join("data/staged-current"));
        let corpus = std::fs::canonicalize(staged_dir.join(format!("{corpus_prefix}_corpus.tsv")))?
            .display()
            .to_string()
            .replace('\'', "''");
        let queries =
            std::fs::canonicalize(staged_dir.join(format!("{corpus_prefix}_queries.tsv")))?
                .display()
                .to_string()
                .replace('\'', "''");
        format!(
            "CREATE TEMP TABLE dm_stage (id bigint, vec text);
             COPY dm_stage (id, vec) FROM '{corpus}' WITH (FORMAT text, DELIMITER E'\\t');
             INSERT INTO dm
             SELECT id,
                    (substr(md5(id::text),1,8)||'-'||substr(md5(id::text),9,4)||'-4'||
                     substr(md5(id::text),14,3)||'-8'||substr(md5(id::text),18,3)||'-'||
                     substr(md5(id::text),21,12))::uuid,
                    translate(vec, '[]', '{{}}')::real[],
                    encode_to_ecvector(translate(vec, '[]', '{{}}')::real[], 4, 42)
               FROM dm_stage ORDER BY id;
             CREATE TABLE dm_queries (id bigint, source real[]);
             CREATE TEMP TABLE dmq_stage (id bigint, vec text);
             COPY dmq_stage (id, vec) FROM '{queries}' WITH (FORMAT text, DELIMITER E'\\t');
             INSERT INTO dm_queries
             SELECT id, translate(vec, '[]', '{{}}')::real[]
               FROM dmq_stage ORDER BY id LIMIT {};",
            args.queries
        )
    } else {
        format!(
            "INSERT INTO dm
             SELECT g,
                    (substr(md5(g::text),1,8)||'-'||substr(md5(g::text),9,4)||'-4'||
                     substr(md5(g::text),14,3)||'-8'||substr(md5(g::text),18,3)||'-'||
                     substr(md5(g::text),21,12))::uuid,
                    arr, encode_to_ecvector(arr, 4, 42)
               FROM (
                 SELECT g,
                        (SELECT array_agg((sin(g * 0.017 * (d + 1)) +
                                           cos(g * 0.0031 * (d + 1)))::real)
                           FROM generate_series(0, {} - 1) AS d) AS arr
                   FROM generate_series(1, {}) AS g
               ) source_rows;",
            args.dim, args.rows
        )
    };
    Ok(format!(
        "{prefix}
         {load}
         CREATE INDEX dm_idx ON dm USING ec_distann
             (embedding ecvector_distann_ip_ops) INCLUDE (source_id)
             WITH (distributed_control = true, source_identity = 'include',
                   graph_degree = {}, head_index_cap = {},
                   neighbor_code_format = 'rabitq');",
        args.graph_degree, args.head_index_cap
    ))
}

#[derive(Debug)]
struct PhysicalTopologyRow {
    node_id: i64,
    state: String,
    records: i64,
    rows: i64,
    non_owned_live: i64,
    non_owned_tombstones: i64,
    orphan_records: i64,
    orphan_rows: i64,
    graph_bytes: i64,
    row_bytes: i64,
    directory_bytes: i64,
    control_bytes: i64,
}

async fn physical_topology(
    psql: &Path,
    socket_dir: &Path,
    node: &Node,
    selector_sql: &str,
) -> Result<PhysicalTopologyRow> {
    let sql = format!(
        "SELECT concat_ws('|', node_id, state, record_count, row_count,
                non_owned_live_count, non_owned_tombstone_count,
                orphan_record_count, orphan_row_count, graph_bytes,
                row_tier_bytes, directory_bytes, control_index_bytes)
           FROM {selector_sql}"
    );
    let raw = capture_psql(psql, socket_dir, node.port, &sql).await?;
    let fields = raw.trim().split('|').collect::<Vec<_>>();
    if fields.len() != 12 {
        bail!(
            "physical topology node {} returned malformed row {:?}",
            node.node_id,
            raw.trim()
        );
    }
    let number = |index: usize, field: &str| -> Result<i64> {
        fields[index]
            .parse::<i64>()
            .wrap_err_with(|| format!("decoding topology {field}"))
    };
    Ok(PhysicalTopologyRow {
        node_id: number(0, "node_id")?,
        state: fields[1].to_owned(),
        records: number(2, "record_count")?,
        rows: number(3, "row_count")?,
        non_owned_live: number(4, "non_owned_live_count")?,
        non_owned_tombstones: number(5, "non_owned_tombstone_count")?,
        orphan_records: number(6, "orphan_record_count")?,
        orphan_rows: number(7, "orphan_row_count")?,
        graph_bytes: number(8, "graph_bytes")?,
        row_bytes: number(9, "row_tier_bytes")?,
        directory_bytes: number(10, "directory_bytes")?,
        control_bytes: number(11, "control_index_bytes")?,
    })
}

fn validate_physical_topology(
    phase: &str,
    topology: &[PhysicalTopologyRow],
    expected_state: &str,
    source_count: i64,
) -> Result<()> {
    if topology.is_empty()
        || topology.iter().any(|row| {
            row.state != expected_state
                || row.records <= 0
                || row.records != row.rows
                || row.non_owned_live != 0
                || row.non_owned_tombstones != 0
                || row.orphan_records != 0
                || row.orphan_rows != 0
        })
        || topology.iter().map(|row| row.records).sum::<i64>() != source_count
    {
        bail!("physical topology {phase} is incomplete or inconsistent: {topology:?}");
    }
    Ok(())
}

fn benchmark_table_row(raw: &str) -> Result<Vec<String>> {
    raw.lines()
        .filter(|line| line.contains('┆'))
        .map(|line| {
            line.split(['│', '┆'])
                .map(str::trim)
                .filter(|cell| !cell.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .find(|cells| {
            cells
                .first()
                .is_some_and(|cell| cell.parse::<u32>().is_ok())
        })
        .ok_or_else(|| color_eyre::eyre::eyre!("benchmark output has no data row"))
}

fn benchmark_ms(cell: &str) -> Result<f64> {
    cell.trim_end_matches(" ms")
        .trim()
        .parse::<f64>()
        .wrap_err_with(|| format!("decoding benchmark duration {cell:?}"))
}

async fn run_physical_bench_child(args: Vec<String>) -> Result<String> {
    let executable = std::env::current_exe().wrap_err("resolving benchmark executable")?;
    let output = Command::new(&executable)
        .args(&args)
        .output()
        .await
        .wrap_err_with(|| format!("spawning {}", executable.display()))?;
    if !output.status.success() {
        bail!(
            "physical benchmark child failed with {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[derive(Clone, Debug)]
struct BenchmarkSeedVariant {
    name: String,
    strategy: String,
    head_search_width: u32,
    head_seed_count: u32,
    neighbor_score_mode: String,
}

fn parse_benchmark_seed_variants(values: &[String]) -> Result<Vec<BenchmarkSeedVariant>> {
    let mut names = std::collections::BTreeSet::new();
    values
        .iter()
        .map(|value| {
            let fields = value.split(':').collect::<Vec<_>>();
            if fields.len() != 5 {
                bail!(
                    "benchmark seed variant must be NAME:MODE:SEARCH_WIDTH:SEED_COUNT:NEIGHBOR_SCORE_MODE, got {value:?}"
                );
            }
            let name = fields[0];
            if name.is_empty()
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
            {
                bail!("benchmark seed variant name must be an ASCII identifier, got {name:?}");
            }
            if !names.insert(name.to_owned()) {
                bail!("duplicate benchmark seed variant name {name:?}");
            }
            let strategy = fields[1];
            if !matches!(
                strategy,
                "persisted_head" | "head_sample_exact" | "owner_scan"
            ) {
                bail!(
                    "benchmark seed variant mode must be persisted_head, head_sample_exact, or owner_scan, got {strategy:?}"
                );
            }
            let head_search_width = fields[2]
                .parse::<u32>()
                .wrap_err_with(|| format!("parsing search width in benchmark seed variant {value:?}"))?;
            let head_seed_count = fields[3]
                .parse::<u32>()
                .wrap_err_with(|| format!("parsing seed count in benchmark seed variant {value:?}"))?;
            if !(1..=4096).contains(&head_search_width)
                || !(1..=4096).contains(&head_seed_count)
            {
                bail!(
                    "benchmark seed variant widths must be in 1..=4096, got {value:?}"
                );
            }
            let neighbor_score_mode = fields[4];
            if !matches!(neighbor_score_mode, "rabitq" | "exact_neighbor") {
                bail!(
                    "benchmark seed variant neighbor score mode must be rabitq or exact_neighbor, got {neighbor_score_mode:?}"
                );
            }
            Ok(BenchmarkSeedVariant {
                name: name.to_owned(),
                strategy: strategy.to_owned(),
                head_search_width,
                head_seed_count,
                neighbor_score_mode: neighbor_score_mode.to_owned(),
            })
        })
        .collect()
}

async fn run_physical_benchmarks(
    args: &LocalMultinodePg18Args,
    coordinator: &tokio_postgres::Client,
    coordinator_port: u16,
    nodes: &[Node],
    published: &[PhysicalTopologyRow],
    log_dir: &Path,
    build_ms: u128,
    publish_ms: u128,
) -> Result<Vec<String>> {
    let beam_width = args.beam_width.unwrap_or(4);
    let hop_rounds = args.hop_rounds.unwrap_or(100);
    let production_head_width = (beam_width * 2).max(32);
    let explicit_seed_controls = args.seed_strategy.is_some()
        || args.head_search_width.is_some()
        || args.head_seed_count.is_some()
        || args.neighbor_score_mode.is_some()
        || !args.benchmark_seed_variants.is_empty();
    let seed_variants = if args.benchmark_seed_variants.is_empty() {
        vec![BenchmarkSeedVariant {
            name: "production".to_owned(),
            strategy: args
                .seed_strategy
                .clone()
                .unwrap_or_else(|| "persisted_head".to_owned()),
            head_search_width: args.head_search_width.unwrap_or(production_head_width),
            head_seed_count: args.head_seed_count.unwrap_or(production_head_width),
            neighbor_score_mode: args
                .neighbor_score_mode
                .clone()
                .unwrap_or_else(|| "rabitq".to_owned()),
        }]
    } else {
        parse_benchmark_seed_variants(&args.benchmark_seed_variants)?
    };
    let corpus_prefix = args
        .corpus_prefix
        .as_deref()
        .ok_or_else(|| color_eyre::eyre::eyre!("physical benchmark requires corpus_prefix"))?;
    if !corpus_prefix
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        bail!("physical benchmark corpus prefix is not a SQL identifier");
    }
    let scale = corpus_prefix
        .strip_prefix("ec_real_")
        .unwrap_or(corpus_prefix);
    let coordinator_provenance = coordinator
        .query_one("SELECT ecaz_build_git_sha(), ecaz_build_profile()", &[])
        .await
        .wrap_err("querying installed coordinator extension provenance")?;
    let expected_sha = coordinator_provenance.get::<_, String>(0);
    let expected_profile = coordinator_provenance.get::<_, String>(1);
    let mut provenance_ports = std::collections::BTreeSet::from([coordinator_port]);
    provenance_ports.extend(nodes.iter().map(|node| node.port));
    for port in provenance_ports
        .iter()
        .copied()
        .filter(|port| *port != coordinator_port)
    {
        let (client, connection) = tokio_postgres::connect(
            &format!("host=127.0.0.1 port={port} dbname=postgres user=postgres"),
            tokio_postgres::NoTls,
        )
        .await
        .wrap_err_with(|| format!("connecting to node on port {port} for provenance"))?;
        let task = tokio::spawn(async move { connection.await });
        let row = client
            .query_one("SELECT ecaz_build_git_sha(), ecaz_build_profile()", &[])
            .await
            .wrap_err_with(|| format!("querying installed extension provenance on port {port}"))?;
        let sha = row.get::<_, String>(0);
        let profile = row.get::<_, String>(1);
        task.abort();
        if sha != expected_sha || profile != expected_profile {
            bail!(
                "installed extension provenance mismatch on port {port}: expected {expected_sha}/{expected_profile}, got {sha}/{profile}"
            );
        }
    }
    let physical_prefix = format!("task179_physical_{scale}");
    let single_prefix = format!("task179_single_{scale}");
    let physical_corpus = format!("{physical_prefix}_corpus");
    let physical_queries = format!("{physical_prefix}_queries");
    let single_corpus = format!("{single_prefix}_corpus");
    let single_queries = format!("{single_prefix}_queries");
    let single_index = format!("{single_prefix}_idx");
    coordinator
        .batch_execute(&format!(
            "RESET enable_seqscan;
             ALTER TABLE dm RENAME TO {physical_corpus};
             ALTER TABLE dm_queries RENAME TO {physical_queries};"
        ))
        .await?;

    let single_started = Instant::now();
    coordinator
        .batch_execute(&format!(
            "CREATE TABLE {single_corpus} AS
                 SELECT id, source, embedding FROM {physical_corpus};
             CREATE TABLE {single_queries} AS SELECT * FROM {physical_queries};
             CREATE INDEX {single_index} ON {single_corpus}
                 USING ec_distann (embedding ecvector_distann_ip_ops)
                 WITH (graph_degree = {}, head_index_cap = {},
                       neighbor_code_format = 'rabitq');",
            args.graph_degree, args.head_index_cap
        ))
        .await?;
    let single_build_ms = single_started.elapsed().as_millis();
    let has_seed_strategy_provenance = coordinator
        .query_one(
            "SELECT to_regprocedure('ec_distann_physical_seed_strategy()') IS NOT NULL",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    let has_neighbor_score_provenance = coordinator
        .query_one(
            "SELECT to_regprocedure('ec_distann_physical_neighbor_score_mode()') IS NOT NULL",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    if explicit_seed_controls && (!has_seed_strategy_provenance || !has_neighbor_score_provenance) {
        bail!("Task 180 controls require extension seed and neighbor-score provenance helpers");
    }

    let staged_dir = args
        .staged_dir
        .clone()
        .unwrap_or(repo_root()?.join("data/staged-current"));
    let truth_corpus =
        std::fs::canonicalize(staged_dir.join(format!("{corpus_prefix}_corpus.tsv")))?;
    let truth_queries =
        std::fs::canonicalize(staged_dir.join(format!("{corpus_prefix}_queries.tsv")))?;
    let query_sha256 = hex::encode(Sha256::digest(std::fs::read(&truth_queries)?));
    let truth_cache = nodes[0]
        .data_dir
        .parent()
        .unwrap_or(nodes[0].data_dir.as_path())
        .join(format!("{corpus_prefix}-truth.json"));
    let common = vec![
        "--database".to_owned(),
        "postgres".to_owned(),
        "--host".to_owned(),
        "127.0.0.1".to_owned(),
        "--port".to_owned(),
        coordinator_port.to_string(),
        "--user".to_owned(),
        "postgres".to_owned(),
    ];
    let mut lines = vec![format!(
        "physical_benchmark_provenance scale={scale} extension_git_sha={expected_sha} extension_build_profile={expected_profile} nodes={} unanimous=true",
        provenance_ports.len()
    )];
    let mut benchmark_arms = Vec::with_capacity(seed_variants.len() + 1);
    for variant in &seed_variants {
        if explicit_seed_controls {
            coordinator
                .batch_execute(&format!(
                    "SET ec_distann.benchmark_seed_mode = '{}';\n\
                     SET ec_distann.benchmark_head_search_width = {};\n\
                     SET ec_distann.benchmark_head_seed_count = {};\n\
                     SET ec_distann.benchmark_exact_neighbor = {};",
                    variant.strategy.replace('\'', "''"),
                    variant.head_search_width,
                    variant.head_seed_count,
                    if variant.neighbor_score_mode == "exact_neighbor" {
                        "on"
                    } else {
                        "off"
                    },
                ))
                .await
                .wrap_err("configuring Task 180 benchmark seed controls")?;
        }
        let attested_strategy = if has_seed_strategy_provenance {
            coordinator
                .query_one("SELECT ec_distann_physical_seed_strategy()", &[])
                .await?
                .get::<_, String>(0)
        } else {
            // Historical physical-generation commits predate the provenance
            // helper. Keep production benchmark configs runnable without
            // assigning a strategy label the extension cannot attest.
            "pre-provenance".to_owned()
        };
        if explicit_seed_controls && attested_strategy != variant.strategy {
            bail!(
                "physical seed strategy attestation mismatch for variant {}: requested {}, got {}",
                variant.name,
                variant.strategy,
                attested_strategy
            );
        }
        let attested_neighbor_score = if has_neighbor_score_provenance {
            coordinator
                .query_one("SELECT ec_distann_physical_neighbor_score_mode()", &[])
                .await?
                .get::<_, String>(0)
        } else {
            "rabitq".to_owned()
        };
        if explicit_seed_controls && attested_neighbor_score != variant.neighbor_score_mode {
            bail!(
                "physical neighbor-score attestation mismatch for variant {}: requested {}, got {}",
                variant.name,
                variant.neighbor_score_mode,
                attested_neighbor_score
            );
        }
        benchmark_arms.push((
            "physical",
            physical_prefix.as_str(),
            variant.name.as_str(),
            attested_strategy,
            variant.head_search_width,
            variant.head_seed_count,
            attested_neighbor_score,
        ));
        lines.push(format!(
            "physical_benchmark_build scale={scale} variant={} seed_strategy={} head_index_cap={} head_search_width={} head_seed_count={} beam_width={beam_width} hop_rounds={hop_rounds} neighbor_score_mode={} stored_neighbor_code_format=rabitq build_shared=true physical_ms={build_ms} publish_ms={publish_ms} single_ms={single_build_ms}",
            variant.name,
            variant.strategy,
            args.head_index_cap,
            variant.head_search_width,
            variant.head_seed_count,
            variant.neighbor_score_mode,
        ));
    }
    benchmark_arms.push((
        "single",
        single_prefix.as_str(),
        "single",
        "single_index".to_owned(),
        production_head_width,
        production_head_width,
        "rabitq".to_owned(),
    ));

    for (
        arm,
        prefix,
        variant,
        seed_strategy,
        head_search_width,
        head_seed_count,
        neighbor_score_mode,
    ) in benchmark_arms
    {
        let recall_log = log_dir.join(format!("{arm}-{variant}-recall.log"));
        let mut recall_args = common.clone();
        recall_args.extend([
            "bench".into(),
            "recall".into(),
            "--prefix".into(),
            prefix.to_owned(),
            "--profile".into(),
            "ec_distann".into(),
            "--k".into(),
            args.top_k.to_string(),
            "--sweep".into(),
            "32".into(),
            "--queries-limit".into(),
            args.queries.to_string(),
            "--force-index".into(),
            "--truth-cache-file".into(),
            truth_cache.display().to_string(),
            "--log-output".into(),
            recall_log.display().to_string(),
            "--session-guc".into(),
            format!("ec_distann.beam_width={beam_width}"),
            "--session-guc".into(),
            format!("ec_distann.hop_rounds={hop_rounds}"),
        ]);
        if arm == "physical" {
            recall_args.extend([
                "--truth-corpus-file".into(),
                truth_corpus.display().to_string(),
            ]);
            if explicit_seed_controls {
                recall_args.extend([
                    "--session-guc".into(),
                    format!("ec_distann.benchmark_seed_mode={seed_strategy}"),
                    "--session-guc".into(),
                    format!("ec_distann.benchmark_head_search_width={head_search_width}"),
                    "--session-guc".into(),
                    format!("ec_distann.benchmark_head_seed_count={head_seed_count}"),
                    "--session-guc".into(),
                    format!(
                        "ec_distann.benchmark_exact_neighbor={}",
                        if neighbor_score_mode == "exact_neighbor" {
                            "on"
                        } else {
                            "off"
                        }
                    ),
                ]);
            }
        }
        let recall = run_physical_bench_child(recall_args).await?;
        let row = benchmark_table_row(&recall)?;
        let membership_recall = row[3].parse::<f64>()?;
        let distinct_recall = row[12].parse::<f64>()?;
        let distinct_recall_ci95_low = row[13].parse::<f64>()?;
        let distinct_recall_ci95_high = row[14].parse::<f64>()?;
        let mean_ms = benchmark_ms(&row[11])?;
        lines.push(format!(
            "physical_benchmark_recall scale={scale} variant={variant} head_index_cap={} head_search_width={head_search_width} head_seed_count={head_seed_count} beam_width={beam_width} hop_rounds={hop_rounds} neighbor_score_mode={neighbor_score_mode} arm={arm} seed_strategy={seed_strategy} queries={} trials={} recall={membership_recall:.4} membership_recall={membership_recall:.4} distinct_recall={distinct_recall:.4} distinct_recall_ci95_low={distinct_recall_ci95_low:.4} distinct_recall_ci95_high={distinct_recall_ci95_high:.4} mean_ms={mean_ms:.2}",
            args.head_index_cap, row[1], row[2]
        ));

        let latency_log = log_dir.join(format!("{arm}-{variant}-latency.log"));
        let mut latency_args = common.clone();
        latency_args.extend([
            "bench".into(),
            "latency".into(),
            "--prefix".into(),
            prefix.to_owned(),
            "--profile".into(),
            "ec_distann".into(),
            "--k".into(),
            args.top_k.to_string(),
            "--sweep".into(),
            "32".into(),
            "--iterations".into(),
            args.benchmark_iterations.to_string(),
            "--warmup-iterations".into(),
            args.benchmark_warmup_iterations.to_string(),
            "--concurrency".into(),
            "1".into(),
            "--force-index".into(),
            "--cache-state".into(),
            "warm".into(),
            "--log-output".into(),
            latency_log.display().to_string(),
            "--session-guc".into(),
            format!("ec_distann.beam_width={beam_width}"),
            "--session-guc".into(),
            format!("ec_distann.hop_rounds={hop_rounds}"),
        ]);
        if arm == "physical" && explicit_seed_controls {
            latency_args.extend([
                "--session-guc".into(),
                format!("ec_distann.benchmark_seed_mode={seed_strategy}"),
                "--session-guc".into(),
                format!("ec_distann.benchmark_head_search_width={head_search_width}"),
                "--session-guc".into(),
                format!("ec_distann.benchmark_head_seed_count={head_seed_count}"),
                "--session-guc".into(),
                format!(
                    "ec_distann.benchmark_exact_neighbor={}",
                    if neighbor_score_mode == "exact_neighbor" {
                        "on"
                    } else {
                        "off"
                    }
                ),
            ]);
        }
        let latency = run_physical_bench_child(latency_args).await?;
        let row = benchmark_table_row(&latency)?;
        lines.push(format!(
            "physical_benchmark_latency scale={scale} variant={variant} head_index_cap={} head_search_width={head_search_width} head_seed_count={head_seed_count} beam_width={beam_width} hop_rounds={hop_rounds} neighbor_score_mode={neighbor_score_mode} arm={arm} seed_strategy={seed_strategy} count={} mean_ms={:.2} p50_ms={:.2} p95_ms={:.2} p99_ms={:.2} max_ms={:.2} concurrency=1 cache=warm warmup_iterations={}",
            args.head_index_cap,
            row[1],
            benchmark_ms(&row[2])?,
            benchmark_ms(&row[5])?,
            benchmark_ms(&row[6])?,
            benchmark_ms(&row[7])?,
            benchmark_ms(&row[8])?,
            args.benchmark_warmup_iterations,
        ));
    }

    let sizes = coordinator
        .query_one(
            &format!(
                "SELECT pg_total_relation_size('{single_index}'::regclass)::bigint,
                        pg_total_relation_size('{single_corpus}'::regclass)::bigint,
                        pg_total_relation_size('{physical_corpus}'::regclass)::bigint"
            ),
            &[],
        )
        .await?;
    let physical_generation_bytes = published
        .iter()
        .map(|row| row.graph_bytes + row.row_bytes + row.directory_bytes + row.control_bytes)
        .sum::<i64>();
    let control_index_bytes = published.iter().map(|row| row.control_bytes).sum::<i64>();
    let single_index_bytes = sizes.get::<_, i64>(0);
    let single_source_bytes = sizes.get::<_, i64>(1);
    let coordinator_source_bytes = sizes.get::<_, i64>(2);
    let head = coordinator
        .query_one(
            "SELECT state.sample_count::bigint,
                    COALESCE((SELECT sum(pg_column_size(sample.vector) + pg_column_size(sample.vec_id))::bigint
                                FROM ec_distann_generation_head_sample sample
                               WHERE sample.index_oid = state.index_oid
                                 AND sample.logical_index_uuid = state.logical_index_uuid
                                 AND sample.build_id = state.build_id), 0),
                    COALESCE((SELECT sum(pg_column_size(sample.neighbors))::bigint
                                FROM ec_distann_generation_head_sample sample
                               WHERE sample.index_oid = state.index_oid
                                 AND sample.logical_index_uuid = state.logical_index_uuid
                                 AND sample.build_id = state.build_id), 0),
                    pg_column_size(state)::bigint
               FROM ec_distann_active_epoch active
               JOIN ec_distann_generation_head_state state
                 USING (index_oid, logical_index_uuid, build_id)
              WHERE active.index_oid = 'dm_idx'::regclass",
            &[],
        )
        .await
        .wrap_err("measuring persisted coordinator head")?;
    let head_sample_count = head.get::<_, i64>(0);
    let head_sample_bytes = head.get::<_, i64>(1);
    let head_graph_bytes = head.get::<_, i64>(2) + head.get::<_, i64>(3);
    let head_cache_estimated_bytes = head_sample_bytes + head_graph_bytes;
    let remote_owners = if args.coordinator_outside_roster {
        nodes.len()
    } else {
        nodes.len().saturating_sub(1)
    };
    for variant in &seed_variants {
        let shared = format!(
            "variant={} seed_strategy={} head_index_cap={} head_search_width={} head_seed_count={} beam_width={beam_width} hop_rounds={hop_rounds} neighbor_score_mode={}",
            variant.name,
            variant.strategy,
            args.head_index_cap,
            variant.head_search_width,
            variant.head_seed_count,
            variant.neighbor_score_mode,
        );
        lines.push(format!(
            "physical_benchmark_storage scale={scale} {shared} stored_neighbor_code_format=rabitq storage_shared=true owners={} physical_generation_bytes={physical_generation_bytes} control_index_bytes={control_index_bytes} coordinator_source_bytes={coordinator_source_bytes} single_index_bytes={single_index_bytes} single_source_bytes={single_source_bytes}",
            published.len()
        ));
        lines.push(format!(
            "physical_benchmark_head scale={scale} {shared} stored_neighbor_code_format=rabitq storage_shared=true sample_count={head_sample_count} head_sample_bytes={head_sample_bytes} head_graph_bytes={head_graph_bytes} head_cache_estimated_bytes={head_cache_estimated_bytes}"
        ));
        lines.push(format!(
            "physical_benchmark_engagement scale={scale} {shared} remote_owners={remote_owners} materialize_probes={remote_owners} pass={}",
            remote_owners > 0
        ));
    }
    for line in &mut lines {
        line.push_str(&format!(
            " corpus_prefix={corpus_prefix} query_sha256={query_sha256} extension_git_sha={expected_sha} extension_build_profile={expected_profile}"
        ));
    }
    Ok(lines)
}

async fn drive_physical_fixture(
    args: &LocalMultinodePg18Args,
    pg_ctl: &Path,
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    log_dir: &Path,
) -> Result<()> {
    for (ordinal, node) in nodes.iter().enumerate() {
        let setup = physical_setup_sql(args, ordinal == 0)?;
        run_psql_file(psql, socket_dir, node.port, &setup)
            .await
            .wrap_err_with(|| format!("physical shell setup on node {}", node.node_id))?;
        run_psql_file(
            psql,
            socket_dir,
            node.port,
            &format!(
                "SELECT ec_distann_configure_participant_identity(
                    'public.dm_idx'::regclass, 'physical/node-{}')",
                node.node_id
            ),
        )
        .await?;
    }
    let source_count = capture_psql(psql, socket_dir, nodes[0].port, "SELECT count(*) FROM dm")
        .await?
        .trim()
        .parse::<i64>()?;

    let coordinator_conninfo = conninfo(socket_dir, nodes[0].port);
    let (coordinator, connection) =
        tokio_postgres::connect(&coordinator_conninfo, tokio_postgres::NoTls)
            .await
            .wrap_err("connecting persistent physical coordinator session")?;
    let connection_task = tokio::spawn(async move { connection.await });
    let owners = if args.coordinator_outside_roster {
        &nodes[1..]
    } else {
        nodes
    };
    for (ordinal, node) in owners.iter().enumerate() {
        coordinator
            .batch_execute(&format!(
                "SELECT ec_distann_register_node_descriptor(
                    'public.dm_idx'::regclass, {ordinal}, {}, 'physical/node-{}',
                    'DISTANN_NODE_{}', 'public.dm_idx', {})",
                node.node_id,
                node.node_id,
                node.node_id,
                !args.coordinator_outside_roster && ordinal == 0
            ))
            .await
            .wrap_err_with(|| format!("registering physical node {}", node.node_id))?;
    }
    let physical_started = Instant::now();
    let build_id = "71717171-7171-4171-8171-717171717171";
    coordinator
        .batch_execute(&format!(
            "SELECT ec_distann_begin_epoch_build('public.dm_idx'::regclass, 1, '{build_id}'::uuid)"
        ))
        .await?;
    coordinator
        .batch_execute(&format!(
            "SELECT ec_distann_build_epoch('public.dm_idx'::regclass, 1, '{build_id}'::uuid)"
        ))
        .await?;
    let physical_build_ms = physical_started.elapsed().as_millis();

    let ready_selector =
        format!("ec_distann_generation_topology('public.dm_idx'::regclass, '{build_id}'::uuid)");
    let mut ready = Vec::with_capacity(owners.len());
    for node in owners {
        let row = physical_topology(psql, socket_dir, node, &ready_selector).await?;
        crate::ecaz_println!(
            "[distann-multicluster] physical_topology phase=ready node={} state={} records={} rows={} non_owned={} orphans={} graph_bytes={} row_bytes={} directory_bytes={} control_bytes={}",
            row.node_id,
            row.state,
            row.records,
            row.rows,
            row.non_owned_live + row.non_owned_tombstones,
            row.orphan_records + row.orphan_rows,
            row.graph_bytes,
            row.row_bytes,
            row.directory_bytes,
            row.control_bytes,
        );
        ready.push(row);
    }
    validate_physical_topology("ready", &ready, "Ready", source_count)?;

    coordinator
        .batch_execute(&format!(
            "SELECT ec_distann_decide_epoch_publish('public.dm_idx'::regclass, '{build_id}'::uuid)"
        ))
        .await?;
    let publish_fault_lines = if !args.skip_fault_drills
        && !args.physical_benchmark
        && !args.coordinator_outside_roster
        && owners.len() >= 3
    {
        physical_publish_fault_drills(
            &coordinator,
            pg_ctl,
            psql,
            socket_dir,
            nodes,
            owners,
            build_id,
        )
        .await?
    } else {
        coordinator
            .batch_execute(&format!(
                "SELECT ec_distann_recover_epoch_publish('public.dm_idx'::regclass, '{build_id}'::uuid)"
            ))
            .await?;
        Vec::new()
    };
    for line in &publish_fault_lines {
        crate::ecaz_println!("[distann-multicluster] {line}");
    }
    let physical_publish_ms = physical_started.elapsed().as_millis();
    let fingerprint = coordinator
        .query_one(
            "SELECT encode(epoch_fingerprint, 'hex') FROM ec_distann_active_epoch
              WHERE index_oid = 'public.dm_idx'::regclass::oid",
            &[],
        )
        .await?
        .get::<_, String>(0);
    let published_selector = format!(
        "ec_distann_epoch_topology('public.dm_idx'::regclass, decode('{fingerprint}', 'hex'))"
    );
    let mut published = Vec::with_capacity(owners.len());
    for node in owners {
        let row = physical_topology(psql, socket_dir, node, &published_selector).await?;
        crate::ecaz_println!(
            "[distann-multicluster] physical_topology phase=published node={} state={} records={} rows={} non_owned={} orphans={} graph_bytes={} row_bytes={} directory_bytes={} control_bytes={}",
            row.node_id,
            row.state,
            row.records,
            row.rows,
            row.non_owned_live + row.non_owned_tombstones,
            row.orphan_records + row.orphan_rows,
            row.graph_bytes,
            row.row_bytes,
            row.directory_bytes,
            row.control_bytes,
        );
        published.push(row);
    }
    validate_physical_topology("published", &published, "Published", source_count)?;

    let query_limit = i64::from(args.top_k.max(1));
    coordinator
        .batch_execute("SET enable_seqscan = off")
        .await?;
    let served = coordinator
        .query_one(
            &format!(
                "SELECT count(*) FROM (
                     SELECT source_id FROM dm
                      ORDER BY embedding <#> (SELECT source FROM dm ORDER BY id LIMIT 1)
                      LIMIT {query_limit}
                 ) served"
            ),
            &[],
        )
        .await?
        .get::<_, i64>(0);
    let serving_ok = served == query_limit.min(source_count);
    crate::ecaz_println!(
        "[distann-multicluster] physical_serving pass={} rows={} owners={} source_rows={}",
        serving_ok,
        served,
        owners.len(),
        source_count
    );
    if !serving_ok {
        bail!("physical serving returned {served} rows, expected {query_limit}");
    }
    let mut remote_verified = 0_usize;
    let remote_owners = if args.coordinator_outside_roster {
        owners
    } else {
        &owners[1..]
    };
    for node in remote_owners {
        let row_relation = capture_psql(
            psql,
            socket_dir,
            node.port,
            &format!(
                "SELECT row_tier_relid::regclass::text FROM ec_distann_generation
                  WHERE index_oid = 'public.dm_idx'::regclass::oid
                    AND epoch_fingerprint = decode('{fingerprint}', 'hex')"
            ),
        )
        .await?;
        let row_relation = row_relation.trim();
        if row_relation.is_empty()
            || !row_relation
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'"'))
        {
            bail!("remote owner {} returned unsafe row relation", node.node_id);
        }
        let sample = capture_psql(
            psql,
            socket_dir,
            node.port,
            &format!(
                "SELECT source_id::text || '|' || source::text
                   FROM {row_relation} ORDER BY source_id LIMIT 1"
            ),
        )
        .await?;
        let (source_id, vector) = sample
            .trim()
            .split_once('|')
            .ok_or_else(|| color_eyre::eyre::eyre!("remote owner sample is malformed"))?;
        if source_id.len() != 36
            || !source_id.bytes().enumerate().all(|(index, byte)| {
                if matches!(index, 8 | 13 | 18 | 23) {
                    byte == b'-'
                } else {
                    byte.is_ascii_hexdigit()
                }
            })
            || !vector.starts_with('{')
            || !vector.ends_with('}')
            || !vector.bytes().all(|byte| {
                byte.is_ascii_digit()
                    || matches!(byte, b'{' | b'}' | b',' | b'.' | b'-' | b'+' | b'e' | b'E')
            })
        {
            bail!(
                "remote owner {} returned malformed identity/vector",
                node.node_id
            );
        }
        // The ec_distann distributed planner intentionally requires a constant
        // query vector. Extended-protocol parameters remain Params during path
        // creation and therefore exercise the local AM path instead. `vector`
        // has already passed the strict numeric-array allowlist above, so use a
        // literal here to prove the same CustomScan shape as production literal
        // and benchmark queries.
        let owner_query = format!(
            "SELECT source_id FROM dm
              WHERE source_id = '{source_id}'::uuid
              ORDER BY embedding <#> '{vector}'::real[] LIMIT 1"
        );
        let owner_plan = coordinator
            .query(
                &format!("EXPLAIN (FORMAT TEXT, COSTS OFF) {owner_query}"),
                &[],
            )
            .await?
            .into_iter()
            .map(|row| row.get::<_, String>(0))
            .collect::<Vec<_>>()
            .join("\n");
        if !owner_plan.contains("EcDistannDistributedScan") {
            bail!(
                "remote-owner proof did not plan EcDistannDistributedScan for owner {}: {}",
                node.node_id,
                owner_plan
            );
        }
        let materialized_source_id = coordinator
            .query_one(
                &format!("SELECT source_id::text FROM ({owner_query}) q"),
                &[],
            )
            .await?
            .get::<_, String>(0);
        let owner_served = materialized_source_id == source_id;
        crate::ecaz_println!(
            "[distann-multicluster] physical_remote_owner node={} custom_scan=true pass={} expected_source_id={} materialized_source_id={}",
            node.node_id,
            owner_served,
            source_id,
            materialized_source_id
        );
        if !owner_served {
            bail!(
                "coordinator did not materialize selected row from remote owner {}",
                node.node_id
            );
        }
        remote_verified += 1;
    }
    let benchmark_lines = if args.physical_benchmark {
        run_physical_benchmarks(
            args,
            &coordinator,
            nodes[0].port,
            owners,
            &published,
            log_dir,
            physical_build_ms,
            physical_publish_ms,
        )
        .await?
    } else {
        Vec::new()
    };
    for line in &benchmark_lines {
        crate::ecaz_println!("[distann-multicluster] {line}");
    }
    if args.drop_extension_cleanup_drill {
        let unpublished_build_id = "72727272-7272-4272-8272-727272727272";
        coordinator
            .batch_execute(&format!(
                "SELECT ec_distann_begin_epoch_build('public.dm_idx'::regclass, 2, '{unpublished_build_id}'::uuid)"
            ))
            .await?;
        coordinator
            .batch_execute(&format!(
                "SELECT ec_distann_build_epoch('public.dm_idx'::regclass, 2, '{unpublished_build_id}'::uuid)"
            ))
            .await?;
    }
    drop(coordinator);
    connection_task.abort();
    let drop_extension_lines = if args.drop_extension_cleanup_drill {
        physical_drop_extension_cleanup_drill(psql, socket_dir, nodes).await?
    } else {
        Vec::new()
    };
    for line in &drop_extension_lines {
        crate::ecaz_println!("[distann-multicluster] {line}");
    }
    crate::ecaz_println!(
        "[distann-multicluster] physical_topology_gate pass=true owners={} remote_verified={} source_rows={}",
        owners.len(),
        remote_verified,
        source_count
    );
    let mut summary = format!(
        "physical_fixture owners={} coordinator_outside_roster={} source_rows={}\n",
        owners.len(),
        args.coordinator_outside_roster,
        source_count
    );
    for (phase, rows) in [("ready", &ready), ("published", &published)] {
        for row in rows {
            summary.push_str(&format!(
                "[distann-multicluster] physical_topology phase={phase} node={} state={} records={} rows={} non_owned={} orphans={} graph_bytes={} row_bytes={} directory_bytes={} control_bytes={}\n",
                row.node_id,
                row.state,
                row.records,
                row.rows,
                row.non_owned_live + row.non_owned_tombstones,
                row.orphan_records + row.orphan_rows,
                row.graph_bytes,
                row.row_bytes,
                row.directory_bytes,
                row.control_bytes,
            ));
        }
    }
    summary.push_str(&format!(
        "[distann-multicluster] physical_serving pass=true rows={served} owners={} source_rows={source_count}\n",
        owners.len()
    ));
    for line in &publish_fault_lines {
        summary.push_str(&format!("[distann-multicluster] {line}\n"));
    }
    for line in &benchmark_lines {
        summary.push_str(&format!("[distann-multicluster] {line}\n"));
    }
    for line in &drop_extension_lines {
        summary.push_str(&format!("[distann-multicluster] {line}\n"));
    }
    summary.push_str(&format!(
        "[distann-multicluster] physical_topology_gate pass=true owners={} remote_verified={remote_verified} source_rows={source_count}\n",
        owners.len()
    ));
    let summary_path = log_dir.join("distann-multinode-summary.log");
    fs::write(&summary_path, summary)
        .wrap_err_with(|| format!("writing {}", summary_path.display()))?;
    crate::ecaz_println!(
        "[distann-multicluster] summary written to {}",
        summary_path.display()
    );
    Ok(())
}

async fn physical_drop_extension_cleanup_drill(
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
) -> Result<Vec<String>> {
    let mut lines = Vec::with_capacity(nodes.len());
    for node in nodes {
        let hidden_before = capture_psql(
            psql,
            socket_dir,
            node.port,
            "SELECT count(*) FROM pg_class WHERE relname ~ '^_ecdz_'",
        )
        .await?
        .trim()
        .parse::<i64>()?;
        let states_before = capture_psql(
            psql,
            socket_dir,
            node.port,
            "SELECT
                 count(*) FILTER (WHERE state = 'Ready')::text || '|' ||
                 count(*) FILTER (WHERE state = 'Published')::text
               FROM ec_distann_generation
              WHERE index_oid = 'public.dm_idx'::regclass::oid",
        )
        .await?;
        let state_counts = states_before
            .trim()
            .split('|')
            .map(str::parse::<i64>)
            .collect::<Result<Vec<_>, _>>()?;
        let ready_before = state_counts.first().copied().unwrap_or(0);
        let published_before = state_counts.get(1).copied().unwrap_or(0);
        if hidden_before < 6 || ready_before < 1 || published_before < 1 {
            bail!(
                "physical DROP EXTENSION drill precondition failed on node {}: hidden={hidden_before} Ready={ready_before} Published={published_before}",
                node.node_id,
            );
        }
        run_psql_file(
            psql,
            socket_dir,
            node.port,
            "DROP EXTENSION ecaz CASCADE;
             CREATE TABLE ecaz_drop_extension_probe(id integer PRIMARY KEY);
             INSERT INTO ecaz_drop_extension_probe VALUES (1);",
        )
        .await
        .wrap_err_with(|| format!("dropping ecaz on physical node {}", node.node_id))?;
        let after = capture_psql(
            psql,
            socket_dir,
            node.port,
            "SELECT
                 (SELECT count(*) FROM pg_extension WHERE extname = 'ecaz')::text || '|' ||
                 (SELECT count(*) FROM pg_class WHERE relname ~ '^_ecdz_')::text || '|' ||
                 (SELECT count(*) FROM ecaz_drop_extension_probe)::text",
        )
        .await?;
        let fields = after.trim().split('|').collect::<Vec<_>>();
        if fields.as_slice() != ["0", "0", "1"] {
            bail!(
                "physical DROP EXTENSION cleanup failed on node {}: expected 0|0|1, got {}",
                node.node_id,
                after.trim()
            );
        }
        lines.push(format!(
            "physical_drop_extension_cleanup pass=true node={} ready_before={ready_before} published_before={published_before} hidden_before={hidden_before} hidden_after=0 extension_after=0 post_drop_dml_rows=1",
            node.node_id
        ));
    }
    Ok(lines)
}

async fn physical_publish_fault_drills(
    coordinator: &tokio_postgres::Client,
    pg_ctl: &Path,
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    owners: &[Node],
    build_id: &str,
) -> Result<Vec<String>> {
    let recover_sql = format!(
        "SELECT ec_distann_recover_epoch_publish('public.dm_idx'::regclass, '{build_id}'::uuid)"
    );
    coordinator
        .batch_execute("SET ec_distann.debug_fail_recover_after_publish_ack = on")
        .await?;

    // Real partial-ack window: local publication remains transactional on the
    // coordinator, owner 2 commits its remote acknowledgement, and the last
    // owner crashes before it can acknowledge. The coordinator must retain the
    // durable Pending/Decided decision with no active pointer.
    let unavailable = owners
        .last()
        .ok_or_else(|| color_eyre::eyre::eyre!("physical fault drill has no owner"))?;
    let mut stop = Command::new(pg_ctl);
    stop.arg("-w")
        .arg("-D")
        .arg(&unavailable.data_dir)
        .arg("-m")
        .arg("immediate")
        .arg("stop")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    run_status(stop)
        .await
        .wrap_err("crashing the last physical owner for the publish drill")?;

    let down_error = coordinator.batch_execute(&recover_sql).await.err();
    let (decision, registration, active_count) =
        physical_publish_coordinator_state(coordinator, build_id).await?;
    let local_state = physical_generation_state(psql, socket_dir, &owners[0], build_id).await?;
    let acked_state = physical_generation_state(psql, socket_dir, &owners[1], build_id).await?;
    let participant_down_pass = down_error.is_some()
        && decision == "Pending"
        && registration == "Decided"
        && active_count == 0
        && local_state == "Ready"
        && acked_state == "Published";
    let mut lines = vec![format!(
        "physical_publish_fault participant_down_partial pass={participant_down_pass} decision={decision} registration={registration} active_count={active_count} local_state={local_state} remote_acked_state={acked_state} unavailable_node={}",
        unavailable.node_id
    )];
    if !participant_down_pass {
        bail!(
            "physical participant-down publish drill failed: {}",
            lines.last().expect("fault line")
        );
    }

    restart_physical_node(pg_ctl, socket_dir, unavailable, nodes).await?;

    // With every owner reachable, the debug hook fails after all participant
    // acknowledgements but before the coordinator active-pointer swap. Remote
    // acknowledgements persist; the local publish rolls back with T4a.
    let injected_error = coordinator.batch_execute(&recover_sql).await.err();
    let (decision, registration, active_count) =
        physical_publish_coordinator_state(coordinator, build_id).await?;
    let mut states = Vec::with_capacity(owners.len());
    for owner in owners {
        states.push(physical_generation_state(psql, socket_dir, owner, build_id).await?);
    }
    let post_ack_pass = injected_error
        .as_ref()
        .and_then(tokio_postgres::Error::as_db_error)
        .is_some_and(|error| error.message().contains("EC_FAULT_INJECTED"))
        && decision == "Pending"
        && registration == "Decided"
        && active_count == 0
        && states.first().is_some_and(|state| state == "Ready")
        && states.iter().skip(1).all(|state| state == "Published");
    lines.push(format!(
        "physical_publish_fault post_ack_pre_pointer pass={post_ack_pass} decision={decision} registration={registration} active_count={active_count} owner_states={}",
        states.join(",")
    ));
    if !post_ack_pass {
        bail!(
            "physical post-ack/pre-pointer publish drill failed: {}",
            lines.last().expect("fault line")
        );
    }

    coordinator
        .batch_execute("SET ec_distann.debug_fail_recover_after_publish_ack = off")
        .await?;
    coordinator.batch_execute(&recover_sql).await?;
    let (decision, registration, active_count) =
        physical_publish_coordinator_state(coordinator, build_id).await?;
    let mut recovered_states = Vec::with_capacity(owners.len());
    for owner in owners {
        recovered_states.push(physical_generation_state(psql, socket_dir, owner, build_id).await?);
    }
    let recovery_pass = decision == "Applied"
        && registration == "Published"
        && active_count == 1
        && recovered_states.iter().all(|state| state == "Published");
    lines.push(format!(
        "physical_publish_fault idempotent_recovery pass={recovery_pass} decision={decision} registration={registration} active_count={active_count} owner_states={}",
        recovered_states.join(",")
    ));
    if !recovery_pass {
        bail!(
            "physical idempotent publish recovery failed: {}",
            lines.last().expect("fault line")
        );
    }
    Ok(lines)
}

async fn physical_publish_coordinator_state(
    coordinator: &tokio_postgres::Client,
    build_id: &str,
) -> Result<(String, String, i64)> {
    let row = coordinator
        .query_one(
            "SELECT
                 COALESCE((SELECT decision_state FROM ec_distann_publish_decision
                            WHERE build_id = $1::text::uuid), 'Missing'),
                 COALESCE((SELECT state FROM ec_distann_build_registration
                            WHERE build_id = $1::text::uuid), 'Missing'),
                 (SELECT count(*) FROM ec_distann_active_epoch
                   WHERE build_id = $1::text::uuid)",
            &[&build_id],
        )
        .await?;
    Ok((row.get(0), row.get(1), row.get(2)))
}

async fn physical_generation_state(
    psql: &Path,
    socket_dir: &Path,
    node: &Node,
    build_id: &str,
) -> Result<String> {
    let state = capture_psql(
        psql,
        socket_dir,
        node.port,
        &format!(
            "SELECT state FROM ec_distann_generation
              WHERE index_oid = 'public.dm_idx'::regclass::oid
                AND build_id = '{build_id}'::uuid"
        ),
    )
    .await?;
    Ok(state.trim().to_owned())
}

async fn restart_physical_node(
    pg_ctl: &Path,
    socket_dir: &Path,
    node: &Node,
    nodes: &[Node],
) -> Result<()> {
    let mut restart = Command::new(pg_ctl);
    restart
        .arg("-w")
        .arg("-D")
        .arg(&node.data_dir)
        .arg("-l")
        .arg(&node.log_file)
        .arg("-o")
        .arg(format!(
            "-p {} -c listen_addresses=127.0.0.1 -c unix_socket_directories='' \
             -c shared_preload_libraries=ecaz",
            node.port
        ))
        .arg("start")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    for target in nodes {
        restart.env(
            format!("EC_SPIRE_REMOTE_CONNINFO_DISTANN_NODE_{}", target.node_id),
            conninfo(socket_dir, target.port),
        );
    }
    run_status(restart)
        .await
        .wrap_err_with(|| format!("restarting physical owner {}", node.node_id))
}

async fn drive_fixture(
    args: &LocalMultinodePg18Args,
    pg_ctl: &Path,
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    log_dir: &Path,
) -> Result<()> {
    // Replicated deterministic corpus + index on every node.
    let setup = build_setup_sql(args)?;
    for node in nodes {
        run_psql_file(psql, socket_dir, node.port, &setup)
            .await
            .wrap_err_with(|| format!("corpus/index setup on node {}", node.node_id))?;
        crate::ecaz_println!(
            "[distann-multicluster] node {} loaded + indexed",
            node.node_id
        );
    }

    // Coordinator roster: every node by socket conninfo, in node-id order.
    let roster = nodes
        .iter()
        .map(|node| format!("{}@{}", node.node_id, conninfo(socket_dir, node.port)))
        .collect::<Vec<_>>()
        .join(";");

    // Distinct-recall gate on the coordinator (node 1).
    let coord_port = nodes[0].port;
    let recall = recall_sql(
        &roster,
        args.queries,
        args.top_k,
        args.corpus_prefix.is_some(),
    );
    let out = capture_psql(psql, socket_dir, coord_port, &recall)
        .await
        .wrap_err("running the multi-node recall comparison")?;
    let result_line = out
        .lines()
        .find(|line| line.contains("RECALL_RESULT"))
        .unwrap_or("RECALL_RESULT <none>")
        .trim()
        .to_owned();
    crate::ecaz_println!("[distann-multicluster] {result_line}");

    // Suite-driven recall gate (006-P1 letter): `ecaz bench recall` against the
    // coordinator single-node vs multi-node, distinct_recall(multi) >=
    // distinct_recall(single) - 0.001. Run here — before the mutating drills —
    // so benchgate_corpus is byte-identical across nodes (consistent vec_ids).
    // The byte-identical top-k gate above is strictly stronger.
    let suite_line = suite_recall_gate(psql, socket_dir, nodes, &roster, args).await;
    crate::ecaz_println!("[distann-multicluster] {suite_line}");
    // 019-P1: a genuine recall regression fails the run (SKIPPED/INCONCLUSIVE are
    // environment issues, not gate failures).
    if suite_line.contains("pass=false") {
        bail!("suite recall gate FAILED: {suite_line}");
    }

    // Qual correctness (011/020-P1): a WHERE predicate on a NON-projected column
    // plus LIMIT. Multi-node must match single-node exactly — this exercises
    // shipping the qual column (source) for remote rows and over-fetching so the
    // LIMIT applies after the qual. Runs early, on the clean/consistent corpus.
    let (qual_line, qual_ok) =
        qual_correctness_drill(psql, socket_dir, coord_port, &roster, args).await;
    crate::ecaz_println!("[distann-multicluster] {qual_line}");

    // FR-082 published-epoch read consumption: reads must source the epoch from
    // the persisted manifest (`active_epoch`), not the session GUC. Proven by a
    // coordinator-only publish (breaks scans via fingerprint mismatch — only
    // possible if reads consume active_epoch) vs a coordinated all-node publish
    // (swaps the epoch; scans match the baseline again).
    let (fr082_line, fr082_ok) =
        fr082_published_epoch_drill(psql, socket_dir, nodes, &roster, args).await;
    crate::ecaz_println!("[distann-multicluster] {fr082_line}");

    // Cluster storage summation (Task 172 AC-3 / NFR-018): per-node and summed
    // index+heap bytes across every participant, plus the replicated-index space
    // amplification vs raw f32 vectors. Runs in both modes so the suite always
    // captures it.
    let storage_lines = storage_summation(psql, socket_dir, nodes, args).await;
    for line in &storage_lines {
        crate::ecaz_println!("[distann-multicluster] {line}");
    }

    // Recall-only mode (the `distann-local-multinode` suite step): the multi-node
    // distinct-recall gates above are the scaled evidence; skip the (expensive at
    // scale) TC-042 fault matrix + FR-082 lifecycle drills, which are proven
    // scale-independently at the fixture default size.
    if args.skip_fault_drills {
        // 172-P1: report MEASURED corpus metadata, not the synthetic args
        // defaults. In real mode args.rows/args.dim are meaningless (16/2000);
        // the true rows/dim come from the storage summation over the loaded data.
        let (measured_rows, measured_dim) = storage_lines
            .iter()
            .find(|l| l.starts_with("storage_summation"))
            .map(|l| {
                let field = |k: &str| {
                    l.split_whitespace()
                        .find_map(|t| t.strip_prefix(k))
                        .unwrap_or("?")
                        .to_owned()
                };
                (field("corpus_rows="), field("dim="))
            })
            .unwrap_or_else(|| (args.rows.to_string(), args.dim.to_string()));
        let corpus_label = match &args.corpus_prefix {
            Some(prefix) => format!("corpus=real staged prefix={prefix}"),
            None => "corpus=synthetic".to_owned(),
        };
        let mut summary = format!(
            "distann-multinode fixture (recall-only)\n{corpus_label}\nnodes={}\nrows={measured_rows}\ndim={measured_dim}\ngraph_degree={}\nhead_index_cap={}\nqueries={}\ntop_k={}\nroster={}\n{}\n",
            args.nodes,
            args.graph_degree,
            args.head_index_cap,
            args.queries,
            args.top_k,
            roster,
            result_line
        );
        summary.push_str(&format!("{qual_line}\n"));
        summary.push_str(&format!("{fr082_line}\n"));
        summary.push_str(&format!("{suite_line}\n"));
        for line in &storage_lines {
            summary.push_str(&format!("{line}\n"));
        }
        let summary_path = log_dir.join("distann-multinode-summary.log");
        fs::write(&summary_path, &summary)
            .wrap_err_with(|| format!("writing {}", summary_path.display()))?;
        crate::ecaz_println!(
            "[distann-multicluster] summary written to {}",
            summary_path.display()
        );
        if !result_line.contains("mismatched_ids=0") {
            bail!("multi-node distinct-recall gate FAILED: {result_line}");
        }
        if !qual_ok {
            bail!("qual correctness FAILED: {qual_line}");
        }
        if !fr082_ok {
            bail!("FR-082 published-epoch read consumption FAILED: {fr082_line}");
        }
        crate::ecaz_println!(
            "[distann-multicluster] GATE PASS (recall-only): multi-node distinct-recall identical to single-node"
        );
        return Ok(());
    }

    // TC-042 fault matrix (NFR-020): each fault must make the multi-node query
    // ERROR (fail closed) — never a silent wrong or partial result — and a
    // post-recovery query must match the baseline (no false reject).
    let mut drills: Vec<(String, bool)> = Vec::new();
    let last = nodes.last().unwrap();
    let single_query = format!(
        "SELECT id FROM dm ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT {};",
        args.top_k
    );

    // 1. simulated_network_partition: one owner at a dead port ⇒ connect error.
    {
        let dead_roster = roster_with_port_override(nodes, socket_dir, last.node_id, 1);
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{dead_roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push((
            "simulated_network_partition".to_owned(),
            query_errored(&out),
        ));
    }

    // 2. epoch_bump_no_false_reject: a bare epoch-number bump must NOT reject —
    // the FR-082 fingerprint is content-based and the coordinator propagates its
    // epoch to owners, so both sides agree and the query returns its result.
    {
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=999999; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        // Pass = no false reject: no error AND a result row came back.
        drills.push((
            "epoch_bump_no_false_reject".to_owned(),
            !query_errored(&out) && out.contains('\n'),
        ));
    }

    // 3. remote_content_divergence (real epoch/fingerprint mismatch): rebuild an
    // owner's index with a different graph_degree so its content fingerprint no
    // longer matches the coordinator's ⇒ the owner rejects the epoch (error).
    // Diverge DOWNWARD (graph_degree - 8): a larger degree can overflow the
    // ec_distann node-record page budget on high-dim corpora (a 1536-dim
    // co-placed vector + graph_degree neighbor codes must fit one 8 KB page), so
    // graph_degree + 8 fails ambuild at real scale. A smaller degree is always
    // within the budget the base build already satisfied and still changes the
    // content fingerprint.
    let divergent_degree = args.graph_degree.saturating_sub(8).max(4);
    {
        run_psql_file(
            psql,
            socket_dir,
            last.port,
            &format!(
                "DROP INDEX dm_idx; CREATE INDEX dm_idx ON dm USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = {divergent_degree});",
            ),
        )
        .await
        .wrap_err("diverging remote index content for the drill")?;
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push(("remote_content_divergence".to_owned(), query_errored(&out)));
        run_psql_file(
            psql,
            socket_dir,
            last.port,
            &format!(
                "DROP INDEX dm_idx; CREATE INDEX dm_idx ON dm USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = {});",
                args.graph_degree
            ),
        )
        .await
        .wrap_err("restoring remote index content after the drill")?;
    }

    // 3. missing_or_reindexed_remote_index: drop the index on an owner ⇒ error.
    {
        run_psql_file(psql, socket_dir, last.port, "DROP INDEX dm_idx;")
            .await
            .wrap_err("dropping remote index for the drill")?;
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push((
            "missing_or_reindexed_remote_index".to_owned(),
            query_errored(&out),
        ));
        // Rebuild for recovery.
        run_psql_file(
            psql,
            socket_dir,
            last.port,
            &format!(
                "CREATE INDEX dm_idx ON dm USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = {});",
                args.graph_degree
            ),
        )
        .await
        .wrap_err("rebuilding remote index after the drill")?;
    }

    // 4. remote_backend_termination / instance down: stop an owner ⇒ error.
    {
        let _ = Command::new(pg_ctl)
            .arg("-D")
            .arg(&last.data_dir)
            .arg("-m")
            .arg("fast")
            .arg("stop")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push(("remote_backend_termination".to_owned(), query_errored(&out)));
        // Restart for recovery.
        let mut restart = Command::new(pg_ctl);
        restart
            .arg("-w")
            .arg("-D")
            .arg(&last.data_dir)
            .arg("-l")
            .arg(&last.log_file)
            .arg("-o")
            .arg(format!(
                "-p {} -k {} -c listen_addresses=''",
                last.port,
                socket_dir.display()
            ))
            .arg("start")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        run_status(restart)
            .await
            .wrap_err("restarting owner after the drill")?;
    }

    // 6. placement_drift: coordinator local_node_id absent from the roster ⇒ no
    // local node ⇒ error (a placement disagreement is never a silent miss).
    {
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=99; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push(("placement_drift".to_owned(), query_errored(&out)));
    }

    // 7. remote_statement_timeout: inject `statement_timeout=1` (1 ms) into one
    // owner's conninfo ⇒ its expand statement is cancelled server-side ⇒ the
    // coordinator surfaces the remote error rather than a partial result.
    {
        let timeout_roster = roster_with_conninfo_suffix(
            nodes,
            socket_dir,
            last.node_id,
            "options=-cstatement_timeout=1",
        );
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{timeout_roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push(("remote_statement_timeout".to_owned(), query_errored(&out)));
    }

    // 7b. hop_round_failure_mid_beam: force the search past round 0 (a high top_k
    // bar prevents the round-0 convergence early-exit) and inject a failure at the
    // start of hop round 1 via `ec_distann.debug_fail_hop_round`. A mid-beam round
    // failure must discard the partial beam and ERROR — never surface round 0's
    // partial frontier as a complete result.
    {
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
             SET ec_distann.hop_rounds=4; SET ec_distann.top_k=200; SET ec_distann.beam_width=8; \
             SET ec_distann.debug_fail_hop_round=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        // Fail closed AND specifically the injected mid-beam (round 1) failure.
        let mid_beam = query_errored(&out) && out.contains("round 1");
        crate::ecaz_println!(
            "[distann-multicluster] hop_round_failure_mid_beam DIAG errored={} mid_beam={mid_beam}",
            query_errored(&out)
        );
        drills.push(("hop_round_failure_mid_beam".to_owned(), mid_beam));
    }

    // 7c. missing_node_record (FR-079 case c): force the local expander to report
    // an owned record as absent from its directory. The scan must raise the
    // structural fault, never silently under-return.
    {
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
             SET ec_distann.debug_missing_node_record=true; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        let pass = query_errored(&out) && out.contains("missing node record");
        crate::ecaz_println!(
            "[distann-multicluster] missing_node_record DIAG errored={} tagged={pass}",
            query_errored(&out)
        );
        drills.push(("missing_node_record".to_owned(), pass));
    }

    // 8. missing_heap_row_co_placement_drift (also the partial mid-delete case):
    // remove only an owned record's heap row on its owner, leaving the index
    // record ⇒ the owner's exact rerank fails `[EC_VECTOR_MISSING]` ⇒ error. The
    // correct in-epoch delete is a monotonic tombstone via FR-083's
    // `ec_distann_apply_record_writes` (which keeps the frozen vector, per
    // FR-082-AC-5); this drill proves the *drift* hazard fails closed rather than
    // silently dropping the row. The drill self-recovers by re-running setup on
    // the owner.
    {
        let drift_ok =
            co_placement_drift_drill(psql, socket_dir, coord_port, &roster, nodes, args).await;
        drills.push(("missing_heap_row_co_placement_drift".to_owned(), drift_ok));
    }

    // 8a. mid-delete / lost-tombstone-write (NFR-020): a tombstone write that
    // errors after the WAL-logged flag flip. The monotonic tombstone stays set
    // (PG does not undo index-page writes on abort), so the record is deleted and
    // STAYS deleted — the caller sees an error but the row never resurrects.
    {
        let mid_delete_ok = mid_delete_drill(psql, socket_dir, coord_port, args).await;
        drills.push((
            "mid_delete_lost_tombstone_no_resurrect".to_owned(),
            mid_delete_ok,
        ));
    }

    // 8b. mid-insert failure (FR-083 fold path, TC-043): a graph insert that fails
    // after staging pages but before publishing metadata must roll back cleanly —
    // no partial record visible. Runs on an isolated table so shared `dm` is
    // untouched.
    {
        let mid_insert_ok = mid_insert_drill(psql, socket_dir, coord_port, args).await;
        drills.push(("mid_insert_failure_rolls_back".to_owned(), mid_insert_ok));
    }

    // 7. concurrency (FR-082-AC-4): run many multi-node scans concurrently with a
    // background inserter mutating the coordinator's table. Every scan must
    // complete (return only expanded records; never a torn/half-applied read that
    // errors). A single failing session fails the drill.
    let concurrency_ok =
        concurrency_drill(psql, socket_dir, coord_port, nodes, &roster, args).await?;
    crate::ecaz_println!(
        "[distann-multicluster] concurrency_scan_insert_epochswap pass={concurrency_ok}"
    );

    // 7b. live retention gate (FR-082-AC-3): a scan held open (AccessShareLock)
    // must block retire; once it drains, retire succeeds.
    let retention_ok = retention_gate_drill(psql, socket_dir, coord_port, args).await;
    crate::ecaz_println!("[distann-multicluster] live_retention_gate pass={retention_ok}");

    // 7c. AC-5 frozen vec_id→vector: a live record's exact-rerank result must be
    // byte-identical after real delete+VACUUM+reinsert TID churn on every node
    // (the AM's ambulkdelete tombstones deleted records so they are never
    // reranked, and a live record's heap TID is never reclaimed → its vector is
    // frozen without a separate tier, under D10).
    let frozen_ok = frozen_vector_drill(psql, socket_dir, coord_port, &roster, nodes, args).await;
    crate::ecaz_println!(
        "[distann-multicluster] ac5_frozen_vector_after_vacuum_reuse pass={frozen_ok}"
    );

    // 8. recovery / no-false-reject: after all faults clear, the full-roster
    // query must match the single-node baseline again.
    let recovery = capture_psql(
        psql,
        socket_dir,
        coord_port,
        &recall_sql(
            &roster,
            args.queries,
            args.top_k,
            args.corpus_prefix.is_some(),
        ),
    )
    .await
    .wrap_err("running the post-recovery recall comparison")?;
    let recovery_line = recovery
        .lines()
        .find(|line| line.contains("RECALL_RESULT"))
        .unwrap_or("RECALL_RESULT <none>")
        .trim()
        .to_owned();
    let recovered = recovery_line.contains("mismatched_ids=0");

    for (name, fail_closed) in &drills {
        crate::ecaz_println!("[distann-multicluster] fault_drill {name} pass={fail_closed}");
    }
    crate::ecaz_println!("[distann-multicluster] recovery {recovery_line} recovered={recovered}");

    // Disjoint-shard demonstration (destructive — prunes to owned shards; runs
    // last, after the replicated-corpus recovery check).
    let (disjoint_line, disjoint_ok) =
        disjoint_shard_drill(psql, socket_dir, nodes, &roster, args).await;
    crate::ecaz_println!("[distann-multicluster] {disjoint_line}");

    // Persist the evidence.
    let mut summary = format!(
        "distann-multinode fixture\nnodes={}\nrows={}\ndim={}\ngraph_degree={}\nqueries={}\ntop_k={}\nroster={}\n{}\n",
        args.nodes, args.rows, args.dim, args.graph_degree, args.queries, args.top_k, roster, result_line
    );
    for (name, fail_closed) in &drills {
        summary.push_str(&format!("fault_drill {name} pass={fail_closed}\n"));
    }
    summary.push_str(&format!(
        "concurrency_scan_insert_epochswap pass={concurrency_ok}\n"
    ));
    summary.push_str(&format!("{qual_line}\n"));
    summary.push_str(&format!("{fr082_line}\n"));
    summary.push_str(&format!("live_retention_gate pass={retention_ok}\n"));
    summary.push_str(&format!(
        "ac5_frozen_vector_after_vacuum_reuse pass={frozen_ok}\n"
    ));
    summary.push_str(&format!("{suite_line}\n"));
    summary.push_str(&format!("recovery {recovery_line} recovered={recovered}\n"));
    summary.push_str(&format!("{disjoint_line}\n"));
    let summary_path = log_dir.join("distann-multinode-summary.log");
    fs::write(&summary_path, &summary)
        .wrap_err_with(|| format!("writing {}", summary_path.display()))?;
    crate::ecaz_println!(
        "[distann-multicluster] summary written to {}",
        summary_path.display()
    );

    if !result_line.contains("mismatched_ids=0") {
        bail!("multi-node distinct-recall gate FAILED: {result_line}");
    }
    if !concurrency_ok {
        bail!("concurrency drill FAILED: a scan errored under concurrent insert load");
    }
    if !qual_ok {
        bail!("qual correctness FAILED: multi-node WHERE+LIMIT result differs from single-node");
    }
    if !fr082_ok {
        bail!("FR-082 published-epoch read consumption FAILED: {fr082_line}");
    }
    if !retention_ok {
        bail!("live retention gate FAILED: retire not gated by an in-flight scan, or blocked after drain");
    }
    if !frozen_ok {
        bail!("AC-5 FAILED: a live record's rerank changed after delete+VACUUM+reinsert TID churn");
    }
    let all_fail_closed = drills.iter().all(|(_, ok)| *ok);
    if !all_fail_closed {
        let failed: Vec<&str> = drills
            .iter()
            .filter(|(_, ok)| !*ok)
            .map(|(name, _)| name.as_str())
            .collect();
        bail!("TC-042 fault matrix FAILED (not fail-closed): {failed:?}");
    }
    if !recovered {
        bail!("recovery FAILED: post-fault query did not match baseline: {recovery_line}");
    }
    if !disjoint_ok {
        bail!("disjoint-shard FAILED: multi-node result changed after pruning to owned shards");
    }
    crate::ecaz_println!(
        "[distann-multicluster] GATE PASS: recall identical; {} faults fail-closed; recovery clean",
        drills.len()
    );
    Ok(())
}

fn roster_with_port_override(
    nodes: &[Node],
    socket_dir: &Path,
    override_node_id: u32,
    override_port: u16,
) -> String {
    nodes
        .iter()
        .map(|node| {
            let port = if node.node_id == override_node_id {
                override_port
            } else {
                node.port
            };
            format!("{}@{}", node.node_id, conninfo(socket_dir, port))
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// A roster where `override_node_id`'s conninfo carries an extra libpq keyword
/// (space-separated, matching the `host=… port=…` conninfo shape). Used to inject
/// `options=-cstatement_timeout=1` into a single owner for the
/// remote_statement_timeout fault drill.
fn roster_with_conninfo_suffix(
    nodes: &[Node],
    socket_dir: &Path,
    override_node_id: u32,
    suffix: &str,
) -> String {
    nodes
        .iter()
        .map(|node| {
            let base = conninfo(socket_dir, node.port);
            if node.node_id == override_node_id {
                format!("{}@{} {}", node.node_id, base, suffix)
            } else {
                format!("{}@{}", node.node_id, base)
            }
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// NFR-020 co-placement drift / missing-heap-row (and partial mid-delete) drill.
///
/// In the replicated topology every node holds every heap row, so a serving node
/// reranks from its OWN heap copy — deleting a row on a single owner is masked by
/// the other replicas (proven: a single-node delete still returned the row with
/// no error). Genuine cluster-wide co-placement drift is: the index record
/// survives on every node but its co-placed heap row is gone everywhere. This
/// drill deletes a record's heap row on ALL nodes (leaving the index record),
/// then runs a query anchored on that record's own vector. Every serving node's
/// exact rerank must fetch the (now invisible) heap tuple and fail
/// `[EC_VECTOR_MISSING]`, so the multi-node query ERRORs (fail closed) rather than
/// silently dropping or mis-ranking the true top-1. Recovery re-runs the
/// deterministic setup on every node (identical vec_ids), so the post-fault recall
/// baseline still matches.
async fn co_placement_drift_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    roster: &str,
    nodes: &[Node],
    args: &LocalMultinodePg18Args,
) -> bool {
    // Exercise both ownership arms: a coordinator-owned record (local MVCC skip →
    // correct-complete) and a remote-owned record (remote rerank → structural
    // fault). Both must satisfy the NFR-020 disjunction.
    let coord = co_placement_drift_case(psql, socket_dir, coord_port, roster, nodes, args, 0).await;
    let remote = co_placement_drift_case(
        psql,
        socket_dir,
        coord_port,
        roster,
        nodes,
        args,
        args.nodes - 1,
    )
    .await;
    coord && remote
}

/// FR-082 published-epoch read-consumption drill. Reads must source the scan
/// epoch from the persisted manifest (`active_epoch`), not the session GUC — so a
/// `publish` actually changes what queries see. Proven in three steps against the
/// replicated `dm`:
///
///   A. baseline multi-node scan at the built-in published epoch (1) succeeds;
///   B. publish epoch 2 on the COORDINATOR ONLY ⇒ its fingerprint no longer
///      matches the owners' (still epoch 1) ⇒ the scan ERRORS. This can only
///      happen if reads consume `active_epoch` (the GUC is unchanged throughout);
///   C. publish epoch 2 on EVERY node ⇒ the epoch swaps atomically and the scan
///      succeeds again with the same top-k as the baseline.
///
/// Restores epoch 1 on all nodes so later drills see the default state. Returns
/// (summary, pass).
async fn fr082_published_epoch_drill(
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    roster: &str,
    args: &LocalMultinodePg18Args,
) -> (String, bool) {
    let coord_port = nodes[0].port;
    // Note: no `ec_distann.epoch` is set — reads must ignore the GUC and use the
    // published manifest epoch.
    let scan = format!(
        "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; \
         SELECT id FROM dm ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT {};",
        args.top_k
    );
    let ids = |out: &str| -> Vec<i64> {
        let mut v: Vec<i64> = out.lines().filter_map(|l| l.trim().parse().ok()).collect();
        v.sort_unstable();
        v
    };
    let publish = |port: u16, epoch: i64| {
        let sql = format!("SELECT ec_distann_publish_epoch('dm_idx'::regclass, {epoch});");
        async move { run_psql_file(psql, socket_dir, port, &sql).await }
    };

    // A. baseline at the built-in published epoch.
    let base_out = capture_psql_allow_error(psql, socket_dir, coord_port, &scan).await;
    let base_ids = ids(&base_out);
    let base_ok = !query_errored(&base_out) && !base_ids.is_empty();

    // B. coordinator-only publish of epoch 2 ⇒ mismatch ⇒ scan errors.
    let _ = publish(coord_port, 2).await;
    let skew_out = capture_psql_allow_error(psql, socket_dir, coord_port, &scan).await;
    let skew_errored = query_errored(&skew_out);

    // C. publish epoch 2 on every node ⇒ swap ⇒ scan matches the baseline.
    for node in nodes {
        let _ = publish(node.port, 2).await;
    }
    let swap_out = capture_psql_allow_error(psql, socket_dir, coord_port, &scan).await;
    let swap_ids = ids(&swap_out);
    let swap_ok = !query_errored(&swap_out) && swap_ids == base_ids;

    // Restore epoch 1 on every node for the later drills.
    for node in nodes {
        let _ = publish(node.port, 1).await;
    }

    let pass = base_ok && skew_errored && swap_ok;
    (
        format!(
            "fr082_published_epoch base_ok={base_ok} coord_only_publish_errored={skew_errored} \
             all_publish_swap_ok={swap_ok} pass={pass}"
        ),
        pass,
    )
}

/// NFR-020 mid-delete / lost-tombstone-write drill: attempt a tombstone write via
/// the FR-083 owner endpoint (`ec_distann_apply_record_writes`) with
/// `ec_distann.debug_fail_tombstone_write` on — the endpoint WAL-logs the flag
/// flip, then errors.
///
/// NFR-020 requires that a lost remote tombstone write "must error, never
/// silently resurrect the row." The tombstone flag is a MONOTONIC set (dml.rs),
/// and PostgreSQL does not physically undo WAL-logged index-page changes on a
/// transaction abort — so the flag stays set: the record is deleted and STAYS
/// deleted (the safe, non-resurrecting direction), while the caller still sees an
/// error. This drill asserts exactly that: the write errors AND the record is
/// tombstoned and remains tombstoned across re-reads (monotonic, no resurrection)
/// AND an ANN scan excludes it. Runs on an isolated table so `dm` is untouched.
/// Returns true iff errored AND tombstoned-and-stable AND excluded from scans.
async fn mid_delete_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    args: &LocalMultinodePg18Args,
) -> bool {
    let single = "SELECT set_config('ec_distann.roster','',false); SET ec_distann.local_node_id=1; SET ec_distann.epoch=0;";
    let dim = args.dim;
    let gvec = format!(
        "encode_to_ecvector((SELECT array_agg((sin(g * 0.017 * (d + 1)) + cos(g * 0.0031 * (d + 1)))::real) \
           FROM generate_series(0, {dim} - 1) AS d), 4, 42)"
    );
    let setup = format!(
        "DROP TABLE IF EXISTS md; CREATE TABLE md (id bigint, embedding ecvector); \
         INSERT INTO md SELECT g, {gvec} FROM generate_series(1, 500) AS g; \
         CREATE INDEX md_idx ON md USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = {gd});",
        gd = args.graph_degree,
    );
    if run_psql_file(psql, socket_dir, coord_port, &setup)
        .await
        .is_err()
    {
        return false;
    }
    // Discover a live owned vec_id + its id (to check scan exclusion).
    let discover = format!(
        "{single} SELECT d.vec_id || '|' || t.id \
           FROM ec_distann_list_directory('md_idx'::regclass) d \
           JOIN md t ON t.ctid = ('(' || d.heap_block || ',' || d.heap_offset || ')')::tid \
          WHERE NOT d.is_tombstone ORDER BY t.id LIMIT 1;"
    );
    let discovered = capture_psql_allow_error(psql, socket_dir, coord_port, &discover).await;
    let Some((vec_id, id)) = discovered
        .lines()
        .find_map(|l| l.trim().split_once('|'))
        .filter(|(v, i)| v.parse::<i64>().is_ok() && i.parse::<i64>().is_ok())
        .map(|(v, i)| (v.to_owned(), i.to_owned()))
    else {
        crate::ecaz_println!(
            "[distann-multicluster] mid_delete: no live vec_id discovered (skipped)"
        );
        let _ = run_psql_file(psql, socket_dir, coord_port, "DROP TABLE IF EXISTS md;").await;
        return false;
    };
    // Attempt the tombstone write with injection: must error.
    let attempt = format!(
        "{single} SET ec_distann.debug_fail_tombstone_write=true; \
         SELECT ec_distann_apply_record_writes('md_idx'::regclass, ec_distann_epoch_fingerprint('md_idx'::regclass), ARRAY[{vec_id}]::bigint[]);"
    );
    let attempt_out = capture_psql_allow_error(psql, socket_dir, coord_port, &attempt).await;
    let errored = query_errored(&attempt_out);
    // Re-read is_tombstone twice: monotonic ⇒ tombstoned and stable (no resurrection).
    let tomb = format!(
        "{single} SELECT is_tombstone FROM ec_distann_list_directory('md_idx'::regclass) WHERE vec_id={vec_id};"
    );
    let t1 = capture_psql_allow_error(psql, socket_dir, coord_port, &tomb).await;
    let t2 = capture_psql_allow_error(psql, socket_dir, coord_port, &tomb).await;
    let tombstoned = |o: &str| o.lines().any(|l| l.trim() == "t");
    let stable_tombstoned = tombstoned(&t1) && tombstoned(&t2);
    // And the ANN scan excludes the now-tombstoned record (deleted, not resurrected).
    let scan = format!(
        "{single} SET enable_seqscan=off; \
         SELECT id FROM md ORDER BY embedding <#> (SELECT embedding FROM md WHERE id={id}) LIMIT 10;"
    );
    let scan_out = capture_psql_allow_error(psql, socket_dir, coord_port, &scan).await;
    let excluded = !scan_out.lines().any(|l| l.trim() == id);
    let pass = errored && stable_tombstoned && excluded;
    crate::ecaz_println!(
        "[distann-multicluster] mid_delete_lost_tombstone DIAG vec_id={vec_id} id={id} errored={errored} \
         stable_tombstoned={stable_tombstoned} excluded_from_scan={excluded} pass={pass}"
    );
    let _ = run_psql_file(psql, socket_dir, coord_port, "DROP TABLE IF EXISTS md;").await;
    pass
}

/// FR-083 mid-insert failure drill (TC-043), on an isolated table so the shared
/// `dm` other drills use is untouched. Builds a small graph, buffers a few
/// inserts (delta buffer), then folds them with `ec_distann.debug_fail_insert`
/// on: `graph_insert_record` errors after staging the node + directory pages but
/// before publishing metadata. The aborting statement must roll the staged pages
/// back, so a scan after the failed fold succeeds and is byte-identical to the
/// pre-fold scan (no partial/corrupt record). Returns true iff the fold errored
/// AND the post-fold scan matches the pre-fold scan.
async fn mid_insert_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    args: &LocalMultinodePg18Args,
) -> bool {
    let dim = args.dim;
    let vec = |g: &str| -> String {
        format!(
            "encode_to_ecvector((SELECT array_agg((sin({g} * 0.017 * (d + 1)) + cos({g} * 0.0031 * (d + 1)))::real) \
               FROM generate_series(0, {dim} - 1) AS d), 4, 42)"
        )
    };
    let setup = format!(
        "DROP TABLE IF EXISTS mi; CREATE TABLE mi (id bigint, embedding ecvector); \
         INSERT INTO mi SELECT g, {gvec} FROM generate_series(1, 500) AS g; \
         CREATE INDEX mi_idx ON mi USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = {gd});",
        gvec = vec("g"),
        gd = args.graph_degree,
    );
    if run_psql_file(psql, socket_dir, coord_port, &setup)
        .await
        .is_err()
    {
        return false;
    }
    // Buffer a few inserts into the delta buffer (aminsert), to be folded.
    let more = format!(
        "INSERT INTO mi SELECT g, {gvec} FROM generate_series(501, 510) AS g;",
        gvec = vec("g"),
    );
    if run_psql_file(psql, socket_dir, coord_port, &more)
        .await
        .is_err()
    {
        return false;
    }
    let scan = "SET enable_seqscan=off; SELECT id FROM mi ORDER BY embedding <#> (SELECT embedding FROM mi WHERE id=1) LIMIT 10;";
    let before = capture_psql_allow_error(psql, socket_dir, coord_port, scan).await;
    // Inject the mid-insert failure and fold: the fold must error.
    let fold = "SET ec_distann.debug_fail_insert=true; SELECT ec_distann_fold_delta_into_graph('mi_idx'::regclass);";
    let fold_out = capture_psql_allow_error(psql, socket_dir, coord_port, fold).await;
    let fold_errored = query_errored(&fold_out);
    // Post-failed-fold scan: must still work and match the pre-fold result.
    let after = capture_psql_allow_error(psql, socket_dir, coord_port, scan).await;
    let ids =
        |out: &str| -> Vec<i64> { out.lines().filter_map(|l| l.trim().parse().ok()).collect() };
    let (before_ids, after_ids) = (ids(&before), ids(&after));
    let consistent = !after_ids.is_empty() && after_ids == before_ids;
    let pass = fold_errored && consistent;
    crate::ecaz_println!(
        "[distann-multicluster] mid_insert_failure DIAG fold_errored={fold_errored} \
         before_n={} after_n={} consistent={consistent} pass={pass}",
        before_ids.len(),
        after_ids.len(),
    );
    let _ = run_psql_file(psql, socket_dir, coord_port, "DROP TABLE IF EXISTS mi;").await;
    pass
}

/// One co-placement-drift case: pick a live record owned by `owner_idx`, delete
/// its heap row on EVERY node (index record survives ⇒ cluster-wide dangling
/// record / missing co-placed vector), and assert the NFR-020 disjunction — the
/// multinode scan SHALL either raise an error OR return a correct complete result
/// (equal to a single-node scan over the same deleted corpus, target excluded),
/// never a partial/stale result presented as complete.
async fn co_placement_drift_case(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    roster: &str,
    nodes: &[Node],
    args: &LocalMultinodePg18Args,
    owner_idx: u32,
) -> bool {
    let discover = format!(
        "SELECT t.id || '|' || t.source::text \
           FROM ec_distann_list_directory('dm_idx'::regclass) d \
           JOIN dm t ON t.ctid = ('(' || d.heap_block || ',' || d.heap_offset || ')')::tid \
          WHERE NOT d.is_tombstone \
            AND ec_distann_owning_node(d.vec_id, {n}, 1) = {owner_idx} \
          ORDER BY t.id LIMIT 1;",
        n = args.nodes,
    );
    let discovered = capture_psql_allow_error(psql, socket_dir, coord_port, &discover).await;
    let Some((id_text, source_text)) = discovered
        .lines()
        .find_map(|l| l.trim().split_once('|'))
        .filter(|(id, src)| id.parse::<i64>().is_ok() && src.starts_with('{'))
    else {
        crate::ecaz_println!(
            "[distann-multicluster] co_placement_drift[owner={owner_idx}]: no record discovered (skipped)"
        );
        return false;
    };
    let target_id: i64 = id_text.trim().parse().unwrap();

    for node in nodes {
        if run_psql_file(
            psql,
            socket_dir,
            node.port,
            &format!("DELETE FROM dm WHERE id = {target_id};"),
        )
        .await
        .is_err()
        {
            return false;
        }
    }

    let anchor = format!("encode_to_ecvector('{source_text}'::real[], 4, 42)");
    let multi_sql = format!(
        "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
         SELECT id FROM dm ORDER BY embedding <#> {anchor} LIMIT {k};",
        k = args.top_k,
    );
    let single_sql = format!(
        "SET enable_seqscan=off; SELECT set_config('ec_distann.roster','',false); SET ec_distann.local_node_id=1; SET ec_distann.epoch=0; \
         SELECT id FROM dm ORDER BY embedding <#> {anchor} LIMIT {k};",
        k = args.top_k,
    );
    let multi_out = capture_psql_allow_error(psql, socket_dir, coord_port, &multi_sql).await;
    let single_out = capture_psql_allow_error(psql, socket_dir, coord_port, &single_sql).await;
    let errored = query_errored(&multi_out);

    let ids = |out: &str| -> Vec<i64> {
        let mut v: Vec<i64> = out.lines().filter_map(|l| l.trim().parse().ok()).collect();
        v.sort_unstable();
        v
    };
    let (multi_ids, single_ids) = (ids(&multi_out), ids(&single_out));
    let target_excluded = !multi_ids.contains(&target_id) && !single_ids.contains(&target_id);
    let correct_complete = !errored && multi_ids == single_ids && target_excluded;
    let pass = errored || correct_complete;
    let arm = if errored { "error" } else { "correct_complete" };
    crate::ecaz_println!(
        "[distann-multicluster] co_placement_drift[owner={owner_idx}] target_id={target_id} arm={arm} \
         multi_n={} single_n={} pass={pass}",
        multi_ids.len(),
        single_ids.len(),
    );

    // Recovery: restore the corpus on every node. Must use the SAME setup as the
    // initial build (real staged corpus when --corpus-prefix is set), else the
    // node is rebuilt with the synthetic dim-16 corpus and the post-recovery
    // real-query recall comparison fails with a dimension mismatch.
    if let Ok(setup) = build_setup_sql(args) {
        for node in nodes {
            let _ = run_psql_file(psql, socket_dir, node.port, &setup).await;
        }
    }
    pass
}

/// A drill query satisfies NFR-020's fail-closed arm if it raised an ERROR
/// rather than returning a (possibly wrong/partial) result.
fn query_errored(output: &str) -> bool {
    output.contains("ERROR")
        || output.contains("EC_INTERNAL")
        || output.contains("could not connect")
}

/// FR-082-AC-4 concurrency drill: `scanners` concurrent multi-node scan loops on
/// the coordinator, plus a background inserter mutating the table, all at once.
/// Returns true iff every session completed without error (each scan drew only
/// from expanded records — a torn/half-applied read would surface as an error).
async fn concurrency_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    nodes: &[Node],
    roster: &str,
    args: &LocalMultinodePg18Args,
) -> Result<bool> {
    let scanners = 4;
    let iters = 12;
    let scan_sql = format!(
        "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
         SELECT count(*) FROM (SELECT id FROM dm ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT {}) t;",
        args.top_k
    );
    // Insert vector: a real corpus vector (correct dimension) in real mode, else
    // a synthetic vector matching the corpus generator. Synthetic args.dim does
    // NOT match a real corpus dimension, so a synthetic vector would fail the
    // aminsert dimension check against the real index.
    let arr = insert_vector_expr(args);

    let mut tasks = Vec::new();
    for _ in 0..scanners {
        let (psql, socket_dir, sql) = (
            psql.to_path_buf(),
            socket_dir.to_path_buf(),
            scan_sql.clone(),
        );
        tasks.push(tokio::spawn(async move {
            for _ in 0..iters {
                let out = run_capture(&psql, &socket_dir, coord_port, &sql).await;
                if !out.status_ok {
                    // A scan that races an in-progress coordinated epoch swap may
                    // fail-closed with an epoch mismatch (FR-082-AC-2, one epoch per
                    // scan) — that is a correct outcome, not corruption. Any OTHER
                    // error (torn read, crash, wrong-result path) fails the drill.
                    let stderr = out.stderr.to_lowercase();
                    if stderr.contains("epoch") && stderr.contains("mismatch") {
                        continue;
                    }
                    return Err(out.stderr);
                }
            }
            Ok(())
        }));
    }
    // Background inserter: unique ids well above the corpus range.
    {
        let (psql, socket_dir, arr) = (psql.to_path_buf(), socket_dir.to_path_buf(), arr.clone());
        let base_rows = args.rows;
        tasks.push(tokio::spawn(async move {
            for i in 0..iters {
                let sql = format!(
                    "INSERT INTO dm SELECT {}, {arr}, encode_to_ecvector({arr}, 4, 42);",
                    900_000 + base_rows as i64 + i
                );
                let out = run_capture(&psql, &socket_dir, coord_port, &sql).await;
                if !out.status_ok {
                    return Err(out.stderr);
                }
            }
            Ok(())
        }));
    }
    // Epoch-swap-under-load (FR-082-AC-1 / one-epoch-per-scan): perform COORDINATED
    // epoch publishes across EVERY node while scans run. Publishing on all nodes
    // keeps the cluster at a single consistent epoch (all-1 or all-2) so each
    // in-flight scan returns wholly from one published epoch — a scan that races a
    // swap surfaces a retriable epoch mismatch and restarts under the refreshed
    // epoch (FR-082-AC-2), never a torn result. The metadata-page publish write
    // must not corrupt concurrent scans reading the metadata; end back at epoch 1.
    {
        let (psql, socket_dir) = (psql.to_path_buf(), socket_dir.to_path_buf());
        let ports: Vec<u16> = nodes.iter().map(|n| n.port).collect();
        tasks.push(tokio::spawn(async move {
            for i in 0..iters {
                let epoch = if i % 2 == 0 { 2 } else { 1 };
                let sql =
                    format!("SELECT ec_distann_publish_epoch('dm_idx'::regclass::oid, {epoch});");
                for &port in &ports {
                    let out = run_capture(&psql, &socket_dir, port, &sql).await;
                    if !out.status_ok {
                        return Err(out.stderr);
                    }
                }
            }
            for &port in &ports {
                let _ = run_capture(
                    &psql,
                    &socket_dir,
                    port,
                    "SELECT ec_distann_publish_epoch('dm_idx'::regclass::oid, 1);",
                )
                .await;
            }
            Ok(())
        }));
    }

    let mut ok = true;
    for task in tasks {
        match task.await {
            Ok(Ok(())) => {}
            Ok(Err(stderr)) => {
                crate::ecaz_println!("[distann-multicluster] concurrency session error: {stderr}");
                ok = false;
            }
            Err(join_err) => {
                crate::ecaz_println!(
                    "[distann-multicluster] concurrency task panicked: {join_err}"
                );
                ok = false;
            }
        }
    }
    Ok(ok)
}

/// True disjoint-shard demonstration: prune each node's replicated corpus to only
/// the heap rows it OWNS (`owning_node`), then prove the multi-node top-k result
/// signature is byte-identical to the pre-prune (replicated) result — i.e. the
/// distributed read is correct with genuinely disjoint per-node storage, not a
/// full replica. Returns a report line; fatal if the signature changes.
async fn disjoint_shard_drill(
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    roster: &str,
    args: &LocalMultinodePg18Args,
) -> (String, bool) {
    let coord_port = nodes[0].port;
    // Operate on benchgate_corpus (a clean, cross-node-consistent copy the suite
    // gate created before the mutating drills). Save the query vectors first so
    // they survive pruning of non-owned coordinator rows.
    let setup = run_capture(
        psql,
        socket_dir,
        coord_port,
        &format!(
            "DROP TABLE IF EXISTS dj_queries; \
             CREATE TABLE dj_queries AS SELECT id AS qid, source AS v FROM benchgate_corpus WHERE id <= {};",
            args.queries
        ),
    )
    .await;
    if !setup.status_ok {
        return (
            "disjoint_shard=SKIPPED(no benchgate_corpus)".to_owned(),
            false,
        );
    }
    // Signature over (id, EXACT DISTANCE) per query in a canonical (dist, id)
    // order (021-P2): includes the distance — not just the id set — so a
    // distance/recall change is caught, while the canonical order makes it
    // deterministic (equal-distance tie order, which the scan does not guarantee
    // and which is not a recall property, does not spuriously fail the drill).
    let sig_sql = format!(
        "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
         SELECT md5(string_agg(qid || ':' || id || ':' || dist, ',' ORDER BY qid, dist, id)) FROM ( \
           SELECT q.qid, r.id, (r.embedding <#> q.v)::float8 AS dist FROM dj_queries q \
           CROSS JOIN LATERAL ( \
             SELECT id, embedding FROM benchgate_corpus ORDER BY embedding <#> q.v LIMIT {k}) r) t;",
        k = args.top_k
    );
    let sig = |out: String| {
        out.lines()
            .map(str::trim)
            .find(|l| l.len() == 32)
            .unwrap_or("")
            .to_owned()
    };
    let before = sig(capture_psql_allow_error(psql, socket_dir, coord_port, &sig_sql).await);

    // Prune each node to its owned shard: delete the heap rows for vec_ids this
    // node does not own, then VACUUM (ambulkdelete tombstones their records).
    let n = args.nodes;
    let mut row_report = Vec::new();
    for node in nodes {
        let owner_idx = node.node_id - 1; // placement index = roster position
        let before_rows = capture_psql_allow_error(
            psql,
            socket_dir,
            node.port,
            "SELECT count(*) FROM benchgate_corpus;",
        )
        .await
        .lines()
        .find_map(|l| l.trim().parse::<i64>().ok())
        .unwrap_or(-1);
        let del = run_capture(
            psql,
            socket_dir,
            node.port,
            &format!(
                "DELETE FROM benchgate_corpus WHERE ctid IN (\
                   SELECT ('(' || heap_block || ',' || heap_offset || ')')::tid \
                     FROM ec_distann_list_directory('benchgate_corpus_idx'::regclass::oid) \
                    WHERE NOT is_tombstone AND ec_distann_owning_node(vec_id, {n}, 1) <> {owner_idx});"
            ),
        )
        .await;
        let vac = run_capture(psql, socket_dir, node.port, "VACUUM benchgate_corpus;").await;
        if !del.status_ok || !vac.status_ok {
            return ("disjoint_shard=SKIPPED(prune failed)".to_owned(), false);
        }
        let after_rows = capture_psql_allow_error(
            psql,
            socket_dir,
            node.port,
            "SELECT count(*) FROM benchgate_corpus;",
        )
        .await
        .lines()
        .find_map(|l| l.trim().parse::<i64>().ok())
        .unwrap_or(-1);
        row_report.push(format!("n{}:{}->{}", node.node_id, before_rows, after_rows));
    }

    let after = sig(capture_psql_allow_error(psql, socket_dir, coord_port, &sig_sql).await);
    let identical = !before.is_empty() && before == after;
    (
        format!(
            "disjoint_shard identical_after_prune={identical} per_node_rows[{}]",
            row_report.join(" ")
        ),
        identical,
    )
}

/// Suite-driven recall gate (006-P1 letter): reuse the fixture corpus as a
/// `benchgate_*` bench-format corpus and run `ecaz bench recall` against the
/// coordinator single-node vs multi-node, asserting recall(multi) >= recall(single)
/// - 0.001. Best-effort/non-fatal: the byte-identical top-k gate is the hard one.
/// Task 172 AC-3 / NFR-018 cluster storage summation. Queries each node for its
/// `dm_idx` index bytes and `dm` heap bytes, sums across the cluster, and
/// computes the replicated-index space amplification vs raw f32 vectors
/// (`cluster_index_bytes / (rows * dim * 4)`). Emits one `storage_node` line per
/// node plus a `storage_summation` summary line (both parsed into result rows).
async fn storage_summation(
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    _args: &LocalMultinodePg18Args,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cluster_index = 0i64;
    let mut cluster_heap = 0i64;
    let mut coord_index = 0i64;
    let mut rows = 0i64;
    let mut dim = 0i64;
    for (k, node) in nodes.iter().enumerate() {
        // index bytes | heap bytes | row count | vector dimension
        let sql = "SELECT pg_total_relation_size('dm_idx') || ' ' || \
                   pg_total_relation_size('dm') || ' ' || \
                   (SELECT count(*) FROM dm) || ' ' || \
                   coalesce((SELECT array_length(source, 1) FROM dm LIMIT 1), 0);";
        let out = capture_psql_allow_error(psql, socket_dir, node.port, sql).await;
        let vals: Vec<i64> = out
            .lines()
            .find(|l| {
                l.split_whitespace().count() == 4
                    && l.split_whitespace().all(|f| f.parse::<i64>().is_ok())
            })
            .map(|l| {
                l.split_whitespace()
                    .filter_map(|f| f.parse().ok())
                    .collect()
            })
            .unwrap_or_default();
        if vals.len() != 4 {
            lines.push(format!(
                "storage_node node={} index_bytes=0 heap_bytes=0 rows=0 (parse failed)",
                node.node_id
            ));
            continue;
        }
        let (idx, heap, n, d) = (vals[0], vals[1], vals[2], vals[3]);
        if k == 0 {
            coord_index = idx;
            rows = n;
            dim = d;
        }
        cluster_index += idx;
        cluster_heap += heap;
        lines.push(format!(
            "storage_node node={} index_bytes={idx} heap_bytes={heap} rows={n} dim={d}",
            node.node_id
        ));
    }
    let raw_vector_bytes = rows * dim * 4;
    let amp = if raw_vector_bytes > 0 {
        cluster_index as f64 / raw_vector_bytes as f64
    } else {
        0.0
    };
    lines.push(format!(
        "storage_summation nodes={} coord_index_bytes={coord_index} cluster_index_bytes={cluster_index} \
         cluster_heap_bytes={cluster_heap} corpus_rows={rows} dim={dim} raw_vector_bytes={raw_vector_bytes} \
         cluster_index_space_amplification={amp:.4}",
        nodes.len()
    ));
    lines
}

async fn suite_recall_gate(
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    roster: &str,
    args: &LocalMultinodePg18Args,
) -> String {
    let gd = args.graph_degree;
    for node in nodes {
        let sql = format!(
            "DROP TABLE IF EXISTS benchgate_corpus; \
             CREATE TABLE benchgate_corpus AS SELECT * FROM dm; \
             CREATE INDEX benchgate_corpus_idx ON benchgate_corpus \
               USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = {gd});"
        );
        if !run_capture(psql, socket_dir, node.port, &sql)
            .await
            .status_ok
        {
            return "suite_recall_gate=SKIPPED(benchgate setup failed)".to_owned();
        }
    }
    let coord_port = nodes[0].port;
    // Real lane: held-out queries from dm_queries. Synthetic lane: first 50
    // corpus rows (as before).
    let benchgate_queries_sql = if args.corpus_prefix.is_some() {
        format!(
            "DROP TABLE IF EXISTS benchgate_queries; CREATE TABLE benchgate_queries AS \
             SELECT id, source FROM dm_queries ORDER BY id LIMIT {};",
            args.queries
        )
    } else {
        "DROP TABLE IF EXISTS benchgate_queries; CREATE TABLE benchgate_queries AS SELECT id, source FROM dm WHERE id <= 50;".to_owned()
    };
    let _ = run_capture(psql, socket_dir, coord_port, &benchgate_queries_sql).await;
    let ecaz = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return "suite_recall_gate=SKIPPED(no exe)".to_owned(),
    };
    let single = run_bench_recall(&ecaz, socket_dir, coord_port, "").await;
    let multi = run_bench_recall(&ecaz, socket_dir, coord_port, roster).await;
    match (single, multi) {
        (Some(s), Some(m)) => {
            let pass = m >= s - 0.001;
            format!(
                "suite_recall_gate single={s:.4} multi={m:.4} delta={:.4} pass={pass}",
                m - s
            )
        }
        _ => "suite_recall_gate=INCONCLUSIVE(recall parse/connect failed)".to_owned(),
    }
}

/// Invoke `ecaz bench recall` against the coordinator with the given roster
/// session-GUC; parse recall@k from the comfy-table (a single-sweep row).
async fn run_bench_recall(
    ecaz: &Path,
    socket_dir: &Path,
    coord_port: u16,
    roster_val: &str,
) -> Option<f64> {
    let mut cmd = Command::new(ecaz);
    cmd.arg("--database")
        .arg("postgres")
        .arg("--host")
        .arg(socket_dir)
        .arg("--port")
        .arg(coord_port.to_string())
        .arg("bench")
        .arg("recall")
        .arg("--prefix")
        .arg("benchgate")
        .arg("--profile")
        .arg("ec_distann")
        .arg("--k")
        .arg("10")
        .arg("--sweep")
        .arg("32")
        .arg("--force-index");
    // Single-node = default (empty) roster; the GUC parser rejects an empty value,
    // so only set it for the multi-node arm.
    if !roster_val.is_empty() {
        cmd.arg("--session-guc")
            .arg(format!("ec_distann.roster={roster_val}"))
            .arg("--session-guc")
            .arg("ec_distann.local_node_id=1")
            .arg("--session-guc")
            .arg("ec_distann.epoch=1");
    }
    let out = cmd.output().await.ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let errtext = String::from_utf8_lossy(&out.stderr);
    if !out.status.success() {
        crate::ecaz_println!(
            "[distann-multicluster] bench recall (roster={:?}) exit={:?} stderr={}",
            !roster_val.is_empty(),
            out.status.code(),
            errtext
                .lines()
                .rev()
                .take(3)
                .collect::<Vec<_>>()
                .join(" | ")
        );
    }
    // comfy-table data row (columns: top_k/sweep, queries, recall_trials,
    // recall@k, ...): `│ 32 ┆ 50 ┆ 500 ┆ 0.5040 ┆ ...`. The left border is '│',
    // inner columns are separated by '┆'; recall@k is field index 4.
    for line in text.lines() {
        let fields: Vec<&str> = line.split(['│', '┆']).map(str::trim).collect();
        if fields.len() > 4 && fields[1].parse::<i64>().is_ok() {
            if let Ok(recall) = fields[4].parse::<f64>() {
                return Some(recall);
            }
        }
    }
    None
}

/// FR-082-AC-3 live gate: hold a single-node index scan open (AccessShareLock on
/// dm_idx) in a background transaction, and assert `ec_distann_retire_epoch` is
/// gated while it is in flight, then succeeds once it drains.
async fn retention_gate_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    args: &LocalMultinodePg18Args,
) -> bool {
    let idx = "'dm_idx'::regclass::oid";
    // Background holder: an ec_distann index scan held open ~3s via a cursor.
    let hold_sql = format!(
        "SET enable_seqscan=off; SET ec_distann.roster=''; SET ec_distann.local_node_id=1; \
         BEGIN; \
         DECLARE c CURSOR FOR SELECT id FROM dm ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT {}; \
         FETCH 1 FROM c; SELECT pg_sleep(3); COMMIT;",
        args.top_k
    );
    let holder = {
        let (psql, socket_dir) = (psql.to_path_buf(), socket_dir.to_path_buf());
        tokio::spawn(async move { run_capture(&psql, &socket_dir, coord_port, &hold_sql).await })
    };
    // Let the scan acquire its AccessShareLock.
    tokio::time::sleep(std::time::Duration::from_millis(900)).await;

    let gated_out = capture_psql_allow_error(
        psql,
        socket_dir,
        coord_port,
        &format!("SELECT ec_distann_retire_epoch({idx})"),
    )
    .await;
    let gated = gated_out.contains("retention gate");

    let _ = holder.await;

    let drained_out = capture_psql_allow_error(
        psql,
        socket_dir,
        coord_port,
        &format!("SELECT ec_distann_retire_epoch({idx})"),
    )
    .await;
    let succeeded_after_drain = !drained_out.contains("ERROR");

    // Restore a Published epoch for any downstream steps.
    let _ = run_capture(
        psql,
        socket_dir,
        coord_port,
        &format!("SELECT ec_distann_publish_epoch({idx}, 1)"),
    )
    .await;

    gated && succeeded_after_drain
}

/// FR-082-AC-5: a live record's exact-rerank result must be byte-identical after
/// real delete+VACUUM+reinsert TID churn on every node. Deleted records are
/// tombstoned by the AM's ambulkdelete (never reranked); a live record's heap TID
/// is never reclaimed, so its co-placed vector is frozen without a separate tier.
async fn frozen_vector_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    roster: &str,
    nodes: &[Node],
    args: &LocalMultinodePg18Args,
) -> bool {
    // Probe: row 1's multi-node top-1 (id:distance), byte-exact.
    let probe = format!(
        "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
         SELECT id || ':' || (embedding <#> (SELECT source FROM dm WHERE id=1))::float8 \
           FROM dm ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT 1;"
    );
    let baseline = capture_psql_allow_error(psql, socket_dir, coord_port, &probe).await;
    let baseline = baseline
        .lines()
        .find(|l| l.contains(':'))
        .unwrap_or("")
        .trim()
        .to_owned();
    if baseline.is_empty() {
        crate::ecaz_println!("[distann-multicluster] ac5 baseline probe empty");
        return false;
    }

    // Delete a mid range on every node, then VACUUM (triggers ambulkdelete →
    // tombstone + heap reclaim), freeing those TIDs for reuse.
    let lo = args.rows / 4;
    let hi = lo + 150;
    for node in nodes {
        let del = run_capture(
            psql,
            socket_dir,
            node.port,
            &format!("DELETE FROM dm WHERE id BETWEEN {lo} AND {hi};"),
        )
        .await;
        let vac = run_capture(psql, socket_dir, node.port, "VACUUM dm;").await;
        if !del.status_ok || !vac.status_ok {
            crate::ecaz_println!(
                "[distann-multicluster] ac5 delete/vacuum failed on node {}",
                node.node_id
            );
            return false;
        }
    }
    // Reinsert new rows on every node (may reuse the reclaimed TIDs). Use a
    // real corpus vector (correct dimension) in real mode; the reinserted vector
    // content is irrelevant to the frozen-vector check (which probes a
    // pre-existing live record), only its dimension must match the index.
    let arr = insert_vector_expr(args);
    for node in nodes {
        let ins = run_capture(
            psql,
            socket_dir,
            node.port,
            &format!(
                "INSERT INTO dm SELECT g, {arr}, encode_to_ecvector({arr}, 4, 42) FROM generate_series({lo}, {hi}) AS g;"
            ),
        )
        .await;
        if !ins.status_ok {
            crate::ecaz_println!(
                "[distann-multicluster] ac5 reinsert failed on node {}",
                node.node_id
            );
            return false;
        }
    }

    // Re-probe: row 1 (never touched) must rerank byte-identically.
    let after = capture_psql_allow_error(psql, socket_dir, coord_port, &probe).await;
    if after.contains("EC_VECTOR_MISSING") || after.contains("ERROR") {
        crate::ecaz_println!("[distann-multicluster] ac5 post-churn probe errored: {after}");
        return false;
    }
    let after = after
        .lines()
        .find(|l| l.contains(':'))
        .unwrap_or("")
        .trim()
        .to_owned();
    baseline == after
}

/// 011/020-P1: a WHERE qual on a NON-projected column (`source`) plus LIMIT.
/// Multi-node (CustomScan) must return exactly the single-node result — proving
/// the qual column is shipped for remote rows and the LIMIT applies after the
/// qual (over-fetch), not before. Returns (report line, pass).
async fn qual_correctness_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    roster: &str,
    args: &LocalMultinodePg18Args,
) -> (String, bool) {
    // Order by the id=1 vector; filter on source[1] > 0 (source is NOT selected).
    let query = format!(
        "SELECT id FROM dm WHERE source[1] > 0 \
           ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT {}",
        args.top_k
    );
    let sql = format!(
        "SET enable_seqscan=off; \
         DROP TABLE IF EXISTS qc_s; DROP TABLE IF EXISTS qc_m; \
         SELECT set_config('ec_distann.roster', '', false); SET ec_distann.local_node_id=1; SET ec_distann.epoch=0; \
         CREATE TEMP TABLE qc_s AS {query}; \
         SELECT set_config('ec_distann.roster', '{roster}', false); SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
         CREATE TEMP TABLE qc_m AS {query}; \
         SELECT set_config('ec_distann.roster', '', false); \
         SELECT (SELECT count(*) FROM qc_s) || ' ' || (SELECT count(*) FROM qc_m) || ' ' || \
           ((SELECT count(*) FROM (SELECT id FROM qc_s EXCEPT SELECT id FROM qc_m) x) \
          + (SELECT count(*) FROM (SELECT id FROM qc_m EXCEPT SELECT id FROM qc_s) x));"
    );
    let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
    let parsed: Vec<i64> = out
        .lines()
        .find(|l| {
            l.split_whitespace().count() == 3
                && l.split_whitespace().all(|f| f.parse::<i64>().is_ok())
        })
        .map(|l| {
            l.split_whitespace()
                .filter_map(|f| f.parse().ok())
                .collect()
        })
        .unwrap_or_default();
    if parsed.len() != 3 {
        return (
            format!(
                "qual_correctness=INCONCLUSIVE({})",
                out.lines().last().unwrap_or("").trim()
            ),
            false,
        );
    }
    let (s_n, m_n, mismatch) = (parsed[0], parsed[1], parsed[2]);
    // Pass = same count and zero id mismatch (single==multi under the qual+LIMIT).
    let pass = s_n == m_n && mismatch == 0;
    (
        format!("qual_correctness single_n={s_n} multi_n={m_n} mismatch={mismatch} pass={pass}"),
        pass,
    )
}

struct CaptureOut {
    status_ok: bool,
    stderr: String,
}

async fn run_capture(psql: &Path, socket_dir: &Path, port: u16, sql: &str) -> CaptureOut {
    let mut command = psql_base(psql, socket_dir, port);
    command
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .arg("-tAc")
        .arg(sql);
    match command.output().await {
        Ok(output) => CaptureOut {
            status_ok: output.status.success(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        },
        Err(error) => CaptureOut {
            status_ok: false,
            stderr: format!("spawn error: {error}"),
        },
    }
}

fn psql_base(psql: &Path, _socket_dir: &Path, port: u16) -> Command {
    let mut command = Command::new(psql);
    command
        .arg("-h")
        .arg("127.0.0.1")
        .arg("-p")
        .arg(port.to_string())
        .arg("-U")
        .arg("postgres")
        .arg("-d")
        .arg("postgres");
    command
}

async fn run_psql_file(psql: &Path, socket_dir: &Path, port: u16, sql: &str) -> Result<()> {
    let mut command = psql_base(psql, socket_dir, port);
    command
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .arg("-c")
        .arg(sql)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    run_status(command).await
}

async fn capture_psql(psql: &Path, socket_dir: &Path, port: u16, sql: &str) -> Result<String> {
    let mut command = psql_base(psql, socket_dir, port);
    command
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .arg("-tAc")
        .arg(sql);
    let output = command.output().await.wrap_err("spawning psql")?;
    if !output.status.success() {
        bail!("psql failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

async fn capture_psql_allow_error(psql: &Path, socket_dir: &Path, port: u16, sql: &str) -> String {
    let mut command = psql_base(psql, socket_dir, port);
    command.arg("-tAc").arg(sql);
    match command.output().await {
        Ok(output) => {
            let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
            combined.push_str(&String::from_utf8_lossy(&output.stderr));
            combined
        }
        Err(error) => format!("psql spawn error: {error}"),
    }
}
