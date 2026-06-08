# Task 85 Packet 010 Artifact Manifest

- head SHA: `f091b5c8ec8efed2a8f412d5a24f635111ebee9b`
- task bucket: `reviews/task-85/`
- packet path: `reviews/task-85/010-aws-retained-funnel-breakdown/`
- timestamp: `2026-06-07T15:41:13Z`
- lane: AWS 1M retained SPIRE funnel
- fixture: `task67_1m_hnsw_m7g2xlarge`
- storage format: SPIRE retained `block16`, topgraph `256`, global selected block cap `1152`
- rerank mode: heap rerank width `25/query`, `heap_rerank_sum=12,500`
- isolated/shared surface: existing AWS 1M retained surface, shared benchmark database

## Commands

```sh
target/debug/ecaz bench suite audit --config reviews/task-85/010-aws-retained-funnel-breakdown/suite-aws-1m-retained-funnel-breakdown-q500.json --log-file reviews/task-85/010-aws-retained-funnel-breakdown/artifacts/suite-audit.log
```

```sh
target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-85/010-aws-retained-funnel-breakdown/artifacts/cloud-resume-before-retained-funnel.log
```

```sh
target/debug/ecaz cloud install --profile 1m --database postgres --git-ref task-85-spire-product-scale-pareto --skip-extension-recreate --log-file reviews/task-85/010-aws-retained-funnel-breakdown/artifacts/cloud-install-retained-funnel.log
```

```sh
target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/010-aws-retained-funnel-breakdown/suite-aws-1m-retained-funnel-breakdown-q500.json --suite task85-aws-1m-retained-funnel-breakdown-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/010-aws-retained-funnel-breakdown/artifacts/cloud-bench-retained-funnel.log
```

```sh
target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-85/010-aws-retained-funnel-breakdown/artifacts/cloud-pause-after-retained-funnel.log
```

```sh
target/debug/ecaz bench suite report --manifest reviews/task-85/010-aws-retained-funnel-breakdown/artifacts/aws-1m-retained-funnel-breakdown-q500/suite-manifest.json --results-output reviews/task-85/010-aws-retained-funnel-breakdown/artifacts/aws-1m-retained-funnel-breakdown-q500/results-report.jsonl --log-file reviews/task-85/010-aws-retained-funnel-breakdown/artifacts/aws-1m-retained-funnel-breakdown-q500/suite-report.md
```

## Artifacts

- `suite-aws-1m-retained-funnel-breakdown-q500.json`: checked-in `ecaz bench suite` config.
- `artifacts/suite-audit.log`: suite audit output; audit passed with 3 steps.
- `artifacts/cloud-resume-before-retained-funnel.log`: AWS 1M resume log.
- `artifacts/cloud-install-retained-funnel.log`: AWS install log for `task-85-spire-product-scale-pareto`.
- `artifacts/cloud-bench-retained-funnel.log`: cloud bench wrapper log; synced artifacts from S3 run `20260607T152215Z`.
- `artifacts/aws-1m-retained-funnel-breakdown-q500/suite-run.log`: remote suite run log.
- `artifacts/aws-1m-retained-funnel-breakdown-q500/suite-manifest.json`: structured suite manifest.
- `artifacts/aws-1m-retained-funnel-breakdown-q500/results.jsonl`: raw suite results.
- `artifacts/aws-1m-retained-funnel-breakdown-q500/results-report.jsonl`: parsed report rows.
- `artifacts/aws-1m-retained-funnel-breakdown-q500/suite-report.md`: human-readable report.
- `artifacts/aws-1m-retained-funnel-breakdown-q500/funnel-retained-global1152-q500-first.jsonl`: first-run per-query funnel output.
- `artifacts/aws-1m-retained-funnel-breakdown-q500/funnel-retained-global1152-q500-repeat.jsonl`: warm-repeat per-query funnel output.
- `artifacts/funnel-first-summary.json`: jq summary of first-run funnel metrics.
- `artifacts/funnel-repeat-summary.json`: jq summary of warm-repeat funnel metrics.
- `artifacts/cloud-pause-after-retained-funnel.log`: AWS pause log after the run.
- `artifacts/aws-ec2-status-final.log`: first final AWS EC2 status after pause.
- `artifacts/aws-ec2-status-final-after-wait.log`: final AWS EC2 status after wait; DB and loader stopped.

## Key Results

First run:

- `recall@k=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- `latency_p50=243.656 ms`
- `latency_p95=312.391 ms`
- `latency_p99=2557.207 ms`
- `latency_max=27313.263 ms`

Warm repeat:

- `recall@k=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- `latency_p50=224.787 ms`
- `latency_p95=281.079 ms`
- `latency_p99=292.543 ms`
- `latency_max=299.931 ms`

Warm repeat funnel:

- object-read p50/p95/p99: `181.330 ms` / `242.776 ms` / `257.351 ms`
- summary-score p50/p95/p99: `47.541 ms` / `49.267 ms` / `49.745 ms`
- row-score p50/p95/p99: `10.121 ms` / `10.206 ms` / `10.249 ms`
- leaf object bytes/query p50/p95/p99: `684,831,192` / `708,813,432` / `715,537,148`
- summary bytes/query p50/p95/p99: `74,357,224` / `76,959,928` / `77,688,620`
- row bytes/query p50/p95/p99: `610,463,408` / `631,842,944` / `637,837,968`
- selected blocks/query p50/p95/p99: `1,152` / `1,152` / `1,152`
- candidates/query p50/p95/p99: `18,431` / `18,432` / `18,432`

## Conclusion

The retained same-recall latency bottleneck is dominated by object-read and row
payload bytes. The next Task 85 workstream must target read-path/layout
reduction before treating summary-score CPU as the primary optimization.
