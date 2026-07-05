# SPIRE Phase 13e Representative Pass Preflight Order Manifest

- Head SHA: `847e2fd6651079f593c5d811400352ffc4b351fc`
- Task bucket: `reviews/task-30/1012-spire-phase13e-pass-preflight-order`
- Lane: SPIRE Phase 13e representative performance AWS pass readiness
- Fixture: checked-in representative suites plus packet-local bad pooling suite
- Storage format: not applicable; harness preflight only
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `bash-n-preflight-order.log`

- Command: `bash -n scripts/spire-aws/preflight-representative-performance.sh`
- Result: shell syntax validation passed.

### `preflight-order.log`

- Command: `scripts/spire-aws/preflight-representative-performance.sh`
- Key result: `SPIRE representative performance preflight passed`.
- Coverage: includes the new static assertion that `pass-representative-performance-body` runs `preflight-representative-performance` before `provision`.

### `make-preflight-order.log`

- Command: `make -C infra/spire-aws preflight-representative-performance`
- Key result: Make target invokes the representative preflight and passes.

### `preflight-bad-pooling-suite.log`

- Command: `SPIRE_AWS_REPRESENTATIVE_POOLING_SUITE=reviews/task-30/1012-spire-phase13e-pass-preflight-order/artifacts/bad-suite/suite-representative-pooling.json scripts/spire-aws/preflight-representative-performance.sh`
- Expected result: exit code `2`.
- Key result: `ERROR: representative pooling suite enabled profile failed`.

### `make-preflight-with-order.log`

- Command: `make -C infra/spire-aws preflight`
- Key result lines:
  - `Success! The configuration is valid.`
  - `shellcheck not found; skipping shellcheck`
  - `SPIRE representative performance preflight passed`

### `bad-suite/`

- Command: packet-local copy of checked-in suites with enabled pooling production profile disabled.
- Purpose: proves the recurring preflight catches a representative pooling suite that cannot produce required profile evidence.
