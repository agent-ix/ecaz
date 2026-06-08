# Task 85 Packet 025 Artifact Manifest

- Head SHA during AWS measurement:
  `568e46baec724e9a8380b1c6a55a7ed108034f68`
- Code checkpoint under measurement:
  `a9a938771dde55ac2ed02984071c2360949c2ec0`
- Task bucket:
  `reviews/task-85/`
- Packet path:
  `reviews/task-85/025-aws-rerank-locality-measurement/`
- Lane:
  AWS 1M/q500
- Fixture:
  `task67_1m_hnsw_m7g2xlarge`
- Storage format:
  RaBitQ
- Rerank mode:
  retained `rerank_width=25`
- Index:
  `aws_spire_1m_rabitq_t85_v5_block16_tg256_idx`
- Surface isolation:
  shared corpus table, existing one-index-per-profile index surface
- Timestamp:
  2026-06-07

## Commands

Local CLI build:

```text
cargo build -p ecaz-cli --locked --offline
```

Audit:

```text
target/debug/ecaz bench suite audit --config reviews/task-85/025-aws-rerank-locality-measurement/suite-aws-1m-rerank-locality-q500.json --log-file reviews/task-85/025-aws-rerank-locality-measurement/artifacts/suite-audit.log
target/debug/ecaz bench suite audit --config reviews/task-85/025-aws-rerank-locality-measurement/suite-aws-1m-rerank-locality-q500.json --log-file reviews/task-85/025-aws-rerank-locality-measurement/artifacts/suite-audit-rerun.log
```

AWS status before resume:

```text
script -q -c 'target/debug/ecaz cloud status --profile 1m --database postgres' reviews/task-85/025-aws-rerank-locality-measurement/artifacts/cloud-status-before-rerank-locality.log
```

AWS resume:

```text
target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-85/025-aws-rerank-locality-measurement/artifacts/cloud-resume-rerank-locality.log
```

AWS install:

```text
target/debug/ecaz cloud install --profile 1m --database postgres --git-ref task-85-spire-product-scale-pareto --skip-extension-recreate --timeout 3600 --log-file reviews/task-85/025-aws-rerank-locality-measurement/artifacts/cloud-install-rerank-locality.log
```

AWS suite, first failed precheck:

```text
target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/025-aws-rerank-locality-measurement/suite-aws-1m-rerank-locality-q500.json --suite task85-aws-1m-rerank-locality-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/025-aws-rerank-locality-measurement/artifacts/cloud-bench-rerank-locality-q500.log
```

This failed before q500 execution because `--skip-extension-recreate` retained
the old SQL catalog and the new diagnostic function was not registered.

AWS suite, successful rerun:

```text
target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/025-aws-rerank-locality-measurement/suite-aws-1m-rerank-locality-q500.json --suite task85-aws-1m-rerank-locality-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/025-aws-rerank-locality-measurement/artifacts/cloud-bench-rerank-locality-q500-rerun.log
```

AWS pause and final status:

```text
target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-85/025-aws-rerank-locality-measurement/artifacts/cloud-pause-after-rerank-locality-q500.log
script -q -c 'target/debug/ecaz cloud status --profile 1m --database postgres' reviews/task-85/025-aws-rerank-locality-measurement/artifacts/cloud-status-final-after-rerank-locality-q500.log
script -q -c 'target/debug/ecaz cloud status --profile 1m --database postgres' reviews/task-85/025-aws-rerank-locality-measurement/artifacts/cloud-status-final-after-rerank-locality-q500-paused.log
```

Suite report:

```text
target/debug/ecaz bench suite report --manifest reviews/task-85/025-aws-rerank-locality-measurement/artifacts/aws-1m-rerank-locality-q500/suite-manifest.json --results-output reviews/task-85/025-aws-rerank-locality-measurement/artifacts/aws-1m-rerank-locality-q500/results-report.jsonl --log-file reviews/task-85/025-aws-rerank-locality-measurement/artifacts/aws-1m-rerank-locality-q500/suite-report.md
```

Rerank locality summary:

```text
jq -s 'def pct($p): sort | .[((length * $p / 100) | ceil) - 1]; ...' reviews/task-85/025-aws-rerank-locality-measurement/artifacts/aws-1m-rerank-locality-q500/funnel-v5-summary-fast-path-rerank-locality-q500-repeat.jsonl
```

## Artifacts

