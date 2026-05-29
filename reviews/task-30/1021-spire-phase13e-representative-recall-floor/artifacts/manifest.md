# Artifact Manifest

- Head SHA: `70074aa89`
- Task bucket: `reviews/task-30/1021-spire-phase13e-representative-recall-floor`
- Timestamp: `2026-05-27T10:03:22-07:00`
- Lane: Phase 13e representative AWS performance readiness
- Fixture / storage / rerank mode: local suite and summary-gate validation only
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `git-show-stat.log`

- Command: `git show --stat --oneline --decorate --no-renames 70074aa89`
- Key lines:
  - `70074aa89 Gate SPIRE representative recall floor`
  - `4 files changed, 163 insertions(+), 2 deletions(-)`

### `jq-suite-parse.log`

- Command: `jq empty scripts/spire-aws/suite-representative-priority.json scripts/spire-aws/suite-representative-pooling.json`
- Result: passed with no output.

### `bash-syntax.log`

- Command: `bash -n scripts/spire-aws/preflight-representative-performance.sh scripts/spire-aws/verify-representative-performance-summary.sh`
- Result: passed with no output.

### `preflight-representative-performance.log`

- Command: `bash scripts/spire-aws/preflight-representative-performance.sh`
- Key line:
  - `SPIRE representative performance preflight passed: priority=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-priority.json pooling=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-pooling.json`

### `suite-priority-dry-run.log`

- Command: `target/release/ecaz bench suite --config scripts/spire-aws/suite-representative-priority.json --dry-run`
- Key lines:
  - `13a3a-recall-k10 -> --database tqvector_bench bench recall ... --sweep "8,16,24,32" --queries-limit 1000`
  - `13e3-production-read-profile-k10 -> --database tqvector_bench bench spire-pipeline ... --include-recall --include-production-read-profile --production-read-only`

### `suite-pooling-dry-run.log`

- Command: `target/release/ecaz bench suite --config scripts/spire-aws/suite-representative-pooling.json --dry-run`
- Key lines:
  - `13e4-pooling-disabled-profile-k10 -> --database tqvector_bench bench spire-pipeline ... --include-recall --include-production-read-profile --production-read-only`
  - `13e4-pooling-enabled-profile-k10 -> --database tqvector_bench bench spire-pipeline ... --include-recall --include-production-read-profile --production-read-only`

## Notes

Dry-run suite manifests generated under `scripts/spire-aws/artifacts/` were
removed after the packet-local logs were captured.
