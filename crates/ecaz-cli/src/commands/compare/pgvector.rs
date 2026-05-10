//! `ecaz compare pgvector` — side-by-side recall + latency against pgvector.
//!
//! # Flow
//!
//! 1. Ensure the `vector` extension is installed.
//! 2. Materialize a `<prefix>_corpus_pgvector` sidecar `(id bigint, embedding
//!    vector(dim))` sourced from the ecaz `<prefix>_corpus.source` column
//!    (idempotent unless `--rebuild`).
//! 3. Build a pgvector HNSW or IVFFlat index on the sidecar with the requested
//!    reloptions (idempotent by index name unless `--rebuild`).
//! 4. Compute ground truth once with the brute-force matmul helper already
//!    used by `bench recall`.
//! 5. Run KNN against ecaz and pgvector at the configured tuning points,
//!    scoring recall@k + capturing per-query latency.
//! 6. Render a comparison table with absolute numbers and a delta row.
//!
//! # Purity boundary
//!
//! SQL builders, comparison-row construction, and `pct_delta` / `format_pct`
//! are pure functions with unit tests. The orchestration shell is a thin
//! tokio-postgres driver on top.

use clap::{Args, ValueEnum};
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use indicatif::ProgressStyle;
use std::time::{Duration, Instant};
use tokio_postgres::Client;

use crate::profiles;
use crate::psql::{self, ConnectionOptions};

use super::super::bench::latency::{summarize, LatencyStats};
use super::super::bench::recall::{
    brute_force_top_k, build_knn_sql, map_indices_to_ids, ndcg_at_k, recall_at_k,
};

#[derive(Args, Debug)]
pub struct PgvectorArgs {
    /// Prefix identifying the ecaz corpus (as loaded by `ecaz corpus load`).
    #[arg(long)]
    pub prefix: String,
    /// Ecaz profile to compare against pgvector.
    #[arg(long, default_value = "ec_hnsw")]
    pub profile: String,
    /// k for recall@k / latency measurement.
    #[arg(long, default_value_t = 10)]
    pub k: usize,
    /// Ecaz-side tuning value for the selected profile's sweep axis
    /// (`ef_search` for HNSW, `list_size` for DiskANN).
    #[arg(long = "ecaz-sweep", alias = "ecaz-ef-search", default_value_t = 100)]
    pub ecaz_sweep: i32,
    /// Matched sweep values for ecaz and pgvector `hnsw.ef_search`.
    #[arg(long, value_delimiter = ',')]
    pub sweep: Vec<i32>,
    /// pgvector access method to build on the sidecar.
    #[arg(long = "pgvector-am", default_value_t = PgvectorIndexKind::Hnsw)]
    pub pgvector_am: PgvectorIndexKind,
    /// pgvector-side `hnsw.ef_search` for the timed queries.
    #[arg(long, default_value_t = 100)]
    pub pgvector_ef_search: i32,
    /// pgvector HNSW build `m`.
    #[arg(long, default_value_t = 16)]
    pub pgvector_m: i32,
    /// pgvector HNSW build `ef_construction`.
    #[arg(long, default_value_t = 128)]
    pub pgvector_ef_construction: i32,
    /// pgvector IVFFlat build `lists`.
    #[arg(long, default_value_t = 128)]
    pub pgvector_lists: i32,
    /// pgvector IVFFlat query `ivfflat.probes` for a single-point comparison.
    #[arg(long, default_value_t = 10)]
    pub pgvector_probes: i32,
    /// Session maintenance_work_mem used while building the pgvector sidecar
    /// index, for example `256MB`.
    #[arg(long)]
    pub pgvector_maintenance_work_mem: Option<String>,
    /// IVF-only: ecaz session override for heap-f32 rerank frontier width.
    /// Use -1 for the index reloption, 0 for the full probed frontier.
    #[arg(long)]
    pub rerank_width: Option<i32>,
    /// Extra ecaz session GUC to set before the sweep, in NAME=VALUE form.
    #[arg(long = "set-guc")]
    pub set_gucs: Vec<String>,
    /// Ecaz session GUC whose value should be set to each sweep point.
    #[arg(long = "set-guc-from-sweep")]
    pub set_gucs_from_sweep: Vec<String>,
    /// Cap the query set (default: all rows).
    #[arg(long)]
    pub queries_limit: Option<usize>,
    /// Drop + rebuild the pgvector sidecar table + index before measuring.
    #[arg(long, default_value_t = false)]
    pub rebuild: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum PgvectorIndexKind {
    Hnsw,
    Ivfflat,
}

impl PgvectorIndexKind {
    fn axis_label(self) -> &'static str {
        match self {
            Self::Hnsw => "ef_search",
            Self::Ivfflat => "probes",
        }
    }

