# Task 67 Packet 017: Intel Measurement Final Attempt

## Summary

This packet runs the Slice J Intel measurement lane on the `10k-intel` AWS profile added in packets 015 and 016. The host is x86_64 Intel with AVX2, FMA, AVX-512F, AVX512BW, AVX512 VPOPCNTDQ, and AVX512 BF16 available.

The result is mixed:

- Functional validation passes on the Intel host: `cargo test -p ecaz quant::rabitq -- --nocapture` reports 46 matching tests passed, 0 failed.
- Recall is unchanged between scalar and auto-SIMD in the primary bits=1 benchmark.
- Auto-SIMD improves measured primary bits=1 mean latency by 1.88x to 2.54x depending on nprobe.
- Auto-SIMD improves `rabitq8*` sidecar total-bound p50 by about 2.0x across all four variants.
- This does **not** close Task 67's headline performance gate yet. The task asks for at least 3x on the bits=1 batched path and at least 4x across the four bits=8 variants.

## Hardware And Preflight

See `artifacts/preflight/validation.log`.

Key facts:

- DB instance: `i-02811174cc6ded75c`
- CPU: Intel Xeon Platinum 8488C
- Extension: `ecaz|0.1.1`
- `shared_preload_libraries`: `ecaz`
- Staged real-10k fixtures:
  - corpus: 10,000 rows, 202 MB TSV
  - queries: 200 rows, 4.1 MB TSV

## Suite Shape

Primary scalar and auto suites use `rerank=heap_f32` and `rerank_width=100`:

- `artifacts/task67-intel-10k-scalar-suite.json`
- `artifacts/task67-intel-10k-auto-suite.json`

Sidecar scalar and auto suites use separate `rerank=off` fixtures, because `bench sidecar-rerank` requires an isolated sidecar-compatible index:

- `artifacts/task67-intel-10k-scalar-sidecar-suite.json`
- `artifacts/task67-intel-10k-auto-sidecar-suite.json`

All four configs pass `ecaz bench suite audit`.

## Primary Bits=1 Results

Scalar primary:

| nprobe | recall@10 | mean latency |
| --- | ---: | ---: |
| 16 | 0.9985 | 2.28 ms |
| 32 | 1.0000 | 3.70 ms |
| 64 | 1.0000 | 6.57 ms |

Auto-SIMD primary:

| nprobe | recall@10 | mean latency | speedup |
| --- | ---: | ---: | ---: |
| 16 | 0.9985 | 1.21 ms | 1.88x |
| 32 | 1.0000 | 1.67 ms | 2.22x |
| 64 | 1.0000 | 2.59 ms | 2.54x |

Source artifacts:

- `artifacts/scalar/recall-10k-rabitq1-scalar.log`
- `artifacts/scalar/latency-10k-rabitq1-scalar.log`
- `artifacts/auto/recall-10k-rabitq1-auto.log`
- `artifacts/auto/latency-10k-rabitq1-auto.log`

## Sidecar `rabitq8*` Results

Scalar sidecar, total-bound p50:

| variant | recall@10 | sidecar score p50 | total-bound p50 |
| --- | ---: | ---: | ---: |
| `rabitq8` | 0.9865 | 0.026 ms | 7.974 ms |
| `rabitq8ls` | 0.9835 | 0.025 ms | 8.003 ms |
| `rabitq8c3` | 0.9945 | 0.027 ms | 8.002 ms |
| `rabitq8c4` | 0.9990 | 0.027 ms | 7.990 ms |

Auto-SIMD sidecar, total-bound p50:

| variant | recall@10 | sidecar score p50 | total-bound p50 | speedup |
| --- | ---: | ---: | ---: | ---: |
| `rabitq8` | 0.9865 | 0.026 ms | 3.967 ms | 2.01x |
| `rabitq8ls` | 0.9835 | 0.024 ms | 3.959 ms | 2.02x |
| `rabitq8c3` | 0.9945 | 0.026 ms | 4.002 ms | 2.00x |
| `rabitq8c4` | 0.9990 | 0.027 ms | 4.002 ms | 2.00x |

Source artifacts:

- `artifacts/scalar-sidecar/sidecar-10k-rabitq8-variants-scalar.log`
- `artifacts/auto-sidecar/sidecar-10k-rabitq8-variants-auto.log`

## Interpretation

This packet validates that the Intel lane runs correctly on AVX-512-capable hardware and that auto-SIMD is faster than scalar without recall loss on the measured real-10k fixture.

It also shows Task 67 is not complete on performance. The current implementation does not meet the Slice J headline gate. The next slice should either:

- add a per-kernel RaBitQ microbench so kernel speedups are separated from SQL, candidate fetch, and sidecar I/O overhead; or
- continue optimizing the hot x86 kernels and/or batched scan path before rerunning this packet shape.

## Review Request

Please review:

1. Whether the split primary-vs-sidecar suite shape is acceptable for Slice J evidence.
2. Whether the performance-gate interpretation is correct.
3. Whether the next step should be a per-kernel benchmark harness or another kernel optimization slice.
