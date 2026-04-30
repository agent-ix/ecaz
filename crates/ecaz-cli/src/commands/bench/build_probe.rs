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
    approximate_medoid, bfs_reachable, build_vamana_graph_with_stats, MetricSummary,
    VamanaBuildStats,
};
use ndarray::Array2;

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

    let corpus_table = format!("{}_corpus", args.prefix);
    let client = psql::connect(conn).await?;
    if !psql::relation_exists(&client, &corpus_table, 'r').await? {
        return Err(eyre!("no corpus table {:?} in this database", corpus_table));
    }

    let fetch_started = Instant::now();
    let (_ids, corpus) =
        super::recall::fetch_sources_public(&client, &corpus_table, args.rows_limit).await?;
    let fetch_elapsed = fetch_started.elapsed();
    if corpus.nrows() == 0 {
        return Err(eyre!("corpus table {corpus_table} is empty"));
    }

    let dist = |a: u32, b: u32| unit_ip_distance(&corpus, a, b);
    let medoid_started = Instant::now();
    let medoid = approximate_medoid(corpus.nrows(), args.medoid_sample_cap, args.seed, dist);
    let medoid_elapsed = medoid_started.elapsed();

    let build_started = Instant::now();
    let (graph, stats) = build_vamana_graph_with_stats(
        corpus.nrows(),
        medoid,
        args.graph_degree,
        args.build_list_size,
        args.alpha,
        args.seed,
        dist,
    );
    let build_elapsed = build_started.elapsed();
    let reachable = bfs_reachable(&graph, medoid).len();

    let rendered = render_probe(
        &args,
        &corpus_table,
        corpus.nrows(),
        corpus.ncols(),
        reachable,
        fetch_elapsed.as_secs_f64(),
        medoid_elapsed.as_secs_f64(),
        build_elapsed.as_secs_f64(),
        &stats,
    );
    crate::ecaz_println!("{rendered}");
    if let Some(path) = args.log_output {
        std::fs::write(&path, &rendered)
            .wrap_err_with(|| format!("writing DiskANN build probe to {}", path.display()))?;
    }
    Ok(())
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
    reachable: usize,
    fetch_seconds: f64,
    medoid_seconds: f64,
    build_seconds: f64,
    stats: &VamanaBuildStats,
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
        Cell::new("build_seconds"),
        Cell::new(format!("{build_seconds:.3}")),
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
    write!(out, "{degree}").expect("writing to String should not fail");
    out
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
        let rendered = render_probe(&args, "p_corpus", 2, 3, 2, 0.1, 0.2, 0.3, &stats);
        assert!(rendered.contains("DiskANN build probe"));
        assert!(rendered.contains("reachable_fraction"));
        assert!(rendered.contains("visited mean/p95"));
        assert!(rendered.contains("reprunes"));
    }
}
