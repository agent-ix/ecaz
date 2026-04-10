//! Pure-Rust recall benchmark harness.
//!
//! Measures quantizer-level recall independent of HNSW graph quality.
//! Uses brute-force fp32 inner product as ground truth.
//!
//! Run with: cargo test --features bench --test recall_integration -- --ignored --nocapture

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::f32::consts::PI;

use tqvector::bench_api::{
    decode_indices, decode_mse_only, lloyd_max, orthonormal_fwht_in_place, pad_input, qjl_project,
    quantize_to_indices, sign_vector, srht, transform_dim, EncodedTq, ProdQuantizer,
};

fn random_unit_vector(dim: usize, seed: u64) -> Vec<f32> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut values: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let norm = values.iter().map(|v| v * v).sum::<f32>().sqrt();
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
        .map(|i| random_unit_vector(dim, seed + 100_000 + i as u64))
        .collect();

    let mut rng = ChaCha8Rng::seed_from_u64(seed + 200_000);
    let mut corpus = Vec::with_capacity(n);

    for i in 0..n {
        let center = &centers[i % n_clusters];
        let mut vec: Vec<f32> = center
            .iter()
            .map(|&c| {
                let u1: f32 = rng.gen_range(0.0001f32..1.0);
                let u2: f32 = rng.gen_range(0.0f32..std::f32::consts::TAU);
                let noise = (-2.0 * u1.ln()).sqrt() * u2.cos() * spread;
                c + noise
            })
            .collect();
        let norm = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
        for v in &mut vec {
            *v /= norm.max(f32::EPSILON);
        }
        corpus.push(vec);
    }
    corpus
}

fn near_duplicate_pairs(
    dim: usize,
    n: usize,
    angle_radians: f32,
    seed: u64,
) -> (Vec<Vec<f32>>, Vec<Vec<f32>>) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed + 300_000);
    let mut bases = Vec::with_capacity(n);
    let mut perturbed = Vec::with_capacity(n);

    for i in 0..n {
        let base = random_unit_vector(dim, seed + i as u64);

        let mut noise: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0f32..1.0)).collect();
        let dot: f32 = noise.iter().zip(base.iter()).map(|(n, b)| n * b).sum();
        for (n, b) in noise.iter_mut().zip(base.iter()) {
            *n -= dot * b;
        }
        let noise_norm = noise.iter().map(|v| v * v).sum::<f32>().sqrt();
        if noise_norm < f32::EPSILON {
            perturbed.push(base.clone());
            bases.push(base);
            continue;
        }
        for v in &mut noise {
            *v /= noise_norm;
        }

        let cos_a = angle_radians.cos();
        let sin_a = angle_radians.sin();
        let pert: Vec<f32> = base
            .iter()
            .zip(noise.iter())
            .map(|(&b, &n)| cos_a * b + sin_a * n)
            .collect();
        let norm = pert.iter().map(|v| v * v).sum::<f32>().sqrt();
        let pert: Vec<f32> = pert.iter().map(|v| v / norm.max(f32::EPSILON)).collect();

        bases.push(base);
        perturbed.push(pert);
    }
    (bases, perturbed)
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
    let relevance: std::collections::HashMap<usize, f32> =
        true_top_k.iter().take(k).map(|(i, s)| (*i, *s)).collect();

    let dcg: f32 = pred_top_k
        .iter()
        .take(k)
        .enumerate()
        .map(|(rank, (idx, _))| {
            let rel = relevance.get(idx).copied().unwrap_or(0.0).max(0.0);
            rel / ((rank as f32 + 2.0).ln() / 2.0_f32.ln())
        })
        .sum();

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

    let pred_rank: std::collections::HashMap<usize, usize> = pred_top_k
        .iter()
        .enumerate()
        .take(n)
        .map(|(rank, (idx, _))| (*idx, rank))
        .collect();

    let mut d_squared_sum = 0.0f64;
    for (true_rank, (idx, _)) in true_top_k.iter().enumerate().take(n) {
        let pred_r = pred_rank.get(idx).copied().unwrap_or(n);
        let d = true_rank as f64 - pred_r as f64;
        d_squared_sum += d * d;
    }

    let n = n as f64;
    1.0 - (6.0 * d_squared_sum / (n * (n * n - 1.0))) as f32
}

struct RecallReport {
    recall_at_1: f32,
    recall_at_10: f32,
    recall_at_100: f32,
    ndcg_at_10: f32,
    mean_abs_error: f32,
    spearman_rho: f32,
    top_k_overlap: f32,
}

#[derive(Clone, Copy)]
enum ScoringVariant {
    Exact,
    GammaZero,
    CodeProxy,
    DecodedApprox,
}

fn encoded_code_bytes(encoded: &EncodedTq) -> Vec<u8> {
    let mut code_bytes = encoded.mse_packed.clone();
    code_bytes.extend_from_slice(&encoded.qjl_packed);
    code_bytes
}

