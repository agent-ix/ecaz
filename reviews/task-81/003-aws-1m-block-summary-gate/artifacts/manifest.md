# Task 81 Packet 003 Artifact Manifest

- Task bucket: `reviews/task-81/`
- Packet: `reviews/task-81/003-aws-1m-block-summary-gate/`
- Head SHA: `36dbdad746abe3b6ba40d5d55d191f3d912a067d`
- Captured: `2026-06-04 20:29:17 PDT`
- Lane: AWS 1M / PG18 / Graviton / RaBitQ
- Fixture: retained AWS profile `1m`, database `postgres`
- Corpus table: `task67_1m_hnsw_m7g2xlarge_corpus`
- Query table: `task67_1m_hnsw_m7g2xlarge_queries`
- Index: `aws_spire_1m_rabitq_t80_block16_tg256_idx`
- Standard runner: `ecaz bench suite` via `ecaz cloud bench`
- Suite config: `reviews/task-81/003-aws-1m-block-summary-gate/suite-aws-1m-block-summary-gate.json`
- Suite manifest: `reviews/task-81/003-aws-1m-block-summary-gate/artifacts/aws-1m-block-summary-gate/suite-manifest.json`
- Results JSONL: `reviews/task-81/003-aws-1m-block-summary-gate/artifacts/aws-1m-block-summary-gate/results.jsonl`
- Storage format: `rabitq`
- Rerank mode: `rerank_width=25`
- Query count: `500`
- Isolation mode: retained shared-table AWS 1M surface

## Commands

Status before:

```sh
target/debug/ecaz cloud status --profile 1m --database postgres --log-file reviews/task-81/003-aws-1m-block-summary-gate/artifacts/cloud-status-before.log
```

Resume:

```sh
target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-81/003-aws-1m-block-summary-gate/artifacts/cloud-resume-before-task81.log
```

Install current branch, preserving retained tables:

```sh
target/debug/ecaz cloud install --profile 1m --git-ref task-81-spire-leaf-block-summary-format --database postgres --skip-extension-recreate --log-file reviews/task-81/003-aws-1m-block-summary-gate/artifacts/cloud-install-task81.log
```

Audit:

```sh
script -q -c "target/debug/ecaz bench suite audit --config reviews/task-81/003-aws-1m-block-summary-gate/suite-aws-1m-block-summary-gate.json --database postgres --host /var/run/postgresql" reviews/task-81/003-aws-1m-block-summary-gate/artifacts/suite-audit.log
```

Successful cloud run:

```sh
target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-81/003-aws-1m-block-summary-gate/suite-aws-1m-block-summary-gate.json --suite task81-aws-1m-block-summary-gate --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-81/003-aws-1m-block-summary-gate/artifacts/cloud-bench-task81-aws-1m-block-summary-gate-repair-upload.log
```

Pause and final status:

```sh
target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-81/003-aws-1m-block-summary-gate/artifacts/cloud-pause-after-task81.log
script -q -c "target/debug/ecaz cloud status --profile 1m --database postgres" reviews/task-81/003-aws-1m-block-summary-gate/artifacts/cloud-status-after-pause-final.log
```

Suite status and report:

```sh
target/debug/ecaz bench suite status --manifest reviews/task-81/003-aws-1m-block-summary-gate/artifacts/aws-1m-block-summary-gate/suite-manifest.json --database postgres --host /var/run/postgresql --log-file reviews/task-81/003-aws-1m-block-summary-gate/artifacts/aws-1m-block-summary-gate/suite-status.log
target/debug/ecaz bench suite report --manifest reviews/task-81/003-aws-1m-block-summary-gate/artifacts/aws-1m-block-summary-gate/suite-manifest.json --results-output reviews/task-81/003-aws-1m-block-summary-gate/artifacts/aws-1m-block-summary-gate/suite-report-results.jsonl --database postgres --host /var/run/postgresql --log-file reviews/task-81/003-aws-1m-block-summary-gate/artifacts/aws-1m-block-summary-gate/suite-report.log
```

## Artifacts

- `suite-audit.log`: final suite audit, passed for four steps.
- `cloud-bench-task81-aws-1m-block-summary-gate.log`: first failed cloud wrapper attempt; precheck used wrong query vector column `embedding`.
- `cloud-bench-task81-aws-1m-block-summary-gate-rerun.log`: second failed attempt; retained AWS function catalog still exposed the old SQL return table.
- `cloud-bench-task81-aws-1m-block-summary-gate-after-repair.log`: third failed attempt; remote stale suite config was reused by the cloud wrapper.
- `cloud-status-after-pause-final.log`: final durable status, `state: paused`, `$0.00/hr running`.
- `aws-1m-block-summary-gate/repair-aws-1m-task81-leaf-candidate-snapshot.log`: catalog repair for the new diagnostic return columns.
- `aws-1m-block-summary-gate/precheck-aws-1m-task81-surface.log`: fixture/index/precheck with new diagnostic columns visible.
- `aws-1m-block-summary-gate/pipeline-spire-1m-rabitq-block-summary-global1152.log`: q500 pipeline row.
- `aws-1m-block-summary-gate/funnel-spire-1m-rabitq-block-summary-global1152.jsonl`: q500 funnel output.
- `aws-1m-block-summary-gate/diagnostics-spire-1m-rabitq-block-summary-global1152.log`: aggregate block/timing diagnostics over q500.
- `aws-1m-block-summary-gate/suite-manifest.json`: structured suite manifest.
- `aws-1m-block-summary-gate/results.jsonl`: normalized suite results.
- `aws-1m-block-summary-gate/suite-status.log`: `completed=4 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `aws-1m-block-summary-gate/suite-report.log`: suite report.
- `aws-1m-block-summary-gate/suite-report-results.jsonl`: report result rows.

Note: several cloud lifecycle commands printed useful status to terminal but
left empty `--log-file` paths; those empty files are intentionally not part of
the packet. The durable AWS final-state artifact is
`cloud-status-after-pause-final.log`.

## Key Results

AWS q500 block-summary row:

- `nprobe=96`
- `ec_spire.leaf_block_pruning_max_global_blocks=1152`
- `candidate_sum=9213846`
- `latency_p50=265.911 ms`
- `latency_p95=329.407 ms`
- `latency_p99=342.454 ms`
- `recall@k=0.9832`

Diagnostic aggregate:

- `query_count=500`
- `candidate_rows=9213846`
- `blocks_available=23389983`
- `blocks_selected=576000`
- `blocks_skipped=22813983`
- `summary_bytes=37126170208`
- `row_bytes=304802815448`
- `summary_score_nanos=23130193665`
- `row_score_nanos=5241865350`
- `candidate_score_nanos=28372059015`

Comparator from Task 80/old tg96 q500 shape:

- recall@10 `0.9832`
- candidates `9,213,846`
- p50 `268.824 ms`

## Gate Readout

- Candidate-surface gate: pass, `9,213,846` does not exceed the old `9,213,846` q500 shape.
- Latency: slight improvement vs old p50 (`265.911 ms` vs `268.824 ms`).
- AWS recall gate: fail, recall remains `0.9832` and does not improve over the old tg96 row.

Conclusion: this packet captures the AWS 1M follow-up required after the local gate, but it does not close Task 81. The current mechanism preserves the old candidate surface and slightly improves p50, but does not satisfy the Task 81 AWS recall criterion.
