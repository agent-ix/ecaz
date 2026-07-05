# Review Request: SPIRE Current AWS Preflight Status

Task: `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`

Code commit: `cc4f2c2d0`

## Summary

This doc/evidence checkpoint records the current AWS preflight state for the
remaining Phase 13e representative performance run. No AWS provisioning was
started.

The task note now records that:

- the active `infra/spire-aws/terraform.tfvars` is still on the established
  Graviton lane: `us-west-2`, `us-west-2a`, `m7g.large`, three remotes, and an
  `arm64` AMI;
- local Terraform state has no managed resources;
- default permissions preflight still fails on old documented
  `ecaz-spire-aws-*` buckets because the operator identity lacks
  `s3:ListBucketVersions`;
- the reviewed pre-existing residue override
  `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1` makes the combined preflight pass.

That means the next approved AWS representative run should either use the
reviewed residue override with packet-local evidence or wait until the S3
permission/residue issue is resolved.

## Validation

- `bash scripts/spire-aws/preflight-operator.sh infra/spire-aws/terraform.tfvars`
- `bash scripts/spire-aws/preflight-state.sh infra/spire-aws/terraform.tfstate`
- `bash scripts/spire-aws/preflight-permissions.sh`
  - expected current-state failure without the residue override
- `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 bash scripts/spire-aws/preflight-permissions.sh`
- `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 make -C infra/spire-aws preflight-operator preflight-state preflight-permissions preflight-representative-performance`
- `git diff --check cc4f2c2d0^ cc4f2c2d0 -- plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/preflight-operator-current.log`
- `artifacts/preflight-state-current.log`
- `artifacts/preflight-permissions-current.log`
- `artifacts/preflight-permissions-current-override.log`
- `artifacts/make-current-aws-preflight.log`
- `artifacts/git-show-stat.log`
- `artifacts/git-diff-check.log`

The existing untracked SPIRE artifact directories were left untouched and were
not staged.
