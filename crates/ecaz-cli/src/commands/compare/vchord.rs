//! `ecaz compare vchord` — side-by-side recall + latency against
//! VectorChord's vchordrq RaBitQ-on-IVF access method.

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
pub struct VchordArgs {
    /// Prefix identifying the ecaz corpus (as loaded by `ecaz corpus load`).
    #[arg(long)]
    pub prefix: String,
    /// Ecaz profile to compare against vchord.
    #[arg(long, default_value = "ec_ivf")]
    pub profile: String,
    /// k for recall@k / latency measurement.
    #[arg(long, default_value_t = 10)]
    pub k: usize,
    /// Matched sweep values for ecaz nprobe and vchordrq.probes.
    #[arg(long, value_delimiter = ',')]
    pub sweep: Vec<i32>,
    /// Fallback single ecaz-side tuning value when --sweep is omitted.
    #[arg(long = "ecaz-sweep", default_value_t = 64)]
    pub ecaz_sweep: i32,
    /// vchord query probes for a single-point comparison.
    #[arg(long, default_value_t = 64)]
    pub vchord_probes: i32,
    /// vchord build list count.
    #[arg(long, default_value_t = 128)]
    pub vchord_lists: i32,
    /// Session maintenance_work_mem used while building the vchord index.
    #[arg(long)]
    pub vchord_maintenance_work_mem: Option<String>,
    /// IVF-only: ecaz session override for heap-f32 rerank frontier width.
    /// Use -1 for the index reloption, 0 for the full probed frontier.
    #[arg(long)]
    pub rerank_width: Option<i32>,
    /// Cap the query set (default: all rows).
    #[arg(long)]
    pub queries_limit: Option<usize>,
    /// Drop + rebuild the vchord sidecar table + index before measuring.
    #[arg(long, default_value_t = false)]
    pub rebuild: bool,
}

pub async fn run(conn: &ConnectionOptions, args: VchordArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    if args.k == 0 {
        return Err(eyre!("--k must be >= 1"));
    }
    validate_vchord_args(&args)?;

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
    let ecaz_sweep_values = if args.sweep.is_empty() {
        vec![args.ecaz_sweep]
    } else {
        args.sweep.clone()
    };
    let vchord_sweep_values = if args.sweep.is_empty() {
        vec![args.vchord_probes]
    } else {
        args.sweep.clone()
    };

    let corpus_table = format!("{}_corpus", args.prefix);
    let queries_table = format!("{}_queries", args.prefix);
    let sidecar_table = vchord_sidecar_name(&args.prefix);
    let sidecar_index = vchord_index_name(&args.prefix);

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
        .batch_execute("CREATE EXTENSION IF NOT EXISTS vchord CASCADE")
        .await
        .wrap_err("ensuring vchord extension")?;

    let dim = read_dim(&client, &corpus_table).await?;
    ensure_vchord_sidecar(
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
    let truth_ids = map_indices_to_ids(&gt.indices, &corpus_ids);
    let ecaz_sql = build_knn_sql(profile, &corpus_table);
    let vchord_sql = build_vchord_knn_sql(&sidecar_table, dim);

    let mut rows = Vec::with_capacity(ecaz_sweep_values.len() * 2);
    for (ecaz_value, vchord_value) in ecaz_sweep_values.into_iter().zip(vchord_sweep_values) {
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
        let ecaz_label =
            configured_engine_label(profile.name, profile.sweep_axis_label(), ecaz_value);
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
            ecaz_value,
            ecaz_recall,
            ecaz_ndcg,
            ecaz_stats,
        ));

        client
            .batch_execute(&format!("SET vchordrq.probes = {vchord_value}"))
            .await
            .wrap_err("setting vchordrq.probes")?;
        let vchord_label = configured_engine_label("vchord_rabitq", "probes", vchord_value);
        let (vchord_recall, vchord_ndcg, vchord_stats) = measure_engine(
            &client,
            &vchord_label,
            &vchord_sql,
            &queries,
            &gt,
            &corpus_ids,
            &truth_ids,
            args.k,
        )
        .await?;
        rows.push(ComparisonRow::new(
            &vchord_label,
            vchord_value,
            vchord_recall,
            vchord_ndcg,
            vchord_stats,
        ));
    }

    print_comparison(&rows);
    Ok(())
}

pub fn vchord_sidecar_name(prefix: &str) -> String {
    format!("{prefix}_corpus_vchord")
}

pub fn vchord_index_name(prefix: &str) -> String {
    format!("{prefix}_corpus_vchord_rabitq_idx")
}

pub fn build_vchord_sidecar_ddl(sidecar: &str, dim: usize) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {sidecar} (\n    id bigint PRIMARY KEY,\n    embedding vector({dim}) NOT NULL\n)"
    )
}

pub fn build_vchord_populate_sql(corpus_table: &str, sidecar: &str, dim: usize) -> String {
    format!(
        "INSERT INTO {sidecar} (id, embedding)\n         SELECT id, source::vector({dim}) FROM {corpus_table}\n         ON CONFLICT (id) DO NOTHING"
    )
}

pub fn build_vchord_create_index_sql(sidecar: &str, index_name: &str, lists: i32) -> String {
    format!(
        "CREATE INDEX {index_name} ON {sidecar}\n         USING vchordrq (embedding vector_ip_ops)\n         WITH (options = $vco$\nresidual_quantization = true\n[build.internal]\nlists = [{lists}]\n$vco$)"
    )
}

