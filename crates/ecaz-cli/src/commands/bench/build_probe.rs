//! `ecaz bench diskann-build-probe` — in-memory Vamana build diagnostics.
//!
//! This command replays the DiskANN Vamana build core over a loaded
//! `<prefix>_corpus.source` table and reports candidate-generation and
//! pruning counters. It is intentionally outside PostgreSQL's index build
//! callback so tuning packets can isolate algorithm shape from page I/O.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::Instant;

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use ecaz::bench_api::{
    approximate_medoid, bfs_reachable, build_vamana_graph_with_pass1_extra_candidates,
    build_vamana_graph_with_stats, greedy_search, MetricSummary, VamanaBuildStats, VamanaGraph,
};
use ndarray::Array2;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use rayon::prelude::*;

use crate::{
    profiles,
    psql::{self, ConnectionOptions},
};

#[derive(Args, Debug)]
pub struct BuildProbeArgs {
    /// Prefix identifying the corpus.
    #[arg(long)]
    pub prefix: String,
    /// Cap corpus rows fetched from `<prefix>_corpus`.
    #[arg(long)]
    pub rows_limit: Option<usize>,
    /// Vamana graph degree R.
    #[arg(long, default_value_t = 32)]
    pub graph_degree: usize,
    /// Vamana build search list size L.
    #[arg(long, default_value_t = 100)]
    pub build_list_size: usize,
    /// Final alpha pruning value. The first build pass always uses alpha=1.0.
    #[arg(long, default_value_t = 1.2)]
    pub alpha: f32,
    /// Deterministic seed used for medoid sampling and build permutation.
    #[arg(long, default_value_t = 42)]
    pub seed: u64,
    /// Maximum rows sampled for approximate medoid selection.
    #[arg(long, default_value_t = 1024)]
    pub medoid_sample_cap: usize,
    /// Probe-only: add this many nearest nodes from a fixed global sample to
    /// each pivot's pass-1 candidate pool. Zero preserves the production build.
    #[arg(long, default_value_t = 0)]
    pub pass1_sample_candidates: usize,
    /// Probe-only: global sample size used by --pass1-sample-candidates.
    #[arg(long, default_value_t = 1024)]
    pub pass1_sample_pool_size: usize,
    /// k for in-memory recall@k over `<prefix>_queries`.
    #[arg(long, default_value_t = 10)]
    pub recall_k: usize,
    /// Search list size for in-memory graph recall.
    #[arg(long, default_value_t = 100)]
    pub scan_list_size: usize,
    /// Cap query rows fetched from `<prefix>_queries`.
    #[arg(long)]
    pub queries_limit: Option<usize>,
    /// Emit exact/in-memory/SQL result IDs for this many query rows.
    #[arg(long, default_value_t = 0)]
    pub compare_queries: usize,
    /// Write the rendered diagnostics to this path in addition to stdout.
    #[arg(long)]
    pub log_output: Option<PathBuf>,
}

