# Review Request: SPIRE Phase 13e Representative Make Artifact Guard

## Summary

This checkpoint closes the direct Make bypass for representative performance evidence preservation.

`scripts/spire-aws/preflight-representative-performance.sh` now validates `ARTIFACT_DIR` when Make exports it:

- It must be packet-local under `reviews/task-30/<packet>/artifacts`.
- It must not be the legacy default `957-spire-aws-verification` artifact bucket.
- Unless `SPIRE_AWS_REUSE_ARTIFACT_DIR=1` is explicit, it must not already contain representative topology, suite result, suite manifest, summary TSV, or pass marker output.

The standard entrypoint also propagates `--reuse-artifact-dir` into Make as `SPIRE_AWS_REUSE_ARTIFACT_DIR=1`, so deliberate continuation works consistently through both entrypoints.

No AWS provisioning was started.

## Evidence

Artifacts live under `reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard/artifacts/`.

- `make-preflight-clean-artifact-dir.log`: clean packet-local `ARTIFACT_DIR` passed, exit 0.
- `make-preflight-reused-artifact-dir.log`: direct Make preflight refused pre-existing representative output, command exit 2.
- `make-preflight-legacy-default-artifact-dir.log`: direct Make preflight refused the legacy default artifact directory, command exit 2.
- `make-preflight-reuse-override.log`: explicit `SPIRE_AWS_REUSE_ARTIFACT_DIR=1` passed, exit 0.
- `entrypoint-reuse-dry-run.log`: entrypoint dry-run prints the reuse env var in the future execution command.

## Reviewer Focus

- Confirm the direct Make path now fails before provisioning on stale/default evidence directories.
- Confirm the standard entrypoint and direct Make path use the same reuse control.
- Confirm standalone preflight remains usable for local checks that do not set `ARTIFACT_DIR`.
