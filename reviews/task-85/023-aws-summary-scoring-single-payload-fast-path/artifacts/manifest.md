# Task 85 Packet 023 Artifact Manifest

- head SHA: `6f6afd61d1869ab764fc57645e62f84debee4f0b`
- code checkpoint under measurement: `f90c8202e0f79fc2df8e5ff2763d1fd856b427d3`
- task bucket: `reviews/task-85/`
- packet path: `reviews/task-85/023-aws-summary-scoring-single-payload-fast-path/`
- lane: AWS 1M/q500
- fixture: `task67_1m_hnsw_m7g2xlarge`
- storage format: RaBitQ
- rerank mode: retained `rerank_width=25`
- index: `aws_spire_1m_rabitq_t85_v5_block16_tg256_idx`
- surface isolation: shared corpus table, existing one-index-per-profile index
  surface
- timestamp: 2026-06-07

## Commands

Audit:

```text
target/debug/ecaz bench suite audit --config reviews/task-85/023-aws-summary-scoring-single-payload-fast-path/suite-aws-1m-summary-scoring-single-payload-fast-path-q500.json --log-file reviews/task-85/023-aws-summary-scoring-single-payload-fast-path/artifacts/suite-audit.log
```

AWS resume:

```text
target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-85/023-aws-summary-scoring-single-payload-fast-path/artifacts/cloud-resume-summary-fast-path.log
```

AWS install:

```text
target/debug/ecaz cloud install --profile 1m --database postgres --git-ref task-85-spire-product-scale-pareto --skip-extension-recreate --timeout 3600 --log-file reviews/task-85/023-aws-summary-scoring-single-payload-fast-path/artifacts/cloud-install-summary-fast-path.log
```

AWS suite:

```text
target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/023-aws-summary-scoring-single-payload-fast-path/suite-aws-1m-summary-scoring-single-payload-fast-path-q500.json --suite task85-aws-1m-summary-scoring-single-payload-fast-path-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/023-aws-summary-scoring-single-payload-fast-path/artifacts/cloud-bench-summary-fast-path-q500.log
```

AWS pause and final status:

```text
target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-85/023-aws-summary-scoring-single-payload-fast-path/artifacts/cloud-pause-after-summary-fast-path-q500.log
script -q -c 'target/debug/ecaz cloud status --profile 1m --database postgres' reviews/task-85/023-aws-summary-scoring-single-payload-fast-path/artifacts/cloud-status-final-after-summary-fast-path-q500-paused.log
```

Suite report:

```text
target/debug/ecaz bench suite report --manifest reviews/task-85/023-aws-summary-scoring-single-payload-fast-path/artifacts/aws-1m-summary-scoring-single-payload-fast-path-q500/suite-manifest.json --results-output reviews/task-85/023-aws-summary-scoring-single-payload-fast-path/artifacts/aws-1m-summary-scoring-single-payload-fast-path-q500/results-report.jsonl --log-file reviews/task-85/023-aws-summary-scoring-single-payload-fast-path/artifacts/aws-1m-summary-scoring-single-payload-fast-path-q500/suite-report.md
```

## Artifacts

- `suite-aws-1m-summary-scoring-single-payload-fast-path-q500.json`: checked-in
  SuiteConfig.
- `artifacts/suite-audit.log`: suite audit; key line:
  `[suite:task85-aws-1m-summary-scoring-single-payload-fast-path-q500] audit passed: 4 steps`.
- `artifacts/cloud-resume-summary-fast-path.log`: AWS resume log.
- `artifacts/cloud-install-summary-fast-path.log`: AWS install log; key line:
  `install: profile=1m db=10.42.1.131 ref=task-85-spire-product-scale-pareto ok`.
- `artifacts/cloud-bench-summary-fast-path-q500.log`: cloud bench wrapper log.
- `artifacts/aws-1m-summary-scoring-single-payload-fast-path-q500/suite-manifest.json`:
  suite manifest.
- `artifacts/aws-1m-summary-scoring-single-payload-fast-path-q500/suite-run.log`:
  suite run log.
- `artifacts/aws-1m-summary-scoring-single-payload-fast-path-q500/results.jsonl`:
  parsed suite results.
- `artifacts/aws-1m-summary-scoring-single-payload-fast-path-q500/results-report.jsonl`:
  report output.
- `artifacts/aws-1m-summary-scoring-single-payload-fast-path-q500/suite-report.md`:
  generated suite report.
- `artifacts/aws-1m-summary-scoring-single-payload-fast-path-q500/pipeline-v5-summary-fast-path-q500-first.log`:
  first pipeline pass.
- `artifacts/aws-1m-summary-scoring-single-payload-fast-path-q500/pipeline-v5-summary-fast-path-q500-repeat.log`:
  warm repeat pipeline pass.
- `artifacts/aws-1m-summary-scoring-single-payload-fast-path-q500/funnel-v5-summary-fast-path-q500-first.jsonl`:
  first funnel counters.
- `artifacts/aws-1m-summary-scoring-single-payload-fast-path-q500/funnel-v5-summary-fast-path-q500-repeat.jsonl`:
  repeat funnel counters.
- `artifacts/aws-1m-summary-scoring-single-payload-fast-path-q500/miss-attribution-v5-summary-fast-path-q500-first.jsonl`:
  first miss-attribution output.
- `artifacts/aws-1m-summary-scoring-single-payload-fast-path-q500/miss-attribution-v5-summary-fast-path-q500-repeat.jsonl`:
  repeat miss-attribution output.
- `artifacts/aws-1m-summary-scoring-single-payload-fast-path-q500/storage-spire-1m-rabitq-v5-summary-fast-path.log`:
  storage report.
- `artifacts/cloud-pause-after-summary-fast-path-q500.log`: pause log.
- `artifacts/cloud-status-final-after-summary-fast-path-q500.log`: first final
  status capture, caught `state: stopping`.
- `artifacts/cloud-status-final-after-summary-fast-path-q500-paused.log`: final
  status capture; key line: `state:    paused`.

## Key Results

First run:

- `recall@10=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- `latency_p50=245.446 ms`
- `latency_p95=310.607 ms`
- `latency_p99=2558.302 ms`
- `latency_max=27342.467 ms`

Repeat run:

- `recall@10=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- `latency_p50=222.692 ms`
- `latency_p95=275.769 ms`
- `latency_p99=286.980 ms`
- `latency_max=296.157 ms`
- object-read p50/p95: `26.635/27.635 ms`
- candidate-score p50/p95: `56.327/58.003 ms`
- summary-score p50/p95: `46.270/47.924 ms`
- row-score p50/p95: `10.063/10.168 ms`

Decision: accepted as the current retained-recall product candidate because
packet 023 repeat beats packet 019 retained repeat
(`227.388/284.166/297.164 ms` p50/p95/p99) at unchanged recall, candidates,
and heap rerank count.
