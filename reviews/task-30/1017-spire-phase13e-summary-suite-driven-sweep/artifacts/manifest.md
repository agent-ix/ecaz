# SPIRE Phase 13e Summary Suite-Driven Sweep Gate Manifest

- Head SHA: `216c01654bec4bf4673641f04343b10371650dbe`
- Task bucket: `reviews/task-30/1017-spire-phase13e-summary-suite-driven-sweep`
- Lane: SPIRE Phase 13e representative performance readiness
- Fixture: checked-in representative suite configs plus packet-local bad pooling suite
- Storage format: not applicable
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Timestamp: `2026-05-27T16:26:18Z`

## Artifacts

### `bash-n-suite-driven-sweep.log`

- Command: `bash -n scripts/spire-aws/verify-representative-performance-summary.sh scripts/spire-aws/preflight-representative-performance.sh`
- Result: shell syntax validation passed.

### `preflight-suite-driven-sweep.log`

- Command: `scripts/spire-aws/preflight-representative-performance.sh`
- Key result: `SPIRE representative performance preflight passed`.
- Coverage: verifier reads suite configs from the generated sample output and checks all configured top-k=10 nprobe values.

### `make-preflight-suite-driven-sweep.log`

- Command: `make -C infra/spire-aws preflight-representative-performance`
- Key result: Make target invokes the representative performance preflight and passes.

### `bad-suite/suite-representative-pooling.json`

- Command: `jq '(.steps[] | select(.kind == "spire-pipeline") | .sweep) = [8,16,24]' scripts/spire-aws/suite-representative-pooling.json > reviews/task-30/1017-spire-phase13e-summary-suite-driven-sweep/artifacts/bad-suite/suite-representative-pooling.json`
- Purpose: proves the verifier is driven by suite configs and rejects priority/pooling sweep mismatch.

### `preflight-bad-pooling-sweep.log`

- Command: `SPIRE_AWS_REPRESENTATIVE_POOLING_SUITE=reviews/task-30/1017-spire-phase13e-summary-suite-driven-sweep/artifacts/bad-suite/suite-representative-pooling.json scripts/spire-aws/preflight-representative-performance.sh`
- Expected result: exit code `2`.
- Key result: `ERROR: representative priority and pooling nprobe sweeps differ: priority=[8 16 24 32] pooling=[8 16 24]`.

## Scope

This is a local gate hardening packet only. It does not run AWS and does not alter the deferred fault-rerun path.
