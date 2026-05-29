# Review Request: SPIRE Phase 13e Clean-State Preflight

Requester: coder1
Date: 2026-05-25
Head SHA: `c24f817f39a5590099d988d4c1a973ce05dfc067`
Review focus: verify SPIRE AWS provisioning now refuses to run over stale local Terraform state from the failed 975 attempt.

## Summary

Read-only AWS checks show the failed 975 residue is still real:

- S3 still lists two `ecaz-spire-aws-*` buckets.
- The operator credential cannot list object versions on the 975 bucket, so it cannot prove or perform version-aware cleanup with the current permissions.
- Local `infra/spire-aws/terraform.tfstate` still contains `aws_s3_bucket.artifacts`.

This slice adds `scripts/spire-aws/preflight-state.sh` and wires `make -C infra/spire-aws provision` to require `preflight-state` before Terraform init/apply. It prevents a new correctness run from silently reusing stale bucket-only state.

No AWS resources were provisioned for this packet.

## Validation

- `bash -n scripts/spire-aws/preflight-state.sh scripts/spire-aws/preflight-operator.sh` passed.
- `make -C infra/spire-aws preflight-state` fails on the current stale state and names `aws_s3_bucket.artifacts`.
- The state preflight passes against an empty synthetic Terraform state.
- `aws s3api list-buckets` confirms two `ecaz-spire-aws-*` buckets still exist.
- `aws s3api list-object-versions` is denied for the 975 bucket, confirming the current credential cannot complete the reviewer-requested version cleanup.
- `git diff --check` passed.

See `artifacts/manifest.md` for packet-local logs.

## Operational Note

The next AWS correctness attempt should not run until the stale local state is cleared or moved with packet-local evidence and the remaining S3 buckets are either deleted by a credential with `s3:ListBucketVersions`/version-delete permissions or explicitly accepted as pre-existing residue outside the new run.
