//! `ecaz corpus generate` — synthetic unit-sphere vector dataset → TSV.
//!
//! Writes rows compatible with `ecaz corpus load`: `<id>\t<json_array>` with
//! no header. Draws each row from a standard normal, then L2-normalizes so
//! inner product behaves like cosine similarity — matches the distribution
//! assumed by the existing brute-force ground-truth code.
//!
//! # Purity boundary
//!
//! `generate_unit_vector`, `format_tsv_row`, and the RNG seeding shape are
//! pure functions with unit tests (determinism, norm≈1, TSV shape).

use clap::{Args, ValueEnum};
use color_eyre::eyre::{eyre, Context, Result};
use rand::{rngs::StdRng, SeedableRng};
use rand_distr::{Distribution, StandardNormal};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct GenerateArgs {
    /// Output TSV path. Use `-` to write to stdout.
    #[arg(long)]
    pub output: PathBuf,
    /// Number of vectors to generate.
    #[arg(long)]
    pub n: usize,
    /// Vector dimension.
    #[arg(long, default_value_t = 1536)]
    pub dim: usize,
    /// RNG seed (fixed so repeated runs are reproducible).
    #[arg(long, default_value_t = 42)]
    pub seed: u64,
    /// Starting id. Loader expects unique bigint ids; default 0 matches the
    /// legacy `gen_synthetic_data.py`.
    #[arg(long, default_value_t = 0)]
    pub start_id: i64,
    /// Output format. `corpus` = id + embedding; `queries` = same shape,
    /// kept as a separate flag so a dataset pair shares a dim/seed lineage.
    #[arg(long, value_enum, default_value_t = GenerateKind::Corpus)]
    pub kind: GenerateKind,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenerateKind {
    Corpus,
    Queries,
}

pub async fn run(_database: &str, args: GenerateArgs) -> Result<()> {
    if args.n == 0 {
        return Err(eyre!("--n must be >= 1"));
    }
    if args.dim == 0 {
        return Err(eyre!("--dim must be >= 1"));
    }
    let write_stdout = args.output.as_os_str() == "-";
    let target_display = if write_stdout {
        "<stdout>".to_owned()
    } else {
        args.output.display().to_string()
    };

    let mut writer: Box<dyn Write> = if write_stdout {
        Box::new(BufWriter::new(std::io::stdout().lock()))
    } else {
        Box::new(BufWriter::new(
            File::create(&args.output)
                .wrap_err_with(|| format!("creating {}", args.output.display()))?,
        ))
    };

    let mut rng = StdRng::seed_from_u64(args.seed);
    for i in 0..args.n {
        let id = args.start_id + i as i64;
        let v = generate_unit_vector(&mut rng, args.dim);
        let line = format_tsv_row(id, &v);
        writer
            .write_all(line.as_bytes())
            .wrap_err("writing TSV row")?;
        writer.write_all(b"\n").wrap_err("writing newline")?;
    }
    writer.flush().wrap_err("flushing output")?;
    eprintln!(
        "[generate] wrote {} × dim {} rows to {} (kind={:?}, seed={})",
        args.n, args.dim, target_display, args.kind, args.seed
    );
    Ok(())
}

/// Draw a single dim-dimensional unit vector from the standard normal
/// (i.e. isotropic on the sphere once L2-normalized). The RNG is caller-
/// provided so a single seeded stream produces both corpus and query
/// files without cross-contamination.
pub fn generate_unit_vector<R: rand::Rng>(rng: &mut R, dim: usize) -> Vec<f32> {
    let mut v: Vec<f32> = (0..dim)
        .map(|_| <StandardNormal as Distribution<f32>>::sample(&StandardNormal, rng))
        .collect();
    let norm = (v.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt() as f32;
    let denom = norm.max(1e-10);
    for x in v.iter_mut() {
        *x /= denom;
    }
    v
}

/// Render one TSV row in loader-compatible form: `<id>\t<json_array>`.
/// JSON array is compact (no spaces) and uses `{:.6}` to match the legacy
/// Python output precision.
pub fn format_tsv_row(id: i64, values: &[f32]) -> String {
    let mut out = String::with_capacity(16 + values.len() * 10);
    out.push_str(&id.to_string());
    out.push('\t');
    out.push('[');
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("{v:.6}"));
    }
    out.push(']');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    // --- generate_unit_vector ---

    #[test]
    fn generated_vector_has_requested_dimension() {
        let mut rng = StdRng::seed_from_u64(7);
        let v = generate_unit_vector(&mut rng, 32);
        assert_eq!(v.len(), 32);
    }

    #[test]
    fn generated_vector_is_unit_norm() {
        let mut rng = StdRng::seed_from_u64(7);
        let v = generate_unit_vector(&mut rng, 256);
        let norm = (v.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
        // 1e-5 tolerance is fine: the f32 renormalization leaves a tiny residual.
        assert!((norm - 1.0).abs() < 1e-5, "norm={norm}");
    }

    #[test]
    fn generation_is_deterministic_for_a_given_seed() {
        let a = {
            let mut rng = StdRng::seed_from_u64(42);
            generate_unit_vector(&mut rng, 8)
        };
        let b = {
            let mut rng = StdRng::seed_from_u64(42);
            generate_unit_vector(&mut rng, 8)
        };
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_produce_different_vectors() {
        let mut r1 = StdRng::seed_from_u64(1);
        let mut r2 = StdRng::seed_from_u64(2);
        let a = generate_unit_vector(&mut r1, 16);
        let b = generate_unit_vector(&mut r2, 16);
        assert_ne!(a, b);
    }

    // --- format_tsv_row ---

    #[test]
    fn format_tsv_row_has_tab_between_id_and_json_array() {
        let line = format_tsv_row(5, &[0.0, 1.0]);
        assert_eq!(line, "5\t[0.000000,1.000000]");
    }

    #[test]
    fn format_tsv_row_uses_six_decimal_precision_matching_legacy_python() {
        let line = format_tsv_row(0, &[0.123_456_78, -0.987_654_3]);
        assert!(line.contains("0.123457"), "got {line}");
        assert!(line.contains("-0.987654"), "got {line}");
    }

    #[test]
    fn format_tsv_row_empty_vector_is_empty_json_array() {
        // Degenerate but well-defined: an empty embedding serializes to "[]".
        assert_eq!(format_tsv_row(42, &[]), "42\t[]");
    }

    #[test]
    fn format_tsv_row_is_parseable_as_json_after_splitting_on_tab() {
        let line = format_tsv_row(7, &[0.1_f32, 0.2, 0.3]);
        let (id_str, rest) = line.split_once('\t').unwrap();
        assert_eq!(id_str, "7");
        let parsed: Vec<f32> = serde_json::from_str(rest).unwrap();
        assert_eq!(parsed.len(), 3);
        assert!((parsed[0] - 0.1).abs() < 1e-6);
    }
}
