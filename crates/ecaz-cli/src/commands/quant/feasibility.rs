//! `ecaz quant feasibility` — offline recall + error-bound study.
//!
//! Loads a corpus TSV and a queries TSV (canonical ecaz-cli shape
//! `<id>\t<json_array>`), encodes the corpus under the selected
//! quantizer, computes brute-force exact top-K per query, scores
//! every encoded vector with the quantizer's estimator, and reports
//! recall@K along with the Cauchy-Schwarz (or equivalent) error
//! bound distribution.
//!
//! The output line format is designed to be grep-able from a review
//! packet without post-processing — each stat is one
//! `key: value` line.

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Args, ValueEnum};
use color_eyre::eyre::{eyre, Result};

use ecaz::bench_api::{ProdQuantizer, Quantizer, RaBitQQuantizer};

use crate::tsv;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum QuantizerKind {
    /// RaBitQ binary quantizer (ADR-045 Stage 1).
    Rabitq,
}

#[derive(Args, Debug)]
pub struct FeasibilityArgs {
    /// Which quantizer to study. Today: rabitq only.
    #[arg(long, value_enum, default_value_t = QuantizerKind::Rabitq)]
    pub quantizer: QuantizerKind,

    /// Corpus TSV (`<id>\t<json_array>` per row). Canonical ecaz-cli shape.
    #[arg(long)]
    pub corpus_file: PathBuf,

    /// Queries TSV, same shape as `--corpus-file`.
    #[arg(long)]
    pub queries_file: PathBuf,

    /// Vector dimensionality. Validated against every TSV row.
    #[arg(long, default_value_t = 1536)]
    pub dim: usize,

    /// `K` in recall@K.
    #[arg(long, default_value_t = 10)]
    pub top_k: usize,

    /// Seed for any quantizer randomness (SRHT signs, codebook training).
    #[arg(long, default_value_t = 42)]
    pub seed: u64,

    /// Cap corpus rows loaded. `0` = no cap.
    #[arg(long, default_value_t = 0)]
    pub corpus_limit: usize,

    /// Cap query rows loaded. `0` = no cap.
    #[arg(long, default_value_t = 0)]
    pub query_limit: usize,

    /// Rerank candidate pool size `K'`. `0` disables reranking
    /// (pure estimator ranking, Symphony Stage-3 style). When
    /// `K' > top_k`, the estimator selects the top `K'` candidates
    /// and exact inner product reranks them to the final top-`K`.
    /// Non-Symphony RaBitQ pipelines (DiskANN in-memory tier, a
    /// general prefilter) typically run with `K' ≈ 100`.
    #[arg(long, default_value_t = 0)]
    pub rerank_k: usize,
}

pub async fn run(args: FeasibilityArgs) -> Result<()> {
    let corpus = load_limited(&args.corpus_file, args.dim, args.corpus_limit)?;
    let queries = load_limited(&args.queries_file, args.dim, args.query_limit)?;
    if corpus.is_empty() {
        return Err(eyre!("empty corpus"));
    }
    if queries.is_empty() {
        return Err(eyre!("empty queries"));
    }

    println!("# ecaz quant feasibility");
    println!(
        "# quantizer: {:?}  corpus_file: {}  queries_file: {}",
        args.quantizer,
        args.corpus_file.display(),
        args.queries_file.display(),
    );
    println!(
        "# loaded: {} corpus vectors, {} queries (dim={}, top_k={}, seed={}, rerank_k={})",
        corpus.len(),
        queries.len(),
        args.dim,
        args.top_k,
        args.seed,
        args.rerank_k,
    );
    if args.rerank_k > 0 && args.rerank_k < args.top_k {
        return Err(eyre!(
            "rerank_k ({}) must be 0 or >= top_k ({})",
            args.rerank_k,
            args.top_k,
        ));
    }

    match args.quantizer {
        QuantizerKind::Rabitq => run_rabitq(args, corpus, queries),
    }
}

