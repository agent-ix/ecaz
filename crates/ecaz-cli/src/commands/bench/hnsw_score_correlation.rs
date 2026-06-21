//! `ecaz bench hnsw-score-correlation` — HNSW approximate/exact score drift diagnostics.

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use serde::Serialize;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use tokio_postgres::{Client, Row};

use crate::profiles;
use crate::psql::{self, ConnectionOptions};

#[derive(Args, Debug)]
pub struct HnswScoreCorrelationArgs {
    /// Prefix identifying the loaded corpus.
    #[arg(long)]
    pub prefix: String,
    /// HNSW index name. If omitted, the command requires exactly one ec_hnsw
    /// index on `<prefix>_corpus`.
    #[arg(long)]
    pub index: Option<String>,
    /// HNSW build `m` value to record in output rows.
    #[arg(long)]
    pub m: i32,
    /// ef_search values to audit. Accepts `--sweep 100,200` or repeated flags.
    #[arg(long, value_delimiter = ',')]
    pub sweep: Vec<i32>,
    /// Cap the query set.
    #[arg(long, default_value_t = 200)]
    pub queries_limit: usize,
    /// Write compact summary output to this path in addition to stdout.
    #[arg(long)]
    pub log_output: Option<PathBuf>,
    /// Write per-query diagnostic rows as JSONL.
    #[arg(long)]
    pub jsonl_output: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
struct ScoreCorrelationRow {
    prefix: String,
    corpus_table: String,
    query_table: String,
    index_name: String,
    m: i32,
    ef_search: i32,
    query_index: i32,
    emitted_result_count: i32,
    compared_result_count: i32,
    missing_comparison_count: i32,
    mean_abs_score_delta: f64,
    max_abs_score_delta: f32,
    mean_signed_score_delta: f64,
    mean_abs_rank_shift: f64,
    max_abs_rank_shift: i32,
    spearman_rank_correlation: f64,
    exact_best_approx_rank: Option<i32>,
    exact_top4_max_approx_rank: Option<i32>,
    compared_row_indices: Vec<i64>,
    compared_approx_ranks: Vec<i32>,
    compared_approx_scores: Vec<f32>,
    compared_exact_scores: Vec<f32>,
    compared_exact_ranks: Vec<i32>,
}

#[derive(Debug, Clone)]
struct ScoreCorrelationSummary {
    ef_search: i32,
    queries: usize,
    mean_emitted_result_count: f64,
    mean_compared_result_count: f64,
    mean_missing_comparison_count: f64,
    mean_abs_score_delta: f64,
    mean_signed_score_delta: f64,
    mean_abs_rank_shift: f64,
    max_abs_rank_shift: i32,
    mean_spearman_rank_correlation: f64,
    mean_exact_best_approx_rank: f64,
    mean_exact_top4_max_approx_rank: f64,
}

pub async fn run(conn: &ConnectionOptions, args: HnswScoreCorrelationArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    if let Some(index) = &args.index {
        profiles::validate_ident(index).wrap_err_with(|| format!("invalid index {:?}", index))?;
    }
    if args.m <= 0 {
        return Err(eyre!("--m must be positive"));
    }
    if args.queries_limit == 0 {
        return Err(eyre!("--queries-limit must be positive"));
    }
    let sweep = if args.sweep.is_empty() {
        profiles::EC_HNSW.default_sweep.to_vec()
    } else {
        args.sweep.clone()
    };
    if sweep.iter().any(|value| *value <= 0) {
        return Err(eyre!("--sweep values must be positive"));
    }

    let corpus_table = format!("{}_corpus", args.prefix);
    let query_table = format!("{}_queries", args.prefix);
    let client = psql::connect(conn).await?;
    if !psql::relation_exists(&client, &corpus_table, 'r').await? {
        return Err(eyre!("no corpus table {:?} in this database", corpus_table));
    }
    if !psql::relation_exists(&client, &query_table, 'r').await? {
        return Err(eyre!("no query table {:?} in this database", query_table));
    }
    let index_name = resolve_hnsw_index(&client, &corpus_table, args.index.as_deref()).await?;

    let mut jsonl_writer = match args.jsonl_output.as_deref() {
        Some(path) => Some(BufWriter::new(create_output_file(path)?)),
        None => None,
    };
    let mut summaries = Vec::new();
    for ef_search in sweep {
        let rows = fetch_score_correlation_rows(
            &client,
            &args.prefix,
            &corpus_table,
            &query_table,
            &index_name,
            args.m,
            ef_search,
            args.queries_limit,
        )
        .await?;
        if let Some(writer) = jsonl_writer.as_mut() {
            for row in &rows {
                serde_json::to_writer(&mut *writer, row)
                    .wrap_err("writing score correlation JSON row")?;
                writer
                    .write_all(b"\n")
                    .wrap_err("writing score correlation JSONL newline")?;
            }
        }
        summaries.push(summarize_rows(ef_search, &rows));
    }
    if let Some(writer) = jsonl_writer.as_mut() {
        writer
            .flush()
            .wrap_err("flushing score correlation JSONL output")?;
    }

    let output = render_summary(&args.prefix, &index_name, args.m, &summaries);
    crate::ecaz_println!("{output}");
    if let Some(path) = args.log_output.as_deref() {
        let mut file = create_output_file(path)?;
        file.write_all(output.as_bytes())
            .wrap_err_with(|| format!("writing {}", path.display()))?;
        file.write_all(b"\n")
            .wrap_err_with(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

async fn resolve_hnsw_index(
    client: &Client,
    corpus_table: &str,
    explicit: Option<&str>,
) -> Result<String> {
    if let Some(index) = explicit {
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
                      AND am.amname = 'ec_hnsw'
                )",
                &[&corpus_table, &index],
            )
            .await
            .wrap_err_with(|| format!("checking HNSW index {index:?} on {corpus_table:?}"))?;
        if !row.get::<_, bool>(0) {
            return Err(eyre!(
                "index {:?} is not an ec_hnsw index on {:?}",
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
               AND am.amname = 'ec_hnsw'
             ORDER BY i.relname",
            &[&corpus_table],
        )
        .await
        .wrap_err_with(|| format!("listing HNSW indexes on {corpus_table:?}"))?;
    match rows.as_slice() {
        [row] => Ok(row.get::<_, String>(0)),
        [] => Err(eyre!("no ec_hnsw index found on {:?}", corpus_table)),
        _ => Err(eyre!(
            "multiple ec_hnsw indexes found on {:?}; pass --index explicitly",
            corpus_table
        )),
    }
}

async fn fetch_score_correlation_rows(
    client: &Client,
    prefix: &str,
    corpus_table: &str,
    query_table: &str,
    index_name: &str,
    m: i32,
    ef_search: i32,
    queries_limit: usize,
) -> Result<Vec<ScoreCorrelationRow>> {
    let queries_limit = i32::try_from(queries_limit).wrap_err("--queries-limit exceeds int4")?;
    let rows = client
        .query(
            "SELECT *
             FROM tests.ec_hnsw_graph_scan_score_correlation_rows($1::text, $2::text, $3::text, $4::int4, $5::int4, $6::int4)
             ORDER BY query_index",
            &[
                &corpus_table,
                &query_table,
                &index_name,
                &m,
                &ef_search,
                &queries_limit,
            ],
        )
        .await
        .wrap_err(
            "running ec_hnsw score correlation diagnostic; ensure the extension is installed with pg_test diagnostics",
        )?;
    Ok(rows
        .into_iter()
        .map(|row| decode_score_correlation_row(prefix, corpus_table, query_table, index_name, row))
        .collect())
}

fn decode_score_correlation_row(
    prefix: &str,
    corpus_table: &str,
    query_table: &str,
    index_name: &str,
    row: Row,
) -> ScoreCorrelationRow {
    ScoreCorrelationRow {
        prefix: prefix.to_owned(),
        corpus_table: corpus_table.to_owned(),
        query_table: query_table.to_owned(),
        index_name: index_name.to_owned(),
        m: row.get("m"),
        ef_search: row.get("ef_search"),
        query_index: row.get("query_index"),
        emitted_result_count: row.get("emitted_result_count"),
        compared_result_count: row.get("compared_result_count"),
        missing_comparison_count: row.get("missing_comparison_count"),
        mean_abs_score_delta: row.get("mean_abs_score_delta"),
        max_abs_score_delta: row.get("max_abs_score_delta"),
        mean_signed_score_delta: row.get("mean_signed_score_delta"),
        mean_abs_rank_shift: row.get("mean_abs_rank_shift"),
        max_abs_rank_shift: row.get("max_abs_rank_shift"),
        spearman_rank_correlation: row.get("spearman_rank_correlation"),
        exact_best_approx_rank: row.get("exact_best_approx_rank"),
        exact_top4_max_approx_rank: row.get("exact_top4_max_approx_rank"),
        compared_row_indices: row.get("compared_row_indices"),
        compared_approx_ranks: row.get("compared_approx_ranks"),
        compared_approx_scores: row.get("compared_approx_scores"),
        compared_exact_scores: row.get("compared_exact_scores"),
        compared_exact_ranks: row.get("compared_exact_ranks"),
    }
}

fn summarize_rows(ef_search: i32, rows: &[ScoreCorrelationRow]) -> ScoreCorrelationSummary {
    let queries = rows.len();
    if queries == 0 {
        return ScoreCorrelationSummary {
            ef_search,
            queries,
            mean_emitted_result_count: 0.0,
            mean_compared_result_count: 0.0,
            mean_missing_comparison_count: 0.0,
            mean_abs_score_delta: 0.0,
            mean_signed_score_delta: 0.0,
            mean_abs_rank_shift: 0.0,
            max_abs_rank_shift: 0,
            mean_spearman_rank_correlation: 0.0,
            mean_exact_best_approx_rank: 0.0,
            mean_exact_top4_max_approx_rank: 0.0,
        };
    }
    ScoreCorrelationSummary {
        ef_search,
        queries,
        mean_emitted_result_count: mean_i32(rows, |row| row.emitted_result_count),
        mean_compared_result_count: mean_i32(rows, |row| row.compared_result_count),
        mean_missing_comparison_count: mean_i32(rows, |row| row.missing_comparison_count),
        mean_abs_score_delta: mean_f64(rows, |row| row.mean_abs_score_delta),
        mean_signed_score_delta: mean_f64(rows, |row| row.mean_signed_score_delta),
        mean_abs_rank_shift: mean_f64(rows, |row| row.mean_abs_rank_shift),
        max_abs_rank_shift: rows
            .iter()
            .map(|row| row.max_abs_rank_shift)
            .max()
            .unwrap_or(0),
        mean_spearman_rank_correlation: mean_f64(rows, |row| row.spearman_rank_correlation),
        mean_exact_best_approx_rank: mean_opt_i32(rows, |row| row.exact_best_approx_rank),
        mean_exact_top4_max_approx_rank: mean_opt_i32(rows, |row| row.exact_top4_max_approx_rank),
    }
}

fn mean_i32(rows: &[ScoreCorrelationRow], value: impl Fn(&ScoreCorrelationRow) -> i32) -> f64 {
    rows.iter().map(|row| f64::from(value(row))).sum::<f64>() / rows.len() as f64
}

fn mean_f64(rows: &[ScoreCorrelationRow], value: impl Fn(&ScoreCorrelationRow) -> f64) -> f64 {
    rows.iter().map(value).sum::<f64>() / rows.len() as f64
}

fn mean_opt_i32(
    rows: &[ScoreCorrelationRow],
    value: impl Fn(&ScoreCorrelationRow) -> Option<i32>,
) -> f64 {
    let mut count = 0usize;
    let mut sum = 0.0f64;
    for row in rows {
        if let Some(value) = value(row) {
            count += 1;
            sum += f64::from(value);
        }
    }
    if count == 0 {
        0.0
    } else {
        sum / count as f64
    }
}

fn render_summary(
    prefix: &str,
    index_name: &str,
    m: i32,
    summaries: &[ScoreCorrelationSummary],
) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "ef_search",
        "queries",
        "emitted",
        "compared",
        "missing cmp",
        "mean |score delta|",
        "mean signed delta",
        "mean |rank shift|",
        "max |rank shift|",
        "mean spearman",
        "exact best approx rank",
        "exact top4 max approx rank",
    ]);
    for summary in summaries {
        table.add_row(vec![
            Cell::new(summary.ef_search),
            Cell::new(summary.queries),
            Cell::new(format!("{:.1}", summary.mean_emitted_result_count)),
            Cell::new(format!("{:.1}", summary.mean_compared_result_count)),
            Cell::new(format!("{:.1}", summary.mean_missing_comparison_count)),
            Cell::new(format!("{:.6}", summary.mean_abs_score_delta)),
            Cell::new(format!("{:.6}", summary.mean_signed_score_delta)),
            Cell::new(format!("{:.2}", summary.mean_abs_rank_shift)),
            Cell::new(summary.max_abs_rank_shift),
            Cell::new(format!("{:.4}", summary.mean_spearman_rank_correlation)),
            Cell::new(format!("{:.1}", summary.mean_exact_best_approx_rank)),
            Cell::new(format!("{:.1}", summary.mean_exact_top4_max_approx_rank)),
        ]);
    }
    format!("prefix: {prefix}\nindex: {index_name}\nm: {m}\n{table}")
}

fn create_output_file(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .wrap_err_with(|| format!("creating {}", parent.display()))?;
        }
    }
    File::create(path).wrap_err_with(|| format!("creating {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(ef_search: i32, compared: i32, mean_abs_rank_shift: f64) -> ScoreCorrelationRow {
        ScoreCorrelationRow {
            prefix: "p".into(),
            corpus_table: "p_corpus".into(),
            query_table: "p_queries".into(),
            index_name: "p_m16_idx".into(),
            m: 16,
            ef_search,
            query_index: 0,
            emitted_result_count: 10,
            compared_result_count: compared,
            missing_comparison_count: 10 - compared,
            mean_abs_score_delta: 0.25,
            max_abs_score_delta: 0.5,
            mean_signed_score_delta: -0.1,
            mean_abs_rank_shift,
            max_abs_rank_shift: 4,
            spearman_rank_correlation: 0.75,
            exact_best_approx_rank: Some(3),
            exact_top4_max_approx_rank: Some(7),
            compared_row_indices: Vec::new(),
            compared_approx_ranks: Vec::new(),
            compared_approx_scores: Vec::new(),
            compared_exact_scores: Vec::new(),
            compared_exact_ranks: Vec::new(),
        }
    }

    #[test]
    fn summarize_rows_computes_means_and_maxima() {
        let rows = vec![row(200, 8, 2.0), row(200, 6, 4.0)];
        let summary = summarize_rows(200, &rows);
        assert_eq!(summary.queries, 2);
        assert!((summary.mean_compared_result_count - 7.0).abs() < 1e-9);
        assert!((summary.mean_missing_comparison_count - 3.0).abs() < 1e-9);
        assert!((summary.mean_abs_rank_shift - 3.0).abs() < 1e-9);
        assert_eq!(summary.max_abs_rank_shift, 4);
        assert!((summary.mean_exact_best_approx_rank - 3.0).abs() < 1e-9);
    }
}
