# vchord G4 Warm Probe Sweep

- Timestamp: 2026-06-16T23:35:39Z
- Head SHA: `74ebaacd4d21fa8101ea64f036b5fc9df2cee4e2`
- Branch: `task-108-109-comparator-unification`
- Packet: `benchmarks/comparators-vchord-warm-g4`
- Host lane: AWS Graviton, `10k-medium` profile, DB host `m8g.2xlarge`, loader `c8g.medium`
- Database: `tqvector_bench`, PostgreSQL 18 socket `/var/run/postgresql`
- Source snapshot restored: `snap-0e9c7743263e61d70`
- S3 run prefix: `s3://ecaz-cloud-10k-medium-268ea93e/bench-artifacts/vchord-g4-warm/20260616T224554Z/`
- Surface isolation: comparator sidecar tables, one vchord table/index per fixture prefix
- Storage/rerank fixtures:
  - `real_50k_ivf_rabitq`, lists `224`
  - `real_100k_ivf_rabitq1_rerank`, lists `320`
  - `real_1m_ivf_rabitq1_rerank`, lists `1024`
- Query policy: `k=10`, `queries_limit=200`, probe sweep `1,4,16,64`

## Warm Policy

This packet is a warm benchmark. Each fixture runs a rebuild warmup step first:

- `warmup-vchord-50k-probe-sweep --rebuild`
- `warmup-vchord-100k-probe-sweep --rebuild`
- `warmup-vchord-1m-probe-sweep --rebuild`

The cited result rows are the subsequent measured steps without `--rebuild`:

- `measured-warm-vchord-50k-probe-sweep`
- `measured-warm-vchord-100k-probe-sweep`
- `measured-warm-vchord-1m-probe-sweep`

The 1M warmup was observed through SSM progress captures moving through:

- `INSERT INTO real_1m_ivf_rabitq1_rerank_corpus_vchord ...`
- `CREATE INDEX real_1m_ivf_rabitq1_rerank_corpus_vchord_idx ... USING vchordrq`
- `measured-warm-vchord-1m-probe-sweep` without `--rebuild`

## Commands

```bash
target/debug/ecaz bench suite audit \
  --config benchmarks/comparators-vchord-warm-g4/suite.json \
  --log-file benchmarks/comparators-vchord-warm-g4/artifacts/suite-audit-debug.log

target/debug/ecaz cloud up \
  --profile 10k-medium \
  --from-snapshot snap-0e9c7743263e61d70 \
  --git-ref task-108-109-comparator-unification \
  --confirm-cost 8 \
  --log-file benchmarks/comparators-vchord-warm-g4/artifacts/cloud-up-valid-snapshot.log

target/debug/ecaz cloud install \
  --profile 10k-medium \
  --git-ref task-108-109-comparator-unification \
  --skip-extension-recreate \
  --timeout 3600 \
  --log-file benchmarks/comparators-vchord-warm-g4/artifacts/cloud-install.log

aws ssm send-command ... install pgvector/vchord comparator prerequisites

target/debug/ecaz cloud bench \
  --profile 10k-medium \
  --database tqvector_bench \
  --suite vchord-g4-warm \
  --config benchmarks/comparators-vchord-warm-g4/suite.json \
  --ecaz-bin /usr/local/bin/ecaz \
  --log-file benchmarks/comparators-vchord-warm-g4/artifacts/cloud-bench.log

aws s3 sync \
  s3://ecaz-cloud-10k-medium-268ea93e/bench-artifacts/vchord-g4-warm/20260616T224554Z/ \
  benchmarks/comparators-vchord-warm-g4/artifacts \
  --region us-west-2 --only-show-errors

target/debug/ecaz cloud down \
  --profile 10k-medium \
  --yes \
  --no-snapshot-required \
  --log-file benchmarks/comparators-vchord-warm-g4/artifacts/cloud-down.log
```

Teardown removed the EC2 instances and EBS data volume. Terraform then failed to delete
`ecaz-cloud-10k-medium-268ea93e` because it also contains an older
`vchord-g4-probe/20260616T191230Z` artifact prefix. Post-teardown verification found
both instances terminated and the data volume deleted.

