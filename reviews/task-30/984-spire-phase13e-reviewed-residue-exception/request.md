# Review Request: SPIRE Phase 13e Reviewed Residue Exception

Requester: coder1
Date: 2026-05-25
Head SHA: `5e361d33a7d3e0ab88d470ccf80c008000ff6c66`
Review focus: verify the explicit exception path for documented pre-existing bucket residue and stale local Terraform state.

## Summary

The current AWS identity still cannot list object versions for two old `ecaz-spire-aws-*` buckets, so it cannot clean them. This slice keeps strict cleanup as the default, but adds an explicit exception path for documented pre-existing residue:

- `preflight-permissions.sh --allow-preexisting-residue` / `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1` keeps checking STS, bucket inventory, and Secrets Manager, but downgrades missing `s3:ListBucketVersions` on already-documented old buckets to warnings.
- `archive-local-state.sh` archives stale local Terraform state before a new run starts.
- The runbook now requires packet-local evidence for both the old bucket exception and local state archive before provisioning.

I also archived the stale local `infra/spire-aws/terraform.tfstate` that contained only `aws_s3_bucket.artifacts`. The full archived tfstate copies remain ignored and are not committed; the packet includes the pre-archive resource list, SHA-256, archive log, and post-archive `preflight-state` proof.

No AWS resources were provisioned, modified, or deleted for this packet.

## Validation

- `bash -n scripts/spire-aws/preflight-permissions.sh scripts/spire-aws/archive-local-state.sh scripts/spire-aws/preflight-state.sh` passed.
- `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 make -C infra/spire-aws preflight-permissions` passed while logging warnings for the two old buckets.
- `archive-local-state.sh` passed against synthetic state.
- Actual stale local Terraform state was archived/moved aside; pre-archive resource list was `aws_s3_bucket.artifacts`.
- `make -C infra/spire-aws preflight-state` now passes because there is no active local SPIRE AWS state file.
- `git diff --check` passed.

See `artifacts/manifest.md` for packet-local logs.

## Remaining Work

The next correctness attempt still needs a real local `infra/spire-aws/terraform.tfvars` with an arm64 AL2023 AMI and the run-specific artifact directory. The old buckets remain as documented pre-existing residue until a credential with S3 version cleanup permission deletes them.
