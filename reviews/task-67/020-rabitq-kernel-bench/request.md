# Task 67 Packet 020: RaBitQ Kernel Bench Command And AVX-512 Measurement

## Summary

This packet adds `ecaz bench rabitq-kernel`, a packet-runnable microbenchmark
for Task 67's prepared RaBitQ estimator paths, then runs it on the AWS
`10k-intel` AVX-512 host in scalar and auto-SIMD modes.

Reviewer feedback on the initial packet was addressed before the final AWS
run:

- `single-dispatch` now uses `PreparedEstimator::estimate_ip`, not the scalar
  reference helper.
- `single-scalar` remains as an explicit reference row.
- `batch` now covers bits1, bits4, and bits8 variants.
- suite configs force `--simd-mode scalar` / `--simd-mode auto` in-band.

## AWS AVX-512 Kernel Results

Final code under test: `5df1308d40bda38d1da65f2325bab32e48fdf10b`.

Scalar run backend: `scalar`.
Auto run backend: `avx512f+vpopcntdq+bw+bf16`.

| variant | mode | scalar ns/score | auto ns/score | speedup |
| --- | --- | ---: | ---: | ---: |
| bits1 | batch | 456.83 | 81.67 | 5.59x |
| bits1 | single-dispatch | 469.95 | 124.83 | 3.76x |
| bits4 | batch | 3547.63 | 393.13 | 9.02x |
| bits4 | single-dispatch | 3589.92 | 404.28 | 8.88x |
| bits8 | batch | 817.25 | 69.50 | 11.76x |
| bits8 | single-dispatch | 827.94 | 145.90 | 5.67x |
| bits8c3 | batch | 819.06 | 69.39 | 11.80x |
| bits8c3 | single-dispatch | 811.84 | 141.29 | 5.75x |
| bits8c4 | batch | 818.39 | 69.55 | 11.77x |
| bits8c4 | single-dispatch | 889.27 | 158.16 | 5.62x |

Source artifacts:

- `artifacts/scalar/rabitq-kernel-scalar.log`
- `artifacts/auto/rabitq-kernel-auto.log`
- `artifacts/scalar/suite-manifest.json`
- `artifacts/auto/suite-manifest.json`

## Local Desktop Diagnostic

The local desktop is Intel AVX2/FMA only, not AVX-512:

- CPU: `Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz`
- auto backend: `avx2+fma`
- source: `artifacts/local/cpu-features.log`,
  `artifacts/local/rabitq-kernel-auto-smoke-v3.log`

This local evidence is diagnostic only. The Task 67 AVX-512 evidence is the
AWS `10k-intel` run above.

## Interpretation

The per-kernel AVX-512 measurements clear the individual Task 67 kernel
targets for the covered bits1, bits4, and bits8 paths. They also explain why
packet 017's SQL-level wall-time result was limited: the SIMD kernels are
substantially faster, while end-to-end query wall time is still dominated by
non-kernel overhead in that harness.

This packet does not claim full Task 67 completion by itself. Packet 017
remains the latest real-10k recall/wall-time Slice J run, and it showed no
recall regression but did not meet the total wall-time interpretation of the
headline gate.

## Validation

- `cargo build -p ecaz-cli` passed locally.
- local `rabitq-kernel` scalar/auto smoke runs passed and wrote packet-local
  logs.
- scalar and auto suite configs passed `ecaz bench suite audit`.
- AWS `10k-intel` install of `5df1308d4` passed.
- AWS scalar and auto raw suites completed and synced artifacts.
- AWS `10k-intel` was paused after measurement; final status is
  `state: paused`, `~$0.00/hr running`.

Please review whether this closes the packet 020 blockers and whether the next
Task 67 step should treat the remaining gap as SQL/query-path overhead rather
than kernel throughput.
