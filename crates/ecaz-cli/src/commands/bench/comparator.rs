//! `ecaz bench comparator` — standalone, engine-generic competitor measurement.
//!
//! Unlike the retired head-to-head `compare` surface, this command measures
//! **one** external engine (vchord / pgvector-hnsw / pgvector-ivfflat /
//! pgvectorscale) on its own — there is no ecaz engine in the loop. That keeps
//! comparator measurement decoupled from ecaz re-measurement, which is required
//! by the no-re-run policy (comparators are re-run only on a competitor-version
//! or hardware change).
//!
//! # Flow
//!
//! 1. Ensure the engine's extension is installed (for vchord, surface the
//!    `shared_preload_libraries` + PG-restart prerequisite — install stays a
//!    manual operator step).
//! 2. Materialize a `<prefix>_corpus_<engine>` sidecar `(id bigint, embedding
//!    vector(dim))` sourced from the ecaz `<prefix>_corpus.source` column, and
//!    build the engine's index once, idempotently (skip if present unless
//!    `--rebuild`).
//! 3. Compute brute-force ground truth once with the helper `bench recall`
//!    already uses.
//! 4. For each `--sweep` value (the engine's query GUC), run the query set once,
//!    capturing per-query top-k ids + durations.
//! 5. Emit one Pareto row per sweep value (recall@k + latency percentiles) plus
//!    a `pg_relation_size` storage line, in the same table shape the suite
//!    parser reads for recall/latency/storage.
//!
//! # Purity boundary
//!
//! SQL builders, the per-engine GUC/name mapping, and `default_lists_for_rows`
//! are pure functions with unit tests. The orchestration shell is a thin
//! tokio-postgres driver on top.

use clap::{Args, ValueEnum};
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use indicatif::ProgressStyle;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio_postgres::Client;

use crate::profiles;
use crate::psql::{self, ConnectionOptions};

use super::latency::{summarize, LatencyStats};
use super::recall::{brute_force_top_k, map_indices_to_ids, ndcg_at_k, recall_at_k};

/// External engine selector. Each variant pins the extension, the query-time
/// tuning GUC, and the index DDL shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum ComparatorEngine {
    /// VectorChord `vchordrq` (RaBitQ-on-IVF). Requires preload + PG restart.
    Vchord,
    /// pgvector HNSW.
    PgvectorHnsw,
    /// pgvector IVFFlat.
    PgvectorIvfflat,
    /// pgvectorscale StreamingDiskANN.
    Pgvectorscale,
}

#[derive(Args, Debug)]
pub struct ComparatorArgs {
    /// External engine to measure standalone.
    #[arg(long, value_enum)]
    pub engine: ComparatorEngine,
    /// Prefix identifying the ecaz corpus (as loaded by `ecaz corpus load`).
    #[arg(long)]
    pub prefix: String,
    /// k for recall@k / latency measurement.
    #[arg(long, default_value_t = 10)]
    pub k: usize,
    /// Query-GUC sweep values (the engine's tuning axis): vchord
    /// `vchordrq.probes`, pgvector-hnsw `hnsw.ef_search`, pgvector-ivfflat
    /// `ivfflat.probes`, pgvectorscale `diskann.query_search_list_size`.
    #[arg(long, value_delimiter = ',')]
    pub sweep: Vec<i32>,
    /// Cap the query set (default: all rows).
    #[arg(long)]
    pub queries_limit: Option<usize>,
    /// IVF-family (vchord / pgvector-ivfflat) build `lists`. Defaults to
    /// `ceil(sqrt(row_count))`; pin (224/320/1024) to reproduce recorded data.
    #[arg(long)]
    pub lists: Option<i32>,
    /// pgvector HNSW build `m`.
    #[arg(long, default_value_t = 16)]
    pub m: i32,
    /// pgvector HNSW build `ef_construction`.
    #[arg(long, default_value_t = 128)]
    pub ef_construction: i32,
    /// pgvectorscale StreamingDiskANN build num_neighbors.
    #[arg(long, default_value_t = 32)]
    pub num_neighbors: i32,
    /// pgvectorscale StreamingDiskANN build search_list_size.
    #[arg(long, default_value_t = 100)]
    pub build_search_list_size: i32,
    /// pgvectorscale StreamingDiskANN build max_alpha.
    #[arg(long, default_value_t = 1.2)]
    pub max_alpha: f32,
    /// pgvectorscale storage_layout index option.
    #[arg(long, default_value = "memory_optimized")]
    pub storage_layout: String,
    /// Session maintenance_work_mem used while building the sidecar index.
    #[arg(long, default_value = "4GB")]
    pub maintenance_work_mem: String,
    /// Drop + rebuild the sidecar table + index before measuring.
    #[arg(long, default_value_t = false)]
    pub rebuild: bool,
    /// Mirror the Pareto table + storage line to a packet-local artifact file.
    #[arg(long)]
    pub log_output: Option<PathBuf>,
}

