# SPIRE Phase 13e Representative Performance Preflight

This slice adds a local-only readiness gate for the next AWS representative performance proof. It does not provision or contact AWS.

## Change

- Added `scripts/spire-aws/preflight-representative-performance.sh`.
- Added `make -C infra/spire-aws preflight-representative-performance`.
- Integrated that check into `make -C infra/spire-aws preflight`.
- Updated the Phase 13e task note to record that packets `1009`, `1010`, and `1011` now harden the representative AWS proof path.

The preflight fails unless:

- the priority suite has representative recall coverage,
- the priority suite has representative latency coverage,
- the priority suite has production `spire-pipeline` rows with remote placements, query metrics, recall, production read profile, and production-read-only mode,
- the pooling suite has both disabled and enabled pooling profile rows with the same evidence fields,
- the representative performance Make target runs priority and pooling benches, then summary and verifier,
- the representative performance target does not include fault reruns.

## Validation

All validation is local. No AWS resources were started.

- `bash -n scripts/spire-aws/preflight-representative-performance.sh`
  - artifact: `artifacts/bash-n-preflight-representative-performance.log`
- `scripts/spire-aws/preflight-representative-performance.sh`
  - artifact: `artifacts/preflight-representative-performance.log`
- `make -C infra/spire-aws preflight-representative-performance`
  - artifact: `artifacts/make-preflight-representative-performance.log`
- `SPIRE_AWS_REPRESENTATIVE_POOLING_SUITE=artifacts/bad-suite/suite-representative-pooling.json scripts/spire-aws/preflight-representative-performance.sh`
  - artifact: `artifacts/preflight-bad-pooling-suite.log`
  - result: expected exit code `2`
- `make -C infra/spire-aws preflight`
  - artifact: `artifacts/make-preflight-after-integration.log`

## Next

The remaining Phase 13e blocker is still the explicit AWS representative performance run on the established Graviton lane. This preflight ensures the run cannot silently omit the prioritized latency, recall, pooling, and profile-evidence steps.
