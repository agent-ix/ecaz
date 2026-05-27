# Task 30 Packet 1033 Manifest

- head SHA before checkpoint: `fd275efffcc180cf3fc35638ce1974623cc1fe25`
- code checkpoint SHA: `3afd00b1b`
- task bucket: `reviews/task-30/1033-spire-phase13e-representative-artifact-reuse-guard`
- timestamp: `2026-05-27T11:21:27-07:00`
- lane: SPIRE Phase 13e representative AWS performance entrypoint
- fixture: local artifact reuse collision guard; no AWS provisioning
- storage format: task packet artifacts
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surfaces: not applicable

Artifacts:

- `suite-results-representative-priority.jsonl`
  - fixture: pre-existing representative suite result marker used to prove the execute path refuses reuse before provisioning.

- `bash-n-run-representative-performance-pass.log`
  - command: `script -q -e -c "bash -n scripts/spire-aws/run-representative-performance-pass.sh" reviews/task-30/1033-spire-phase13e-representative-artifact-reuse-guard/artifacts/bash-n-run-representative-performance-pass.log`
  - result: exit 0.

- `make-n-plan-representative-performance.log`
  - command: `script -q -e -c "make -C infra/spire-aws -n ARTIFACT_DIR=reviews/task-30/1033-spire-phase13e-representative-artifact-reuse-guard/artifacts plan-representative-performance" reviews/task-30/1033-spire-phase13e-representative-artifact-reuse-guard/artifacts/make-n-plan-representative-performance.log`
  - result: exit 0; Make dry-run still delegates to the representative pass entrypoint.

- `execute-collision-guard.log`
  - command: `script -q -c "scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir reviews/task-30/1033-spire-phase13e-representative-artifact-reuse-guard/artifacts --execute --skip-preflight" reviews/task-30/1033-spire-phase13e-representative-artifact-reuse-guard/artifacts/execute-collision-guard.log`
  - result: command exit 2 before Make execution; log contains `ERROR: refusing to reuse artifact directory with prior representative output`.

- `preflight-representative-performance.log`
  - command: `script -q -e -c "bash scripts/spire-aws/preflight-representative-performance.sh" reviews/task-30/1033-spire-phase13e-representative-artifact-reuse-guard/artifacts/preflight-representative-performance.log`
  - result: exit 0.

- `git-diff-check.log`
  - command: `script -q -e -c "git diff --check HEAD^ HEAD -- scripts/spire-aws/run-representative-performance-pass.sh plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md" reviews/task-30/1033-spire-phase13e-representative-artifact-reuse-guard/artifacts/git-diff-check.log`
  - result: exit 0.

- `git-show-stat.log`
  - command: `script -q -e -c "git show --stat --oneline HEAD" reviews/task-30/1033-spire-phase13e-representative-artifact-reuse-guard/artifacts/git-show-stat.log`
  - result: `3afd00b1b Guard SPIRE representative artifact reuse`, 2 files changed, 33 insertions.