impl ComparatorEngine {
    /// Short engine label used for sidecar/index naming and table rows.
    pub fn label(self) -> &'static str {
        match self {
            Self::Vchord => "vchord",
            Self::PgvectorHnsw => "pgvector_hnsw",
            Self::PgvectorIvfflat => "pgvector_ivfflat",
            Self::Pgvectorscale => "pgvectorscale",
        }
    }

    /// Postgres extension name installed for this engine.
    fn extension(self) -> &'static str {
        match self {
            Self::Vchord => "vchord",
            Self::PgvectorHnsw | Self::PgvectorIvfflat => "vector",
            Self::Pgvectorscale => "vectorscale",
        }
    }

    /// Query-time tuning GUC swept by `--sweep`.
    fn query_guc(self) -> &'static str {
        match self {
            Self::Vchord => "vchordrq.probes",
            Self::PgvectorHnsw => "hnsw.ef_search",
            Self::PgvectorIvfflat => "ivfflat.probes",
            Self::Pgvectorscale => "diskann.query_search_list_size",
        }
    }

    /// Tuning-axis label embedded in the row label, e.g. `probes`/`ef_search`.
    fn axis_label(self) -> &'static str {
        match self {
            Self::Vchord | Self::PgvectorIvfflat => "probes",
            Self::PgvectorHnsw => "ef_search",
            Self::Pgvectorscale => "query_search_list_size",
        }
    }

    /// Whether the build uses the `--lists` knob (IVF-family).
    fn uses_lists(self) -> bool {
        matches!(self, Self::Vchord | Self::PgvectorIvfflat)
    }
}

