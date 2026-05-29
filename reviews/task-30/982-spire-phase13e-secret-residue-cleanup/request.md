# Review Request: SPIRE Phase 13e Secret Residue Cleanup

Requester: coder1
Date: 2026-05-25
Head SHA: `465f4795bf530caae1e459578815fc8205e67b32`
Review focus: record the destructive cleanup of old SPIRE Secrets Manager residue and the remaining S3 permission blocker before the next AWS correctness run.

## Summary

I ran `scripts/spire-aws/cleanup-residue.sh --execute` after approval. The command could not clean S3 buckets because the operator still lacks `s3:ListBucketVersions`, but it force-deleted the four old SPIRE Secrets Manager entries that were blocking name reuse in packet 975.

Post-cleanup inventory now shows:

- no Secrets Manager entries matching `ecaz-spire-aws`;
- two old `ecaz-spire-aws-*` S3 buckets still present;
- `s3:ListBucketVersions` is still denied for the 975 bucket.

No AWS resources were provisioned. Only old matching Secrets Manager residue was deleted.

## Validation

- `aws secretsmanager list-secrets --include-planned-deletion ...` returns `[]`.
- `aws s3api list-buckets ...` still lists the two old SPIRE buckets.
- `aws s3api list-object-versions ...` is still denied for the 975 bucket.
- `git diff --check` passed.

See `artifacts/manifest.md` for packet-local logs.

## Remaining Blocker

The actual AWS correctness run is still gated by the two old versioned buckets and stale local Terraform state. The current credential cannot clean the buckets because it lacks `s3:ListBucketVersions`. The next step requires either that permission, bucket cleanup by another credential, or an explicit reviewed state/residue handling decision before provisioning.