    fn query_guc(self) -> &'static str {
        match self {
            Self::Hnsw => "hnsw.ef_search",
            Self::Ivfflat => "ivfflat.probes",
        }
    }

    fn default_query_value(self, args: &PgvectorArgs) -> i32 {
        match self {
            Self::Hnsw => args.pgvector_ef_search,
            Self::Ivfflat => args.pgvector_probes,
        }
    }

    fn engine_label(self) -> &'static str {
        match self {
            Self::Hnsw => "pgvector",
            Self::Ivfflat => "pgvector_ivfflat",
        }
    }
}

impl std::fmt::Display for PgvectorIndexKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hnsw => f.write_str("hnsw"),
            Self::Ivfflat => f.write_str("ivfflat"),
        }
    }
}

pub async fn run(conn: &ConnectionOptions, args: PgvectorArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    if args.k == 0 {
        return Err(eyre!("--k must be >= 1"));
    }
    validate_pgvector_args(&args)?;
    let profile = profiles::resolve(&args.profile).ok_or_else(|| {
        eyre!(
            "unknown profile {:?}; try {}",
            args.profile,
            profiles::names().join(", ")
        )
    })?;
    let ecaz_guc = profile
        .ef_search_guc
        .ok_or_else(|| eyre!("profile {:?} has no tuning GUC to set", profile.name))?;
    validate_rerank_width_arg(profile, args.rerank_width)?;
    let set_gucs = args
        .set_gucs
        .iter()
        .map(|raw| psql::parse_session_setting(raw))
        .collect::<Result<Vec<_>>>()?;
    for name in &args.set_gucs_from_sweep {
        psql::validate_session_guc_name(name)?;
    }
    let ecaz_sweep_values = if args.sweep.is_empty() {
        vec![args.ecaz_sweep]
    } else {
        args.sweep.clone()
    };
    let pgvector_sweep_values = if args.sweep.is_empty() {
        vec![args.pgvector_am.default_query_value(&args)]
    } else {
        args.sweep.clone()
    };

    let corpus_table = format!("{}_corpus", args.prefix);
    let queries_table = format!("{}_queries", args.prefix);
    let sidecar_table = pgvector_sidecar_name(&args.prefix);
    let sidecar_index = pgvector_index_name(&args.prefix, args.pgvector_am);

    let client = psql::connect(conn).await?;

    if !psql::relation_exists(&client, &corpus_table, 'r').await? {
        return Err(eyre!(
            "no ecaz corpus table {corpus_table} in this database"
        ));
    }
    if !psql::relation_exists(&client, &queries_table, 'r').await? {
        return Err(eyre!("no queries table {queries_table} in this database"));
    }
    if psql::index_count_with_am(&client, &corpus_table, profile.access_method).await? == 0 {
        return Err(eyre!(
            "{} on {:?}",
            crate::commands::bench::missing_am_error(profile, profile.access_method),
            corpus_table
        ));
    }

    client
        .batch_execute("CREATE EXTENSION IF NOT EXISTS vector")
        .await
        .wrap_err("ensuring pgvector extension")?;

    let dim = read_dim(&client, &corpus_table).await?;
    ensure_pgvector_sidecar(
        &client,
        &corpus_table,
        &sidecar_table,
        &sidecar_index,
        dim,
        PgvectorIndexConfig {
            kind: args.pgvector_am,
            hnsw_m: args.pgvector_m,
            hnsw_ef_construction: args.pgvector_ef_construction,
            ivfflat_lists: args.pgvector_lists,
            maintenance_work_mem: args.pgvector_maintenance_work_mem.as_deref(),
        },
        args.rebuild,
    )
    .await?;

    crate::ecaz_eprintln!("[compare] fetching corpus + queries for ground truth ...");
    let (corpus_ids, corpus) =
        super::super::bench::recall::fetch_sources_public(&client, &corpus_table, None).await?;
    let (_, queries) = super::super::bench::recall::fetch_sources_public(
        &client,
        &queries_table,
        args.queries_limit,
    )
    .await?;
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

    crate::ecaz_eprintln!("[compare] computing ground truth ...");
    let t0 = Instant::now();
    let gt = brute_force_top_k(&corpus, &queries, args.k);
    crate::ecaz_eprintln!("[compare] ground truth in {:.2?}", t0.elapsed());
    psql::prefer_ordered_ann_path(&client).await?;
    psql::apply_session_settings(&client, &set_gucs).await?;
    let truth_ids = map_indices_to_ids(&gt.indices, &corpus_ids);
    let ecaz_sql = build_knn_sql(profile, &corpus_table);
    let pgv_sql = build_pgvector_knn_sql(&sidecar_table, dim);
    let mut rows = Vec::with_capacity(ecaz_sweep_values.len() * 2);
    for (ecaz_value, pgvector_value) in ecaz_sweep_values.into_iter().zip(pgvector_sweep_values) {
        let ecaz_label =
            configured_engine_label(profile.name, profile.sweep_axis_label(), ecaz_value);
        client
            .batch_execute(&format!("SET {ecaz_guc} = {ecaz_value}"))
            .await
            .wrap_err_with(|| format!("SET {ecaz_guc}"))?;
        if let Some(rerank_width) = args.rerank_width {
            client
                .batch_execute(&format!("SET ec_ivf.rerank_width = {rerank_width}"))
                .await
                .wrap_err_with(|| format!("SET ec_ivf.rerank_width = {rerank_width}"))?;
        }
        let sweep_settings = args
            .set_gucs_from_sweep
            .iter()
            .map(|name| psql::session_setting_from_sweep(name, ecaz_value))
            .collect::<Result<Vec<_>>>()?;
        psql::apply_session_settings(&client, &sweep_settings).await?;
        let (ecaz_recall, ecaz_ndcg, ecaz_stats) = measure_engine(
            &client,
            &ecaz_label,
            &ecaz_sql,
            &queries,
            &gt,
            &corpus_ids,
            &truth_ids,
            args.k,
            EngineBinds::Ecaz,
        )
        .await?;
        rows.push(ComparisonRow::with_sweep(
            &ecaz_label,
            ecaz_value,
            ecaz_recall,
            ecaz_ndcg,
            ecaz_stats,
        ));

        client
            .batch_execute(&format!(
                "SET {} = {pgvector_value}",
                args.pgvector_am.query_guc()
            ))
            .await
            .wrap_err_with(|| format!("SET {}", args.pgvector_am.query_guc()))?;
        let pgv_label = configured_engine_label(
            args.pgvector_am.engine_label(),
            args.pgvector_am.axis_label(),
            pgvector_value,
        );
        let (pgv_recall, pgv_ndcg, pgv_stats) = measure_engine(
            &client,
            &pgv_label,
            &pgv_sql,
            &queries,
            &gt,
            &corpus_ids,
            &truth_ids,
            args.k,
            EngineBinds::Pgvector,
        )
        .await?;
        rows.push(ComparisonRow::with_sweep(
            &pgv_label,
            pgvector_value,
            pgv_recall,
            pgv_ndcg,
            pgv_stats,
        ));
    }
    print_comparison(&rows);
    Ok(())
}

