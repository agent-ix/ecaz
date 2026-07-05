//! Microbenchmarks for the scoring hot loop — the innermost path called per candidate.
#![allow(clippy::single_element_loop)]

#[path = "../helpers.rs"]
mod helpers;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ecaz::bench_api::{
    qjl32_score_block32, qjl32_score_scalar, rabitq32_multibit_score_block32, ProdQuantizer,
    Quantizer, RaBitQQuantizer, QJL32_BLOCK_WIDTH, RABITQ32_BLOCK_WIDTH,
};

fn bench_score_ip_encoded(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_ip_encoded");
    for &(dim, bits) in &[
        (256, 4u8),
        (768, 4),
        (1536, 3),
        (1536, 4),
        (1536, 6),
        (3072, 4),
    ] {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let prepared = quantizer.prepare_ip_query(&helpers::random_unit_vector(dim, 1));
        let payloads: Vec<Vec<u8>> = (0..1000)
            .map(|i| {
                quantizer
                    .pack_payload(&quantizer.encode(&helpers::random_unit_vector(dim, i + 100)))
            })
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let score = quantizer.score_ip_encoded(&prepared, &payloads[idx % 1000]);
                idx += 1;
                score
            });
        });
    }
    group.finish();
}

fn bench_score_ip_codes_lite(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_ip_codes_lite");
    for &(dim, bits) in &[(256, 4u8), (1536, 4), (3072, 4)] {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let codes: Vec<Vec<u8>> = (0..1000)
            .map(|i| {
                let enc = quantizer.encode(&helpers::random_unit_vector(dim, i + 200));
                let mut code = enc.mse_packed;
                code.extend_from_slice(&enc.qjl_packed);
                code
            })
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let a = &codes[idx % 1000];
                let b_code = &codes[(idx + 1) % 1000];
                idx += 1;
                quantizer.score_ip_codes_lite(a, b_code)
            });
        });
    }
    group.finish();
}

fn bench_score_ip_from_parts(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_ip_from_parts");
    for &(dim, bits) in &[(256, 4u8), (768, 4), (1024, 4), (1536, 4), (3072, 4)] {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let prepared = quantizer.prepare_ip_query(&helpers::random_unit_vector(dim, 1));
        let candidates: Vec<(f32, Vec<u8>)> = (0..1000)
            .map(|i| {
                let enc = quantizer.encode(&helpers::random_unit_vector(dim, i + 300));
                let mut code = enc.mse_packed;
                code.extend_from_slice(&enc.qjl_packed);
                (enc.gamma, code)
            })
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let (gamma, code_bytes) = &candidates[idx % 1000];
                idx += 1;
                quantizer.score_ip_from_parts(&prepared, *gamma, code_bytes)
            });
        });
        #[cfg(target_arch = "x86_64")]
        if dim == 1024
            && bits == 4
            && quantizer
                .score_ip_from_parts_avx2_multi_accum_pre_b0efa19d9_for_test(
                    &prepared,
                    candidates[0].0,
                    &candidates[0].1,
                )
                .is_some()
        {
            group.bench_function(
                BenchmarkId::new("pre_b0efa19d9_multi_accum", "d1024_b4"),
                |b| {
                    let mut idx = 0usize;
                    b.iter(|| {
                        let (gamma, code_bytes) = &candidates[idx % 1000];
                        idx += 1;
                        quantizer
                            .score_ip_from_parts_avx2_multi_accum_pre_b0efa19d9_for_test(
                                &prepared, *gamma, code_bytes,
                            )
                            .expect("AVX2/FMA old-path diagnostic should be available")
                    });
                },
            );
        }
    }
    group.finish();
}

fn bench_score_ip_encoded_lite(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_ip_encoded_lite");
    for &(dim, bits) in &[(256, 4u8), (1536, 4), (3072, 4)] {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let payloads: Vec<Vec<u8>> = (0..1000)
            .map(|i| {
                quantizer
                    .pack_payload(&quantizer.encode(&helpers::random_unit_vector(dim, i + 400)))
            })
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let a = &payloads[idx % 1000];
                let b_payload = &payloads[(idx + 1) % 1000];
                idx += 1;
                quantizer.score_ip_encoded_lite(a, b_payload)
            });
        });
    }
    group.finish();
}

fn bench_decode_approximate(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/decode_approximate");
    for &(dim, bits) in &[(1536, 4u8), (3072, 4)] {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let payloads: Vec<Vec<u8>> = (0..1000)
            .map(|i| {
                quantizer
                    .pack_payload(&quantizer.encode(&helpers::random_unit_vector(dim, i + 500)))
            })
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let payload = &payloads[idx % 1000];
                idx += 1;
                quantizer.decode_approximate(payload)
            });
        });
    }
    group.finish();
}

