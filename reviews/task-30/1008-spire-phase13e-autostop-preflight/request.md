# Review Request: SPIRE AWS Auto-Stop Preflight

## Scope

This packet covers commit `e5c26d1b07f4017f4458fed9084b560402095d7a`, which
rejects expired AWS operator run windows before provisioning.

The immediate local operator config had an old `auto_stop_at`; it has been
updated in the ignored local `infra/spire-aws/terraform.tfvars` to
`2026-05-28T06:00:00Z` for the next explicitly approved run. That local file is
not committed.

## Change Summary

- `scripts/spire-aws/preflight-operator.sh` now parses `auto_stop_at` and
  requires it to be in the future.
- Tests can pin current time through `SPIRE_AWS_PREFLIGHT_NOW_EPOCH`.
- `infra/spire-aws/terraform.tfvars.example` no longer ships with an already
  expired `auto_stop_at` value.

## Validation

No AWS provisioning, Terraform apply, Terraform destroy, EC2 start, real SSM
session, or real AWS API call was used for the operator-preflight tests. The
AMI architecture lookup was served by a packet-local fake `aws` command.

- `bash -n scripts/spire-aws/preflight-operator.sh`
  - artifact: `artifacts/bash-n-preflight-operator.log`
  - result: exit 0
- future auto-stop fixture at fixed now `2026-05-27T12:00:00Z`
  - artifact: `artifacts/preflight-future.log`
  - result: exit 0
- expired auto-stop fixture at fixed now `2026-05-27T12:00:00Z`
  - artifact: `artifacts/preflight-expired.log`
  - result: exit 2
- current ignored local `infra/spire-aws/terraform.tfvars`
  - artifact: `artifacts/preflight-local-tfvars.log`
  - result: exit 0 with fake AMI architecture lookup
- `make -C infra/spire-aws preflight`
  - artifact: `artifacts/preflight.log`
  - result: exit 0

## Remaining Phase 13e Work

The remaining proof is still the explicitly approved Graviton representative
performance run: p50/p95/p99 latency, recall, production read profile, and
pooling A/B deltas.
