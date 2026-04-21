//! `ecaz bench recall` — recall@k (and NDCG@k) sweep over a profile's tuning axis.
//!
//! # Flow
//!
//! 1. Fetch `<prefix>_corpus.source` and `<prefix>_queries.source` into an
//!    ndarray `Array2<f32>`.
//! 2. Compute ground truth with a parallel `queries · corpusᵀ` matmul
//!    (ndarray+rayon), then argsort the top-k per row.
//! 3. For each sweep value, set the profile's tuning GUC and run one
//!    `ORDER BY embedding <#> encode_to_<embedding>(...) LIMIT k` per query.
//! 4. Print a comfy-table: sweep value, recall@k, NDCG@k, mean query time.
//!
//! # Purity boundary
//!
//! The numerics (metrics + brute force) and the SQL template are pure
//! functions unit-tested in this module. The orchestration (`run`) is a
//! thin DB shell on top; live-Postgres coverage lands with the integration
//! suite.

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use indicatif::ProgressStyle;
use ndarray::Array2;
use rayon::prelude::*;
use std::time::{Duration, Instant};
use tokio_postgres::Client;

use crate::profiles::{self, IndexProfile};
use crate::psql::{self, ConnectionOptions};

#[derive(Args, Debug)]
pub struct RecallArgs {
    /// Prefix identifying the corpus (as loaded by `ecaz corpus load`).
    #[arg(long)]
    pub prefix: String,
    /// Access-method profile to measure.
    #[arg(long, default_value = "ec_hnsw")]
    pub profile: String,
    /// k for recall@k / NDCG@k.
    #[arg(long, default_value_t = 10)]
    pub k: usize,
    /// Sweep values for the profile's tuning axis. Accepts `--sweep 100,200,400`
    /// or repeated `--sweep 100 --sweep 200`.
    #[arg(long, value_delimiter = ',')]
    pub sweep: Vec<i32>,
    /// Cap the query set (default: all rows in `<prefix>_queries`).
    #[arg(long)]
    pub queries_limit: Option<usize>,
    /// Quantization bits used when encoding query vectors at scan time.
    /// Must match the loader's `--bits` for the embedding column.
    #[arg(long, default_value_t = 4)]
    pub bits: i32,
    /// Quantizer seed (must match loader's `--seed`).
    #[arg(long, default_value_t = 42)]
    pub seed: i64,
}

