//! `ecaz bench tq-bias-audit` -- offline TurboQuant estimator-bias audit.
//!
//! This is a measurement harness for Task 148. It reads staged TSV corpus/query
//! files directly and does not touch SQL, product scoring paths, or on-disk
//! index data.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use serde::Serialize;

use ecaz::bench_api::ProdQuantizer;

#[derive(Args, Debug)]
pub struct TqBiasAuditArgs {
    /// Staged corpus TSV: `<id>\t<json float array>`.
    #[arg(long)]
    pub corpus: PathBuf,

    /// Staged query TSV used for deterministic paired-query score ratios.
    #[arg(long)]
    pub queries: PathBuf,

    /// Human-readable label for this corpus scale.
    #[arg(long)]
    pub label: String,

    /// Stop after this many corpus rows. Defaults to all rows.
    #[arg(long)]
    pub max_rows: Option<usize>,

    /// Only include paired query score ratios with `abs(<q,x>) >= threshold`
    /// in the filtered query-dot distribution.
    #[arg(long, default_value_t = 0.02)]
    pub min_abs_query_dot: f64,

    /// Write JSON summary to this packet-local path.
    #[arg(long)]
    pub json_output: Option<PathBuf>,

    /// Write the text table to this packet-local path.
    #[arg(long)]
    pub log_output: Option<PathBuf>,
}

pub async fn run(args: TqBiasAuditArgs) -> Result<()> {
    let started = Instant::now();
    let queries = load_vectors_tsv(&args.queries)
        .wrap_err_with(|| format!("loading query TSV {}", args.queries.display()))?;
    if queries.rows.is_empty() {
        return Err(eyre!("{} contains no query rows", args.queries.display()));
    }

    let mut audit = Audit::new(&args.label, args.min_abs_query_dot);
    let mut quantizer: Option<ProdQuantizer> = None;
    let file = File::open(&args.corpus)
        .wrap_err_with(|| format!("opening corpus TSV {}", args.corpus.display()))?;
    let reader = BufReader::new(file);

    for (idx, line) in reader.lines().enumerate() {
        if let Some(max_rows) = args.max_rows {
            if audit.rows >= max_rows {
                break;
            }
        }
        let line_number = idx + 1;
        let raw =
            line.wrap_err_with(|| format!("{}:{line_number}: read error", args.corpus.display()))?;
        let Some((_id, vector)) = parse_tsv_vector_line(&args.corpus, line_number, &raw)? else {
            continue;
        };
        if vector.len() != queries.dim {
            return Err(eyre!(
                "{}:{line_number}: corpus dim {} does not match query dim {}",
                args.corpus.display(),
                vector.len(),
                queries.dim
            ));
        }
        let quantizer_ref =
            quantizer.get_or_insert_with(|| ProdQuantizer::new(vector.len(), 4, 42));
        let query = &queries.rows[audit.rows % queries.rows.len()];
        audit.observe(quantizer_ref, &vector, query);
    }

    if audit.rows == 0 {
        return Err(eyre!("{} contains no corpus rows", args.corpus.display()));
    }

    let summary = audit.finish(
        args.corpus,
        args.queries,
        queries.rows.len(),
        started.elapsed(),
    );
    let text = render_summary(&summary);
    println!("{text}");

    if let Some(path) = args.log_output.as_ref() {
        write_text(path, &text).await?;
    }
    if let Some(path) = args.json_output.as_ref() {
        write_json(path, &summary).await?;
    }
    Ok(())
}

#[derive(Debug)]
struct LoadedVectors {
    dim: usize,
    rows: Vec<Vec<f32>>,
}

