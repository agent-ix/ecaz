---
date: 2026-05-26
head_sha: d5f0eebd6
task_bucket: reviews/task-30/993-spire-phase13e-aws-correctness-after-pooling
artifact_path: reviews/task-30/993-spire-phase13e-aws-correctness-after-pooling/artifacts/aws-correctness-after-degraded-mode-fix/preexisting-residue-exception.md
---

# Preexisting Residue Exception

The Phase 13b runbook allows provisioning with
`SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1` when old versioned S3 buckets predate
the current run and the operator lacks `s3:ListBucketVersions` to inspect or
empty them.

Read-only preflight on 2026-05-26 reported the following preexisting buckets:

- `ecaz-spire-aws-20260523165108075000000003`
- `ecaz-spire-aws-20260525201045387900000003`
- `ecaz-spire-aws-20260525221947629900000007`
- `ecaz-spire-aws-20260526155520698800000007`
- `ecaz-spire-aws-20260526171747646900000007`
- `ecaz-spire-aws-20260526184930637300000007`

`make -C infra/spire-aws preflight-state` passed before this exception:
local Terraform state has no managed resources.

This exception does not permit reusing local Terraform state or provisioning
over active Phase 13 instances.
