# Review Request: SPIRE Representative Pass Entrypoint

Task: `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`

Code commit: `b8fbe56cc`

## Summary

This checkpoint adds a standard script for the remaining Phase 13e AWS proof:
`scripts/spire-aws/run-representative-performance-pass.sh`.

The script is dry-run by default and does not provision unless `--execute` is
present. It:

- requires a packet-local artifact directory under
  `reviews/task-30/<packet>/artifacts`;
- runs the current preflight stack by default;
- uses the reviewed pre-existing residue exception by default;
- prints the exact `pass-representative-performance` command with
  `SPIRE_AWS_CONFIRM_PROVISION=yes`;
- runs the pass only when rerun with `--execute` after explicit AWS approval.

No AWS provisioning was started.

## Validation

- `bash -n scripts/spire-aws/run-representative-performance-pass.sh`
- dry run with `--skip-preflight`
- bad artifact directory rejection
- dry run with preflights enabled
  - operator preflight passed on the established Graviton lane
  - state preflight passed
  - permissions preflight passed under the reviewed residue exception
  - representative performance preflight passed
- `git diff --check HEAD^ HEAD -- scripts/spire-aws/run-representative-performance-pass.sh plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/bash-n-run-representative-performance-pass.log`
- `artifacts/dry-run-skip-preflight.log`
- `artifacts/dry-run-bad-artifact-dir.log`
- `artifacts/dry-run-with-preflight.log`
- `artifacts/git-show-stat.log`
- `artifacts/git-diff-check.log`

The existing untracked SPIRE artifact directories were left untouched and were
not staged.
