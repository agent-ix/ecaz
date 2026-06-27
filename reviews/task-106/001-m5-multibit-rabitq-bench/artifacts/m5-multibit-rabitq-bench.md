# Task 106 — M5 multi-bit RaBitQ kernel bench + smoke evidence

Host: Apple M5 (aarch64, NEON). Built and run locally 2026-06-12.
Scope: the multi-bit (bits=2/4) RaBitQ block kernel added by Task 106 and the
IVF routing that consumes it. AVX2 is built but cfg-gated to x86 (not
executable on M5); SVE routes to NEON. Intel/G4 lanes are a separate trip.

## Build

- `cargo build --release` → `target/release/libecaz.dylib` (real cdylib),
  Finished in 2m35s.
- `cargo check --all-targets --features bench` → clean (lib + tests + benches
  + bins type-check).

## pg smoke (real Postgres 18)

`cargo pgrx test pg18 test_ec_ivf_rabitq_storage_build_scan_insert_vacuum`:

```
test tests::pg_test_ec_ivf_rabitq_storage_build_scan_insert_vacuum ... ok
test result: ok. 1 passed; 0 failed; ... finished in 18.33s
```

The IVF RaBitQ build → scan → insert → vacuum round-trip passes end-to-end,
exercising the new multi-bit routing, the scratch-SoA batch-decode gate
(now admits `Auto` + all RaBitQ widths), and the bits=4 → arithmetic-estimator
path. (A pgrx test-port collision from a stale `tqvector` postmaster was
cleared first; not a code issue.)

## Microbench sweep (criterion, 32-candidate block, median of 100 samples)

`cargo bench --features bench --bench quant_score -- rabitq32_multibit`

`scalar_estimate` = the per-candidate path taken today
(`estimate_ip_scalar_only`): NeonBits4 on M5 for bits=4; true scalar for
bits=2 (no NeonBits2 kernel exists). `block_dispatch` = the new multi-bit
block kernel (NEON on M5).

| dim  | bits | scalar_estimate | block_dispatch | block speedup |
| ---- | ---- | --------------- | -------------- | ------------- |
| 256  | 2    | 8.93 µs         | 2.90 µs        | **3.08×**     |
| 256  | 4    | 1.18 µs         | 3.22 µs        | 0.37×         |
| 768  | 2    | 25.81 µs        | 8.54 µs        | **3.02×**     |
| 768  | 4    | 3.40 µs         | 9.49 µs        | 0.36×         |
| 1024 | 2    | 34.14 µs        | 11.37 µs       | **3.00×**     |
| 1024 | 4    | 4.53 µs         | 12.67 µs       | 0.36×         |
| 1536 | 2    | 51.03 µs        | 17.02 µs       | **3.00×**     |
| 1536 | 4    | 6.79 µs         | 18.98 µs       | 0.36×         |
| 3072 | 2    | 104.27 µs       | 33.98 µs       | **3.07×**     |
| 3072 | 4    | 13.52 µs        | 37.82 µs       | 0.36×         |

## Conclusion → routing (evidence, not assumption)

- **bits=2 → block kernel.** Consistent ~3.0–3.1× win across all dims; bits=2
  has no per-candidate SIMD kernel, so the alternative is true scalar.
- **bits=4 → per-candidate arithmetic estimator (NeonBits4).** The block
  kernel is consistently ~2.7× *slower* on M5 NEON — its per-dim scalar gather
  loses to NeonBits4's vectorized nibble unpack. So IVF bits=4 routes to
  `estimate_ip_batch`, not the block kernel.
- **bits=8 → arithmetic estimator** (full-byte level, no LUT fast-scan shape).

The block kernel stays built and tested for all ISAs so the Intel AVX2
hardware-gather (`permutevar8x32`) path can be evaluated on the Intel lane,
where the gather may change the bits=4 verdict. Routing preference is per-ISA
and set by measured evidence.
