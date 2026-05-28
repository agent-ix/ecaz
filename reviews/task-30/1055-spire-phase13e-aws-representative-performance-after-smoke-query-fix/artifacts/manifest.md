# Task 30 Packet 1055 Artifact Manifest

- head SHA: `d54695456193158aba17b9740fba3bf8fa6c9070`
- task bucket: `reviews/task-30/1055-spire-phase13e-aws-representative-performance-after-smoke-query-fix`
- timestamp: `2026-05-28T20:20:26Z`
- lane: Phase 13e AWS representative performance
- fixture: real representative corpus, Graviton AWS, 1 coordinator plus 3 remotes
- AWS shape: `us-west-2`, `us-west-2a`, `m7g.large` coordinator, 3 x `m7g.large` remotes
- storage format: `rabitq`
- rerank mode: default suite settings
- table surface: representative prefix `ec_spire_aws_repr_1m`

## Curated Artifacts

- `artifacts/run-representative-performance-pass.log`
  - command: `scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir reviews/task-30/1055-spire-phase13e-aws-representative-performance-after-smoke-query-fix/artifacts --execute`
  - key lines: `profile=ec_real_100k corpus_rows=100000 query_rows=1000`; `published_static_remote_placements`; `remote_fanout: 3`; production read profile `result_source remote_heap_candidates`; `ERROR: ec_spire production executor cannot merge remote heap candidates while node_id 3 is in state CandidateReceiveFailed with status remote_candidate_receive_failed`; `teardown complete and Terraform state is clean`

- `artifacts/aws-running-before.log`
  - command: `aws ec2 describe-instances --region us-west-2 --filters Name=instance-state-name,Values=pending,running,stopping ...`
  - key result: no pending/running/stopping EC2 instances printed before provisioning

- `artifacts/aws-running-after-failure.log`
  - command: `aws ec2 describe-instances --region us-west-2 --filters Name=instance-state-name,Values=pending,running,stopping ...`
  - key result: no pending/running/stopping EC2 instances printed after teardown

- `artifacts/coordinator-load-representative.log`
  - command: representative coordinator node-local load
  - key result: loaded 100,000 corpus rows and 1,000 query rows; built `ec_spire_aws_repr_1m_idx` in 134.84s; completed in 202.97s

- `artifacts/remote-node-2-load-representative.log`, `artifacts/remote-node-3-load-representative.log`, `artifacts/remote-node-4-load-representative.log`
  - command: representative remote node-local loads
  - key result: remote shards loaded and indexed before materialization

- `artifacts/placement-remotes.json`
  - command: distributed placement registration output
  - key result: remote node IDs `2`, `3`, and `4`

- `artifacts/remote-leaf-materialization/node-2-remote-materialize.log`
  - command: remote leaf materialization for node 2
  - key result: materialized 34,589 rows

- `artifacts/remote-leaf-materialization/node-3-remote-materialize.log`
  - command: remote leaf materialization for node 3
  - key result: materialized 32,930 rows

- `artifacts/remote-leaf-materialization/node-4-remote-materialize.log`
  - command: remote leaf materialization for node 4
  - key result: materialized 32,481 rows

- `artifacts/coordinator-placement-snapshot-after-remote-publish.log`
  - command: placement snapshot after remote publish
  - key result: one local placement plus ready remote placements for node IDs 2, 3, and 4

- `artifacts/smoke-customscan-read.log`
  - command: `scripts/spire-aws/smoke-customscan-read.sql`
  - key result: `EcSpireDistributedScan`, `remote_fanout: 3`, execution through CustomScan

- `artifacts/production-read-profile-smoke.log`
  - command: production read profile smoke query
  - key result: `status ready`, `result_source remote_heap_candidates`, `final_heap_fetch_status remote_ready`, `returned_candidate_count 10`

- `artifacts/bench-spire-pipeline-smoke.log`
  - command: `ecaz bench spire-pipeline --queries-limit 5 --sweep 8,16,32 --include-remote --include-recall --include-production-read-profile --production-read-only`
  - key result: 5-query smoke recall/latency and production-read profile ready for all swept nprobes

- `artifacts/13a3a-recall-k10.log`
  - command: representative priority suite `k=10` recall step over 1,000 real queries
  - key result: recall@10 `0.7868`, `0.8626`, `0.8962`, `0.9187` for nprobe `8`, `16`, `24`, `32`

- `artifacts/suite-manifest-representative-priority.json`
  - command: `ecaz bench suite run` for representative priority suite
  - key result: `13a3a-recall-k10` completed; `13a3a-recall-k100` failed before later priority steps

- `artifacts/pg-stat-activity-during-smoke-bench.log` through `artifacts/pg-stat-activity-during-smoke-bench-7.log`
  - command: coordinator `pg_stat_activity` snapshots
  - key result: smoke recall setup spent time in `SELECT id, source FROM ec_spire_aws_repr_1m_corpus ORDER BY id` with `ClientWrite`

- `artifacts/pg-stat-activity-during-priority-recall-1.log` through `artifacts/pg-stat-activity-during-priority-recall-7.log`
  - command: coordinator `pg_stat_activity` snapshots
  - key result: priority `k=10` recall setup repeated the same full-corpus SSM transfer

- `artifacts/pg-stat-activity-during-priority-recall-k100-1.log` through `artifacts/pg-stat-activity-during-priority-recall-k100-6.log`
  - command: coordinator `pg_stat_activity` snapshots
  - key result: priority `k=100` recall setup repeated the same full-corpus SSM transfer before the remote candidate receive failure

## Deliberately Not Committed

The raw packet directory also contains generated tarballs, vendored source build trees, TLS keys/certs, and real corpus work files. Those are left local and are not part of the curated review evidence.
