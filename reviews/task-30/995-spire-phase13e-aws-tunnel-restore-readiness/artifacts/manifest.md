# Manifest

Head SHA: `de3d909dfccf423334444fed707ac49d4198e91d`

Task bucket: `reviews/task-30/995-spire-phase13e-aws-tunnel-restore-readiness`

Timestamp: `2026-05-26T21:25:00-07:00`

## Artifacts

### `aws-tunnel-restore-failure-summary.md`

- Lane: AWS Graviton synthetic correctness.
- Fixture: `ec_spire_aws_synth_10k`.
- Storage format / rerank mode: Phase 13e correctness suite defaults, `rabitq`, production read only for production profile.
- Command: `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 make -C infra/spire-aws pass-correctness ARTIFACT_DIR=reviews/task-30/994-spire-phase13e-aws-fault-postgres-restore/artifacts/aws-correctness-after-postgres-restore-fix`.
- Key result: SPIRE remote placement/read/profile/fault-degraded behavior passed; restore failed because local SSM port-forward readiness never produced a usable SQL connection after EC2 restart.

### `preflight.log`

- Lane: local infra/harness validation.
- Command: `script -q -e -c "make -C infra/spire-aws preflight" reviews/task-30/995-spire-phase13e-aws-tunnel-restore-readiness/artifacts/preflight.log`
- Key result: Terraform fmt/init/validate passed; `bash -n scripts/spire-aws/*.sh` passed; suite JSON validation passed; shellcheck unavailable and skipped by preflight.
