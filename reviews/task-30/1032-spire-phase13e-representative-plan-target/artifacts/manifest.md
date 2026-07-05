# Task 30 Packet 1032 Manifest

- head SHA before checkpoint: `742c0e9cb623beb0e01daa68bf67637d4a5c87e6`
- code checkpoint SHA: `e868d5401`
- task bucket: `reviews/task-30/1032-spire-phase13e-representative-plan-target`
- timestamp: `2026-05-27T11:16:27-07:00`
- lane: SPIRE Phase 13e representative AWS performance plan target
- fixture: representative AWS pass dry-run only
- storage format: task packet artifacts
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surfaces: not applicable

Artifacts:

- `bash-n-representative-entrypoint.log`
  - command: `script -q -e -c "bash -n scripts/spire-aws/preflight-representative-performance.sh scripts/spire-aws/run-representative-performance-pass.sh" reviews/task-30/1032-spire-phase13e-representative-plan-target/artifacts/bash-n-representative-entrypoint.log`
  - result: exit 0.

- `make-n-plan-representative-performance.log`
  - command: `script -q -e -c "make -C infra/spire-aws -n ARTIFACT_DIR=reviews/task-30/1032-spire-phase13e-representative-plan-target/artifacts plan-representative-performance" reviews/task-30/1032-spire-phase13e-representative-plan-target/artifacts/make-n-plan-representative-performance.log`
  - result: exit 0; dry-run Make output expands to `scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir ...`.

- `preflight-representative-performance.log`
  - command: `script -q -e -c "bash scripts/spire-aws/preflight-representative-performance.sh" reviews/task-30/1032-spire-phase13e-representative-plan-target/artifacts/preflight-representative-performance.log`
  - result: exit 0; representative performance preflight passed for priority and pooling suite configs.

- `make-plan-representative-performance.log`
  - command: `script -q -e -c "make -C infra/spire-aws ARTIFACT_DIR=reviews/task-30/1032-spire-phase13e-representative-plan-target/artifacts plan-representative-performance" reviews/task-30/1032-spire-phase13e-representative-plan-target/artifacts/make-plan-representative-performance.log`
  - result: exit 0; dry-run target ran read-only preflights, allowed known pre-existing S3 residue by policy, and printed the exact future execution command:
    `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 SPIRE_AWS_CONFIRM_PROVISION=yes make -C /home/peter/dev/ecaz/infra/spire-aws ARTIFACT_DIR=/home/peter/dev/ecaz/reviews/task-30/1032-spire-phase13e-representative-plan-target/artifacts pass-representative-performance`

- `git-diff-check.log`
  - command: `script -q -e -c "git diff --check HEAD^ HEAD -- infra/spire-aws/Makefile scripts/spire-aws/preflight-representative-performance.sh plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md" reviews/task-30/1032-spire-phase13e-representative-plan-target/artifacts/git-diff-check.log`
  - result: exit 0.

- `git-show-stat.log`
  - command: `script -q -e -c "git show --stat --oneline HEAD" reviews/task-30/1032-spire-phase13e-representative-plan-target/artifacts/git-show-stat.log`
  - result: `e868d5401 Wire SPIRE representative pass plan target`, 3 files changed, 9 insertions, 1 deletion.