pub async fn run(conn: &ConnectionOptions, args: ComparatorArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    if args.k == 0 {
        return Err(eyre!("--k must be >= 1"));
    }
    if args.sweep.is_empty() {
        return Err(eyre!("--sweep must include at least one query-GUC value"));
    }
    validate_args(&args)?;

    let engine = args.engine;
    let corpus_table = format!("{}_corpus", args.prefix);
    let queries_table = format!("{}_queries", args.prefix);
    let sidecar_table = sidecar_name(&args.prefix, engine);
    let sidecar_index = index_name(&args.prefix, engine);

    let client = psql::connect(conn).await?;

    if !psql::relation_exists(&client, &corpus_table, 'r').await? {
        return Err(eyre!(
            "no ecaz corpus table {corpus_table} in this database"
        ));
    }
    if !psql::relation_exists(&client, &queries_table, 'r').await? {
        return Err(eyre!("no queries table {queries_table} in this database"));
    }

    client
        .batch_execute(&format!(
            "CREATE EXTENSION IF NOT EXISTS {} CASCADE",
            engine.extension()
        ))
        .await
        .wrap_err_with(|| extension_error(engine))?;

    let dim = read_dim(&client, &corpus_table).await?;
    let corpus_rows = count_rows(&client, &corpus_table).await?;
    let lists = args
        .lists
        .unwrap_or_else(|| default_lists_for_rows(corpus_rows));

    // Accumulate the parser-visible lines (build + size + table) so the
    // `--log-output` artifact is self-contained for the suite parser.
    let mut output = String::new();

    let build_summary = ensure_sidecar(
        &client,
        &corpus_table,
        &sidecar_table,
        &sidecar_index,
        dim,
        BuildConfig {
            engine,
            lists,
            hnsw_m: args.m,
            hnsw_ef_construction: args.ef_construction,
            num_neighbors: args.num_neighbors,
            build_search_list_size: args.build_search_list_size,
            max_alpha: args.max_alpha,
            storage_layout: &args.storage_layout,
            maintenance_work_mem: &args.maintenance_work_mem,
        },
        args.rebuild,
    )
    .await?;
    if let Some(seconds) = build_summary.build_seconds {
        output.push_str(&format!(
            "[comparator] built {sidecar_index} in {seconds:.2}s\n"
        ));
    }
    output.push_str(&format!(
        "[comparator] {sidecar_index} pg_relation_size={} bytes\n",
        build_summary.index_bytes
    ));

    crate::ecaz_eprintln!("[comparator] fetching corpus + queries for ground truth ...");
    let (corpus_ids, corpus) =
        super::recall::fetch_sources_public(&client, &corpus_table, None).await?;
    let (_, queries) =
        super::recall::fetch_sources_public(&client, &queries_table, args.queries_limit).await?;
    if corpus.nrows() == 0 || queries.nrows() == 0 {
        return Err(eyre!("corpus or queries empty"));
    }
    if corpus.ncols() != queries.ncols() {
        return Err(eyre!(
            "dim mismatch: corpus={} queries={}",
            corpus.ncols(),
            queries.ncols()
        ));
    }

    crate::ecaz_eprintln!("[comparator] computing ground truth ...");
    let t0 = Instant::now();
    let gt = brute_force_top_k(&corpus, &queries, args.k);
    crate::ecaz_eprintln!("[comparator] ground truth in {:.2?}", t0.elapsed());
    psql::prefer_ordered_ann_path(&client).await?;
    let truth_ids = map_indices_to_ids(&gt.indices, &corpus_ids);
    let knn_sql = build_knn_sql(&sidecar_table, dim);

    let query_guc = engine.query_guc();
    let mut rows = Vec::with_capacity(args.sweep.len());
    for value in &args.sweep {
        client
            .batch_execute(&format!("SET {query_guc} = {value}"))
            .await
            .wrap_err_with(|| format!("SET {query_guc}"))?;
        let label = configured_engine_label(engine.label(), engine.axis_label(), *value);
        let (recall, ndcg, stats) = measure_engine(
            &client,
            &label,
            &knn_sql,
            &queries,
            &gt,
            &corpus_ids,
            &truth_ids,
            args.k,
        )
        .await?;
        rows.push(ParetoRow {
            engine: label,
            sweep: *value,
            recall,
            ndcg,
            stats,
        });
    }

    output.push_str(&render_table(&rows));
    println!("{output}");
    if let Some(path) = &args.log_output {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .wrap_err_with(|| format!("creating {}", parent.display()))?;
        }
        tokio::fs::write(path, format!("{output}\n"))
            .await
            .wrap_err_with(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

/// Name of the sidecar table for a given ecaz prefix + engine.
pub fn sidecar_name(prefix: &str, engine: ComparatorEngine) -> String {
    format!("{prefix}_corpus_{}", engine.label())
}

/// Name of the index built on the sidecar.
pub fn index_name(prefix: &str, engine: ComparatorEngine) -> String {
    format!("{prefix}_corpus_{}_idx", engine.label())
}

/// `CREATE TABLE IF NOT EXISTS` for the sidecar. Idempotent; the caller
/// separately decides whether to re-populate.
pub fn build_sidecar_ddl(sidecar: &str, dim: usize) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {sidecar} (\n    id bigint PRIMARY KEY,\n    embedding vector({dim}) NOT NULL\n)"
    )
}

/// Idempotent populate from the ecaz `source` column (`real[]` casts to
/// `vector(dim)` natively).
pub fn build_populate_sql(corpus_table: &str, sidecar: &str, dim: usize) -> String {
    format!(
        "INSERT INTO {sidecar} (id, embedding)\n         SELECT id, source::vector({dim}) FROM {corpus_table}\n         ON CONFLICT (id) DO NOTHING"
    )
}

/// KNN template: binds are `($1::real[], $2::bigint)` = (query_source, k).
/// Uses the inner-product operator to match ecaz's IP semantics.
pub fn build_knn_sql(sidecar: &str, dim: usize) -> String {
    format!(
        "SELECT id FROM {sidecar} \
         ORDER BY embedding <#> \
         $1::real[]::vector({dim}) \
         LIMIT $2"
    )
}

/// `CREATE INDEX ... USING vchordrq (embedding vector_ip_ops)` with the
/// RaBitQ-on-IVF residual-quantization options block.
pub fn build_vchord_create_index_sql(sidecar: &str, index_name: &str, lists: i32) -> String {
    format!(
        "CREATE INDEX {index_name} ON {sidecar}\n         USING vchordrq (embedding vector_ip_ops)\n         WITH (options = $vco$\nresidual_quantization = true\n[build.internal]\nlists = [{lists}]\n$vco$)"
    )
}

