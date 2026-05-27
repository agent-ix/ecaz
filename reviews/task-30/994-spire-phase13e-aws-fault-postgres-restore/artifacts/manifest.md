# Manifest

Head SHA: `256f1e99bc37a6b1a3fea33e1a162ae8ff0d0881`

Task bucket: `reviews/task-30/994-spire-phase13e-aws-fault-postgres-restore`

Timestamp: `2026-05-26T20:16:14-07:00`

## Artifacts

### `aws-fault-restore-failure-summary.md`

- Lane: AWS Graviton synthetic correctness, prior run evidence.
- Fixture: `ec_spire_aws_synth_10k`.
- Storage format / rerank mode: default Phase 13e correctness suite settings.
- Command cited from prior packet: `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 make -C infra/spire-aws pass-correctness ARTIFACT_DIR=reviews/task-30/993-spire-phase13e-aws-correctness-after-pooling/artifacts/aws-correctness-after-tunnel-restart-fix`.
- Key result: degraded mode passed, restore then failed because remote node 2 SQL did not become ready within 300s after EC2 start and tunnel restart.

### `preflight.log`

- Lane: local infra/harness validation.
- Command: `script -q -e -c "make -C infra/spire-aws preflight" reviews/task-30/994-spire-phase13e-aws-fault-postgres-restore/artifacts/preflight.log`
- Key result: Terraform fmt/init/validate passed; `bash -n scripts/spire-aws/*.sh` passed; suite JSON validation passed; shellcheck unavailable and skipped by preflight.