fn run_recall_benchmark_with_corpus(
    corpus: &[Vec<f32>],
    queries: &[Vec<f32>],
    dim: usize,
    bits: u8,
    seed: u64,
) -> RecallReport {
    let quantizer = ProdQuantizer::new(dim, bits, seed);
    let payloads: Vec<Vec<u8>> = corpus
        .iter()
        .map(|v| quantizer.pack_payload(&quantizer.encode(v)))
        .collect();

    let n_corpus = corpus.len();
    let n_queries = queries.len();
    let mut total_recall_1 = 0.0f32;
    let mut total_recall_10 = 0.0f32;
    let mut total_recall_100 = 0.0f32;
    let mut total_ndcg_10 = 0.0f32;
    let mut total_mae = 0.0f32;
    let mut total_spearman = 0.0f32;
    let mut total_overlap = 0.0f32;

    let k_max = 100.min(n_corpus);

    for query in queries {
        let true_top = brute_force_top_k(corpus, query, k_max);

        let prepared = quantizer.prepare_ip_query(query);
        let mut scored: Vec<(usize, f32)> = payloads
            .iter()
            .enumerate()
            .map(|(i, payload)| (i, quantizer.score_ip_encoded(&prepared, payload)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.truncate(k_max);

        total_recall_1 += recall_at_k(&true_top, &scored, 1.min(k_max));
        total_recall_10 += recall_at_k(&true_top, &scored, 10.min(k_max));
        total_recall_100 += recall_at_k(&true_top, &scored, k_max);
        total_ndcg_10 += ndcg_at_k(&true_top, &scored, 10.min(k_max));

        let true_scores: Vec<f32> = true_top.iter().take(10).map(|(_, s)| *s).collect();
        let pred_scores: Vec<f32> = scored.iter().take(10).map(|(_, s)| *s).collect();
        total_mae += mean_absolute_score_error(&true_scores, &pred_scores);
        total_spearman += spearman_rank_correlation(&true_top, &scored);

        let true_set: std::collections::HashSet<usize> =
            true_top.iter().take(10).map(|(i, _)| *i).collect();
        let pred_set: std::collections::HashSet<usize> =
            scored.iter().take(10).map(|(i, _)| *i).collect();
        total_overlap +=
            true_set.intersection(&pred_set).count() as f32 / 10.0f32.min(k_max as f32);
    }

    let n = n_queries as f32;
    RecallReport {
        recall_at_1: total_recall_1 / n,
        recall_at_10: total_recall_10 / n,
        recall_at_100: total_recall_100 / n,
        ndcg_at_10: total_ndcg_10 / n,
        mean_abs_error: total_mae / n,
        spearman_rho: total_spearman / n,
        top_k_overlap: total_overlap / n,
    }
}

fn run_recall_benchmark_with_quantizer(
    corpus: &[Vec<f32>],
    queries: &[Vec<f32>],
    quantizer: &ProdQuantizer,
) -> RecallReport {
    let payloads: Vec<Vec<u8>> = corpus
        .iter()
        .map(|v| quantizer.pack_payload(&quantizer.encode(v)))
        .collect();

    let n_corpus = corpus.len();
    let n_queries = queries.len();
    let mut total_recall_1 = 0.0f32;
    let mut total_recall_10 = 0.0f32;
    let mut total_recall_100 = 0.0f32;
    let mut total_ndcg_10 = 0.0f32;
    let mut total_mae = 0.0f32;
    let mut total_spearman = 0.0f32;
    let mut total_overlap = 0.0f32;

    let k_max = 100.min(n_corpus);

    for query in queries {
        let true_top = brute_force_top_k(corpus, query, k_max);

        let prepared = quantizer.prepare_ip_query(query);
        let mut scored: Vec<(usize, f32)> = payloads
            .iter()
            .enumerate()
            .map(|(i, payload)| (i, quantizer.score_ip_encoded(&prepared, payload)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.truncate(k_max);

        total_recall_1 += recall_at_k(&true_top, &scored, 1.min(k_max));
        total_recall_10 += recall_at_k(&true_top, &scored, 10.min(k_max));
        total_recall_100 += recall_at_k(&true_top, &scored, k_max);
        total_ndcg_10 += ndcg_at_k(&true_top, &scored, 10.min(k_max));

        let true_scores: Vec<f32> = true_top.iter().take(10).map(|(_, s)| *s).collect();
        let pred_scores: Vec<f32> = scored.iter().take(10).map(|(_, s)| *s).collect();
        total_mae += mean_absolute_score_error(&true_scores, &pred_scores);
        total_spearman += spearman_rank_correlation(&true_top, &scored);

        let true_set: std::collections::HashSet<usize> =
            true_top.iter().take(10).map(|(i, _)| *i).collect();
        let pred_set: std::collections::HashSet<usize> =
            scored.iter().take(10).map(|(i, _)| *i).collect();
        total_overlap +=
            true_set.intersection(&pred_set).count() as f32 / 10.0f32.min(k_max as f32);
    }

    let n = n_queries as f32;
    RecallReport {
        recall_at_1: total_recall_1 / n,
        recall_at_10: total_recall_10 / n,
        recall_at_100: total_recall_100 / n,
        ndcg_at_10: total_ndcg_10 / n,
        mean_abs_error: total_mae / n,
        spearman_rho: total_spearman / n,
        top_k_overlap: total_overlap / n,
    }
}

fn prod_quantizer_with_codebook_dim(
    dim: usize,
    bits: u8,
    seed: u64,
    codebook_dim: usize,
) -> ProdQuantizer {
    let mut quantizer = ProdQuantizer::new(dim, bits, seed);
    quantizer.codebook = lloyd_max((bits - 1) as usize, codebook_dim, 20_000)
        .into_iter()
        .map(|value| value as f32)
        .collect();
    quantizer
}

fn current_prod_quantizer_with_codebook_dim(
    dim: usize,
    bits: u8,
    seed: u64,
    codebook_dim: usize,
) -> ProdQuantizer {
    let mut quantizer = ProdQuantizer::new(dim, bits, seed);
    let mse_bits = if dim == 1536 && bits == 4 {
        bits
    } else {
        bits.saturating_sub(1)
    };
    quantizer.codebook = lloyd_max(mse_bits as usize, codebook_dim, 20_000)
        .into_iter()
        .map(|value| value as f32)
        .collect();
    quantizer
}

fn run_recall_benchmark_with_corpus_variant(
    corpus: &[Vec<f32>],
    queries: &[Vec<f32>],
    dim: usize,
    bits: u8,
    seed: u64,
    variant: ScoringVariant,
) -> RecallReport {
    let quantizer = ProdQuantizer::new(dim, bits, seed);
    let encoded_corpus: Vec<EncodedTq> = corpus.iter().map(|v| quantizer.encode(v)).collect();
    let payloads: Vec<Vec<u8>> = encoded_corpus
        .iter()
        .map(|encoded| quantizer.pack_payload(encoded))
        .collect();
    let code_bytes: Vec<Vec<u8>> = encoded_corpus.iter().map(encoded_code_bytes).collect();
    let decoded_approx: Vec<Vec<f32>> = payloads
        .iter()
        .map(|payload| quantizer.decode_approximate(payload))
        .collect();

    let n_corpus = corpus.len();
    let n_queries = queries.len();
    let mut total_recall_1 = 0.0f32;
    let mut total_recall_10 = 0.0f32;
    let mut total_recall_100 = 0.0f32;
    let mut total_ndcg_10 = 0.0f32;
    let mut total_mae = 0.0f32;
    let mut total_spearman = 0.0f32;
    let mut total_overlap = 0.0f32;

    let k_max = 100.min(n_corpus);

    for query in queries {
        let true_top = brute_force_top_k(corpus, query, k_max);
        let prepared = quantizer.prepare_ip_query(query);
        let query_payload = match variant {
            ScoringVariant::CodeProxy => Some(quantizer.pack_payload(&quantizer.encode(query))),
            _ => None,
        };

        let mut scored: Vec<(usize, f32)> = match variant {
            ScoringVariant::Exact => payloads
                .iter()
                .enumerate()
                .map(|(i, payload)| (i, quantizer.score_ip_encoded(&prepared, payload)))
                .collect(),
            ScoringVariant::GammaZero => code_bytes
                .iter()
                .enumerate()
                .map(|(i, code)| (i, quantizer.score_ip_from_parts(&prepared, 0.0, code)))
                .collect(),
            ScoringVariant::CodeProxy => payloads
                .iter()
                .enumerate()
                .map(|(i, payload)| {
                    (
                        i,
                        quantizer.score_ip_encoded_lite(
                            query_payload
                                .as_ref()
                                .expect("code-proxy query payload should exist"),
                            payload,
                        ),
                    )
                })
                .collect(),
            ScoringVariant::DecodedApprox => decoded_approx
                .iter()
                .enumerate()
                .map(|(i, approx)| (i, dot_product(query, approx)))
                .collect(),
        };
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.truncate(k_max);

        total_recall_1 += recall_at_k(&true_top, &scored, 1.min(k_max));
        total_recall_10 += recall_at_k(&true_top, &scored, 10.min(k_max));
        total_recall_100 += recall_at_k(&true_top, &scored, k_max);
        total_ndcg_10 += ndcg_at_k(&true_top, &scored, 10.min(k_max));

        let true_scores: Vec<f32> = true_top.iter().take(10).map(|(_, s)| *s).collect();
        let pred_scores: Vec<f32> = scored.iter().take(10).map(|(_, s)| *s).collect();
        total_mae += mean_absolute_score_error(&true_scores, &pred_scores);
        total_spearman += spearman_rank_correlation(&true_top, &scored);

        let true_set: std::collections::HashSet<usize> =
            true_top.iter().take(10).map(|(i, _)| *i).collect();
        let pred_set: std::collections::HashSet<usize> =
            scored.iter().take(10).map(|(i, _)| *i).collect();
        total_overlap +=
            true_set.intersection(&pred_set).count() as f32 / 10.0f32.min(k_max as f32);
    }

    let n = n_queries as f32;
    RecallReport {
        recall_at_1: total_recall_1 / n,
        recall_at_10: total_recall_10 / n,
        recall_at_100: total_recall_100 / n,
        ndcg_at_10: total_ndcg_10 / n,
        mean_abs_error: total_mae / n,
        spearman_rho: total_spearman / n,
        top_k_overlap: total_overlap / n,
    }
}

fn run_tail_mse_reference(
    corpus: &[Vec<f32>],
    queries: &[Vec<f32>],
    dim: usize,
    bits: u8,
    seed: u64,
    codebook_dim: usize,
) -> RecallReport {
    let transform = transform_dim(dim);
    let codebook: Vec<f32> = lloyd_max((bits - 1) as usize, codebook_dim, 20_000)
        .into_iter()
        .map(|value| value as f32)
        .collect();
    let signs = sign_vector(transform, seed);

    let corpus_codes: Vec<Vec<u8>> = corpus
        .iter()
        .map(|vector| {
            let padded = pad_input(vector, transform);
            let rotated = srht(&padded, &signs);
            let indices = quantize_to_indices(&codebook, &rotated, transform);
            tqvector::bench_api::pack_mse_indices(&indices, bits - 1)
        })
        .collect();

    let n_corpus = corpus.len();
    let n_queries = queries.len();
    let mut total_recall_1 = 0.0f32;
    let mut total_recall_10 = 0.0f32;
    let mut total_recall_100 = 0.0f32;
    let mut total_ndcg_10 = 0.0f32;
    let mut total_mae = 0.0f32;
    let mut total_spearman = 0.0f32;
    let mut total_overlap = 0.0f32;

    let k_max = 100.min(n_corpus);
    for query in queries {
        let true_top = brute_force_top_k(corpus, query, k_max);

        let rotated_query = srht(&pad_input(query, transform), &signs);
        let mut scored: Vec<(usize, f32)> = corpus_codes
            .iter()
            .enumerate()
            .map(|(i, packed)| {
                let indices = tqvector::bench_api::unpack_mse_indices(packed, transform, bits - 1);
                let mse_values = decode_indices(&codebook, &indices);
                let score = mse_values
                    .iter()
                    .zip(rotated_query.iter())
                    .map(|(approx, rotated)| approx * rotated)
                    .sum();
                (i, score)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.truncate(k_max);

        total_recall_1 += recall_at_k(&true_top, &scored, 1.min(k_max));
        total_recall_10 += recall_at_k(&true_top, &scored, 10.min(k_max));
        total_recall_100 += recall_at_k(&true_top, &scored, k_max);
        total_ndcg_10 += ndcg_at_k(&true_top, &scored, 10.min(k_max));

        let true_scores: Vec<f32> = true_top.iter().take(10).map(|(_, s)| *s).collect();
        let pred_scores: Vec<f32> = scored.iter().take(10).map(|(_, s)| *s).collect();
        total_mae += mean_absolute_score_error(&true_scores, &pred_scores);
        total_spearman += spearman_rank_correlation(&true_top, &scored);

        let true_set: std::collections::HashSet<usize> =
            true_top.iter().take(10).map(|(i, _)| *i).collect();
        let pred_set: std::collections::HashSet<usize> =
            scored.iter().take(10).map(|(i, _)| *i).collect();
        total_overlap +=
            true_set.intersection(&pred_set).count() as f32 / 10.0f32.min(k_max as f32);
    }

    let n = n_queries as f32;
    RecallReport {
        recall_at_1: total_recall_1 / n,
        recall_at_10: total_recall_10 / n,
        recall_at_100: total_recall_100 / n,
        ndcg_at_10: total_ndcg_10 / n,
        mean_abs_error: total_mae / n,
        spearman_rho: total_spearman / n,
        top_k_overlap: total_overlap / n,
    }
}

fn run_tail_full_reference(
    corpus: &[Vec<f32>],
    queries: &[Vec<f32>],
    dim: usize,
    bits: u8,
    seed: u64,
    codebook_dim: usize,
) -> RecallReport {
    const QJL_SIGN_SEED_XOR: u64 = 0x9E37_79B9_7F4A_7C15;

    let transform = transform_dim(dim);
    let codebook: Vec<f32> = lloyd_max((bits - 1) as usize, codebook_dim, 20_000)
        .into_iter()
        .map(|value| value as f32)
        .collect();
    let signs = sign_vector(transform, seed);
    let qjl_signs = sign_vector(transform, seed ^ QJL_SIGN_SEED_XOR);

    let encoded_corpus: Vec<(Vec<u8>, f32, Vec<bool>)> = corpus
        .iter()
        .map(|vector| {
            let padded = pad_input(vector, transform);
            let rotated = srht(&padded, &signs);
            let indices = quantize_to_indices(&codebook, &rotated, transform);
            let mse_values = decode_indices(&codebook, &indices);
            let decoded_mse = decode_mse_only(&mse_values, &signs, dim);
            let residual: Vec<f32> = vector
                .iter()
                .zip(decoded_mse.iter())
                .map(|(input, approx)| input - approx)
                .collect();
            let gamma = residual
                .iter()
                .map(|value| value * value)
                .sum::<f32>()
                .sqrt();
            let qjl_bits = qjl_project(&residual, &qjl_signs)
                .into_iter()
                .map(|value| value >= 0.0)
                .collect();
            (
                tqvector::bench_api::pack_mse_indices(&indices, bits - 1),
                gamma,
                qjl_bits,
            )
        })
        .collect();

    let n_corpus = corpus.len();
    let n_queries = queries.len();
    let mut total_recall_1 = 0.0f32;
    let mut total_recall_10 = 0.0f32;
    let mut total_recall_100 = 0.0f32;
    let mut total_ndcg_10 = 0.0f32;
    let mut total_mae = 0.0f32;
    let mut total_spearman = 0.0f32;
    let mut total_overlap = 0.0f32;

    let k_max = 100.min(n_corpus);
    let qjl_scale = (PI / 2.0).sqrt() / transform as f32;

    for query in queries {
        let true_top = brute_force_top_k(corpus, query, k_max);
        let rotated_query = srht(&pad_input(query, transform), &signs);
        let query_qjl = qjl_project(query, &qjl_signs);

        let mut scored: Vec<(usize, f32)> = encoded_corpus
            .iter()
            .enumerate()
            .map(|(i, (packed, gamma, qjl_bits))| {
                let indices = tqvector::bench_api::unpack_mse_indices(packed, transform, bits - 1);
                let mse_values = decode_indices(&codebook, &indices);
                let mse_sum: f32 = mse_values
                    .iter()
                    .zip(rotated_query.iter())
                    .map(|(approx, rotated)| approx * rotated)
                    .sum();
                let qjl_sum: f32 = query_qjl
                    .iter()
                    .zip(qjl_bits.iter())
                    .map(|(query_value, bit)| query_value * if *bit { 1.0 } else { -1.0 })
                    .sum();
                (i, mse_sum + (*gamma * qjl_scale * qjl_sum))
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.truncate(k_max);

        total_recall_1 += recall_at_k(&true_top, &scored, 1.min(k_max));
        total_recall_10 += recall_at_k(&true_top, &scored, 10.min(k_max));
        total_recall_100 += recall_at_k(&true_top, &scored, k_max);
        total_ndcg_10 += ndcg_at_k(&true_top, &scored, 10.min(k_max));

        let true_scores: Vec<f32> = true_top.iter().take(10).map(|(_, s)| *s).collect();
        let pred_scores: Vec<f32> = scored.iter().take(10).map(|(_, s)| *s).collect();
        total_mae += mean_absolute_score_error(&true_scores, &pred_scores);
        total_spearman += spearman_rank_correlation(&true_top, &scored);

        let true_set: std::collections::HashSet<usize> =
            true_top.iter().take(10).map(|(i, _)| *i).collect();
        let pred_set: std::collections::HashSet<usize> =
            scored.iter().take(10).map(|(i, _)| *i).collect();
        total_overlap +=
            true_set.intersection(&pred_set).count() as f32 / 10.0f32.min(k_max as f32);
    }

    let n = n_queries as f32;
    RecallReport {
        recall_at_1: total_recall_1 / n,
        recall_at_10: total_recall_10 / n,
        recall_at_100: total_recall_100 / n,
        ndcg_at_10: total_ndcg_10 / n,
        mean_abs_error: total_mae / n,
        spearman_rho: total_spearman / n,
        top_k_overlap: total_overlap / n,
    }
}

fn tiled_srht(input: &[f32], signs: &[f32], tile_dim: usize) -> Vec<f32> {
    assert_eq!(input.len(), signs.len(), "tiled srht input/sign length mismatch");
    assert_eq!(
        input.len() % tile_dim,
        0,
        "tile_dim must divide input length"
    );
    let mut workspace = input.to_vec();
    for (value, sign) in workspace.iter_mut().zip(signs.iter()) {
        *value *= *sign;
    }
    for chunk in workspace.chunks_mut(tile_dim) {
        orthonormal_fwht_in_place(chunk);
    }
    workspace
}

fn tiled_inverse_srht(input: &[f32], signs: &[f32], tile_dim: usize) -> Vec<f32> {
    assert_eq!(
        input.len(),
        signs.len(),
        "tiled inverse input/sign length mismatch"
    );
    assert_eq!(
        input.len() % tile_dim,
        0,
        "tile_dim must divide input length"
    );
    let mut workspace = input.to_vec();
    for chunk in workspace.chunks_mut(tile_dim) {
        orthonormal_fwht_in_place(chunk);
    }
    for (value, sign) in workspace.iter_mut().zip(signs.iter()) {
        *value *= *sign;
    }
    workspace
}

fn run_tiled_full_reference(
    corpus: &[Vec<f32>],
    queries: &[Vec<f32>],
    dim: usize,
    bits: u8,
    seed: u64,
    tile_dim: usize,
    codebook_dim: usize,
) -> RecallReport {
    const QJL_SIGN_SEED_XOR: u64 = 0x9E37_79B9_7F4A_7C15;

    assert_eq!(dim % tile_dim, 0, "tile_dim must divide dim");
    let codebook: Vec<f32> = lloyd_max((bits - 1) as usize, codebook_dim, 20_000)
        .into_iter()
        .map(|value| value as f32)
        .collect();
    let signs = sign_vector(dim, seed);
    let qjl_signs = sign_vector(dim, seed ^ QJL_SIGN_SEED_XOR);

    let encoded_corpus: Vec<(Vec<u8>, f32, Vec<bool>)> = corpus
        .iter()
        .map(|vector| {
            let rotated = tiled_srht(vector, &signs, tile_dim);
            let indices = quantize_to_indices(&codebook, &rotated, dim);
            let mse_values = decode_indices(&codebook, &indices);
            let decoded_mse = tiled_inverse_srht(&mse_values, &signs, tile_dim);
            let residual: Vec<f32> = vector
                .iter()
                .zip(decoded_mse.iter())
                .map(|(input, approx)| input - approx)
                .collect();
            let gamma = residual
                .iter()
                .map(|value| value * value)
                .sum::<f32>()
                .sqrt();
            let qjl_bits = tiled_srht(&residual, &qjl_signs, tile_dim)
                .into_iter()
                .map(|value| value >= 0.0)
                .collect();
            (
                tqvector::bench_api::pack_mse_indices(&indices, bits - 1),
                gamma,
                qjl_bits,
            )
        })
        .collect();

    let n_corpus = corpus.len();
    let n_queries = queries.len();
    let mut total_recall_1 = 0.0f32;
    let mut total_recall_10 = 0.0f32;
    let mut total_recall_100 = 0.0f32;
    let mut total_ndcg_10 = 0.0f32;
    let mut total_mae = 0.0f32;
    let mut total_spearman = 0.0f32;
    let mut total_overlap = 0.0f32;

    let k_max = 100.min(n_corpus);
    let qjl_scale = (PI / 2.0).sqrt() / dim as f32;

    for query in queries {
        let true_top = brute_force_top_k(corpus, query, k_max);
        let rotated_query = tiled_srht(query, &signs, tile_dim);
        let query_qjl = tiled_srht(query, &qjl_signs, tile_dim);

        let mut scored: Vec<(usize, f32)> = encoded_corpus
            .iter()
            .enumerate()
            .map(|(i, (packed, gamma, qjl_bits))| {
                let indices = tqvector::bench_api::unpack_mse_indices(packed, dim, bits - 1);
                let mse_values = decode_indices(&codebook, &indices);
                let mse_sum: f32 = mse_values
                    .iter()
                    .zip(rotated_query.iter())
                    .map(|(approx, rotated)| approx * rotated)
                    .sum();
                let qjl_sum: f32 = query_qjl
                    .iter()
                    .zip(qjl_bits.iter())
                    .map(|(query_value, bit)| query_value * if *bit { 1.0 } else { -1.0 })
                    .sum();
                (i, mse_sum + (*gamma * qjl_scale * qjl_sum))
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.truncate(k_max);

        total_recall_1 += recall_at_k(&true_top, &scored, 1.min(k_max));
        total_recall_10 += recall_at_k(&true_top, &scored, 10.min(k_max));
        total_recall_100 += recall_at_k(&true_top, &scored, k_max);
        total_ndcg_10 += ndcg_at_k(&true_top, &scored, 10.min(k_max));

        let true_scores: Vec<f32> = true_top.iter().take(10).map(|(_, s)| *s).collect();
        let pred_scores: Vec<f32> = scored.iter().take(10).map(|(_, s)| *s).collect();
        total_mae += mean_absolute_score_error(&true_scores, &pred_scores);
        total_spearman += spearman_rank_correlation(&true_top, &scored);

        let true_set: std::collections::HashSet<usize> =
            true_top.iter().take(10).map(|(i, _)| *i).collect();
        let pred_set: std::collections::HashSet<usize> =
            scored.iter().take(10).map(|(i, _)| *i).collect();
        total_overlap +=
            true_set.intersection(&pred_set).count() as f32 / 10.0f32.min(k_max as f32);
    }

    let n = n_queries as f32;
    RecallReport {
        recall_at_1: total_recall_1 / n,
        recall_at_10: total_recall_10 / n,
        recall_at_100: total_recall_100 / n,
        ndcg_at_10: total_ndcg_10 / n,
        mean_abs_error: total_mae / n,
        spearman_rho: total_spearman / n,
        top_k_overlap: total_overlap / n,
    }
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
    run_recall_benchmark_with_corpus(&corpus, &queries, dim, bits, seed)
}

fn print_report(label: &str, report: &RecallReport) {
    println!("\n=== {label} ===");
    println!("Recall@1:      {:.2}%", report.recall_at_1 * 100.0);
    println!("Recall@10:     {:.2}%", report.recall_at_10 * 100.0);
    println!("Recall@100:    {:.2}%", report.recall_at_100 * 100.0);
    println!("NDCG@10:       {:.4}", report.ndcg_at_10);
    println!("MAE (top-10):  {:.6}", report.mean_abs_error);
    println!("Spearman rho:  {:.4}", report.spearman_rho);
    println!("Top-10 overlap:{:.2}%", report.top_k_overlap * 100.0);
}

// --- Uniform corpus tests (baseline) ---

#[test]
#[ignore]
fn quantizer_recall_50k_1536_4bit() {
    let report = run_recall_benchmark(50_000, 100, 1536, 4, 42);
    print_report("Quantizer Recall — Uniform (50K x 1536, 4-bit)", &report);
}

#[test]
#[ignore]
fn quantizer_recall_1k_1536_bitwidth_sweep() {
    println!("\n=== Bit-Width Sensitivity — Uniform (1K x 1536) ===");
    println!(
        "{:>5} {:>9} {:>10} {:>10} {:>8}",
        "bits", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );
    for bits in [2u8, 3, 4, 5, 6, 7, 8] {
        let report = run_recall_benchmark(1_000, 20, 1536, bits, 42);
        println!(
            "{:>5} {:>8.2}% {:>9.2}% {:>10.4} {:>8.6}",
            bits,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_1k_dimension_sweep() {
    println!("\n=== Dimension Sensitivity — Uniform (1K, 4-bit) ===");
    println!(
        "{:>6} {:>9} {:>10} {:>10}",
        "dim", "Recall@1", "Recall@10", "NDCG@10"
    );
    for dim in [128, 384, 768, 1536] {
        let report = run_recall_benchmark(1_000, 20, dim, 4, 42);
        println!(
            "{:>6} {:>8.2}% {:>9.2}% {:>10.4}",
            dim,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10
        );
    }
}

// --- Clustered corpus tests (realistic) ---

#[test]
#[ignore]
fn quantizer_recall_clustered_10k() {
    let dim = 1536;
    let seed = 42u64;
    let corpus = random_clustered_corpus(dim, 10_000, 50, 0.3, seed);
    // Queries are cluster members too — simulates "find similar" workload
    let queries = random_clustered_corpus(dim, 50, 50, 0.3, seed + 500_000);

    let report = run_recall_benchmark_with_corpus(&corpus, &queries, dim, 4, seed);
    print_report(
        "Quantizer Recall — Clustered (10K x 1536, 50 clusters, spread=0.3, 4-bit)",
        &report,
    );
}

#[test]
#[ignore]
fn quantizer_recall_clustered_10k_bitwidth_spot_check() {
    let dim = 1536;
    let seed = 42u64;
    let corpus = random_clustered_corpus(dim, 10_000, 50, 0.3, seed);
    let queries = random_clustered_corpus(dim, 50, 50, 0.3, seed + 500_000);

    println!("\n=== Bit-Width Spot Check — Clustered (10K x 1536, 50 clusters) ===");
    println!(
        "{:>5} {:>9} {:>10} {:>10} {:>8}",
        "bits", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );
    for bits in [3u8, 4, 6, 8] {
        let report = run_recall_benchmark_with_corpus(&corpus, &queries, dim, bits, seed);
        println!(
            "{:>5} {:>8.2}% {:>9.2}% {:>10.4} {:>8.6}",
            bits,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_clustered_bitwidth_sweep() {
    let dim = 1536;
    let seed = 42u64;
    let corpus = random_clustered_corpus(dim, 1_000, 20, 0.3, seed);
    let queries = random_clustered_corpus(dim, 20, 20, 0.3, seed + 500_000);

    println!("\n=== Bit-Width Sensitivity — Clustered (1K x 1536, 20 clusters) ===");
    println!(
        "{:>5} {:>9} {:>10} {:>10} {:>8}",
        "bits", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );
    for bits in [2u8, 3, 4, 6, 8] {
        let report = run_recall_benchmark_with_corpus(&corpus, &queries, dim, bits, seed);
        println!(
            "{:>5} {:>8.2}% {:>9.2}% {:>10.4} {:>8.6}",
            bits,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

// --- Near-duplicate stress test ---

#[test]
#[ignore]
fn quantizer_recall_near_duplicates() {
    let dim = 1536;
    let seed = 42u64;
    let quantizer = ProdQuantizer::new(dim, 4, seed);

    println!("\n=== Near-Duplicate Ranking Preservation (1536-dim, 4-bit) ===");
    println!(
        "{:>12} {:>12} {:>12} {:>12}",
        "angle(rad)", "preserved", "total", "rate"
    );

    for &angle in &[0.01f32, 0.02, 0.05, 0.1, 0.2] {
        let (bases, perturbed) = near_duplicate_pairs(dim, 200, angle, seed);
        let mut preserved = 0usize;

        for (base, pert) in bases.iter().zip(perturbed.iter()) {
            // True: base should be closer to itself than to perturbed (trivially true
            // since dot(base, base) = 1.0 > dot(base, pert) = cos(angle))
            // Test: does quantized scoring preserve this ordering?
            let prepared = quantizer.prepare_ip_query(base);
            let payload_base = quantizer.pack_payload(&quantizer.encode(base));
            let payload_pert = quantizer.pack_payload(&quantizer.encode(pert));
            let score_base = quantizer.score_ip_encoded(&prepared, &payload_base);
            let score_pert = quantizer.score_ip_encoded(&prepared, &payload_pert);

            if score_base > score_pert {
                preserved += 1;
            }
        }

        println!(
            "{:>12.4} {:>12} {:>12} {:>11.1}%",
            angle,
            preserved,
            200,
            preserved as f32 / 200.0 * 100.0
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_scoring_ablation_1k_uniform() {
    let dim = 1536;
    let seed = 42u64;
    let corpus: Vec<Vec<f32>> = (0..1_000)
        .map(|i| random_unit_vector(dim, seed + i as u64))
        .collect();
    let queries: Vec<Vec<f32>> = (0..20)
        .map(|i| random_unit_vector(dim, seed + 1_000_000 + i as u64))
        .collect();

    println!("\n=== Scoring Ablation — Uniform (1K x 1536, 4-bit) ===");
    println!(
        "{:>12} {:>10} {:>10} {:>10} {:>8}",
        "variant", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );

    for (label, variant) in [
        ("exact", ScoringVariant::Exact),
        ("gamma_zero", ScoringVariant::GammaZero),
        ("code_proxy", ScoringVariant::CodeProxy),
        ("decoded", ScoringVariant::DecodedApprox),
    ] {
        let report =
            run_recall_benchmark_with_corpus_variant(&corpus, &queries, dim, 4, seed, variant);
        println!(
            "{:>12} {:>9.2}% {:>9.2}% {:>10.4} {:>8.6}",
            label,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_1536_padding_ablations_1k_uniform() {
    let dim = 1536;
    let seed = 42u64;
    let bits = 4u8;
    let transform = transform_dim(dim);
    let corpus: Vec<Vec<f32>> = (0..1_000)
        .map(|i| random_unit_vector(dim, seed + i as u64))
        .collect();
    let queries: Vec<Vec<f32>> = (0..20)
        .map(|i| random_unit_vector(dim, seed + 1_000_000 + i as u64))
        .collect();

    let baseline = ProdQuantizer::new(dim, bits, seed);
    let mut transform_codebook = ProdQuantizer::new(dim, bits, seed);
    transform_codebook.codebook = lloyd_max((bits - 1) as usize, transform, 20_000)
        .into_iter()
        .map(|value| value as f32)
        .collect();

    println!("\n=== 1536 Padding Ablations — Uniform (1K x 1536, 4-bit) ===");
    println!(
        "{:>24} {:>10} {:>10} {:>10} {:>8}",
        "variant", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );

    for (label, report) in [
        (
            "current_exact",
            run_recall_benchmark_with_quantizer(&corpus, &queries, &baseline),
        ),
        (
            "transform_cb_exact",
            run_recall_benchmark_with_quantizer(&corpus, &queries, &transform_codebook),
        ),
        (
            "tail_ref_cb1536",
            run_tail_mse_reference(&corpus, &queries, dim, bits, seed, dim),
        ),
        (
            "tail_ref_cb2048",
            run_tail_mse_reference(&corpus, &queries, dim, bits, seed, transform),
        ),
    ] {
        println!(
            "{:>24} {:>9.2}% {:>9.2}% {:>10.4} {:>8.6}",
            label,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_1536_padding_ablations_1k_clustered() {
    let dim = 1536;
    let seed = 42u64;
    let bits = 4u8;
    let transform = transform_dim(dim);
    let corpus = random_clustered_corpus(dim, 1_000, 20, 0.3, seed);
    let queries = random_clustered_corpus(dim, 20, 20, 0.3, seed + 500_000);

    let baseline = ProdQuantizer::new(dim, bits, seed);
    let mut transform_codebook = ProdQuantizer::new(dim, bits, seed);
    transform_codebook.codebook = lloyd_max((bits - 1) as usize, transform, 20_000)
        .into_iter()
        .map(|value| value as f32)
        .collect();

    println!("\n=== 1536 Padding Ablations — Clustered (1K x 1536, 4-bit) ===");
    println!(
        "{:>24} {:>10} {:>10} {:>10} {:>8}",
        "variant", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );

    for (label, report) in [
        (
            "current_exact",
            run_recall_benchmark_with_quantizer(&corpus, &queries, &baseline),
        ),
        (
            "transform_cb_exact",
            run_recall_benchmark_with_quantizer(&corpus, &queries, &transform_codebook),
        ),
        (
            "tail_ref_cb1536",
            run_tail_mse_reference(&corpus, &queries, dim, bits, seed, dim),
        ),
        (
            "tail_ref_cb2048",
            run_tail_mse_reference(&corpus, &queries, dim, bits, seed, transform),
        ),
    ] {
        println!(
            "{:>24} {:>9.2}% {:>9.2}% {:>10.4} {:>8.6}",
            label,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_1536_tiled_codebook_dim_sweep_1k_uniform() {
    let dim = 1536;
    let seed = 42u64;
    let bits = 4u8;
    let tile_dim = 512usize;
    let transform = transform_dim(dim);
    let corpus: Vec<Vec<f32>> = (0..1_000)
        .map(|i| random_unit_vector(dim, seed + i as u64))
        .collect();
    let queries: Vec<Vec<f32>> = (0..20)
        .map(|i| random_unit_vector(dim, seed + 1_000_000 + i as u64))
        .collect();

    println!("\n=== 1536 Tiled Production Codebook Sweep — Uniform (1K x 1536, 4-bit) ===");
    println!(
        "{:>24} {:>10} {:>10} {:>10} {:>8}",
        "variant", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );

    for (label, report) in [
        (
            "prod_cb512",
            run_recall_benchmark_with_quantizer(
                &corpus,
                &queries,
                &prod_quantizer_with_codebook_dim(dim, bits, seed, tile_dim),
            ),
        ),
        (
            "prod_cb1536",
            run_recall_benchmark_with_quantizer(
                &corpus,
                &queries,
                &prod_quantizer_with_codebook_dim(dim, bits, seed, dim),
            ),
        ),
        (
            "prod_cb2048",
            run_recall_benchmark_with_quantizer(
                &corpus,
                &queries,
                &prod_quantizer_with_codebook_dim(dim, bits, seed, transform),
            ),
        ),
    ] {
        println!(
            "{:>24} {:>9.2}% {:>9.2}% {:>10.4} {:>8.6}",
            label,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_1536_tiled_codebook_dim_sweep_1k_clustered() {
    let dim = 1536;
    let seed = 42u64;
    let bits = 4u8;
    let tile_dim = 512usize;
    let transform = transform_dim(dim);
    let corpus = random_clustered_corpus(dim, 1_000, 20, 0.3, seed);
    let queries = random_clustered_corpus(dim, 20, 20, 0.3, seed + 500_000);

    println!("\n=== 1536 Tiled Production Codebook Sweep — Clustered (1K x 1536, 4-bit) ===");
    println!(
        "{:>24} {:>10} {:>10} {:>10} {:>8}",
        "variant", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );

    for (label, report) in [
        (
            "prod_cb512",
            run_recall_benchmark_with_quantizer(
                &corpus,
                &queries,
                &prod_quantizer_with_codebook_dim(dim, bits, seed, tile_dim),
            ),
        ),
        (
            "prod_cb1536",
            run_recall_benchmark_with_quantizer(
                &corpus,
                &queries,
                &prod_quantizer_with_codebook_dim(dim, bits, seed, dim),
            ),
        ),
        (
            "prod_cb2048",
            run_recall_benchmark_with_quantizer(
                &corpus,
                &queries,
                &prod_quantizer_with_codebook_dim(dim, bits, seed, transform),
            ),
        ),
    ] {
        println!(
            "{:>24} {:>9.2}% {:>9.2}% {:>10.4} {:>8.6}",
            label,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_1536_full_tail_exact_1k_uniform() {
    let dim = 1536;
    let seed = 42u64;
    let bits = 4u8;
    let transform = transform_dim(dim);
    let corpus: Vec<Vec<f32>> = (0..1_000)
        .map(|i| random_unit_vector(dim, seed + i as u64))
        .collect();
    let queries: Vec<Vec<f32>> = (0..20)
        .map(|i| random_unit_vector(dim, seed + 1_000_000 + i as u64))
        .collect();

    println!("\n=== 1536 Full-Tail Exact Reference — Uniform (1K x 1536, 4-bit) ===");
    println!(
        "{:>24} {:>10} {:>10} {:>10} {:>8}",
        "variant", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );

    for (label, report) in [
        (
            "current_exact",
            run_recall_benchmark_with_quantizer(
                &corpus,
                &queries,
                &ProdQuantizer::new(dim, bits, seed),
            ),
        ),
        (
            "tail_full_cb1536",
            run_tail_full_reference(&corpus, &queries, dim, bits, seed, dim),
        ),
        (
            "tail_full_cb2048",
            run_tail_full_reference(&corpus, &queries, dim, bits, seed, transform),
        ),
    ] {
        println!(
            "{:>24} {:>9.2}% {:>9.2}% {:>10.4} {:>8.6}",
            label,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_1536_full_tail_exact_1k_clustered() {
    let dim = 1536;
    let seed = 42u64;
    let bits = 4u8;
    let transform = transform_dim(dim);
    let corpus = random_clustered_corpus(dim, 1_000, 20, 0.3, seed);
    let queries = random_clustered_corpus(dim, 20, 20, 0.3, seed + 500_000);

    println!("\n=== 1536 Full-Tail Exact Reference — Clustered (1K x 1536, 4-bit) ===");
    println!(
        "{:>24} {:>10} {:>10} {:>10} {:>8}",
        "variant", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );

    for (label, report) in [
        (
            "current_exact",
            run_recall_benchmark_with_quantizer(
                &corpus,
                &queries,
                &ProdQuantizer::new(dim, bits, seed),
            ),
        ),
        (
            "tail_full_cb1536",
            run_tail_full_reference(&corpus, &queries, dim, bits, seed, dim),
        ),
        (
            "tail_full_cb2048",
            run_tail_full_reference(&corpus, &queries, dim, bits, seed, transform),
        ),
    ] {
        println!(
            "{:>24} {:>9.2}% {:>9.2}% {:>10.4} {:>8.6}",
            label,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_1536_upstream_gap_10k_clustered() {
    let dim = 1536;
    let seed = 42u64;
    let bits = 4u8;
    let transform = transform_dim(dim);
    let corpus = random_clustered_corpus(dim, 10_000, 50, 0.3, seed);
    let queries = random_clustered_corpus(dim, 50, 50, 0.3, seed + 500_000);

    println!("\n=== 1536 Upstream Gap Check — Clustered (10K x 1536, 4-bit) ===");
    println!(
        "{:>24} {:>10} {:>10} {:>10} {:>8}",
        "variant", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );

    for (label, report) in [
        (
            "current_exact",
            run_recall_benchmark_with_quantizer(
                &corpus,
                &queries,
                &ProdQuantizer::new(dim, bits, seed),
            ),
        ),
        (
            "prod_cb2048",
            run_recall_benchmark_with_quantizer(
                &corpus,
                &queries,
                &prod_quantizer_with_codebook_dim(dim, bits, seed, transform),
            ),
        ),
        (
            "tail_full_cb1536",
            run_tail_full_reference(&corpus, &queries, dim, bits, seed, dim),
        ),
        (
            "tail_full_cb2048",
            run_tail_full_reference(&corpus, &queries, dim, bits, seed, transform),
        ),
    ] {
        println!(
            "{:>24} {:>9.2}% {:>9.2}% {:>10.4} {:>8.6}",
            label,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_1536_payload_equivalent_operating_points_10k_clustered() {
    let dim = 1536;
    let seed = 42u64;
    let transform = transform_dim(dim);
    let corpus = random_clustered_corpus(dim, 10_000, 50, 0.3, seed);
    let queries = random_clustered_corpus(dim, 50, 50, 0.3, seed + 500_000);

    println!("\n=== 1536 Payload-Equivalent Operating Points — Clustered (10K x 1536) ===");
    println!(
        "{:>24} {:>10} {:>10} {:>10} {:>8}",
        "variant", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );

    for (label, report) in [
        (
            "current_1536_4bit",
            run_recall_benchmark_with_corpus(&corpus, &queries, dim, 4, seed),
        ),
        (
            "fulln_2048_3bit",
            run_tail_full_reference(&corpus, &queries, dim, 3, seed, transform),
        ),
        (
            "current_1536_6bit",
            run_recall_benchmark_with_corpus(&corpus, &queries, dim, 6, seed),
        ),
    ] {
        println!(
            "{:>24} {:>9.2}% {:>9.2}% {:>10.4} {:>8.6}",
            label,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_1536_same_payload_qjl_vs_mse_10k_clustered() {
    let dim = 1536;
    let seed = 42u64;
    let tile_dim = 512usize;
    let corpus = random_clustered_corpus(dim, 10_000, 50, 0.3, seed);
    let queries = random_clustered_corpus(dim, 50, 50, 0.3, seed + 500_000);

    println!("\n=== 1536 Same-Payload QJL vs MSE — Clustered (10K x 1536) ===");
    println!(
        "{:>24} {:>10} {:>10} {:>10} {:>8}",
        "variant", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );

    for (label, report) in [
        (
            "legacy_3mse_plus_qjl",
            run_tiled_full_reference(
                &corpus,
                &queries,
                dim,
                4,
                seed,
                tile_dim,
                dim,
            ),
        ),
        (
            "current_4mse_no_qjl",
            run_recall_benchmark_with_corpus(&corpus, &queries, dim, 4, seed),
        ),
    ] {
        println!(
            "{:>24} {:>9.2}% {:>9.2}% {:>10.4} {:>8.6}",
            label,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_1536_qjl_increment_after_4mse_10k_clustered() {
    let dim = 1536;
    let seed = 42u64;
    let corpus = random_clustered_corpus(dim, 10_000, 50, 0.3, seed);
    let queries = random_clustered_corpus(dim, 50, 50, 0.3, seed + 500_000);

    println!("\n=== 1536 Thin-QJL Increment After 4 MSE Bits — Clustered (10K x 1536) ===");
    println!(
        "{:>24} {:>10} {:>10} {:>10} {:>8}",
        "variant", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );

    for (label, report) in [
        (
            "current_4mse_no_qjl",
            run_recall_benchmark_with_corpus(&corpus, &queries, dim, 4, seed),
        ),
        (
            "4mse_plus_qjl_g0",
            run_recall_benchmark_with_corpus_variant(
                &corpus,
                &queries,
                dim,
                5,
                seed,
                ScoringVariant::GammaZero,
            ),
        ),
        (
            "4mse_plus_qjl",
            run_recall_benchmark_with_corpus_variant(
                &corpus,
                &queries,
                dim,
                5,
                seed,
                ScoringVariant::Exact,
            ),
        ),
    ] {
        println!(
            "{:>24} {:>9.2}% {:>9.2}% {:>10.4} {:>8.6}",
            label,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_1536_4mse_codebook_dim_sweep_10k_clustered() {
    let dim = 1536;
    let bits = 4u8;
    let seed = 42u64;
    let corpus = random_clustered_corpus(dim, 10_000, 50, 0.3, seed);
    let queries = random_clustered_corpus(dim, 50, 50, 0.3, seed + 500_000);

    println!("\n=== 1536 4-MSE Codebook Sweep — Clustered (10K x 1536) ===");
    println!(
        "{:>20} {:>10} {:>10} {:>10} {:>8}",
        "variant", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );

    for (label, report) in [
        (
            "current_cb1536",
            run_recall_benchmark_with_corpus(&corpus, &queries, dim, bits, seed),
        ),
        (
            "4mse_cb512",
            run_recall_benchmark_with_quantizer(
                &corpus,
                &queries,
                &current_prod_quantizer_with_codebook_dim(dim, bits, seed, 512),
            ),
        ),
        (
            "4mse_cb1536",
            run_recall_benchmark_with_quantizer(
                &corpus,
                &queries,
                &current_prod_quantizer_with_codebook_dim(dim, bits, seed, dim),
            ),
        ),
        (
            "4mse_cb2048",
            run_recall_benchmark_with_quantizer(
                &corpus,
                &queries,
                &current_prod_quantizer_with_codebook_dim(dim, bits, seed, transform_dim(dim)),
            ),
        ),
    ] {
        println!(
            "{:>20} {:>9.2}% {:>9.2}% {:>10.4} {:>8.6}",
            label,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

#[test]
#[ignore]
fn quantizer_recall_1536_tiled_fwht_reference_1k_clustered() {
    let dim = 1536;
    let seed = 42u64;
    let bits = 4u8;
    let tile_dim = 512usize;
    let corpus = random_clustered_corpus(dim, 1_000, 20, 0.3, seed);
    let queries = random_clustered_corpus(dim, 20, 20, 0.3, seed + 500_000);

    println!("\n=== 1536 Tiled FWHT Reference — Clustered (1K x 1536, 4-bit) ===");
    println!(
        "{:>24} {:>10} {:>10} {:>10} {:>8}",
        "variant", "Recall@1", "Recall@10", "NDCG@10", "MAE"
    );

    for (label, report) in [
        (
            "current_exact",
            run_recall_benchmark_with_quantizer(
                &corpus,
                &queries,
                &ProdQuantizer::new(dim, bits, seed),
            ),
        ),
        (
            "tiled_full_cb512",
            run_tiled_full_reference(&corpus, &queries, dim, bits, seed, tile_dim, tile_dim),
        ),
        (
            "tiled_full_cb1536",
            run_tiled_full_reference(&corpus, &queries, dim, bits, seed, tile_dim, dim),
        ),
    ] {
        println!(
            "{:>24} {:>9.2}% {:>9.2}% {:>10.4} {:>8.6}",
            label,
            report.recall_at_1 * 100.0,
            report.recall_at_10 * 100.0,
            report.ndcg_at_10,
            report.mean_abs_error
        );
    }
}

// Quick smoke test (not ignored) — validates the harness works
#[test]
fn recall_harness_smoke_test() {
    let report = run_recall_benchmark(100, 5, 32, 4, 42);
    assert!(report.recall_at_1 >= 0.0 && report.recall_at_1 <= 1.0);
    assert!(report.recall_at_10 >= 0.0 && report.recall_at_10 <= 1.0);
    assert!(report.ndcg_at_10 >= 0.0);
}

#[test]
fn recall_harness_clustered_smoke_test() {
    let corpus = random_clustered_corpus(32, 100, 5, 0.5, 42);
    let queries = random_clustered_corpus(32, 5, 5, 0.5, 99);
    let report = run_recall_benchmark_with_corpus(&corpus, &queries, 32, 4, 42);
    assert!(report.recall_at_1 >= 0.0 && report.recall_at_1 <= 1.0);
    assert!(report.recall_at_10 >= 0.0 && report.recall_at_10 <= 1.0);
}

// ---------------------------------------------------------------------------
// ann-benchmarks reference anchor (task 10055)
// ---------------------------------------------------------------------------

/// One-shot oracle: drives the canonical converter, the real-corpus loader,
/// and the `tqhnsw_graph_scan_recall_ann_benchmarks_reference` SQL probe end
/// to end against the Qdrant DBpedia 1M parquet, then asserts the measured
/// `recall@10` stays within the published 2% tolerance.
///
/// This test is `#[ignore]`d on purpose. It is **not** a CI gate. It is a
/// manual oracle that a reviewer can run when something feels off about the
/// real-corpus lane. See `docs/RECALL_ANN_BENCHMARKS_ANCHOR.md`.
///
/// Required environment variables:
///
/// - `TQV_ANCHOR_PARQUET` — path to the Qdrant
///   `dbpedia-entities-openai3-text-embedding-3-large-1536-1M` parquet file
///   or directory.
/// - `TQV_ANCHOR_OUTPUT_DIR` — directory the converter writes the staged TSV
///   pair and manifest into. Must be writable.
///
/// Optional environment variables:
///
/// - `TQV_PSQL_BIN` — `psql` client binary to use (defaults to `psql` on
///   `PATH`).
/// - `PGDATABASE`, `PGHOST`, `PGPORT`, `PGUSER` — standard libpq
///   connection environment.
/// - `TQV_ANCHOR_SKIP_LOAD=1` — skip the converter and loader steps and just
///   re-run the probe against an already-loaded corpus.
///
/// Run with:
///
/// ```bash
/// TQV_ANCHOR_PARQUET=/path/to/dbpedia-entities-...-1M/data \
/// TQV_ANCHOR_OUTPUT_DIR=/path/to/staged \
/// PGDATABASE=tqvector_bench \
/// cargo test --test recall_integration \
///     ann_benchmarks_anchor_within_tolerance -- --ignored --nocapture
/// ```
#[test]
#[ignore]
fn ann_benchmarks_anchor_within_tolerance() {
    use std::env;
    use std::path::PathBuf;
    use std::process::Command;

    // Keep this in sync with `ANN_BENCHMARKS_ANCHOR_PUBLISHED_RECALL_AT_10`
    // and `ANN_BENCHMARKS_ANCHOR_TOLERANCE` in `src/lib.rs`. The constants
    // are duplicated rather than re-exported because this test must be able
    // to run without linking the pgrx-built crate.
    const PUBLISHED_RECALL_AT_10: f32 = 0.96082_f32;
    const TOLERANCE: f32 = 0.02_f32;
    const PROFILE: &str = "tqhnsw_real_ann_benchmarks_anchor";

    let parquet = env::var("TQV_ANCHOR_PARQUET").expect(
        "TQV_ANCHOR_PARQUET must point at the Qdrant DBpedia 1M parquet (file or directory)",
    );
    let output_dir = env::var("TQV_ANCHOR_OUTPUT_DIR")
        .expect("TQV_ANCHOR_OUTPUT_DIR must point at a writable directory for staged TSVs");
    let skip_load = env::var("TQV_ANCHOR_SKIP_LOAD")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let psql_bin = env::var("TQV_PSQL_BIN").unwrap_or_else(|_| "psql".to_string());

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let converter = repo_root.join("scripts/qdrant_dbpedia_to_tsv.py");
    let loader = repo_root.join("scripts/load_real_corpus.py");
    let corpus_tsv = PathBuf::from(&output_dir).join(format!("{PROFILE}_corpus.tsv"));
    let queries_tsv = PathBuf::from(&output_dir).join(format!("{PROFILE}_queries.tsv"));

    if !skip_load {
        let convert_status = Command::new("python3")
            .arg(&converter)
            .arg("--profile")
            .arg(PROFILE)
            .arg("--parquet")
            .arg(&parquet)
            .arg("--output-dir")
            .arg(&output_dir)
            .status()
            .expect("converter should be invokable");
        assert!(
            convert_status.success(),
            "converter exited with {convert_status:?}"
        );

        let load_status = Command::new("python3")
            .arg(&loader)
            .arg("--prefix")
            .arg(PROFILE)
            .arg("--corpus-file")
            .arg(&corpus_tsv)
            .arg("--queries-file")
            .arg(&queries_tsv)
            .arg("--m")
            .arg("16")
            .status()
            .expect("loader should be invokable");
        assert!(load_status.success(), "loader exited with {load_status:?}");
    }

    let probe_sql = format!(
        "SELECT recall_at_10::text || '|' || absolute_delta::text || '|' || within_two_percent::text \
         FROM tqhnsw_graph_scan_recall_ann_benchmarks_reference(\
             '{PROFILE}_corpus', '{PROFILE}_queries', '{PROFILE}_m16_idx', 16, 128);"
    );
    let psql_output = Command::new(&psql_bin)
        .args(["-X", "-A", "-t", "-q", "-c"])
        .arg(&probe_sql)
        .output()
        .expect("psql should be invokable");
    assert!(
        psql_output.status.success(),
        "psql exited with {:?}\nstdout: {}\nstderr: {}",
        psql_output.status,
        String::from_utf8_lossy(&psql_output.stdout),
        String::from_utf8_lossy(&psql_output.stderr),
    );
    let stdout = String::from_utf8_lossy(&psql_output.stdout);
    let line = stdout
        .lines()
        .find(|l| l.contains('|'))
        .unwrap_or_else(|| panic!("anchor probe returned no rows: {stdout:?}"));
    let parts: Vec<&str> = line.split('|').collect();
    assert_eq!(
        parts.len(),
        3,
        "expected 3 fields, got {parts:?} from {line:?}"
    );
    let recall_at_10: f32 = parts[0]
        .parse()
        .unwrap_or_else(|e| panic!("could not parse recall_at_10 from {:?}: {e}", parts[0]));
    let absolute_delta: f32 = parts[1]
        .parse()
        .unwrap_or_else(|e| panic!("could not parse absolute_delta from {:?}: {e}", parts[1]));
    let within_two_percent: bool = parts[2].trim() == "t" || parts[2].trim() == "true";

    println!(
        "ann_benchmarks anchor: recall_at_10={recall_at_10:.5} \
         published={PUBLISHED_RECALL_AT_10:.5} \
         absolute_delta={absolute_delta:+.5} within_two_percent={within_two_percent}"
    );

    assert!(
        within_two_percent && absolute_delta.abs() <= TOLERANCE,
        "ann-benchmarks anchor drifted: measured recall@10={recall_at_10:.5}, \
         published recall@10={PUBLISHED_RECALL_AT_10:.5}, |delta|={:.5} > {TOLERANCE:.5}. \
         Do not adjust the published constant — investigate the converter, loader, \
         build path, or scan path. See docs/RECALL_ANN_BENCHMARKS_ANCHOR.md.",
        absolute_delta.abs()
    );
}
