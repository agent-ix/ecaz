//! Re-score historical SPIRE result-identity JSONL artifacts with the
//! duplicate-tolerant and distinct-neighbor recall metrics (Task 138).
//!
//! Historical packets emit `spire_result_identity` records with the exact
//! per-query returned ids. This command recomputes ground truth by brute
//! force from the packet's corpus/queries TSVs and reports the current
//! (duplicate-tolerant) `recall@k` next to `distinct_recall@k`, without
//! editing any historical artifact.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre::{eyre, Result, WrapErr};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use ndarray::Array2;
use serde::Deserialize;

use super::recall::{
    brute_force_top_k, distinct_recall_summary_at_k, distinct_returned_count,
    load_sources_tsv_file, map_indices_to_ids, recall_summary_at_k,
};

#[derive(Args, Debug)]
pub struct RescoreIdentityArgs {
    /// SPIRE result-identity JSONL artifact(s) to re-score.
    #[arg(long, required = true, num_args = 1..)]
    pub identity_jsonl: Vec<PathBuf>,
    /// Corpus TSV (`<id>\t<json_array>`) the identity run queried.
    #[arg(long)]
    pub corpus_file: PathBuf,
    /// Queries TSV (`<id>\t<json_array>`) matching the identity run's query ids.
    #[arg(long)]
    pub queries_file: PathBuf,
    /// Override k; defaults to the k recorded in each identity record.
    #[arg(long)]
    pub k: Option<usize>,
    /// Optional path to also write the rendered table to.
    #[arg(long)]
    pub log_output: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct IdentityRecord {
    kind: String,
    nprobe: i32,
    #[allow(dead_code)]
    query_ordinal: usize,
    query_id: i64,
    k: usize,
    returned_ids: Vec<i64>,
}

fn read_identity_records(path: &PathBuf) -> Result<Vec<IdentityRecord>> {
    let file = File::open(path).wrap_err_with(|| format!("opening {}", path.display()))?;
    let mut records = Vec::new();
    for (idx, line) in BufReader::new(file).lines().enumerate() {
        let line = line.wrap_err_with(|| format!("{}:{}: read error", path.display(), idx + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        let record: IdentityRecord = serde_json::from_str(&line)
            .wrap_err_with(|| format!("{}:{}: invalid identity record", path.display(), idx + 1))?;
        if record.kind != "spire_result_identity" {
            return Err(eyre!(
                "{}:{}: unexpected record kind {:?}",
                path.display(),
                idx + 1,
                record.kind
            ));
        }
        records.push(record);
    }
    if records.is_empty() {
        return Err(eyre!("{}: no identity records found", path.display()));
    }
    Ok(records)
}

struct RescoredGroup {
    file: String,
    nprobe: i32,
    queries: usize,
    k: usize,
    recall: f64,
    distinct_recall: f64,
    distinct_returned_min: usize,
    distinct_returned_mean: f64,
    duplicate_query_count: usize,
}

fn rescore_group(
    file: &str,
    nprobe: i32,
    records: &[&IdentityRecord],
    k_override: Option<usize>,
    corpus_ids: &[i64],
    corpus: &Array2<f32>,
    query_rows_by_id: &HashMap<i64, usize>,
    queries: &Array2<f32>,
) -> Result<RescoredGroup> {
    let k = match k_override {
        Some(k) => k,
        None => {
            let k = records[0].k;
            if records.iter().any(|record| record.k != k) {
                return Err(eyre!(
                    "{file}: mixed k values in one nprobe group; pass --k to override"
                ));
            }
            k
        }
    };
    let mut selected = Vec::with_capacity(records.len() * queries.ncols());
    for record in records {
        let row = query_rows_by_id.get(&record.query_id).ok_or_else(|| {
            eyre!(
                "{file}: query_id {} not present in queries file",
                record.query_id
            )
        })?;
        selected.extend_from_slice(
            queries
                .row(*row)
                .as_slice()
                .ok_or_else(|| eyre!("queries matrix row {row} is not contiguous"))?,
        );
    }
    let selected = Array2::from_shape_vec((records.len(), queries.ncols()), selected)?;
    let truth = brute_force_top_k(corpus, &selected, k);
    let truth_ids = map_indices_to_ids(&truth.indices, corpus_ids);
    let pred: Vec<Vec<i64>> = records
        .iter()
        .map(|record| record.returned_ids.clone())
        .collect();
    let recall = recall_summary_at_k(&truth_ids, &pred, k);
    let distinct = distinct_recall_summary_at_k(&truth_ids, &pred, k);
    let counts: Vec<usize> = pred
        .iter()
        .map(|row| distinct_returned_count(row, k))
        .collect();
    let duplicate_query_count = pred
        .iter()
        .zip(&counts)
        .filter(|(row, count)| **count < row.len().min(k))
        .count();
    Ok(RescoredGroup {
        file: file.to_owned(),
        nprobe,
        queries: records.len(),
        k,
        recall: recall.recall,
        distinct_recall: distinct.recall,
        distinct_returned_min: counts.iter().copied().min().unwrap_or(0),
        distinct_returned_mean: counts.iter().sum::<usize>() as f64 / counts.len().max(1) as f64,
        duplicate_query_count,
    })
}

pub async fn run(args: RescoreIdentityArgs) -> Result<()> {
    let (corpus_ids, corpus) = load_sources_tsv_file(&args.corpus_file)?;
    if corpus_ids.is_empty() {
        return Err(eyre!("corpus file {} is empty", args.corpus_file.display()));
    }
    let (query_ids, queries) = load_sources_tsv_file(&args.queries_file)?;
    if query_ids.is_empty() {
        return Err(eyre!(
            "queries file {} is empty",
            args.queries_file.display()
        ));
    }
    let query_rows_by_id: HashMap<i64, usize> = query_ids
        .iter()
        .enumerate()
        .map(|(row, id)| (*id, row))
        .collect();

    let mut groups = Vec::new();
    for path in &args.identity_jsonl {
        let records = read_identity_records(path)?;
        let mut by_nprobe: HashMap<i32, Vec<&IdentityRecord>> = HashMap::new();
        for record in &records {
            by_nprobe.entry(record.nprobe).or_default().push(record);
        }
        let mut nprobes: Vec<i32> = by_nprobe.keys().copied().collect();
        nprobes.sort_unstable();
        let file = path.display().to_string();
        for nprobe in nprobes {
            groups.push(rescore_group(
                &file,
                nprobe,
                &by_nprobe[&nprobe],
                args.k,
                &corpus_ids,
                &corpus,
                &query_rows_by_id,
                &queries,
            )?);
        }
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "identity_file",
        "nprobe",
        "queries",
        "k",
        "recall@k",
        "distinct_recall@k",
        "distinct_returned_min",
        "distinct_returned_mean",
        "duplicate_query_count",
    ]);
    for group in &groups {
        table.add_row(vec![
            Cell::new(&group.file),
            Cell::new(group.nprobe),
            Cell::new(group.queries),
            Cell::new(group.k),
            Cell::new(format!("{:.4}", group.recall)),
            Cell::new(format!("{:.4}", group.distinct_recall)),
            Cell::new(group.distinct_returned_min),
            Cell::new(format!("{:.2}", group.distinct_returned_mean)),
            Cell::new(group.duplicate_query_count),
        ]);
    }
    let output = format!("Identity rescore\n{table}");
    println!("{output}");
    if let Some(path) = &args.log_output {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .wrap_err_with(|| format!("creating {}", parent.display()))?;
        }
        tokio::fs::write(path, format!("{output}\n"))
            .await
            .wrap_err_with(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(nprobe: i32, query_id: i64, returned_ids: Vec<i64>) -> IdentityRecord {
        IdentityRecord {
            kind: "spire_result_identity".to_owned(),
            nprobe,
            query_ordinal: 1,
            query_id,
            k: 3,
            returned_ids,
        }
    }

    #[test]
    fn rescore_group_counts_duplicates_once() {
        // Corpus of four unit-ish vectors; query aligned with rows 0..2.
        let corpus_ids = vec![10, 20, 30, 40];
        let corpus =
            Array2::from_shape_vec((4, 2), vec![1.0, 0.0, 0.9, 0.1, 0.8, 0.2, -1.0, 0.0]).unwrap();
        let queries = Array2::from_shape_vec((1, 2), vec![1.0, 0.0]).unwrap();
        let query_rows_by_id = HashMap::from([(7_i64, 0_usize)]);
        // Truth top-3 = {10, 20, 30}. Prediction repeats id 10 three times.
        let rec = record(8, 7, vec![10, 10, 10]);
        let group = rescore_group(
            "test.jsonl",
            8,
            &[&rec],
            None,
            &corpus_ids,
            &corpus,
            &query_rows_by_id,
            &queries,
        )
        .unwrap();
        // Duplicate-tolerant metric counts 3 hits; distinct counts 1.
        assert!((group.recall - 1.0).abs() < 1e-9);
        assert!((group.distinct_recall - (1.0 / 3.0)).abs() < 1e-9);
        assert_eq!(group.distinct_returned_min, 1);
        assert_eq!(group.duplicate_query_count, 1);
    }

    #[test]
    fn rescore_group_distinct_matches_current_when_no_duplicates() {
        let corpus_ids = vec![10, 20, 30, 40];
        let corpus =
            Array2::from_shape_vec((4, 2), vec![1.0, 0.0, 0.9, 0.1, 0.8, 0.2, -1.0, 0.0]).unwrap();
        let queries = Array2::from_shape_vec((1, 2), vec![1.0, 0.0]).unwrap();
        let query_rows_by_id = HashMap::from([(7_i64, 0_usize)]);
        let rec = record(8, 7, vec![10, 20, 30]);
        let group = rescore_group(
            "test.jsonl",
            8,
            &[&rec],
            None,
            &corpus_ids,
            &corpus,
            &query_rows_by_id,
            &queries,
        )
        .unwrap();
        assert!((group.recall - 1.0).abs() < 1e-9);
        assert!((group.distinct_recall - 1.0).abs() < 1e-9);
        assert_eq!(group.distinct_returned_min, 3);
        assert_eq!(group.duplicate_query_count, 0);
    }
}
