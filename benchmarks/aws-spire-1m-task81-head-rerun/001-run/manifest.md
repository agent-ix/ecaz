# AWS SPIRE 1M Task 81 HEAD Rerun

- head SHA: `e0622e50a84729f020eab1da7e2f964a0b1aae98`
- branch: `task-81-spire-leaf-block-summary-format`
- packet: `benchmarks/aws-spire-1m-task81-head-rerun/001-run`
- date: `2026-06-05`
- lane: AWS Graviton / PG18 / retained `1m` profile
- database: `postgres`
- DB instance: `i-06ace3e95ab942623`
- S3 bucket: `ecaz-cloud-1m-b62eb804`
- corpus table: `task67_1m_hnsw_m7g2xlarge_corpus`
- query table: `task67_1m_hnsw_m7g2xlarge_queries`
- corpus rows: `990000`
- query rows: `10000`
- index: `aws_spire_1m_rabitq_t80_block16_tg256_idx`
- storage format: `rabitq`
- index reloptions: `nlists=128`, `recursive_fanout=8`, `top_graph_search_list_size=256`, `rerank_width=25`
- surface mode: retained AWS 1M shared-table surface
- runner: `ecaz bench suite` via `ecaz cloud bench`
- AWS final state: `paused`, `$0.00/hr running`

## Purpose

Rerun the Task 81 1M/q500 SPIRE path from current branch HEAD after the 100k
closeout. The accepted run uses the retained 1M Task 80/81 SPIRE index and the
q500 exact-truth cache to check whether the optimized global block cap shape
moves recall or latency at 1M scale.

## Commands

Audit:

```sh
target/debug/ecaz bench suite audit \
  --config benchmarks/aws-spire-1m-task81-head-rerun/001-run/suite-q500-nprobe96-light.json \
  --log-file benchmarks/aws-spire-1m-task81-head-rerun/001-run/artifacts/suite-audit-light.log
```

Resume and install current branch while preserving retained tables:

```sh
target/debug/ecaz cloud resume --profile 1m --database postgres \
  --log-file benchmarks/aws-spire-1m-task81-head-rerun/001-run/artifacts/cloud-resume-before-task81-head.log

target/debug/ecaz cloud install --profile 1m \
  --git-ref task-81-spire-leaf-block-summary-format \
  --database postgres \
  --skip-extension-recreate \
  --log-file benchmarks/aws-spire-1m-task81-head-rerun/001-run/artifacts/cloud-install-task81-head.log
```

Accepted light q500 run:

```sh
target/debug/ecaz cloud bench --profile 1m --database postgres \
  --config benchmarks/aws-spire-1m-task81-head-rerun/001-run/suite-q500-nprobe96-light.json \
  --suite aws-spire-1m-task81-head-q500-nprobe96-light \
  --ecaz-bin /usr/local/bin/ecaz \
  --log-file benchmarks/aws-spire-1m-task81-head-rerun/001-run/artifacts/cloud-bench-task81-head-q500-nprobe96-light.log
```

Report and final AWS state:

```sh
target/debug/ecaz bench suite status \
  --manifest benchmarks/aws-spire-1m-task81-head-rerun/001-run/artifacts/task81-head-q500-nprobe96-light/suite-manifest.json \
  --log-file benchmarks/aws-spire-1m-task81-head-rerun/001-run/artifacts/task81-head-q500-nprobe96-light/suite-status.log

target/debug/ecaz bench suite report \
  --manifest benchmarks/aws-spire-1m-task81-head-rerun/001-run/artifacts/task81-head-q500-nprobe96-light/suite-manifest.json \
  --results-output benchmarks/aws-spire-1m-task81-head-rerun/001-run/artifacts/task81-head-q500-nprobe96-light/results-report.jsonl \
  --log-file benchmarks/aws-spire-1m-task81-head-rerun/001-run/artifacts/task81-head-q500-nprobe96-light/suite-report.md

target/debug/ecaz cloud pause --profile 1m --database postgres \
  --log-file benchmarks/aws-spire-1m-task81-head-rerun/001-run/artifacts/cloud-pause-after-light-run.log

script -q -c "target/debug/ecaz cloud status --profile 1m --database postgres" \
  benchmarks/aws-spire-1m-task81-head-rerun/001-run/artifacts/cloud-status-after-light-run-paused.script.log
```

## Accepted Result

Suite:
`benchmarks/aws-spire-1m-task81-head-rerun/001-run/suite-q500-nprobe96-light.json`

Artifacts:
`benchmarks/aws-spire-1m-task81-head-rerun/001-run/artifacts/task81-head-q500-nprobe96-light/`

Key q500 row:

```text
nprobe: 96
effective_nprobe: 96
queries: 500
truth cache: benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/truth-aws-real-1m-q500-k10.json
leaf_block_pruning_max_global_blocks: 1152
rerank_width: 25
candidate_sum: 9,213,846
ready_sum: 12,500
heap_rerank_sum: 12,500
route_sum: 48,000
recall@10: 0.9832
latency_min: 164.209 ms
latency_p50: 250.251 ms
latency_p95: 307.810 ms
latency_p99: 322.053 ms
latency_max: 331.759 ms
suite step duration: 379,197 ms
```

Precheck:

```text
PostgreSQL 18.3 on aarch64-amazon-linux-gnu
corpus_rows: 990000
query_rows: 10000
aws_spire_1m_rabitq_t80_block16_tg256_idx | ec_spire | 872 MB
task67_1m_hnsw_m7g2xlarge_m16_idx         | ec_hnsw  | 1289 MB
```

Suite status:

```text
completed=2 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Comparison

Prior Task 81 AWS 1M q500 packet 003 on the same retained index shape:

```text
nprobe=96
candidate_sum=9,213,846
latency_p50=265.911 ms
latency_p95=329.407 ms
latency_p99=342.454 ms
recall@10=0.9832
```

Current HEAD light rerun:

```text
nprobe=96
candidate_sum=9,213,846
latency_p50=250.251 ms
latency_p95=307.810 ms
latency_p99=322.053 ms
recall@10=0.9832
```

Readout: current HEAD is faster on this lighter q500 measurement shape, but
recall remains unchanged at `0.9832`. This does not clear the 1M high-recall
gap; it reinforces that the remaining issue is recall/coverage at 1M, not just
local candidate materialization overhead.

## Cancelled Diagnostic Sweep

`suite-q500-nprobe96-128.json` attempted a heavier q500 sweep over
`nprobe=96,128` with production-read profiling and local-store overlap. It was
cancelled after the pipeline remained a single CPU-bound PostgreSQL `SELECT`
for about 15 minutes without producing pipeline artifacts. That run is not used
as a result row; it is retained only as evidence that the diagnostic-heavy
sweep shape is too expensive for the immediate 1M answer.

The initial heavy-suite attempt also failed immediately due to an invalid
`ALTER EXTENSION ... DROP FUNCTION IF EXISTS` repair SQL statement. The suite
config was corrected before the cancelled rerun and the accepted light run did
not require the repair step.
