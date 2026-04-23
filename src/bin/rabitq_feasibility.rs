//! RaBitQ feasibility binary — ADR-045 Stage 1 Phase 2.
//!
//! Measures recall@10 of the RaBitQ asymmetric estimator vs. exact
//! inner product at PQ4-parity storage, plus the empirical
//! distribution of the Cauchy-Schwarz error bound Stage 3 will use
//! to size candidate pools.
//!
//! Usage:
//!
//! ```text
//! cargo run --release --bin rabitq_feasibility -- \
//!     [--corpus N]   [--queries N]   [--dim D]   [--top-k K] [--seed S]
//!     [--corpus-file path.tsv] [--queries-file path.tsv]
//! ```
//!
//! Corpus/query files (if supplied) are one f32 vector per line,
//! whitespace-separated. When absent, the binary synthesizes
//! deterministic unit-sphere Gaussian vectors — fine for sanity
//! checks; the 50k / 1M real-corpus runs the gate actually cares
//! about require `--corpus-file` pointing at the canonical TSV.

use std::env;
use std::fs;
use std::sync::Arc;

use ecaz::bench_api::{ProdQuantizer, Quantizer, RaBitQQuantizer};

const DEFAULT_CORPUS: usize = 2_000;
const DEFAULT_QUERIES: usize = 200;
const DEFAULT_DIM: usize = 1_536;
const DEFAULT_TOP_K: usize = 10;
const DEFAULT_SEED: u64 = 42;

struct Config {
    corpus: usize,
    queries: usize,
    dim: usize,
    top_k: usize,
    seed: u64,
    corpus_file: Option<String>,
    queries_file: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            corpus: DEFAULT_CORPUS,
            queries: DEFAULT_QUERIES,
            dim: DEFAULT_DIM,
            top_k: DEFAULT_TOP_K,
            seed: DEFAULT_SEED,
            corpus_file: None,
            queries_file: None,
        }
    }
}

fn parse_args() -> Config {
    let mut cfg = Config::default();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--corpus" => cfg.corpus = args.next().unwrap().parse().unwrap(),
            "--queries" => cfg.queries = args.next().unwrap().parse().unwrap(),
            "--dim" => cfg.dim = args.next().unwrap().parse().unwrap(),
            "--top-k" => cfg.top_k = args.next().unwrap().parse().unwrap(),
            "--seed" => cfg.seed = args.next().unwrap().parse().unwrap(),
            "--corpus-file" => cfg.corpus_file = Some(args.next().unwrap()),
            "--queries-file" => cfg.queries_file = Some(args.next().unwrap()),
            other => panic!("unknown arg: {other}"),
        }
    }
    cfg
}