/// `CREATE INDEX ... USING hnsw (embedding vector_ip_ops) WITH (m, ef_construction)`
pub fn build_pgvector_hnsw_index_sql(
    sidecar: &str,
    index_name: &str,
    m: i32,
    ef_construction: i32,
) -> String {
    format!(
        "CREATE INDEX {index_name} ON {sidecar}\n         USING hnsw (embedding vector_ip_ops)\n         WITH (m = {m}, ef_construction = {ef_construction})"
    )
}

/// `CREATE INDEX ... USING ivfflat (embedding vector_ip_ops) WITH (lists)`
pub fn build_pgvector_ivfflat_index_sql(sidecar: &str, index_name: &str, lists: i32) -> String {
    format!(
        "CREATE INDEX {index_name} ON {sidecar}\n         USING ivfflat (embedding vector_ip_ops)\n         WITH (lists = {lists})"
    )
}

/// `CREATE INDEX ... USING diskann (embedding vector_ip_ops) WITH (...)`
pub fn build_vectorscale_index_sql(
    sidecar: &str,
    index_name: &str,
    num_neighbors: i32,
    search_list_size: i32,
    max_alpha: f32,
    storage_layout: &str,
) -> String {
    format!(
        "CREATE INDEX {index_name} ON {sidecar}\n         USING diskann (embedding vector_ip_ops)\n         WITH (num_neighbors = {num_neighbors}, search_list_size = {search_list_size}, max_alpha = {max_alpha}, storage_layout = {storage_layout})"
    )
}

/// Default IVF list count: `ceil(sqrt(row_count))`, clamped to >= 1.
pub fn default_lists_for_rows(rows: i64) -> i32 {
    let rows = rows.max(1) as f64;
    rows.sqrt().ceil().max(1.0) as i32
}

#[derive(Clone, Copy)]
struct BuildConfig<'a> {
    engine: ComparatorEngine,
    lists: i32,
    hnsw_m: i32,
    hnsw_ef_construction: i32,
    num_neighbors: i32,
    build_search_list_size: i32,
    max_alpha: f32,
    storage_layout: &'a str,
    maintenance_work_mem: &'a str,
}

impl BuildConfig<'_> {
    fn create_index_sql(&self, sidecar: &str, index_name: &str) -> String {
        match self.engine {
            ComparatorEngine::Vchord => {
                build_vchord_create_index_sql(sidecar, index_name, self.lists)
            }
            ComparatorEngine::PgvectorHnsw => build_pgvector_hnsw_index_sql(
                sidecar,
                index_name,
                self.hnsw_m,
                self.hnsw_ef_construction,
            ),
            ComparatorEngine::PgvectorIvfflat => {
                build_pgvector_ivfflat_index_sql(sidecar, index_name, self.lists)
            }
            ComparatorEngine::Pgvectorscale => build_vectorscale_index_sql(
                sidecar,
                index_name,
                self.num_neighbors,
                self.build_search_list_size,
                self.max_alpha,
                self.storage_layout,
            ),
        }
    }
}

struct BuildSummary {
    build_seconds: Option<f64>,
    index_bytes: i64,
}

