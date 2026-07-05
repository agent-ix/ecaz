# Task 30 Packet 1034 Manifest

- head SHA before checkpoint: `46917d76d0cd90e460cfa420f8c891da983ec0d7`
- code checkpoint SHA: `bbb9bcef2`
- task bucket: `reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard`
- timestamp: `2026-05-27T11:25:48-07:00`
- lane: SPIRE Phase 13e representative AWS performance direct Make preflight
- fixture: local artifact-directory guard; no AWS provisioning
- storage format: task packet artifacts
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surfaces: not applicable

Artifacts:

- `bash-n-preflight-representative-performance.log`
  - command: `script -q -e -c "bash -n scripts/spire-aws/preflight-representative-performance.sh" reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard/artifacts/bash-n-preflight-representative-performance.log`
  - result: exit 0.

- `standalone-preflight-representative-performance.log`
  - command: `script -q -e -c "bash scripts/spire-aws/preflight-representative-performance.sh" reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard/artifacts/standalone-preflight-representative-performance.log`
  - result: exit 0; standalone local preflight remains usable without `ARTIFACT_DIR`.

- `make-preflight-clean-artifact-dir.log`
  - command: `script -q -e -c "make -C infra/spire-aws ARTIFACT_DIR=reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard/artifacts preflight-representative-performance" reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard/artifacts/make-preflight-clean-artifact-dir.log`
  - result: exit 0 before the fixture file was added; packet-local clean artifact directory passed.

- `suite-results-representative-priority.jsonl`
  - fixture: pre-existing representative suite result marker used to prove direct Make preflight refuses reuse.

- `make-preflight-reused-artifact-dir.log`
  - command: `script -q -c "make -C infra/spire-aws ARTIFACT_DIR=reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard/artifacts preflight-representative-performance" reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard/artifacts/make-preflight-reused-artifact-dir.log`
  - result: command exit 2; preflight refused an artifact directory with representative output.

- `make-preflight-legacy-default-artifact-dir.log`
  - command: `script -q -c "make -C infra/spire-aws preflight-representative-performance" reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard/artifacts/make-preflight-legacy-default-artifact-dir.log`
  - result: command exit 2; preflight refused the legacy default `957-spire-aws-verification` artifact directory.

- `make-preflight-reuse-override.log`
  - command: `script -q -e -c "SPIRE_AWS_REUSE_ARTIFACT_DIR=1 make -C infra/spire-aws ARTIFACT_DIR=reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard/artifacts preflight-representative-performance" reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard/artifacts/make-preflight-reuse-override.log`
  - result: exit 0; deliberate reuse remains available.

- `bash-n-representative-artifact-guards.log`
  - command: `script -q -e -c "bash -n scripts/spire-aws/preflight-representative-performance.sh scripts/spire-aws/run-representative-performance-pass.sh" reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard/artifacts/bash-n-representative-artifact-guards.log`
  - result: exit 0 after propagating the reuse override through the entrypoint.

- `entrypoint-reuse-dry-run.log`
  - command: `script -q -e -c "scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard/artifacts --reuse-artifact-dir --skip-preflight" reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard/artifacts/entrypoint-reuse-dry-run.log`
  - result: exit 0; dry-run output includes `SPIRE_AWS_REUSE_ARTIFACT_DIR=1` in the future execution command.

- `git-diff-check.log`
  - command: `script -q -e -c "git diff --check HEAD^ HEAD -- scripts/spire-aws/preflight-representative-performance.sh scripts/spire-aws/run-representative-performance-pass.sh plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md" reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard/artifacts/git-diff-check.log`
  - result: exit 0.

- `git-show-stat.log`
  - command: `script -q -e -c "git show --stat --oneline HEAD" reviews/task-30/1034-spire-phase13e-representative-make-artifact-guard/artifacts/git-show-stat.log`
  - result: `bbb9bcef2 Guard SPIRE representative Make artifact reuse`, 3 files changed, 65 insertions.
