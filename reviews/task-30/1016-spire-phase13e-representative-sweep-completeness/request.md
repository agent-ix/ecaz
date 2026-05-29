# SPIRE Phase 13e Representative Sweep Completeness

This slice tightens the representative performance evidence gate so an AWS packet cannot pass with only a partial nprobe sweep.

## Change

- `scripts/spire-aws/verify-representative-performance-summary.sh` now requires representative latency, recall, production SPIRE pipeline, production read profile, and pooling delta evidence for all priority nprobe values: `8`, `16`, `24`, and `32`.
- `scripts/spire-aws/preflight-representative-performance.sh` now generates self-check rows for all four priority nprobe values, so the stricter verifier is exercised locally before provisioning.

No AWS resources were started.

## Validation

- `bash -n scripts/spire-aws/verify-representative-performance-summary.sh scripts/spire-aws/preflight-representative-performance.sh`
  - artifact: `artifacts/bash-n-verifier-preflight.log`
- `scripts/spire-aws/preflight-representative-performance.sh`
  - artifact: `artifacts/preflight-sweep-completeness.log`
- `make -C infra/spire-aws preflight-representative-performance`
  - artifact: `artifacts/make-preflight-sweep-completeness.log`
- `scripts/spire-aws/verify-representative-performance-summary.sh reviews/task-30/1013-spire-phase13e-pooling-latency-delta-gate/artifacts/sample-output`
  - artifact: `artifacts/verify-missing-sweep-rejected.log`
  - result: expected exit code `2`; the previous one-nprobe sample is now rejected.

## Next

The remaining Phase 13e proof is still the explicit Graviton `pass-representative-performance` run. This gate now requires complete representative sweep evidence, not just one successful nprobe cell.
