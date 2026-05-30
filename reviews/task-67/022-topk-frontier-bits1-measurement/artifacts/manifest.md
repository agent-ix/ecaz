# Task 67 Packet 022 Artifact Manifest

- Head SHA: `a0cb83cb3bf4a1404e39b8261820f92dcf34b8ab`
- Task bucket: `reviews/task-67/`
- Packet path: `reviews/task-67/022-topk-frontier-bits1-measurement/`
- Timestamp: `2026-05-30T06:43:11Z`
- Lane: AWS Intel real-10k primary bits=1 `ec_ivf` measurement with
  `ec_ivf.scratch_soa_batch_decode=on`, after top-K frontier candidate
  rejection
- Fixture: `target/real-corpus/staged-task50/ec_real_10k_corpus.tsv`,
  `target/real-corpus/staged-task50/ec_real_10k_queries.tsv`, 200 queries
- Storage format: isolated one-index-per-table `rabitq`, `quant_bits=1`
- Rerank mode: `heap_f32`, `rerank_width=100`
- Surface isolation: scalar and auto lanes use separate prefixes:
  `task67_topk_10k_rabitq1_scalar` and `task67_topk_10k_rabitq1_auto`

## Code Change

- Commit: `a0cb83cb3 Skip ec_ivf candidates below full top-k frontier`
- Touched file: `src/am/ec_ivf/scan.rs`
- Change: when the pre-rerank top-K frontier is full, skip dedup-map candidate
  insertion for scores strictly worse than the current worst retained score.
  Equal scores are retained for existing deterministic heap-TID tie handling.

## Local Validation

### Focused unit test

- Command:
  `cargo test -p ecaz candidate_top_k_rejects_only_scores_worse_than_full_worst`
- Result: passed.
- Key line:
  `test am::ec_ivf::scan::tests::candidate_top_k_rejects_only_scores_worse_than_full_worst ... ok`

### Format check

- Command: `cargo fmt --check`
- Result: passed.
- Note: stable rustfmt emitted existing warnings about nightly-only
  `imports_granularity` and `group_imports` options.

## Suite Configs And Audits

### `artifacts/task67-topk-frontier-bits1-scalar-suite.json`

- Command:
  `target/debug/ecaz bench suite audit --config reviews/task-67/022-topk-frontier-bits1-measurement/artifacts/task67-topk-frontier-bits1-scalar-suite.json`
- Artifact: `artifacts/local/suite-audit-scalar.log`
- Result: `audit passed: 3 steps`

### `artifacts/task67-topk-frontier-bits1-auto-suite.json`

- Command:
  `target/debug/ecaz bench suite audit --config reviews/task-67/022-topk-frontier-bits1-measurement/artifacts/task67-topk-frontier-bits1-auto-suite.json`
- Artifact: `artifacts/local/suite-audit-auto.log`
- Result: `audit passed: 3 steps`

## AWS Commands

### Resume

- Command:
  `target/debug/ecaz cloud resume --profile 10k-intel --log-file reviews/task-67/022-topk-frontier-bits1-measurement/artifacts/preflight/cloud-resume.log`
- Result: `resume: profile=10k-intel db=10.42.1.147 ready`

### Install

- Command:
  `target/debug/ecaz cloud install --profile 10k-intel --git-ref a0cb83cb3 --skip-extension-recreate --database postgres --timeout 3600 --log-file reviews/task-67/022-topk-frontier-bits1-measurement/artifacts/preflight/cloud-install-a0cb83cb3.log`
- Result: `install: profile=10k-intel db=10.42.1.147 ref=a0cb83cb3 ok`

### Scalar Scratch-SoA Suite

- Command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode scalar --config reviews/task-67/022-topk-frontier-bits1-measurement/artifacts/task67-topk-frontier-bits1-scalar-suite.json --suite task67-topk-frontier-bits1-scalar --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/022-topk-frontier-bits1-measurement/artifacts/scalar/cloud-bench-topk-frontier-scalar.log`
- S3 run:
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-topk-frontier-bits1-scalar/20260530T064122Z/`
- Result: passed and synced artifacts.
- Suite log: `artifacts/scalar/suite-run.log`
- Structured results: `artifacts/scalar/results.jsonl`

### Auto Scratch-SoA Suite

- Command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode auto --config reviews/task-67/022-topk-frontier-bits1-measurement/artifacts/task67-topk-frontier-bits1-auto-suite.json --suite task67-topk-frontier-bits1-auto --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/022-topk-frontier-bits1-measurement/artifacts/auto/cloud-bench-topk-frontier-auto.log`
- S3 run:
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-topk-frontier-bits1-auto/20260530T064149Z/`
- Result: passed and synced artifacts.
- Suite log: `artifacts/auto/suite-run.log`
- Structured results: `artifacts/auto/results.jsonl`

### Pause

- Command:
  `target/debug/ecaz cloud pause --profile 10k-intel --log-file reviews/task-67/022-topk-frontier-bits1-measurement/artifacts/preflight/cloud-pause.log`
- Final status command:
  `target/debug/ecaz cloud status --profile 10k-intel`
- Final status artifact:
  `artifacts/preflight/cloud-status-after-pause.log`
- Result: `state: paused`, `~$0.00/hr running`, retained storage `~$8.00/mo`

## Key Result Lines

Recall is unchanged between packet 022 scalar and auto:

| lane | nprobe | recall@10 | mean q-time |
| --- | ---: | ---: | ---: |
| scalar | 16 | 0.9985 | 1.29 ms |
| scalar | 32 | 1.0000 | 1.49 ms |
| scalar | 64 | 1.0000 | 2.25 ms |
| auto | 16 | 0.9985 | 1.42 ms |
| auto | 32 | 1.0000 | 1.75 ms |
| auto | 64 | 1.0000 | 2.38 ms |

Latency:

| lane | nprobe | mean | p50 | p95 |
| --- | ---: | ---: | ---: | ---: |
| scalar | 16 | 1.07 ms | 1.05 ms | 1.30 ms |
| scalar | 32 | 1.48 ms | 1.45 ms | 1.68 ms |
| scalar | 64 | 2.20 ms | 2.19 ms | 2.43 ms |
| auto | 16 | 1.08 ms | 1.06 ms | 1.31 ms |
| auto | 32 | 1.47 ms | 1.46 ms | 1.64 ms |
| auto | 64 | 2.14 ms | 2.12 ms | 2.37 ms |

Speedup versus packet 017 no-scratch scalar baseline:

| nprobe | packet 017 scalar mean | packet 022 auto mean | speedup |
| --- | ---: | ---: | ---: |
| 16 | 2.28 ms | 1.08 ms | 2.11x |
| 32 | 3.70 ms | 1.47 ms | 2.52x |
| 64 | 6.57 ms | 2.14 ms | 3.07x |

## Limitation

This packet improves the SQL-level bits=1 lane but does not prove that every
measured nprobe point satisfies a strict total wall-time 3x interpretation.
