# Review Request: Phase 13e AWS Correctness Profile Cleanup Checkpoint

## Summary

This packet records the AWS setup reset after the local Phase 13e production read gate passed. No AWS correctness test has run in this packet yet.

Code checkpoint: `65e2f7528`

## What Changed

- Extended `scripts/spire-aws/cleanup-residue.sh` so the established cleanup entry point now handles stale Phase 13 VPC resources and the static node IAM role/profile, not only S3 buckets and Secrets Manager entries.
- Added the same documented pre-existing bucket residue override used by `preflight-permissions`, so missing `s3:ListBucketVersions` on old buckets does not block cleanup of EC2/IAM residue.
- Cleaned the stale Phase 13 AWS residue that blocked provisioning:
  - no running/stopped Phase 13 instances were present,
  - old active SPIRE secrets were force-deleted,
  - stale Phase 13 VPC, endpoints, endpoint security group, subnet, and route table were deleted,
  - static `ecaz-spire-aws-node` IAM role and instance profile were deleted.

## Evidence

Packet-local manifest: `reviews/task-30/988-spire-phase13e-aws-correctness-profile/artifacts/manifest.md`

Key result lines:

- `cleanup-residue-execute-2.log`: `Deleted VPC vpc-08e477285812abc44`
- `cleanup-residue-execute-2.log`: `Deleted IAM role ecaz-spire-aws-node`
- `cleanup-residue-execute-2.log`: `Deleted IAM instance profile ecaz-spire-aws-node`
- `preflight-after-residue-cleanup.log`: `SPIRE AWS operator preflight passed: region=us-west-2 ... coordinator=m7g.large remote=m7g.large remote_count=3`
- `preflight-after-residue-cleanup.log`: `SPIRE AWS state preflight passed: local Terraform state has no managed resources`
- `preflight-after-residue-cleanup.log`: `SPIRE AWS permission preflight passed`
- `preflight-static-after-cleanup-script.log`: `terraform validate` succeeded; shell `bash -n` succeeded; suite JSON parsed with `jq`.

## Notes

The remaining old S3 buckets are documented pre-existing residue. The current operator identity still lacks `s3:ListBucketVersions`, so they remain under the runbook exception path and are not part of the new AWS run.

The next step is the established Graviton correctness path after this clean preflight: provision, install, load, register, smoke, `ecaz bench suite`, and fault checks.