## Artifacts

- `suite.json`: checked-in SuiteConfig.
- `artifacts/suite-audit-debug.log`, `artifacts/suite-audit.log`: suite audit evidence.
- `artifacts/cloud-up-valid-snapshot.log`: successful stack creation from `snap-0e9c7743263e61d70`.
- `artifacts/cloud-install.log`: branch install on the DB host.
- `artifacts/install-comparator-prereqs-commands.json`: SSM command body for pgvector/vchord prerequisites.
- `artifacts/cloud-bench.log`: local cloud bench wrapper log.
- `artifacts/ssm-bench-final.json`: final SSM invocation, status `Success`, elapsed `PT43M46.509S`.
- `artifacts/suite-manifest.json`: remote suite manifest.
- `artifacts/results.jsonl`: structured suite results.
- `artifacts/suite-run.log`: remote suite runner log.
- `artifacts/warmup-vchord-*.log`: warmup comparator logs.
- `artifacts/measured-warm-vchord-*.log`: measured warm comparator logs.
- `artifacts/ssm-inspect-vchord-warm-*.json`: packet-local SSM progress captures proving the 1M warmup and measured transition.
- `artifacts/cloud-down.log`, `artifacts/cloud-status-after-down.log`: teardown evidence.

Earlier failed setup attempts are recorded in:

- `artifacts/cloud-up.log`: failed because stale snapshot `snap-0f546929f70d60fb5` was missing.
- `artifacts/cloud-down-after-up-fail.log`: cleanup after the failed stack creation attempt.
- `artifacts/cloud-status-after-up-fail.log`: status snapshot after failed setup cleanup.

## Key Results

Measured warm rows only:

| Step | Probes | Recall@10 | NDCG@10 | p50 | p95 | p99 | Mean |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k | 1 | 0.5010 | 0.9345 | 0.36 ms | 0.47 ms | 0.56 ms | 0.34 ms |
| 50k | 4 | 0.7745 | 0.9789 | 0.30 ms | 0.48 ms | 0.54 ms | 0.33 ms |
| 50k | 16 | 0.9045 | 0.9926 | 0.47 ms | 0.64 ms | 0.76 ms | 0.49 ms |
| 50k | 64 | 0.9830 | 0.9988 | 1.05 ms | 1.19 ms | 1.40 ms | 1.05 ms |
| 100k | 1 | 0.3975 | 0.8956 | 0.28 ms | 0.50 ms | 0.62 ms | 0.33 ms |
| 100k | 4 | 0.6530 | 0.9511 | 0.38 ms | 0.55 ms | 0.60 ms | 0.40 ms |
| 100k | 16 | 0.8475 | 0.9828 | 0.61 ms | 0.80 ms | 0.88 ms | 0.63 ms |
| 100k | 64 | 0.9470 | 0.9962 | 1.49 ms | 1.70 ms | 1.75 ms | 1.50 ms |
| 1M | 1 | 0.6350 | 0.9559 | 2.33 ms | 6.77 ms | 10.2 ms | 2.98 ms |
| 1M | 4 | 0.8445 | 0.9848 | 1.33 ms | 3.02 ms | 6.17 ms | 2.18 ms |
| 1M | 16 | 0.9325 | 0.9955 | 1.98 ms | 3.03 ms | 3.95 ms | 2.05 ms |
| 1M | 64 | 0.9740 | 0.9985 | 6.17 ms | 9.88 ms | 11.0 ms | 6.41 ms |

Build and storage rows:

| Fixture | Index | Build Seconds | Index Bytes |
| --- | --- | ---: | ---: |
| 50k | `real_50k_ivf_rabitq_corpus_vchord_idx` | 9.720000 | 430678016 |
| 100k | `real_100k_ivf_rabitq1_rerank_corpus_vchord_idx` | 23.730000 | 856301568 |
| 1M | `real_1m_ivf_rabitq1_rerank_corpus_vchord_idx` | 275.570000 | 8396496896 |
