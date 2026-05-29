# Review Request: SPIRE Phase 13e Representative Plan Target

## Summary

This checkpoint wires the reviewed representative performance dry-run entrypoint into the standard AWS Make surface:

- Adds `make -C infra/spire-aws plan-representative-performance`.
- Keeps the target dry-run only by delegating to `scripts/spire-aws/run-representative-performance-pass.sh` without `--execute`.
- Extends representative preflight coverage so the dry-run/execute entrypoint must remain executable.
- Records the target in the Phase 13e task file.

No AWS provisioning or representative execution was started in this packet. The target ran read-only preflights and printed the exact command to use only after explicit AWS approval.

## Evidence

Artifacts live under `reviews/task-30/1032-spire-phase13e-representative-plan-target/artifacts/`.

- `bash-n-representative-entrypoint.log`: syntax validation for representative preflight and entrypoint, exit 0.
- `make-n-plan-representative-performance.log`: Make dry-run expands to the representative entrypoint, exit 0.
- `preflight-representative-performance.log`: representative suite/preflight validation, exit 0.
- `make-plan-representative-performance.log`: actual dry-run target, exit 0; ran preflights only and printed the future `pass-representative-performance` command.

## Reviewer Focus

- Confirm the Make target preserves the established AWS path instead of introducing a new provisioning flow.
- Confirm the target cannot provision unless the entrypoint is rerun with `--execute`.
- Confirm the representative preflight now protects the entrypoint's executable bit.
