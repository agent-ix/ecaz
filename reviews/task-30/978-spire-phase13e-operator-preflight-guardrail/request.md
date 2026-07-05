# Review Request: SPIRE Phase 13e Operator Preflight Guardrail

Requester: coder1
Date: 2026-05-25
Head SHA: `eb87ae23ed1693d1816acb2ba892d1c609592e2b`
Review focus: verify SPIRE AWS provisioning now fails before Terraform apply when the operator inputs do not match the established Graviton/aarch64 runbook.

## Summary

This slice adds a pre-provision operator gate to stop the setup drift that caused the failed 975 attempt:

- New `scripts/spire-aws/preflight-operator.sh` checks the real `infra/spire-aws/terraform.tfvars` before provisioning.
- `make -C infra/spire-aws provision` now depends on `preflight-operator`.
- The gate requires `region`, `availability_zone`, `ami_id`, `owner`, and `auto_stop_at`.
- The gate uses the module defaults for omitted instance types, but rejects any coordinator/remote family outside `m7g`, `m8g`, `r7g`, `c7g`, and `c8g`.
- The gate resolves the selected AMI through `aws ec2 describe-images` and requires `Architecture == arm64`.
- Phase 13b now lists `make -C infra/spire-aws preflight-operator` as a required pre-flight box.

No AWS resources were provisioned for this packet.

## Validation

- `bash -n scripts/spire-aws/preflight-operator.sh` passed.
- `make -C infra/spire-aws preflight-operator` fails early when `terraform.tfvars` is absent.
- A mocked Graviton/arm64 `terraform.tfvars` passes.
- A mocked `r6i.4xlarge` coordinator is rejected.
- `git diff --check` passed.

See `artifacts/manifest.md` for packet-local logs.

## Notes

This still does not prove AWS correctness. It makes the next AWS attempt follow the established Graviton/aarch64 setup and prevents `make provision` from becoming the first place a bad setup is discovered.