async fn ensure_sidecar(
    client: &Client,
    corpus_table: &str,
    sidecar: &str,
    index_name: &str,
    dim: usize,
    config: BuildConfig<'_>,
    rebuild: bool,
) -> Result<BuildSummary> {
    if rebuild {
        crate::ecaz_eprintln!("[comparator] --rebuild: dropping {sidecar} (and dependent index)");
        client
            .batch_execute(&format!("DROP TABLE IF EXISTS {sidecar} CASCADE"))
            .await
            .wrap_err("dropping comparator sidecar")?;
    }

    client
        .batch_execute(&build_sidecar_ddl(sidecar, dim))
        .await
        .wrap_err("creating comparator sidecar")?;

    let existing = count_rows(client, sidecar).await?;
    let corpus_rows = count_rows(client, corpus_table).await?;
    if existing < corpus_rows {
        crate::ecaz_eprintln!(
            "[comparator] populating {sidecar}: {} rows missing from {corpus_rows}",
            corpus_rows - existing
        );
        client
            .batch_execute(&build_populate_sql(corpus_table, sidecar, dim))
            .await
            .wrap_err("populating comparator sidecar")?;
    }

    let mut build_seconds = None;
    if !psql::relation_exists(client, index_name, 'i').await? {
        crate::ecaz_eprintln!(
            "[comparator] SET maintenance_work_mem = '{}'",
            config.maintenance_work_mem
        );
        client
            .batch_execute(&format!(
                "SET maintenance_work_mem = '{}'",
                config.maintenance_work_mem
            ))
            .await
            .wrap_err("SET maintenance_work_mem")?;
        crate::ecaz_eprintln!(
            "[comparator] building {} index {index_name}",
            config.engine.label()
        );
        let t0 = Instant::now();
        client
            .batch_execute(&config.create_index_sql(sidecar, index_name))
            .await
            .wrap_err("creating comparator index")?;
        build_seconds = Some(t0.elapsed().as_secs_f64());
    }

    let index_bytes: i64 = client
        .query_one(
            &format!("SELECT pg_relation_size('{index_name}'::regclass)"),
            &[],
        )
        .await
        .wrap_err("reading comparator index size")?
        .get(0);
    Ok(BuildSummary {
        build_seconds,
        index_bytes,
    })
}

#[allow(clippy::too_many_arguments)]
async fn measure_engine(
    client: &Client,
    label: &str,
    sql: &str,
    queries: &ndarray::Array2<f32>,
    gt: &super::recall::GroundTruth,
    corpus_ids: &[i64],
    truth_ids: &[Vec<i64>],
    k: usize,
) -> Result<(f64, f64, LatencyStats)> {
    let stmt = client.prepare(sql).await.wrap_err("preparing KNN")?;
    let bar = crate::output::progress_bar(queries.nrows() as u64);
    bar.set_style(
        ProgressStyle::with_template("[comparator {msg}] {wide_bar} {pos}/{len} ({per_sec})")
            .unwrap(),
    );
    bar.set_message(label.to_owned());
    bar.enable_steady_tick(Duration::from_millis(250));

    let k_i64 = k as i64;
    let mut pred: Vec<Vec<i64>> = Vec::with_capacity(queries.nrows());
    let mut durations: Vec<Duration> = Vec::with_capacity(queries.nrows());
    for q in 0..queries.nrows() {
        let row_vec: Vec<f32> = queries.row(q).to_vec();
        let t0 = Instant::now();
        let result = client
            .query(&stmt, &[&row_vec, &k_i64])
            .await
            .wrap_err_with(|| format!("{label} KNN"))?;
        durations.push(t0.elapsed());
        pred.push(result.iter().map(|r| r.get::<_, i64>(0)).collect());
        bar.inc(1);
    }
    bar.finish_and_clear();

    let recall = recall_at_k(truth_ids, &pred, k);
    let ndcg = ndcg_at_k(&gt.scores, &pred, corpus_ids, &gt.all_scores, k);
    let stats = summarize(&durations);
    Ok((recall, ndcg, stats))
}

/// One Pareto cell for a single sweep value.
struct ParetoRow {
    engine: String,
    sweep: i32,
    recall: f64,
    ndcg: f64,
    stats: LatencyStats,
}

fn render_table(rows: &[ParetoRow]) -> String {
    let mut t = Table::new();
    t.load_preset(UTF8_FULL);
    t.set_header(vec![
        "engine", "sweep", "recall@k", "ndcg@k", "p50", "p95", "p99", "mean",
    ]);
    for r in rows {
        t.add_row(vec![
            Cell::new(&r.engine),
            Cell::new(r.sweep),
            Cell::new(format!("{:.4}", r.recall)),
            Cell::new(format!("{:.4}", r.ndcg)),
            Cell::new(format_ms(r.stats.p50)),
            Cell::new(format_ms(r.stats.p95)),
            Cell::new(format_ms(r.stats.p99)),
            Cell::new(format_ms(r.stats.mean)),
        ]);
    }
    t.to_string()
}

fn configured_engine_label(engine: &str, axis_label: &str, value: i32) -> String {
    format!("{engine}[{axis_label}={value}]")
}

fn format_ms(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms >= 10.0 {
        format!("{ms:.1} ms")
    } else {
        format!("{ms:.2} ms")
    }
}

