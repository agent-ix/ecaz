# Task 67 Packet 019 Artifact Manifest

- Head SHA at experiment: `47fed5ba2`
- Revert SHA: `5c51abfc8`
- Restore SHA: `12ed902df`
- Task bucket: `reviews/task-67/`
- Packet path: `reviews/task-67/019-avx512-bits1-sign-flip/`
- Timestamp: `2026-05-30T05:08Z`
- Lane: AWS Intel AVX-512 10k real-corpus bits=1 primary measurement
- Fixture: `ec_real_10k`, 200 queries, nprobe sweep `16,32,64`
- Storage format: `rabitq`, `quant_bits=1`, `rerank=heap_f32`,
  `rerank_width=100`
- Rerank mode: heap_f32 rerank for primary recall/latency
- Isolated one-index-per-table surfaces: yes, scalar and auto use distinct
  table prefixes.

## Code Timeline

- `47fed5ba2 Use sign flips for AVX512 bits1 scoring`
  - Exact bits=1 sign-flip accumulation experiment.
- `5c51abfc8 Revert "Use sign flips for AVX512 bits1 scoring"`
  - Reverted because auto-SIMD measured slower than scalar.
- `12ed902df Restore AVX512 bits1 byte LUT scoring`
  - Restored the prior byte-LUT implementation that had the best measured
    Task 67 Intel result so far.

## Local Validation

- Command: `cargo test -p ecaz quant::rabitq -- --nocapture`
- Result: passed locally, 46 tests, 0 failed.

## Suite Config Audits

- Command:
  `target/debug/ecaz bench suite audit --config reviews/task-67/019-avx512-bits1-sign-flip/artifacts/task67-bits1-sign-scalar-suite.json`
- Result: `[suite:task67-bits1-sign-scalar] audit passed: 3 steps`
- Command:
  `target/debug/ecaz bench suite audit --config reviews/task-67/019-avx512-bits1-sign-flip/artifacts/task67-bits1-sign-auto-suite.json`
- Result: `[suite:task67-bits1-sign-auto] audit passed: 3 steps`

## Cloud Resume and Install

- Resume command:
  `target/debug/ecaz cloud resume --profile 10k-intel --log-file reviews/task-67/019-avx512-bits1-sign-flip/artifacts/preflight/cloud-resume.log`
- Install command:
  `target/debug/ecaz cloud install --profile 10k-intel --git-ref 47fed5ba2 --skip-extension-recreate --database postgres --timeout 3600 --log-file reviews/task-67/019-avx512-bits1-sign-flip/artifacts/preflight/cloud-install-47fed5ba2.log`
- Result: `install: profile=10k-intel db=10.42.1.147 ref=47fed5ba2 ok`

## Scalar Primary Suite

- Command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode scalar --config reviews/task-67/019-avx512-bits1-sign-flip/artifacts/task67-bits1-sign-scalar-suite.json --suite task67-bits1-sign-scalar --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/019-avx512-bits1-sign-flip/artifacts/scalar/cloud-bench-sign-scalar.log`
- S3 run:
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-bits1-sign-scalar/20260530T050716Z/`
- Artifacts:
  - `artifacts/scalar/cloud-bench-sign-scalar.log`
  - `artifacts/scalar/suite-run.log`
  - `artifacts/scalar/suite-manifest.json`
  - `artifacts/scalar/results.jsonl`
  - `artifacts/scalar/load-10k-rabitq1-scalar.log`
  - `artifacts/scalar/recall-10k-rabitq1-scalar.log`
  - `artifacts/scalar/latency-10k-rabitq1-scalar.log`
- Key results:
  - recall@10: nprobe 16 = 0.9985, nprobe 32 = 1.0000,
    nprobe 64 = 1.0000.
  - latency mean: nprobe 16 = 1.12 ms, nprobe 32 = 1.49 ms,
    nprobe 64 = 2.16 ms.

## Auto Primary Suite

- Command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode auto --config reviews/task-67/019-avx512-bits1-sign-flip/artifacts/task67-bits1-sign-auto-suite.json --suite task67-bits1-sign-auto --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/019-avx512-bits1-sign-flip/artifacts/auto/cloud-bench-sign-auto.log`
- S3 run:
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-bits1-sign-auto/20260530T050801Z/`
- Artifacts:
  - `artifacts/auto/cloud-bench-sign-auto.log`
  - `artifacts/auto/suite-run.log`
  - `artifacts/auto/suite-manifest.json`
  - `artifacts/auto/results.jsonl`
  - `artifacts/auto/load-10k-rabitq1-auto.log`
  - `artifacts/auto/recall-10k-rabitq1-auto.log`
  - `artifacts/auto/latency-10k-rabitq1-auto.log`
- Key results:
  - recall@10: nprobe 16 = 0.9985, nprobe 32 = 1.0000,
    nprobe 64 = 1.0000.
  - latency mean: nprobe 16 = 1.19 ms, nprobe 32 = 1.56 ms,
    nprobe 64 = 2.35 ms.
  - scalar/auto ratio: nprobe 16 = 0.94x, nprobe 32 = 0.96x,
    nprobe 64 = 0.92x.

## Cloud Pause

- Command:
  `target/debug/ecaz cloud pause --profile 10k-intel --log-file reviews/task-67/019-avx512-bits1-sign-flip/artifacts/preflight/cloud-pause.log`
- Result: `pause: profile=10k-intel stopped (db + loader)`
- Follow-up status: `state: paused`, running cost `$0.00/hr`.

## Conclusion

The sign-flip path is rejected. It preserved recall but made the auto lane
slower than scalar for this fixture. Continue Task 67 from the restored
byte-LUT AVX-512 bits=1 baseline.
