# Task 67 Packet 021 Artifact Manifest

- Head SHA: `1e12293c1ae1eb82d939ff390f8a87789f269c50`
- Code under test: `/usr/local/bin/ecaz` on AWS `10k-intel`, installed from the
  packet 020 code line (`5df1308d40bda38d1da65f2325bab32e48fdf10b`); current
  head only adds packet metadata after that code commit.
- Task bucket: `reviews/task-67/`
- Packet path: `reviews/task-67/021-scratch-soa-bits1-measurement/`
- Timestamp: `2026-05-30T06:23:25Z`
- Lane: AWS Intel real-10k primary bits=1 `ec_ivf` measurement with
  `ec_ivf.scratch_soa_batch_decode=on`
- Fixture: `target/real-corpus/staged-task50/ec_real_10k_corpus.tsv`,
  `target/real-corpus/staged-task50/ec_real_10k_queries.tsv`, 200 queries
- Storage format: isolated one-index-per-table `rabitq`, `quant_bits=1`
- Rerank mode: `heap_f32`, `rerank_width=100`
- Surface isolation: scalar and auto lanes use separate prefixes:
  `task67_scratch_10k_rabitq1_scalar` and
  `task67_scratch_10k_rabitq1_auto`

## Suite Configs And Audits

### `artifacts/task67-scratch-soa-bits1-scalar-suite.json`

- Command:
  `target/debug/ecaz bench suite audit --config reviews/task-67/021-scratch-soa-bits1-measurement/artifacts/task67-scratch-soa-bits1-scalar-suite.json`
- Artifact: `artifacts/local/suite-audit-scalar.log`
- Result: `audit passed: 3 steps`

### `artifacts/task67-scratch-soa-bits1-auto-suite.json`

- Command:
  `target/debug/ecaz bench suite audit --config reviews/task-67/021-scratch-soa-bits1-measurement/artifacts/task67-scratch-soa-bits1-auto-suite.json`
- Artifact: `artifacts/local/suite-audit-auto.log`
- Result: `audit passed: 3 steps`

## AWS Commands

### Resume

- Command:
  `target/debug/ecaz cloud resume --profile 10k-intel --log-file reviews/task-67/021-scratch-soa-bits1-measurement/artifacts/preflight/cloud-resume.log`
- Result: `resume: profile=10k-intel db=10.42.1.147 ready`

### Scalar Scratch-SoA Suite

- Command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode scalar --config reviews/task-67/021-scratch-soa-bits1-measurement/artifacts/task67-scratch-soa-bits1-scalar-suite.json --suite task67-scratch-soa-bits1-scalar --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/021-scratch-soa-bits1-measurement/artifacts/scalar/cloud-bench-scratch-soa-scalar.log`
- S3 run:
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-scratch-soa-bits1-scalar/20260530T061925Z/`
- Result: passed and synced artifacts.
- Suite log: `artifacts/scalar/suite-run.log`
- Structured results: `artifacts/scalar/results.jsonl`

### Auto Scratch-SoA Suite

- Command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode auto --config reviews/task-67/021-scratch-soa-bits1-measurement/artifacts/task67-scratch-soa-bits1-auto-suite.json --suite task67-scratch-soa-bits1-auto --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/021-scratch-soa-bits1-measurement/artifacts/auto/cloud-bench-scratch-soa-auto.log`
- S3 run:
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-scratch-soa-bits1-auto/20260530T062009Z/`
- Result: passed and synced artifacts.
- Suite log: `artifacts/auto/suite-run.log`
- Structured results: `artifacts/auto/results.jsonl`

### Pause

- Command:
  `target/debug/ecaz cloud pause --profile 10k-intel --log-file reviews/task-67/021-scratch-soa-bits1-measurement/artifacts/preflight/cloud-pause.log`
- Final status command:
  `target/debug/ecaz cloud status --profile 10k-intel`
- Final status artifact:
  `artifacts/preflight/cloud-status-after-pause.log`
- Result: `state: paused`, `~$0.00/hr running`, retained storage `~$8.00/mo`

## Key Result Lines

Recall is unchanged between scratch-SoA scalar and auto:

| lane | nprobe | recall@10 | mean q-time |
| --- | ---: | ---: | ---: |
| scalar | 16 | 0.9985 | 1.33 ms |
| scalar | 32 | 1.0000 | 1.54 ms |
| scalar | 64 | 1.0000 | 2.25 ms |
| auto | 16 | 0.9985 | 1.47 ms |
| auto | 32 | 1.0000 | 1.90 ms |
| auto | 64 | 1.0000 | 2.62 ms |

Latency:

| lane | nprobe | mean | p50 | p95 |
| --- | ---: | ---: | ---: | ---: |
| scalar | 16 | 1.13 ms | 1.10 ms | 1.38 ms |
| scalar | 32 | 1.53 ms | 1.52 ms | 1.74 ms |
| scalar | 64 | 2.33 ms | 2.30 ms | 2.56 ms |
| auto | 16 | 1.15 ms | 1.13 ms | 1.38 ms |
| auto | 32 | 1.45 ms | 1.43 ms | 1.60 ms |
| auto | 64 | 2.30 ms | 2.27 ms | 2.56 ms |

Speedup versus packet 017 no-scratch scalar baseline:

| nprobe | packet 017 scalar mean | packet 021 auto mean | speedup |
| --- | ---: | ---: | ---: |
| 16 | 2.28 ms | 1.15 ms | 1.98x |
| 32 | 3.70 ms | 1.45 ms | 2.55x |
| 64 | 6.57 ms | 2.30 ms | 2.86x |

## Limitation

This packet does not close Task 67's performance gate. It narrows the remaining
gap to scan/query-path overhead above the already-fast AVX-512 kernel measured
in packet 020.
