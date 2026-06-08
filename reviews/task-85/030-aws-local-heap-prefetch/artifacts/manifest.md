# Task 85 Packet 030 Artifact Manifest

- head SHA: `6b15592cf`
- task bucket: `reviews/task-85/030-aws-local-heap-prefetch/`
- lane: AWS 1M/q500 SPIRE retained-recall latency
- fixture: `task67_1m_hnsw_m7g2xlarge`
- storage format: V5 selected row-segment locator surface from packet 021
- rerank mode: heap rerank width `25` per query, local heap block prefetch from
  packet 029
- timestamp: 2026-06-07
- isolation: one retained AWS 1M SPIRE index/table surface; no shared-table
  benchmark sweep

## Commands

Suite audit:

```bash
target/debug/ecaz bench suite audit --config reviews/task-85/030-aws-local-heap-prefetch/suite-aws-1m-local-heap-prefetch-q500.json --log-file reviews/task-85/030-aws-local-heap-prefetch/artifacts/suite-audit.log
```

AWS status before run:

```bash
target/debug/ecaz cloud status --profile 1m --log-file reviews/task-85/030-aws-local-heap-prefetch/artifacts/cloud-status-before-local-heap-prefetch.log
```

Resume AWS profile:

```bash
target/debug/ecaz cloud resume --profile 1m --log-file reviews/task-85/030-aws-local-heap-prefetch/artifacts/cloud-resume-local-heap-prefetch.log
```

Install current branch:

```bash
target/debug/ecaz cloud install --profile 1m --database postgres --git-ref task-85-spire-product-scale-pareto --skip-extension-recreate --skip-cli-build --timeout 3600 --log-file reviews/task-85/030-aws-local-heap-prefetch/artifacts/cloud-install-local-heap-prefetch.log
```

Run suite:

```bash
target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/030-aws-local-heap-prefetch/suite-aws-1m-local-heap-prefetch-q500.json --log-file reviews/task-85/030-aws-local-heap-prefetch/artifacts/cloud-bench-local-heap-prefetch-q500.log
```

Pause AWS profile:

```bash
target/debug/ecaz cloud pause --profile 1m --log-file reviews/task-85/030-aws-local-heap-prefetch/artifacts/cloud-pause-after-local-heap-prefetch-q500.log
```

Final AWS status:

```bash
target/debug/ecaz cloud status --profile 1m --log-file reviews/task-85/030-aws-local-heap-prefetch/artifacts/cloud-status-final-after-local-heap-prefetch-q500-paused.log
```

## Key Results

Corrected suite report:

- `reviews/task-85/030-aws-local-heap-prefetch/artifacts/aws-1m-local-heap-prefetch-q500/suite-report.md`
- steps completed: `3`
- failed: `0`

First run:

- `recall@10=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- `latency_p50=244.557 ms`
- `latency_p95=312.621 ms`
- `latency_p99=2563.915 ms`
- `latency_max=27325.150 ms`

Repeat run:

- `recall@10=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- `latency_p50=227.414 ms`
- `latency_p95=282.375 ms`
- `latency_p99=297.652 ms`
- `latency_max=348.608 ms`

Repeat funnel summary:

- object-read p50/p95: `27.036747/28.276362 ms`
- candidate-score p50/p95: `57.470473/59.307596 ms`
- summary-score p50/p95: `47.367637/49.191821 ms`
- row-score p50/p95: `10.067289/10.193761 ms`
- rerank-prefix rows: `12,500`
- unique heap blocks p50/p95/max: `22/25/25`
- heap-block transitions p50/p95/max: `24/24/24`

Decision:

- reject packet 029's local heap prefetch sublever because it preserves
  recall/candidates/rerank width but worsens p50, p95, and p99 latency versus
  packet 023/025 accepted controls.

Final AWS status:

- `profile: 1m`
- `state: paused`
- `cost: ~$0.00/hr running, ~$8.00/mo retained storage`
