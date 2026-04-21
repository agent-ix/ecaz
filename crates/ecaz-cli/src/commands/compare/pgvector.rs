//! `ecaz compare pgvector` — side-by-side recall + latency against pgvector.
//!
//! # Flow
//!
//! 1. Ensure the `vector` extension is installed.
//! 2. Materialize a `<prefix>_corpus_pgvector` sidecar `(id bigint, embedding
//!    vector(dim))` sourced from the ecaz `<prefix>_corpus.source` column
//!    (idempotent unless `--rebuild`).
//! 3. Build a pgvector HNSW index on the sidecar with the requested `m` /
//!    `ef_construction` (idempotent by index name unless `--rebuild`).
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

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use indicatif::{ProgressBar, ProgressStyle};
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
    /// pgvector-side `hnsw.ef_search` for the timed queries.
    #[arg(long, default_value_t = 100)]
    pub pgvector_ef_search: i32,
    /// pgvector HNSW build `m`.
    #[arg(long, default_value_t = 16)]
    pub pgvector_m: i32,
    /// pgvector HNSW build `ef_construction`.
    #[arg(long, default_value_t = 128)]
    pub pgvector_ef_construction: i32,
    /// Cap the query set (default: all rows).
    #[arg(long)]
    pub queries_limit: Option<usize>,
    /// Quantization bits used when encoding query vectors (must match loader).
    #[arg(long, default_value_t = 4)]
    pub bits: i32,
    /// Quantizer seed (must match loader).
    #[arg(long, default_value_t = 42)]
    pub seed: i64,
    /// Drop + rebuild the pgvector sidecar table + index before measuring.
    #[arg(long, default_value_t = false)]
    pub rebuild: bool,
}

pub async fn run(conn: &ConnectionOptions, args: PgvectorArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    if args.k == 0 {
        return Err(eyre!("--k must be >= 1"));
    }
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

    let corpus_table = format!("{}_corpus", args.prefix);
    let queries_table = format!("{}_queries", args.prefix);
    let sidecar_table = pgvector_sidecar_name(&args.prefix);
    let sidecar_index = pgvector_index_name(&args.prefix);

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
        args.pgvector_m,
        args.pgvector_ef_construction,
        args.rebuild,
    )
    .await?;

    eprintln!("[compare] fetching corpus + queries for ground truth ...");
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

    eprintln!("[compare] computing ground truth ...");
    let t0 = Instant::now();
    let gt = brute_force_top_k(&corpus, &queries, args.k);
    eprintln!("[compare] ground truth in {:.2?}", t0.elapsed());
    let truth_ids = map_indices_to_ids(&gt.indices, &corpus_ids);
    let ecaz_label =
        configured_engine_label(profile.name, profile.sweep_axis_label(), args.ecaz_sweep);
    let pgv_label = configured_engine_label("pgvector", "ef_search", args.pgvector_ef_search);

    // Ecaz side.
    client
        .batch_execute(&format!("SET {ecaz_guc} = {}", args.ecaz_sweep))
        .await
        .wrap_err_with(|| format!("SET {ecaz_guc}"))?;
    let ecaz_sql = build_knn_sql(profile, &corpus_table);
    let (ecaz_recall, ecaz_ndcg, ecaz_stats) = measure_engine(
        &client,
        &ecaz_label,
        &ecaz_sql,
        &queries,
        &gt,
        &corpus_ids,
        &truth_ids,
        args.k,
        EngineBinds::Ecaz {
            bits: args.bits,
            seed: args.seed,
        },
    )
    .await?;

    // pgvector side.
    client
        .batch_execute(&format!("SET hnsw.ef_search = {}", args.pgvector_ef_search))
        .await
        .wrap_err("SET hnsw.ef_search")?;
    let pgv_sql = build_pgvector_knn_sql(&sidecar_table, dim);
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

    let rows = vec![
        ComparisonRow::new(&ecaz_label, ecaz_recall, ecaz_ndcg, ecaz_stats),
        ComparisonRow::new(&pgv_label, pgv_recall, pgv_ndcg, pgv_stats),
    ];
    print_comparison(&rows);
    Ok(())
}

/// Name of the pgvector sidecar table for a given ecaz prefix. Kept as a
/// free function so tests can pin the naming rule.
pub fn pgvector_sidecar_name(prefix: &str) -> String {
    format!("{prefix}_corpus_pgvector")
}

