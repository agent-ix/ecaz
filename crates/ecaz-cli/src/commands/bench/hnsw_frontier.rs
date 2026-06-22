//! `ecaz bench hnsw-frontier` — HNSW emitted candidate-pool containment diagnostics.
//!
//! This command is intentionally tied to the pg_test diagnostic SQL functions
//! added for Task 118. It does not benchmark the production SQL path directly;
//! instead, it emits per-query evidence about the AM-emitted candidate pool.
//! The current diagnostic cannot distinguish a broader pre-rerank frontier from
//! the final emitted stream; JSONL rows expose that explicitly.

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
pub struct HnswFrontierArgs {
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
struct FrontierRow {
    prefix: String,
    corpus_table: String,
    query_table: String,
    index_name: String,
    m: i32,
    ef_search: i32,
    query_index: i32,
    visited_before_output: i32,
    pre_final_frontier_size: i32,
    final_visited_count: i32,
    final_emitted_count: i32,
    exact_reranked_candidates: i32,
    quantized_reranked_candidates: i32,
    candidates_dropped_before_exact_rerank: i32,
    truth_top10_in_frontier: i32,
    truth_top100_in_frontier: i32,
    truth_top10_row_indices: Vec<i64>,
    frontier_row_indices: Vec<i64>,
    frontier_approx_scores: Vec<f32>,
    frontier_exact_scores: Vec<f32>,
    frontier_approx_ranks: Vec<i32>,
    frontier_exact_ranks: Vec<i32>,
    final_emitted_row_indices: Vec<i64>,
    frontier_equals_final_emitted: bool,
}

#[derive(Debug, Clone)]
struct FrontierSummary {
    ef_search: i32,
    queries: usize,
    recall_top10_in_frontier: f64,
    recall_top100_in_frontier: f64,
    mean_visited_before_output: f64,
    mean_pre_final_frontier_size: f64,
    mean_final_visited_count: f64,
    mean_final_emitted_count: f64,
    mean_exact_reranked_candidates: f64,
    mean_quantized_reranked_candidates: f64,
    mean_candidates_dropped_before_exact_rerank: f64,
    all_frontier_equals_final_emitted: bool,
}

pub async fn run(conn: &ConnectionOptions, args: HnswFrontierArgs) -> Result<()> {
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
        let rows = fetch_frontier_rows(
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
                serde_json::to_writer(&mut *writer, row).wrap_err("writing frontier JSON row")?;
                writer
                    .write_all(b"\n")
                    .wrap_err("writing frontier JSONL newline")?;
            }
        }
        summaries.push(summarize_frontier_rows(ef_search, &rows));
    }
    if let Some(writer) = jsonl_writer.as_mut() {
        writer.flush().wrap_err("flushing frontier JSONL output")?;
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

async fn fetch_frontier_rows(
    client: &Client,
    prefix: &str,
    corpus_table: &str,
    query_table: &str,
    index_name: &str,
    m: i32,
    ef_search: i32,
    queries_limit: usize,
) -> Result<Vec<FrontierRow>> {
    let queries_limit = i32::try_from(queries_limit).wrap_err("--queries-limit exceeds int4")?;
    let rows = client
        .query(
            "SELECT *
             FROM tests.ec_hnsw_graph_scan_recall_frontier_containment_rows($1::text, $2::text, $3::text, $4::int4, $5::int4, $6::int4)
             ORDER BY query_index",
            &[&corpus_table, &query_table, &index_name, &m, &ef_search, &queries_limit],
        )
        .await
        .wrap_err(
            "running ec_hnsw frontier containment diagnostic; ensure the extension is installed with pg_test diagnostics",
        )?;
    Ok(rows
        .into_iter()
        .map(|row| decode_frontier_row(prefix, corpus_table, query_table, index_name, row))
        .collect())
}

fn decode_frontier_row(
    prefix: &str,
    corpus_table: &str,
    query_table: &str,
    index_name: &str,
    row: Row,
) -> FrontierRow {
    FrontierRow {
        prefix: prefix.to_owned(),
        corpus_table: corpus_table.to_owned(),
        query_table: query_table.to_owned(),
        index_name: index_name.to_owned(),
        m: row.get("m"),
        ef_search: row.get("ef_search"),
        query_index: row.get("query_index"),
        visited_before_output: row.get("visited_before_output"),
        pre_final_frontier_size: row.get("pre_final_frontier_size"),
        final_visited_count: row.get("final_visited_count"),
        final_emitted_count: row.get("final_emitted_count"),
        exact_reranked_candidates: row.get("exact_reranked_candidates"),
        quantized_reranked_candidates: row.get("quantized_reranked_candidates"),
        candidates_dropped_before_exact_rerank: row.get("candidates_dropped_before_exact_rerank"),
        truth_top10_in_frontier: row.get("truth_top10_in_frontier"),
        truth_top100_in_frontier: row.get("truth_top100_in_frontier"),
        truth_top10_row_indices: row.get("truth_top10_row_indices"),
        frontier_row_indices: row.get("frontier_row_indices"),
        frontier_approx_scores: row.get("frontier_approx_scores"),
        frontier_exact_scores: row.get("frontier_exact_scores"),
        frontier_approx_ranks: row.get("frontier_approx_ranks"),
        frontier_exact_ranks: row.get("frontier_exact_ranks"),
        final_emitted_row_indices: row.get("final_emitted_row_indices"),
        frontier_equals_final_emitted: row.get("frontier_equals_final_emitted"),
    }
}

fn summarize_frontier_rows(ef_search: i32, rows: &[FrontierRow]) -> FrontierSummary {
    let queries = rows.len();
    if queries == 0 {
        return FrontierSummary {
            ef_search,
            queries,
            recall_top10_in_frontier: 0.0,
            recall_top100_in_frontier: 0.0,
            mean_visited_before_output: 0.0,
            mean_pre_final_frontier_size: 0.0,
            mean_final_visited_count: 0.0,
            mean_final_emitted_count: 0.0,
            mean_exact_reranked_candidates: 0.0,
            mean_quantized_reranked_candidates: 0.0,
            mean_candidates_dropped_before_exact_rerank: 0.0,
            all_frontier_equals_final_emitted: true,
        };
    }
    let denom = queries as f64;
    FrontierSummary {
        ef_search,
        queries,
        recall_top10_in_frontier: rows
            .iter()
            .map(|row| f64::from(row.truth_top10_in_frontier))
            .sum::<f64>()
            / (denom * 10.0),
        recall_top100_in_frontier: rows
            .iter()
            .map(|row| f64::from(row.truth_top100_in_frontier))
            .sum::<f64>()
            / (denom * 100.0),
        mean_visited_before_output: mean_i32(rows, |row| row.visited_before_output),
        mean_pre_final_frontier_size: mean_i32(rows, |row| row.pre_final_frontier_size),
        mean_final_visited_count: mean_i32(rows, |row| row.final_visited_count),
        mean_final_emitted_count: mean_i32(rows, |row| row.final_emitted_count),
        mean_exact_reranked_candidates: mean_i32(rows, |row| row.exact_reranked_candidates),
        mean_quantized_reranked_candidates: mean_i32(rows, |row| row.quantized_reranked_candidates),
        mean_candidates_dropped_before_exact_rerank: mean_i32(rows, |row| {
            row.candidates_dropped_before_exact_rerank
        }),
        all_frontier_equals_final_emitted: rows.iter().all(|row| row.frontier_equals_final_emitted),
    }
}

fn mean_i32(rows: &[FrontierRow], value: impl Fn(&FrontierRow) -> i32) -> f64 {
    rows.iter().map(|row| f64::from(value(row))).sum::<f64>() / rows.len() as f64
}

fn render_summary(prefix: &str, index_name: &str, m: i32, summaries: &[FrontierSummary]) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "ef_search",
        "queries",
        "truth@10 in emitted pool",
        "truth@100 in emitted pool",
        "visited after rescan",
        "emitted pool",
        "visited final",
        "emitted",
        "exact rerank",
        "quantized rerank",
        "pool dropped before exact",
        "pool == emitted",
    ]);
    for summary in summaries {
        table.add_row(vec![
            Cell::new(summary.ef_search),
            Cell::new(summary.queries),
            Cell::new(format!("{:.4}", summary.recall_top10_in_frontier)),
            Cell::new(format!("{:.4}", summary.recall_top100_in_frontier)),
            Cell::new(format!("{:.1}", summary.mean_visited_before_output)),
            Cell::new(format!("{:.1}", summary.mean_pre_final_frontier_size)),
            Cell::new(format!("{:.1}", summary.mean_final_visited_count)),
            Cell::new(format!("{:.1}", summary.mean_final_emitted_count)),
            Cell::new(format!("{:.1}", summary.mean_exact_reranked_candidates)),
            Cell::new(format!("{:.1}", summary.mean_quantized_reranked_candidates)),
            Cell::new(format!(
                "{:.1}",
                summary.mean_candidates_dropped_before_exact_rerank
            )),
            Cell::new(summary.all_frontier_equals_final_emitted),
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

    fn row(ef_search: i32, top10: i32, top100: i32, frontier: i32) -> FrontierRow {
        FrontierRow {
            prefix: "p".into(),
            corpus_table: "p_corpus".into(),
            query_table: "p_queries".into(),
            index_name: "p_m16_idx".into(),
            m: 16,
            ef_search,
            query_index: 0,
            visited_before_output: frontier,
            pre_final_frontier_size: frontier,
            final_visited_count: frontier + 1,
            final_emitted_count: 10,
            exact_reranked_candidates: 4,
            quantized_reranked_candidates: 2,
            candidates_dropped_before_exact_rerank: frontier - 4,
            truth_top10_in_frontier: top10,
            truth_top100_in_frontier: top100,
            truth_top10_row_indices: Vec::new(),
            frontier_row_indices: Vec::new(),
            frontier_approx_scores: Vec::new(),
            frontier_exact_scores: Vec::new(),
            frontier_approx_ranks: Vec::new(),
            frontier_exact_ranks: Vec::new(),
            final_emitted_row_indices: Vec::new(),
            frontier_equals_final_emitted: true,
        }
    }

    #[test]
    fn summarize_frontier_rows_computes_recall_and_means() {
        let rows = vec![row(200, 5, 40, 100), row(200, 7, 50, 120)];
        let summary = summarize_frontier_rows(200, &rows);
        assert_eq!(summary.queries, 2);
        assert!((summary.recall_top10_in_frontier - 0.6).abs() < 1e-9);
        assert!((summary.recall_top100_in_frontier - 0.45).abs() < 1e-9);
        assert!((summary.mean_pre_final_frontier_size - 110.0).abs() < 1e-9);
        assert!((summary.mean_final_visited_count - 111.0).abs() < 1e-9);
    }
}
