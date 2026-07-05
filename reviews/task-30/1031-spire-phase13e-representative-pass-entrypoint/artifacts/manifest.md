# Artifact Manifest

- Head SHA: `b8fbe56cc`
- Task bucket: `reviews/task-30/1031-spire-phase13e-representative-pass-entrypoint`
- Timestamp: `2026-05-27T11:12:26-07:00`
- Lane: Phase 13e representative AWS pass entrypoint
- Fixture / storage / rerank mode: not applicable; no benchmark fixture
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `bash-n-run-representative-performance-pass.log`

- Command: `bash -n scripts/spire-aws/run-representative-performance-pass.sh`
- Result: passed.

### `dry-run-skip-preflight.log`

- Command: `scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir reviews/task-30/1031-spire-phase13e-representative-pass-entrypoint/artifacts --skip-preflight`
- Result: passed, dry-run only.
- Key lines:
  - `execute=0`
  - `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 \`
  - `SPIRE_AWS_CONFIRM_PROVISION=yes \`
  - `make -C /home/peter/dev/ecaz/infra/spire-aws ARTIFACT_DIR=/home/peter/dev/ecaz/reviews/task-30/1031-spire-phase13e-representative-pass-entrypoint/artifacts pass-representative-performance`
  - `Dry run only. Re-run with --execute after explicit AWS approval.`

### `dry-run-bad-artifact-dir.log`

- Command: `scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir tmp/not-packet/artifacts --skip-preflight`
- Result: expected rejection, exit 2.
- Key line:
  - `ERROR: --artifact-dir must be packet-local under reviews/task-30/<packet>/artifacts`

### `dry-run-with-preflight.log`

- Command: `scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir reviews/task-30/1031-spire-phase13e-representative-pass-entrypoint/artifacts`
- Result: passed, dry-run only.
- Key lines:
  - `SPIRE AWS operator preflight passed: region=us-west-2 az=us-west-2a ami=ami-04e0d7d889f694536 coordinator=m7g.large remote=m7g.large remote_count=3`
  - `SPIRE AWS state preflight passed: local Terraform state has no managed resources`
  - `SPIRE AWS permission preflight passed`
  - `SPIRE representative performance preflight passed: priority=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-priority.json pooling=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-pooling.json`
  - `Dry run only. Re-run with --execute after explicit AWS approval.`

### `git-show-stat.log`

- Command: `git show --stat --oneline --decorate --no-renames b8fbe56cc`
- Key lines:
  - `b8fbe56cc (HEAD -> diskann-aws-optimization) Add SPIRE representative pass entrypoint`
  - `scripts/spire-aws/run-representative-performance-pass.sh | 118 +++++++++++++++++++++`

### `git-diff-check.log`

- Command: `git diff --check HEAD^ HEAD -- scripts/spire-aws/run-representative-performance-pass.sh plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`
- Result: passed with no output.

## Notes

No provisioning, teardown, EC2 start, EC2 stop, Terraform apply, or Terraform
destroy command was run. The only real AWS calls were read-only calls made by
the dry-run preflight path.
