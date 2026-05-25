# Review Request: SPIRE Phase 13e Graviton Procedure Alignment

Requester: coder1
Date: 2026-05-25
Head SHA: `d397b93a38a5073ad71c1e3ad118725b367c84d9`
Review focus: confirm the SPIRE AWS verification path now follows the repo's established Graviton/aarch64 convention and blocks stale r6i/x86 defaults.

## Summary

This slice corrects the Phase 13e AWS procedure drift found during the failed 975 AWS attempt. No AWS resources were provisioned for this packet.

Changes:

- `infra/spire-aws` defaults now use the Phase 13a Graviton topology: coordinator `r7g.4xlarge`, three remotes defaulting to `r7g.2xlarge`, and AL2023 arm64 AMI wording.
- Terraform variable validation rejects non-Graviton coordinator/remote instance families outside `m7g`, `m8g`, `r7g`, `c7g`, and `c8g`.
- Phase 13a, Phase 13b, Phase 13e, and the parent AWS verification packet now state that x86/r6i hardware is not valid SPIRE AWS evidence.
- Phase 13e now requires task/runbook amendment before changing AWS hardware shapes, regions, or setup procedure.
- `.gitignore` now excludes private TLS keys and compiled extension payloads under review-packet artifact directories.

## Validation

- `terraform -chdir=infra/spire-aws fmt -check` passed.
- `terraform -chdir=infra/spire-aws validate` passed.
- `bash -n scripts/spire-aws/*.sh` passed for the SPIRE AWS helper scripts.
- `git diff --check` passed.

See `artifacts/manifest.md` for packet-local command logs.

## Notes For Reviewer

This does not claim AWS correctness. Packet 975 remains a failed provisioning record, not product evidence. This packet only fixes the procedure guardrail so the next AWS attempt starts from the established Graviton lane instead of the stale r6i defaults.
