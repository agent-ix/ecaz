# Review Request: Representative Shutdown Preflight Gate

## Summary

This packet covers commit `3acd2c2fb02e539355d2cff7393907b4add810fd`
(`Gate representative AWS pass setup locally`).

The representative AWS pass now catches the two setup problems that caused the
latest churn before any provision step:

- `preflight-representative-performance.sh` requires and runs the local
  watchdog teardown and cleanup-residue race self-checks.
- `run-representative-performance-pass.sh` refreshes the ignored local
  `infra/spire-aws/terraform.tfvars` `auto_stop_at` deadline before preflight,
  through the new `scripts/spire-aws/refresh-auto-stop-at.sh` helper. This
  keeps the established Graviton lane while avoiding repeated manual timestamp
  edits.

No AWS infrastructure was started. The packet includes empty before/after EC2
running-state checks for `us-west-2`.

## Evidence

Artifact manifest: `artifacts/manifest.md`

Key post-commit validation:

- `artifacts/bash-n-post-commit.log`: shell syntax passed for the touched
  scripts and shutdown self-check scripts.
- `artifacts/preflight-post-commit.log`: representative local preflight passed,
  including the newly required shutdown/cleanup self-checks.
- `artifacts/refresh-auto-stop-at.log`: packet-local fixture proves the new
  helper rewrites `auto_stop_at`.
- `artifacts/representative-pass-dry-run-post-commit.log`: the standard
  representative pass dry-run refreshed `auto_stop_at`, passed operator/state/
  permissions/representative preflights, printed the execute command, and
  stopped with `Dry run only`.
- `artifacts/aws-running-before.log` and
  `artifacts/aws-running-after-post-commit.log`: no pending/running/stopping
  EC2 instances were present.

The pre-fix dry-run failure is preserved in
`artifacts/representative-pass-dry-run.log`:

`ERROR: auto_stop_at must be in the future before provisioning, got: 2026-05-28T14:00:00Z`

## Reviewer Focus

Please check that the representative preflight is not bypassing shutdown
cleanup protections, and that the auto-stop refresh behavior is narrow enough:
it only updates the ignored local operator `terraform.tfvars` deadline and does
not alter the Graviton lane, AMI, owner, or topology.