/// Name of the pgvector sidecar table for a given ecaz prefix. Kept as a
/// free function so tests can pin the naming rule.
pub fn pgvector_sidecar_name(prefix: &str) -> String {
    format!("{prefix}_corpus_pgvector")
}

/// Name of the pgvector index built on the sidecar.
pub fn pgvector_index_name(prefix: &str, kind: PgvectorIndexKind) -> String {
    match kind {
        PgvectorIndexKind::Hnsw => format!("{prefix}_corpus_pgvector_hnsw_idx"),
        PgvectorIndexKind::Ivfflat => format!("{prefix}_corpus_pgvector_ivfflat_idx"),
    }
}

/// DDL to create the pgvector sidecar. Creation is idempotent; the caller
/// separately decides whether to re-populate it.
pub fn build_pgvector_sidecar_ddl(sidecar: &str, dim: usize) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {sidecar} (\n    id bigint PRIMARY KEY,\n    embedding vector({dim}) NOT NULL\n)"
    )
}

/// Idempotent populate: insert rows from the ecaz corpus that are not yet
/// present. The ecaz `source` column is `real[]`, which pgvector casts from
/// natively via `::vector(dim)`.
pub fn build_pgvector_populate_sql(corpus_table: &str, sidecar: &str, dim: usize) -> String {
    format!(
        "INSERT INTO {sidecar} (id, embedding)\n         SELECT id, source::vector({dim}) FROM {corpus_table}\n         ON CONFLICT (id) DO NOTHING"
    )
}

