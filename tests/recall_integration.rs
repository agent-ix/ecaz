//! Pure-Rust recall benchmark harness.
//!
//! Measures quantizer-level recall independent of HNSW graph quality.
//! Uses brute-force fp32 inner product as ground truth.
//!
//! Run with: cargo test --features bench --test recall_integration -- --ignored --nocapture

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use tqvector::bench_api::ProdQuantizer;

fn random_unit_vector(dim: usize, seed: u64) -> Vec<f32> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut values: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let norm = values.iter().map(|v| v * v).sum::<f32>().sqrt();
    for value in &mut values {
        *value /= norm.max(f32::EPSILON);
    }
    values
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Brute-force top-k by inner product. Returns (indices, scores) sorted descending.
fn brute_force_top_k(corpus: &[Vec<f32>], query: &[f32], k: usize) -> Vec<(usize, f32)> {
    let mut scores: Vec<(usize, f32)> = corpus
        .iter()
        .enumerate()
        .map(|(i, v)| (i, dot_product(query, v)))
        .collect();
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scores.truncate(k);
    scores
}

fn recall_at_k(true_top_k: &[(usize, f32)], pred_top_k: &[(usize, f32)], k: usize) -> f32 {
    let true_set: std::collections::HashSet<usize> =
        true_top_k.iter().take(k).map(|(i, _)| *i).collect();
    let pred_set: std::collections::HashSet<usize> =
        pred_top_k.iter().take(k).map(|(i, _)| *i).collect();
    true_set.intersection(&pred_set).count() as f32 / k as f32
}

fn ndcg_at_k(true_top_k: &[(usize, f32)], pred_top_k: &[(usize, f32)], k: usize) -> f32 {
    // Build relevance map from ground truth
    let relevance: std::collections::HashMap<usize, f32> =
        true_top_k.iter().take(k).map(|(i, s)| (*i, *s)).collect();

    // DCG of predicted ranking
    let dcg: f32 = pred_top_k
        .iter()
        .take(k)
        .enumerate()
        .map(|(rank, (idx, _))| {
            let rel = relevance.get(idx).copied().unwrap_or(0.0).max(0.0);
            rel / ((rank as f32 + 2.0).ln() / 2.0_f32.ln())
        })
        .sum();

    // Ideal DCG (ground truth ranking)
    let idcg: f32 = true_top_k
        .iter()
        .take(k)
        .enumerate()
        .map(|(rank, (_, score))| {
            let rel = score.max(0.0);
            rel / ((rank as f32 + 2.0).ln() / 2.0_f32.ln())
        })
        .sum();

    if idcg == 0.0 {
        0.0
    } else {
        dcg / idcg
    }
}

fn mean_absolute_score_error(true_scores: &[f32], pred_scores: &[f32]) -> f32 {
    true_scores
        .iter()
        .zip(pred_scores)
        .map(|(t, p)| (t - p).abs())
        .sum::<f32>()
        / true_scores.len() as f32
}

fn spearman_rank_correlation(true_top_k: &[(usize, f32)], pred_top_k: &[(usize, f32)]) -> f32 {
    let n = true_top_k.len().min(pred_top_k.len());
    if n < 2 {
        return 0.0;
    }

    // Map index -> rank in predicted
    let pred_rank: std::collections::HashMap<usize, usize> = pred_top_k
        .iter()
        .enumerate()
        .take(n)
        .map(|(rank, (idx, _))| (*idx, rank))
        .collect();

    let mut d_squared_sum = 0.0f64;
    for (true_rank, (idx, _)) in true_top_k.iter().enumerate().take(n) {
        let pred_r = pred_rank.get(idx).copied().unwrap_or(n); // not found = worst rank
        let d = true_rank as f64 - pred_r as f64;
        d_squared_sum += d * d;
    }

    let n = n as f64;
    1.0 - (6.0 * d_squared_sum / (n * (n * n - 1.0))) as f32
}

struct RecallReport {
    recall_at_10: f32,
    recall_at_100: f32,
    ndcg_at_10: f32,
    mean_abs_error: f32,
    spearman_rho: f32,
    top_k_overlap: f32,
}

