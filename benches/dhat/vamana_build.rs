//! dhat heap profiling: Vamana build hot loop.
//!
//! This profiles `build_vamana_graph_with_stats` only. Input parsing and
//! medoid selection happen before the profiler starts so the output is focused
//! on build-time greedy search, prune, backlink repair, and graph storage.

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use ecaz::bench_api::{
    approximate_medoid, build_vamana_graph_with_stats,
    diskann_source_inner_product_scalar_reference,
};

#[derive(Debug)]
struct Config {
    input: PathBuf,
    rows: usize,
    graph_degree: usize,
    list_size: usize,
    alpha: f32,
    seed: u64,
    output: PathBuf,
    summary_output: PathBuf,
}

fn main() {
    let config = parse_args();
    let vectors = load_vectors(&config.input, config.rows);
    assert!(!vectors.is_empty(), "input produced no vectors");

    let node_count = vectors.len();
    let medoid = approximate_medoid(
        node_count,
        node_count.min(1024),
        config.seed,
        |left, right| vector_distance(&vectors[left as usize], &vectors[right as usize]),
    );

    let started = Instant::now();
    #[cfg(feature = "dhat-heap")]
    let profiler = dhat::Profiler::builder()
        .file_name(&config.output)
        .trim_backtraces(Some(20))
        .build();

    let (graph, stats) = build_vamana_graph_with_stats(
        node_count,
        medoid,
        config.graph_degree,
        config.list_size,
        config.alpha,
        config.seed,
        |left, right| vector_distance(&vectors[left as usize], &vectors[right as usize]),
    );

    #[cfg(feature = "dhat-heap")]
    drop(profiler);
    let elapsed = started.elapsed();

    let final_out_degree = graph
        .neighbors
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or_default();
    let pass = stats
        .passes
        .last()
        .expect("Vamana build should report a pass");

    let summary = format!(
        concat!(
            "# dhat Vamana Build Profile\n\n",
            "- input: `{}`\n",
            "- rows: `{}`\n",
            "- graph_degree: `{}`\n",
            "- build_list_size: `{}`\n",
            "- alpha: `{}`\n",
            "- seed: `{}`\n",
            "- medoid: `{}`\n",
            "- elapsed_ms: `{}`\n",
            "- max_out_degree: `{}`\n",
            "- stats_final_out_degree_max: `{}`\n",
            "- greedy_search_ms: `{}`\n",
            "- candidate_pool_ms: `{}`\n",
            "- robust_prune_ms: `{}`\n",
            "- backlink_ms: `{}`\n",
            "- visited_p95: `{}`\n",
            "- candidate_pool_p95: `{}`\n",
            "- dhat_output: `{}`\n"
        ),
        config.input.display(),
        node_count,
        config.graph_degree,
        config.list_size,
        config.alpha,
        config.seed,
        medoid,
        elapsed.as_millis(),
        final_out_degree,
        stats.final_out_degree.max,
        pass.greedy_search_ms,
        pass.candidate_pool_ms,
        pass.robust_prune_ms,
        pass.backlink_ms,
        pass.visited.p95,
        pass.candidate_pool.p95,
        config.output.display()
    );
    fs::write(&config.summary_output, summary).expect("write summary output");
}

fn parse_args() -> Config {
    let mut args = env::args().skip(1);
    let mut config = Config {
        input: PathBuf::new(),
        rows: 1000,
        graph_degree: 32,
        list_size: 100,
        alpha: 1.2,
        seed: 42,
        output: PathBuf::from("dhat-vamana-build.json"),
        summary_output: PathBuf::from("dhat-vamana-build-summary.md"),
    };

    while let Some(arg) = args.next() {
        let value = args
            .next()
            .unwrap_or_else(|| panic!("missing value for argument {arg}"));
        match arg.as_str() {
            "--input" => config.input = PathBuf::from(value),
            "--rows" => config.rows = value.parse().expect("parse --rows"),
            "--graph-degree" => config.graph_degree = value.parse().expect("parse --graph-degree"),
            "--list-size" => config.list_size = value.parse().expect("parse --list-size"),
            "--alpha" => config.alpha = value.parse().expect("parse --alpha"),
            "--seed" => config.seed = value.parse().expect("parse --seed"),
            "--output" => config.output = PathBuf::from(value),
            "--summary-output" => config.summary_output = PathBuf::from(value),
            other => panic!("unknown argument {other}"),
        }
    }

    assert!(!config.input.as_os_str().is_empty(), "--input is required");
    config
}

fn load_vectors(path: &PathBuf, limit: usize) -> Vec<Vec<f32>> {
    let input = fs::read_to_string(path).expect("read input TSV");
    input
        .lines()
        .take(limit)
        .map(|line| {
            let (_, vector) = line.split_once('\t').expect("TSV row with id and vector");
            serde_json::from_str::<Vec<f32>>(vector).expect("parse vector JSON")
        })
        .collect()
}

fn vector_distance(left: &[f32], right: &[f32]) -> f32 {
    (1.0 - diskann_source_inner_product_scalar_reference(left, right)).max(0.0)
}
