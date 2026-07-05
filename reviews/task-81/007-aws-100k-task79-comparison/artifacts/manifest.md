# Task 81 Packet 007 Manifest: AWS 100k Task 79 Comparison

- head SHA at packet creation: `41889dc20`
- branch: `task-81-spire-leaf-block-summary-format`
- task bucket: `reviews/task-81/`
- packet: `reviews/task-81/007-aws-100k-task79-comparison/`
- timestamp: `2026-06-05`
- lane: AWS Graviton retained `1m` stack
- database for accepted run: `task79_aws`
- PostgreSQL: PG18 on `/var/run/postgresql`
- runner: `ecaz cloud bench` driving `ecaz bench suite`
- remote runner binary: `/usr/local/bin/ecaz`
- accepted suite config:
  `reviews/task-81/007-aws-100k-task79-comparison/suite-aws-100k-task79-retained-surface-warm.json`
- accepted suite config SHA256:
  `7dfab6948ee474d04bcbdc4e4fe2f945eeb2d11a32b2d6b62d7654692239d56c`
- accepted artifact dir:
  `reviews/task-81/007-aws-100k-task79-comparison/artifacts/task79-retained-surface-warm/`
- accepted S3 source:
  `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task81-aws-100k-task79-retained-surface-warm/20260605T045855Z/`
- AWS final state: `paused`, `$0.00/hr` running compute

## Comparison Baseline

The corrected acceptance comparison is Task 79's accepted AWS 100k/q200 row
from `reviews/task-79/045-aws-rabitq-deferred-decode/`:

- prefix: `task79_aws_rabitq_k3_block16_tg96_b0`
- global blocks: `1152`
- nprobe: `96`
- candidates: `3,672,619`
- recall@10: `0.9945`
- p50: `35.199 ms`
- p95: `36.203 ms`
- p99: `36.591 ms`

This packet reuses the retained Task 79 AWS surface and reruns the same
`global1152` row with the current Task 81 branch code. It avoids copying another
100k corpus after a fresh-copy attempt failed with no space left on device.

## Commands

Resume before the accepted run:

```text
target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-81/007-aws-100k-task79-comparison/artifacts/cloud-resume-before-warm-retained-surface.log
```

Accepted warm-order suite:

```text
target/debug/ecaz cloud bench --profile 1m --database task79_aws --config reviews/task-81/007-aws-100k-task79-comparison/suite-aws-100k-task79-retained-surface-warm.json --suite task81-aws-100k-task79-retained-surface-warm --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-81/007-aws-100k-task79-comparison/artifacts/cloud-bench-retained-task79-surface-warm.log
```

Report/status:

```text
target/debug/ecaz bench suite status --manifest reviews/task-81/007-aws-100k-task79-comparison/artifacts/task79-retained-surface-warm/suite-manifest.json --database task79_aws --host /var/run/postgresql --log-file reviews/task-81/007-aws-100k-task79-comparison/artifacts/task79-retained-surface-warm/suite-status.log
target/debug/ecaz bench suite report --manifest reviews/task-81/007-aws-100k-task79-comparison/artifacts/task79-retained-surface-warm/suite-manifest.json --results-output reviews/task-81/007-aws-100k-task79-comparison/artifacts/task79-retained-surface-warm/suite-report-results.jsonl --database task79_aws --host /var/run/postgresql --log-file reviews/task-81/007-aws-100k-task79-comparison/artifacts/task79-retained-surface-warm/suite-report.log
```

Shutdown:

```text
target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-81/007-aws-100k-task79-comparison/artifacts/cloud-pause-after-warm-retained-surface.log
target/debug/ecaz cloud status --profile 1m --database postgres
```

## Suite Status

- completed: `3`
- failed: `0`
- skipped: `0`
- dry-run: `0`
- missing artifacts: `0`
- stale artifacts: `0`

## Key Results

Accepted comparison row:

| Step | Candidates | Recall@10 | Latency p50 | Latency p95 | Latency p99 | Production read p50/p95 |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Task 79 accepted AWS `global1152` | `3,672,619` | `0.9945` | `35.199 ms` | `36.203 ms` | `36.591 ms` | `31 / 32 ms` |
| Task 81 current branch retained-surface warm `global1152` | `3,672,619` | `0.9945` | `32.023 ms` | `32.940 ms` | `33.315 ms` | `28 / 29 ms` |

Delta versus Task 79 accepted row:

- p50: `3.176 ms` faster (`9.02%`)
- p95: `3.263 ms` faster (`9.01%`)
- p99: `3.276 ms` faster (`8.95%`)
- candidates: unchanged
- recall@10: unchanged

Warmup row:

| Step | Candidates | Recall@10 | Latency p50 | Latency p95 | Latency p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| retained-surface `global1024` warmup | `3,264,695` | `0.9920` | `39.469 ms` | `52.210 ms` | `56.679 ms` |

The first retained-surface attempt without the warmup row is retained as
provenance only:

- `artifacts/task79-retained-surface/results.jsonl`
- `global1152`: recall@10 `0.9945`, candidates `3,672,619`, p50 `42.467 ms`

The failed fresh-copy attempt is also retained:

- `artifacts/ssm-cloud-bench-aws-100k-task79-comparison-rerun-fail.json`
- failure: PostgreSQL could not extend a copied corpus table due to
  `No space left on device`.

## Artifact Inventory

- `suite-aws-100k-task79-comparison.json`: fresh-copy suite, failed due to disk
  space before index build.
- `suite-aws-100k-task79-retained-surface.json`: first retained-surface suite,
  cold `global1152` only.
- `suite-aws-100k-task79-retained-surface-warm.json`: accepted warm-order
  retained-surface suite.
- `artifacts/task79-retained-surface-warm/suite-manifest.json`: accepted suite
  manifest.
- `artifacts/task79-retained-surface-warm/results.jsonl`: accepted raw result
  stream.
- `artifacts/task79-retained-surface-warm/suite-report-results.jsonl`: accepted
  parsed report result stream.
- `artifacts/task79-retained-surface-warm/suite-status.log`: accepted suite
  status.
- `artifacts/task79-retained-surface-warm/suite-report.log`: accepted suite
  report.
- `artifacts/task79-retained-surface-warm/pipeline-retained-task79-global1152.log`:
  accepted comparison-row pipeline log.
- `artifacts/task79-retained-surface-warm/funnel-retained-task79-global1152.jsonl`:
  accepted comparison-row funnel output.
- `artifacts/cloud-resume-before-warm-retained-surface.log`: accepted run
  resume log.
- `artifacts/cloud-bench-retained-task79-surface-warm.log`: accepted cloud bench
  wrapper log.
- `artifacts/cloud-pause-after-warm-retained-surface.log`: accepted run pause
  log.
