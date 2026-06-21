# Task 120 AWS Representative Suite Override

Please review this SPIRE AWS harness follow-up for the Task 120 Phase 5 run.

## Scope

The code checkpoint updates:

- `scripts/spire-aws/bench.sh`
- `scripts/spire-aws/preflight-representative-performance.sh`
- `scripts/spire-aws/run-representative-performance-pass.sh`

`bench.sh` now honors representative suite override environment variables:

- `SPIRE_AWS_REPRESENTATIVE_PRIORITY_SUITE`
- `SPIRE_AWS_REPRESENTATIVE_SUITE`
- `SPIRE_AWS_REPRESENTATIVE_POOLING_SUITE`

The representative preflight now asserts those hooks exist. This lets Task 120
run a packet-local Phase 5 suite through the existing `bench-representative-*`
targets, keeping the benchmark path on `ecaz bench suite`.

This checkpoint also processes the packet 012 reviewer note by mirroring the
legacy-default artifact-dir rejection into
`run-representative-performance-pass.sh`, so `--skip-preflight` and
`--reserve-artifact-dir` cannot target
`reviews/task-30/957-spire-aws-verification/artifacts`.

No benchmark behavior, scan behavior, SQL surface, Terraform resource
definition, or load logic changed.

## Evidence

- Artifact manifest:
  `reviews/task-120/014-aws-representative-suite-override/artifacts/manifest.md`
- Shell syntax validation:
  `reviews/task-120/014-aws-representative-suite-override/artifacts/bash-n.log`
- Legacy-default negative assertion:
  `reviews/task-120/014-aws-representative-suite-override/artifacts/legacy-default-reject.log`
- Render-only suite override check:
  `reviews/task-120/014-aws-representative-suite-override/artifacts/render-override.log`
- Read-only AWS/preflight validation:
  `reviews/task-120/014-aws-representative-suite-override/artifacts/preflight-task120-suite-override.log`

## Result

The existing AWS representative benchmark wrapper can now run a Task
120-specific suite while preserving the standard `ecaz bench suite` execution
path and packet-local artifact discipline. This clears the harness blocker for
the upcoming distributed near-data rerank AWS packet.

This is not Task 120 closeout.
