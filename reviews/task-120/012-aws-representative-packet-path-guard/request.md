# Task 120 AWS Representative Packet Path Guard

Please review the narrow SPIRE AWS harness change that allows the representative
performance pass to write packet-local artifacts outside Task 30.

## Scope

The code checkpoint updates:

- `scripts/spire-aws/run-representative-performance-pass.sh`
- `scripts/spire-aws/preflight-representative-performance.sh`

Both scripts now accept packet-local review artifact paths matching
`reviews/task-*/*/artifacts` instead of only `reviews/task-30/*/artifacts`.
The legacy Task 30 default packet remains explicitly rejected.

No benchmark behavior, suite content, Terraform resource definition, load logic,
or remote read path changed.

## Evidence

- Artifact manifest:
  `reviews/task-120/012-aws-representative-packet-path-guard/artifacts/manifest.md`
- Shell syntax validation:
  `reviews/task-120/012-aws-representative-packet-path-guard/artifacts/bash-n.log`
- Read-only AWS/preflight validation:
  `reviews/task-120/012-aws-representative-packet-path-guard/artifacts/preflight-task120-artifact-dir.log`

The preflight was run with
`SPIRE_AWS_ALLOW_NONDEFAULT_GRAVITON_LANE=1` because the local ignored
`infra/spire-aws/terraform.tfvars` is currently set to the existing
`r7g.4xlarge` coordinator / `r7g.2xlarge` remote lane. It did not provision EC2
resources.

## Result

Task 120 can now use the standard representative AWS pass with packet-local
artifacts under `reviews/task-120/.../artifacts`, which is required before the
Phase 5 distributed/AWS measurement packet.

This is not Task 120 closeout. It is only harness enablement for the upcoming
distributed near-data rerank and AWS evidence.