/// pq_fastscan-flavor scoring path (storage_format='pq_fastscan' on ec_ivf).
/// Uses a precomputed lookup table over packed 4-bit mse codes, with no QJL
/// rotation. This is the hot path inside `IvfQuantizerProfile::PqFastScan`
/// and the dominant kernel for the pq_fastscan storage format.
fn bench_score_ip_from_parts_lut_no_qjl_4bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_ip_from_parts_lut_no_qjl_4bit");
    // The no-QJL 4-bit lane is gated on `rotation::tile_dim(dim).is_some()`,
    // which today is only true for dim==1536 (the TILED_FWHT_COMPAT_DIM).
    // Other dims would hit the `prepare_ip_query_lut_no_qjl_4bit` assert.
    for &dim in &[1536usize] {
        let bits = 4u8; // lut_no_qjl_4bit is 4-bit only by construction
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let prepared =
            quantizer.prepare_ip_query_lut_no_qjl_4bit(&helpers::random_unit_vector(dim, 1));
        let payloads: Vec<Vec<u8>> = (0..1000)
            .map(|i| {
                quantizer
                    .encode(&helpers::random_unit_vector(dim, i + 600))
                    .mse_packed
            })
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let mse_packed = &payloads[idx % 1000];
                idx += 1;
                quantizer.score_ip_from_parts_lut_no_qjl_4bit(&prepared, mse_packed)
            });
        });
    }
    group.finish();
}

/// Tiled variant of the pq_fastscan path. Same query/code shape but with the
/// LUT tiled to fit in cache for large dim; used at high dimensionalities.
fn bench_score_ip_from_parts_tiled_lut_no_qjl_4bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_ip_from_parts_tiled_lut_no_qjl_4bit");
    let tile_size = 512;
    // See bench_score_ip_from_parts_lut_no_qjl_4bit: tile_dim() is only
    // Some for dim==1536, so the no-QJL 4-bit lane only accepts that dim.
    for &dim in &[1536usize] {
        let bits = 4u8;
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let prepared = quantizer.prepare_ip_query_tiled_lut_no_qjl_4bit(
            &helpers::random_unit_vector(dim, 1),
            tile_size,
        );
        let payloads: Vec<Vec<u8>> = (0..1000)
            .map(|i| {
                quantizer
                    .encode(&helpers::random_unit_vector(dim, i + 700))
                    .mse_packed
            })
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(
            BenchmarkId::new(format!("d{dim}_b{bits}_t{tile_size}"), dim),
            |b| {
                let mut idx = 0usize;
                b.iter(|| {
                    let mse_packed = &payloads[idx % 1000];
                    idx += 1;
                    quantizer.score_ip_from_parts_tiled_lut_no_qjl_4bit(&prepared, mse_packed)
                });
            },
        );
    }
    group.finish();
}

/// int8-approx variant of the pq_fastscan path: same code-set, quantized
/// LUT to int8 for further throughput at marginal recall cost.
fn bench_score_ip_from_parts_int8_approx_no_qjl_4bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_ip_from_parts_int8_approx_no_qjl_4bit");
    // See bench_score_ip_from_parts_lut_no_qjl_4bit: tile_dim() is only
    // Some for dim==1536, so the no-QJL 4-bit lane only accepts that dim.
    for &dim in &[1536usize] {
        let bits = 4u8;
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let prepared = quantizer
            .prepare_ip_query_int8_approx_no_qjl_4bit(&helpers::random_unit_vector(dim, 1));
        let payloads: Vec<Vec<u8>> = (0..1000)
            .map(|i| {
                quantizer
                    .encode(&helpers::random_unit_vector(dim, i + 800))
                    .mse_packed
            })
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let mse_packed = &payloads[idx % 1000];
                idx += 1;
                quantizer.score_ip_from_parts_int8_approx_no_qjl_4bit(&prepared, mse_packed)
            });
        });
    }
    group.finish();
}

