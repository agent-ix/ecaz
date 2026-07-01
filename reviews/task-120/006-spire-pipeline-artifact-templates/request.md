# Review Request: SPIRE Pipeline Artifact Templates

- task: Task 120
- packet: `reviews/task-120/006-spire-pipeline-artifact-templates`
- code commit under review: `8225571c2d98c8164b56a0355b79826809a1fb9d`

## Summary

This slice fixes a suite-runner issue found while preparing the phase-1 local
containment benchmark matrix: `ecaz bench suite` rewrote `${artifact_dir}` for
SPIRE pipeline `log_output`, but not the other SPIRE pipeline path fields.

Code changes:

- Rewrites `${artifact_dir}` for SPIRE pipeline truth corpus/cache paths.
- Rewrites `${artifact_dir}` for leaf/target block rank, target candidate-rank,
  miss-attribution, funnel, and stage-containment outputs.
- Adds a regression test covering command expansion and expected-artifact
  tracking for those SPIRE pipeline outputs.

## Validation

Artifacts are under `artifacts/`; see `artifacts/manifest.md`.

- `cargo fmt --check` passed.
- `cargo test -p ecaz-cli spire_pipeline` passed: 23 tests.
- `git diff --check` passed.

## Closeout Status

This is still benchmark plumbing. Task 120 remains open pending actual
10k/50k/100k `ecaz bench suite` measurement evidence.
