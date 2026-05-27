# SPIRE Phase 13e Representative Preflight Self-Check

This slice keeps the next AWS run focused on the priority evidence: pooling, latency, and recall. The representative performance preflight now exercises the summary gate locally before any provisioning path can proceed.

## Change

- `scripts/spire-aws/preflight-representative-performance.sh` now builds a small synthetic representative suite fixture under `target/`.
- The preflight runs `summarize-representative-performance.sh` and `verify-representative-performance-summary.sh` against a good fixture.
- It then mutates the generated pooling delta summary so `latency_p99_delta_ms=0` and requires the verifier to reject it.

This does not start AWS and does not touch the deferred fault-rerun path.

## Validation

- `bash -n scripts/spire-aws/preflight-representative-performance.sh`
  - artifact: `artifacts/bash-n-preflight-selfcheck.log`
- `scripts/spire-aws/preflight-representative-performance.sh`
  - artifact: `artifacts/preflight-selfcheck.log`
- `make -C infra/spire-aws preflight-representative-performance`
  - artifact: `artifacts/make-preflight-selfcheck.log`

## Next

The remaining Phase 13e proof is still the explicit Graviton `pass-representative-performance` run. With this slice, that path now fails locally before provisioning if the representative summary gate stops enforcing pooled-vs-unpooled p50/p95/p99 latency improvement and recall preservation.