pub async fn run(conn: &ConnectionOptions, args: BuildProbeArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    if args.graph_degree == 0 {
        return Err(eyre!("--graph-degree must be >= 1"));
    }
    if args.build_list_size == 0 {
        return Err(eyre!("--build-list-size must be >= 1"));
    }
    if !(1.0..=2.0).contains(&args.alpha) {
        return Err(eyre!("--alpha must be between 1.0 and 2.0"));
    }
    if args.medoid_sample_cap == 0 {
        return Err(eyre!("--medoid-sample-cap must be >= 1"));
    }
    if args.pass1_sample_candidates > 0 && args.pass1_sample_pool_size == 0 {
        return Err(eyre!(
            "--pass1-sample-pool-size must be >= 1 when --pass1-sample-candidates is set"
        ));
    }
    if args.recall_k == 0 {
        return Err(eyre!("--recall-k must be >= 1"));
    }
    if args.scan_list_size == 0 {
        return Err(eyre!("--scan-list-size must be >= 1"));
    }

    let corpus_table = format!("{}_corpus", args.prefix);
    let queries_table = format!("{}_queries", args.prefix);
    let client = psql::connect(conn).await?;
    if !psql::relation_exists(&client, &corpus_table, 'r').await? {
        return Err(eyre!("no corpus table {:?} in this database", corpus_table));
    }
    if !psql::relation_exists(&client, &queries_table, 'r').await? {
        return Err(eyre!(
            "no queries table {:?} in this database",
            queries_table
        ));
    }

    let fetch_started = Instant::now();
    let (corpus_ids, corpus) =
        super::recall::fetch_sources_public(&client, &corpus_table, args.rows_limit).await?;
    let (query_ids, queries) =
        super::recall::fetch_sources_public(&client, &queries_table, args.queries_limit).await?;
    let fetch_elapsed = fetch_started.elapsed();
    if corpus.nrows() == 0 {
        return Err(eyre!("corpus table {corpus_table} is empty"));
    }
    if queries.nrows() == 0 {
        return Err(eyre!("queries table {queries_table} is empty"));
    }
    if queries.ncols() != corpus.ncols() {
        return Err(eyre!(
            "{queries_table} dimensions {} do not match corpus dimensions {}",
            queries.ncols(),
            corpus.ncols()
        ));
    }

    let dist = |a: u32, b: u32| unit_ip_distance(&corpus, a, b);
    let medoid_started = Instant::now();
    let medoid = approximate_medoid(corpus.nrows(), args.medoid_sample_cap, args.seed, dist);
    let medoid_elapsed = medoid_started.elapsed();

    let augmentation_started = Instant::now();
    let pass1_extra_candidates = if args.pass1_sample_candidates == 0 {
        Vec::new()
    } else {
        build_pass1_sample_candidates(
            &corpus,
            args.pass1_sample_candidates,
            args.pass1_sample_pool_size,
            args.seed,
        )
    };
    let augmentation_elapsed = augmentation_started.elapsed();

    let build_started = Instant::now();
    let (graph, stats) = if pass1_extra_candidates.is_empty() {
        build_vamana_graph_with_stats(
            corpus.nrows(),
            medoid,
            args.graph_degree,
            args.build_list_size,
            args.alpha,
            args.seed,
            dist,
        )
    } else {
        build_vamana_graph_with_pass1_extra_candidates(
            corpus.nrows(),
            medoid,
            args.graph_degree,
            args.build_list_size,
            args.alpha,
            args.seed,
            &pass1_extra_candidates,
            dist,
        )
    };
    let build_elapsed = build_started.elapsed();
    let reachable = bfs_reachable(&graph, medoid).len();

    let recall_started = Instant::now();
    let recall = in_memory_recall_at_k(
        &graph,
        &corpus,
        &queries,
        medoid,
        args.scan_list_size,
        args.recall_k,
    );
    let recall_elapsed = recall_started.elapsed();

    let comparisons = compare_query_results(
        &client,
        &corpus_table,
        &corpus_ids,
        &corpus,
        &query_ids,
        &queries,
        &graph,
        medoid,
        args.scan_list_size,
        args.recall_k,
        args.compare_queries,
    )
    .await?;

    let rendered = render_probe(
        &args,
        &corpus_table,
        corpus.nrows(),
        corpus.ncols(),
        queries.nrows(),
        reachable,
        fetch_elapsed.as_secs_f64(),
        medoid_elapsed.as_secs_f64(),
        augmentation_elapsed.as_secs_f64(),
        build_elapsed.as_secs_f64(),
        recall_elapsed.as_secs_f64(),
        recall,
        &stats,
        &comparisons,
    );
    crate::ecaz_println!("{rendered}");
    if let Some(path) = args.log_output {
        std::fs::write(&path, &rendered)
            .wrap_err_with(|| format!("writing DiskANN build probe to {}", path.display()))?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueryComparison {
    query_id: i64,
    exact_ids: Vec<i64>,
    memory_ids: Vec<i64>,
    sql_ids: Vec<i64>,
    exact_memory_hits: usize,
    exact_sql_hits: usize,
    memory_sql_hits: usize,
}

async fn compare_query_results(
    client: &tokio_postgres::Client,
    corpus_table: &str,
    corpus_ids: &[i64],
    corpus: &Array2<f32>,
    query_ids: &[i64],
    queries: &Array2<f32>,
    graph: &VamanaGraph,
    medoid: u32,
    scan_list_size: usize,
    k: usize,
    compare_queries: usize,
) -> Result<Vec<QueryComparison>> {
    if compare_queries == 0 {
        return Ok(Vec::new());
    }
    psql::prefer_ordered_ann_path(client).await?;
    client
        .batch_execute(&format!("SET ec_diskann.list_size = {scan_list_size}"))
        .await
        .wrap_err_with(|| format!("SET ec_diskann.list_size = {scan_list_size}"))?;
    let sql = format!("SELECT id FROM {corpus_table} ORDER BY embedding <#> $1::real[] LIMIT $2");
    let limit = i64::try_from(k).wrap_err("--recall-k exceeds i64")?;
    let compare_count = compare_queries.min(queries.nrows());
    let mut comparisons = Vec::with_capacity(compare_count);
    for query_row in 0..compare_count {
        let exact_nodes = exact_top_k(corpus, queries, query_row, k);
        let memory_nodes = greedy_search(graph, medoid, scan_list_size, |node| {
            query_unit_ip_distance(corpus, queries, query_row, node)
        })
        .frontier
        .into_iter()
        .take(k)
        .map(|candidate| candidate.node)
        .collect::<Vec<_>>();
        let query_source = queries.row(query_row).to_vec();
        let sql_rows = client
            .query(sql.as_str(), &[&query_source, &limit])
            .await
            .wrap_err_with(|| format!("running SQL DiskANN query for row {query_row}"))?;
        let sql_ids = sql_rows
            .into_iter()
            .map(|row| row.get::<_, i64>(0))
            .collect::<Vec<_>>();
        let exact_ids = nodes_to_ids(corpus_ids, &exact_nodes);
        let memory_ids = nodes_to_ids(corpus_ids, &memory_nodes);
        comparisons.push(QueryComparison {
            query_id: query_ids[query_row],
            exact_memory_hits: overlap_count(&exact_ids, &memory_ids),
            exact_sql_hits: overlap_count(&exact_ids, &sql_ids),
            memory_sql_hits: overlap_count(&memory_ids, &sql_ids),
            exact_ids,
            memory_ids,
            sql_ids,
        });
    }
    Ok(comparisons)
}

fn nodes_to_ids(corpus_ids: &[i64], nodes: &[u32]) -> Vec<i64> {
    nodes
        .iter()
        .filter_map(|node| corpus_ids.get(*node as usize).copied())
        .collect()
}

fn overlap_count(left: &[i64], right: &[i64]) -> usize {
    let right = right
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    left.iter().filter(|id| right.contains(id)).count()
}

fn build_pass1_sample_candidates(
    corpus: &Array2<f32>,
    candidate_count: usize,
    sample_pool_size: usize,
    seed: u64,
) -> Vec<Vec<u32>> {
    let mut sample: Vec<u32> = (0..corpus.nrows() as u32).collect();
    let mut rng = StdRng::seed_from_u64(seed ^ 0xD15C_A117_BA5E_0001);
    sample.shuffle(&mut rng);
    sample.truncate(sample_pool_size.min(sample.len()));

    (0..corpus.nrows() as u32)
        .into_par_iter()
        .map(|pivot| nearest_from_sample(corpus, pivot, &sample, candidate_count))
        .collect()
}

fn nearest_from_sample(
    corpus: &Array2<f32>,
    pivot: u32,
    sample: &[u32],
    candidate_count: usize,
) -> Vec<u32> {
    let mut scored: Vec<(u32, f32)> = sample
        .iter()
        .copied()
        .filter(|node| *node != pivot)
        .map(|node| (node, unit_ip_distance(corpus, node, pivot)))
        .collect();
    scored.sort_by(|(left_node, left_dist), (right_node, right_dist)| {
        left_dist
            .partial_cmp(right_dist)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left_node.cmp(right_node))
    });
    scored
        .into_iter()
        .take(candidate_count)
        .map(|(node, _)| node)
        .collect()
}

fn in_memory_recall_at_k(
    graph: &VamanaGraph,
    corpus: &Array2<f32>,
    queries: &Array2<f32>,
    medoid: u32,
    scan_list_size: usize,
    k: usize,
) -> f64 {
    let k = k.min(corpus.nrows());
    if k == 0 || queries.nrows() == 0 {
        return 0.0;
    }
    let hits: usize = (0..queries.nrows())
        .into_par_iter()
        .map(|query_row| {
            let exact = exact_top_k(corpus, queries, query_row, k);
            let exact: std::collections::HashSet<u32> = exact.into_iter().collect();
            let result = greedy_search(graph, medoid, scan_list_size, |node| {
                query_unit_ip_distance(corpus, queries, query_row, node)
            });
            result
                .frontier
                .iter()
                .take(k)
                .filter(|candidate| exact.contains(&candidate.node))
                .count()
        })
        .sum();
    hits as f64 / (queries.nrows() * k) as f64
}

fn exact_top_k(
    corpus: &Array2<f32>,
    queries: &Array2<f32>,
    query_row: usize,
    k: usize,
) -> Vec<u32> {
    let mut scored: Vec<(u32, f32)> = (0..corpus.nrows() as u32)
        .map(|node| (node, query_inner_product(corpus, queries, query_row, node)))
        .collect();
    scored.sort_by(|(left_node, left_score), (right_node, right_score)| {
        right_score
            .partial_cmp(left_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left_node.cmp(right_node))
    });
    scored.into_iter().take(k).map(|(node, _)| node).collect()
}

fn query_unit_ip_distance(
    corpus: &Array2<f32>,
    queries: &Array2<f32>,
    query_row: usize,
    node: u32,
) -> f32 {
    (1.0 - query_inner_product(corpus, queries, query_row, node)).max(0.0)
}

fn query_inner_product(
    corpus: &Array2<f32>,
    queries: &Array2<f32>,
    query_row: usize,
    node: u32,
) -> f32 {
    queries
        .row(query_row)
        .iter()
        .zip(corpus.row(node as usize).iter())
        .map(|(left, right)| left * right)
        .sum()
}

fn unit_ip_distance(corpus: &Array2<f32>, a: u32, b: u32) -> f32 {
    let ip: f32 = corpus
        .row(a as usize)
        .iter()
        .zip(corpus.row(b as usize).iter())
        .map(|(left, right)| left * right)
        .sum();
    (1.0 - ip).max(0.0)
}

fn render_probe(
    args: &BuildProbeArgs,
    corpus_table: &str,
    rows: usize,
    dim: usize,
    query_rows: usize,
    reachable: usize,
    fetch_seconds: f64,
    medoid_seconds: f64,
    augmentation_seconds: f64,
    build_seconds: f64,
    recall_seconds: f64,
    recall: f64,
    stats: &VamanaBuildStats,
    comparisons: &[QueryComparison],
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "DiskANN build probe for prefix={} corpus={corpus_table}",
        args.prefix
    )
    .expect("writing to String should not fail");

    let mut header = Table::new();
    header.load_preset(UTF8_FULL);
    header.set_header(vec!["field", "value"]);
    header.add_row(vec![Cell::new("rows"), Cell::new(rows)]);
    header.add_row(vec![Cell::new("dimensions"), Cell::new(dim)]);
    header.add_row(vec![Cell::new("queries"), Cell::new(query_rows)]);
    header.add_row(vec![
        Cell::new("graph_degree"),
        Cell::new(args.graph_degree),
    ]);
    header.add_row(vec![
        Cell::new("build_list_size"),
        Cell::new(args.build_list_size),
    ]);
    header.add_row(vec![
        Cell::new("alpha"),
        Cell::new(format!("{:.3}", args.alpha)),
    ]);
    header.add_row(vec![Cell::new("seed"), Cell::new(args.seed)]);
    header.add_row(vec![Cell::new("medoid"), Cell::new(stats.medoid)]);
    header.add_row(vec![
        Cell::new("medoid_sample_cap"),
        Cell::new(args.medoid_sample_cap),
    ]);
    header.add_row(vec![
        Cell::new("pass1_sample_candidates"),
        Cell::new(args.pass1_sample_candidates),
    ]);
    header.add_row(vec![
        Cell::new("pass1_sample_pool_size"),
        Cell::new(args.pass1_sample_pool_size),
    ]);
    header.add_row(vec![
        Cell::new("scan_list_size"),
        Cell::new(args.scan_list_size),
    ]);
    header.add_row(vec![Cell::new("recall_k"), Cell::new(args.recall_k)]);
    header.add_row(vec![Cell::new("reachable"), Cell::new(reachable)]);
    header.add_row(vec![
        Cell::new("reachable_fraction"),
        Cell::new(format!("{:.6}", reachable as f64 / rows as f64)),
    ]);
    header.add_row(vec![
        Cell::new("fetch_seconds"),
        Cell::new(format!("{fetch_seconds:.3}")),
    ]);
    header.add_row(vec![
        Cell::new("medoid_seconds"),
        Cell::new(format!("{medoid_seconds:.3}")),
    ]);
    header.add_row(vec![
        Cell::new("augmentation_seconds"),
        Cell::new(format!("{augmentation_seconds:.3}")),
    ]);
    header.add_row(vec![
        Cell::new("build_seconds"),
        Cell::new(format!("{build_seconds:.3}")),
    ]);
    header.add_row(vec![
        Cell::new("recall_seconds"),
        Cell::new(format!("{recall_seconds:.3}")),
    ]);
    header.add_row(vec![
        Cell::new(format!("recall@{}", args.recall_k)),
        Cell::new(format!("{recall:.4}")),
    ]);
    writeln!(out, "{header}").expect("writing to String should not fail");

    let mut passes = Table::new();
    passes.load_preset(UTF8_FULL);
    passes.set_header(vec![
        "pass",
        "alpha",
        "pivots",
        "visited mean/p95",
        "existing mean/p95",
        "pool mean/p95",
        "selected mean/p95",
        "backlinks",
        "reprunes",
    ]);
    for (idx, pass) in stats.passes.iter().enumerate() {
        passes.add_row(vec![
            Cell::new(idx + 1),
            Cell::new(format!("{:.3}", pass.alpha)),
            Cell::new(pass.pivot_count),
            Cell::new(mean_p95(pass.visited)),
            Cell::new(mean_p95(pass.existing_neighbors)),
            Cell::new(mean_p95(pass.candidate_pool)),
            Cell::new(mean_p95(pass.selected_neighbors)),
            Cell::new(pass.backlinks_added),
            Cell::new(pass.reprunes),
        ]);
    }
    writeln!(out, "{passes}").expect("writing to String should not fail");

    let mut degree = Table::new();
    degree.load_preset(UTF8_FULL);
    degree.set_header(vec!["direction", "min", "mean", "p50", "p95", "p99", "max"]);
    add_summary_row(&mut degree, "out", stats.final_out_degree);
    add_summary_row(&mut degree, "in", stats.final_in_degree);
    writeln!(out, "{degree}").expect("writing to String should not fail");

    if !comparisons.is_empty() {
        let mut compare = Table::new();
        compare.load_preset(UTF8_FULL);
        compare.set_header(vec![
            "query_id",
            "exact/memory",
            "exact/sql",
            "memory/sql",
            "exact ids",
            "memory ids",
            "sql ids",
        ]);
        for row in comparisons {
            compare.add_row(vec![
                Cell::new(row.query_id),
                Cell::new(format!("{}/{}", row.exact_memory_hits, args.recall_k)),
                Cell::new(format!("{}/{}", row.exact_sql_hits, args.recall_k)),
                Cell::new(format!("{}/{}", row.memory_sql_hits, args.recall_k)),
                Cell::new(join_ids(&row.exact_ids)),
                Cell::new(join_ids(&row.memory_ids)),
                Cell::new(join_ids(&row.sql_ids)),
            ]);
        }
        write!(out, "{compare}").expect("writing to String should not fail");
    } else {
        let _ = out.pop();
    }
    out
}