fn bench_qjl32_block32(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/qjl32_block32");
    let dim = 1024;
    let bits = 4u8;
    let quantizer = ProdQuantizer::new(dim, bits, 42);
    let prepared = quantizer.prepare_ip_query(&helpers::random_unit_vector(dim, 1));
    let encoded: Vec<_> = (0..QJL32_BLOCK_WIDTH)
        .map(|i| quantizer.encode(&helpers::random_unit_vector(dim, i as u64 + 1000)))
        .collect();
    let codes: Vec<Vec<u8>> = encoded
        .iter()
        .map(|encoded| {
            let mut code = Vec::with_capacity(encoded.mse_packed.len() + encoded.qjl_packed.len());
            code.extend_from_slice(&encoded.mse_packed);
            code.extend_from_slice(&encoded.qjl_packed);
            code
        })
        .collect();
    let code_refs: [&[u8]; QJL32_BLOCK_WIDTH] = codes
        .iter()
        .map(Vec::as_slice)
        .collect::<Vec<_>>()
        .try_into()
        .expect("qjl32 benchmark fixture is exactly one block");
    let gammas: [f32; QJL32_BLOCK_WIDTH] = encoded
        .iter()
        .map(|encoded| encoded.gamma)
        .collect::<Vec<_>>()
        .try_into()
        .expect("qjl32 benchmark fixture is exactly one block");
    let mut out_scores = [0.0_f32; QJL32_BLOCK_WIDTH];

    group.throughput(Throughput::Elements(QJL32_BLOCK_WIDTH as u64));
    group.bench_function(BenchmarkId::new("scalar", "d1024_b4"), |b| {
        b.iter(|| {
            let mut sum = 0.0_f32;
            for lane in 0..QJL32_BLOCK_WIDTH {
                sum += qjl32_score_scalar(&quantizer, &prepared, code_refs[lane], gammas[lane]);
            }
            sum
        });
    });
    group.bench_function(BenchmarkId::new("dispatch", "d1024_b4"), |b| {
        b.iter(|| {
            let _isa =
                qjl32_score_block32(&quantizer, &prepared, code_refs, gammas, &mut out_scores);
            out_scores.iter().copied().sum::<f32>()
        });
    });
    group.finish();
}

/// Multi-bit (bits=2/4) RaBitQ block kernel vs the per-candidate scalar
/// estimate. Reports the dispatched-block scoring-share on this host
/// (NEON on M5, AVX2 on Intel). Task 106.
fn bench_rabitq32_multibit_block32(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/rabitq32_multibit_block32");
    // Full M5 sweep across the embedding dimensions the project benches, both
    // multi-bit widths. `scalar_estimate` is the per-candidate path actually
    // taken today (NeonBits4 on M5 for bits=4; true scalar for bits=2, which
    // has no NeonBits2 kernel); `block_dispatch` is the new multi-bit block
    // kernel. Task 106.
    for &dim in &[256usize, 768, 1024, 1536, 3072] {
        let prod = ProdQuantizer::cached(dim, 4, 42);
        for &(bits, label) in &[(2_u8, "bits2"), (4_u8, "bits4")] {
            let quantizer =
                RaBitQQuantizer::with_srht_bits_clip(dim, prod.clone(), bits, 2.0).unwrap();
            let prepared = quantizer.prepare_estimator(&helpers::random_unit_vector(dim, 1));
            let code_len = <RaBitQQuantizer as Quantizer>::code_len(&quantizer);
            let codes: Vec<Vec<u8>> = (0..RABITQ32_BLOCK_WIDTH)
                .map(|i| {
                    <RaBitQQuantizer as Quantizer>::encode_code(
                        &quantizer,
                        &helpers::random_unit_vector(dim, i as u64 + 1000),
                    )
                    .into_vec()
                })
                .collect();
            let code_refs: [&[u8]; RABITQ32_BLOCK_WIDTH] = codes
                .iter()
                .map(Vec::as_slice)
                .collect::<Vec<_>>()
                .try_into()
                .expect("multi-bit rabitq32 benchmark fixture is exactly one block");
            let mut out = [0.0_f32; RABITQ32_BLOCK_WIDTH];

            group.throughput(Throughput::Elements(RABITQ32_BLOCK_WIDTH as u64));
            group.bench_function(
                BenchmarkId::new(format!("scalar_estimate_{label}"), dim),
                |b| {
                    b.iter(|| {
                        let mut sum = 0.0_f32;
                        for code in &code_refs {
                            sum += prepared.estimate_ip_scalar_only(code);
                        }
                        sum
                    });
                },
            );
            group.bench_function(
                BenchmarkId::new(format!("block_dispatch_{label}"), dim),
                |b| {
                    b.iter(|| {
                        let _isa = rabitq32_multibit_score_block32(
                            &prepared, code_len, code_refs, &mut out,
                        );
                        out.iter().copied().sum::<f32>()
                    });
                },
            );
        }
    }
    group.finish();
}

