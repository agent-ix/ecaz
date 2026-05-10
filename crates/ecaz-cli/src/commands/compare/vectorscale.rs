//! `ecaz compare vectorscale` — side-by-side recall + latency against
//! pgvectorscale StreamingDiskANN on the same ecaz corpus.

use clap::Args;
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
pub struct VectorscaleArgs {
    /// Prefix identifying the ecaz corpus (as loaded by `ecaz corpus load`).
    #[arg(long)]
    pub prefix: String,
    /// Ecaz profile to compare against pgvectorscale.
    #[arg(long, default_value = "ec_diskann")]
    pub profile: String,
    /// k for recall@k / latency measurement.
    #[arg(long, default_value_t = 10)]
    pub k: usize,
    /// Matched sweep values for ecaz list_size and pgvectorscale
    /// diskann.query_search_list_size.
    #[arg(long, value_delimiter = ',')]
    pub sweep: Vec<i32>,
    /// Fallback single ecaz-side tuning value when --sweep is omitted.
    #[arg(long = "ecaz-sweep", default_value_t = 200)]
    pub ecaz_sweep: i32,
    /// pgvectorscale StreamingDiskANN build num_neighbors.
    #[arg(long, default_value_t = 32)]
    pub vectorscale_num_neighbors: i32,
    /// pgvectorscale StreamingDiskANN build search_list_size.
    #[arg(long, default_value_t = 100)]
    pub vectorscale_build_search_list_size: i32,
    /// pgvectorscale StreamingDiskANN build max_alpha.
    #[arg(long, default_value_t = 1.2)]
    pub vectorscale_max_alpha: f32,
    /// pgvectorscale storage_layout index option.
    #[arg(long, default_value = "memory_optimized")]
    pub vectorscale_storage_layout: String,
    /// pgvectorscale diskann.query_rescore. Defaults to each sweep value.
    #[arg(long)]
    pub vectorscale_query_rescore: Option<i32>,
    /// Extra ecaz session GUC to set before the sweep, in NAME=VALUE form.
    #[arg(long = "set-guc")]
    pub set_gucs: Vec<String>,
    /// Ecaz session GUC whose value should be set to each sweep point.
    #[arg(long = "set-guc-from-sweep")]
    pub set_gucs_from_sweep: Vec<String>,
    /// Cap the query set (default: all rows).
    #[arg(long)]
    pub queries_limit: Option<usize>,
    /// Drop + rebuild the pgvectorscale sidecar table + index before measuring.
    #[arg(long, default_value_t = false)]
    pub rebuild: bool,
}

