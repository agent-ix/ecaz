# Review Request: SPIRE Phase 13e Residue Cleanup Surface

Requester: coder1
Date: 2026-05-25
Head SHA: `b158d045f960dcbefc05491e233c6460e26e8a39`
Review focus: verify failed-run residue cleanup is now a standard SPIRE AWS operator command rather than ad hoc manual setup.

## Summary

This slice adds `scripts/spire-aws/cleanup-residue.sh` and exposes it as
`make -C infra/spire-aws cleanup-residue`.

The command is dry-run by default. It:

- lists S3 buckets matching the SPIRE AWS artifact-bucket prefix;
- lists object versions/delete markers before deleting any bucket;
- lists Secrets Manager secrets matching the SPIRE prefix, including planned-deletion secrets;
- only deletes bucket versions, buckets, and secrets when `--execute` is explicitly supplied.

The runbook now requires residue cleanup output in the owning packet before a new provision if state or AWS inventory shows residue from a prior failed run. `infra/spire-aws/terraform.tfvars` is also ignored so local operator inputs do not get committed.

No AWS resources were provisioned or deleted for this packet.

## Validation

- `bash -n scripts/spire-aws/cleanup-residue.sh scripts/spire-aws/preflight-state.sh scripts/spire-aws/preflight-operator.sh` passed.
- `git diff --check` passed.
- `make -C infra/spire-aws cleanup-residue` ran in dry-run mode and reported:
  - two `ecaz-spire-aws-*` buckets;
  - missing `s3:ListBucketVersions` permission for both buckets;
  - four matching Secrets Manager secrets.

See `artifacts/manifest.md` for packet-local logs.

## Notes

This does not unblock the AWS correctness run by itself. It standardizes the cleanup procedure and confirms the current credential still lacks the S3 version-list permission required to clean the leaked buckets.
