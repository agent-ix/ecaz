use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;
use std::hint::black_box;
use std::path::Path;
use std::time::{Duration, Instant};

use tqvector::bench_api::ProdQuantizer;

const DIM: usize = 1536;
const BITS: u8 = 4;

#[derive(Debug, Clone)]
struct Config {
    corpus_size: usize,
    query_count: usize,
    clusters: usize,
    spread: f32,
    seed: u64,
    top_k: usize,
    bench_iters: usize,
    corpus_file: Option<String>,
    queries_file: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            corpus_size: 10_000,
            query_count: 20,
            clusters: 50,
            spread: 0.3,
            seed: 42,
            top_k: 10,
            bench_iters: 8,
            corpus_file: None,
            queries_file: None,
        }
    }
}

fn main() {
    let config = parse_args();
    let quantizer = ProdQuantizer::new(DIM, BITS, config.seed);

    let (source_label, corpus, queries) = if let (Some(corpus_file), Some(queries_file)) = (
        config.corpus_file.as_deref(),
        config.queries_file.as_deref(),
    ) {
        let corpus = load_vectors_from_tsv(corpus_file);
        let queries = load_vectors_from_tsv(queries_file);
        (
            format!(
                "tsv:{}:{}",
                basename(corpus_file),
                basename(queries_file)
            ),
            corpus,
            queries,
        )
    } else {
        let corpus = random_clustered_corpus(
            DIM,
            config.corpus_size,
            config.clusters,
            config.spread,
            config.seed,
        );
        let queries = random_clustered_corpus(
            DIM,
            config.query_count,
            config.clusters,
            config.spread,
            config.seed + 500_000,
        );
        ("synthetic_clustered".to_string(), corpus, queries)
    };
    assert!(
        config.query_count <= queries.len(),
        "--query-count exceeds available queries: requested {}, have {}",
        config.query_count,
        queries.len()
    );
    let queries = queries.into_iter().take(config.query_count).collect::<Vec<_>>();
    assert!(
        !corpus.is_empty(),
        "study corpus must contain at least one vector"
    );
    assert!(
        corpus.iter().all(|vector| vector.len() == DIM),
        "all corpus vectors must have dimension {DIM}"
    );
    assert!(
        queries.iter().all(|vector| vector.len() == DIM),
        "all query vectors must have dimension {DIM}"
    );
    let corpus_len = corpus.len();
    assert!(config.top_k <= corpus_len, "--top-k must be <= corpus size");
    let codes: Vec<Vec<u8>> = corpus
        .iter()
        .map(|vector| quantizer.encode(vector).mse_packed)
        .collect();

    let capture_limits = [20_usize, 50, 100, 200, 500, 1_000]
        .into_iter()
        .filter(|limit| *limit <= corpus_len)
        .collect::<Vec<_>>();

    let mut spearman_sum = 0.0_f32;
    let mut spearman_min = 1.0_f32;
    let mut pearson_sum = 0.0_f32;
    let mut pearson_min = 1.0_f32;
    let mut top_k_overlap_sum = 0.0_f32;
    let mut capture_sums = vec![0.0_f32; capture_limits.len()];

    for query in &queries {
        let exact_prepared = quantizer.prepare_ip_query(query);
        let approx_prepared = quantizer.prepare_ip_query_int8_approx_no_qjl_4bit(query);

        let mut exact_scores = Vec::with_capacity(codes.len());
        let mut approx_scores = Vec::with_capacity(codes.len());
        for code in &codes {
            exact_scores.push(quantizer.score_ip_from_parts(&exact_prepared, 0.0, code));
            approx_scores.push(
                quantizer.score_ip_from_parts_int8_approx_no_qjl_4bit(&approx_prepared, code),
            );
        }

        let exact_order = sort_indices_desc(&exact_scores);
        let approx_order = sort_indices_desc(&approx_scores);
        let spearman = spearman_rank_correlation(&exact_order, &approx_order);
        let pearson = pearson_correlation(&exact_scores, &approx_scores);

        spearman_sum += spearman;
        spearman_min = spearman_min.min(spearman);
        pearson_sum += pearson;
        pearson_min = pearson_min.min(pearson);
        top_k_overlap_sum +=
            overlap_fraction(&exact_order[..config.top_k], &approx_order[..config.top_k]);

        for (index, limit) in capture_limits.iter().enumerate() {
            capture_sums[index] +=
                capture_fraction(&exact_order[..config.top_k], &approx_order[..*limit]);
        }
    }

    let exact_prepared = quantizer.prepare_ip_query(&queries[0]);
    let approx_prepared = quantizer.prepare_ip_query_int8_approx_no_qjl_4bit(&queries[0]);
    let exact_elapsed = time_scores(config.bench_iters, || {
        let mut sum = 0.0_f32;
        for code in &codes {
            sum += quantizer.score_ip_from_parts(&exact_prepared, 0.0, code);
        }
        black_box(sum);
    });
    let approx_elapsed = time_scores(config.bench_iters, || {
        let mut sum = 0.0_f32;
        for code in &codes {
            sum += quantizer.score_ip_from_parts_int8_approx_no_qjl_4bit(&approx_prepared, code);
        }
        black_box(sum);
    });

    let score_count = (codes.len() * config.bench_iters) as f64;
    let exact_ns_per_score = exact_elapsed.as_secs_f64() * 1e9 / score_count;
    let approx_ns_per_score = approx_elapsed.as_secs_f64() * 1e9 / score_count;

    println!("study=int8_approx_no_qjl_4bit");
    println!(
        "dim={DIM} bits={BITS} corpus={} queries={} clusters={} spread={:.3} seed={}",
        corpus_len, config.query_count, config.clusters, config.spread, config.seed
    );
    println!("source={source_label}");
    println!(
        "spearman_rho mean={:.4} min={:.4}",
        spearman_sum / config.query_count as f32,
        spearman_min
    );
    println!(
        "pearson_r mean={:.4} min={:.4}",
        pearson_sum / config.query_count as f32,
        pearson_min
    );
    println!(
        "top{}_overlap mean={:.4}",
        config.top_k,
        top_k_overlap_sum / config.query_count as f32
    );
    for (index, limit) in capture_limits.iter().enumerate() {
        println!(
            "exact_top{}_captured_by_approx_top{} mean={:.4}",
            config.top_k,
            limit,
            capture_sums[index] / config.query_count as f32
        );
    }
    println!(
        "microbench exact_ns_per_score={:.1} approx_ns_per_score={:.1} speedup={:.2}x",
        exact_ns_per_score,
        approx_ns_per_score,
        exact_ns_per_score / approx_ns_per_score.max(f64::EPSILON)
    );
}