pub async fn run(conn: &ConnectionOptions, args: VectorscaleArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    if args.k == 0 {
        return Err(eyre!("--k must be >= 1"));
    }
    if args.vectorscale_num_neighbors <= 10 {
        return Err(eyre!("--vectorscale-num-neighbors must be > 10"));
    }
    if args.vectorscale_build_search_list_size <= 0 {
        return Err(eyre!("--vectorscale-build-search-list-size must be > 0"));
    }
    if args.vectorscale_max_alpha <= 0.0 {
        return Err(eyre!("--vectorscale-max-alpha must be > 0"));
    }
    validate_storage_layout(&args.vectorscale_storage_layout)?;

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
    let sweep_values = if args.sweep.is_empty() {
        vec![args.ecaz_sweep]
    } else {
        args.sweep.clone()
    };
    let set_gucs = args
        .set_gucs
        .iter()
        .map(|raw| psql::parse_session_setting(raw))
        .collect::<Result<Vec<_>>>()?;
    for name in &args.set_gucs_from_sweep {
        psql::validate_session_guc_name(name)?;
    }

    let corpus_table = format!("{}_corpus", args.prefix);
    let queries_table = format!("{}_queries", args.prefix);
    let sidecar_table = vectorscale_sidecar_name(&args.prefix);
    let sidecar_index = vectorscale_index_name(&args.prefix);

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
    client
        .batch_execute("CREATE EXTENSION IF NOT EXISTS vectorscale CASCADE")
        .await
        .wrap_err("ensuring pgvectorscale extension")?;

    let dim = read_dim(&client, &corpus_table).await?;
    ensure_vectorscale_sidecar(
        &client,
        &corpus_table,
        &sidecar_table,
        &sidecar_index,
        dim,
        &args,
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
    let vectorscale_sql = build_vectorscale_knn_sql(&sidecar_table, dim);

    let mut rows = Vec::with_capacity(sweep_values.len() * 2);
    for value in sweep_values {
        client
            .batch_execute(&format!("SET {ecaz_guc} = {value}"))
            .await
            .wrap_err_with(|| format!("SET {ecaz_guc}"))?;
        let sweep_settings = args
            .set_gucs_from_sweep
            .iter()
            .map(|name| psql::session_setting_from_sweep(name, value))
            .collect::<Result<Vec<_>>>()?;
        psql::apply_session_settings(&client, &sweep_settings).await?;
        let ecaz_label = configured_engine_label(profile.name, profile.sweep_axis_label(), value);
        let (ecaz_recall, ecaz_ndcg, ecaz_stats) = measure_engine(
            &client,
            &ecaz_label,
            &ecaz_sql,
            &queries,
            &gt,
            &corpus_ids,
            &truth_ids,
            args.k,
        )
        .await?;
        rows.push(ComparisonRow::new(
            &ecaz_label,
            value,
            ecaz_recall,
            ecaz_ndcg,
            ecaz_stats,
        ));

        let rescore = args.vectorscale_query_rescore.unwrap_or(value);
        client
            .batch_execute(&format!(
                "SET diskann.query_search_list_size = {value}; SET diskann.query_rescore = {rescore}"
            ))
            .await
            .wrap_err("setting pgvectorscale query GUCs")?;
        let vectorscale_label =
            configured_engine_label("pgvectorscale", "query_search_list_size", value);
        let (vectorscale_recall, vectorscale_ndcg, vectorscale_stats) = measure_engine(
            &client,
            &vectorscale_label,
            &vectorscale_sql,
            &queries,
            &gt,
            &corpus_ids,
            &truth_ids,
            args.k,
        )
        .await?;
        rows.push(ComparisonRow::new(
            &vectorscale_label,
            value,
            vectorscale_recall,
            vectorscale_ndcg,
            vectorscale_stats,
        ));
    }

    print_comparison(&rows);
    Ok(())
}

pub fn vectorscale_sidecar_name(prefix: &str) -> String {
    format!("{prefix}_corpus_vectorscale")
}

pub fn vectorscale_index_name(prefix: &str) -> String {
    format!("{prefix}_corpus_vectorscale_diskann_idx")
}

pub fn build_vectorscale_sidecar_ddl(sidecar: &str, dim: usize) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {sidecar} (\n    id bigint PRIMARY KEY,\n    embedding vector({dim}) NOT NULL\n)"
    )
}

pub fn build_vectorscale_populate_sql(corpus_table: &str, sidecar: &str, dim: usize) -> String {
    format!(
        "INSERT INTO {sidecar} (id, embedding)\n         SELECT id, source::vector({dim}) FROM {corpus_table}\n         ON CONFLICT (id) DO NOTHING"
    )
}

