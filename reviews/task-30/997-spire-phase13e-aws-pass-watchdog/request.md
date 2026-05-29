# Review Request: SPIRE AWS Pass Watchdog

## Summary

This checkpoint makes the standard AWS pass targets safer before any further
AWS rerun. `pass-correctness` and `pass-representative` now route through
`scripts/spire-aws/run-pass-with-watchdog.sh`, which owns teardown on normal
exit, signal exit, pass failure, or timeout.

The wrapper starts a detached watchdog so an interrupted operator session has a
bounded spend window. Defaults are 2 hours for correctness and 4 hours for the
representative tier, overridable with `SPIRE_AWS_PASS_TIMEOUT_SECONDS`.

## Code

- Commit: `4e675c592a05254284bfe21476d372908c0f3711`
- `infra/spire-aws/Makefile`
  - Public `pass-correctness` / `pass-representative` targets call the
    watchdog wrapper.
  - The original provision/install/verify sequences moved into body targets.
- `scripts/spire-aws/run-pass-with-watchdog.sh`
  - Creates packet-local `aws-pass-watchdog.log`.
  - Runs teardown once via a packet-local lock.
  - Verifies `preflight-state` after teardown.
- `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`
  - Updates the current AWS evidence note and records that AWS reruns are
    paused behind this safety guard.

## Validation

- `artifacts/preflight.log`: `make -C infra/spire-aws preflight` passed.
- `artifacts/preflight-state.log`: local Terraform state has no managed resources.
- `artifacts/aws-phase13-instances.log`: Phase 13 EC2 query returned `[]`.
- `artifacts/pass-correctness-dry-run.log`: public correctness target resolves
  through the watchdog wrapper.
- `artifacts/pass-representative-dry-run.log`: public representative target
  resolves through the watchdog wrapper.

No AWS provisioning was run for this packet.
