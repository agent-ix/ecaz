use std::hint::black_box;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use ecaz::bench_api::{
    fwht_in_place, orthonormal_fwht_in_place, pad_input, sign_vector, simd_backend, srht,
    ProdQuantizer,
};

const WARMUP_ITERATIONS: usize = 256;

fn main() {
    let options = BenchOptions::parse();
    let mut output = BenchOutput::new(options.log_output);
    let iterations = options.iterations;

    output.line(&format!("backend={}", simd_backend()));
    output.line(&format!("iterations={iterations}"));
    output.line(&format!(
        "warmup_iterations={}",
        iterations.clamp(1, WARMUP_ITERATIONS)
    ));

    run_fwht_bench(&mut output, 1_024, iterations);
    run_fwht_bench(&mut output, 2_048, iterations);
    run_fwht_bench(&mut output, 4_096, iterations / 2);
    run_orthonormal_fwht_bench(&mut output, 1_024, iterations);
    run_orthonormal_fwht_bench(&mut output, 2_048, iterations);
    run_orthonormal_fwht_bench(&mut output, 4_096, iterations / 2);
    run_srht_bench(&mut output, 1_024, 1_024, iterations);
    run_srht_bench(&mut output, 1_536, 2_048, iterations);
    run_srht_bench(&mut output, 2_048, 2_048, iterations);
    run_f32_inner_product_bench(&mut output, 1_536, iterations);
    run_prepare_ip_query_bench(&mut output, 1_024, 4, iterations / 4);
    run_prepare_ip_query_bench(&mut output, 1_536, 4, iterations / 4);
    run_prepare_ip_query_bench(&mut output, 2_048, 4, iterations / 4);
    run_score_ip_encoded_bench(&mut output, 1_536, 4, iterations);
    run_score_ip_lut_no_qjl_4bit_bench(&mut output, 1_536, iterations);
    run_score_ip_codes_lite_bench(&mut output, 1_536, 4, iterations);
}

#[derive(Debug)]
struct BenchOptions {
    iterations: usize,
    log_output: Option<PathBuf>,
}

impl BenchOptions {
    fn parse() -> Self {
        let mut iterations = 1_000;
        let mut log_output = None;
        let mut args = std::env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--iterations" => {
                    let value = args.next().expect("--iterations requires a value");
                    iterations = value
                        .parse::<usize>()
                        .expect("--iterations must be a positive integer");
                }
                "--log-output" => {
                    let value = args.next().expect("--log-output requires a path");
                    log_output = Some(PathBuf::from(value));
                }
                "-h" | "--help" => {
                    println!("Usage: simd_bench [--iterations N] [--log-output PATH]");
                    std::process::exit(0);
                }
                value if !value.starts_with('-') => {
                    iterations = value
                        .parse::<usize>()
                        .expect("positional iteration count must be a positive integer");
                }
                other => panic!("unknown argument: {other}"),
            }
        }

        Self {
            iterations,
            log_output,
        }
    }
}

struct BenchOutput {
    log: Option<std::fs::File>,
}

impl BenchOutput {
    fn new(log_output: Option<PathBuf>) -> Self {
        let log = log_output.map(|path| {
            std::fs::File::create(&path)
                .unwrap_or_else(|e| panic!("failed to create {}: {e}", path.display()))
        });
        Self { log }
    }

    fn line(&mut self, value: &str) {
        println!("{value}");
        if let Some(log) = &mut self.log {
            writeln!(log, "{value}").expect("failed to write simd_bench log");
        }
    }
}

fn run_fwht_bench(output: &mut BenchOutput, size: usize, iterations: usize) {
    let template: Vec<f32> = (0..size).map(|i| (i as f32) * 0.001).collect();
    let mut data = template.clone();
    let elapsed = time_loop(iterations.max(1), || {
        data.copy_from_slice(&template);
        fwht_in_place(black_box(&mut data));
        black_box(data[0]);
    });
    print_result(output, &format!("fwht/{size}"), iterations.max(1), elapsed);
}

fn run_orthonormal_fwht_bench(output: &mut BenchOutput, size: usize, iterations: usize) {
    let template: Vec<f32> = (0..size).map(|i| (i as f32) * 0.001).collect();
    let mut data = template.clone();
    let elapsed = time_loop(iterations.max(1), || {
        data.copy_from_slice(&template);
        orthonormal_fwht_in_place(black_box(&mut data));
        black_box(data[0]);
    });
    print_result(
        output,
        &format!("orthonormal_fwht/{size}"),
        iterations.max(1),
        elapsed,
    );
}

fn run_srht_bench(output: &mut BenchOutput, dim: usize, transform_dim: usize, iterations: usize) {
    let input: Vec<f32> = (0..dim).map(|i| (i as f32) * 0.001).collect();
    let padded = pad_input(&input, transform_dim);
    let signs = sign_vector(transform_dim, 42);
    let elapsed = time_loop(iterations.max(1), || {
        let rotated = srht(black_box(&padded), black_box(&signs));
        black_box(rotated[0]);
    });
    print_result(
        output,
        &format!("srht/d{dim}_td{transform_dim}"),
        iterations.max(1),
        elapsed,
    );
}

