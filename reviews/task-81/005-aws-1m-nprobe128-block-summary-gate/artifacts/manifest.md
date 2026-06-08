# Task 81 Packet 005 Artifact Manifest

- head SHA: `c9948f31ad0f7aa2499be50a27166e9e17d8c166e`
- branch: `task-81-spire-leaf-block-summary-format`
- task bucket: `reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/`
- timestamp: `2026-06-04T21:06:39-07:00`
- lane: AWS 1M / PG18 / Graviton / RaBitQ
- retained profile: `1m`
- database: `postgres`
- corpus table: `task67_1m_hnsw_m7g2xlarge_corpus`
- query table: `task67_1m_hnsw_m7g2xlarge_queries`
- index: `aws_spire_1m_rabitq_t80_block16_tg256_idx`
- storage format: `rabitq`
- surface isolation: retained shared-table AWS 1M surface
- runner: `ecaz bench suite` via `ecaz cloud bench`
- suite config: `reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/suite-aws-1m-nprobe128-block-summary-gate.json`
- suite config SHA-256: `9f4df056b3e809a508cd68dfb86f0420034fc4b4788e98dba190c599d546cd5a`
- suite manifest: `reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/artifacts/aws-1m-nprobe128-block-summary-gate/suite-manifest.json`
- results JSONL: `reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/artifacts/aws-1m-nprobe128-block-summary-gate/results.jsonl`
- query/truth shape: q500, k10, truth cache `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/truth-aws-real-1m-q500-k10.json`
- nprobe: `128`
- global candidate cap: `1152`
- rerank mode: heap rerank width `25`
- AWS final state: `paused` in `artifacts/cloud-status-after-pause-final-paused.log`

## Commands

Status before:

```sh
script -q -c "target/debug/ecaz cloud status --profile 1m --database postgres" reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/artifacts/cloud-status-before.log
```

Resume:

```sh
target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/artifacts/cloud-resume-before-task81-nprobe128.log
```

Audit:

```sh
script -q -c "target/debug/ecaz bench suite audit --config reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/suite-aws-1m-nprobe128-block-summary-gate.json --database postgres --host /var/run/postgresql" reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/artifacts/suite-audit.log
```

Cloud bench:

```sh
target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/suite-aws-1m-nprobe128-block-summary-gate.json --suite task81-aws-1m-nprobe128-block-summary-gate --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/artifacts/cloud-bench-task81-aws-1m-nprobe128-block-summary-gate.log
```

Pause and final status:

```sh
target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/artifacts/cloud-pause-after-task81-nprobe128.log
script -q -c "target/debug/ecaz cloud status --profile 1m --database postgres" reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/artifacts/cloud-status-after-pause-final.log
script -q -c "target/debug/ecaz cloud status --profile 1m --database postgres" reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/artifacts/cloud-status-after-pause-final-paused.log
```

Suite status and report:

```sh
target/debug/ecaz bench suite status --manifest reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/artifacts/aws-1m-nprobe128-block-summary-gate/suite-manifest.json --database postgres --host /var/run/postgresql --log-file reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/artifacts/aws-1m-nprobe128-block-summary-gate/suite-status.log
target/debug/ecaz bench suite report --manifest reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/artifacts/aws-1m-nprobe128-block-summary-gate/suite-manifest.json --results-output reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/artifacts/aws-1m-nprobe128-block-summary-gate/suite-report-results.jsonl --database postgres --host /var/run/postgresql --log-file reviews/task-81/005-aws-1m-nprobe128-block-summary-gate/artifacts/aws-1m-nprobe128-block-summary-gate/suite-report.log
```

## Artifacts

- `suite-audit.log`: suite audit, passed for four steps.
- `cloud-status-before.log`: initial status, `state: paused`.
- `cloud-status-after-pause-final.log`: immediate post-pause status, `state: stopping`.
- `cloud-status-after-pause-final-paused.log`: final durable status, `state: paused`, `$0.00/hr running`.
- `aws-1m-nprobe128-block-summary-gate/repair-aws-1m-task81-leaf-candidate-snapshot.log`: diagnostic function catalog repair.
- `aws-1m-nprobe128-block-summary-gate/precheck-aws-1m-task81-nprobe128-surface.log`: fixture/index/GUC precheck.
- `aws-1m-nprobe128-block-summary-gate/pipeline-spire-1m-rabitq-block-summary-global1152-nprobe128.log`: q500 pipeline log.
- `aws-1m-nprobe128-block-summary-gate/funnel-spire-1m-rabitq-block-summary-global1152-nprobe128.jsonl`: q500 funnel output.
- `aws-1m-nprobe128-block-summary-gate/diagnostics-spire-1m-rabitq-block-summary-global1152-nprobe128.log`: aggregate block/timing diagnostics over q500.
- `aws-1m-nprobe128-block-summary-gate/suite-manifest.json`: structured suite manifest.
- `aws-1m-nprobe128-block-summary-gate/results.jsonl`: normalized suite results.
- `aws-1m-nprobe128-block-summary-gate/suite-status.log`: `completed=4 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `aws-1m-nprobe128-block-summary-gate/suite-report.log`: suite report.
- `aws-1m-nprobe128-block-summary-gate/suite-report-results.jsonl`: parsed report result rows.

## Key Results

AWS q500 nprobe128 block-summary row:

- `effective_nprobe=128`
- `ec_spire.leaf_block_pruning_max_global_blocks=1152`
- `candidate_sum=9,213,838`
- `latency_p50=303.107 ms`
- `latency_p95=390.202 ms`
- `latency_p99=408.786 ms`
- `recall@k=0.9832`

Diagnostic aggregate:

- `query_count=500`
- `candidate_rows=9,213,838`
- `blocks_available=30,966,000`
- `blocks_selected=576,000`
- `blocks_skipped=30,390,000`
- `summary_bytes=49,151,386,000`
- `row_bytes=403,526,232,000`
- `summary_score_nanos=31,464,838,658`
- `row_score_nanos=5,068,876,571`
- `candidate_score_nanos=36,533,715,229`

Comparators:

| Row | Candidates | p50 ms | p95 ms | p99 ms | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: |
| old tg96 comparator | 9,213,846 | 268.824 | 331.460 | 345.762 | 0.9832 |
| packet 003 nprobe96 global1152 | 9,213,846 | 265.911 | 329.407 | 342.454 | 0.9832 |
| packet 005 nprobe128 global1152 | 9,213,838 | 303.107 | 390.202 | 408.786 | 0.9832 |

## Gate Readout

- Candidate-surface gate: pass, `9,213,838` is below the old `9,213,846` q500 shape.
- AWS recall gate: fail, recall remains `0.9832` and does not improve over the old tg96 row.
- Latency: fail/regression versus packet 003 and the old comparator.

Conclusion: the local nprobe128 improvement from packet 004 does not transfer to the AWS 1M retained surface. Task 81 remains active.
