# SPIRE Phase 13e Summary Nprobe Log Manifest

- Head SHA: `77fae2237684f0c058484896cf1f1e79713ea419`
- Task bucket: `reviews/task-30/1019-spire-phase13e-summary-nprobe-log`
- Lane: SPIRE Phase 13e representative performance readiness
- Fixture: packet-local synthetic summary output plus checked-in representative suite configs
- Storage format: not applicable
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Timestamp: `2026-05-27T16:33:05Z`

## Artifacts

### `bash-n-summary-verifier.log`

- Command: `bash -n scripts/spire-aws/verify-representative-performance-summary.sh`
- Result: shell syntax validation passed.

### `preflight-summary-nprobe-log.log`

- Command: `scripts/spire-aws/preflight-representative-performance.sh`
- Key result: `SPIRE representative performance preflight passed`.

### `make-preflight-summary-nprobe-log.log`

- Command: `make -C infra/spire-aws preflight-representative-performance`
- Key result: Make target invokes representative performance preflight and passes.

### `sample-output/`

- Command: packet-local synthetic summary output with one base row expanded across nprobes `8`, `16`, `24`, and `32`, plus checked-in representative suite configs copied into the directory.
- Purpose: direct verifier fixture for the new success log.

### `verify-summary-nprobe-log.log`

- Command: `scripts/spire-aws/verify-representative-performance-summary.sh reviews/task-30/1019-spire-phase13e-summary-nprobe-log/artifacts/sample-output`
- Key result: `representative performance summary verified: reviews/task-30/1019-spire-phase13e-summary-nprobe-log/artifacts/sample-output nprobes=[8 16 24 32]`.

## Scope

This is a local evidence-log quality packet only. It does not run AWS and does not alter the deferred fault-rerun path.
