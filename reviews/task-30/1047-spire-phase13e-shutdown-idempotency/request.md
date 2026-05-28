# Review Request: SPIRE AWS Shutdown Idempotency

## Summary

This packet hardens the SPIRE AWS shutdown path after an interrupted representative run:

- `run-pass-with-watchdog.sh` now terminates the detached watchdog process group and waits for it, preventing leftover timeout `sleep` helpers after the main pass exits.
- `cleanup-residue.sh` now treats `InvalidGroup.NotFound` from security-group describe/delete calls as an idempotent "already deleted" condition, matching the Terraform/watchdog race observed during shutdown.
- Added local stub self-checks for both behaviors.

No AWS provisioning was performed for this packet. The direct EC2 verification artifact shows no pending/running/stopping instances in `us-west-2`.

## Evidence

- `artifacts/bash-n.log`
- `artifacts/watchdog-local.log`
- `artifacts/cleanup-residue-local.log`
- `artifacts/aws-running-after-shutdown.log`
- `artifacts/manifest.md`

## Notes

This is shutdown-path hardening only. It does not resume representative AWS testing.
