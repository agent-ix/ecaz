# Task 30 Packet 1035 Manifest

- head SHA before checkpoint: `1c373521fb51f6390c19c36757a35e3120915898`
- code checkpoint SHA: `c9d41be59`
- task bucket: `reviews/task-30/1035-spire-phase13e-representative-start-marker`
- timestamp: `2026-05-27T11:31:15-07:00`
- lane: SPIRE Phase 13e representative AWS performance direct Make start marker
- fixture: local start-marker reservation; no AWS provisioning
- storage format: task packet artifacts
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surfaces: not applicable

Artifacts:

- `bash-n-representative-start-marker-final.log`
  - command: `script -q -e -c "bash -n scripts/spire-aws/preflight-representative-performance.sh scripts/spire-aws/run-representative-performance-pass.sh" reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts/bash-n-representative-start-marker-final.log`
  - result: exit 0.

- `standalone-preflight-representative-performance-final.log`
  - command: `script -q -e -c "bash scripts/spire-aws/preflight-representative-performance.sh" reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts/standalone-preflight-representative-performance-final.log`
  - result: exit 0; standalone preflight still validates the Make sequence and scripts.

- `make-n-mark-representative-performance-start.log`
  - command: `script -q -e -c "make -C infra/spire-aws -n ARTIFACT_DIR=reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts mark-representative-performance-start" reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts/make-n-mark-representative-performance-start.log`
  - result: exit 0; target expands to the representative entrypoint with `--reserve-artifact-dir`.

- `make-mark-representative-performance-start.log`
  - command: `script -q -e -c "make -C infra/spire-aws ARTIFACT_DIR=reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts mark-representative-performance-start" reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts/make-mark-representative-performance-start.log`
  - result: exit 0; target created `.representative-performance-pass.started`.

- `.representative-performance-pass.started`
  - result: marker file created by `mark-representative-performance-start` with `started_at=2026-05-27T11:30:14-07:00`.

- `start-marker-exists.log`
  - command: `script -q -e -c "test -s reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts/.representative-performance-pass.started" reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts/start-marker-exists.log`
  - result: exit 0.

- `make-mark-representative-performance-start-duplicate.log`
  - command: `script -q -c "make -C infra/spire-aws ARTIFACT_DIR=reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts mark-representative-performance-start" reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts/make-mark-representative-performance-start-duplicate.log`
  - result: command exit 2; duplicate marker reservation was refused.

- `make-preflight-after-start-marker.log`
  - command: `script -q -c "make -C infra/spire-aws ARTIFACT_DIR=reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts preflight-representative-performance" reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts/make-preflight-after-start-marker.log`
  - result: command exit 2; direct Make preflight refused the reserved packet without reuse override.

- `make-mark-reuse-override-after-marker.log`
  - command: `script -q -e -c "SPIRE_AWS_REUSE_ARTIFACT_DIR=1 make -C infra/spire-aws ARTIFACT_DIR=reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts mark-representative-performance-start" reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts/make-mark-reuse-override-after-marker.log`
  - result: exit 0; deliberate reuse override works for the marker target.

- `make-preflight-reuse-override-after-marker.log`
  - command: `script -q -e -c "SPIRE_AWS_REUSE_ARTIFACT_DIR=1 make -C infra/spire-aws ARTIFACT_DIR=reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts preflight-representative-performance" reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts/make-preflight-reuse-override-after-marker.log`
  - result: exit 0; deliberate reuse override works for direct Make preflight.

- `make-n-pass-representative-performance-body.log`
  - command: `script -q -e -c "make -C infra/spire-aws -n ARTIFACT_DIR=reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts pass-representative-performance-body" reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts/make-n-pass-representative-performance-body.log`
  - result: command exit 2; retained as non-gating evidence that parent `make -n` is not a safe validator for recursive pass bodies.

- `git-diff-check.log`
  - command: `script -q -e -c "git diff --check HEAD^ HEAD -- infra/spire-aws/Makefile scripts/spire-aws/preflight-representative-performance.sh scripts/spire-aws/run-representative-performance-pass.sh plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md" reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts/git-diff-check.log`
  - result: exit 0.

- `git-show-stat.log`
  - command: `script -q -e -c "git show --stat --oneline HEAD" reviews/task-30/1035-spire-phase13e-representative-start-marker/artifacts/git-show-stat.log`
  - result: `c9d41be59 Reserve SPIRE representative packet before provisioning`, 4 files changed, 54 insertions, 24 deletions.
