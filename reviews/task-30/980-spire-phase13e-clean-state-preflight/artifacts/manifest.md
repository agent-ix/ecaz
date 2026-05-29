# Artifact Manifest: SPIRE Phase 13e Clean-State Preflight

Head SHA: `c24f817f39a5590099d988d4c1a973ce05dfc067`
Task bucket: `reviews/task-30/980-spire-phase13e-clean-state-preflight`
Timestamp: `2026-05-25T21:47:46Z`
Lane: local operator-surface validation plus read-only AWS residue inventory
AWS resources provisioned: no
Storage format / rerank mode: not applicable
Surface isolation: not applicable; no benchmark tables were created

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `bash-syntax.log` | `bash -n scripts/spire-aws/preflight-state.sh scripts/spire-aws/preflight-operator.sh` | exit 0 |
| `preflight-current-stale-state.log` | `make -C infra/spire-aws preflight-state` against current checkout state | expected exit 2; stale `aws_s3_bucket.artifacts` found |
| `preflight-empty-state-pass.log` | `scripts/spire-aws/preflight-state.sh` against synthetic empty state | exit 0 |
| `aws-spire-buckets.log` | `aws s3api list-buckets --query 'Buckets[?starts_with(Name, \`ecaz-spire-aws-\`)].Name' --output json` | exit 0; two buckets listed |
| `aws-list-object-versions-denied.log` | `aws s3api list-object-versions --bucket ecaz-spire-aws-20260525201045387900000003 ...` | expected AWS access denied; missing `s3:ListBucketVersions` |
| `git-diff-check.log` | `git diff --check` | exit 0 |

## Key Result Lines

- Current stale state: `aws_s3_bucket.artifacts`
- Empty state pass: `SPIRE AWS state preflight passed: local Terraform state has no managed resources`
- Bucket inventory: `ecaz-spire-aws-20260523165108075000000003`, `ecaz-spire-aws-20260525201045387900000003`
- Version cleanup permission gap: `not authorized to perform: s3:ListBucketVersions`