fn run_rabitq(args: FeasibilityArgs, corpus: Vec<Vec<f32>>, queries: Vec<Vec<f32>>) -> Result<()> {
    let prod = ProdQuantizer::cached(args.dim, 4, args.seed);
    let rabitq = Arc::new(RaBitQQuantizer::with_srht(args.dim, prod));
    let rabitq_code_bytes = <RaBitQQuantizer as Quantizer>::code_len(rabitq.as_ref());
    let pq4_code_bytes = ecaz::bench_api::payload_len(args.dim, 4) - 4;
    println!(
        "# storage: RaBitQ code {} B, PQ4 code {} B (parity ratio {:.2}x)",
        rabitq_code_bytes,
        pq4_code_bytes,
        pq4_code_bytes as f32 / rabitq_code_bytes as f32,
    );

    let codes: Vec<Box<[u8]>> = corpus
        .iter()
        .map(|v| <RaBitQQuantizer as Quantizer>::encode_code(rabitq.as_ref(), v))
        .collect();

    let mut recall_no_rerank_sum = 0.0_f64;
    let mut recall_rerank_sum = 0.0_f64;
    let mut bound_samples: Vec<f32> = Vec::with_capacity(queries.len() * args.top_k);
    let mut error_samples: Vec<f32> = Vec::with_capacity(queries.len() * args.top_k);

    for (qi, query) in queries.iter().enumerate() {
        let prepared = rabitq.prepare_estimator(query);

        let mut exact: Vec<(usize, f32)> = corpus
            .iter()
            .enumerate()
            .map(|(i, c)| (i, dot(query, c)))
            .collect();
        exact.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        exact.truncate(args.top_k);
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

        // Pure single-stage recall (Symphony Stage-3 target).
        let top_k_no_rerank: Vec<(usize, f32, f32)> =
            approx.iter().take(args.top_k).cloned().collect();
        let hit_no_rerank = top_k_no_rerank
            .iter()
            .filter(|(i, _, _)| truth.contains(i))
            .count();
        recall_no_rerank_sum += hit_no_rerank as f64 / args.top_k as f64;

        for (i, est_ip, bound) in &top_k_no_rerank {
            bound_samples.push(*bound);
            let truth_ip = dot(query, &corpus[*i]);
            error_samples.push((truth_ip - est_ip).abs());
        }

        // Optional two-stage recall with K'-size candidate pool.
        let hit_rerank = if args.rerank_k > 0 {
            let kp = args.rerank_k.min(approx.len());
            let mut pool: Vec<(usize, f32)> = approx
                .iter()
                .take(kp)
                .map(|(i, _, _)| (*i, dot(query, &corpus[*i])))
                .collect();
            pool.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            pool.truncate(args.top_k);
            let hit = pool.iter().filter(|(i, _)| truth.contains(i)).count();
            recall_rerank_sum += hit as f64 / args.top_k as f64;
            Some(hit)
        } else {
            None
        };

        if qi < 3 {
            match hit_rerank {
                None => println!(
                    "#   query {}: recall@{} (no rerank) = {}/{}",
                    qi, args.top_k, hit_no_rerank, args.top_k,
                ),
                Some(r) => println!(
                    "#   query {}: recall@{} no-rerank = {}/{}, rerank(K'={}) = {}/{}",
                    qi, args.top_k, hit_no_rerank, args.top_k, args.rerank_k, r, args.top_k,
                ),
            }
        }
    }

    let recall_no_rerank = recall_no_rerank_sum / queries.len() as f64;
    let (mean_bound, p50_bound, p99_bound) = summarize(&bound_samples);
    let (mean_err, p50_err, p99_err) = summarize(&error_samples);

    println!();
    println!("recall@{} (no rerank) mean: {:.4}", args.top_k, recall_no_rerank);
    if args.rerank_k > 0 {
        let recall_rerank = recall_rerank_sum / queries.len() as f64;
        println!(
            "recall@{} (rerank K'={}) mean: {:.4}",
            args.top_k, args.rerank_k, recall_rerank,
        );
    }
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

    // Gate verdict uses the Symphony-aligned no-rerank number.
    let gap_pp = (1.0 - recall_no_rerank) * 100.0;
    println!();
    if gap_pp <= 1.0 {
        println!(
            "GATE (no-rerank / Symphony Stage-3): PASS (recall gap {:.3} pp ≤ 1.0 pp)",
            gap_pp,
        );
    } else if gap_pp <= 2.0 {
        println!(
            "GATE (no-rerank / Symphony Stage-3): MARGINAL (recall gap {:.3} pp)",
            gap_pp,
        );
    } else {
        println!(
            "GATE (no-rerank / Symphony Stage-3): FAIL (recall gap {:.3} pp > 2.0 pp)",
            gap_pp,
        );
    }

    Ok(())
}

fn load_limited(path: &std::path::Path, dim: usize, limit: usize) -> Result<Vec<Vec<f32>>> {
    let rows = tsv::iter_rows(path, dim)?;
    let mut out: Vec<Vec<f32>> = Vec::new();
    for row in rows {
        let vec_line = row?;
        out.push(vec_line.values);
        if limit > 0 && out.len() >= limit {
            break;
        }
    }
    Ok(out)
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
