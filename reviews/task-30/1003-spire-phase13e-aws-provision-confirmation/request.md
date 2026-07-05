# Review Request: SPIRE AWS Provisioning Confirmation Gate

## Scope

This packet covers commit `bce40804bb0dece862f690e72559b25a48b8891c`, which
adds a fail-closed confirmation gate before SPIRE AWS provisioning.

The current Phase 13e priority remains representative latency, recall, and
pooling A/B evidence first. Fault/resilience reruns stay deferred until that
evidence is captured.

## Change Summary

- Added `scripts/spire-aws/confirm-provision.sh`.
- Added `make -C infra/spire-aws confirm-provision`.
- Made `provision` depend on `confirm-provision`, so pass targets that reach
  Terraform provisioning require:

  ```sh
  SPIRE_AWS_CONFIRM_PROVISION=yes
  ```

Preflight, teardown, cleanup, and read-only checks remain available without this
confirmation.

## Validation

No AWS provisioning, Terraform apply, or EC2 start was run for this packet.

- `bash -n scripts/spire-aws/confirm-provision.sh`
  - artifact: `artifacts/bash-n-confirm-provision.log`
  - result: exit 0
- `scripts/spire-aws/confirm-provision.sh`
  - artifact: `artifacts/confirm-provision-deny.log`
  - result: exit 2, expected denial
- `SPIRE_AWS_CONFIRM_PROVISION=yes scripts/spire-aws/confirm-provision.sh`
  - artifact: `artifacts/confirm-provision-allow.log`
  - result: exit 0
- `make -C infra/spire-aws confirm-provision`
  - artifact: `artifacts/make-confirm-provision-deny.log`
  - result: exit 2, expected denial
- `SPIRE_AWS_CONFIRM_PROVISION=yes make -C infra/spire-aws confirm-provision`
  - artifact: `artifacts/make-confirm-provision-allow.log`
  - result: exit 0

## Next AWS Command When Explicitly Approved

The next production evidence pass should be the representative performance lane,
not the fault lane:

```sh
SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 \
SPIRE_AWS_CONFIRM_PROVISION=yes \
make -C infra/spire-aws \
  ARTIFACT_DIR=reviews/task-30/<next-packet>/artifacts \
  pass-representative-performance
```
