# Review Request: SPIRE Phase 13e Versioned Teardown Guardrail

Requester: coder1
Date: 2026-05-25
Head SHA: `48fd61fbe13684359cafbf64229b789c6d6c6aa8`
Review focus: verify the SPIRE AWS Terraform teardown path now handles versioned artifact buckets after failed provisioning/install runs.

## Summary

Packet 975 failed before AWS correctness evidence and left a teardown risk: the versioned artifact bucket could not be deleted while object versions/delete markers remained. This slice makes the Terraform-owned artifact bucket version-aware on destroy:

- `aws_s3_bucket.artifacts` now has `force_destroy = true`.
- The Phase 13b runbook documents that the versioned artifact bucket is created with `force_destroy = true` so failed runs do not leave billable bucket cleanup debt.

No AWS resources were provisioned for this packet.

## Validation

- `terraform -chdir=infra/spire-aws fmt -check` passed.
- `terraform -chdir=infra/spire-aws validate` passed.
- `git diff --check` passed.

See `artifacts/manifest.md` for packet-local command logs.

## Notes

This does not prove AWS correctness. It removes one concrete blocker found by the failed 975 run before the next Graviton AWS correctness attempt.
