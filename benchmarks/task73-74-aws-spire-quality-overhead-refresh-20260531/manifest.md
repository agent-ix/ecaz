# Task 73/74 AWS SPIRE Quality + Overhead Refresh

- head SHA: `361f6bf64d997234eafe26564bce7501678623d5`
- branch: `task-73-spire-perf`
- packet: `benchmarks/task73-74-aws-spire-quality-overhead-refresh-20260531/`
- timestamp: `2026-05-31T16:45:35Z`
- lane: AWS Graviton `1m` retained stack
- DB host: `m7g.2xlarge`, PostgreSQL `18.3` on `aarch64-amazon-linux-gnu`
- runner: `ecaz cloud bench` driving `ecaz bench suite`
- remote binary: `/usr/local/bin/ecaz`, extension `ecaz 0.1.1`
- suite config: `benchmarks/task73-74-aws-spire-quality-overhead-refresh-20260531/suite.json`
- suite config SHA256: `cfd8238d7a3c5684035158be486adc0ba4b15a14030e1ee28c73bb8c00c82688`
- storage layout: isolated one-index-per-table surfaces
- cache policy: post-recall warm for IVF latency; SPIRE pipeline metrics from the suite runner
- S3 artifact source:
  `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task73-74-aws-spire-quality-overhead-refresh-20260531/20260531T165013Z/`

## Commands

Dry run:

```text
target/debug/ecaz bench suite run --dry-run --config benchmarks/task73-74-aws-spire-quality-overhead-refresh-20260531/suite.json --database postgres --manifest-output benchmarks/task73-74-aws-spire-quality-overhead-refresh-20260531/artifacts/suite-dry-run-manifest.json
```

AWS run:

```text
target/debug/ecaz cloud bench --profile 1m --suite task73-74-aws-spire-quality-overhead-refresh-20260531 --database postgres --config benchmarks/task73-74-aws-spire-quality-overhead-refresh-20260531/suite.json --ecaz-bin /usr/local/bin/ecaz --log-file benchmarks/task73-74-aws-spire-quality-overhead-refresh-20260531/artifacts/cloud-bench-1m-20260531T164535Z-usr-local.log
```

Report:

```text
target/debug/ecaz bench suite report --manifest benchmarks/task73-74-aws-spire-quality-overhead-refresh-20260531/artifacts/suite-manifest.json --results-output benchmarks/task73-74-aws-spire-quality-overhead-refresh-20260531/artifacts/results-report.jsonl
```

Cost guardrail:

```text
aws ec2 stop-instances --region us-west-2 --instance-ids i-0056e46b981edbb17 i-08b9ea039ef27adbc
target/debug/ecaz cloud status --profile 1m --json --log-file benchmarks/task73-74-aws-spire-quality-overhead-refresh-20260531/artifacts/cloud-status-1m-after-refresh.json
```

## Key Results

- Suite status: completed `8`, failed `0`, skipped `0`, missing artifacts `0`.
- SPIRE default `tg16/b0`, nprobe `16`: recall@10 `0.8525`, p50 `33.879 ms`,
  p95 `122.728 ms`, p99 `246.520 ms`.
- SPIRE high-recall `tg128/b0`, nprobe `96`: recall@10 `0.9975`,
  p50 `134.458 ms`, p95 `149.487 ms`, p99 `587.682 ms`.
- SPIRE ceiling `tg128/b0`, nprobe `128`: recall@10 `1.0000`,
  p50 `159.866 ms`, p95 `160.505 ms`, p99 `161.241 ms`.
- IVF control, nprobe `96`, heap rerank `500`: recall@10 `0.9980`,
  p50 `28.7 ms`, p95 `30.4 ms`, p99 `30.9 ms`.
- IVF control, nprobe `128`, heap rerank `500`: recall@10 `1.0000`,
  p50 `35.2 ms`, p95 `36.7 ms`, p99 `37.2 ms`.
- AWS refreshed p50 ratio: SPIRE/IVF is `4.69x` at nprobe `96` and `4.54x`
  at nprobe `128`.
- The `1m` stack was stopped after the run; status reports `state=paused`,
  DB instance `stopped`, and estimated running compute `$0.00/hr`.

## Artifacts

- Suite config: `suite.json`
- Dry-run manifest: `artifacts/suite-dry-run-manifest.json`
- Suite run log: `artifacts/suite-run.log`
- Suite manifest: `artifacts/suite-manifest.json`
- Parsed results: `artifacts/results.jsonl`
- Report rows: `artifacts/results-report.jsonl`
- AWS runner logs:
  - `artifacts/cloud-bench-1m-20260531T164535Z.log`
  - `artifacts/cloud-bench-1m-20260531T164535Z-rerun.log`
  - `artifacts/cloud-bench-1m-20260531T164535Z-usr-local.log`
- Precheck: `artifacts/precheck-host-and-inputs.log`
- SPIRE logs:
  - `artifacts/load-100k-spire-default-tg16-b0.log`
  - `artifacts/pipeline-100k-spire-default-tg16-b0.log`
  - `artifacts/load-100k-spire-highrecall-tg128-b0.log`
  - `artifacts/pipeline-100k-spire-highrecall-tg128-b0.log`
- IVF logs:
  - `artifacts/load-100k-ivf-control.log`
  - `artifacts/recall-100k-ivf-control.log`
  - `artifacts/latency-100k-ivf-control.log`
- Cost guardrail: `artifacts/cloud-status-1m-after-refresh.json`

## Notes

The first two `cloud bench` attempts did not run the suite: the first remote
attempt was missing the new packet-local suite file, and the second used the
wrong remote binary path (`target/debug/ecaz`). The successful run is the
`usr-local` log above and used `/usr/local/bin/ecaz`.