- `suite-aws-1m-rerank-locality-q500.json`: checked-in SuiteConfig.
- `artifacts/suite-audit.log`: initial suite audit; key line:
  `[suite:task85-aws-1m-rerank-locality-q500] audit passed: 3 steps`.
- `artifacts/suite-audit-rerun.log`: corrected suite audit after adding the
  retained-extension SQL registration step; key line:
  `[suite:task85-aws-1m-rerank-locality-q500] audit passed: 4 steps`.
- `artifacts/cloud-status-before-rerank-locality.log`: starting AWS status;
  key line: `state:    paused`.
- `artifacts/cloud-resume-rerank-locality.log`: AWS resume log.
- `artifacts/cloud-install-rerank-locality.log`: AWS install log; key line:
  `install: profile=1m db=10.42.1.131 ref=task-85-spire-product-scale-pareto ok`.
- `artifacts/cloud-bench-rerank-locality-q500.log`: failed first cloud bench
  precheck proving the SQL catalog needed explicit function registration.
- `artifacts/cloud-bench-rerank-locality-q500-rerun.log`: successful cloud
  bench wrapper log; synced artifacts from S3 run `20260607T212005Z`.
- `artifacts/aws-1m-rerank-locality-q500/apply-rerank-locality-snapshot-signature.log`:
  raw SQL step that registers the diagnostic function without dropping the
  retained extension/index surface.
- `artifacts/aws-1m-rerank-locality-q500/precheck-aws-1m-rerank-locality-surface.log`:
  precheck proving the function exists and returns locality fields.
- `artifacts/aws-1m-rerank-locality-q500/suite-manifest.json`: suite manifest.
- `artifacts/aws-1m-rerank-locality-q500/suite-run.log`: suite run log.
- `artifacts/aws-1m-rerank-locality-q500/results.jsonl`: parsed suite results.
- `artifacts/aws-1m-rerank-locality-q500/results-report.jsonl`: report output.
- `artifacts/aws-1m-rerank-locality-q500/suite-report.md`: generated suite
  report.
- `artifacts/aws-1m-rerank-locality-q500/pipeline-v5-summary-fast-path-rerank-locality-q500-first.log`:
  first pipeline pass.
- `artifacts/aws-1m-rerank-locality-q500/pipeline-v5-summary-fast-path-rerank-locality-q500-repeat.log`:
  warm repeat pipeline pass.
- `artifacts/aws-1m-rerank-locality-q500/funnel-v5-summary-fast-path-rerank-locality-q500-first.jsonl`:
  first funnel counters.
- `artifacts/aws-1m-rerank-locality-q500/funnel-v5-summary-fast-path-rerank-locality-q500-repeat.jsonl`:
  repeat funnel counters.
- `artifacts/aws-1m-rerank-locality-q500/rerank-locality-repeat-summary.json`:
  derived locality and funnel timing summary.
- `artifacts/cloud-pause-after-rerank-locality-q500.log`: pause log.
- `artifacts/cloud-status-final-after-rerank-locality-q500.log`: first final
  status capture, caught `state: stopping`.
- `artifacts/cloud-status-final-after-rerank-locality-q500-paused.log`: final
  status capture; key line: `state:    paused`.

## Key Results

First run:

- `recall@10=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- `latency_p50=243.197 ms`
- `latency_p95=309.324 ms`
- `latency_p99=2558.404 ms`
- `latency_max=27344.473 ms`

Repeat run:

- `recall@10=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- `latency_p50=222.140 ms`
- `latency_p95=275.753 ms`
- `latency_p99=288.894 ms`
- `latency_max=296.358 ms`
- object-read p50/p95: `26.236/27.353 ms`
- candidate-score p50/p95: `57.594/59.303 ms`
- summary-score p50/p95: `47.525/49.225 ms`
- row-score p50/p95: `10.067/10.144 ms`

Rerank-prefix locality, repeat run:

- `queries=500`
- `rerank_prefix_sum=12,500`
- unique heap blocks p50/p95/max: `22/25/25`
- adjacent heap-block transitions p50/p95/max: `24/24/24`
- heap-block span p50/p95/max: `8,366/8,993/9,235`
- heap-block jump sum p50/p95/max: `70,314/93,600/122,314`
- max adjacent heap-block jump p50/p95/max: `7,533/8,766/9,225`

Decision: this packet does not accept a latency win by itself, but it shows
material heap-block scatter at the unchanged packet 023 recall/candidate
surface. Candidate-set-preserving rerank locality moves to implementation.
