# SPIRE Phase 13e Representative Performance Preflight Manifest

- Head SHA: `a6823cc39ad2a7e9bfc09397d9e871ee5607c92d`
- Task bucket: `reviews/task-30/1011-spire-phase13e-representative-performance-preflight`
- Lane: SPIRE Phase 13e representative performance readiness
- Fixture: checked-in representative priority and pooling suite JSON, plus one packet-local bad pooling suite
- Storage format: not applicable; harness preflight only
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `bash-n-preflight-representative-performance.log`

- Command: `bash -n scripts/spire-aws/preflight-representative-performance.sh`
- Result: shell syntax validation passed.

### `preflight-representative-performance.log`

- Command: `scripts/spire-aws/preflight-representative-performance.sh`
- Key result: `SPIRE representative performance preflight passed`.

### `make-preflight-representative-performance.log`

- Command: `make -C infra/spire-aws preflight-representative-performance`
- Key result: Make target invokes the representative performance preflight and passes.

### `preflight-bad-pooling-suite.log`

- Command: `SPIRE_AWS_REPRESENTATIVE_POOLING_SUITE=reviews/task-30/1011-spire-phase13e-representative-performance-preflight/artifacts/bad-suite/suite-representative-pooling.json scripts/spire-aws/preflight-representative-performance.sh`
- Expected result: exit code `2`.
- Key result: `ERROR: representative pooling suite enabled profile failed`, because the bad fixture disables production read profile capture on the enabled pooling row.

### `make-preflight-after-integration.log`

- Command: `make -C infra/spire-aws preflight`
- Key result lines:
  - `Success! The configuration is valid.`
  - `shellcheck not found; skipping shellcheck`
  - `SPIRE representative performance preflight passed`

### `make-preflight.log`

- Command: `make -C infra/spire-aws preflight`
- Result: initial full preflight run before the new representative check was integrated into the default target.

### `bad-suite/`

- Command: packet-local fixture copied from checked-in suites, with enabled pooling production profile disabled.
- Purpose: proves the new preflight fails when the representative pooling suite cannot produce required profile evidence.
