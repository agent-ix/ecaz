# SPIRE Phase 13e Summary Suite-Driven Sweep Gate

This slice removes a stale local assumption from the representative performance verifier. The verifier no longer hardcodes the expected nprobe sweep; it reads the packet-local suite configs that `bench.sh` writes into the artifact directory.

## Change

- `scripts/spire-aws/verify-representative-performance-summary.sh` now requires:
  - `suite-representative-priority.json`;
  - `suite-representative-pooling.json`;
  - matching priority and pooling top-k=10 nprobe sweeps.
- The expected sweep used for latency, recall, production profile, and pooling delta completeness now comes from those suite configs.
- `scripts/spire-aws/preflight-representative-performance.sh` copies the checked-in suite configs into its synthetic self-check output before invoking the verifier.

No AWS resources were started.

## Validation

- `bash -n scripts/spire-aws/verify-representative-performance-summary.sh scripts/spire-aws/preflight-representative-performance.sh`
  - artifact: `artifacts/bash-n-suite-driven-sweep.log`
- `scripts/spire-aws/preflight-representative-performance.sh`
  - artifact: `artifacts/preflight-suite-driven-sweep.log`
- `make -C infra/spire-aws preflight-representative-performance`
  - artifact: `artifacts/make-preflight-suite-driven-sweep.log`
- `SPIRE_AWS_REPRESENTATIVE_POOLING_SUITE=artifacts/bad-suite/suite-representative-pooling.json scripts/spire-aws/preflight-representative-performance.sh`
  - artifact: `artifacts/preflight-bad-pooling-sweep.log`
  - result: expected exit code `2`; the preflight rejects a pooling suite whose sweep is `[8,16,24]` while the priority suite is `[8,16,24,32]`.

## Next

The remaining Phase 13e proof is still the explicit Graviton `pass-representative-performance` run. That run now verifies completeness against the suite configs included in the packet artifacts, rather than a duplicated hardcoded sweep list.