/// `CREATE INDEX ... USING hnsw (embedding vector_ip_ops) WITH (m=..., ef_construction=...)`
pub fn build_pgvector_create_index_sql(
    sidecar: &str,
    index_name: &str,
    m: i32,
    ef_construction: i32,
) -> String {
    format!(
        "CREATE INDEX {index_name} ON {sidecar}\n         USING hnsw (embedding vector_ip_ops)\n         WITH (m = {m}, ef_construction = {ef_construction})"
    )
}

/// `CREATE INDEX ... USING ivfflat (embedding vector_ip_ops) WITH (lists=...)`
pub fn build_pgvector_create_ivfflat_index_sql(
    sidecar: &str,
    index_name: &str,
    lists: i32,
) -> String {
    format!(
        "CREATE INDEX {index_name} ON {sidecar}\n         USING ivfflat (embedding vector_ip_ops)\n         WITH (lists = {lists})"
    )
}

/// KNN template for the pgvector sidecar: binds are `($1::real[], $2::bigint)`
/// = (query_source, k). Uses the inner-product operator to match ecaz's
/// `encode_to_ecvector`-preserving IP semantics.
pub fn build_pgvector_knn_sql(sidecar: &str, dim: usize) -> String {
    format!(
        "SELECT id FROM {sidecar} \
         ORDER BY embedding <#> \
         $1::real[]::vector({dim}) \
         LIMIT $2"
    )
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

#[derive(Clone, Copy)]
struct PgvectorIndexConfig<'a> {
    kind: PgvectorIndexKind,
    hnsw_m: i32,
    hnsw_ef_construction: i32,
    ivfflat_lists: i32,
    maintenance_work_mem: Option<&'a str>,
}

#[allow(clippy::too_many_arguments)]
async fn ensure_pgvector_sidecar(
    client: &Client,
    corpus_table: &str,
    sidecar: &str,
    index_name: &str,
    dim: usize,
    index_config: PgvectorIndexConfig<'_>,
    rebuild: bool,
) -> Result<()> {
    if rebuild {
        crate::ecaz_eprintln!("[compare] --rebuild: dropping {sidecar} (and dependent index)");
        client
            .batch_execute(&format!("DROP TABLE IF EXISTS {sidecar} CASCADE"))
            .await
            .wrap_err("dropping pgvector sidecar")?;
    }

    client
        .batch_execute(&build_pgvector_sidecar_ddl(sidecar, dim))
        .await
        .wrap_err("creating pgvector sidecar")?;

    let existing: i64 = client
        .query_one(&format!("SELECT count(*) FROM {sidecar}"), &[])
        .await?
        .get(0);
    let corpus_rows: i64 = client
        .query_one(&format!("SELECT count(*) FROM {corpus_table}"), &[])
        .await?
        .get(0);
    if existing < corpus_rows {
        crate::ecaz_eprintln!(
            "[compare] populating {sidecar}: {} rows missing from {corpus_rows}",
            corpus_rows - existing
        );
        client
            .batch_execute(&build_pgvector_populate_sql(corpus_table, sidecar, dim))
            .await
            .wrap_err("populating pgvector sidecar")?;
    }

    if !psql::relation_exists(client, index_name, 'i').await? {
        if let Some(memory) = index_config.maintenance_work_mem {
            crate::ecaz_eprintln!("[compare] SET maintenance_work_mem = '{memory}'");
            client
                .batch_execute(&format!("SET maintenance_work_mem = '{memory}'"))
                .await
                .wrap_err_with(|| format!("SET maintenance_work_mem = {memory}"))?;
        }
        crate::ecaz_eprintln!(
            "[compare] building pgvector {:?} index {index_name}",
            index_config.kind
        );
        let t0 = Instant::now();
        let create_index_sql = match index_config.kind {
            PgvectorIndexKind::Hnsw => build_pgvector_create_index_sql(
                sidecar,
                index_name,
                index_config.hnsw_m,
                index_config.hnsw_ef_construction,
            ),
            PgvectorIndexKind::Ivfflat => build_pgvector_create_ivfflat_index_sql(
                sidecar,
                index_name,
                index_config.ivfflat_lists,
            ),
        };
        client
            .batch_execute(&create_index_sql)
            .await
            .wrap_err("creating pgvector index")?;
        crate::ecaz_eprintln!("[compare] built {index_name} in {:.2?}", t0.elapsed());
    }
    let size: i64 = client
        .query_one(
            &format!("SELECT pg_relation_size('{index_name}'::regclass)"),
            &[],
        )
        .await
        .wrap_err("reading pgvector index size")?
        .get(0);
    crate::ecaz_eprintln!("[compare] {index_name} pg_relation_size={size} bytes");
    Ok(())
}