fn main() {
    let cfg = parse_args();
    println!("# RaBitQ feasibility study — ADR-045 Stage 1");
    println!(
        "# config: corpus={}, queries={}, dim={}, top_k={}, seed={}",
        cfg.corpus, cfg.queries, cfg.dim, cfg.top_k, cfg.seed,
    );

    let corpus = match &cfg.corpus_file {
        Some(path) => load_tsv_vectors(path, cfg.dim),
        None => synthesize_vectors(cfg.corpus, cfg.dim, cfg.seed),
    };
    let queries = match &cfg.queries_file {
        Some(path) => load_tsv_vectors(path, cfg.dim),
        None => synthesize_vectors(cfg.queries, cfg.dim, cfg.seed.wrapping_add(1)),
    };
    let corpus_size = corpus.len();
    let query_size = queries.len();
    println!(
        "# loaded: {} corpus vectors, {} queries (dim={})",
        corpus_size, query_size, cfg.dim,
    );

    let prod = ProdQuantizer::cached(cfg.dim, 4, cfg.seed);
    let rabitq = RaBitQQuantizer::with_srht(cfg.dim, prod);
    let rabitq_code_bytes = <RaBitQQuantizer as Quantizer>::code_len(&rabitq);
    let pq4_code_bytes = ecaz::bench_api::payload_len(cfg.dim, 4) - 4;
    println!(
        "# storage: RaBitQ code {} B, PQ4 code {} B (parity ratio {:.2}x)",
        rabitq_code_bytes,
        pq4_code_bytes,
        pq4_code_bytes as f32 / rabitq_code_bytes as f32,
    );

    println!("# encoding corpus");
    let codes: Vec<Box<[u8]>> = corpus
        .iter()
        .map(|v| <RaBitQQuantizer as Quantizer>::encode_code(&rabitq, v))
        .collect();

    println!("# scoring queries");
    let mut recall_sum = 0.0_f64;
    let mut bound_samples: Vec<f32> = Vec::with_capacity(query_size * cfg.top_k);
    let mut error_samples: Vec<f32> = Vec::with_capacity(query_size * cfg.top_k);
    for (qi, query) in queries.iter().enumerate() {
        let prepared = rabitq.prepare_estimator(query);
        let mut exact: Vec<(usize, f32)> = corpus
            .iter()
            .enumerate()
            .map(|(i, c)| (i, dot(query, c)))
            .collect();
        exact.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        exact.truncate(cfg.top_k);
        let truth: std::collections::HashSet<usize> = exact.iter().map(|(i, _)| *i).collect();

        let mut approx: Vec<(usize, f32, f32)> = codes
            .iter()
            .enumerate()
            .map(|(i, code)| {
                let est = rabitq.estimate_ip(&prepared, code);
                (i, est.estimate, est.bound)
            })
            .collect();
        approx.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        approx.truncate(cfg.top_k);

        let hit = approx.iter().filter(|(i, _, _)| truth.contains(i)).count();
        recall_sum += hit as f64 / cfg.top_k as f64;

        for (i, est_ip, bound) in &approx {
            bound_samples.push(*bound);
            let truth_ip = dot(query, &corpus[*i]);
            error_samples.push((truth_ip - est_ip).abs());
        }

        if qi < 3 {
            println!(
                "#   query {}: recall@{} = {}/{}",
                qi, cfg.top_k, hit, cfg.top_k,
            );
        }
    }

    let mean_recall = recall_sum / query_size as f64;
    let (mean_bound, p50_bound, p99_bound) = summarize(&bound_samples);
    let (mean_err, p50_err, p99_err) = summarize(&error_samples);

    println!();
    println!("recall@{} mean: {:.4}", cfg.top_k, mean_recall);
    println!(
        "bound  mean={:.3}  p50={:.3}  p99={:.3}",
        mean_bound, p50_bound, p99_bound,
    );
    println!(
        "error  mean={:.3}  p50={:.3}  p99={:.3}",
        mean_err, p50_err, p99_err,
    );
    println!(
        "tightness (error / bound) mean: {:.3}",
        mean_err / mean_bound.max(1e-9),
    );

    // ADR-045 Stage 1 gate.
    let gap_pp = (1.0 - mean_recall) * 100.0;
    println!();
    if gap_pp <= 1.0 {
        println!("GATE: PASS (recall gap {:.3} pp ≤ 1.0 pp)", gap_pp);
    } else if gap_pp <= 2.0 {
        println!(
            "GATE: MARGINAL (recall gap {:.3} pp, 1.0 < gap ≤ 2.0 pp)",
            gap_pp,
        );
    } else {
        println!("GATE: FAIL (recall gap {:.3} pp > 2.0 pp)", gap_pp);
    }
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn summarize(samples: &[f32]) -> (f32, f32, f32) {
    if samples.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    let mean = samples.iter().sum::<f32>() / samples.len() as f32;
    let p50 = sorted[sorted.len() / 2];
    let p99 = sorted[(sorted.len() as f32 * 0.99) as usize];
    (mean, p50, p99)
}

fn synthesize_vectors(count: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (0..count)
        .map(|_| {
            let v: Vec<f32> = (0..dim).map(|_| standard_normal(&mut rng)).collect();
            v
        })
        .collect()
}

fn standard_normal<R: rand::Rng>(rng: &mut R) -> f32 {
    // Box-Muller; one sample per call (we discard the second).
    use rand::distributions::{Distribution, Uniform};
    let u = Uniform::new_inclusive(f32::EPSILON, 1.0 - f32::EPSILON);
    let u1 = u.sample(rng);
    let u2 = u.sample(rng);
    (-2.0_f32 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos()
}

fn load_tsv_vectors(path: &str, dim: usize) -> Vec<Vec<f32>> {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    text.lines()
        .map(|line| {
            let v: Vec<f32> = line
                .split_whitespace()
                .map(|s| s.parse::<f32>().unwrap())
                .collect();
            assert_eq!(v.len(), dim, "{path}: expected dim {dim}, got {}", v.len());
            v
        })
        .collect()
}

// Silence the unused-import lint for workspaces that build this
// crate without the binary target feature gate. `Arc` is here so
// downstream consumers copying the pattern see how rotations plug
// in without needing to scan the quantizer module.
#[allow(dead_code)]
fn _arc_hint() -> Arc<()> {
    Arc::new(())
}
