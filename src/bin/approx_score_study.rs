use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;
use std::hint::black_box;
use std::path::Path;
use std::time::{Duration, Instant};

use tqvector::bench_api::{
    build_grouped_pq_lut_f32 as shared_build_grouped_pq_lut_f32,
    encode_grouped_pq as shared_encode_grouped_pq, grouped_pq_nibble,
    grouped_pq_score_f32 as shared_grouped_pq_score_f32,
    nearest_centroid_l2 as shared_nearest_centroid_l2, pad_input, srht, ProdQuantizer,
};

const DIM: usize = 1536;
const BITS: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StudyMode {
    Int8Approx,
    BinarySign,
    GroupedMeanF32,
    GroupedMeanU8,
    GroupedPqF32,
    GroupedPqU8,
}

impl StudyMode {
    fn parse(value: &str) -> Self {
        match value {
            "int8-approx" => Self::Int8Approx,
            "binary-sign" => Self::BinarySign,
            "grouped-f32" => Self::GroupedMeanF32,
            "grouped-u8" => Self::GroupedMeanU8,
            "grouped-pq-f32" => Self::GroupedPqF32,
            "grouped-pq-u8" => Self::GroupedPqU8,
            other => panic!("unknown study mode: {other}"),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Int8Approx => "int8_approx_no_qjl_4bit",
            Self::BinarySign => "binary_sign_no_qjl_4bit",
            Self::GroupedMeanF32 => "grouped_mean_f32_no_qjl_4bit",
            Self::GroupedMeanU8 => "grouped_mean_u8_no_qjl_4bit",
            Self::GroupedPqF32 => "grouped_pq_f32_srht",
            Self::GroupedPqU8 => "grouped_pq_u8_srht",
        }
    }
}

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
    study_mode: StudyMode,
    group_size: usize,
    train_size: usize,
    kmeans_iters: usize,
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
            study_mode: StudyMode::Int8Approx,
            group_size: 16,
            train_size: 4096,
            kmeans_iters: 15,
        }
    }
}

#[derive(Debug, Clone)]
struct StudyAggregate {
    spearman_sum: f32,
    spearman_min: f32,
    pearson_sum: f32,
    pearson_min: f32,
    top_k_overlap_sum: f32,
    capture_sums: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupedScoreMode {
    F32,
    U8,
}

#[derive(Debug, Clone)]
struct GroupedMeanCode {
    counts: Vec<u8>,
}

#[derive(Debug, Clone)]
struct GroupedMeanPreparedQuery {
    lut_f32: Vec<f32>,
    lut_u8: Vec<u8>,
    row_bias: Vec<f32>,
    row_scale: Vec<f32>,
}

#[derive(Debug, Clone)]
struct GroupedPqCode {
    packed_nibbles: Vec<u8>,
}

#[derive(Debug, Clone)]
struct GroupedPqModel {
    codebooks: Vec<Vec<f32>>,
    group_count: usize,
    group_size: usize,
}

#[derive(Debug, Clone)]
struct GroupedPqPreparedQuery {
    lut_f32: Vec<f32>,
    lut_u8: Vec<u8>,
    row_bias: Vec<f32>,
    row_scale: Vec<f32>,
}

impl StudyAggregate {
    fn new(capture_len: usize) -> Self {
        Self {
            spearman_sum: 0.0,
            spearman_min: 1.0,
            pearson_sum: 0.0,
            pearson_min: 1.0,
            top_k_overlap_sum: 0.0,
            capture_sums: vec![0.0; capture_len],
        }
    }

    fn record(
        &mut self,
        exact_scores: &[f32],
        approx_scores: &[f32],
        top_k: usize,
        capture_limits: &[usize],
    ) {
        let exact_order = sort_indices_desc(exact_scores);
        let approx_order = sort_indices_desc(approx_scores);
        let spearman = spearman_rank_correlation(&exact_order, &approx_order);
        let pearson = pearson_correlation(exact_scores, approx_scores);

        self.spearman_sum += spearman;
        self.spearman_min = self.spearman_min.min(spearman);
        self.pearson_sum += pearson;
        self.pearson_min = self.pearson_min.min(pearson);
        self.top_k_overlap_sum += overlap_fraction(&exact_order[..top_k], &approx_order[..top_k]);

        for (index, limit) in capture_limits.iter().enumerate() {
            self.capture_sums[index] +=
                capture_fraction(&exact_order[..top_k], &approx_order[..*limit]);
        }
    }