fn validate_pgvector_args(args: &PgvectorArgs) -> Result<()> {
    if args.pgvector_ef_search <= 0 {
        return Err(eyre!("--pgvector-ef-search must be > 0"));
    }
    if args.pgvector_m <= 0 {
        return Err(eyre!("--pgvector-m must be > 0"));
    }
    if args.pgvector_ef_construction <= 0 {
        return Err(eyre!("--pgvector-ef-construction must be > 0"));
    }
    if args.pgvector_lists <= 0 {
        return Err(eyre!("--pgvector-lists must be > 0"));
    }
    if args.pgvector_probes <= 0 {
        return Err(eyre!("--pgvector-probes must be > 0"));
    }
    if let Some(value) = args.pgvector_maintenance_work_mem.as_deref() {
        validate_postgres_memory_value(value)?;
    }
    Ok(())
}

fn validate_rerank_width_arg(
    profile: &'static profiles::IndexProfile,
    rerank_width: Option<i32>,
) -> Result<()> {
    let Some(value) = rerank_width else {
        return Ok(());
    };
    if profile.name != "ec_ivf" {
        return Err(eyre!(
            "--rerank-width is only supported with --profile ec_ivf"
        ));
    }
    if value < -1 {
        return Err(eyre!("--rerank-width must be >= -1"));
    }
    Ok(())
}

fn validate_postgres_memory_value(value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(eyre!("--pgvector-maintenance-work-mem cannot be empty"));
    }
    let digits = value.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 || digits == value.len() {
        return Err(eyre!(
            "--pgvector-maintenance-work-mem must look like 256MB, 1GB, or 65536kB"
        ));
    }
    let unit = &value[digits..];
    if !matches!(unit, "B" | "kB" | "MB" | "GB" | "TB") {
        return Err(eyre!(
            "--pgvector-maintenance-work-mem unit must be B, kB, MB, GB, or TB"
        ));
    }
    Ok(())
}

enum EngineBinds {
    Ecaz,
    Pgvector,
}

#[allow(clippy::too_many_arguments)]
async fn measure_engine(
    client: &Client,
    label: &str,
    sql: &str,
    queries: &ndarray::Array2<f32>,
    gt: &super::super::bench::recall::GroundTruth,
    corpus_ids: &[i64],
    truth_ids: &[Vec<i64>],
    k: usize,
    binds: EngineBinds,
) -> Result<(f64, f64, LatencyStats)> {
    let stmt = client.prepare(sql).await.wrap_err("preparing KNN")?;
    let bar = crate::output::progress_bar(queries.nrows() as u64);
    bar.set_style(
        ProgressStyle::with_template("[compare {msg}] {wide_bar} {pos}/{len} ({per_sec})").unwrap(),
    );
    bar.set_message(label.to_owned());
    bar.enable_steady_tick(Duration::from_millis(250));

    let k_i64 = k as i64;
    let mut pred: Vec<Vec<i64>> = Vec::with_capacity(queries.nrows());
    let mut durations: Vec<Duration> = Vec::with_capacity(queries.nrows());
    for q in 0..queries.nrows() {
        let row_vec: Vec<f32> = queries.row(q).to_vec();
        let t0 = Instant::now();
        let rows = match &binds {
            EngineBinds::Ecaz => client.query(&stmt, &[&row_vec, &k_i64]).await,
            EngineBinds::Pgvector => client.query(&stmt, &[&row_vec, &k_i64]).await,
        }
        .wrap_err_with(|| format!("{label} KNN"))?;
        durations.push(t0.elapsed());
        pred.push(rows.iter().map(|r| r.get::<_, i64>(0)).collect());
        bar.inc(1);
    }
    bar.finish_and_clear();

    let recall = recall_at_k(truth_ids, &pred, k);
    let ndcg = ndcg_at_k(&gt.scores, &pred, corpus_ids, &gt.all_scores, k);
    let stats = summarize(&durations);
    Ok((recall, ndcg, stats))
}

