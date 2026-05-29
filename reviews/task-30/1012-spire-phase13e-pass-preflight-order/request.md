# SPIRE Phase 13e Representative Pass Preflight Order

This slice closes a local harness gap in the AWS representative performance pass. The representative preflight existed and was part of `make preflight`, but `pass-representative-performance-body` could still proceed to `provision` if the operator skipped the standalone preflight target.

## Change

- `pass-representative-performance-body` now runs `preflight-representative-performance` before `provision`.
- `scripts/spire-aws/preflight-representative-performance.sh` now statically verifies that ordering, so future edits cannot silently move provisioning ahead of the representative evidence preflight.

This does not start AWS and does not change the representative suite contents. It makes the expensive AWS path fail before provisioning if the local suite/wiring evidence gates are broken.

## Validation

All validation is local. No AWS resources were started.

- `bash -n scripts/spire-aws/preflight-representative-performance.sh`
  - artifact: `artifacts/bash-n-preflight-order.log`
- `scripts/spire-aws/preflight-representative-performance.sh`
  - artifact: `artifacts/preflight-order.log`
- `make -C infra/spire-aws preflight-representative-performance`
  - artifact: `artifacts/make-preflight-order.log`
- `SPIRE_AWS_REPRESENTATIVE_POOLING_SUITE=artifacts/bad-suite/suite-representative-pooling.json scripts/spire-aws/preflight-representative-performance.sh`
  - artifact: `artifacts/preflight-bad-pooling-suite.log`
  - result: expected exit code `2`
- `make -C infra/spire-aws preflight`
  - artifact: `artifacts/make-preflight-with-order.log`

## Next

The remaining Phase 13e blocker is still the explicit Graviton representative performance run. With this change, that run now gates representative suite/readiness evidence before any Terraform provisioning step.
