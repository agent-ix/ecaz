use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use std::{
    hint::black_box,
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};

use ecaz::bench_api::{simd_backend, ProdQuantizer, Quantizer, RaBitQQuantizer};

#[derive(Args, Debug)]
pub struct RabitqKernelArgs {
    /// Vector dimensionality for generated query/candidate fixtures.
    #[arg(long, default_value = "1536")]
    pub dim: usize,

    /// Number of encoded candidates in the reusable fixture slab.
    #[arg(long, default_value = "1000")]
    pub candidates: usize,

    /// Timed repetitions for each benchmark row.
    #[arg(long, default_value = "1000")]
    pub iterations: usize,

    /// Force the SIMD backend for this benchmark process, for example scalar,
    /// auto, avx2, or avx512.
    #[arg(long)]
    pub simd_mode: Option<String>,

    /// Write the same text output to a packet-local artifact.
    #[arg(long)]
    pub log_output: Option<PathBuf>,
}

pub async fn run(args: RabitqKernelArgs) -> Result<()> {
    if let Some(mode) = args.simd_mode.as_deref() {
        if mode == "auto" {
            std::env::remove_var("ECAZ_SIMD");
        } else {
            std::env::set_var("ECAZ_SIMD", mode);
        }
    }

    let mut output = Output::new(args.log_output.as_deref())?;
    let dim = args.dim;
    let candidates = args.candidates.max(1);
    let iterations = args.iterations.max(1);

    output.line(&format!("backend={}", simd_backend()))?;
    output.line(&format!("dim={dim}"))?;
    output.line(&format!("candidates={candidates}"))?;
    output.line(&format!("iterations={iterations}"))?;
    output
        .line("variant\tmode\tbits\tclip\titerations\tscores\ttotal_ns\tns_per_score\tchecksum")?;

    let prod = ProdQuantizer::cached(dim, 4, 42);
    for variant in [
        Variant::new("bits1", 1, 2.0),
        Variant::new("bits4", 4, 2.0),
        Variant::new("bits8", 8, 2.0),
        Variant::new("bits8c3", 8, 3.0),
        Variant::new("bits8c4", 8, 4.0),
    ] {
        let quantizer =
            RaBitQQuantizer::with_srht_bits_clip(dim, prod.clone(), variant.bits, variant.clip)
                .map_err(|err| eyre!("construct RaBitQ quantizer for {}: {err}", variant.name))?;
        let query = random_unit_vector(dim, 1);
        let prepared = quantizer.prepare_estimator(&query);
        let codes = (0..candidates)
            .map(|i| {
                <RaBitQQuantizer as Quantizer>::encode_code(
                    &quantizer,
                    &random_unit_vector(dim, i + 100),
                )
                .into_vec()
            })
            .collect::<Vec<_>>();

        bench_single_dispatch(&mut output, variant, &prepared, &codes, iterations)?;
        bench_single_scalar(&mut output, variant, &prepared, &codes, iterations)?;

        if matches!(variant.bits, 1 | 4 | 8) {
            let code_len = <RaBitQQuantizer as Quantizer>::code_len(&quantizer);
            let slab = codes
                .iter()
                .flat_map(|code| code.iter().copied())
                .collect::<Vec<_>>();
            bench_batch(
                &mut output,
                variant,
                &prepared,
                &slab,
                code_len,
                candidates,
                iterations,
            )?;
        }
    }

    Ok(())
}

#[derive(Clone, Copy)]
struct Variant {
    name: &'static str,
    bits: u8,
    clip: f32,
}

impl Variant {
    const fn new(name: &'static str, bits: u8, clip: f32) -> Self {
        Self { name, bits, clip }
    }
}

fn bench_single_dispatch(
    output: &mut Output,
    variant: Variant,
    prepared: &ecaz::bench_api::PreparedEstimator,
    codes: &[Vec<u8>],
    iterations: usize,
) -> Result<()> {
    let warmup = iterations.clamp(1, 64);
    let mut checksum = 0.0_f32;
    for i in 0..warmup {
        checksum += prepared
            .estimate_ip(black_box(&codes[i % codes.len()]))
            .estimate;
    }
    black_box(checksum);

    checksum = 0.0;
    let mut idx = 0_usize;
    let elapsed = time_loop(iterations, || {
        let code = &codes[idx % codes.len()];
        idx += 1;
        let score = prepared.estimate_ip(black_box(code)).estimate;
        checksum += black_box(score);
    });
    print_row(
        output,
        variant,
        "single-dispatch",
        iterations,
        iterations,
        elapsed,
        checksum,
    )
}

