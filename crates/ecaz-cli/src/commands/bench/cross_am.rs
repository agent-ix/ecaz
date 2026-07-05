//! Cross-AM top-k consistency metrics.
//!
//! This command consumes prediction files emitted by `ecaz bench recall` and
//! reports pairwise Jaccard membership agreement plus a bounded Kendall-style
//! rank agreement over the union of each AM's top-k results.

use clap::Args;
use color_eyre::eyre::{bail, eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use super::recall::PredictionFile;

#[derive(Args, Debug)]
pub struct CrossAmArgs {
    /// Prediction input in the form label=path. Repeat once per AM.
    #[arg(long = "input")]
    pub inputs: Vec<String>,

    /// k to compare. Defaults to the prediction file k.
    #[arg(long)]
    pub k: Option<usize>,

    /// Write the final cross-AM table to this path in addition to stdout.
    #[arg(long)]
    pub log_output: Option<PathBuf>,
}

#[derive(Debug)]
struct LabeledPrediction {
    label: String,
    file: PredictionFile,
}

#[derive(Clone, Debug, PartialEq)]
struct PairMetrics {
    left: String,
    right: String,
    queries: usize,
    k: usize,
    jaccard: f64,
    kendall_tau: f64,
}

pub async fn run(args: CrossAmArgs) -> Result<()> {
    if args.inputs.len() < 2 {
        bail!("cross-am requires at least two --input label=path entries");
    }
    let mut inputs = Vec::with_capacity(args.inputs.len());
    let mut labels = HashSet::new();
    for raw in &args.inputs {
        let input = load_labeled_prediction(raw).await?;
        if !labels.insert(input.label.clone()) {
            bail!("duplicate cross-am input label {:?}", input.label);
        }
        inputs.push(input);
    }
    let k = args.k.unwrap_or(inputs[0].file.k);
    let rows = pairwise_metrics(&inputs, k)?;

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["pair", "queries", "k", "jaccard@k", "kendall_tau@k"]);
    for row in &rows {
        table.add_row(vec![
            Cell::new(format!("{}~{}", row.left, row.right)),
            Cell::new(row.queries),
            Cell::new(row.k),
            Cell::new(format!("{:.4}", row.jaccard)),
            Cell::new(format!("{:.4}", row.kendall_tau)),
        ]);
    }
    let output = table.to_string();
    println!("{output}");
    if let Some(path) = args.log_output {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .wrap_err_with(|| format!("creating {}", parent.display()))?;
        }
        tokio::fs::write(&path, format!("{output}\n"))
            .await
            .wrap_err_with(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

async fn load_labeled_prediction(raw: &str) -> Result<LabeledPrediction> {
    let (label, path) = raw
        .split_once('=')
        .ok_or_else(|| eyre!("--input must be label=path, got {raw:?}"))?;
    if label.trim().is_empty() {
        bail!("--input label must not be empty");
    }
    let path = path.trim();
    if path.is_empty() {
        bail!("--input path must not be empty");
    }
    let path = Path::new(path);
    let bytes = tokio::fs::read(path)
        .await
        .wrap_err_with(|| format!("reading {}", path.display()))?;
    let file: PredictionFile =
        serde_json::from_slice(&bytes).wrap_err_with(|| format!("parsing {}", path.display()))?;
    validate_prediction_file(path, &file)?;
    Ok(LabeledPrediction {
        label: label.trim().to_string(),
        file,
    })
}

fn validate_prediction_file(path: &Path, file: &PredictionFile) -> Result<()> {
    if file.version != 1 {
        bail!(
            "prediction file {} has unsupported version {}",
            path.display(),
            file.version
        );
    }
    if file.k == 0 {
        bail!("prediction file {} has k=0", path.display());
    }
    if file.query_ids.is_empty() {
        bail!("prediction file {} has no query ids", path.display());
    }
    if file.rows.len() != 1 {
        bail!(
            "prediction file {} must contain exactly one sweep row for cross-AM comparison, found {}",
            path.display(),
            file.rows.len()
        );
    }
    let predictions = &file.rows[0].predictions;
    if predictions.len() != file.query_ids.len() {
        bail!(
            "prediction file {} has {} query ids but {} prediction rows",
            path.display(),
            file.query_ids.len(),
            predictions.len()
        );
    }
    Ok(())
}

fn pairwise_metrics(inputs: &[LabeledPrediction], k: usize) -> Result<Vec<PairMetrics>> {
    if k == 0 {
        bail!("--k must be >= 1");
    }
    for input in inputs {
        validate_prediction_depth(input, k)?;
    }
    let mut rows = Vec::new();
    for i in 0..inputs.len() {
        for j in (i + 1)..inputs.len() {
            rows.push(compare_pair(&inputs[i], &inputs[j], k)?);
        }
    }
    Ok(rows)
}

fn validate_prediction_depth(input: &LabeledPrediction, k: usize) -> Result<()> {
    if input.file.k < k {
        bail!(
            "prediction file for {} was emitted with k={} but cross-am requested k={}",
            input.label,
            input.file.k,
            k
        );
    }
    if let Some((idx, row)) = input.file.rows[0]
        .predictions
        .iter()
        .enumerate()
        .find(|(_, row)| row.len() < k)
    {
        bail!(
            "prediction file for {} query index {} has {} predictions, fewer than k={}",
            input.label,
            idx,
            row.len(),
            k
        );
    }
    Ok(())
}

fn compare_pair(
    left: &LabeledPrediction,
    right: &LabeledPrediction,
    k: usize,
) -> Result<PairMetrics> {
    if left.file.query_ids != right.file.query_ids {
        bail!(
            "prediction query ids differ between {} and {}",
            left.label,
            right.label
        );
    }
    let left_pred = &left.file.rows[0].predictions;
    let right_pred = &right.file.rows[0].predictions;
    let mut jaccard_sum = 0.0;
    let mut kendall_sum = 0.0;
    for (a, b) in left_pred.iter().zip(right_pred.iter()) {
        jaccard_sum += jaccard_at_k(a, b, k);
        kendall_sum += kendall_tau_at_k(a, b, k);
    }
    let queries = left_pred.len();
    if queries == 0 {
        bail!("prediction files contain no query rows");
    }
    Ok(PairMetrics {
        left: left.label.clone(),
        right: right.label.clone(),
        queries,
        k,
        jaccard: jaccard_sum / queries as f64,
        kendall_tau: kendall_sum / queries as f64,
    })
}

fn jaccard_at_k(a: &[i64], b: &[i64], k: usize) -> f64 {
    let a_set: HashSet<i64> = a.iter().take(k).copied().collect();
    let b_set: HashSet<i64> = b.iter().take(k).copied().collect();
    let union = a_set.union(&b_set).count();
    if union == 0 {
        return 1.0;
    }
    a_set.intersection(&b_set).count() as f64 / union as f64
}

fn kendall_tau_at_k(a: &[i64], b: &[i64], k: usize) -> f64 {
    let a_rank = rank_map(a, k);
    let b_rank = rank_map(b, k);
    let mut ids: Vec<i64> = a_rank.keys().chain(b_rank.keys()).copied().collect();
    ids.sort_unstable();
    ids.dedup();
    if ids.len() < 2 {
        return 1.0;
    }

    let absent_rank = k + 1;
    let mut concordant = 0usize;
    let mut discordant = 0usize;
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            let a_cmp = rank_order(
                *a_rank.get(&ids[i]).unwrap_or(&absent_rank),
                *a_rank.get(&ids[j]).unwrap_or(&absent_rank),
            );
            let b_cmp = rank_order(
                *b_rank.get(&ids[i]).unwrap_or(&absent_rank),
                *b_rank.get(&ids[j]).unwrap_or(&absent_rank),
            );
            if a_cmp == 0 || b_cmp == 0 {
                continue;
            }
            if a_cmp == b_cmp {
                concordant += 1;
            } else {
                discordant += 1;
            }
        }
    }
    let comparable = concordant + discordant;
    if comparable == 0 {
        return 1.0;
    }
    (concordant as f64 - discordant as f64) / comparable as f64
}