/// One row of the cross-engine comparison table.
#[derive(Debug, Clone)]
pub struct ComparisonRow {
    pub engine: String,
    pub sweep: Option<i32>,
    pub recall: f64,
    pub ndcg: f64,
    pub stats: LatencyStats,
}

impl ComparisonRow {
    pub fn with_sweep(
        engine: &str,
        sweep: i32,
        recall: f64,
        ndcg: f64,
        stats: LatencyStats,
    ) -> Self {
        Self {
            engine: engine.to_owned(),
            sweep: Some(sweep),
            recall,
            ndcg,
            stats,
        }
    }
}

fn print_comparison(rows: &[ComparisonRow]) {
    let mut t = Table::new();
    t.load_preset(UTF8_FULL);
    let include_sweep = rows.iter().any(|row| row.sweep.is_some());
    if include_sweep {
        t.set_header(vec![
            "engine", "sweep", "recall@k", "ndcg@k", "p50", "p95", "p99", "mean",
        ]);
    } else {
        t.set_header(vec![
            "engine", "recall@k", "ndcg@k", "p50", "p95", "p99", "mean",
        ]);
    }
    for r in rows {
        let mut cells = vec![Cell::new(&r.engine)];
        if include_sweep {
            cells.push(Cell::new(
                r.sweep
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "-".into()),
            ));
        }
        cells.extend([
            Cell::new(format!("{:.4}", r.recall)),
            Cell::new(format!("{:.4}", r.ndcg)),
            Cell::new(format_ms(r.stats.p50)),
            Cell::new(format_ms(r.stats.p95)),
            Cell::new(format_ms(r.stats.p99)),
            Cell::new(format_ms(r.stats.mean)),
        ]);
        t.add_row(cells);
    }
    if rows.len() == 2 && !include_sweep {
        let a = &rows[0];
        let b = &rows[1];
        t.add_row(vec![
            Cell::new(format!("Δ ({} vs {})", b.engine, a.engine)),
            Cell::new(format_pct_delta(a.recall, b.recall)),
            Cell::new(format_pct_delta(a.ndcg, b.ndcg)),
            Cell::new(format_pct_delta_ms(a.stats.p50, b.stats.p50)),
            Cell::new(format_pct_delta_ms(a.stats.p95, b.stats.p95)),
            Cell::new(format_pct_delta_ms(a.stats.p99, b.stats.p99)),
            Cell::new(format_pct_delta_ms(a.stats.mean, b.stats.mean)),
        ]);
    }
    crate::ecaz_println!("{t}");
}

/// Percent change from `base` to `other`. Returns `None` when `base` is
/// zero (or non-finite) so the caller renders `n/a` rather than Inf/NaN.
pub fn pct_delta(base: f64, other: f64) -> Option<f64> {
    if !base.is_finite() || !other.is_finite() || base == 0.0 {
        return None;
    }
    Some((other - base) / base * 100.0)
}

/// Render `pct_delta` with a leading sign so readers can tell better/worse
/// at a glance; `n/a` when the base is zero.
pub fn format_pct_delta(base: f64, other: f64) -> String {
    match pct_delta(base, other) {
        Some(p) => format!("{p:+.1}%"),
        None => "n/a".to_owned(),
    }
}

fn format_pct_delta_ms(base: Duration, other: Duration) -> String {
    format_pct_delta(base.as_secs_f64() * 1000.0, other.as_secs_f64() * 1000.0)
}