/// Name of the pgvector HNSW index built on the sidecar.
pub fn pgvector_index_name(prefix: &str) -> String {
    format!("{prefix}_corpus_pgvector_hnsw_idx")
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

/// KNN template for the pgvector sidecar: binds are `($1::real[], $2::bigint)`
/// = (query_source, k). Uses the inner-product operator to match ecaz's
/// `encode_to_ecvector`-preserving IP semantics.
pub fn build_pgvector_knn_sql(sidecar: &str, dim: usize) -> String {
    format!(
        "SELECT id FROM {sidecar} \
         ORDER BY embedding OPERATOR(pg_catalog.<#>) \
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

#[allow(clippy::too_many_arguments)]
async fn ensure_pgvector_sidecar(
    client: &Client,
    corpus_table: &str,
    sidecar: &str,
    index_name: &str,
    dim: usize,
    m: i32,
    ef_construction: i32,
    rebuild: bool,
) -> Result<()> {
    if rebuild {
        eprintln!("[compare] --rebuild: dropping {sidecar} (and dependent index)");
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
        eprintln!(
            "[compare] populating {sidecar}: {} rows missing from {corpus_rows}",
            corpus_rows - existing
        );
        client
            .batch_execute(&build_pgvector_populate_sql(corpus_table, sidecar, dim))
            .await
            .wrap_err("populating pgvector sidecar")?;
    }

    if !psql::relation_exists(client, index_name, 'i').await? {
        eprintln!("[compare] building pgvector HNSW index {index_name}");
        let t0 = Instant::now();
        client
            .batch_execute(&build_pgvector_create_index_sql(
                sidecar,
                index_name,
                m,
                ef_construction,
            ))
            .await
            .wrap_err("creating pgvector index")?;
        eprintln!("[compare] built {index_name} in {:.2?}", t0.elapsed());
    }
    Ok(())
}

enum EngineBinds {
    Ecaz { bits: i32, seed: i64 },
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
    let bar = ProgressBar::new(queries.nrows() as u64);
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
            EngineBinds::Ecaz { bits, seed } => {
                client.query(&stmt, &[&row_vec, bits, seed, &k_i64]).await
            }
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
    pub recall: f64,
    pub ndcg: f64,
    pub stats: LatencyStats,
}

impl ComparisonRow {
    pub fn new(engine: &str, recall: f64, ndcg: f64, stats: LatencyStats) -> Self {
        Self {
            engine: engine.to_owned(),
            recall,
            ndcg,
            stats,
        }
    }
}

fn print_comparison(rows: &[ComparisonRow]) {
    let mut t = Table::new();
    t.load_preset(UTF8_FULL);
    t.set_header(vec![
        "engine", "recall@k", "ndcg@k", "p50", "p95", "p99", "mean",
    ]);
    for r in rows {
        t.add_row(vec![
            Cell::new(&r.engine),
            Cell::new(format!("{:.4}", r.recall)),
            Cell::new(format!("{:.4}", r.ndcg)),
            Cell::new(format_ms(r.stats.p50)),
            Cell::new(format_ms(r.stats.p95)),
            Cell::new(format_ms(r.stats.p99)),
            Cell::new(format_ms(r.stats.mean)),
        ]);
    }
    if rows.len() == 2 {
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
    println!("{t}");
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
            pgvector_index_name("dbpedia_10k"),
            "dbpedia_10k_corpus_pgvector_hnsw_idx"
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
    fn configured_engine_label_is_self_describing() {
        assert_eq!(
            configured_engine_label("ec_diskann", "list_size", 200),
            "ec_diskann[list_size=200]"
        );
        assert_eq!(
            configured_engine_label("pgvector", "ef_search", 100),
            "pgvector[ef_search=100]"
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
    fn knn_sql_uses_ip_operator_and_bind_cast() {
        let sql = build_pgvector_knn_sql("t_corpus_pgvector", 1536);
        assert!(sql.contains("FROM t_corpus_pgvector"));
        assert!(sql.contains("<#>"));
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
        let row = ComparisonRow::new("ec_diskann[list_size=200]", 0.9, 0.8, stats);
        assert_eq!(row.engine, "ec_diskann[list_size=200]");
        assert!((row.recall - 0.9).abs() < 1e-9);
        assert!((row.ndcg - 0.8).abs() < 1e-9);
        assert_eq!(row.stats.count, 10);
    }
}
