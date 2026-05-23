# Task 51 AWS IVF/RaBitQ Baseline Confirmation Gate

- head SHA at suite config commit: `697b6d690b4311f36e95ff6c63f368acda1367b8`
- task bucket: `reviews/task-51/`
- benchmark packet: `benchmarks/task51-aws-ivf-rabitq-final-gate/`
- lane: AWS baseline confirmation gate, IVF and RaBitQ only
- role in round: confirms the preserved 1M Graviton IVF/RaBitQ shape before later Exp 3/5/7 follow-ups; not the final Task 51 round result
- AWS profile: `10k-medium`
- database host shape: `m8g.2xlarge`, restored on a 100 GB data volume
- source snapshot: `snap-091251b06d2da2df4`
- post-run snapshot: `snap-0e0632400184fadd4`
- database: `tqvector_bench`
- corpus/query prefix: `real_1m_ivf_rabitq1_rerank`
- access method: `ec_ivf`
- storage format: `rabitq`
- index reloptions: `quant_bits=1`, `rerank=heap_f32`, `rerank_width=50`, `storage_format=rabitq`
- isolated one-index-per-table surface: yes, reusing the preserved IVF/RaBitQ table and index from the source snapshot
- vchord / pgvectorscale: not run
- adaptive nprobe: off
- adaptive nprobe score-gap / margin-ratio GUCs: not enabled
- scratch SoA GUC: off
- sidecar real-I/O paths: not exercised in this suite; sidecar remains harness-only in separate packets

## Lifecycle

The stack was restored directly with Terraform to preserve the existing snapshot and use the required 100 GB / `m8g.2xlarge` shape:

```text
terraform apply
  -var-file=profiles/10k-medium.tfvars
  -var=from_snapshot_id=snap-091251b06d2da2df4
  -var=ecaz_git_ref=aws-optimization-ivf-rabitq-spire
  -var=db_volume_gb=100
  -var=db_instance_type=m8g.2xlarge
```

Bring-up artifacts:

- `artifacts/terraform-plan-restore-snap-091-m8g2xlarge.log`
- `artifacts/terraform-apply-restore-snap-091-m8g2xlarge.log`
- `artifacts/cloud-status-after-apply.log`

Precheck verified the preserved database before running the suite:

```text
/usr/bin/pg_config
PostgreSQL 18.3
active
tqvector_bench
real_1m_ivf_rabitq1_rerank_rabitq_idx|ec_ivf|{quant_bits=1,rerank=heap_f32,rerank_width=50,storage_format=rabitq}
990000
10000
```

Post-run snapshot:

```text
snapshot: profile=10k-medium id=snap-0e0632400184fadd4
```

## Commands

Local preflight:

```text
target/release/ecaz bench suite audit --config benchmarks/task51-aws-ivf-rabitq-final-gate/suite.json
target/release/ecaz bench suite run --dry-run --config benchmarks/task51-aws-ivf-rabitq-final-gate/suite.json --manifest-output benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/suite-dry-run-manifest.json
```

AWS suite command, executed through SSM on the restored database host:

```text
target/release/ecaz bench suite run --config benchmarks/task51-aws-ivf-rabitq-final-gate/suite.json --manifest-output benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/suite-manifest.json
target/release/ecaz bench suite status --manifest benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/suite-manifest.json
target/release/ecaz bench suite report --manifest benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/suite-manifest.json --results-output benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/results-report.jsonl
```

Snapshot and teardown:

```text
target/release/ecaz cloud snapshot --profile 10k-medium --description task51-aws-ivf-rabitq-final-gate-post-suite
target/release/ecaz cloud down --profile 10k-medium --yes
```

## Suite Status

```text
[suite:task51-aws-ivf-rabitq-final-gate] completed=5 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Completed steps:

- `recall-1m-rabitq1-rerank-q500`
- `latency-1m-rabitq1-rerank`
- `storage-1m-rabitq1-rerank`
- `explain-1m-rabitq1-rerank-p128`
- `explain-1m-rabitq1-rerank-p256`

## Results

Recall, q=500, k=10, `rerank_width=50`:

| nprobe | recall@10 | CI95 low | CI95 high | NDCG@10 | mean q-time |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 0.9716 | 0.9666 | 0.9759 | 0.9983 | 1532.04 ms |
| 128 | 0.9864 | 0.9828 | 0.9893 | 0.9993 | 46.94 ms |
| 256 | 0.9936 | 0.9910 | 0.9955 | 0.9998 | 75.44 ms |

Latency, q=200, concurrency=1:

| nprobe | mean | p50 | p95 | p99 | max |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 18.8 ms | 18.7 ms | 22.2 ms | 23.3 ms | 23.8 ms |
| 128 | 35.1 ms | 34.6 ms | 41.5 ms | 48.0 ms | 50.0 ms |
| 256 | 66.5 ms | 66.2 ms | 72.5 ms | 75.7 ms | 81.5 ms |

Storage:

| Field | Value |
| --- | --- |
| rows | 990000 |
| table | 15.4 GiB |
| indexes | 340.4 MiB |
| total | 15.7 GiB |
| ec_ivf index | 298.0 MiB |
| ec_ivf per row | 315.6 B |

EXPLAIN counters:

| nprobe | actual total | selected lists | candidates scored | rerank rows | heap blocks fetched | approximate scan | exact rerank |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 128 | 40.475 ms | 128 | 142240 | 50 | 48 | 36774 us | 945 us |
| 256 | 76.996 ms | 256 | 293022 | 50 | 48 | 73649 us | 454 us |

The `nprobe=64` recall mean q-time includes cold snapshot / exact-truth setup effects from the first recall sweep. The separate latency step, run after the recall step on the same restored host, reports `nprobe=64` p50 `18.7 ms`.

## Artifacts

- `suite.json`: checked-in SuiteConfig.
- `artifacts/suite-manifest.json`: suite execution manifest.
- `artifacts/results.jsonl`: structured run results.
- `artifacts/results-report.jsonl`: report output.
- `artifacts/suite-run.log`: remote suite run log.
- `artifacts/suite-status.log`: suite status.
- `artifacts/suite-report.log`: parsed suite report.
- `artifacts/recall-1m-rabitq1-rerank-q500.log`: recall run.
- `artifacts/latency-1m-rabitq1-rerank.log`: latency run.
- `artifacts/storage-1m-rabitq1-rerank.log`: storage run.
- `artifacts/explain-1m-rabitq1-rerank-p128.{sql,log}`: planner and execution counters at nprobe 128.
- `artifacts/explain-1m-rabitq1-rerank-p256.{sql,log}`: planner and execution counters at nprobe 256.
- `artifacts/truth-aws-real-1m-q500-k10.json`: generated truth cache.
- `artifacts/ssm-preserved-db-precheck-invocation.json`: preserved DB/index precheck.
- `artifacts/ssm-run-suite-invocation.json`: SSM command result for the suite.
- `artifacts/cloud-snapshot-post-final-gate*.log`: post-suite snapshot evidence.
- `artifacts/cloud-down-post-final-gate*.log`: teardown evidence.