fn run_recall_benchmark(
    n_corpus: usize,
    n_queries: usize,
    dim: usize,
    bits: u8,
    seed: u64,
) -> RecallReport {
    let corpus: Vec<Vec<f32>> = (0..n_corpus)
        .map(|i| random_unit_vector(dim, seed + i as u64))
        .collect();
    let queries: Vec<Vec<f32>> = (0..n_queries)
        .map(|i| random_unit_vector(dim, seed + 1_000_000 + i as u64))
        .collect();

    let quantizer = ProdQuantizer::new(dim, bits, seed);
    let payloads: Vec<Vec<u8>> = corpus
        .iter()
        .map(|v| quantizer.pack_payload(&quantizer.encode(v)))
        .collect();

    let mut total_recall_10 = 0.0f32;
    let mut total_recall_100 = 0.0f32;
    let mut total_ndcg_10 = 0.0f32;
    let mut total_mae = 0.0f32;
    let mut total_spearman = 0.0f32;
    let mut total_overlap = 0.0f32;

    let k_max = 100.min(n_corpus);

    for query in &queries {
        // Ground truth
        let true_top = brute_force_top_k(&corpus, query, k_max);

        // Quantized scoring
        let prepared = quantizer.prepare_ip_query(query);
        let mut scored: Vec<(usize, f32)> = payloads
            .iter()
            .enumerate()
            .map(|(i, payload)| (i, quantizer.score_ip_encoded(&prepared, payload)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.truncate(k_max);

        total_recall_10 += recall_at_k(&true_top, &scored, 10.min(k_max));
        total_recall_100 += recall_at_k(&true_top, &scored, k_max);
        total_ndcg_10 += ndcg_at_k(&true_top, &scored, 10.min(k_max));

        let true_scores: Vec<f32> = true_top.iter().take(10).map(|(_, s)| *s).collect();
        let pred_scores: Vec<f32> = scored.iter().take(10).map(|(_, s)| *s).collect();
        total_mae += mean_absolute_score_error(&true_scores, &pred_scores);
        total_spearman += spearman_rank_correlation(&true_top, &scored);

        // top-k set overlap
        let true_set: std::collections::HashSet<usize> =
            true_top.iter().take(10).map(|(i, _)| *i).collect();
        let pred_set: std::collections::HashSet<usize> =
            scored.iter().take(10).map(|(i, _)| *i).collect();
        total_overlap +=
            true_set.intersection(&pred_set).count() as f32 / 10.0f32.min(k_max as f32);
    }

    let n = n_queries as f32;
    RecallReport {
        recall_at_10: total_recall_10 / n,
        recall_at_100: total_recall_100 / n,
        ndcg_at_10: total_ndcg_10 / n,
        mean_abs_error: total_mae / n,
        spearman_rho: total_spearman / n,
        top_k_overlap: total_overlap / n,
    }
}

#[test]
#[ignore]
fn quantizer_recall_10k_1536_4bit() {
    let report = run_recall_benchmark(10_000, 50, 1536, 4, 42);
    println!("\n=== Quantizer Recall (10K x 1536, 4-bit) ===");
    println!("Recall@10:     {:.2}%", report.recall_at_10 * 100.0);
    println!("Recall@100:    {:.2}%", report.recall_at_100 * 100.0);
    println!("NDCG@10:       {:.4}", report.ndcg_at_10);
    println!("MAE (top-10):  {:.6}", report.mean_abs_error);
    println!("Spearman rho:  {:.4}", report.spearman_rho);
    println!("Top-10 overlap:{:.2}%", report.top_k_overlap * 100.0);
}

#[test]
#[ignore]
fn quantizer_recall_1k_1536_bitwidth_sweep() {
    println!("\n=== Bit-Width Sensitivity (1K x 1536) ===");
    println!(
        "{:>5} {:>10} {:>10} {:>8}",
        "bits", "Recall@10", "NDCG@10", "MAE"
    );
    for bits in [2u8, 3, 4, 5, 6, 7, 8] {
        let report = run_recall_benchmark(1_000, 20, 1536, bits, 42);
        println!(
            "{:>5} {:>9.2}% {:>10.4} {:>8.6}",
            bits,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_1k_dimension_sweep() {
    println!("\n=== Dimension Sensitivity (1K, 4-bit) ===");
    println!("{:>6} {:>10} {:>10}", "dim", "Recall@10", "NDCG@10");
    for dim in [128, 384, 768, 1536] {
        let report = run_recall_benchmark(1_000, 20, dim, 4, 42);
        println!(
            "{:>6} {:>9.2}% {:>10.4}",
            dim,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10
        );
    }
}

// Quick smoke test (not ignored) — validates the harness works
#[test]
fn recall_harness_smoke_test() {
    let report = run_recall_benchmark(100, 5, 32, 4, 42);
    // Just verify it runs without panic and returns reasonable values
    assert!(report.recall_at_10 >= 0.0 && report.recall_at_10 <= 1.0);
    assert!(report.ndcg_at_10 >= 0.0);
}
