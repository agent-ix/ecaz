# Manifest: SPIRE Phase 13e Local AWS Harness Gate

- head SHA: `61cd1aeef4edadfbe1ccb43abe1e715427a91fdc`
- task bucket: `reviews/task-30`
- packet: `reviews/task-30/989-spire-phase13e-local-aws-harness`
- lane: local PG18 AWS-harness correctness gate
- fixture: 1 coordinator + 3 remote local PostgreSQL instances
- corpus: AWS correctness shape, `10000 x dim 1536` corpus and `100 x dim 1536` queries
- storage format: `rabitq`
- rerank mode: default / production read only
- isolated one-index-per-table: yes
- timestamp: 2026-05-26

## Commands

Reproduction before fix:

```bash
bash scripts/run_spire_phase13e_aws_harness_local_pg18.sh --skip-install --artifact-dir reviews/task-30/989-spire-phase13e-local-aws-harness/artifacts/escalated-skip-install-run
```

Passing gate after fix:

```bash
bash scripts/run_spire_phase13e_aws_harness_local_pg18.sh --skip-install --artifact-dir reviews/task-30/989-spire-phase13e-local-aws-harness/artifacts/conninfo-before-start-run
```

Static validation:

```bash
bash -n scripts/run_spire_phase13e_aws_harness_local_pg18.sh scripts/spire-aws/install.sh scripts/spire-aws/smoke.sh
jq empty scripts/spire-aws/suite-correctness.json scripts/spire-aws/suite-representative.json scripts/spire-aws/suite-stress.json
```

## Artifacts

- `artifacts/escalated-skip-install-run/phase13e-aws-harness-local.log`
  - Reproduces the missing backend conninfo failure.
  - Key lines: `conninfo_secret_missing` during registration; `remote_candidate_receive_failed` at CustomScan.
- `artifacts/conninfo-before-start-run/phase13e-aws-harness-local.log`
  - Full passing local AWS-harness run.
  - Key lines: `Custom Scan (EcSpireDistributedScan)`, `remote_fanout: 3`, `status ready`, final pass marker.
- `artifacts/conninfo-before-start-run/smoke-customscan-read.log`
  - Focused smoke log with CustomScan plan and production read profile.
  - Key lines: `remote_fanout: 3`, `status ready`, `result_source remote_heap_candidates`.
- `artifacts/conninfo-before-start-run/production-read-profile-smoke.log`
  - Focused production read profile.
  - Key rows:
    - `selected_pid_count 10`
    - `remote_pid_count 10`
    - `dispatch_count 3`
    - `remote_heap_ready_dispatch_count 3`
    - `remote_heap_failed_dispatch_count 0`
    - `result_source remote_heap_candidates`
    - `status ready`
    - `socket_open_count 3`
    - `candidate_receive_query_count 3`
    - `heap_receive_query_count 3`
    - `remote_timeout_count 0`
    - `remote_cancel_count 0`
    - `degraded_skipped_dispatch_count 0`
- `artifacts/conninfo-before-start-run/bench-spire-pipeline-smoke.log`
  - Production read only bench smoke over `queries=5`, sweep `[8,16,32]`.
  - Key table: production read profile rows are `ready` and `remote_heap_candidates`.
- `artifacts/conninfo-before-start-run/register-remotes.log`
  - Registration sees remote conninfo; no `conninfo_secret_missing` warnings remain.
- `artifacts/conninfo-before-start-run/remote-node-snapshot-baseline.log`
  - Remote descriptor snapshot: node 2 has 34 placements, node 3 has 33, node 4 has 33.
- `artifacts/static-validation.log`
  - `bash -n` and `jq empty` validation: `static-validation=pass`.

Large generated leaf-assignment TSVs and generated corpus files are intentionally
left uncommitted; the durable evidence is the packet-local logs above.