pub async fn run(conn: &ConnectionOptions, args: RecallArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    if args.k == 0 {
        return Err(eyre!("--k must be >= 1"));
    }
    let profile = profiles::resolve(&args.profile).ok_or_else(|| {
        eyre!(
            "unknown profile {:?}; try {}",
            args.profile,
            profiles::names().join(", ")
        )
    })?;
    let guc = profile
        .ef_search_guc
        .ok_or_else(|| eyre!("profile {:?} has no tuning GUC to sweep", profile.name))?;
    let sweep_values: Vec<i32> = if args.sweep.is_empty() {
        if profile.default_sweep.is_empty() {
            return Err(eyre!(
                "--sweep is required for profile {:?} (no default sweep registered)",
                profile.name
            ));
        }
        crate::ecaz_eprintln!(
            "[recall] no --sweep provided; using profile default {} values {:?}",
            profile.sweep_axis_label(),
            profile.default_sweep
        );
        profile.default_sweep.to_vec()
    } else {
        args.sweep.clone()
    };

    let corpus_table = format!("{}_corpus", args.prefix);
    let queries_table = format!("{}_queries", args.prefix);

    let client = psql::connect(conn).await?;
    if psql::index_count_with_am(&client, &corpus_table, profile.access_method).await? == 0 {
        return Err(eyre!(
            "{} on {:?}",
            super::missing_am_error(profile, profile.access_method),
            corpus_table
        ));
    }
    crate::ecaz_eprintln!("[recall] fetching corpus from {corpus_table} ...");
    let (corpus_ids, corpus) = fetch_sources(&client, &corpus_table, None).await?;
    crate::ecaz_eprintln!("[recall] fetching queries from {queries_table} ...");
    let (_query_ids, queries) = fetch_sources(&client, &queries_table, args.queries_limit).await?;
    if corpus.nrows() == 0 || queries.nrows() == 0 {
        return Err(eyre!(
            "corpus ({} rows) or queries ({} rows) is empty",
            corpus.nrows(),
            queries.nrows()
        ));
    }
    if corpus.ncols() != queries.ncols() {
        return Err(eyre!(
            "dim mismatch: corpus={} queries={}",
            corpus.ncols(),
            queries.ncols()
        ));
    }

    crate::ecaz_eprintln!(
        "[recall] computing ground truth: {} queries vs {} corpus rows (dim={}) ...",
        queries.nrows(),
        corpus.nrows(),
        corpus.ncols()
    );
    let t0 = Instant::now();
    let gt = brute_force_top_k(&corpus, &queries, args.k);
    crate::ecaz_eprintln!("[recall] ground truth in {:.2?}", t0.elapsed());

    let sql = build_knn_sql(profile, &corpus_table);

    let mut t = Table::new();
    t.load_preset(UTF8_FULL);
    t.set_header(vec![
        profile.sweep_axis_label(),
        "recall@k",
        "ndcg@k",
        "mean q-time",
    ]);

    for value in &sweep_values {
        client
            .batch_execute(&format!("SET {guc} = {value}"))
            .await
            .wrap_err_with(|| format!("SET {guc} = {value}"))?;
        let bar = crate::output::progress_bar(queries.nrows() as u64);
        bar.set_style(
            ProgressStyle::with_template("[recall {msg}] {wide_bar} {pos}/{len} ({per_sec})")
                .unwrap(),
        );
        bar.set_message(super::sweep_value_label(profile, *value));
        bar.enable_steady_tick(Duration::from_millis(250));

        let mut pred: Vec<Vec<i64>> = Vec::with_capacity(queries.nrows());
        let mut total_ns: u128 = 0;
        let stmt = client
            .prepare(&sql)
            .await
            .wrap_err("preparing recall KNN statement")?;
        for q in 0..queries.nrows() {
            let row_vec: Vec<f32> = queries.row(q).to_vec();
            let t0 = Instant::now();
            let rows = client
                .query(&stmt, &[&row_vec, &args.bits, &args.seed, &(args.k as i64)])
                .await
                .wrap_err("executing recall KNN query")?;
            total_ns += t0.elapsed().as_nanos();
            pred.push(rows.iter().map(|r| r.get::<_, i64>(0)).collect());
            bar.inc(1);
        }
        bar.finish_and_clear();

        let truth_ids = map_indices_to_ids(&gt.indices, &corpus_ids);
        let recall = recall_at_k(&truth_ids, &pred, args.k);
        let ndcg = ndcg_at_k(&gt.scores, &pred, &corpus_ids, &gt.all_scores, args.k);
        let mean_ms = (total_ns as f64 / queries.nrows() as f64) / 1e6;

        t.add_row(vec![
            Cell::new(value),
            Cell::new(format!("{:.4}", recall)),
            Cell::new(format!("{:.4}", ndcg)),
            Cell::new(format!("{:.2} ms", mean_ms)),
        ]);
    }

    crate::ecaz_println!("{t}");
    Ok(())
}

/// `fetch_sources` reachable from sibling modules (e.g. `compare::pgvector`)
/// without exporting from the binary crate root.
pub async fn fetch_sources_public(
    client: &Client,
    table: &str,
    limit: Option<usize>,
) -> Result<(Vec<i64>, Array2<f32>)> {
    fetch_sources(client, table, limit).await
}

async fn fetch_sources(
    client: &Client,
    table: &str,
    limit: Option<usize>,
) -> Result<(Vec<i64>, Array2<f32>)> {
    let limit_clause = match limit {
        Some(n) => format!(" LIMIT {n}"),
        None => String::new(),
    };
    let sql = format!("SELECT id, source FROM {table} ORDER BY id{limit_clause}");
    let rows = client
        .query(sql.as_str(), &[])
        .await
        .wrap_err_with(|| format!("fetching {table}"))?;
    if rows.is_empty() {
        return Ok((vec![], Array2::<f32>::zeros((0, 0))));
    }
    let first: Vec<f32> = rows[0].get(1);
    let dim = first.len();
    let mut ids = Vec::with_capacity(rows.len());
    let mut flat = Vec::with_capacity(rows.len() * dim);
    for r in &rows {
        ids.push(r.get::<_, i64>(0));
        let v: Vec<f32> = r.get(1);
        if v.len() != dim {
            return Err(eyre!(
                "{table}: row id={} has dim {}, expected {}",
                ids.last().unwrap(),
                v.len(),
                dim
            ));
        }
        flat.extend_from_slice(&v);
    }
    let arr = Array2::from_shape_vec((rows.len(), dim), flat)?;
    Ok((ids, arr))
}