fn extension_error(engine: ComparatorEngine) -> String {
    match engine {
        ComparatorEngine::Vchord => format!(
            "creating the {} extension; vchord requires it in \
             shared_preload_libraries and a PostgreSQL restart before \
             CREATE EXTENSION can succeed — run scripts/comparators/vchord/install.sh \
             (or the operator-equivalent) first",
            engine.extension()
        ),
        _ => format!("ensuring the {} extension", engine.extension()),
    }
}

fn validate_args(args: &ComparatorArgs) -> Result<()> {
    if args.sweep.iter().any(|v| *v <= 0) {
        return Err(eyre!("--sweep values must all be > 0"));
    }
    if let Some(lists) = args.lists {
        if lists <= 0 {
            return Err(eyre!("--lists must be > 0"));
        }
    }
    match args.engine {
        ComparatorEngine::PgvectorHnsw => {
            if args.m <= 0 {
                return Err(eyre!("--m must be > 0"));
            }
            if args.ef_construction <= 0 {
                return Err(eyre!("--ef-construction must be > 0"));
            }
        }
        ComparatorEngine::Pgvectorscale => {
            if args.num_neighbors <= 10 {
                return Err(eyre!("--num-neighbors must be > 10"));
            }
            if args.build_search_list_size <= 0 {
                return Err(eyre!("--build-search-list-size must be > 0"));
            }
            if args.max_alpha <= 0.0 {
                return Err(eyre!("--max-alpha must be > 0"));
            }
            validate_storage_layout(&args.storage_layout)?;
        }
        _ => {}
    }
    if args.engine.uses_lists() {
        // lists already validated above when explicitly set.
    }
    validate_postgres_memory_value(&args.maintenance_work_mem)?;
    Ok(())
}

fn validate_storage_layout(value: &str) -> Result<()> {
    match value {
        "memory_optimized" | "plain" => Ok(()),
        other => Err(eyre!(
            "unsupported --storage-layout {other:?}; expected memory_optimized or plain"
        )),
    }
}

fn validate_postgres_memory_value(value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(eyre!("--maintenance-work-mem cannot be empty"));
    }
    let digits = value.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 || digits == value.len() {
        return Err(eyre!(
            "--maintenance-work-mem must look like 256MB, 1GB, or 65536kB"
        ));
    }
    let unit = &value[digits..];
    if !matches!(unit, "B" | "kB" | "MB" | "GB" | "TB") {
        return Err(eyre!(
            "--maintenance-work-mem unit must be B, kB, MB, GB, or TB"
        ));
    }
    Ok(())
}

async fn read_dim(client: &Client, corpus_table: &str) -> Result<usize> {
    let row = client
        .query_opt(
            &format!("SELECT array_length(source, 1) FROM {corpus_table} LIMIT 1"),
            &[],
        )
        .await
        .wrap_err("reading corpus dim")?
        .ok_or_else(|| eyre!("{corpus_table} is empty; cannot infer dim"))?;
    let dim: i32 = row.get(0);
    if dim <= 0 {
        return Err(eyre!("invalid dim {dim} in {corpus_table}"));
    }
    Ok(dim as usize)
}