fn bench_single_scalar(
    output: &mut Output,
    variant: Variant,
    prepared: &ecaz::bench_api::PreparedEstimator,
    codes: &[Vec<u8>],
    iterations: usize,
) -> Result<()> {
    let warmup = iterations.clamp(1, 64);
    let mut checksum = 0.0_f32;
    for i in 0..warmup {
        checksum += prepared.estimate_ip_scalar_only(black_box(&codes[i % codes.len()]));
    }
    black_box(checksum);

    checksum = 0.0;
    let mut idx = 0_usize;
    let elapsed = time_loop(iterations, || {
        let code = &codes[idx % codes.len()];
        idx += 1;
        let score = prepared.estimate_ip_scalar_only(black_box(code));
        checksum += black_box(score);
    });
    print_row(
        output,
        variant,
        "single-scalar",
        iterations,
        iterations,
        elapsed,
        checksum,
    )
}

fn bench_batch(
    output: &mut Output,
    variant: Variant,
    prepared: &ecaz::bench_api::PreparedEstimator,
    slab: &[u8],
    code_len: usize,
    candidates: usize,
    iterations: usize,
) -> Result<()> {
    let mut scores = Vec::new();
    let warmup = iterations.clamp(1, 16);
    for _ in 0..warmup {
        prepared
            .estimate_ip_batch(black_box(slab), code_len, &mut scores)
            .expect("batch fixture uses matching code length");
        black_box(scores.first().copied().unwrap_or_default());
    }

    let mut checksum = 0.0_f32;
    let elapsed = time_loop(iterations, || {
        prepared
            .estimate_ip_batch(black_box(slab), code_len, &mut scores)
            .expect("batch fixture uses matching code length");
        checksum += black_box(scores.first().copied().unwrap_or_default());
    });
    print_row(
        output,
        variant,
        "batch",
        iterations,
        iterations * candidates,
        elapsed,
        checksum,
    )
}

fn time_loop(iterations: usize, mut f: impl FnMut()) -> Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    start.elapsed()
}

fn print_row(
    output: &mut Output,
    variant: Variant,
    mode: &str,
    iterations: usize,
    scores: usize,
    elapsed: Duration,
    checksum: f32,
) -> Result<()> {
    let total_ns = elapsed.as_nanos();
    let ns_per_score = total_ns as f64 / scores.max(1) as f64;
    output.line(&format!(
        "{}\t{}\t{}\t{:.1}\t{}\t{}\t{}\t{:.2}\t{:.6}",
        variant.name,
        mode,
        variant.bits,
        variant.clip,
        iterations,
        scores,
        total_ns,
        ns_per_score,
        checksum
    ))
}

fn random_unit_vector(dim: usize, seed: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(dim);
    let mut norm2 = 0.0_f32;
    for i in 0..dim {
        let x = ((i + 1) as f32 * 0.017 + seed as f32 * 0.113).sin()
            + ((i + seed + 3) as f32 * 0.031).cos() * 0.5;
        out.push(x);
        norm2 += x * x;
    }
    let inv_norm = norm2.sqrt().recip();
    for value in &mut out {
        *value *= inv_norm;
    }
    out
}

struct Output {
    log: Option<std::fs::File>,
}

impl Output {
    fn new(log_output: Option<&std::path::Path>) -> Result<Self> {
        let log = if let Some(path) = log_output {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            Some(
                std::fs::File::create(path)
                    .with_context(|| format!("create {}", path.display()))?,
            )
        } else {
            None
        };
        Ok(Self { log })
    }

    fn line(&mut self, value: &str) -> Result<()> {
        println!("{value}");
        if let Some(log) = &mut self.log {
            writeln!(log, "{value}").context("write rabitq kernel log")?;
        }
        Ok(())
    }
}
