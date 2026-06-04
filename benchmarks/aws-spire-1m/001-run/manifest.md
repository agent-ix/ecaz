# AWS SPIRE 1M Benchmark Manifest

- head SHA: `010520a9e6cdeb07a7384a6505a44e65f08d80a5`
- task bucket / packet path: `benchmarks/aws-spire-1m/001-run`
- lane: `aws-graviton`
- host profile: `1m`
- database: `postgres`
- AWS region: `us-west-2`
- bucket: `s3://ecaz-cloud-1m-b62eb804`
- run timestamp: `2026-06-03T19:22:23Z`
- suite config: `benchmarks/aws-spire-1m/001-run/suite-minimal.json`
- suite name: `aws-spire-1m-rabitq-global1152-minimal`
- storage format: `rabitq`
- rerank mode: `rerank_width=25`
- index surface: isolated one-index-per-table surface on `task67_1m_hnsw_m7g2xlarge_corpus`

## Commands

Audit:

```bash
target/debug/ecaz bench suite audit --config benchmarks/aws-spire-1m/001-run/suite-minimal.json
```

Cloud benchmark:

```bash
target/debug/ecaz cloud bench --profile 1m --database postgres \
  --config benchmarks/aws-spire-1m/001-run/suite-minimal.json \
  --suite aws-spire-1m-rabitq-global1152-minimal \
  --ecaz-bin /usr/local/bin/ecaz \
  --log-file benchmarks/aws-spire-1m/001-run/artifacts/cloud-bench-minimal.log
```

Report:

```bash
target/debug/ecaz bench suite report \
  --manifest benchmarks/aws-spire-1m/001-run/artifacts/minimal/suite-manifest.json \
  --results-output benchmarks/aws-spire-1m/001-run/artifacts/minimal/results-report.jsonl \
  --log-file benchmarks/aws-spire-1m/001-run/artifacts/minimal/suite-report.md
```

Pause:

```bash
target/debug/ecaz cloud pause --profile 1m --database postgres \
  --log-file benchmarks/aws-spire-1m/001-run/artifacts/cloud-pause-after-minimal.log
```

## Artifacts

- `artifacts/suite-minimal-audit.log`: suite audit, passed 3 steps.
- `artifacts/cloud-bench-minimal.log`: cloud wrapper log; synced from `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/aws-spire-1m-rabitq-global1152-minimal/20260603T192223Z/`.
- `artifacts/minimal/suite-run.log`: suite execution log.
- `artifacts/minimal/suite-manifest.json`: structured suite manifest; all 3 steps succeeded.
- `artifacts/minimal/precheck-existing-spire-1m-index.log`: precheck row counts and index sizes.
- `artifacts/minimal/results.jsonl`: normalized pipeline and storage result rows.
- `artifacts/minimal/results-report.jsonl`: report output placeholder; no
  additional rows were emitted because the synced suite manifest retained
  `${artifact_dir}` placeholders for two step-local expected artifacts.
- `artifacts/minimal/suite-report.md`: markdown suite report.
- `artifacts/cloud-pause-after-minimal.log`: pause request for DB and loader instances.
- `artifacts/cloud-status-final-stopped.log`: final cloud status, `state: paused`, `$0.00/hr running`.

Note: the suite report records two missing expected artifacts because the step-local
`log_output` / `log_file` values with `${artifact_dir}` were not expanded before
artifact validation. The normalized `results.jsonl` rows did sync and are the
source of truth for the pipeline and storage metrics below.

## Key Results

Precheck:

```text
captured_at: 2026-06-03 19:22:23.120245+00
PostgreSQL 18.3 on aarch64-amazon-linux-gnu
corpus_rows: 990000
query_rows: 10000
aws_spire_1m_rabitq_global1152_idx | ec_spire | 872 MB
task67_1m_hnsw_m7g2xlarge_m16_idx  | ec_hnsw  | 1289 MB
```

Suite manifest:

```text
precheck-existing-spire-1m-index: succeeded, 40 ms
pipeline-spire-1m-rabitq-global1152-minimal: succeeded, 134092 ms
storage-spire-1m-rabitq-global1152-minimal: succeeded, 39 ms
```

SPIRE pipeline result rows from `artifacts/minimal/results.jsonl`:

```text
nprobe: 96
rerank_width: 25
queries: 200
candidate stage candidate_sum: 3685287
candidate stage ready_sum: 5000
candidate stage next_blocker: candidate_budget
heap_rerank heap_rerank_sum: 5000
latency_min: 193.739 ms
latency_p50: 286.677 ms
latency_p95: 348.602 ms
latency_p99: 356.936 ms
latency_max: 378.486 ms
```

Storage result rows from `artifacts/minimal/results.jsonl`:

```text
rows: 990000
table total: 17.5 GiB
ec_spire aws_spire_1m_rabitq_global1152_idx: 872.1 MiB, 923.7 B/row
ec_hnsw task67_1m_hnsw_m7g2xlarge_m16_idx: 1.3 GiB, 1365.3 B/row
btree task67_1m_hnsw_m7g2xlarge_corpus_pkey: 21.2 MiB, 22.5 B/row
```

Final AWS state:

```text
profile:  1m
state:    paused
db:       10.42.1.131 (i-06ace3e95ab942623)
bucket:   ecaz-cloud-1m-b62eb804
cost:     ~$0.00/hr running, ~$8.00/mo retained storage
```

## Notes

- Earlier full-recall and no-recall diagnostic attempts are preserved in
  `artifacts/cloud-bench-rerun.log` and `artifacts/cloud-bench-no-recall.log`.
- The full-recall run built the SPIRE index but lost SSM connectivity during the
  heavier benchmark stage.
- The no-recall diagnostic run failed fast because the installed extension did
  not expose `ec_spire_index_scan_leaf_candidate_snapshot`; this minimal suite
  avoids that optional funnel diagnostic and benchmarks the existing 1M SPIRE
  index successfully.
- On 2026-06-04, `suite-recall-10.json` attempted a 10-query
  `--include-recall` smoke. It produced no synced artifacts or stdout within
  approximately 10 minutes, so AWS was paused and the stale local wrapper was
  terminated. This indicates the current recall path is dominated by exact
  truth generation over the full 990k corpus before the suite can emit results;
  full 200-query recall should use a precomputed truth cache/export path rather
  than the naive `spire-pipeline --include-recall` path.
