# Artifact Manifest: Task 85 Packet 021

- head SHA: `c02408c7c257ea1291d4b7ff0247821d200f4a0a`
- task bucket: `reviews/task-85/`
- packet: `reviews/task-85/021-aws-v5-selected-segment-locators/`
- timestamp: `2026-06-07T20:06:53Z`
- lane: AWS 1M/q500, profile `1m`, database `postgres`
- fixture: `task67_1m_hnsw_m7g2xlarge`
- storage format: `rabitq`
- rerank mode: `rerank_width=25`
- surface: isolated index names on shared retained 1M corpus

## Commands

Resume:

```sh
target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-85/021-aws-v5-selected-segment-locators/artifacts/cloud-resume-v5.log
```

Install:

```sh
target/debug/ecaz cloud install --profile 1m --database postgres --git-ref task-85-spire-product-scale-pareto --skip-extension-recreate --timeout 3600 --log-file reviews/task-85/021-aws-v5-selected-segment-locators/artifacts/cloud-install-v5.log
```

Audit:

```sh
target/debug/ecaz bench suite audit --config reviews/task-85/021-aws-v5-selected-segment-locators/suite-aws-1m-v5-selected-segment-locators-q500.json --log-file reviews/task-85/021-aws-v5-selected-segment-locators/artifacts/suite-audit.log
```

Run:

```sh
target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/021-aws-v5-selected-segment-locators/suite-aws-1m-v5-selected-segment-locators-q500.json --suite task85-aws-1m-v5-selected-segment-locators-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/021-aws-v5-selected-segment-locators/artifacts/cloud-bench-v5-q500.log
```

Report:

```sh
target/debug/ecaz bench suite report --manifest reviews/task-85/021-aws-v5-selected-segment-locators/artifacts/aws-1m-v5-selected-segment-locators-q500/suite-manifest.json --results-output reviews/task-85/021-aws-v5-selected-segment-locators/artifacts/aws-1m-v5-selected-segment-locators-q500/results-report.jsonl --log-file reviews/task-85/021-aws-v5-selected-segment-locators/artifacts/aws-1m-v5-selected-segment-locators-q500/suite-report.md
```

Pause:

```sh
target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-85/021-aws-v5-selected-segment-locators/artifacts/cloud-pause-after-v5-q500.log
```

Final status capture:

```sh
script -q -c 'target/debug/ecaz cloud status --profile 1m --database postgres' reviews/task-85/021-aws-v5-selected-segment-locators/artifacts/cloud-status-final-after-v5-q500.log
```

## Artifacts

- `cloud-resume-v5.log`: AWS resume log.
- `cloud-install-v5.log`: AWS install log for branch
  `task-85-spire-product-scale-pareto`.
- `suite-audit.log`: suite audit; passed with 6 steps.
- `cloud-bench-v5-q500.log`: cloud bench wrapper log and S3 sync result.
- `aws-1m-v5-selected-segment-locators-q500/suite-config.json`: executed
  suite config copy.
- `aws-1m-v5-selected-segment-locators-q500/suite-manifest.json`: suite
  manifest.
- `aws-1m-v5-selected-segment-locators-q500/suite-run.log`: suite run log.
- `aws-1m-v5-selected-segment-locators-q500/results.jsonl`: parsed suite
  results.
- `aws-1m-v5-selected-segment-locators-q500/results-report.jsonl`: parsed
  report output.
- `aws-1m-v5-selected-segment-locators-q500/suite-report.md`: report output.
- `aws-1m-v5-selected-segment-locators-q500/build-spire-1m-rabitq-v5-block16-tg256.log`:
  V5 index build log.
- `aws-1m-v5-selected-segment-locators-q500/pipeline-retained-old-block16-global1152-q500-warm-control.log`:
  same-suite old retained control.
- `aws-1m-v5-selected-segment-locators-q500/pipeline-v5-block16-global1152-q500-first.log`:
  V5 first query run.
- `aws-1m-v5-selected-segment-locators-q500/pipeline-v5-block16-global1152-q500-repeat.log`:
  V5 warm repeat query run.
- `aws-1m-v5-selected-segment-locators-q500/funnel-*.jsonl`: per-query funnel
  evidence for old control, V5 first, and V5 repeat.
- `aws-1m-v5-selected-segment-locators-q500/miss-attribution-*.jsonl`:
  recall miss attribution evidence.
- `aws-1m-v5-selected-segment-locators-q500/storage-spire-1m-rabitq-v5-block16-tg256.log`:
  storage report.
- `cloud-pause-after-v5-q500.log`: AWS pause log.
- `cloud-status-final-after-v5-q500.log`: final AWS status, `state: paused`.

## Key Result Lines

- Suite completed 6/6 steps with no failures.
- V5 build: `total_ms=1701828`.
- Same-suite retained old control:
  `recall@10=0.9876`, `candidate_sum=9,213,846`,
  `heap_rerank_sum=12,500`, `p50=257.559 ms`, `p95=329.660 ms`,
  `p99=2577.758 ms`, `max=27738.801 ms`.
- V5 first:
  `recall@10=0.9876`, `candidate_sum=9,213,846`,
  `heap_rerank_sum=12,500`, `p50=233.653 ms`, `p95=289.871 ms`,
  `p99=303.805 ms`, `max=308.862 ms`.
- V5 repeat:
  `recall@10=0.9876`, `candidate_sum=9,213,846`,
  `heap_rerank_sum=12,500`, `p50=233.850 ms`, `p95=290.126 ms`,
  `p99=302.307 ms`, `max=307.818 ms`.
- Funnel old control:
  `selected_blocks_sum=576,000`,
  `row_segment_read_count_sum=1,180,606`,
  `row_segment_read_bytes_sum=9,622,405,352`,
  `object-read p50/p95=196.359/262.521 ms`,
  `candidate-score p50=57.758 ms`.
- Funnel V5 repeat:
  `selected_blocks_sum=576,000`,
  `row_segment_read_count_sum=1,180,606`,
  `row_segment_read_bytes_sum=9,622,405,352`,
  `object-read p50/p95=26.855/27.891 ms`,
  `candidate-score p50=57.668 ms`.
- Storage: V5 index `872.1 MiB`, `923.7 B/row`; retained block16 index
  `872.1 MiB`, `923.7 B/row`.
- Final AWS status: `state: paused`, `cost: ~$0.00/hr running`.
