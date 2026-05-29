# Artifact Manifest

- Head SHA: `b21c718c9`
- Task bucket: `reviews/task-30/1024-spire-phase13e-representative-watchdog-gate`
- Timestamp: `2026-05-27T10:47:54-07:00`
- Lane: Phase 13e representative performance preflight
- Fixture / storage / rerank mode: local preflight only
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `git-show-stat.log`

- Command: `git show --stat --oneline --decorate --no-renames b21c718c9`
- Key lines:
  - `b21c718c9 (HEAD -> diskann-aws-optimization) Gate SPIRE representative pass watchdog wiring`
  - `scripts/spire-aws/preflight-representative-performance.sh | 57 ++++++++++++++++++++++`
  - `1 file changed, 57 insertions(+)`

### `bash-syntax.log`

- Command: `bash -n scripts/spire-aws/preflight-representative-performance.sh`
- Result: passed with no output.

### `preflight-representative-performance.log`

- Command: `bash scripts/spire-aws/preflight-representative-performance.sh`
- Key line:
  - `SPIRE representative performance preflight passed: priority=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-priority.json pooling=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-pooling.json`

## Notes

The preflight run includes an embedded negative self-check that creates a
temporary watchdog file with `pass-representative-performance-body` timeout set
to `60` seconds and requires the timeout gate to reject it.

The existing untracked dry-run manifest at
`scripts/spire-aws/artifacts/representative-pooling/suite-manifest.json` was
left in place and was not staged.
