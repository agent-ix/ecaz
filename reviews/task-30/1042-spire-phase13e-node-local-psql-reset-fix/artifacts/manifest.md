# Manifest: Node-Local psql Reset Fix

- Head SHA: `895323f1420731df5862fa5249b14c7af7f8e59c`
- Task bucket: `reviews/task-30`
- Packet: `reviews/task-30/1042-spire-phase13e-node-local-psql-reset-fix`
- Timestamp: `2026-05-27 20:46:00-07:00`
- Lane: local static/preflight validation for AWS representative performance harness
- Fixture: representative harness self-check fixtures only
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: AWS harness scripts; no PostgreSQL data plane exercised
- Isolated one-index-per-table: not applicable
- Shared-table surfaces: not applicable

## Commands

```bash
bash -n scripts/spire-aws/load.sh scripts/spire-aws/preflight-representative-performance.sh
ARTIFACT_DIR=reviews/task-30/1042-spire-phase13e-node-local-psql-reset-fix/artifacts scripts/spire-aws/preflight-representative-performance.sh
git diff --check
```

## Key Artifacts

- `bash-n-rerun.log`: shell syntax validation after the preflight guard adjustment.
- `representative-preflight-rerun.log`: representative preflight validation.
- `git-diff-check-rerun.log`: whitespace validation.

## Key Result Lines

```text
SPIRE representative performance preflight passed: priority=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-priority.json pooling=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-pooling.json
```