    fn print(&self, query_count: usize, top_k: usize, capture_limits: &[usize]) {
        println!(
            "spearman_rho mean={:.4} min={:.4}",
            self.spearman_sum / query_count as f32,
            self.spearman_min
        );
        println!(
            "pearson_r mean={:.4} min={:.4}",
            self.pearson_sum / query_count as f32,
            self.pearson_min
        );
        println!(
            "top{}_overlap mean={:.4}",
            top_k,
            self.top_k_overlap_sum / query_count as f32
        );
        for (index, limit) in capture_limits.iter().enumerate() {
            println!(
                "exact_top{}_captured_by_approx_top{} mean={:.4}",
                top_k,
                limit,
                self.capture_sums[index] / query_count as f32
            );
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
            format!("tsv:{}:{}", basename(corpus_file), basename(queries_file)),
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
    let queries = queries
        .into_iter()
        .take(config.query_count)
        .collect::<Vec<_>>();
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

    println!("study={}", config.study_mode.label());
    println!(
        "dim={DIM} bits={BITS} corpus={} queries={} clusters={} spread={:.3} seed={}",
        corpus_len, config.query_count, config.clusters, config.spread, config.seed
    );
    println!("source={source_label}");
    if matches!(
        config.study_mode,
        StudyMode::GroupedPqF32 | StudyMode::GroupedPqU8
    ) {
        println!(
            "group_size={} train_size={} kmeans_iters={}",
            config.group_size, config.train_size, config.kmeans_iters
        );
    }
    match config.study_mode {
        StudyMode::Int8Approx => {
            run_int8_study(&config, &quantizer, &queries, &codes, &capture_limits)
        }
        StudyMode::BinarySign => {
            run_binary_sign_study(&config, &quantizer, &queries, &codes, &capture_limits)
        }
        StudyMode::GroupedMeanF32 => run_grouped_mean_study(
            &config,
            &quantizer,
            &queries,
            &codes,
            &capture_limits,
            GroupedScoreMode::F32,
        ),
        StudyMode::GroupedMeanU8 => run_grouped_mean_study(
            &config,
            &quantizer,
            &queries,
            &codes,
            &capture_limits,
            GroupedScoreMode::U8,
        ),
        StudyMode::GroupedPqF32 => run_grouped_pq_study(
            &config,
            &quantizer,
            &corpus,
            &queries,
            &capture_limits,
            GroupedScoreMode::F32,
        ),
        StudyMode::GroupedPqU8 => run_grouped_pq_study(
            &config,
            &quantizer,
            &corpus,
            &queries,
            &capture_limits,
            GroupedScoreMode::U8,
        ),
    }
}

fn run_int8_study(
    config: &Config,
    quantizer: &ProdQuantizer,
    queries: &[Vec<f32>],
    codes: &[Vec<u8>],
    capture_limits: &[usize],
) {
    let mut aggregate = StudyAggregate::new(capture_limits.len());

    for query in queries {
        let exact_prepared = quantizer.prepare_ip_query(query);
        let approx_prepared = quantizer.prepare_ip_query_int8_approx_no_qjl_4bit(query);

        let mut exact_scores = Vec::with_capacity(codes.len());
        let mut approx_scores = Vec::with_capacity(codes.len());
        for code in codes {
            exact_scores.push(quantizer.score_ip_from_parts(&exact_prepared, 0.0, code));
            approx_scores.push(
                quantizer.score_ip_from_parts_int8_approx_no_qjl_4bit(&approx_prepared, code),
            );
        }

        aggregate.record(&exact_scores, &approx_scores, config.top_k, capture_limits);
    }

    let exact_prepared = quantizer.prepare_ip_query(&queries[0]);
    let approx_prepared = quantizer.prepare_ip_query_int8_approx_no_qjl_4bit(&queries[0]);
    let exact_elapsed = time_scores(config.bench_iters, || {
        let mut sum = 0.0_f32;
        for code in codes {
            sum += quantizer.score_ip_from_parts(&exact_prepared, 0.0, code);
        }
        black_box(sum);
    });
    let approx_elapsed = time_scores(config.bench_iters, || {
        let mut sum = 0.0_f32;
        for code in codes {
            sum += quantizer.score_ip_from_parts_int8_approx_no_qjl_4bit(&approx_prepared, code);
        }
        black_box(sum);
    });

    let score_count = (codes.len() * config.bench_iters) as f64;
    let exact_ns_per_score = exact_elapsed.as_secs_f64() * 1e9 / score_count;
    let approx_ns_per_score = approx_elapsed.as_secs_f64() * 1e9 / score_count;

    aggregate.print(config.query_count, config.top_k, capture_limits);
    println!(
        "microbench exact_ns_per_score={:.1} approx_ns_per_score={:.1} speedup={:.2}x",
        exact_ns_per_score,
        approx_ns_per_score,
        exact_ns_per_score / approx_ns_per_score.max(f64::EPSILON)
    );
}

fn run_binary_sign_study(
    config: &Config,
    quantizer: &ProdQuantizer,
    queries: &[Vec<f32>],
    codes: &[Vec<u8>],
    capture_limits: &[usize],
) {
    let sign_lookup = sign_lookup_from_codebook(&quantizer.codebook);
    let binary_codes = codes
        .iter()
        .map(|code| binary_sign_words_from_packed(code, DIM, &sign_lookup))
        .collect::<Vec<_>>();
    let mut aggregate = StudyAggregate::new(capture_limits.len());

    for query in queries {
        let exact_prepared = quantizer.prepare_ip_query(query);
        let query_words = binary_sign_words_from_rotated(&exact_prepared.rotated);

        let mut exact_scores = Vec::with_capacity(codes.len());
        let mut approx_scores = Vec::with_capacity(codes.len());
        for (code, binary_code) in codes.iter().zip(binary_codes.iter()) {
            exact_scores.push(quantizer.score_ip_from_parts(&exact_prepared, 0.0, code));
            approx_scores.push(binary_sign_similarity(&query_words, binary_code, DIM));
        }

        aggregate.record(&exact_scores, &approx_scores, config.top_k, capture_limits);
    }

    let exact_prepared = quantizer.prepare_ip_query(&queries[0]);
    let query_words = binary_sign_words_from_rotated(&exact_prepared.rotated);
    let exact_elapsed = time_scores(config.bench_iters, || {
        let mut sum = 0.0_f32;
        for code in codes {
            sum += quantizer.score_ip_from_parts(&exact_prepared, 0.0, code);
        }
        black_box(sum);
    });
    let binary_cached_elapsed = time_scores(config.bench_iters, || {
        let mut sum = 0.0_f32;
        for binary_code in &binary_codes {
            sum += binary_sign_similarity(&query_words, binary_code, DIM);
        }
        black_box(sum);
    });
    let binary_derived_elapsed = time_scores(config.bench_iters, || {
        let mut sum = 0.0_f32;
        for code in codes {
            sum += binary_sign_similarity_from_packed(&query_words, code, DIM, &sign_lookup);
        }
        black_box(sum);
    });

    let score_count = (codes.len() * config.bench_iters) as f64;
    let exact_ns_per_score = exact_elapsed.as_secs_f64() * 1e9 / score_count;
    let binary_cached_ns_per_score = binary_cached_elapsed.as_secs_f64() * 1e9 / score_count;
    let binary_derived_ns_per_score = binary_derived_elapsed.as_secs_f64() * 1e9 / score_count;

    aggregate.print(config.query_count, config.top_k, capture_limits);
    println!(
        "microbench exact_ns_per_score={:.1} binary_cached_ns_per_score={:.1} binary_derived_ns_per_score={:.1} cached_speedup={:.2}x derived_speedup={:.2}x",
        exact_ns_per_score,
        binary_cached_ns_per_score,
        binary_derived_ns_per_score,
        exact_ns_per_score / binary_cached_ns_per_score.max(f64::EPSILON),
        exact_ns_per_score / binary_derived_ns_per_score.max(f64::EPSILON)
    );
}

fn run_grouped_mean_study(
    config: &Config,
    quantizer: &ProdQuantizer,
    queries: &[Vec<f32>],
    codes: &[Vec<u8>],
    capture_limits: &[usize],
    mode: GroupedScoreMode,
) {
    let grouped_codes = codes
        .iter()
        .map(|code| grouped_mean_code_from_packed(code, DIM, config.group_size))
        .collect::<Vec<_>>();
    let mut aggregate = StudyAggregate::new(capture_limits.len());

    for query in queries {
        let exact_prepared = quantizer.prepare_ip_query(query);
        let grouped_prepared = prepare_grouped_mean_query(
            &exact_prepared.rotated,
            &quantizer.codebook,
            config.group_size,
        );

        let mut exact_scores = Vec::with_capacity(codes.len());
        let mut approx_scores = Vec::with_capacity(codes.len());
        for (code, grouped_code) in codes.iter().zip(grouped_codes.iter()) {
            exact_scores.push(quantizer.score_ip_from_parts(&exact_prepared, 0.0, code));
            approx_scores.push(match mode {
                GroupedScoreMode::F32 => grouped_mean_score_f32(&grouped_prepared, grouped_code),
                GroupedScoreMode::U8 => grouped_mean_score_u8(&grouped_prepared, grouped_code),
            });
        }

        aggregate.record(&exact_scores, &approx_scores, config.top_k, capture_limits);
    }

    let exact_prepared = quantizer.prepare_ip_query(&queries[0]);
    let grouped_prepared = prepare_grouped_mean_query(
        &exact_prepared.rotated,
        &quantizer.codebook,
        config.group_size,
    );
    let exact_elapsed = time_scores(config.bench_iters, || {
        let mut sum = 0.0_f32;
        for code in codes {
            sum += quantizer.score_ip_from_parts(&exact_prepared, 0.0, code);
        }
        black_box(sum);
    });
    let grouped_f32_elapsed = time_scores(config.bench_iters, || {
        let mut sum = 0.0_f32;
        for grouped_code in &grouped_codes {
            sum += grouped_mean_score_f32(&grouped_prepared, grouped_code);
        }
        black_box(sum);
    });
    let grouped_u8_elapsed = time_scores(config.bench_iters, || {
        let mut sum = 0.0_f32;
        for grouped_code in &grouped_codes {
            sum += grouped_mean_score_u8(&grouped_prepared, grouped_code);
        }
        black_box(sum);
    });

    let score_count = (codes.len() * config.bench_iters) as f64;
    let exact_ns_per_score = exact_elapsed.as_secs_f64() * 1e9 / score_count;
    let grouped_f32_ns_per_score = grouped_f32_elapsed.as_secs_f64() * 1e9 / score_count;
    let grouped_u8_ns_per_score = grouped_u8_elapsed.as_secs_f64() * 1e9 / score_count;

    aggregate.print(config.query_count, config.top_k, capture_limits);
    println!("group_size={}", config.group_size);
    println!(
        "microbench exact_ns_per_score={:.1} grouped_f32_ns_per_score={:.1} grouped_u8_ns_per_score={:.1} grouped_f32_speedup={:.2}x grouped_u8_speedup={:.2}x",
        exact_ns_per_score,
        grouped_f32_ns_per_score,
        grouped_u8_ns_per_score,
        exact_ns_per_score / grouped_f32_ns_per_score.max(f64::EPSILON),
        exact_ns_per_score / grouped_u8_ns_per_score.max(f64::EPSILON)
    );
}

fn run_grouped_pq_study(
    config: &Config,
    quantizer: &ProdQuantizer,
    corpus: &[Vec<f32>],
    queries: &[Vec<f32>],
    capture_limits: &[usize],
    mode: GroupedScoreMode,
) {
    let transformed_corpus = corpus
        .iter()
        .map(|vector| rotate_vector(quantizer, vector))
        .collect::<Vec<_>>();
    let model = train_grouped_pq_model(
        &transformed_corpus,
        config.group_size,
        config.train_size,
        config.kmeans_iters,
        config.seed ^ 0xA5A5_5A5A_DEAD_BEEF,
    );
    let grouped_codes = transformed_corpus
        .iter()
        .map(|vector| encode_grouped_pq(vector, &model))
        .collect::<Vec<_>>();

    let mut aggregate = StudyAggregate::new(capture_limits.len());

    for query in queries {
        let rotated_query = rotate_vector(quantizer, query);
        let grouped_prepared = prepare_grouped_pq_query(&rotated_query, &model);

        let mut exact_scores = Vec::with_capacity(corpus.len());
        let mut approx_scores = Vec::with_capacity(corpus.len());
        for (candidate, grouped_code) in corpus.iter().zip(grouped_codes.iter()) {
            exact_scores.push(inner_product(query, candidate));
            approx_scores.push(match mode {
                GroupedScoreMode::F32 => grouped_pq_score_f32(&grouped_prepared, grouped_code),
                GroupedScoreMode::U8 => grouped_pq_score_u8(&grouped_prepared, grouped_code),
            });
        }

        aggregate.record(&exact_scores, &approx_scores, config.top_k, capture_limits);
    }

    let rotated_query = rotate_vector(quantizer, &queries[0]);
    let grouped_prepared = prepare_grouped_pq_query(&rotated_query, &model);
    let exact_elapsed = time_scores(config.bench_iters, || {
        let mut sum = 0.0_f32;
        for candidate in corpus {
            sum += inner_product(&queries[0], candidate);
        }
        black_box(sum);
    });
    let grouped_f32_elapsed = time_scores(config.bench_iters, || {
        let mut sum = 0.0_f32;
        for grouped_code in &grouped_codes {
            sum += grouped_pq_score_f32(&grouped_prepared, grouped_code);
        }
        black_box(sum);
    });
    let grouped_u8_elapsed = time_scores(config.bench_iters, || {
        let mut sum = 0.0_f32;
        for grouped_code in &grouped_codes {
            sum += grouped_pq_score_u8(&grouped_prepared, grouped_code);
        }
        black_box(sum);
    });

    let score_count = (corpus.len() * config.bench_iters) as f64;
    let exact_ns_per_score = exact_elapsed.as_secs_f64() * 1e9 / score_count;
    let grouped_f32_ns_per_score = grouped_f32_elapsed.as_secs_f64() * 1e9 / score_count;
    let grouped_u8_ns_per_score = grouped_u8_elapsed.as_secs_f64() * 1e9 / score_count;

    aggregate.print(config.query_count, config.top_k, capture_limits);
    println!("group_size={}", config.group_size);
    println!("group_count={}", model.group_count);
    println!("grouped_code_bytes={}", model.group_count.div_ceil(2));
    println!(
        "microbench exact_ns_per_score={:.1} grouped_pq_f32_ns_per_score={:.1} grouped_pq_u8_ns_per_score={:.1} grouped_pq_f32_speedup={:.2}x grouped_pq_u8_speedup={:.2}x",
        exact_ns_per_score,
        grouped_f32_ns_per_score,
        grouped_u8_ns_per_score,
        exact_ns_per_score / grouped_f32_ns_per_score.max(f64::EPSILON),
        exact_ns_per_score / grouped_u8_ns_per_score.max(f64::EPSILON)
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
            "--corpus-file" => {
                config.corpus_file = Some(parse_string_arg("--corpus-file", args.next()))
            }
            "--queries-file" => {
                config.queries_file = Some(parse_string_arg("--queries-file", args.next()))
            }
            "--study-mode" => {
                config.study_mode = StudyMode::parse(&parse_string_arg("--study-mode", args.next()))
            }
            "--group-size" => config.group_size = parse_usize_arg("--group-size", args.next()),
            "--train-size" => config.train_size = parse_usize_arg("--train-size", args.next()),
            "--kmeans-iters" => {
                config.kmeans_iters = parse_usize_arg("--kmeans-iters", args.next())
            }
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
    assert!(config.group_size > 0, "--group-size must be positive");
    assert!(config.train_size > 0, "--train-size must be positive");
    assert!(config.kmeans_iters > 0, "--kmeans-iters must be positive");
    assert_eq!(
        DIM % config.group_size,
        0,
        "--group-size must evenly divide {DIM}"
    );
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
    println!("  --study-mode <mode> default: int8-approx; one of: int8-approx, binary-sign, grouped-f32, grouped-u8, grouped-pq-f32, grouped-pq-u8");
    println!("  --group-size <n>    default: 16; required for grouped study modes");
    println!("  --train-size <n>    default: 4096; grouped-pq training sample cap");
    println!("  --kmeans-iters <n>  default: 15; grouped-pq k-means iterations");
    println!(
        "  --corpus-file <tsv> optional: real-corpus TSV with `id<TAB>comma,separated,floats`"
    );
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

fn rotate_vector(quantizer: &ProdQuantizer, vector: &[f32]) -> Vec<f32> {
    let padded = pad_input(vector, quantizer.transform_dim);
    let rotated = srht(&padded, &quantizer.signs);
    rotated[..quantizer.original_dim].to_vec()
}

fn inner_product(lhs: &[f32], rhs: &[f32]) -> f32 {
    lhs.iter()
        .zip(rhs.iter())
        .map(|(left, right)| left * right)
        .sum()
}

fn train_grouped_pq_model(
    transformed_corpus: &[Vec<f32>],
    group_size: usize,
    train_size: usize,
    kmeans_iters: usize,
    seed: u64,
) -> GroupedPqModel {
    assert!(
        !transformed_corpus.is_empty(),
        "grouped-pq training corpus must not be empty"
    );
    let dim = transformed_corpus[0].len();
    let group_count = dim / group_size;
    let sample_count = train_size.min(transformed_corpus.len());
    let sample_indices = sample_indices(transformed_corpus.len(), sample_count, seed);
    let mut codebooks = Vec::with_capacity(group_count);

    for group_index in 0..group_count {
        let mut samples = Vec::with_capacity(sample_count * group_size);
        for &sample_index in &sample_indices {
            let start = group_index * group_size;
            let end = start + group_size;
            samples.extend_from_slice(&transformed_corpus[sample_index][start..end]);
        }
        codebooks.push(train_group_codebook(
            &samples,
            group_size,
            kmeans_iters,
            seed ^ (group_index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
        ));
    }

    GroupedPqModel {
        codebooks,
        group_count,
        group_size,
    }
}

fn sample_indices(len: usize, sample_count: usize, seed: u64) -> Vec<usize> {
    if sample_count >= len {
        return (0..len).collect();
    }

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut indices = (0..len).collect::<Vec<_>>();
    for i in 0..sample_count {
        let swap_index = rng.gen_range(i..len);
        indices.swap(i, swap_index);
    }
    indices.truncate(sample_count);
    indices
}

fn train_group_codebook(
    samples: &[f32],
    group_size: usize,
    kmeans_iters: usize,
    seed: u64,
) -> Vec<f32> {
    const CENTROIDS: usize = 16;

    let sample_count = samples.len() / group_size;
    assert!(
        sample_count >= CENTROIDS,
        "need at least {CENTROIDS} samples"
    );

    let init_indices = sample_indices(sample_count, CENTROIDS, seed);
    let mut centroids = vec![0.0_f32; CENTROIDS * group_size];
    for (centroid_index, sample_index) in init_indices.into_iter().enumerate() {
        let sample = sample_slice(samples, sample_index, group_size);
        centroid_slice_mut(&mut centroids, centroid_index, group_size).copy_from_slice(sample);
    }

    let mut assignments = vec![0usize; sample_count];
    let mut sums = vec![0.0_f32; CENTROIDS * group_size];
    let mut counts = [0usize; CENTROIDS];

    for _ in 0..kmeans_iters {
        sums.fill(0.0);
        counts.fill(0);

        for (sample_index, assignment) in assignments.iter_mut().enumerate() {
            let sample = sample_slice(samples, sample_index, group_size);
            let centroid_index = shared_nearest_centroid_l2(sample, &centroids, group_size);
            *assignment = centroid_index;
            counts[centroid_index] += 1;
            let centroid_sum = centroid_slice_mut(&mut sums, centroid_index, group_size);
            for (dst, value) in centroid_sum.iter_mut().zip(sample.iter()) {
                *dst += *value;
            }
        }

        for (centroid_index, &count) in counts.iter().enumerate() {
            if count == 0 {
                let fallback_sample = sample_slice(
                    samples,
                    (seed as usize + centroid_index) % sample_count,
                    group_size,
                );
                centroid_slice_mut(&mut centroids, centroid_index, group_size)
                    .copy_from_slice(fallback_sample);
                continue;
            }

            let inv_count = (count as f32).recip();
            let centroid_sum = centroid_slice(&sums, centroid_index, group_size);
            let centroid = centroid_slice_mut(&mut centroids, centroid_index, group_size);
            for (dst, value) in centroid.iter_mut().zip(centroid_sum.iter()) {
                *dst = *value * inv_count;
            }
        }
    }

    centroids
}

fn encode_grouped_pq(vector: &[f32], model: &GroupedPqModel) -> GroupedPqCode {
    GroupedPqCode {
        packed_nibbles: shared_encode_grouped_pq(
            vector,
            model.codebooks.iter().map(Vec::as_slice),
            model.group_size,
        ),
    }
}

fn prepare_grouped_pq_query(
    rotated_query: &[f32],
    model: &GroupedPqModel,
) -> GroupedPqPreparedQuery {
    let flat_codebooks = model
        .codebooks
        .iter()
        .flat_map(|codebook| codebook.iter().copied())
        .collect::<Vec<_>>();
    let mut lut_f32 =
        shared_build_grouped_pq_lut_f32(rotated_query, &flat_codebooks, model.group_size);
    let mut lut_u8 = vec![0_u8; model.group_count * 16];
    let mut row_bias = vec![0.0_f32; model.group_count];
    let mut row_scale = vec![0.0_f32; model.group_count];

    for group_index in 0..model.group_count {
        let row = &mut lut_f32[group_index * 16..(group_index + 1) * 16];

        let row_min = row
            .iter()
            .copied()
            .fold(f32::INFINITY, |acc, value| acc.min(value));
        let row_max = row
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, |acc, value| acc.max(value));
        let scale = ((row_max - row_min) / 255.0).max(f32::EPSILON);
        row_bias[group_index] = row_min;
        row_scale[group_index] = scale;

        for (centroid_index, value) in row.iter().copied().enumerate() {
            lut_u8[group_index * 16 + centroid_index] =
                ((value - row_min) / scale).round().clamp(0.0, 255.0) as u8;
        }
    }

    GroupedPqPreparedQuery {
        lut_f32,
        lut_u8,
        row_bias,
        row_scale,
    }
}

fn grouped_pq_score_f32(prepared: &GroupedPqPreparedQuery, code: &GroupedPqCode) -> f32 {
    shared_grouped_pq_score_f32(
        &prepared.lut_f32,
        prepared.row_bias.len(),
        &code.packed_nibbles,
    )
}

fn grouped_pq_score_u8(prepared: &GroupedPqPreparedQuery, code: &GroupedPqCode) -> f32 {
    (0..prepared.row_bias.len())
        .map(|group_index| {
            let centroid_index = grouped_pq_nibble(&code.packed_nibbles, group_index);
            prepared.row_bias[group_index]
                + prepared.row_scale[group_index]
                    * prepared.lut_u8[group_index * 16 + centroid_index] as f32
        })
        .sum()
}

fn sample_slice(samples: &[f32], sample_index: usize, group_size: usize) -> &[f32] {
    let start = sample_index * group_size;
    let end = start + group_size;
    &samples[start..end]
}

fn centroid_slice(centroids: &[f32], centroid_index: usize, group_size: usize) -> &[f32] {
    let start = centroid_index * group_size;
    let end = start + group_size;
    &centroids[start..end]
}

fn centroid_slice_mut(
    centroids: &mut [f32],
    centroid_index: usize,
    group_size: usize,
) -> &mut [f32] {
    let start = centroid_index * group_size;
    let end = start + group_size;
    &mut centroids[start..end]
}

fn sign_lookup_from_codebook(codebook: &[f32]) -> [u8; 16] {
    assert_eq!(
        codebook.len(),
        16,
        "binary-sign study requires the no-QJL 4-bit lane"
    );

    let mut signs = [0_u8; 16];
    for (index, value) in codebook.iter().copied().enumerate() {
        signs[index] = u8::from(value >= 0.0);
    }
    signs
}

fn binary_sign_words_from_rotated(rotated: &[f32]) -> Vec<u64> {
    let mut words = vec![0_u64; rotated.len().div_ceil(64)];
    for (index, value) in rotated.iter().copied().enumerate() {
        if value >= 0.0 {
            words[index / 64] |= 1_u64 << (index % 64);
        }
    }
    words
}

fn binary_sign_words_from_packed(
    code_bytes: &[u8],
    dim: usize,
    sign_lookup: &[u8; 16],
) -> Vec<u64> {
    let mut words = vec![0_u64; dim.div_ceil(64)];
    let mut dim_index = 0usize;

    for &packed in code_bytes {
        if dim_index >= dim {
            break;
        }

        let low_nibble = (packed & 0x0F) as usize;
        if sign_lookup[low_nibble] != 0 {
            words[dim_index / 64] |= 1_u64 << (dim_index % 64);
        }
        dim_index += 1;

        if dim_index >= dim {
            break;
        }

        let high_nibble = (packed >> 4) as usize;
        if sign_lookup[high_nibble] != 0 {
            words[dim_index / 64] |= 1_u64 << (dim_index % 64);
        }
        dim_index += 1;
    }

    words
}

fn binary_sign_similarity(query_words: &[u64], candidate_words: &[u64], dim: usize) -> f32 {
    let hamming_distance = query_words
        .iter()
        .zip(candidate_words.iter())
        .map(|(query, candidate)| (query ^ candidate).count_ones())
        .sum::<u32>();
    let dim_i32 = i32::try_from(dim).expect("study dimensions should fit in i32");
    let distance_i32 = i32::try_from(hamming_distance).expect("hamming distance should fit in i32");
    (dim_i32 - (2 * distance_i32)) as f32
}

fn binary_sign_similarity_from_packed(
    query_words: &[u64],
    code_bytes: &[u8],
    dim: usize,
    sign_lookup: &[u8; 16],
) -> f32 {
    let candidate_words = binary_sign_words_from_packed(code_bytes, dim, sign_lookup);
    binary_sign_similarity(query_words, &candidate_words, dim)
}

fn grouped_mean_code_from_packed(
    code_bytes: &[u8],
    dim: usize,
    group_size: usize,
) -> GroupedMeanCode {
    let group_count = dim / group_size;
    let mut counts = vec![0_u8; group_count * 16];
    let mut dim_index = 0usize;

    for &packed in code_bytes {
        if dim_index >= dim {
            break;
        }

        let group_index = dim_index / group_size;
        counts[group_index * 16 + (packed & 0x0F) as usize] += 1;
        dim_index += 1;

        if dim_index >= dim {
            break;
        }

        let group_index = dim_index / group_size;
        counts[group_index * 16 + (packed >> 4) as usize] += 1;
        dim_index += 1;
    }

    GroupedMeanCode { counts }
}

fn prepare_grouped_mean_query(
    rotated: &[f32],
    codebook: &[f32],
    group_size: usize,
) -> GroupedMeanPreparedQuery {
    assert_eq!(
        codebook.len(),
        16,
        "grouped mean study requires 4-bit codebook"
    );

    let group_count = rotated.len() / group_size;
    let mut lut_f32 = vec![0.0_f32; group_count * 16];
    let mut lut_u8 = vec![0_u8; group_count * 16];
    let mut row_bias = vec![0.0_f32; group_count];
    let mut row_scale = vec![0.0_f32; group_count];

    for (group_index, group) in rotated.chunks_exact(group_size).enumerate() {
        let group_mean = group.iter().sum::<f32>() / group.len() as f32;
        let row = &mut lut_f32[group_index * 16..(group_index + 1) * 16];
        for (centroid_index, slot) in row.iter_mut().enumerate() {
            *slot = codebook[centroid_index] * group_mean;
        }

        let row_min = row
            .iter()
            .copied()
            .fold(f32::INFINITY, |acc, value| acc.min(value));
        let row_max = row
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, |acc, value| acc.max(value));
        let scale = ((row_max - row_min) / 255.0).max(f32::EPSILON);
        row_bias[group_index] = row_min;
        row_scale[group_index] = scale;

        for (centroid_index, value) in row.iter().copied().enumerate() {
            let quantized = ((value - row_min) / scale).round().clamp(0.0, 255.0) as u8;
            lut_u8[group_index * 16 + centroid_index] = quantized;
        }
    }

    GroupedMeanPreparedQuery {
        lut_f32,
        lut_u8,
        row_bias,
        row_scale,
    }
}

fn grouped_mean_score_f32(
    prepared: &GroupedMeanPreparedQuery,
    grouped_code: &GroupedMeanCode,
) -> f32 {
    grouped_code
        .counts
        .chunks_exact(16)
        .zip(prepared.lut_f32.chunks_exact(16))
        .map(|(counts, lut_row)| {
            counts
                .iter()
                .zip(lut_row.iter())
                .map(|(&count, &value)| count as f32 * value)
                .sum::<f32>()
        })
        .sum()
}

fn grouped_mean_score_u8(
    prepared: &GroupedMeanPreparedQuery,
    grouped_code: &GroupedMeanCode,
) -> f32 {
    grouped_code
        .counts
        .chunks_exact(16)
        .zip(prepared.lut_u8.chunks_exact(16))
        .enumerate()
        .map(|(group_index, (counts, lut_row))| {
            let bias = prepared.row_bias[group_index];
            let scale = prepared.row_scale[group_index];
            counts
                .iter()
                .zip(lut_row.iter())
                .map(|(&count, &value)| count as f32 * (bias + scale * value as f32))
                .sum::<f32>()
        })
        .sum()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_sign_words_from_packed_matches_expected_sign_bits() {
        let sign_lookup = [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
        let words = binary_sign_words_from_packed(&[0x10, 0x32], 4, &sign_lookup);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0] & 0b1111, 0b1010);
    }

    #[test]
    fn binary_sign_similarity_from_packed_matches_precomputed_words() {
        let sign_lookup = [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
        let query_words = vec![0b1010_u64];
        let candidate_words = binary_sign_words_from_packed(&[0x10, 0x32], 4, &sign_lookup);

        assert_eq!(
            binary_sign_similarity_from_packed(&query_words, &[0x10, 0x32], 4, &sign_lookup),
            binary_sign_similarity(&query_words, &candidate_words, 4)
        );
    }

    #[test]
    fn grouped_mean_code_counts_track_centroid_histogram_per_group() {
        let grouped = grouped_mean_code_from_packed(&[0x10, 0x32], 4, 2);
        assert_eq!(grouped.counts.len(), 32);
        assert_eq!(grouped.counts[0], 1);
        assert_eq!(grouped.counts[1], 1);
        assert_eq!(grouped.counts[16 + 2], 1);
        assert_eq!(grouped.counts[16 + 3], 1);
    }

    #[test]
    fn grouped_pq_encode_packs_two_nibbles_per_byte() {
        let model = GroupedPqModel {
            codebooks: vec![
                vec![
                    -1.0, -1.0, 1.0, 1.0, 10.0, 10.0, 11.0, 11.0, 12.0, 12.0, 13.0, 13.0, 14.0,
                    14.0, 15.0, 15.0, 16.0, 16.0, 17.0, 17.0, 18.0, 18.0, 19.0, 19.0, 20.0, 20.0,
                    21.0, 21.0, 22.0, 22.0, 23.0, 23.0,
                ],
                vec![
                    10.0, 10.0, 0.0, 0.0, -2.0, -2.0, 11.0, 11.0, 12.0, 12.0, 13.0, 13.0, 14.0,
                    14.0, 15.0, 15.0, 16.0, 16.0, 17.0, 17.0, 18.0, 18.0, 19.0, 19.0, 20.0, 20.0,
                    21.0, 21.0, 22.0, 22.0, 23.0, 23.0,
                ],
            ],
            group_count: 2,
            group_size: 2,
        };
        let code = encode_grouped_pq(&[1.0, 1.0, -2.0, -2.0], &model);
        assert_eq!(code.packed_nibbles, vec![0x21]);
        assert_eq!(grouped_pq_nibble(&code.packed_nibbles, 0), 1);
        assert_eq!(grouped_pq_nibble(&code.packed_nibbles, 1), 2);
    }

    #[test]
    fn grouped_pq_u8_score_tracks_f32_for_same_code() {
        let prepared = GroupedPqPreparedQuery {
            lut_f32: vec![1.0, 3.0],
            lut_u8: vec![0, 255],
            row_bias: vec![1.0],
            row_scale: vec![(3.0 - 1.0) / 255.0],
        };
        let code = GroupedPqCode {
            packed_nibbles: vec![0x01],
        };

        assert!((grouped_pq_score_f32(&prepared, &code) - 3.0).abs() < 1e-6);
        assert!((grouped_pq_score_u8(&prepared, &code) - 3.0).abs() < 1e-5);
    }
}