fn join_ids(ids: &[i64]) -> String {
    ids.iter().map(i64::to_string).collect::<Vec<_>>().join(",")
}

fn mean_p95(summary: MetricSummary) -> String {
    format!("{:.2}/{}", summary.mean, summary.p95)
}

fn add_summary_row(table: &mut Table, label: &str, summary: MetricSummary) {
    table.add_row(vec![
        Cell::new(label),
        Cell::new(summary.min),
        Cell::new(format!("{:.2}", summary.mean)),
        Cell::new(summary.p50),
        Cell::new(summary.p95),
        Cell::new(summary.p99),
        Cell::new(summary.max),
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecaz::bench_api::{VamanaBuildPassStats, VamanaBuildStats};

    #[test]
    fn unit_ip_distance_uses_one_minus_inner_product() {
        let corpus = Array2::from_shape_vec((2, 3), vec![1.0, 0.0, 0.0, 0.25, 0.5, 0.75]).unwrap();
        assert_eq!(unit_ip_distance(&corpus, 0, 1), 0.75);
        assert_eq!(unit_ip_distance(&corpus, 0, 0), 0.0);
    }

    #[test]
    fn render_probe_includes_pass_and_degree_summaries() {
        let args = BuildProbeArgs {
            prefix: "p".into(),
            rows_limit: None,
            graph_degree: 32,
            build_list_size: 100,
            alpha: 1.2,
            seed: 42,
            medoid_sample_cap: 1024,
            pass1_sample_candidates: 0,
            pass1_sample_pool_size: 1024,
            recall_k: 10,
            scan_list_size: 100,
            queries_limit: None,
            compare_queries: 0,
            log_output: None,
        };
        let summary = MetricSummary {
            count: 2,
            min: 1,
            mean: 2.5,
            p50: 2,
            p95: 4,
            p99: 4,
            max: 4,
        };
        let stats = VamanaBuildStats {
            node_count: 2,
            medoid: 1,
            max_degree: 32,
            list_size: 100,
            alpha_final: 1.2,
            seed: 42,
            passes: vec![VamanaBuildPassStats {
                alpha: 1.0,
                pivot_count: 2,
                visited: summary,
                existing_neighbors: summary,
                candidate_pool: summary,
                selected_neighbors: summary,
                backlinks_added: 3,
                reprunes: 4,
            }],
            final_out_degree: summary,
            final_in_degree: summary,
        };
        let rendered = render_probe(
            &args,
            "p_corpus",
            2,
            3,
            1,
            2,
            0.1,
            0.2,
            0.0,
            0.3,
            0.4,
            1.0,
            &stats,
            &[],
        );
        assert!(rendered.contains("DiskANN build probe"));
        assert!(rendered.contains("reachable_fraction"));
        assert!(rendered.contains("recall@10"));
        assert!(rendered.contains("visited mean/p95"));
        assert!(rendered.contains("reprunes"));
    }

    #[test]
    fn render_probe_includes_optional_query_comparisons() {
        let args = BuildProbeArgs {
            prefix: "p".into(),
            rows_limit: None,
            graph_degree: 32,
            build_list_size: 100,
            alpha: 1.2,
            seed: 42,
            medoid_sample_cap: 1024,
            pass1_sample_candidates: 0,
            pass1_sample_pool_size: 1024,
            recall_k: 2,
            scan_list_size: 100,
            queries_limit: None,
            compare_queries: 1,
            log_output: None,
        };
        let summary = MetricSummary {
            count: 2,
            min: 1,
            mean: 2.5,
            p50: 2,
            p95: 4,
            p99: 4,
            max: 4,
        };
        let stats = VamanaBuildStats {
            node_count: 2,
            medoid: 1,
            max_degree: 32,
            list_size: 100,
            alpha_final: 1.2,
            seed: 42,
            passes: Vec::new(),
            final_out_degree: summary,
            final_in_degree: summary,
        };
        let comparison = QueryComparison {
            query_id: 10,
            exact_ids: vec![1, 2],
            memory_ids: vec![1, 3],
            sql_ids: vec![4, 5],
            exact_memory_hits: 1,
            exact_sql_hits: 0,
            memory_sql_hits: 0,
        };
        let rendered = render_probe(
            &args,
            "p_corpus",
            2,
            3,
            1,
            2,
            0.1,
            0.2,
            0.0,
            0.3,
            0.4,
            1.0,
            &stats,
            &[comparison],
        );
        assert!(rendered.contains("query_id"));
        assert!(rendered.contains("1,2"));
        assert!(rendered.contains("4,5"));
    }

    #[test]
    fn nearest_from_sample_skips_pivot_and_sorts_by_distance() {
        let corpus =
            Array2::from_shape_vec((4, 2), vec![1.0, 0.0, 0.9, 0.1, 0.0, 1.0, -1.0, 0.0]).unwrap();
        let nearest = nearest_from_sample(&corpus, 0, &[0, 1, 2, 3], 2);
        assert_eq!(nearest, vec![1, 2]);
    }

    #[test]
    fn in_memory_recall_scores_graph_search() {
        let corpus = Array2::from_shape_vec((3, 2), vec![1.0, 0.0, 0.0, 1.0, -1.0, 0.0]).unwrap();
        let queries = Array2::from_shape_vec((1, 2), vec![1.0, 0.0]).unwrap();
        let graph = VamanaGraph {
            neighbors: vec![vec![1, 2], vec![0], vec![0]],
            max_degree: 2,
        };
        let recall = in_memory_recall_at_k(&graph, &corpus, &queries, 1, 3, 1);
        assert_eq!(recall, 1.0);
    }

    #[test]
    fn overlap_count_counts_intersection() {
        assert_eq!(overlap_count(&[1, 2, 3], &[3, 4, 1]), 2);
    }
}