pub fn build_vchord_knn_sql(sidecar: &str, dim: usize) -> String {
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

async fn ensure_vchord_sidecar(
    client: &Client,
    corpus_table: &str,
    sidecar: &str,
    index_name: &str,
    dim: usize,
    args: &VchordArgs,
) -> Result<()> {
    if args.rebuild {
        crate::ecaz_eprintln!("[compare] --rebuild: dropping {sidecar} (and dependent index)");
        client
            .batch_execute(&format!("DROP TABLE IF EXISTS {sidecar} CASCADE"))
            .await
            .wrap_err("dropping vchord sidecar")?;
    }

    client
        .batch_execute(&build_vchord_sidecar_ddl(sidecar, dim))
        .await
        .wrap_err("creating vchord sidecar")?;

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
            .batch_execute(&build_vchord_populate_sql(corpus_table, sidecar, dim))
            .await
            .wrap_err("populating vchord sidecar")?;
    }

    if !psql::relation_exists(client, index_name, 'i').await? {
        if let Some(memory) = args.vchord_maintenance_work_mem.as_deref() {
            crate::ecaz_eprintln!("[compare] SET maintenance_work_mem = '{memory}'");
            client
                .batch_execute(&format!("SET maintenance_work_mem = '{memory}'"))
                .await
                .wrap_err_with(|| format!("SET maintenance_work_mem = {memory}"))?;
        }
        crate::ecaz_eprintln!(
            "[compare] building vchord RaBitQ index {index_name} lists={}",
            args.vchord_lists
        );
        let t0 = Instant::now();
        client
            .batch_execute(&build_vchord_create_index_sql(
                sidecar,
                index_name,
                args.vchord_lists,
            ))
            .await
            .wrap_err("creating vchord index")?;
        crate::ecaz_eprintln!("[compare] built {index_name} in {:.2?}", t0.elapsed());
    }
    let size: i64 = client
        .query_one(
            &format!("SELECT pg_relation_size('{index_name}'::regclass)"),
            &[],
        )
        .await
        .wrap_err("reading vchord index size")?
        .get(0);
    crate::ecaz_eprintln!("[compare] {index_name} pg_relation_size={size} bytes");
    Ok(())
}

fn validate_vchord_args(args: &VchordArgs) -> Result<()> {
    if args.ecaz_sweep <= 0 {
        return Err(eyre!("--ecaz-sweep must be > 0"));
    }
    if args.vchord_probes <= 0 {
        return Err(eyre!("--vchord-probes must be > 0"));
    }
    if args.vchord_lists <= 0 {
        return Err(eyre!("--vchord-lists must be > 0"));
    }
    if let Some(value) = args.vchord_maintenance_work_mem.as_deref() {
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
        return Err(eyre!("--vchord-maintenance-work-mem cannot be empty"));
    }
    let digits = value.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 || digits == value.len() {
        return Err(eyre!(
            "--vchord-maintenance-work-mem must look like 256MB, 1GB, or 65536kB"
        ));
    }
    let unit = &value[digits..];
    if !matches!(unit, "B" | "kB" | "MB" | "GB" | "TB") {
        return Err(eyre!(
            "--vchord-maintenance-work-mem unit must be B, kB, MB, GB, or TB"
        ));
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Command, FromArgMatches};

    #[test]
    fn vchord_names_are_suffixed_from_prefix() {
        assert_eq!(
            vchord_sidecar_name("dbpedia_100k"),
            "dbpedia_100k_corpus_vchord"
        );
        assert_eq!(
            vchord_index_name("dbpedia_100k"),
            "dbpedia_100k_corpus_vchord_rabitq_idx"
        );
    }

    #[test]
    fn vchord_create_index_sql_uses_rabitq_options() {
        let sql = build_vchord_create_index_sql("t_corpus_vchord", "t_vchord_idx", 320);
        assert!(sql.contains("USING vchordrq (embedding vector_ip_ops)"));
        assert!(sql.contains("residual_quantization = true"));
        assert!(sql.contains("lists = [320]"));
    }

    #[test]
    fn vchord_knn_sql_uses_ip_operator_and_bind_cast() {
        let sql = build_vchord_knn_sql("t_corpus_vchord", 1536);
        assert!(sql.contains("FROM t_corpus_vchord"));
        assert!(sql.contains("<#>"));
        assert!(sql.contains("$1::real[]::vector(1536)"));
        assert!(sql.contains("LIMIT $2"));
    }

    #[test]
    fn vchord_args_accept_matched_sweep_list() {
        let cmd = VchordArgs::augment_args(Command::new("vchord"));
        let matches = cmd
            .try_get_matches_from([
                "vchord",
                "--prefix",
                "dbpedia_100k",
                "--sweep",
                "16,32,64",
                "--vchord-lists",
                "320",
            ])
            .unwrap();
        let args = VchordArgs::from_arg_matches(&matches).unwrap();
        assert_eq!(args.sweep, vec![16, 32, 64]);
        assert_eq!(args.vchord_lists, 320);
    }

    #[test]
    fn configured_engine_label_is_self_describing() {
        assert_eq!(
            configured_engine_label("vchord_rabitq", "probes", 64),
            "vchord_rabitq[probes=64]"
        );
    }
}
