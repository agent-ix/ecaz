# Artifact Manifest: Phase 13e AWS SSM Tunnel Fault Restore

- Head SHA: `d4c46b3574ee013ce42b29891bb9918eca1fa53d`
- Task bucket: `reviews/task-30/993-spire-phase13e-aws-correctness-after-pooling`
- Lane: Phase 13e synthetic correctness / AWS harness fault lifecycle
- Fixture: synthetic 10k corpus local PG18 harness; no AWS invoked after the tunnel restart fix
- Storage format: `rabitq`
- Rerank mode: default
- Surface: isolated coordinator and remote local PG instances, one index per table in the harness fixture

## Artifacts

### `preflight-after-tunnel-restart-fix.log`

- Command:
  - `script -q -e -c "make -C infra/spire-aws preflight" reviews/task-30/993-spire-phase13e-aws-correctness-after-pooling/artifacts/preflight-after-tunnel-restart-fix.log`
- Timestamp: 2026-05-27
- Key result lines:
  - Terraform configuration valid.
  - `bash -n scripts/spire-aws/*.sh` passed.
  - `shellcheck not found; skipping shellcheck`
  - `jq empty scripts/spire-aws/suite-correctness.json scripts/spire-aws/suite-representative.json scripts/spire-aws/suite-stress.json` passed.

### `aws-harness-local-after-tunnel-restart-fix/phase13e-aws-harness-local.log`

- Command:
  - `bash scripts/run_spire_phase13e_aws_harness_local_pg18.sh --skip-install --artifact-dir reviews/task-30/993-spire-phase13e-aws-correctness-after-pooling/artifacts/aws-harness-local-after-tunnel-restart-fix`
- Timestamp: 2026-05-27
- Key result lines:
  - coordinator corpus: 10,000 rows
  - remote shard rows: node 2 = 3295, node 3 = 3317, node 4 = 3388
  - CustomScan: `remote_fanout: 3`
  - CustomScan: `tuple_transport_status: ready`
  - production read: `result_source remote_heap_candidates`
  - production read: `status ready`
  - fault drills: `aws_local_fault_drill=degraded`, `aws_local_fault_drill=strict`
  - final: `HARNESS PASSED`

### Selected supporting logs

- `aws-harness-local-after-tunnel-restart-fix/smoke-customscan-read.log`
  - Shows CustomScan remote fanout and ready tuple transport.
- `aws-harness-local-after-tunnel-restart-fix/production-read-profile-smoke.log`
  - Shows remote heap candidates and ready status.
- `aws-harness-local-after-tunnel-restart-fix/aws-local-fault-degraded-summary.log`
  - Shows local degraded drill profile.
- `aws-harness-local-after-tunnel-restart-fix/aws-local-fault-strict-knn.stderr.log`
  - Captures expected strict-fault closed behavior.
