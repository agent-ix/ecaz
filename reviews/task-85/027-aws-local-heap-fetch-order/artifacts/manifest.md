# Task 85 Packet 027 Artifact Manifest

- head SHA: `90065874e8f5452b6da4531b2dcc8e2f22ca36c2`
- task bucket: `reviews/task-85/027-aws-local-heap-fetch-order/`
- lane: AWS 1M/q500 SPIRE retained-recall latency
- fixture: `task67_1m_hnsw_m7g2xlarge`
- storage format: V5 selected row-segment locator surface from packet 021
- rerank mode: heap rerank width `25` per query, local heap fetch order changed
  by packet 026 before source-vector scoring
- timestamp: 2026-06-07
- isolation: one retained AWS 1M SPIRE index/table surface; no shared-table
  benchmark sweep

## Commands

Resume AWS profile:

```bash
target/debug/ecaz cloud resume --profile 1m --log-file reviews/task-85/027-aws-local-heap-fetch-order/artifacts/cloud-resume-local-heap-fetch-order.log
```

Install branch without rebuilding the retained extension/index surface:

```bash
target/debug/ecaz cloud install --profile 1m --database postgres --git-ref task-85-spire-product-scale-pareto --skip-extension-recreate --skip-cli-build --timeout 3600 --log-file reviews/task-85/027-aws-local-heap-fetch-order/artifacts/cloud-install-local-heap-fetch-order.log
```

Audit corrected SuiteConfig:

```bash
target/debug/ecaz bench suite --config reviews/task-85/027-aws-local-heap-fetch-order/suite-aws-1m-local-heap-fetch-order-q500.json --artifact-dir reviews/task-85/027-aws-local-heap-fetch-order/artifacts/aws-1m-local-heap-fetch-order-q500 --audit --log-file reviews/task-85/027-aws-local-heap-fetch-order/artifacts/suite-audit-rerun.log
```

Run corrected SuiteConfig on AWS:

```bash
target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/027-aws-local-heap-fetch-order/suite-aws-1m-local-heap-fetch-order-q500.json --artifact-dir reviews/task-85/027-aws-local-heap-fetch-order/artifacts/aws-1m-local-heap-fetch-order-q500 --timeout 3600 --log-file reviews/task-85/027-aws-local-heap-fetch-order/artifacts/cloud-bench-local-heap-fetch-order-q500-rerun.log
```

Pause AWS profile:

```bash
target/debug/ecaz cloud pause --profile 1m --log-file reviews/task-85/027-aws-local-heap-fetch-order/artifacts/cloud-pause-after-local-heap-fetch-order-q500.log
```

Final AWS status:

```bash
target/debug/ecaz cloud status --profile 1m --log-file reviews/task-85/027-aws-local-heap-fetch-order/artifacts/cloud-status-final-after-local-heap-fetch-order-q500-paused.log
```

## Key Results

Corrected suite report:

- `reviews/task-85/027-aws-local-heap-fetch-order/artifacts/aws-1m-local-heap-fetch-order-q500/suite-report.md`
- steps completed: `3`
- failed: `0`

First run:

- `recall@10=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- `latency_p50=251.481 ms`
- `latency_p95=322.091 ms`
- `latency_p99=2567.680 ms`
- `latency_max=27360.685 ms`

Repeat run:

- `recall@10=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- `latency_p50=228.595 ms`
- `latency_p95=284.140 ms`
- `latency_p99=295.823 ms`
- `latency_max=301.065 ms`

Repeat locality summary:

- `rerank_prefix_sum=12,500`
- unique heap blocks p50/p95/max: `22/25/25`
- heap-block transitions p50/p95/max: `24/24/24`
- span p50/p95/max: `8366/8993/9235`
- max jump p50/p95/max: `7533/8766/9225`
- object-read p50/p95: `27.294221/28.383054 ms`
- candidate-score p50/p95: `56.782346/58.461182 ms`
- summary-score p50/p95: `46.334809/47.982170 ms`
- row-score p50/p95: `10.447833/10.546452 ms`

Control comparison from packet 025 repeat:

- `recall@10=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- `latency_p50=222.140 ms`
- `latency_p95=275.753 ms`
- `latency_p99=288.894 ms`
- `latency_max=296.358 ms`

Decision:

- reject packet 026's TID-ordered local heap fetch sublever because it preserves
  recall/candidates/rerank width but worsens p50, p95, and p99 latency.

Final AWS status:

- `profile: 1m`
- `state: paused`
- `cost: ~$0.00/hr running, ~$8.00/mo retained storage`
