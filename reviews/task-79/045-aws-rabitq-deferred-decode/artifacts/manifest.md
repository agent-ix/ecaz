# Task 79 AWS RaBitQ Deferred Decode Benchmark Manifest

- head SHA: `74ee977da3d5cda335167af882965c65978a231f`
- branch: `task-79-spire-candidate-surface-reduction`
- task bucket: `reviews/task-79/`
- packet: `reviews/task-79/045-aws-rabitq-deferred-decode/`
- timestamp: `2026-06-02T23:23:03Z` suite S3 timestamp
- lane: AWS Graviton `1m` retained stack
- DB instance: `i-0e9723f7b707504c3`, private IP `10.42.1.17`
- loader instance: `i-0d9fb0eac2df5567b`, private IP `10.42.1.142`
- database: `task79_aws`
- PostgreSQL: PG18 on `/var/run/postgresql`
- runner: `ecaz cloud bench` driving `ecaz bench suite`
- local runner binary: `target/debug/ecaz`
- remote runner binary: `/usr/local/bin/ecaz`
- suite config: `reviews/task-79/045-aws-rabitq-deferred-decode/suite.json`
- suite config SHA256: `714697a3fd40bda914c476a935d84b40f912b0bab94f5b929e471f0439192c5c`
- S3 artifact source: `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task79-aws-rabitq-deferred-decode/20260602T232303Z/`
- storage layout: isolated one-index-per-table surface
- storage format: `rabitq`
- SPIRE build shape: `leaf_block_rows=16`, K3 recursive layout,
  `top_graph_search_list_size=96`, `boundary_replica_count=0`
- SPIRE read shape: `nprobe=96`, `rerank_width=25`,
  `leaf_block_pruning_max_blocks_per_leaf=0`,
  `leaf_block_pruning_global_probe_blocks=0`,
  `leaf_block_pruning_sample_rows_per_block=0`,
  `leaf_block_pruning_sample_summary_prior_weight=0.8`,
  `leaf_block_pruning_summary_radius_weight=0.25`,
  `leaf_block_pruning_route_prior_weight=0.0`
- query count: 200
- cloud final state: paused, `$0.00/hr` running compute, retained storage only

## Commands

Initial status:

```text
target/debug/ecaz cloud status --profile 1m --log-file reviews/task-79/045-aws-rabitq-deferred-decode/artifacts/cloud-status-before.log
```

Launch/resume AWS profile:

```text
target/debug/ecaz cloud up --profile 1m --log-file reviews/task-79/045-aws-rabitq-deferred-decode/artifacts/cloud-up.log
```

AWS suite run:

```text
target/debug/ecaz cloud bench --profile 1m --suite task79-aws-rabitq-deferred-decode --database task79_aws --config reviews/task-79/045-aws-rabitq-deferred-decode/suite.json --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-79/045-aws-rabitq-deferred-decode/artifacts/cloud-bench-rerun-freshdb.log
```

Report:

```text
target/debug/ecaz bench suite report --manifest reviews/task-79/045-aws-rabitq-deferred-decode/artifacts/suite-manifest.json --results-output reviews/task-79/045-aws-rabitq-deferred-decode/artifacts/results-report.jsonl
```

Shutdown:

```text
target/debug/ecaz cloud pause --profile 1m --database task79_aws --log-file reviews/task-79/045-aws-rabitq-deferred-decode/artifacts/cloud-pause.log
target/debug/ecaz cloud status --profile 1m --log-file reviews/task-79/045-aws-rabitq-deferred-decode/artifacts/cloud-status-after-pause.log
aws ec2 describe-instances --region us-west-2 --filters Name=tag:ecaz:profile,Values=1m Name=instance-state-name,Values=pending,running,stopping,stopped --query 'Reservations[].Instances[].{InstanceId:InstanceId,State:State.Name,PrivateIp:PrivateIpAddress,Name:Tags[?Key==`Name`]|[0].Value,Role:Tags[?Key==`ecaz:role`]|[0].Value}' --output table
```

## Suite Status

- completed: `5`
- failed: `0`
- skipped: `0`
- dry-run: `0`
- missing artifacts: `0`
- stale artifacts: `0`

## Key Results

