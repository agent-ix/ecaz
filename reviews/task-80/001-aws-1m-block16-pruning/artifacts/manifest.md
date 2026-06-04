# Task 80 AWS 1M Block16 Pruning Manifest

Status: partial AWS run complete; AWS profile paused

- Head SHA: `d12d472efd6ed499acd555e750f47cb938701658`
- Task bucket: `reviews/task-80/001-aws-1m-block16-pruning/`
- Suite config: `reviews/task-80/001-aws-1m-block16-pruning/suite-aws-1m-block16-pruning.json`
- Artifact directory: `reviews/task-80/001-aws-1m-block16-pruning/artifacts/aws-1m-block16-pruning-query-after-repair/`
- Lane: AWS 1M, PG18, SPIRE, RaBitQ
- Fixture: retained `task67_1m_hnsw_m7g2xlarge` corpus and queries in AWS profile `1m`
- Surface: shared retained 1M table with one active SPIRE index after the build step
- Index shape: `nlists=128`, `recursive_fanout=8`, `storage_format=rabitq`,
  `boundary_replica_count=0`, `top_graph_search_list_size=256`,
  `ec_spire.leaf_block_rows=16`
- Query shape: q500, `rerank_width=25`, nprobe sweep `96,128,256`, global
  block caps `1152,2048,4096,8192`, production read profile enabled
- Truth cache: `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/truth-aws-real-1m-q500-k10.json`

## Commands

- Resume profile:
  `target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-80/001-aws-1m-block16-pruning/artifacts/cloud-resume-before-task80.log`
- Cloud suite attempt:
  `target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-80/001-aws-1m-block16-pruning/suite-aws-1m-block16-pruning.json --suite task80-aws-1m-block16-pruning --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-80/001-aws-1m-block16-pruning/artifacts/cloud-bench-task80-aws-1m-block16-pruning.log`
- Query retry before catalog repair:
  `aws ssm send-command --cli-input-json file://reviews/task-80/001-aws-1m-block16-pruning/artifacts/ssm-task80-query-retry-input.json`
  (`1fc54f96-34d2-43c8-97e6-749e152627e4`)
- Catalog repair for missing pgrx SQL function:
  `aws ssm send-command --cli-input-json file://reviews/task-80/001-aws-1m-block16-pruning/artifacts/ssm-task80-register-candidate-snapshot-input.json`
  (`0860d480-fcd1-42b2-981e-36bbc594df6c`)
- Post-repair query retry:
  `aws ssm send-command --cli-input-json file://reviews/task-80/001-aws-1m-block16-pruning/artifacts/ssm-task80-query-after-repair-input.json`
  (`5a6cb359-3556-4a0c-8b81-a728e9f4a626`)
- Explicit remote artifact sync after SSM timeout:
  `aws ssm send-command --instance-ids i-06ace3e95ab942623 --document-name AWS-RunShellScript --comment task80-sync-after-timeout ...`
  (`8688844d-f272-4006-b218-bbcd757f4548`)
- Pause profile:
  `target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-80/001-aws-1m-block16-pruning/artifacts/cloud-pause-after-task80.log`

## Artifacts

- `cloud-resume-before-task80.log`: resumed profile `1m`.
- `cloud-bench-task80-aws-1m-block16-pruning.log`: cloud wrapper run that built
  the index but failed before clean query results.
- `aws-1m-block16-pruning-cloud-wrapper-failed/`: wrapper artifacts including
  successful index build.
- `aws-1m-block16-pruning-query-retry/`: pre-repair retry artifacts; pipeline
  steps failed because `ec_spire_index_scan_leaf_candidate_snapshot(oid, real[])`
  was absent from the retained AWS extension catalog.
- `ssm-register-candidate-snapshot.json`: catalog repair result; PostgreSQL
  reported `CREATE FUNCTION`, `ALTER EXTENSION`, and signature `oid, real[]`.
- `aws-1m-block16-pruning-query-after-repair/`: post-repair artifacts; one full
  q500 pipeline completed for global block cap `1152`, then SSM timed out while
  the `2048` pipeline was still running.
- `ssm-query-after-repair-timeout.json`: post-repair SSM invocation result;
  `Status=TimedOut`, `ResponseCode=137`, `ExecutionElapsedTime=PT1H0.002S`.
- `cloud-pause-after-task80.log` and `cloud-status-after-task80-pause.log`: AWS
  profile `1m` paused after the run.

## Key Results

- Index build succeeded on 990,000 corpus rows:
  `aws_spire_1m_rabitq_t80_block16_tg256_idx`, `ec_spire`, `872 MB`,
  `top_graph_search_list_size=256`, total build `1,704,036 ms`.
- Storage snapshot after the build reported the SPIRE index as `872.1 MiB`,
  `923.7 B` per row.
- Completed post-repair row, global block cap `1152`, q500:
  - nprobe `96`: recall@10 `0.9832`, p50 `301.121 ms`, p95 `377.482 ms`,
    candidates `9,213,846`, heap rerank `12,500`.
  - nprobe `128`: recall@10 `0.9832`, p50 `320.201 ms`, p95 `422.479 ms`,
    candidates `9,213,838`, heap rerank `12,500`.
  - nprobe `256`: recall@10 `0.9832`, p50 `319.523 ms`, p95 `417.068 ms`,
    candidates `9,213,838`, heap rerank `12,500`.
- The remaining global caps (`2048`, `4096`, `8192`) did not complete within
  the one-hour SSM invocation window. The `2048` pipeline was killed by the SSM
  timeout before writing its own pipeline/funnel artifacts.