fn rank_map(values: &[i64], k: usize) -> BTreeMap<i64, usize> {
    let mut ranks = BTreeMap::new();
    for (idx, id) in values.iter().take(k).enumerate() {
        ranks.entry(*id).or_insert(idx + 1);
    }
    ranks
}

fn rank_order(left: usize, right: usize) -> i8 {
    match left.cmp(&right) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prediction(label: &str, rows: Vec<Vec<i64>>) -> LabeledPrediction {
        LabeledPrediction {
            label: label.to_string(),
            file: PredictionFile {
                version: 1,
                prefix: "p".into(),
                profile: label.into(),
                k: 3,
                query_ids: vec![10, 20],
                rows: vec![super::super::recall::PredictionSweep {
                    sweep_axis: "ef_search".into(),
                    sweep_value: 128,
                    rerank_width: None,
                    predictions: rows,
                }],
            },
        }
    }

    #[test]
    fn jaccard_is_one_for_identical_topk() {
        assert_eq!(jaccard_at_k(&[1, 2, 3], &[1, 2, 3], 3), 1.0);
    }

    #[test]
    fn jaccard_uses_topk_membership_only() {
        let got = jaccard_at_k(&[1, 2, 3, 9], &[2, 3, 4, 1], 3);
        assert!((got - 0.5).abs() < 1e-9, "got {got}");
    }

    #[test]
    fn kendall_is_one_for_same_rank_order() {
        assert_eq!(kendall_tau_at_k(&[1, 2, 3], &[1, 2, 3], 3), 1.0);
    }

    #[test]
    fn kendall_is_negative_for_reversed_rank_order() {
        assert_eq!(kendall_tau_at_k(&[1, 2, 3], &[3, 2, 1], 3), -1.0);
    }

    #[test]
    fn kendall_penalizes_membership_and_rank_disagreement() {
        let got = kendall_tau_at_k(&[1, 2, 3], &[2, 3, 4], 3);
        assert!(got < 1.0, "got {got}");
    }

    #[test]
    fn pairwise_metrics_average_over_queries() {
        let left = prediction("hnsw", vec![vec![1, 2, 3], vec![1, 2, 3]]);
        let right = prediction("ivf", vec![vec![1, 2, 3], vec![3, 2, 1]]);
        let got = compare_pair(&left, &right, 3).unwrap();
        assert_eq!(got.queries, 2);
        assert_eq!(got.jaccard, 1.0);
        assert!((got.kendall_tau - 0.0).abs() < 1e-9, "got {got:?}");
    }

    #[test]
    fn compare_pair_rejects_mismatched_query_ids() {
        let left = prediction("hnsw", vec![vec![1], vec![2]]);
        let mut right = prediction("ivf", vec![vec![1], vec![2]]);
        right.file.query_ids = vec![10, 30];
        assert!(compare_pair(&left, &right, 1)
            .unwrap_err()
            .to_string()
            .contains("query ids differ"));
    }

    #[test]
    fn pairwise_metrics_rejects_requested_k_above_file_k() {
        let left = prediction("hnsw", vec![vec![1, 2, 3], vec![1, 2, 3]]);
        let right = prediction("ivf", vec![vec![1, 2, 3], vec![1, 2, 3]]);

        assert!(pairwise_metrics(&[left, right], 4)
            .unwrap_err()
            .to_string()
            .contains("emitted with k=3"));
    }

    #[test]
    fn pairwise_metrics_rejects_short_prediction_rows() {
        let left = prediction("hnsw", vec![vec![1, 2, 3], vec![1, 2]]);
        let right = prediction("ivf", vec![vec![1, 2, 3], vec![1, 2, 3]]);

        assert!(pairwise_metrics(&[left, right], 3)
            .unwrap_err()
            .to_string()
            .contains("fewer than k=3"));
    }
}
