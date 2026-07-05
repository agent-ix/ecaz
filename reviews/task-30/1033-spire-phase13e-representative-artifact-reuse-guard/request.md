# Review Request: SPIRE Phase 13e Representative Artifact Reuse Guard

## Summary

This checkpoint makes the remaining representative AWS performance entrypoint fail closed before provisioning when pointed at a packet artifact directory that already contains representative run output.

The default execute path now refuses existing:

- `aws-topology*.json`
- `suite-results-representative*.jsonl`
- `suite-manifest-representative*.json`
- `suite-representative*.json`
- `representative-*.tsv`
- `.representative-performance-pass.started`

Operators can still deliberately continue or reuse a packet by passing `--reuse-artifact-dir`; the default path protects packet evidence from accidental overwrite.

No AWS provisioning was started. The collision test used `--execute --skip-preflight` only to reach the new guard and exited before invoking Make.

## Evidence

Artifacts live under `reviews/task-30/1033-spire-phase13e-representative-artifact-reuse-guard/artifacts/`.

- `bash-n-run-representative-performance-pass.log`: syntax check, exit 0.
- `make-n-plan-representative-performance.log`: dry-run Make target still delegates to the representative entrypoint, exit 0.
- `execute-collision-guard.log`: execute path refused the pre-existing representative suite result with command exit 2 before provisioning.
- `preflight-representative-performance.log`: representative preflight still passes, exit 0.

## Reviewer Focus

- Confirm the guard runs before the entrypoint invokes `make pass-representative-performance`.
- Confirm the guarded patterns cover the representative latency/recall, pooling, topology, and summary artifacts that would otherwise be overwritten.
- Confirm explicit reuse remains available for a deliberate continuation through `--reuse-artifact-dir`.