pub fn build_vectorscale_create_index_sql(
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

pub fn build_vectorscale_knn_sql(sidecar: &str, dim: usize) -> String {
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

async fn ensure_vectorscale_sidecar(
    client: &Client,
    corpus_table: &str,
    sidecar: &str,
    index_name: &str,
    dim: usize,
    args: &VectorscaleArgs,
) -> Result<()> {
    if args.rebuild {
        crate::ecaz_eprintln!("[compare] --rebuild: dropping {sidecar} (and dependent index)");
        client
            .batch_execute(&format!("DROP TABLE IF EXISTS {sidecar} CASCADE"))
            .await
            .wrap_err("dropping pgvectorscale sidecar")?;
    }

    client
        .batch_execute(&build_vectorscale_sidecar_ddl(sidecar, dim))
        .await
        .wrap_err("creating pgvectorscale sidecar")?;

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
            .batch_execute(&build_vectorscale_populate_sql(corpus_table, sidecar, dim))
            .await
            .wrap_err("populating pgvectorscale sidecar")?;
    }

    if !psql::relation_exists(client, index_name, 'i').await? {
        crate::ecaz_eprintln!("[compare] building pgvectorscale DiskANN index {index_name}");
        let t0 = Instant::now();
        client
            .batch_execute(&build_vectorscale_create_index_sql(
                sidecar,
                index_name,
                args.vectorscale_num_neighbors,
                args.vectorscale_build_search_list_size,
                args.vectorscale_max_alpha,
                &args.vectorscale_storage_layout,
            ))
            .await
            .wrap_err("creating pgvectorscale DiskANN index")?;
        crate::ecaz_eprintln!("[compare] built {index_name} in {:.2?}", t0.elapsed());
    }
    let size: i64 = client
        .query_one(
            &format!("SELECT pg_relation_size('{index_name}'::regclass)"),
            &[],
        )
        .await
        .wrap_err("reading pgvectorscale index size")?
        .get(0);
    crate::ecaz_eprintln!("[compare] {index_name} pg_relation_size={size} bytes");
    Ok(())
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
        let rows = client
            .query(&stmt, &[&row_vec, &k_i64])
            .await
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

#[derive(Debug, Clone)]
pub struct ComparisonRow {
    pub engine: String,
    pub sweep_value: i32,
    pub recall: f64,
    pub ndcg: f64,
    pub stats: LatencyStats,
}

impl ComparisonRow {
    pub fn new(
        engine: &str,
        sweep_value: i32,
        recall: f64,
        ndcg: f64,
        stats: LatencyStats,
    ) -> Self {
        Self {
            engine: engine.to_owned(),
            sweep_value,
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
        "engine", "sweep", "recall@k", "ndcg@k", "p50", "p95", "p99", "mean",
    ]);
    for r in rows {
        t.add_row(vec![
            Cell::new(&r.engine),
            Cell::new(r.sweep_value),
            Cell::new(format!("{:.4}", r.recall)),
            Cell::new(format!("{:.4}", r.ndcg)),
            Cell::new(format_ms(r.stats.p50)),
            Cell::new(format_ms(r.stats.p95)),
            Cell::new(format_ms(r.stats.p99)),
            Cell::new(format_ms(r.stats.mean)),
        ]);
    }
    crate::ecaz_println!("{t}");
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

fn validate_storage_layout(value: &str) -> Result<()> {
    match value {
        "memory_optimized" | "plain" => Ok(()),
        other => Err(eyre!(
            "unsupported --vectorscale-storage-layout {other:?}; expected memory_optimized or plain"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vectorscale_names_are_suffixed_from_prefix() {
        assert_eq!(
            vectorscale_sidecar_name("dbpedia_10k"),
            "dbpedia_10k_corpus_vectorscale"
        );
        assert_eq!(
            vectorscale_index_name("dbpedia_10k"),
            "dbpedia_10k_corpus_vectorscale_diskann_idx"
        );
    }

    #[test]
    fn vectorscale_create_index_sql_uses_diskann_ip_ops_and_reloptions() {
        let sql = build_vectorscale_create_index_sql(
            "t_corpus_vectorscale",
            "t_vsc_idx",
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
    fn vectorscale_knn_sql_uses_ip_operator_and_bind_cast() {
        let sql = build_vectorscale_knn_sql("t_corpus_vectorscale", 1536);
        assert!(sql.contains("FROM t_corpus_vectorscale"));
        assert!(sql.contains("<#>"));
        assert!(sql.contains("$1::real[]::vector(1536)"));
        assert!(sql.contains("LIMIT $2"));
    }

    #[test]
    fn storage_layout_validation_accepts_known_values() {
        validate_storage_layout("memory_optimized").unwrap();
        validate_storage_layout("plain").unwrap();
        assert!(validate_storage_layout("other").is_err());
    }
}