fn load_vectors_tsv(path: &Path) -> Result<LoadedVectors> {
    let file = File::open(path).wrap_err_with(|| format!("opening {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();
    let mut dim = None;
    for (idx, line) in reader.lines().enumerate() {
        let line_number = idx + 1;
        let raw = line.wrap_err_with(|| format!("{}:{line_number}: read error", path.display()))?;
        let Some((_id, vector)) = parse_tsv_vector_line(path, line_number, &raw)? else {
            continue;
        };
        match dim {
            Some(expected) if vector.len() != expected => {
                return Err(eyre!(
                    "{}:{line_number}: expected dim {}, got {}",
                    path.display(),
                    expected,
                    vector.len()
                ));
            }
            None => dim = Some(vector.len()),
            _ => {}
        }
        rows.push(vector);
    }
    Ok(LoadedVectors {
        dim: dim.unwrap_or(0),
        rows,
    })
}

fn parse_tsv_vector_line(
    path: &Path,
    line_number: usize,
    raw: &str,
) -> Result<Option<(i64, Vec<f32>)>> {
    let trimmed = raw.trim_end_matches(['\r', '\n']);
    if trimmed.is_empty() {
        return Ok(None);
    }
    let (id_str, json_str) = trimmed.split_once('\t').ok_or_else(|| {
        eyre!(
            "{}:{line_number}: expected '<id>\\t<json_array>' line",
            path.display()
        )
    })?;
    let id = id_str.parse::<i64>().map_err(|_| {
        eyre!(
            "{}:{line_number}: id {:?} is not an integer",
            path.display(),
            id_str
        )
    })?;
    let vector = serde_json::from_str::<Vec<f32>>(json_str).map_err(|err| {
        eyre!(
            "{}:{line_number}: embedding column is not valid JSON: {err}",
            path.display()
        )
    })?;
    Ok(Some((id, vector)))
}

#[derive(Debug)]
struct Audit {
    label: String,
    min_abs_query_dot: f64,
    rows: usize,
    norm_ratio: Vec<f64>,
    renorm_factor: Vec<f64>,
    self_dot_ratio: Vec<f64>,
    gamma_ratio: Vec<f64>,
    query_dot_ratio_all: Vec<f64>,
    query_dot_ratio_filtered: Vec<f64>,
    skipped_query_zero_denominator: usize,
    skipped_query_filtered_denominator: usize,
}

impl Audit {
    fn new(label: &str, min_abs_query_dot: f64) -> Self {
        Self {
            label: label.to_owned(),
            min_abs_query_dot,
            rows: 0,
            norm_ratio: Vec::new(),
            renorm_factor: Vec::new(),
            self_dot_ratio: Vec::new(),
            gamma_ratio: Vec::new(),
            query_dot_ratio_all: Vec::new(),
            query_dot_ratio_filtered: Vec::new(),
            skipped_query_zero_denominator: 0,
            skipped_query_filtered_denominator: 0,
        }
    }

    fn observe(&mut self, quantizer: &ProdQuantizer, source: &[f32], query: &[f32]) {
        let encoded = quantizer.encode(source);
        let decoded = quantizer.decode_approximate_from_code(&encoded.mse_packed);

        let source_norm2 = dot(source, source);
        let decoded_norm2 = dot(&decoded, &decoded);
        let source_norm = source_norm2.sqrt();
        let decoded_norm = decoded_norm2.sqrt();
        if source_norm > 0.0 {
            self.norm_ratio.push(decoded_norm / source_norm);
            self.renorm_factor
                .push(source_norm / decoded_norm.max(f64::MIN_POSITIVE));
            self.gamma_ratio.push(encoded.gamma as f64 / source_norm);
        }
        if source_norm2.abs() > f64::MIN_POSITIVE {
            self.self_dot_ratio
                .push(dot(source, &decoded) / source_norm2);
        }

        let exact_query_dot = dot(query, source);
        let decoded_query_dot = dot(query, &decoded);
        if exact_query_dot.abs() <= f64::MIN_POSITIVE {
            self.skipped_query_zero_denominator += 1;
        } else {
            let ratio = decoded_query_dot / exact_query_dot;
            if ratio.is_finite() {
                self.query_dot_ratio_all.push(ratio);
            }
        }
        if exact_query_dot.abs() < self.min_abs_query_dot {
            self.skipped_query_filtered_denominator += 1;
        } else {
            let ratio = decoded_query_dot / exact_query_dot;
            if ratio.is_finite() {
                self.query_dot_ratio_filtered.push(ratio);
            }
        }
        self.rows += 1;
    }

    fn finish(
        mut self,
        corpus: PathBuf,
        queries: PathBuf,
        query_rows: usize,
        elapsed: std::time::Duration,
    ) -> BiasAuditSummary {
        BiasAuditSummary {
            label: self.label,
            corpus,
            queries,
            corpus_rows: self.rows,
            query_rows,
            bits: 4,
            seed: 42,
            pairing: "corpus row ordinal paired with query ordinal modulo query row count"
                .to_owned(),
            min_abs_query_dot: self.min_abs_query_dot,
            elapsed_seconds: elapsed.as_secs_f64(),
            skipped_query_zero_denominator: self.skipped_query_zero_denominator,
            skipped_query_filtered_denominator: self.skipped_query_filtered_denominator,
            norm_ratio: Distribution::from_values(&mut self.norm_ratio),
            renorm_factor: Distribution::from_values(&mut self.renorm_factor),
            self_dot_ratio: Distribution::from_values(&mut self.self_dot_ratio),
            gamma_ratio: Distribution::from_values(&mut self.gamma_ratio),
            query_dot_ratio_all: Distribution::from_values(&mut self.query_dot_ratio_all),
            query_dot_ratio_filtered: Distribution::from_values(&mut self.query_dot_ratio_filtered),
        }
    }
}

fn dot(lhs: &[f32], rhs: &[f32]) -> f64 {
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| f64::from(*l) * f64::from(*r))
        .sum()
}

#[derive(Debug, Serialize)]
struct BiasAuditSummary {
    label: String,
    corpus: PathBuf,
    queries: PathBuf,
    corpus_rows: usize,
    query_rows: usize,
    bits: u8,
    seed: u64,
    pairing: String,
    min_abs_query_dot: f64,
    elapsed_seconds: f64,
    skipped_query_zero_denominator: usize,
    skipped_query_filtered_denominator: usize,
    norm_ratio: Distribution,
    renorm_factor: Distribution,
    self_dot_ratio: Distribution,
    gamma_ratio: Distribution,
    query_dot_ratio_all: Distribution,
    query_dot_ratio_filtered: Distribution,
}

#[derive(Debug, Default, Serialize)]
struct Distribution {
    count: usize,
    mean: f64,
    stddev: f64,
    min: f64,
    p01: f64,
    p05: f64,
    p10: f64,
    p50: f64,
    p90: f64,
    p95: f64,
    p99: f64,
    max: f64,
}

impl Distribution {
    fn from_values(values: &mut [f64]) -> Self {
        if values.is_empty() {
            return Self::default();
        }
        values.sort_by(|a, b| a.total_cmp(b));
        let count = values.len();
        let mean = values.iter().sum::<f64>() / count as f64;
        let variance = values
            .iter()
            .map(|value| {
                let delta = value - mean;
                delta * delta
            })
            .sum::<f64>()
            / count as f64;
        Self {
            count,
            mean,
            stddev: variance.sqrt(),
            min: values[0],
            p01: percentile(values, 0.01),
            p05: percentile(values, 0.05),
            p10: percentile(values, 0.10),
            p50: percentile(values, 0.50),
            p90: percentile(values, 0.90),
            p95: percentile(values, 0.95),
            p99: percentile(values, 0.99),
            max: values[count - 1],
        }
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    debug_assert!(!sorted.is_empty());
    if sorted.len() == 1 {
        return sorted[0];
    }
    let p = p.clamp(0.0, 1.0);
    let rank = p * (sorted.len() - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let weight = rank - lo as f64;
        sorted[lo] * (1.0 - weight) + sorted[hi] * weight
    }
}

fn render_summary(summary: &BiasAuditSummary) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "metric", "count", "mean", "stddev", "min", "p01", "p05", "p10", "p50", "p90", "p95",
        "p99", "max",
    ]);
    for (name, distribution) in [
        ("norm_ratio", &summary.norm_ratio),
        ("renorm_factor", &summary.renorm_factor),
        ("self_dot_ratio", &summary.self_dot_ratio),
        ("gamma_ratio", &summary.gamma_ratio),
        ("query_dot_ratio_all", &summary.query_dot_ratio_all),
        (
            "query_dot_ratio_filtered",
            &summary.query_dot_ratio_filtered,
        ),
    ] {
        table.add_row(vec![
            Cell::new(name),
            Cell::new(distribution.count),
            Cell::new(format!("{:.8}", distribution.mean)),
            Cell::new(format!("{:.8}", distribution.stddev)),
            Cell::new(format!("{:.8}", distribution.min)),
            Cell::new(format!("{:.8}", distribution.p01)),
            Cell::new(format!("{:.8}", distribution.p05)),
            Cell::new(format!("{:.8}", distribution.p10)),
            Cell::new(format!("{:.8}", distribution.p50)),
            Cell::new(format!("{:.8}", distribution.p90)),
            Cell::new(format!("{:.8}", distribution.p95)),
            Cell::new(format!("{:.8}", distribution.p99)),
            Cell::new(format!("{:.8}", distribution.max)),
        ]);
    }

    format!(
        "TurboQuant 4-bit no-QJL bias audit\nlabel: {}\ncorpus: {}\nqueries: {}\ncorpus_rows: {}\nquery_rows: {}\npairing: {}\nmin_abs_query_dot: {:.8}\nskipped_query_zero_denominator: {}\nskipped_query_filtered_denominator: {}\nelapsed_seconds: {:.3}\n\n{}",
        summary.label,
        summary.corpus.display(),
        summary.queries.display(),
        summary.corpus_rows,
        summary.query_rows,
        summary.pairing,
        summary.min_abs_query_dot,
        summary.skipped_query_zero_denominator,
        summary.skipped_query_filtered_denominator,
        summary.elapsed_seconds,
        table
    )
}

async fn write_text(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .wrap_err_with(|| format!("creating {}", parent.display()))?;
    }
    tokio::fs::write(path, format!("{text}\n"))
        .await
        .wrap_err_with(|| format!("writing {}", path.display()))
}

async fn write_json(path: &Path, summary: &BiasAuditSummary) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .wrap_err_with(|| format!("creating {}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(summary).wrap_err("serializing bias audit summary")?;
    tokio::fs::write(path, bytes)
        .await
        .wrap_err_with(|| format!("writing {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distribution_percentiles_are_interpolated() {
        let mut values = vec![1.0, 2.0, 3.0, 4.0];
        let summary = Distribution::from_values(&mut values);
        assert_eq!(summary.count, 4);
        assert!((summary.mean - 2.5).abs() < 1e-12);
        assert!((summary.p50 - 2.5).abs() < 1e-12);
        assert!((summary.p95 - 3.85).abs() < 1e-12);
    }
}
