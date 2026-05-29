# Manifest: Node-Local Representative Staging Fix

- Head SHA: `aab77ccbf885b303e107df1738761a9fe740ce39`
- Task bucket: `reviews/task-30`
- Packet: `reviews/task-30/1040-spire-phase13e-node-local-staging-fix`
- Timestamp: `2026-05-27 19:04:09-07:00`
- Lane: local static/preflight validation for AWS representative performance harness
- Fixture: representative harness self-check fixtures only
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: AWS harness scripts; no PostgreSQL data plane exercised
- Isolated one-index-per-table: not applicable
- Shared-table surfaces: not applicable

## Commands

```bash
bash -n scripts/spire-aws/bootstrap-node.sh scripts/spire-aws/load.sh scripts/spire-aws/preflight-representative-performance.sh
ARTIFACT_DIR=reviews/task-30/1040-spire-phase13e-node-local-staging-fix/artifacts scripts/spire-aws/preflight-representative-performance.sh
git diff --check
```

## Key Artifacts

- `bash-n.log`: shell syntax validation.
- `representative-preflight.log`: representative preflight validation.
- `git-diff-check.log`: whitespace validation.

## Key Result Lines

```text
SPIRE representative performance preflight passed: priority=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-priority.json pooling=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-pooling.json
```
