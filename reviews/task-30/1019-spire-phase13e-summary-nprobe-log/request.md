# SPIRE Phase 13e Summary Nprobe Log

This slice makes the representative performance verifier success line self-describing for the next AWS packet.

## Change

- `scripts/spire-aws/verify-representative-performance-summary.sh` now prints the suite-driven nprobe list it verified:
  - `representative performance summary verified: <artifact-dir> nprobes=[8 16 24 32]`

No AWS resources were started.

## Validation

- `bash -n scripts/spire-aws/verify-representative-performance-summary.sh`
  - artifact: `artifacts/bash-n-summary-verifier.log`
- `scripts/spire-aws/preflight-representative-performance.sh`
  - artifact: `artifacts/preflight-summary-nprobe-log.log`
- `make -C infra/spire-aws preflight-representative-performance`
  - artifact: `artifacts/make-preflight-summary-nprobe-log.log`
- `scripts/spire-aws/verify-representative-performance-summary.sh artifacts/sample-output`
  - artifact: `artifacts/verify-summary-nprobe-log.log`
  - key result: `nprobes=[8 16 24 32]`

## Next

The remaining Phase 13e proof is still the explicit Graviton `pass-representative-performance` run. This change makes the accepted representative sweep visible in that run's verifier log.
