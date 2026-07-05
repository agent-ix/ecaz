# Artifact Manifest: SPIRE Phase 13e Permission Preflight

Head SHA: `b677cdc3fb24839259d8c82649a0bbbaccdf5352`
Task bucket: `reviews/task-30/983-spire-phase13e-permission-preflight`
Timestamp: `2026-05-25T21:57:04Z`
Lane: read-only AWS permission preflight
AWS resources provisioned: no
AWS resources modified/deleted: no
Storage format / rerank mode: not applicable
Surface isolation: not applicable; no benchmark tables were created

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `bash-syntax.log` | `bash -n scripts/spire-aws/preflight-permissions.sh scripts/spire-aws/cleanup-residue.sh scripts/spire-aws/preflight-state.sh scripts/spire-aws/preflight-operator.sh` | exit 0 |
| `preflight-permissions-current.log` | `make -C infra/spire-aws preflight-permissions` | expected exit 2; missing `s3:ListBucketVersions` |
| `git-diff-check.log` | `git diff --check` | exit 0 |

## Key Result Lines

- Identity: `arn:aws:iam::932658697181:user/ecaz-operator`
- Buckets: `ecaz-spire-aws-20260523165108075000000003`, `ecaz-spire-aws-20260525201045387900000003`
- Permission blocker: `ERROR: missing s3:ListBucketVersions`
- Secrets permission: `Secrets Manager list permission ok for prefix ecaz-spire-aws`