fn run_f32_inner_product_bench(output: &mut BenchOutput, dim: usize, iterations: usize) {
    let left = random_unit_vector(dim, 11);
    let right = random_unit_vector(dim, 12);
    let elapsed = time_loop(iterations.max(1), || {
        let score = inner_product(black_box(&left), black_box(&right));
        black_box(score);
    });
    print_result(
        output,
        &format!("f32_inner_product/d{dim}"),
        iterations.max(1),
        elapsed,
    );
}

fn run_prepare_ip_query_bench(output: &mut BenchOutput, dim: usize, bits: u8, iterations: usize) {
    let quantizer = ProdQuantizer::new(dim, bits, 42);
    let query = random_unit_vector(dim, 7);
    let elapsed = time_loop(iterations.max(1), || {
        let prepared = quantizer.prepare_ip_query(black_box(&query));
        black_box(prepared.rotated[0]);
    });
    print_result(
        output,
        &format!("prepare_ip_query/d{dim}_b{bits}"),
        iterations.max(1),
        elapsed,
    );
}

fn run_score_ip_encoded_bench(output: &mut BenchOutput, dim: usize, bits: u8, iterations: usize) {
    let quantizer = ProdQuantizer::new(dim, bits, 42);
    let prepared = quantizer.prepare_ip_query(&random_unit_vector(dim, 1));
    let payloads: Vec<Vec<u8>> = (0..256)
        .map(|i| quantizer.pack_payload(&quantizer.encode(&random_unit_vector(dim, i + 100))))
        .collect();

    let mut index = 0usize;
    let elapsed = time_loop(iterations.max(1), || {
        let score = quantizer.score_ip_encoded(&prepared, &payloads[index % payloads.len()]);
        index += 1;
        black_box(score);
    });
    print_result(
        output,
        &format!("score_ip_encoded/d{dim}_b{bits}"),
        iterations.max(1),
        elapsed,
    );
}

fn run_score_ip_lut_no_qjl_4bit_bench(output: &mut BenchOutput, dim: usize, iterations: usize) {
    let quantizer = ProdQuantizer::new(dim, 4, 42);
    let prepared = quantizer.prepare_ip_query_lut_no_qjl_4bit(&random_unit_vector(dim, 1));
    let payloads: Vec<Vec<u8>> = (0..256)
        .map(|i| {
            quantizer
                .encode(&random_unit_vector(dim, i + 700))
                .mse_packed
        })
        .collect();

    let mut index = 0usize;
    let elapsed = time_loop(iterations.max(1), || {
        let score = quantizer
            .score_ip_from_parts_lut_no_qjl_4bit(&prepared, &payloads[index % payloads.len()]);
        index += 1;
        black_box(score);
    });
    print_result(
        output,
        &format!("score_ip_lut_no_qjl_4bit/d{dim}"),
        iterations.max(1),
        elapsed,
    );
}

fn run_score_ip_codes_lite_bench(
    output: &mut BenchOutput,
    dim: usize,
    bits: u8,
    iterations: usize,
) {
    let quantizer = ProdQuantizer::new(dim, bits, 42);
    let codes: Vec<Vec<u8>> = (0..256)
        .map(|i| {
            let encoded = quantizer.encode(&random_unit_vector(dim, i + 400));
            let mut code = encoded.mse_packed;
            code.extend_from_slice(&encoded.qjl_packed);
            code
        })
        .collect();

    let mut index = 0usize;
    let elapsed = time_loop(iterations.max(1), || {
        let a = &codes[index % codes.len()];
        let b = &codes[(index + 1) % codes.len()];
        let score = quantizer.score_ip_codes_lite(a, b);
        index += 1;
        black_box(score);
    });
    print_result(
        output,
        &format!("score_ip_codes_lite/d{dim}_b{bits}"),
        iterations.max(1),
        elapsed,
    );
}

fn time_loop(iterations: usize, mut f: impl FnMut()) -> Duration {
    for _ in 0..iterations.min(WARMUP_ITERATIONS) {
        f();
    }

    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    start.elapsed()
}

fn print_result(output: &mut BenchOutput, name: &str, iterations: usize, elapsed: Duration) {
    let ns_per_iter = elapsed.as_secs_f64() * 1e9 / iterations as f64;
    output.line(&format!(
        "{name}: total={elapsed:?} ns_per_iter={ns_per_iter:.1}"
    ));
}

fn random_unit_vector(dim: usize, seed: usize) -> Vec<f32> {
    use rand::Rng;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let mut rng = ChaCha8Rng::seed_from_u64(seed as u64);
    let mut values: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let norm = values.iter().map(|v| v * v).sum::<f32>().sqrt();
    for value in &mut values {
        *value /= norm.max(f32::EPSILON);
    }
    values
}

fn inner_product(left: &[f32], right: &[f32]) -> f32 {
    left.iter().zip(right).map(|(l, r)| l * r).sum()
}
