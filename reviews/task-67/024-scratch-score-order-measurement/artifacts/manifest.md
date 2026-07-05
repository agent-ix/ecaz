# Task 67 Packet 024 Artifact Manifest

- Head SHA: `db821441e74578d25e0c8ef89395c9b0c2f06e0e`
- Code under test: `34da0492b Order ec_ivf scratch batch candidates by score`
- Measurement base before installing the experimental ref:
  `d8b91e4229be362d3e8a2f4a97962e4cf9767513`
- Task bucket: `reviews/task-67/`
- Packet path: `reviews/task-67/024-scratch-score-order-measurement/`
- Timestamp: `2026-05-30T07:21:05Z`
- Lane: AWS Intel real-10k primary bits=1 `ec_ivf` measurement with
  `ec_ivf.scratch_soa_batch_decode=on`, after scratch batch score-ordering
  experiment
- Fixture: `target/real-corpus/staged-task50/ec_real_10k_corpus.tsv`,
  `target/real-corpus/staged-task50/ec_real_10k_queries.tsv`, 200 queries
- Storage format: isolated one-index-per-table `rabitq`, `quant_bits=1`
- Rerank mode: `heap_f32`, `rerank_width=100`
- Surface isolation: scalar and auto lanes use separate prefixes:
  `task67_order_10k_rabitq1_scalar` and `task67_order_10k_rabitq1_auto`

## Code Change Under Test

- Commit: `34da0492b Order ec_ivf scratch batch candidates by score`
- Touched files:
  - `src/am/ec_ivf/scan.rs`
  - `crates/ecaz-cli/src/commands/bench/rabitq_kernel.rs` (format-only drift
    from `cargo fmt`)
- Result: negative measurement; score-ordering is reverted after this packet.

## Local Validation

### Focused unit tests

- Command:
  `cargo test -p ecaz posting_scratch_soa_sorts_score_indices_best_first_stably`
- Result: passed.
- Command:
  `cargo test -p ecaz candidate_top_k_rejects_only_scores_worse_than_full_worst`
- Result: passed.

### Format check

- Command: `cargo fmt --check`
- Result: passed.
- Note: stable rustfmt emitted existing warnings about nightly-only
  `imports_granularity` and `group_imports` options.

## Suite Configs And Audits

### `artifacts/task67-score-order-bits1-scalar-suite.json`

- Command:
  `target/debug/ecaz bench suite audit --config reviews/task-67/024-scratch-score-order-measurement/artifacts/task67-score-order-bits1-scalar-suite.json`
- Artifact: `artifacts/local/suite-audit-scalar.log`
- Result: `audit passed: 3 steps`

### `artifacts/task67-score-order-bits1-auto-suite.json`

- Command:
  `target/debug/ecaz bench suite audit --config reviews/task-67/024-scratch-score-order-measurement/artifacts/task67-score-order-bits1-auto-suite.json`
- Artifact: `artifacts/local/suite-audit-auto.log`
- Result: `audit passed: 3 steps`

## AWS Commands

### Resume

- Command:
  `target/debug/ecaz cloud resume --profile 10k-intel --log-file reviews/task-67/024-scratch-score-order-measurement/artifacts/preflight/cloud-resume.log`
- Result: `resume: profile=10k-intel db=10.42.1.147 ready`

### Install

- Command:
  `target/debug/ecaz cloud install --profile 10k-intel --git-ref 34da0492b --skip-extension-recreate --database postgres --timeout 3600 --log-file reviews/task-67/024-scratch-score-order-measurement/artifacts/preflight/cloud-install-34da0492b.log`
- Result: `install: profile=10k-intel db=10.42.1.147 ref=34da0492b ok`

### Scalar Score-Order Suite

- Command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode scalar --config reviews/task-67/024-scratch-score-order-measurement/artifacts/task67-score-order-bits1-scalar-suite.json --suite task67-score-order-bits1-scalar --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/024-scratch-score-order-measurement/artifacts/scalar/cloud-bench-score-order-scalar.log`
- S3 run:
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-score-order-bits1-scalar/20260530T071900Z/`
- Result: passed and synced artifacts.

### Auto Score-Order Suite

- Command:
  `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode auto --config reviews/task-67/024-scratch-score-order-measurement/artifacts/task67-score-order-bits1-auto-suite.json --suite task67-score-order-bits1-auto --database postgres --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-67/024-scratch-score-order-measurement/artifacts/auto/cloud-bench-score-order-auto.log`
- S3 run:
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-score-order-bits1-auto/20260530T071927Z/`
- Result: passed and synced artifacts.

### Pause

- Command:
  `target/debug/ecaz cloud pause --profile 10k-intel --log-file reviews/task-67/024-scratch-score-order-measurement/artifacts/preflight/cloud-pause.log`
- Final status command:
  `target/debug/ecaz cloud status --profile 10k-intel`
- Final status artifact:
  `artifacts/preflight/cloud-status-after-pause.log`
- Result: `state: paused`, `~$0.00/hr running`, retained storage `~$8.00/mo`

## Key Result Lines

Recall:

| lane | nprobe | recall@10 | mean q-time |
| --- | ---: | ---: | ---: |
| scalar | 16 | 0.9985 | 1.46 ms |
| scalar | 32 | 1.0000 | 1.87 ms |
| scalar | 64 | 1.0000 | 2.81 ms |
| auto | 16 | 0.9985 | 1.35 ms |
| auto | 32 | 1.0000 | 1.68 ms |
| auto | 64 | 1.0000 | 2.47 ms |

Latency:

| lane | nprobe | mean | p50 | p95 |
| --- | ---: | ---: | ---: | ---: |
| scalar | 16 | 1.18 ms | 1.17 ms | 1.45 ms |
| scalar | 32 | 1.60 ms | 1.60 ms | 1.77 ms |
| scalar | 64 | 2.23 ms | 2.22 ms | 2.42 ms |
| auto | 16 | 1.14 ms | 1.12 ms | 1.34 ms |
| auto | 32 | 1.64 ms | 1.63 ms | 1.86 ms |
| auto | 64 | 2.35 ms | 2.33 ms | 2.60 ms |

Speedup versus packet 017 no-scratch scalar baseline:

| nprobe | packet 017 scalar mean | packet 024 auto mean | speedup |
| --- | ---: | ---: | ---: |
| 16 | 2.28 ms | 1.14 ms | 2.00x |
| 32 | 3.70 ms | 1.64 ms | 2.26x |
| 64 | 6.57 ms | 2.35 ms | 2.80x |

## Limitation

This packet documents a failed optimization experiment. Packet 022 remains the
best current SQL-level bits=1 measurement.
