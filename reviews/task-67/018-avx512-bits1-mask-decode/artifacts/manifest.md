# Task 67 Packet 018 Artifact Manifest

- Head SHA: `561a9a818`
- Code-under-test SHA: `327e083ca` installed on the AWS DB host; includes
  RaBitQ mask-decode code from `9989a1d87`.
- Task bucket: `reviews/task-67/`
- Packet path: `reviews/task-67/018-avx512-bits1-mask-decode/`
- Timestamp: `2026-05-30T04:44Z`
- Lane: AWS Intel AVX-512 10k real-corpus bits=1 primary measurement
- Fixture: `ec_real_10k`, 200 queries, nprobe sweep `16,32,64`
- Storage format: `rabitq`, `quant_bits=1`, `rerank=heap_f32`,
  `rerank_width=100`
- Rerank mode: heap_f32 rerank for primary recall/latency
- Isolated one-index-per-table surfaces: yes, scalar and auto use distinct
  table prefixes.

## Code and Operator Changes

- `9989a1d87 Replace bits1 byte sums with AVX512 mask decode`
  - Reverts the query-byte-sum table path.
  - Uses AVX-512 mask blends for bits=1 single and paired dequant vectors.
- `327e083ca Repair cloud install build ownership`
  - Repairs `/var/lib/pgsql/build` ownership before remote install.
- `561a9a818 Add cloud bench SIMD mode restart`
  - Adds `ecaz cloud bench --simd-mode` for packet-local scalar/auto restarts.

## Local Validation

- Command: `cargo test -p ecaz quant::rabitq -- --nocapture`
- Result: passed locally, 46 tests, 0 failed.
- Artifact: `artifacts/preflight/validation.log`

## Suite Config Audits

- Command:
  `target/debug/ecaz bench suite audit --config reviews/task-67/018-avx512-bits1-mask-decode/artifacts/task67-bits1-mask-scalar-suite.json`
- Result: `[suite:task67-bits1-mask-scalar] audit passed: 3 steps`
- Command:
  `target/debug/ecaz bench suite audit --config reviews/task-67/018-avx512-bits1-mask-decode/artifacts/task67-bits1-mask-auto-suite.json`
- Result: `[suite:task67-bits1-mask-auto] audit passed: 3 steps`
- Artifact: `artifacts/preflight/validation.log`

## Cloud Install

- Command:
  `target/debug/ecaz cloud install --profile 10k-intel --git-ref 327e083ca --skip-extension-recreate --database postgres --timeout 3600 --log-file reviews/task-67/018-avx512-bits1-mask-decode/artifacts/preflight/cloud-install-327e083ca.log`
- Result: `install: profile=10k-intel db=10.42.1.147 ref=327e083ca ok`
- Artifact:
  `artifacts/preflight/cloud-install-327e083ca.log`

## Scalar Primary Suite

- Command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode scalar --config reviews/task-67/018-avx512-bits1-mask-decode/artifacts/task67-bits1-mask-scalar-suite.json --suite task67-bits1-mask-scalar --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/018-avx512-bits1-mask-decode/artifacts/scalar/cloud-bench-mask-scalar.log`
- S3 run:
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-bits1-mask-scalar/20260530T044348Z/`
- Artifacts:
  - `artifacts/scalar/cloud-bench-mask-scalar.log`
  - `artifacts/scalar/suite-run.log`
  - `artifacts/scalar/suite-manifest.json`
  - `artifacts/scalar/results.jsonl`
  - `artifacts/scalar/load-10k-rabitq1-scalar.log`
  - `artifacts/scalar/recall-10k-rabitq1-scalar.log`
  - `artifacts/scalar/latency-10k-rabitq1-scalar.log`
- Key results:
  - recall@10: nprobe 16 = 0.9985, nprobe 32 = 1.0000,
    nprobe 64 = 1.0000.
  - latency mean: nprobe 16 = 2.59 ms, nprobe 32 = 4.04 ms,
    nprobe 64 = 7.00 ms.

## Auto Primary Suite

- Command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode auto --config reviews/task-67/018-avx512-bits1-mask-decode/artifacts/task67-bits1-mask-auto-suite.json --suite task67-bits1-mask-auto --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/018-avx512-bits1-mask-decode/artifacts/auto/cloud-bench-mask-auto.log`
- S3 run:
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-bits1-mask-auto/20260530T044416Z/`
- Artifacts:
  - `artifacts/auto/cloud-bench-mask-auto.log`
  - `artifacts/auto/suite-run.log`
  - `artifacts/auto/suite-manifest.json`
  - `artifacts/auto/results.jsonl`
  - `artifacts/auto/load-10k-rabitq1-auto.log`
  - `artifacts/auto/recall-10k-rabitq1-auto.log`
  - `artifacts/auto/latency-10k-rabitq1-auto.log`
- Key results:
  - recall@10: nprobe 16 = 0.9985, nprobe 32 = 1.0000,
    nprobe 64 = 1.0000.
  - latency mean: nprobe 16 = 1.48 ms, nprobe 32 = 1.95 ms,
    nprobe 64 = 2.80 ms.
  - speedup vs scalar mean: nprobe 16 = 1.75x, nprobe 32 = 2.07x,
    nprobe 64 = 2.50x.

## Limitations

- The Task 67 primary bits=1 speed gate is still not satisfied.
- Sidecar variants were not rerun because the primary gate missed.
- Earlier untracked files in this packet directory document a superseded
  query-byte-sum attempt and are not cited as review evidence for this request.

## Cloud Pause

- Command:
  `target/debug/ecaz cloud pause --profile 10k-intel --log-file reviews/task-67/018-avx512-bits1-mask-decode/artifacts/preflight/cloud-pause.log`
- Result: `pause: profile=10k-intel stopped (db + loader)`
- Follow-up status: `state: paused`, running cost `$0.00/hr`.
- Artifact: `artifacts/preflight/cloud-pause.log`
