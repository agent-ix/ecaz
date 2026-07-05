# Artifact Manifest

Task bucket: `reviews/task-30/963-spire-phase13e-aws-remote-shard-flow`

Head SHA: `300ca60fef6bac22754aa227154eb71a14cda721`

Timestamp: `2026-05-25T09:59:23-07:00`

## bash-syntax-spire-aws-load-register.log

- Command: `script -q -c "bash -n scripts/spire-aws/load.sh scripts/spire-aws/register.sh" reviews/task-30/963-spire-phase13e-aws-remote-shard-flow/artifacts/bash-syntax-spire-aws-load-register.log`
- Lane: local operator-script syntax validation
- Fixture: none
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: AWS operator scripts only
- Result:
  - `COMMAND_EXIT_CODE="0"`

## shellcheck-spire-aws-load-register.log

- Command: `script -q -c "bash -lc 'if command -v shellcheck >/dev/null 2>&1; then shellcheck scripts/spire-aws/load.sh scripts/spire-aws/register.sh; else echo shellcheck-not-found; fi'" reviews/task-30/963-spire-phase13e-aws-remote-shard-flow/artifacts/shellcheck-spire-aws-load-register.log`
- Lane: optional local lint availability check
- Fixture: none
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: AWS operator scripts only
- Result:
  - `shellcheck-not-found`