fn parse_args() -> Config {
    let mut config = Config::default();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--corpus-size" => config.corpus_size = parse_usize_arg("--corpus-size", args.next()),
            "--query-count" => config.query_count = parse_usize_arg("--query-count", args.next()),
            "--clusters" => config.clusters = parse_usize_arg("--clusters", args.next()),
            "--spread" => config.spread = parse_f32_arg("--spread", args.next()),
            "--seed" => config.seed = parse_u64_arg("--seed", args.next()),
            "--top-k" => config.top_k = parse_usize_arg("--top-k", args.next()),
            "--bench-iters" => config.bench_iters = parse_usize_arg("--bench-iters", args.next()),
            "--corpus-file" => config.corpus_file = Some(parse_string_arg("--corpus-file", args.next())),
            "--queries-file" => config.queries_file = Some(parse_string_arg("--queries-file", args.next())),
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            other => panic!("unknown arg: {other}"),
        }
    }

    assert!(config.top_k > 0, "--top-k must be positive");
    assert!(config.query_count > 0, "--query-count must be positive");
    assert!(config.clusters > 0, "--clusters must be positive");
    assert!(config.bench_iters > 0, "--bench-iters must be positive");
    assert_eq!(
        config.corpus_file.is_some(),
        config.queries_file.is_some(),
        "--corpus-file and --queries-file must be supplied together"
    );
    config
}

fn print_help() {
    println!("Usage: cargo run --bin approx_score_study -- [options]");
    println!("  --corpus-size <n>   default: 10000");
    println!("  --query-count <n>   default: 20");
    println!("  --clusters <n>      default: 50");
    println!("  --spread <f32>      default: 0.3");
    println!("  --seed <u64>        default: 42");
    println!("  --top-k <n>         default: 10");
    println!("  --bench-iters <n>   default: 8");
    println!("  --corpus-file <tsv> optional: real-corpus TSV with `id<TAB>comma,separated,floats`");
    println!("  --queries-file <tsv> optional: query TSV with `id<TAB>comma,separated,floats`");
}

fn parse_usize_arg(flag: &str, value: Option<String>) -> usize {
    value
        .unwrap_or_else(|| panic!("{flag} requires a value"))
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("{flag} requires an integer"))
}

fn parse_u64_arg(flag: &str, value: Option<String>) -> u64 {
    value
        .unwrap_or_else(|| panic!("{flag} requires a value"))
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("{flag} requires an integer"))
}

fn parse_f32_arg(flag: &str, value: Option<String>) -> f32 {
    value
        .unwrap_or_else(|| panic!("{flag} requires a value"))
        .parse::<f32>()
        .unwrap_or_else(|_| panic!("{flag} requires a float"))
}

fn parse_string_arg(flag: &str, value: Option<String>) -> String {
    value.unwrap_or_else(|| panic!("{flag} requires a value"))
}

fn random_unit_vector(dim: usize, seed: u64) -> Vec<f32> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut values: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let norm = values.iter().map(|value| value * value).sum::<f32>().sqrt();
    for value in &mut values {
        *value /= norm.max(f32::EPSILON);
    }
    values
}

