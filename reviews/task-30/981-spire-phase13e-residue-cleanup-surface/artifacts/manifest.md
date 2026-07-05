# Artifact Manifest: SPIRE Phase 13e Residue Cleanup Surface

Head SHA: `b158d045f960dcbefc05491e233c6460e26e8a39`
Task bucket: `reviews/task-30/981-spire-phase13e-residue-cleanup-surface`
Timestamp: `2026-05-25T21:51:04Z`
Lane: local operator-surface validation plus dry-run AWS residue inventory
AWS resources provisioned: no
AWS resources deleted: no
Storage format / rerank mode: not applicable
Surface isolation: not applicable; no benchmark tables were created

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `bash-syntax.log` | `bash -n scripts/spire-aws/cleanup-residue.sh scripts/spire-aws/preflight-state.sh scripts/spire-aws/preflight-operator.sh` | exit 0 |
| `cleanup-residue-dry-run.log` | `make -C infra/spire-aws cleanup-residue` | expected exit 2; dry-run reported residue and missing S3 version-list permission |
| `git-diff-check.log` | `git diff --check` | exit 0 |

## Key Result Lines

- Mode: `SPIRE AWS residue cleanup mode: dry-run`
- Buckets: `ecaz-spire-aws-20260523165108075000000003`, `ecaz-spire-aws-20260525201045387900000003`
- Permission gap: `not authorized to perform: s3:ListBucketVersions`
- Secrets: `ecaz-spire-aws-remote-1`, `ecaz-spire-aws-remote-2`, `ecaz-spire-aws-remote-3`, and `ecaz-spire-aws-bc5c9431-remote-1-20260525201045383600000001`
