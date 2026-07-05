# SPIRE Phase 13e Representative Sweep Completeness Manifest

- Head SHA: `4b708e1cc591b194d8c04c664637ba7cc6601879`
- Task bucket: `reviews/task-30/1016-spire-phase13e-representative-sweep-completeness`
- Lane: SPIRE Phase 13e representative performance readiness
- Fixture: representative summary verifier plus synthetic preflight self-check rows
- Storage format: not applicable
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Timestamp: `2026-05-27T16:22:29Z`

## Artifacts

### `bash-n-verifier-preflight.log`

- Command: `bash -n scripts/spire-aws/verify-representative-performance-summary.sh scripts/spire-aws/preflight-representative-performance.sh`
- Result: shell syntax validation passed.

### `preflight-sweep-completeness.log`

- Command: `scripts/spire-aws/preflight-representative-performance.sh`
- Key result: `SPIRE representative performance preflight passed`.
- Coverage: exercises good and bad summary self-check rows across priority nprobe values `8`, `16`, `24`, and `32`.

### `make-preflight-sweep-completeness.log`

- Command: `make -C infra/spire-aws preflight-representative-performance`
- Key result: Make target invokes the representative performance preflight and passes.

### `verify-missing-sweep-rejected.log`

- Command: `scripts/spire-aws/verify-representative-performance-summary.sh reviews/task-30/1013-spire-phase13e-pooling-latency-delta-gate/artifacts/sample-output`
- Expected result: exit code `2`.
- Key result: `ERROR: representative latency p50/p95/p99 rows for all priority nprobe values missing or incomplete`.

## Scope

This is a local gate hardening packet only. It does not run AWS and does not alter the deferred fault-rerun path.