/// Ground-truth bundle. `indices[q]` is the sorted list of *row positions*
/// (not ids) into the corpus; `scores[q]` is the matching IP scores;
/// `all_scores[q]` is the full score row (queries · corpusᵀ) used for NDCG
/// ideal-DCG computation.
#[derive(Debug)]
pub struct GroundTruth {
    pub indices: Vec<Vec<usize>>,
    pub scores: Vec<Vec<f32>>,
    pub all_scores: Array2<f32>,
}

/// Brute-force top-k by inner product. Uses ndarray's BLAS-backed matmul
/// when available, otherwise the Rust-only fallback. Rayon parallelises
/// the per-query argsort.
pub fn brute_force_top_k(corpus: &Array2<f32>, queries: &Array2<f32>, k: usize) -> GroundTruth {
    let scores = queries.dot(&corpus.t()); // (q, n)
    let n = scores.ncols();
    let k = k.min(n);
    let per_query: Vec<(Vec<usize>, Vec<f32>)> = (0..scores.nrows())
        .into_par_iter()
        .map(|q| {
            let row = scores.row(q);
            top_k_desc(row.as_slice().expect("contiguous"), k)
        })
        .collect();
    let (indices, scores_out): (Vec<_>, Vec<_>) = per_query.into_iter().unzip();
    GroundTruth {
        indices,
        scores: scores_out,
        all_scores: scores,
    }
}

/// Return `(indices, scores)` of the top-k entries of `row`, sorted by
/// score descending. Stable ordering on ties (lower index first) so unit
/// tests are deterministic.
fn top_k_desc(row: &[f32], k: usize) -> (Vec<usize>, Vec<f32>) {
    if k == 0 || row.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let mut idx: Vec<usize> = (0..row.len()).collect();
    // Partial sort would be nice, but for ground-truth correctness over
    // corpus sizes up to ~1M this full sort is fine in practice (single-
    // digit seconds, dominated by the matmul anyway).
    idx.sort_by(|&a, &b| {
        row[b]
            .partial_cmp(&row[a])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.cmp(&b))
    });
    idx.truncate(k);
    let scores: Vec<f32> = idx.iter().map(|&i| row[i]).collect();
    (idx, scores)
}

/// Map per-query row-position indices into corpus ids for set-membership
/// comparison against the CLI's SQL results (which return ids).
pub fn map_indices_to_ids(indices: &[Vec<usize>], ids: &[i64]) -> Vec<Vec<i64>> {
    indices
        .iter()
        .map(|row| row.iter().map(|&i| ids[i]).collect())
        .collect()
}

/// Classic recall@k: fraction of the true top-k ids present in the
/// predicted top-k ids, averaged over queries. Per-query denominator is
/// `k` (not `min(k, len(pred))`) to match the legacy benchmark.
pub fn recall_at_k(truth: &[Vec<i64>], pred: &[Vec<i64>], k: usize) -> f64 {
    if truth.is_empty() || k == 0 {
        return 0.0;
    }
    let mut hits = 0usize;
    for (t, p) in truth.iter().zip(pred.iter()) {
        let t_set: std::collections::HashSet<i64> = t.iter().take(k).copied().collect();
        for pid in p.iter().take(k) {
            if t_set.contains(pid) {
                hits += 1;
            }
        }
    }
    hits as f64 / (truth.len() * k) as f64
}

/// NDCG@k using the true IP score as relevance (clamped at 0). Ideal DCG
/// is computed from the ground-truth scores; predicted DCG looks up each
/// predicted id in the row of all scores so irrelevant results contribute
/// their real relevance, not 0.
pub fn ndcg_at_k(
    true_scores: &[Vec<f32>],
    pred_ids: &[Vec<i64>],
    corpus_ids: &[i64],
    all_scores: &Array2<f32>,
    k: usize,
) -> f64 {
    if pred_ids.is_empty() || k == 0 {
        return 0.0;
    }
    // id -> corpus-row-position lookup, built once.
    let id_to_pos: std::collections::HashMap<i64, usize> = corpus_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();
    let log2 = |x: f64| x.log2();
    let mut sum = 0.0f64;
    for (q, pred) in pred_ids.iter().enumerate() {
        let mut dcg = 0.0f64;
        for (rank, pid) in pred.iter().take(k).enumerate() {
            let pos = match id_to_pos.get(pid) {
                Some(&p) => p,
                None => continue,
            };
            let rel = all_scores[[q, pos]].max(0.0) as f64;
            dcg += rel / log2((rank + 2) as f64);
        }
        let mut idcg = 0.0f64;
        for (rank, score) in true_scores[q].iter().take(k).enumerate() {
            let rel = (*score).max(0.0) as f64;
            idcg += rel / log2((rank + 2) as f64);
        }
        let denom = idcg.max(1e-10);
        sum += dcg / denom;
    }
    sum / pred_ids.len() as f64
}

