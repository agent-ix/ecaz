# Review Request: SPIRE Phase 13e Representative Start Marker

## Summary

This checkpoint makes direct `make pass-representative-performance` reserve the packet artifact directory before provisioning.

Changes:

- Adds `mark-representative-performance-start` to `infra/spire-aws/Makefile`.
- Runs that marker target between `preflight-representative-performance` and `provision`.
- Adds `--reserve-artifact-dir` to the representative entrypoint so Make can create only `.representative-performance-pass.started` without starting AWS.
- Makes the entrypoint honor `SPIRE_AWS_REUSE_ARTIFACT_DIR=1` as well as `--reuse-artifact-dir`.
- Extends representative preflight sequence validation to require marker reservation before provisioning.

No AWS provisioning was started.

## Evidence

Artifacts live under `reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts/`.

- `make-mark-representative-performance-start.log`: marker target created `.representative-performance-pass.started`, exit 0.
- `start-marker-exists.log`: marker exists and is non-empty, exit 0.
- `make-mark-representative-performance-start-duplicate.log`: duplicate reservation refused with command exit 2.
- `make-preflight-after-start-marker.log`: direct Make preflight refused the reserved packet without reuse override, command exit 2.
- `make-mark-reuse-override-after-marker.log`: marker target accepted explicit reuse override, exit 0.
- `make-preflight-reuse-override-after-marker.log`: preflight accepted explicit reuse override, exit 0.
- `standalone-preflight-representative-performance-final.log`: sequence and summary preflight still passes, exit 0.

## Reviewer Focus

- Confirm the marker target runs before provisioning in `pass-representative-performance-body`.
- Confirm direct Make and the standard entrypoint share the same artifact reservation behavior.
- Confirm the marker protects interrupted representative AWS proof attempts without blocking explicit continuation.