async fn count_rows(client: &Client, table: &str) -> Result<i64> {
    let row = client
        .query_one(&format!("SELECT count(*) FROM {table}"), &[])
        .await
        .wrap_err_with(|| format!("counting rows in {table}"))?;
    Ok(row.get(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidecar_and_index_names_are_engine_scoped() {
        assert_eq!(
            sidecar_name("dbpedia_10k", ComparatorEngine::Vchord),
            "dbpedia_10k_corpus_vchord"
        );
        assert_eq!(
            index_name("dbpedia_10k", ComparatorEngine::PgvectorIvfflat),
            "dbpedia_10k_corpus_pgvector_ivfflat_idx"
        );
    }

    #[test]
    fn engine_query_gucs_match_each_engine() {
        assert_eq!(ComparatorEngine::Vchord.query_guc(), "vchordrq.probes");
        assert_eq!(ComparatorEngine::PgvectorHnsw.query_guc(), "hnsw.ef_search");
        assert_eq!(
            ComparatorEngine::PgvectorIvfflat.query_guc(),
            "ivfflat.probes"
        );
        assert_eq!(
            ComparatorEngine::Pgvectorscale.query_guc(),
            "diskann.query_search_list_size"
        );
    }

    #[test]
    fn sidecar_ddl_uses_if_not_exists_and_vector_dim() {
        let sql = build_sidecar_ddl("t_corpus_vchord", 1536);
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS t_corpus_vchord"));
        assert!(sql.contains("embedding vector(1536) NOT NULL"));
        assert!(sql.contains("id bigint PRIMARY KEY"));
    }

    #[test]
    fn populate_sql_is_idempotent_via_on_conflict() {
        let sql = build_populate_sql("t_corpus", "t_corpus_vchord", 1536);
        assert!(sql.contains("INSERT INTO t_corpus_vchord"));
        assert!(sql.contains("FROM t_corpus"));
        assert!(sql.contains("source::vector(1536)"));
        assert!(sql.contains("ON CONFLICT (id) DO NOTHING"));
    }

    #[test]
    fn knn_sql_uses_ip_operator_and_bind_cast() {
        let sql = build_knn_sql("t_corpus_vchord", 1536);
        assert!(sql.contains("FROM t_corpus_vchord"));
        assert!(sql.contains("<#>"));
        assert!(!sql.contains("pg_catalog"), "got: {sql}");
        assert!(sql.contains("$1::real[]::vector(1536)"));
        assert!(sql.contains("LIMIT $2"));
    }

    #[test]
    fn vchord_index_sql_pins_rabitq_residual_options() {
        let sql = build_vchord_create_index_sql("t_corpus_vchord", "t_vchord_idx", 320);
        assert!(sql.contains("USING vchordrq (embedding vector_ip_ops)"));
        assert!(sql.contains("residual_quantization = true"));
        assert!(sql.contains("[build.internal]"));
        assert!(sql.contains("lists = [320]"));
        assert!(sql.contains("$vco$"));
    }

    #[test]
    fn pgvector_hnsw_index_sql_pins_ip_ops_and_reloptions() {
        let sql = build_pgvector_hnsw_index_sql("t_corpus_pgvector_hnsw", "t_idx", 16, 128);
        assert!(sql.contains("USING hnsw (embedding vector_ip_ops)"));
        assert!(sql.contains("m = 16"));
        assert!(sql.contains("ef_construction = 128"));
    }

    #[test]
    fn pgvector_ivfflat_index_sql_pins_ip_ops_and_lists() {
        let sql = build_pgvector_ivfflat_index_sql("t_corpus_pgvector_ivfflat", "t_idx", 224);
        assert!(sql.contains("USING ivfflat (embedding vector_ip_ops)"));
        assert!(sql.contains("lists = 224"));
    }

    #[test]
    fn vectorscale_index_sql_uses_diskann_ip_ops_and_reloptions() {
        let sql = build_vectorscale_index_sql(
            "t_corpus_pgvectorscale",
            "t_idx",
            32,
            100,
            1.2,
            "memory_optimized",
        );
        assert!(sql.contains("USING diskann (embedding vector_ip_ops)"));
        assert!(sql.contains("num_neighbors = 32"));
        assert!(sql.contains("search_list_size = 100"));
        assert!(sql.contains("max_alpha = 1.2"));
        assert!(sql.contains("storage_layout = memory_optimized"));
    }

    #[test]
    fn default_lists_is_ceil_sqrt_rows() {
        assert_eq!(default_lists_for_rows(50_000), 224); // ceil(223.6) = 224
        assert_eq!(default_lists_for_rows(100_000), 317); // ceil(316.2) = 317
        assert_eq!(default_lists_for_rows(1_000_000), 1000);
        assert_eq!(default_lists_for_rows(0), 1);
        assert_eq!(default_lists_for_rows(1), 1);
    }

    #[test]
    fn configured_engine_label_is_self_describing() {
        assert_eq!(
            configured_engine_label("vchord", "probes", 16),
            "vchord[probes=16]"
        );
        assert_eq!(
            configured_engine_label("pgvector_hnsw", "ef_search", 100),
            "pgvector_hnsw[ef_search=100]"
        );
    }

    #[test]
    fn storage_layout_validation_accepts_known_values() {
        validate_storage_layout("memory_optimized").unwrap();
        validate_storage_layout("plain").unwrap();
        assert!(validate_storage_layout("other").is_err());
    }

    #[test]
    fn memory_value_validation_matches_postgres_units() {
        validate_postgres_memory_value("4GB").unwrap();
        validate_postgres_memory_value("65536kB").unwrap();
        assert!(validate_postgres_memory_value("").is_err());
        assert!(validate_postgres_memory_value("256").is_err());
        assert!(validate_postgres_memory_value("256ZB").is_err());
    }
}