fn format_ms(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms >= 10.0 {
        format!("{ms:.1} ms")
    } else {
        format!("{ms:.2} ms")
    }
}

fn configured_engine_label(engine: &str, axis_label: &str, value: i32) -> String {
    format!("{engine}[{axis_label}={value}]")
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Command, FromArgMatches};

    // --- name helpers ---

    #[test]
    fn pgvector_sidecar_name_is_suffixed_from_prefix() {
        assert_eq!(
            pgvector_sidecar_name("dbpedia_10k"),
            "dbpedia_10k_corpus_pgvector"
        );
    }

    #[test]
    fn pgvector_index_name_is_sidecar_scoped() {
        assert_eq!(
            pgvector_index_name("dbpedia_10k", PgvectorIndexKind::Hnsw),
            "dbpedia_10k_corpus_pgvector_hnsw_idx"
        );
        assert_eq!(
            pgvector_index_name("dbpedia_10k", PgvectorIndexKind::Ivfflat),
            "dbpedia_10k_corpus_pgvector_ivfflat_idx"
        );
    }

    // --- SQL builders ---

    #[test]
    fn sidecar_ddl_uses_if_not_exists_and_vector_dim() {
        let sql = build_pgvector_sidecar_ddl("t_corpus_pgvector", 1536);
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS t_corpus_pgvector"));
        assert!(sql.contains("embedding vector(1536) NOT NULL"));
        assert!(sql.contains("id bigint PRIMARY KEY"));
    }

    #[test]
    fn populate_sql_is_idempotent_via_on_conflict() {
        let sql = build_pgvector_populate_sql("t_corpus", "t_corpus_pgvector", 1536);
        assert!(sql.contains("INSERT INTO t_corpus_pgvector"));
        assert!(sql.contains("FROM t_corpus"));
        assert!(sql.contains("source::vector(1536)"));
        assert!(sql.contains("ON CONFLICT (id) DO NOTHING"));
    }

    #[test]
    fn create_index_sql_pins_hnsw_ip_ops_and_reloptions() {
        let sql = build_pgvector_create_index_sql("t_corpus_pgvector", "t_pgv_idx", 16, 128);
        assert!(sql.contains("USING hnsw (embedding vector_ip_ops)"));
        assert!(sql.contains("m = 16"));
        assert!(sql.contains("ef_construction = 128"));
    }

    #[test]
    fn create_ivfflat_index_sql_pins_ip_ops_and_lists() {
        let sql = build_pgvector_create_ivfflat_index_sql("t_corpus_pgvector", "t_pgv_idx", 128);
        assert!(sql.contains("USING ivfflat (embedding vector_ip_ops)"));
        assert!(sql.contains("lists = 128"));
    }

    #[test]
    fn configured_engine_label_is_self_describing() {
        assert_eq!(
            configured_engine_label("ec_diskann", "list_size", 200),
            "ec_diskann[list_size=200]"
        );
        assert_eq!(
            configured_engine_label("pgvector", "ef_search", 100),
            "pgvector[ef_search=100]"
        );
        assert_eq!(
            configured_engine_label("pgvector_ivfflat", "probes", 64),
            "pgvector_ivfflat[probes=64]"
        );
    }

    #[test]
    fn pgvector_args_accept_generic_ecaz_sweep_flag() {
        let cmd = PgvectorArgs::augment_args(Command::new("pgvector"));
        let matches = cmd
            .try_get_matches_from(["pgvector", "--prefix", "dbpedia_10k", "--ecaz-sweep", "200"])
            .unwrap();
        let args = PgvectorArgs::from_arg_matches(&matches).unwrap();
        assert_eq!(args.prefix, "dbpedia_10k");
        assert_eq!(args.ecaz_sweep, 200);
        assert!(args.sweep.is_empty());
    }

    #[test]
    fn pgvector_args_keep_legacy_ecaz_ef_search_alias() {
        let cmd = PgvectorArgs::augment_args(Command::new("pgvector"));
        let matches = cmd
            .try_get_matches_from([
                "pgvector",
                "--prefix",
                "dbpedia_10k",
                "--ecaz-ef-search",
                "160",
            ])
            .unwrap();
        let args = PgvectorArgs::from_arg_matches(&matches).unwrap();
        assert_eq!(args.ecaz_sweep, 160);
    }

    #[test]
    fn pgvector_args_accept_matched_sweep_list() {
        let cmd = PgvectorArgs::augment_args(Command::new("pgvector"));
        let matches = cmd
            .try_get_matches_from([
                "pgvector",
                "--prefix",
                "dbpedia_10k",
                "--sweep",
                "64,128,200",
            ])
            .unwrap();
        let args = PgvectorArgs::from_arg_matches(&matches).unwrap();
        assert_eq!(args.sweep, vec![64, 128, 200]);
    }

    #[test]
    fn pgvector_args_accept_ivfflat_options() {
        let cmd = PgvectorArgs::augment_args(Command::new("pgvector"));
        let matches = cmd
            .try_get_matches_from([
                "pgvector",
                "--prefix",
                "dbpedia_10k",
                "--profile",
                "ec_ivf",
                "--pgvector-am",
                "ivfflat",
                "--pgvector-lists",
                "128",
                "--pgvector-probes",
                "64",
                "--rerank-width",
                "500",
            ])
            .unwrap();
        let args = PgvectorArgs::from_arg_matches(&matches).unwrap();
        assert_eq!(args.pgvector_am, PgvectorIndexKind::Ivfflat);
        assert_eq!(args.pgvector_lists, 128);
        assert_eq!(args.pgvector_probes, 64);
        assert_eq!(args.rerank_width, Some(500));
    }

    #[test]
    fn knn_sql_uses_ip_operator_and_bind_cast() {
        let sql = build_pgvector_knn_sql("t_corpus_pgvector", 1536);
        assert!(sql.contains("FROM t_corpus_pgvector"));
        assert!(sql.contains("<#>"));
        assert!(!sql.contains("pg_catalog"), "got: {sql}");
        assert!(sql.contains("$1::real[]::vector(1536)"));
        assert!(sql.contains("LIMIT $2"));
    }

    // --- pct_delta / format_pct_delta ---

    #[test]
    fn pct_delta_positive_when_other_larger() {
        assert!((pct_delta(100.0, 110.0).unwrap() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn pct_delta_negative_when_other_smaller() {
        assert!((pct_delta(100.0, 75.0).unwrap() + 25.0).abs() < 1e-9);
    }

    #[test]
    fn pct_delta_zero_base_is_none_not_infinity() {
        assert!(pct_delta(0.0, 5.0).is_none());
    }

    #[test]
    fn pct_delta_nan_input_is_none() {
        assert!(pct_delta(f64::NAN, 1.0).is_none());
        assert!(pct_delta(1.0, f64::NAN).is_none());
        assert!(pct_delta(1.0, f64::INFINITY).is_none());
    }

    #[test]
    fn format_pct_delta_has_leading_sign_and_one_decimal() {
        assert_eq!(format_pct_delta(100.0, 110.0), "+10.0%");
        assert_eq!(format_pct_delta(100.0, 90.0), "-10.0%");
        assert_eq!(format_pct_delta(0.0, 1.0), "n/a");
    }

    #[test]
    fn format_pct_delta_ms_reads_duration_as_ms() {
        let a = Duration::from_millis(100);
        let b = Duration::from_millis(50);
        // 50 vs 100 → -50%
        assert_eq!(format_pct_delta_ms(a, b), "-50.0%");
    }

    // --- ComparisonRow ---

    #[test]
    fn comparison_row_carries_engine_label_and_metrics() {
        let stats = LatencyStats {
            count: 10,
            mean: Duration::from_millis(5),
            stddev: Duration::ZERO,
            min: Duration::from_millis(4),
            p50: Duration::from_millis(5),
            p95: Duration::from_millis(6),
            p99: Duration::from_millis(7),
            max: Duration::from_millis(8),
        };
        let row = ComparisonRow {
            engine: "ec_diskann[list_size=200]".into(),
            sweep: None,
            recall: 0.9,
            ndcg: 0.8,
            stats,
        };
        assert_eq!(row.engine, "ec_diskann[list_size=200]");
        assert_eq!(row.sweep, None);
        assert!((row.recall - 0.9).abs() < 1e-9);
        assert!((row.ndcg - 0.8).abs() < 1e-9);
        assert_eq!(row.stats.count, 10);
    }
}
