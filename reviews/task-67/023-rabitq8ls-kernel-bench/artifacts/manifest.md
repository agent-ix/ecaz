# Task 67 Packet 023 Artifact Manifest

- Head SHA: `c72003b7b0438965c586a231b34753d1b745c94f`
- Task bucket: `reviews/task-67/`
- Packet path: `reviews/task-67/023-rabitq8ls-kernel-bench/`
- Timestamp: `2026-05-30T06:59:42Z`
- Lane: RaBitQ prepared-estimator kernel microbenchmark, local AVX2 diagnostic
  plus AWS Intel AVX-512 measurement
- Fixture: synthetic deterministic unit vectors, `dim=1536`, `candidates=1000`,
  `iterations=1000`
- Storage format: not applicable; in-process raw benchmark, no SQL storage
  surface
- Rerank mode: not applicable
- Surface isolation: not applicable; raw `ecaz bench rabitq-kernel` suite steps
  do not create index tables

## Code Change

- Commit: `c72003b7b Measure RaBitQ8 least-squares kernel path`
- Touched file: `crates/ecaz-cli/src/commands/bench/rabitq_kernel.rs`
- Change: add a `bits8ls` / `single-least-squares` benchmark row that invokes
  `PreparedEstimator::estimate_ip_least_squares_scalar_only`.

## Local Build And Smoke

### `artifacts/local/cargo-build-ecaz-cli.log`

- Command: `cargo build -p ecaz-cli`
- Result: passed.
- Note: existing unrelated warning remains for
  `crates/ecaz-cli/src/commands/corpus/load.rs:165`.

### `artifacts/local/rabitq-kernel-auto-smoke.log`

- Command:
  `target/debug/ecaz bench rabitq-kernel --dim 256 --candidates 16 --iterations 10 --simd-mode auto --log-output reviews/task-67/023-rabitq8ls-kernel-bench/artifacts/local/rabitq-kernel-auto-smoke.log`
- Result: passed.
- Key line: `backend=avx2+fma`.
- Key row: `bits8ls single-least-squares`.

### Suite Audits

- `artifacts/local/suite-audit-scalar.log`
  - Command:
    `target/debug/ecaz bench suite audit --config reviews/task-67/023-rabitq8ls-kernel-bench/artifacts/task67-rabitq8ls-kernel-scalar-suite.json`
  - Result: `audit passed: 1 steps`.
- `artifacts/local/suite-audit-auto.log`
  - Command:
    `target/debug/ecaz bench suite audit --config reviews/task-67/023-rabitq8ls-kernel-bench/artifacts/task67-rabitq8ls-kernel-auto-suite.json`
  - Result: `audit passed: 1 steps`.

## AWS AVX-512 Measurement

### Preflight

- `artifacts/preflight/cloud-resume.log`
  - Command:
    `target/debug/ecaz cloud resume --profile 10k-intel --log-file reviews/task-67/023-rabitq8ls-kernel-bench/artifacts/preflight/cloud-resume.log`
  - Result: `resume: profile=10k-intel db=10.42.1.147 ready`.
- `artifacts/preflight/cloud-install-c72003b7b.log`
  - Command:
    `target/debug/ecaz cloud install --profile 10k-intel --git-ref c72003b7b --skip-extension-recreate --database postgres --timeout 3600 --log-file reviews/task-67/023-rabitq8ls-kernel-bench/artifacts/preflight/cloud-install-c72003b7b.log`
  - Result: `install: profile=10k-intel db=10.42.1.147 ref=c72003b7b ok`.
- `artifacts/preflight/cloud-status-after-pause.log`
  - Command:
    `target/debug/ecaz cloud status --profile 10k-intel`
  - Result: `state: paused`, `~$0.00/hr running`, retained storage
    `~$8.00/mo`.

### Scalar Suite

- Config: `artifacts/task67-rabitq8ls-kernel-scalar-suite.json`
- Command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode scalar --config reviews/task-67/023-rabitq8ls-kernel-bench/artifacts/task67-rabitq8ls-kernel-scalar-suite.json --suite task67-rabitq8ls-kernel-scalar --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/023-rabitq8ls-kernel-bench/artifacts/scalar/cloud-bench-kernel-scalar.log`
- S3 run:
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-rabitq8ls-kernel-scalar/20260530T065756Z/`
- Result: passed and synced artifacts.
- Key artifact: `artifacts/scalar/rabitq-kernel-scalar.log`
- Key line: `backend=scalar`.

### Auto-SIMD Suite

- Config: `artifacts/task67-rabitq8ls-kernel-auto-suite.json`
- Command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode auto --config reviews/task-67/023-rabitq8ls-kernel-bench/artifacts/task67-rabitq8ls-kernel-auto-suite.json --suite task67-rabitq8ls-kernel-auto --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/023-rabitq8ls-kernel-bench/artifacts/auto/cloud-bench-kernel-auto.log`
- S3 run:
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-rabitq8ls-kernel-auto/20260530T065817Z/`
- Result: passed and synced artifacts.
- Key artifact: `artifacts/auto/rabitq-kernel-auto.log`
- Key line: `backend=avx512f+vpopcntdq+bw+bf16`.

## Key Result Lines

All values are `ns_per_score`, scalar divided by auto:

| variant | mode | scalar | auto | speedup |
| --- | --- | ---: | ---: | ---: |
| bits1 | batch | 461.74 | 81.71 | 5.65x |
| bits1 | single-dispatch | 449.24 | 134.19 | 3.35x |
| bits4 | batch | 3546.69 | 393.00 | 9.03x |
| bits4 | single-dispatch | 3522.71 | 403.56 | 8.73x |
| bits8 | batch | 819.56 | 68.32 | 12.00x |
| bits8 | single-dispatch | 890.64 | 142.45 | 6.25x |
| bits8ls | single-least-squares | 807.72 | 120.74 | 6.69x |
| bits8c3 | batch | 819.55 | 70.25 | 11.67x |
| bits8c3 | single-dispatch | 818.14 | 135.50 | 6.04x |
| bits8c4 | batch | 819.42 | 70.13 | 11.68x |
| bits8c4 | single-dispatch | 830.11 | 131.47 | 6.31x |

## Limitation

This packet proves the `rabitq8ls` prepared-estimator kernel row. It does not
replace the SQL wall-time evidence and interpretation in packets 017, 021, and
022.
