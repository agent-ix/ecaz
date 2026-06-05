# Task 80 AWS 1M Block16 Pruning Manifest

Status: closeout complete; AWS profile paused

- Head SHA: `6a991b7e0ffd04acdcd0e2b57beeecf8bfb7b4cb`
- Task bucket: `reviews/task-80/001-aws-1m-block16-pruning/`
- Suite configs:
  - `reviews/task-80/001-aws-1m-block16-pruning/suite-aws-1m-block16-pruning.json`
  - `reviews/task-80/001-aws-1m-block16-pruning/suite-aws-1m-block16-pruning-continuation.json`
- Artifact directories:
  - `reviews/task-80/001-aws-1m-block16-pruning/artifacts/aws-1m-block16-pruning-query-after-repair/`
  - `reviews/task-80/001-aws-1m-block16-pruning/artifacts/aws-1m-block16-pruning-continuation/`
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
- Continuation suite scaffold:
  `target/debug/ecaz bench suite --config reviews/task-80/001-aws-1m-block16-pruning/suite-aws-1m-block16-pruning-continuation.json --dry-run`
- Continuation global2048 SSM run:
  `aws ssm send-command --cli-input-json file://reviews/task-80/001-aws-1m-block16-pruning/artifacts/ssm-task80-continuation-global2048-input.json`
  (`79090a9a-8462-4908-b831-38f238ab27f5`)
- Continuation artifact sync:
  `aws s3 sync s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task80-aws-1m-block16-pruning-continuation-global2048/ reviews/task-80/001-aws-1m-block16-pruning/artifacts/aws-1m-block16-pruning-continuation --region us-west-2 --only-show-errors`
- Pause profile:
  `target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-80/001-aws-1m-block16-pruning/artifacts/cloud-pause-after-task80.log`
- Pause profile after continuation:
  `target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-80/001-aws-1m-block16-pruning/artifacts/cloud-pause-after-task80-continuation.log`

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
- `suite-aws-1m-block16-pruning-continuation.json`: continuation suite config
  with no build/drop-index step; runs retained-index query rows only.
- `aws-1m-block16-pruning-continuation/`: successful continuation artifacts for
  global block cap `2048`.
- `ssm-continuation-global2048-success.json`: continuation SSM invocation
  result; `Status=Success`, `ResponseCode=0`,
  `ExecutionElapsedTime=PT41M54.992S`.
- `ec2-status-after-task80-continuation-pause.log`: direct EC2 stopped-state
  evidence for both retained profile `1m` instances after the continuation.

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
- Completed continuation row, global block cap `2048`, q500:
  - nprobe `96`: recall@10 `0.9914`, p50 `308.087 ms`, p95 `357.431 ms`,
    candidates `16,379,614`, heap rerank `12,500`.
  - nprobe `128`: recall@10 `0.9918`, p50 `334.571 ms`, p95 `413.553 ms`,
    candidates `16,379,623`, heap rerank `12,500`.
  - nprobe `256`: recall@10 `0.9918`, p50 `334.867 ms`, p95 `413.172 ms`,
    candidates `16,379,623`, heap rerank `12,500`.
- The `2048` row materially improves recall over the old tg96 AWS 1M row
  (`0.9832`) but loses the Task 80 candidate/latency objective: q500 candidates
  rise from the old `9,213,846` shape to about `16.38M`, and p50 remains above
  the old `268.824 ms` comparator. Higher caps `4096` and `8192` were not run
  because this mechanism spends more block budget rather than reducing the
  candidate surface.
