# Task 67 Packet 020 Artifact Manifest

- Head SHA: `5df1308d40bda38d1da65f2325bab32e48fdf10b`
- Task bucket: `reviews/task-67/`
- Packet path: `reviews/task-67/020-rabitq-kernel-bench/`
- Timestamp: `2026-05-30T06:10:26Z`
- Lane: RaBitQ prepared-estimator kernel microbenchmark, local AVX2 diagnostic
  plus AWS Intel AVX-512 measurement
- Fixture: synthetic deterministic unit vectors, `dim=1536`, `candidates=1000`,
  `iterations=1000`
- Storage format: not applicable; in-process raw benchmark, no SQL storage
  surface
- Rerank mode: not applicable
- Surface isolation: not applicable; raw `ecaz bench rabitq-kernel` suite steps
  do not create index tables

## Code Commits

- `2edbb887c` added `ecaz bench rabitq-kernel`.
- `39b201160` added `--simd-mode` so scalar/auto lanes are reproducible from
  the suite config.
- `5df1308d4` fixed reviewer blockers by measuring `single-dispatch`,
  retaining explicit `single-scalar`, and including bits4 batch.

## Local Build And Smoke

### `artifacts/local/cargo-build-ecaz-cli.log`

- Command: `cargo build -p ecaz-cli`
- Result: passed.
- Note: existing unrelated warning remains for
  `crates/ecaz-cli/src/commands/corpus/load.rs:165`.

### `artifacts/local/rabitq-kernel-scalar-smoke-v3.log`

- Command:
  `target/debug/ecaz bench rabitq-kernel --dim 256 --candidates 16 --iterations 10 --simd-mode scalar --log-output reviews/task-67/020-rabitq-kernel-bench/artifacts/local/rabitq-kernel-scalar-smoke-v3.log`
- Result: passed.
- Key line: `backend=scalar`.
- Key shape: includes `single-dispatch`, `single-scalar`, and `batch` rows for
  bits1/bits4/bits8 variants.

### `artifacts/local/rabitq-kernel-auto-smoke-v3.log`

- Command:
  `target/debug/ecaz bench rabitq-kernel --dim 256 --candidates 16 --iterations 10 --simd-mode auto --log-output reviews/task-67/020-rabitq-kernel-bench/artifacts/local/rabitq-kernel-auto-smoke-v3.log`
- Result: passed.
- Key line: `backend=avx2+fma`.

### `artifacts/local/cpu-features.log`

- Command: `lscpu`
- Result: local desktop is AVX2/FMA only, not AVX-512.
- Key lines:
  - `Model name: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz`
  - flags include `fma` and `avx2`; no `avx512f`.

### Suite Audits

- `artifacts/local/suite-audit-scalar-v2.log`
  - Command:
    `target/debug/ecaz bench suite audit --config reviews/task-67/020-rabitq-kernel-bench/artifacts/task67-rabitq-kernel-scalar-suite.json`
  - Result: `audit passed: 1 steps`.
- `artifacts/local/suite-audit-auto-v2.log`
  - Command:
    `target/debug/ecaz bench suite audit --config reviews/task-67/020-rabitq-kernel-bench/artifacts/task67-rabitq-kernel-auto-suite.json`
  - Result: `audit passed: 1 steps`.

## AWS AVX-512 Measurement

### Preflight

- `artifacts/preflight/cloud-install-5df1308d4.log`
  - Command:
    `target/debug/ecaz cloud install --profile 10k-intel --git-ref 5df1308d4 --skip-extension-recreate --database postgres --timeout 3600 --log-file reviews/task-67/020-rabitq-kernel-bench/artifacts/preflight/cloud-install-5df1308d4.log`
  - Result: install passed; the operator-created `--log-file` artifact is
    zero bytes, so the durable evidence that this install was active is the
    successful post-install suite execution from `/usr/local/bin/ecaz` below.
- `artifacts/preflight/cloud-status-after-5df1308d4-final-script.log`
  - Command:
    `target/debug/ecaz cloud status --profile 10k-intel`
  - Result: `state: paused`, `~$0.00/hr running`, retained storage
    `~$8.00/mo`.

### Scalar Suite

- Config: `artifacts/task67-rabitq-kernel-scalar-suite.json`
- Command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode scalar --config reviews/task-67/020-rabitq-kernel-bench/artifacts/task67-rabitq-kernel-scalar-suite.json --suite task67-rabitq-kernel-scalar --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/020-rabitq-kernel-bench/artifacts/scalar/cloud-bench-kernel-scalar-5df1308d4.log`
- S3 run:
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-rabitq-kernel-scalar/20260530T060801Z/`
- Result: passed and synced artifacts.
- Key artifact: `artifacts/scalar/rabitq-kernel-scalar.log`
- Key line: `backend=scalar`.

### Auto-SIMD Suite

- Config: `artifacts/task67-rabitq-kernel-auto-suite.json`
- Command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode auto --config reviews/task-67/020-rabitq-kernel-bench/artifacts/task67-rabitq-kernel-auto-suite.json --suite task67-rabitq-kernel-auto --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/020-rabitq-kernel-bench/artifacts/auto/cloud-bench-kernel-auto-5df1308d4.log`
- S3 run:
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-rabitq-kernel-auto/20260530T060822Z/`
- Result: passed and synced artifacts.
- Key artifact: `artifacts/auto/rabitq-kernel-auto.log`
- Key line: `backend=avx512f+vpopcntdq+bw+bf16`.

## Key Result Lines

All values are `ns_per_score`, scalar divided by auto:

| variant | mode | scalar | auto | speedup |
| --- | --- | ---: | ---: | ---: |
| bits1 | batch | 456.83 | 81.67 | 5.59x |
| bits1 | single-dispatch | 469.95 | 124.83 | 3.76x |
| bits4 | batch | 3547.63 | 393.13 | 9.02x |
| bits4 | single-dispatch | 3589.92 | 404.28 | 8.88x |
| bits8 | batch | 817.25 | 69.50 | 11.76x |
| bits8c3 | batch | 819.06 | 69.39 | 11.80x |
| bits8c4 | batch | 818.39 | 69.55 | 11.77x |

## Limitations

This packet proves the in-process kernel throughput layer. It does not replace
packet 017's real-10k recall and SQL wall-time Slice J evidence. Packet 017
showed no recall regression but did not meet the total wall-time interpretation
of Task 67's headline gate.
