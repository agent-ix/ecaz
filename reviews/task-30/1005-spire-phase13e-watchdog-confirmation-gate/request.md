# Review Request: SPIRE AWS Watchdog Confirmation Gate

## Scope

This packet covers commit `8c962173c2588b4a84d7b31b8c962060a65ab687`, which
moves the explicit AWS provisioning confirmation check ahead of the pass
watchdog setup.

This keeps unconfirmed `pass-*` invocations local-only: they fail before the
watchdog, teardown trap, artifact directory creation for the denied pass, or
Terraform/AWS cleanup paths are entered.

## Change Summary

- `scripts/spire-aws/run-pass-with-watchdog.sh` now calls
  `scripts/spire-aws/confirm-provision.sh` before watchdog setup for:
  - `pass-correctness-body`
  - `pass-representative-body`
  - `pass-representative-performance-body`
- Confirmed target remains unchanged for approved runs:

  ```sh
  SPIRE_AWS_CONFIRM_PROVISION=yes
  ```

## Validation

No AWS provisioning, Terraform apply, Terraform destroy, EC2 start, SSM tunnel,
or PostgreSQL cluster was used for this packet.

- `bash -n scripts/spire-aws/run-pass-with-watchdog.sh`
  - artifact: `artifacts/bash-n-watchdog.log`
  - result: exit 0
- `scripts/spire-aws/run-pass-with-watchdog.sh pass-representative-performance-body artifacts/denied-pass-artifacts`
  - artifact: `artifacts/watchdog-deny-direct.log`
  - result: exit 2 at confirmation gate
- `make -C infra/spire-aws ARTIFACT_DIR=artifacts/denied-make-artifacts pass-representative-performance`
  - artifact: `artifacts/make-pass-deny.log`
  - result: exit 2 at confirmation gate
- `test ! -e artifacts/denied-pass-artifacts`
  - artifact: `artifacts/denied-direct-artifacts-absent.log`
  - result: exit 0
- `test ! -e artifacts/denied-make-artifacts`
  - artifact: `artifacts/denied-make-artifacts-absent.log`
  - result: exit 0
- `make -C infra/spire-aws preflight`
  - artifact: `artifacts/preflight.log`
  - result: exit 0

## Remaining Phase 13e Work

The next required proof is still the Graviton representative performance pass:
latency p50/p95/p99, recall, production read profile, and pooled-vs-unpooled
profile deltas. Fault/rerun resilience remains deferred until that packet is
captured.