| Step | Max global blocks | Candidate sum | Recall@10 | Latency p50 | Latency p95 | Latency p99 | Production read p50 | Production read p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `pipeline-100k-rabitq-k3-deferred-global1024-rw025` | `1024` | `3,264,695` | `0.9920` | `33.354 ms` | `34.217 ms` | `34.638 ms` | `29.000 ms` | `30.000 ms` |
| `pipeline-100k-rabitq-k3-deferred-global1152-rw025` | `1152` | `3,672,619` | `0.9945` | `35.199 ms` | `36.203 ms` | `36.591 ms` | `31.000 ms` | `32.000 ms` |
| `pipeline-100k-rabitq-k3-deferred-global1216-rw025` | `1216` | `3,876,624` | `0.9950` | `35.866 ms` | `36.659 ms` | `36.912 ms` | `32.000 ms` | `32.000 ms` |

Recommended/default AWS point: `global1152`. It passes the recall floor used
for the Task 79 local acceptance work and preserves the latency/candidate
reduction behavior on AWS.

## Baseline Comparison

The prior AWS high-recall SPIRE reference is
`benchmarks/task73-74-aws-spire-quality-overhead-refresh-20260531/`, with
SPIRE `tg128/b0`, `nprobe=96`, recall@10 `0.9975`, p50 `134.458 ms`, p95
`149.487 ms`, and p99 `587.682 ms`.

Compared with that AWS reference, the Task 79 recommended/default
`global1152` point is:

- p50: `134.458 / 35.199 = 3.82x` faster
- p95: `149.487 / 36.203 = 4.13x` faster
- p99: `587.682 / 36.591 = 16.06x` faster

The older local TurboQuant comparison had `15,506,227` candidates at
`141.561 ms` p50. The AWS RaBitQ `global1152` point scans `3,672,619`
candidates, or about `4.22x` fewer.

## Artifact Inventory

- `suite.json`: checked-in suite config used by the successful run.
- `artifacts/suite-config.json`: suite config synced back from AWS.
- `artifacts/suite-dry-run-manifest.json`: local dry-run manifest.
- `artifacts/suite-manifest.json`: successful AWS suite manifest.
- `artifacts/suite-run.log`: successful remote suite log.
- `artifacts/results.jsonl`: raw successful AWS suite results.
- `artifacts/results-report.jsonl`: parsed report rows.
- `artifacts/precheck-host-inputs-and-gucs.log`: host/GUC/input precheck.
- `artifacts/load-100k-spire-rabitq-k3-block16-tg96-b0.log`: load/build log.
- `artifacts/pipeline-100k-rabitq-k3-deferred-global1024-rw025.log`: global1024 pipeline log.
- `artifacts/pipeline-100k-rabitq-k3-deferred-global1152-rw025.log`: global1152 pipeline log.
- `artifacts/pipeline-100k-rabitq-k3-deferred-global1216-rw025.log`: global1216 pipeline log.
- `artifacts/funnel-100k-rabitq-k3-deferred-global1024-rw025.jsonl`: global1024 funnel output.
- `artifacts/funnel-100k-rabitq-k3-deferred-global1152-rw025.jsonl`: global1152 funnel output.
- `artifacts/funnel-100k-rabitq-k3-deferred-global1216-rw025.jsonl`: global1216 funnel output.
- `artifacts/cloud-status-before.log`: pre-run profile status.
- `artifacts/cloud-up.log`: launch/resume command summary.
- `artifacts/cloud-install.log`: first install attempt, failed while cloud-init had not yet created `postgres`.
- `artifacts/cloud-install-retry.log`: second install attempt, failed while cloud-init was still completing toolchain setup.
- `artifacts/cloud-bench.log`: first bench attempt, failed against stale retained `postgres` database SQL objects.
- `artifacts/cloud-bench-rerun-freshdb.log`: successful bench command summary.
- `artifacts/cloud-pause.log`: shutdown command summary.
- `artifacts/cloud-status-after-pause.log`: final cloud status.
- `artifacts/ec2-status-after-pause.log`: EC2 status capture from AWS CLI.
- `artifacts/ec2-status-after-pause.txt`: plain EC2 stopped-state summary.

## Notes

- The successful suite ran on fresh database `task79_aws` after the retained
  `postgres` database exposed stale extension SQL objects from earlier runs.
- The failed install attempts are retained as troubleshooting artifacts only;
  cloud-init completed before the successful suite run.
- The `ecaz cloud` lifecycle log files for some commands did not mirror stdout
  in this run. Where that happened, the packet keeps a short summary artifact
  and the durable suite artifacts contain the actual benchmark output.
