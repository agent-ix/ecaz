# Task 67 Packet 017 Artifact Manifest

- head SHA: `9ba40a701ee86c91a63c6ec23dd697f2d2b8a0e0`
- task bucket: `reviews/task-67/017-intel-measurement-final`
- timestamp: `2026-05-30T03:49:51Z`
- profile: `10k-intel`
- DB host: `m7i.2xlarge`, `i-02811174cc6ded75c`, Intel Xeon Platinum 8488C
- loader host: `c7i.large`, `i-018e3386be2cc0e9b`
- storage format: `rabitq`
- primary rerank mode: `heap_f32`, `rerank_width=100`
- sidecar fixture rerank mode: `off`
- isolated one-index-per-table surfaces: yes, each scalar/auto and primary/sidecar suite uses a distinct table prefix

## Configs

- `artifacts/task67-intel-10k-scalar-suite.json`
  - `ecaz bench suite` config for scalar primary load, recall, and latency.
- `artifacts/task67-intel-10k-auto-suite.json`
  - `ecaz bench suite` config for auto-SIMD primary load, recall, and latency.
- `artifacts/task67-intel-10k-scalar-sidecar-suite.json`
  - `ecaz bench suite` config for scalar sidecar fixture load and `rabitq8*` sidecar measurement.
- `artifacts/task67-intel-10k-auto-sidecar-suite.json`
  - `ecaz bench suite` config for auto-SIMD sidecar fixture load and `rabitq8*` sidecar measurement.

## Preflight

- `artifacts/preflight/validation.log`
  - Records host CPU capabilities, input staging, PostgreSQL preload state, focused Intel test result, suite audits, and cloud bench commands.

## Scalar Primary

- command:
  - `target/debug/ecaz cloud bench --profile 10k-intel --config reviews/task-67/017-intel-measurement-final/artifacts/task67-intel-10k-scalar-suite.json --suite task67-intel-10k-scalar --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/017-intel-measurement-final/artifacts/scalar/cloud-bench-scalar-rerun.log`
- S3 run:
  - `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-intel-10k-scalar/20260530T034705Z/`
- artifacts:
  - `artifacts/scalar/suite-manifest.json`
  - `artifacts/scalar/results.jsonl`
  - `artifacts/scalar/suite-run.log`
  - `artifacts/scalar/load-10k-rabitq1-scalar.log`
  - `artifacts/scalar/recall-10k-rabitq1-scalar.log`
  - `artifacts/scalar/latency-10k-rabitq1-scalar.log`
  - `artifacts/scalar/cloud-bench-scalar-rerun.log`
- key result lines:
  - recall: nprobe 16 = 0.9985, nprobe 32 = 1.0000, nprobe 64 = 1.0000.
  - latency mean: nprobe 16 = 2.28 ms, nprobe 32 = 3.70 ms, nprobe 64 = 6.57 ms.

## Auto Primary

- command:
  - `target/debug/ecaz cloud bench --profile 10k-intel --config reviews/task-67/017-intel-measurement-final/artifacts/task67-intel-10k-auto-suite.json --suite task67-intel-10k-auto --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/017-intel-measurement-final/artifacts/auto/cloud-bench-auto.log`
- S3 run:
  - `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-intel-10k-auto/20260530T034743Z/`
- artifacts:
  - `artifacts/auto/suite-manifest.json`
  - `artifacts/auto/results.jsonl`
  - `artifacts/auto/suite-run.log`
  - `artifacts/auto/load-10k-rabitq1-auto.log`
  - `artifacts/auto/recall-10k-rabitq1-auto.log`
  - `artifacts/auto/latency-10k-rabitq1-auto.log`
  - `artifacts/auto/cloud-bench-auto.log`
- key result lines:
  - recall: nprobe 16 = 0.9985, nprobe 32 = 1.0000, nprobe 64 = 1.0000.
  - latency mean: nprobe 16 = 1.21 ms, nprobe 32 = 1.67 ms, nprobe 64 = 2.59 ms.
  - speedup vs scalar mean: nprobe 16 = 1.88x, nprobe 32 = 2.22x, nprobe 64 = 2.54x.

## Scalar Sidecar

- command:
  - `target/debug/ecaz cloud bench --profile 10k-intel --config reviews/task-67/017-intel-measurement-final/artifacts/task67-intel-10k-scalar-sidecar-suite.json --suite task67-intel-10k-scalar-sidecar --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/017-intel-measurement-final/artifacts/scalar-sidecar/cloud-bench-scalar-sidecar.log`
- S3 run:
  - `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-intel-10k-scalar-sidecar/20260530T034831Z/`
- artifacts:
  - `artifacts/scalar-sidecar/suite-manifest.json`
  - `artifacts/scalar-sidecar/results.jsonl`
  - `artifacts/scalar-sidecar/suite-run.log`
  - `artifacts/scalar-sidecar/load-10k-rabitq1-scalar-sidecar.log`
  - `artifacts/scalar-sidecar/sidecar-10k-rabitq8-variants-scalar.log`
  - `artifacts/scalar-sidecar/cloud-bench-scalar-sidecar.log`
- key result lines:
  - `rabitq8`: recall@10 0.9865, sidecar score p50 0.026 ms, total bound p50 7.974 ms.
  - `rabitq8ls`: recall@10 0.9835, sidecar score p50 0.025 ms, total bound p50 8.003 ms.
  - `rabitq8c3`: recall@10 0.9945, sidecar score p50 0.027 ms, total bound p50 8.002 ms.
  - `rabitq8c4`: recall@10 0.9990, sidecar score p50 0.027 ms, total bound p50 7.990 ms.

## Auto Sidecar

- command:
  - `target/debug/ecaz cloud bench --profile 10k-intel --config reviews/task-67/017-intel-measurement-final/artifacts/task67-intel-10k-auto-sidecar-suite.json --suite task67-intel-10k-auto-sidecar --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/017-intel-measurement-final/artifacts/auto-sidecar/cloud-bench-auto-sidecar.log`
- S3 run:
  - `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-intel-10k-auto-sidecar/20260530T034909Z/`
- artifacts:
  - `artifacts/auto-sidecar/suite-manifest.json`
  - `artifacts/auto-sidecar/results.jsonl`
  - `artifacts/auto-sidecar/suite-run.log`
  - `artifacts/auto-sidecar/load-10k-rabitq1-auto-sidecar.log`
  - `artifacts/auto-sidecar/sidecar-10k-rabitq8-variants-auto.log`
  - `artifacts/auto-sidecar/cloud-bench-auto-sidecar.log`
- key result lines:
  - `rabitq8`: recall@10 0.9865, sidecar score p50 0.026 ms, total bound p50 3.967 ms.
  - `rabitq8ls`: recall@10 0.9835, sidecar score p50 0.024 ms, total bound p50 3.959 ms.
  - `rabitq8c3`: recall@10 0.9945, sidecar score p50 0.026 ms, total bound p50 4.002 ms.
  - `rabitq8c4`: recall@10 0.9990, sidecar score p50 0.027 ms, total bound p50 4.002 ms.
  - total-bound p50 speedup vs scalar: 2.01x, 2.02x, 2.00x, 2.00x respectively.

## Superseded Artifact

- `artifacts/scalar/cloud-bench-scalar.log`
  - First scalar attempt. It is retained for provenance. The primary load, recall, and latency completed, then sidecar failed because the primary fixture used `rerank=heap_f32`; corrected sidecar suites use `rerank=off`.