/// KNN SQL template used for recall. Binds are `($1::real[], $2::integer,
/// $3::bigint, $4::bigint)` = (query_source, bits, seed, k). Exposed so a
/// test can pin the operator and encoder wiring for each profile.
pub fn build_knn_sql(profile: &IndexProfile, corpus_table: &str) -> String {
    format!(
        "SELECT id FROM {corpus_table} \
         ORDER BY embedding <#> \
         {enc}($1::real[], $2::integer, $3::bigint) \
         LIMIT $4",
        enc = profile.encoder_function,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::{EC_DISKANN, EC_HNSW};
    use ndarray::arr2;

    // --- top_k_desc ---

    #[test]
    fn top_k_desc_sorts_by_score_descending() {
        let (idx, scores) = top_k_desc(&[0.1, 0.5, 0.2, 0.9, 0.3], 3);
        assert_eq!(idx, vec![3, 1, 4]);
        assert_eq!(scores, vec![0.9, 0.5, 0.3]);
    }

    #[test]
    fn top_k_desc_stable_on_ties_prefers_lower_index() {
        let (idx, _) = top_k_desc(&[0.5, 0.5, 0.5, 0.5], 2);
        assert_eq!(idx, vec![0, 1]);
    }

    #[test]
    fn top_k_desc_k_greater_than_len_returns_all() {
        let (idx, _) = top_k_desc(&[0.2, 0.8], 5);
        assert_eq!(idx, vec![1, 0]);
    }

    #[test]
    fn top_k_desc_handles_empty_inputs() {
        let (idx, sc) = top_k_desc(&[], 3);
        assert!(idx.is_empty() && sc.is_empty());
        let (idx, sc) = top_k_desc(&[1.0, 2.0], 0);
        assert!(idx.is_empty() && sc.is_empty());
    }

    // --- brute_force_top_k ---

    #[test]
    fn brute_force_matches_hand_computed_inner_products() {
        // 2 queries × 3 corpus rows, dim 2; IP = Q · Cᵀ
        let corpus = arr2(&[[1.0_f32, 0.0], [0.0, 1.0], [1.0, 1.0]]);
        let queries = arr2(&[[1.0_f32, 0.0], [0.5, 0.5]]);
        let gt = brute_force_top_k(&corpus, &queries, 2);
        // q0: scores = [1, 0, 1] → top 2 by (score desc, idx asc) = [0, 2]
        assert_eq!(gt.indices[0], vec![0, 2]);
        // q1: scores = [0.5, 0.5, 1.0] → [2, 0]
        assert_eq!(gt.indices[1], vec![2, 0]);
        // all_scores matches
        assert!((gt.all_scores[[1, 2]] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn brute_force_k_truncation_is_safe() {
        let corpus = arr2(&[[1.0_f32], [2.0], [3.0]]);
        let queries = arr2(&[[1.0_f32]]);
        let gt = brute_force_top_k(&corpus, &queries, 100);
        assert_eq!(gt.indices[0].len(), 3);
    }

    // --- recall_at_k ---

    #[test]
    fn recall_at_k_perfect_predictions_equals_1() {
        let truth = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let pred = truth.clone();
        assert!((recall_at_k(&truth, &pred, 3) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn recall_at_k_disjoint_predictions_equals_0() {
        let truth = vec![vec![1, 2], vec![3, 4]];
        let pred = vec![vec![9, 8], vec![7, 6]];
        assert_eq!(recall_at_k(&truth, &pred, 2), 0.0);
    }

    #[test]
    fn recall_at_k_partial_hit_is_fraction_hits_over_k_times_queries() {
        // 2 queries, k=2. q0 hits 1/2, q1 hits 2/2 → 3 / (2*2) = 0.75
        let truth = vec![vec![1, 2], vec![3, 4]];
        let pred = vec![vec![1, 99], vec![3, 4]];
        let got = recall_at_k(&truth, &pred, 2);
        assert!((got - 0.75).abs() < 1e-9, "got {got}");
    }

    #[test]
    fn recall_at_k_empty_inputs_are_zero() {
        assert_eq!(recall_at_k(&[], &[], 10), 0.0);
        let truth = vec![vec![1]];
        let pred = vec![vec![1]];
        assert_eq!(recall_at_k(&truth, &pred, 0), 0.0);
    }

    #[test]
    fn recall_at_k_respects_k_cap_when_pred_is_longer() {
        // pred has k+1 entries; the extra should not count.
        let truth = vec![vec![1, 2]];
        let pred = vec![vec![1, 99, 2]]; // within k=2, only 1 is a hit
        let got = recall_at_k(&truth, &pred, 2);
        assert!((got - 0.5).abs() < 1e-9, "got {got}");
    }

    // --- ndcg_at_k ---

    fn toy_ndcg_inputs() -> (Vec<Vec<f32>>, Vec<i64>, Array2<f32>) {
        // 1 query, 3 corpus rows with ids [10, 20, 30], all_scores = [1.0, 0.5, 0.0]
        let true_scores = vec![vec![1.0_f32, 0.5, 0.0]];
        let ids = vec![10_i64, 20, 30];
        let all_scores = arr2(&[[1.0_f32, 0.5, 0.0]]);
        (true_scores, ids, all_scores)
    }

    #[test]
    fn ndcg_at_k_perfect_ranking_equals_1() {
        let (ts, ids, sc) = toy_ndcg_inputs();
        let pred = vec![vec![10_i64, 20, 30]];
        let n = ndcg_at_k(&ts, &pred, &ids, &sc, 3);
        assert!((n - 1.0).abs() < 1e-6, "got {n}");
    }

    #[test]
    fn ndcg_at_k_inverted_ranking_is_less_than_perfect() {
        let (ts, ids, sc) = toy_ndcg_inputs();
        let pred = vec![vec![30_i64, 20, 10]];
        let n = ndcg_at_k(&ts, &pred, &ids, &sc, 3);
        assert!(n < 1.0 && n > 0.0, "got {n}");
    }

    #[test]
    fn ndcg_at_k_ignores_unknown_ids() {
        // Predicted id 999 is not in corpus_ids; it should contribute 0 DCG
        // rather than crash.
        let (ts, ids, sc) = toy_ndcg_inputs();
        let pred = vec![vec![10_i64, 999, 20]];
        let n = ndcg_at_k(&ts, &pred, &ids, &sc, 3);
        assert!(n > 0.0 && n < 1.0, "got {n}");
    }

    #[test]
    fn ndcg_at_k_clamps_negative_relevance_to_zero() {
        // Negative IP scores shouldn't produce negative DCG.
        let ts = vec![vec![1.0_f32, -0.5]];
        let ids = vec![1_i64, 2];
        let sc = arr2(&[[1.0_f32, -0.5]]);
        let pred = vec![vec![2_i64, 1]];
        let n = ndcg_at_k(&ts, &pred, &ids, &sc, 2);
        assert!((0.0..=1.0).contains(&n), "got {n}");
    }

    #[test]
    fn ndcg_at_k_zero_ideal_avoids_division_by_zero() {
        // All true scores are 0 → idcg = 0 → denom fallback to 1e-10, final
        // value is a small number, not NaN/Inf.
        let ts = vec![vec![0.0_f32, 0.0]];
        let ids = vec![1_i64, 2];
        let sc = arr2(&[[0.0_f32, 0.0]]);
        let pred = vec![vec![1_i64, 2]];
        let n = ndcg_at_k(&ts, &pred, &ids, &sc, 2);
        assert!(n.is_finite(), "got {n}");
    }

    // --- build_knn_sql ---

    #[test]
    fn build_knn_sql_uses_profile_encoder_and_ip_operator() {
        let sql = build_knn_sql(&EC_HNSW, "dbpedia_10k_corpus");
        assert!(sql.contains("FROM dbpedia_10k_corpus"));
        assert!(sql.contains("encode_to_ecvector($1::real[], $2::integer, $3::bigint)"));
        assert!(sql.contains("<#>"));
        assert!(!sql.contains("pg_catalog"), "got: {sql}");
        assert!(sql.contains("LIMIT $4"));
    }

    #[test]
    fn build_knn_sql_is_profile_polymorphic() {
        // DiskANN uses the same embedding type + encoder today, but the SQL
        // must reference the profile's `encoder_function` field, not a
        // hardcoded name.
        let sql = build_knn_sql(&EC_DISKANN, "corpus");
        assert!(sql.contains(EC_DISKANN.encoder_function));
    }

    // --- map_indices_to_ids ---

    #[test]
    fn map_indices_to_ids_translates_row_positions_to_ids() {
        let indices = vec![vec![0, 2], vec![1]];
        let ids = vec![100_i64, 200, 300];
        assert_eq!(
            map_indices_to_ids(&indices, &ids),
            vec![vec![100, 300], vec![200]]
        );
    }
}
