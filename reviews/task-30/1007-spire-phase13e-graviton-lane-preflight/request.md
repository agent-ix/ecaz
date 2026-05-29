# Review Request: SPIRE AWS Graviton Lane Preflight

## Scope

This packet covers commit `8497313f588bca7809d901d118e282e6a0df737e`, which
turns the established Phase 13e AWS lane into a pre-provisioning guard.

The next Phase 13e AWS proof must use `us-west-2`, `us-west-2a`, and
`m7g.large` coordinator/remote instances unless the task/runbook is amended
before provisioning.

## Change Summary

- `scripts/spire-aws/preflight-operator.sh` now rejects:
  - non-`us-west-2` regions
  - non-`us-west-2a` availability zones
  - non-`m7g.large` coordinator instance types
  - non-`m7g.large` remote instance types
- `infra/spire-aws/terraform.tfvars.example` now uses `us-west-2/us-west-2a`
  and describes the fixed established Graviton lane.

This keeps the next representative latency/recall/pooling run aligned with the
standard Graviton procedure instead of drifting into a new hardware setup.

## Validation

No AWS provisioning, Terraform apply, Terraform destroy, EC2 start, real SSM
session, or real AWS API call was used for the operator-preflight lane tests.
The AMI architecture lookup was served by a packet-local fake `aws` command.

- `bash -n scripts/spire-aws/preflight-operator.sh`
  - artifact: `artifacts/bash-n-preflight-operator.log`
  - result: exit 0
- good lane: `us-west-2/us-west-2a/m7g.large`
  - artifact: `artifacts/preflight-good.log`
  - result: exit 0
- bad region: `us-east-1/us-east-1a`
  - artifact: `artifacts/preflight-bad-region.log`
  - result: exit 2
- bad instance type: `r6i.large`
  - artifact: `artifacts/preflight-bad-instance.log`
  - result: exit 2
- `make -C infra/spire-aws preflight`
  - artifact: `artifacts/preflight.log`
  - result: exit 0

## Remaining Phase 13e Work

The remaining proof is still the explicitly approved Graviton representative
performance run: p50/p95/p99 latency, recall, production read profile, and
pooling A/B deltas.
