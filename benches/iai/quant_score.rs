//! Instruction-count benchmarks for scoring hot loop (iai-callgrind).

#[path = "../helpers.rs"]
mod helpers;

use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;
use ecaz::bench_api::ProdQuantizer;

#[library_benchmark]
fn score_ip_encoded_1536_4() -> f32 {
    let q = ProdQuantizer::new(1536, 4, 42);
    let prepared = q.prepare_ip_query(&helpers::random_unit_vector(1536, 1));
    let payload = q.pack_payload(&q.encode(&helpers::random_unit_vector(1536, 100)));
    black_box(q.score_ip_encoded(&prepared, &payload))
}

#[library_benchmark]
fn score_ip_codes_lite_1536_4() -> f32 {
    let q = ProdQuantizer::new(1536, 4, 42);
    let enc_a = q.encode(&helpers::random_unit_vector(1536, 100));
    let enc_b = q.encode(&helpers::random_unit_vector(1536, 101));
    let mut code_a = enc_a.mse_packed;
    code_a.extend_from_slice(&enc_a.qjl_packed);
    let mut code_b = enc_b.mse_packed;
    code_b.extend_from_slice(&enc_b.qjl_packed);
    black_box(q.score_ip_codes_lite(&code_a, &code_b))
}

#[library_benchmark]
fn score_ip_from_parts_1536_4() -> f32 {
    let q = ProdQuantizer::new(1536, 4, 42);
    let prepared = q.prepare_ip_query(&helpers::random_unit_vector(1536, 1));
    let enc = q.encode(&helpers::random_unit_vector(1536, 100));
    let mut code = enc.mse_packed;
    code.extend_from_slice(&enc.qjl_packed);
    black_box(q.score_ip_from_parts(&prepared, enc.gamma, &code))
}

library_benchmark_group!(
    name = quant_score;
    benchmarks =
        score_ip_encoded_1536_4,
        score_ip_codes_lite_1536_4,
        score_ip_from_parts_1536_4
);
main!(library_benchmark_groups = quant_score);
