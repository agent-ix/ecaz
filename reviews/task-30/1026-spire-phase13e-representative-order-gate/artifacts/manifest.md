# Artifact Manifest

- Head SHA: `09f3265bf`
- Task bucket: `reviews/task-30/1026-spire-phase13e-representative-order-gate`
- Timestamp: `2026-05-27T10:53:21-07:00`
- Lane: Phase 13e representative performance preflight
- Fixture / storage / rerank mode: local preflight only
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `git-show-stat.log`

- Command: `git show --stat --oneline --decorate --no-renames 09f3265bf`
- Key lines:
  - `09f3265bf (HEAD -> diskann-aws-optimization) Gate SPIRE representative pass order`
  - `scripts/spire-aws/preflight-representative-performance.sh | 54 ++++++++++++++++++++++`
  - `1 file changed, 54 insertions(+)`

### `bash-syntax.log`

- Command: `bash -n scripts/spire-aws/preflight-representative-performance.sh`
- Result: passed with no output.

### `preflight-representative-performance.log`

- Command: `bash scripts/spire-aws/preflight-representative-performance.sh`
- Key line:
  - `SPIRE representative performance preflight passed: priority=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-priority.json pooling=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-pooling.json`

## Notes

The existing untracked dry-run manifest at
`scripts/spire-aws/artifacts/representative-pooling/suite-manifest.json` was
left in place and was not staged.