fn random_clustered_corpus(
    dim: usize,
    n: usize,
    n_clusters: usize,
    spread: f32,
    seed: u64,
) -> Vec<Vec<f32>> {
    let centers: Vec<Vec<f32>> = (0..n_clusters)
        .map(|index| random_unit_vector(dim, seed + 100_000 + index as u64))
        .collect();
    let mut rng = ChaCha8Rng::seed_from_u64(seed + 200_000);
    let mut corpus = Vec::with_capacity(n);

    for index in 0..n {
        let center = &centers[index % n_clusters];
        let mut vector: Vec<f32> = center
            .iter()
            .map(|center_value| {
                let u1: f32 = rng.gen_range(0.0001_f32..1.0);
                let u2: f32 = rng.gen_range(0.0_f32..std::f32::consts::TAU);
                let noise = (-2.0 * u1.ln()).sqrt() * u2.cos() * spread;
                *center_value + noise
            })
            .collect();
        let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
        for value in &mut vector {
            *value /= norm.max(f32::EPSILON);
        }
        corpus.push(vector);
    }

    corpus
}

fn load_vectors_from_tsv(path: &str) -> Vec<Vec<f32>> {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_vector_tsv_line)
        .collect()
}

fn parse_vector_tsv_line(line: &str) -> Vec<f32> {
    let vector_text = line
        .split_once('\t')
        .map(|(_, vector)| vector)
        .unwrap_or(line)
        .trim();
    assert!(
        !vector_text.is_empty(),
        "vector TSV line must contain comma-separated floats"
    );
    vector_text
        .split(',')
        .map(|value| {
            value
                .trim()
                .parse::<f32>()
                .unwrap_or_else(|error| panic!("failed to parse float `{value}`: {error}"))
        })
        .collect()
}

fn basename(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(path)
}

fn sort_indices_desc(values: &[f32]) -> Vec<usize> {
    let mut indices = (0..values.len()).collect::<Vec<_>>();
    indices.sort_by(|left, right| {
        values[*right]
            .partial_cmp(&values[*left])
            .expect("scores should be comparable")
    });
    indices
}

fn spearman_rank_correlation(exact_order: &[usize], approx_order: &[usize]) -> f32 {
    let n = exact_order.len().min(approx_order.len());
    if n < 2 {
        return 0.0;
    }

    let mut approx_rank = vec![0usize; n];
    for (rank, index) in approx_order.iter().copied().enumerate().take(n) {
        approx_rank[index] = rank;
    }

    let mut d_squared_sum = 0.0_f64;
    for (exact_rank, index) in exact_order.iter().copied().enumerate().take(n) {
        let delta = exact_rank as f64 - approx_rank[index] as f64;
        d_squared_sum += delta * delta;
    }

    let n = n as f64;
    1.0 - (6.0 * d_squared_sum / (n * (n * n - 1.0))) as f32
}

fn pearson_correlation(exact_scores: &[f32], approx_scores: &[f32]) -> f32 {
    let n = exact_scores.len().min(approx_scores.len());
    if n == 0 {
        return 0.0;
    }

    let exact_mean = exact_scores.iter().take(n).sum::<f32>() / n as f32;
    let approx_mean = approx_scores.iter().take(n).sum::<f32>() / n as f32;
    let mut covariance = 0.0_f32;
    let mut exact_var = 0.0_f32;
    let mut approx_var = 0.0_f32;

    for (exact, approx) in exact_scores.iter().zip(approx_scores.iter()).take(n) {
        let exact_centered = *exact - exact_mean;
        let approx_centered = *approx - approx_mean;
        covariance += exact_centered * approx_centered;
        exact_var += exact_centered * exact_centered;
        approx_var += approx_centered * approx_centered;
    }

    covariance / (exact_var.sqrt() * approx_var.sqrt()).max(f32::EPSILON)
}

fn overlap_fraction(exact_top: &[usize], approx_top: &[usize]) -> f32 {
    let exact = exact_top.iter().copied().collect::<HashSet<_>>();
    let approx = approx_top.iter().copied().collect::<HashSet<_>>();
    exact.intersection(&approx).count() as f32 / exact_top.len().max(1) as f32
}

fn capture_fraction(exact_top: &[usize], approx_survivors: &[usize]) -> f32 {
    let survivors = approx_survivors.iter().copied().collect::<HashSet<_>>();
    exact_top
        .iter()
        .filter(|index| survivors.contains(index))
        .count() as f32
        / exact_top.len().max(1) as f32
}

fn time_scores(iterations: usize, mut scorer: impl FnMut()) -> Duration {
    for _ in 0..2 {
        scorer();
    }

    let start = Instant::now();
    for _ in 0..iterations {
        scorer();
    }
    start.elapsed()
}