fn bench_score_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_throughput");
    let dim = 1536;
    let bits = 4u8;
    let quantizer = ProdQuantizer::new(dim, bits, 42);
    let prepared = quantizer.prepare_ip_query(&helpers::random_unit_vector(dim, 1));
    let payloads: Vec<Vec<u8>> = (0..1000)
        .map(|i| {
            quantizer.pack_payload(&quantizer.encode(&helpers::random_unit_vector(dim, i + 100)))
        })
        .collect();

    group.throughput(Throughput::Elements(1000));
    group.bench_function("d1536_b4_batch1000", |b| {
        b.iter(|| {
            let mut sum = 0.0f32;
            for payload in &payloads {
                sum += quantizer.score_ip_encoded(&prepared, payload);
            }
            sum
        });
    });
    group.finish();
}

fn bench_rabitq_score(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/rabitq_score");
    let dim = 1536;
    let prod = ProdQuantizer::cached(dim, 4, 42);
    for &(bits, clip, label) in &[
        (1_u8, 2.0_f32, "bits1"),
        (4, 2.0, "bits4"),
        (8, 2.0, "bits8"),
        (8, 3.0, "bits8c3"),
        (8, 4.0, "bits8c4"),
    ] {
        let quantizer =
            RaBitQQuantizer::with_srht_bits_clip(dim, prod.clone(), bits, clip).unwrap();
        let prepared = quantizer.prepare_estimator(&helpers::random_unit_vector(dim, 1));
        let codes: Vec<Vec<u8>> = (0..1000)
            .map(|i| {
                <RaBitQQuantizer as Quantizer>::encode_code(
                    &quantizer,
                    &helpers::random_unit_vector(dim, i + 900),
                )
                .into_vec()
            })
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(label, dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let code = &codes[idx % codes.len()];
                idx += 1;
                prepared.estimate_ip_scalar_only(code)
            });
        });

        if bits == 1 || bits == 8 {
            let code_len = <RaBitQQuantizer as Quantizer>::code_len(&quantizer);
            let slab = codes
                .iter()
                .flat_map(|code| code.iter().copied())
                .collect::<Vec<_>>();
            let mut out = Vec::new();
            group.throughput(Throughput::Elements(codes.len() as u64));
            group.bench_function(BenchmarkId::new(format!("{label}_batch1000"), dim), |b| {
                b.iter(|| {
                    prepared
                        .estimate_ip_batch(&slab, code_len, &mut out)
                        .unwrap();
                    out.iter().copied().sum::<f32>()
                });
            });
        }
    }
    group.finish();
}

fn bench_hamming32_block32(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/hamming32_block32");
    for word_count in [24usize, 192] {
        let mut state = 0x9E37_79B9_7F4A_7C15u64;
        let mut next = move || {
            state = state
                .wrapping_mul(0xA076_1D64_78BD_642F)
                .wrapping_add(0xE703_7ED1_A0B4_28DB);
            state
        };
        let query: Vec<u64> = (0..word_count).map(|_| next()).collect();
        let candidates: Vec<Vec<u64>> = (0..32)
            .map(|_| (0..word_count).map(|_| next()).collect())
            .collect();
        let candidate_refs: [&[u64]; 32] = candidates
            .iter()
            .map(Vec::as_slice)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let mut out = [0u32; 32];

        group.throughput(Throughput::Elements(32));
        group.bench_function(BenchmarkId::new("scalar", word_count * 64), |b| {
            b.iter(|| {
                ecaz::bench_api::hamming32_block32_scalar_reference(
                    &query,
                    &candidate_refs,
                    &mut out,
                );
                out.iter().copied().sum::<u32>()
            });
        });
        group.bench_function(BenchmarkId::new("dispatch", word_count * 64), |b| {
            b.iter(|| {
                ecaz::bench_api::hamming32_block32_dispatch(&query, &candidate_refs, &mut out);
                out.iter().copied().sum::<u32>()
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_score_ip_encoded,
    bench_score_ip_codes_lite,
    bench_score_ip_from_parts,
    bench_score_ip_encoded_lite,
    bench_score_ip_from_parts_lut_no_qjl_4bit,
    bench_score_ip_from_parts_tiled_lut_no_qjl_4bit,
    bench_score_ip_from_parts_int8_approx_no_qjl_4bit,
    bench_qjl32_block32,
    bench_rabitq32_multibit_block32,
    bench_decode_approximate,
    bench_score_throughput,
    bench_rabitq_score,
    bench_hamming32_block32
);
criterion_main!(benches);
