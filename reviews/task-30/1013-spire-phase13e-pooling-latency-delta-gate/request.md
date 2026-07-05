# SPIRE Phase 13e Pooling Latency Delta Gate

This slice tightens the representative performance acceptance gate around the priority items for the next AWS run: pooling, latency, and recall.

## Change

- `scripts/spire-aws/summarize-representative-performance.sh` now emits pooled-vs-unpooled latency deltas for p50, p95, and p99.
- `scripts/spire-aws/verify-representative-performance-summary.sh` now fails closed unless pooling comparison rows include socket opens, p50/p95/p99 latency, recall, and endpoint identity query evidence.
- The pooling delta verifier now requires positive p50, p95, and p99 latency improvement, positive socket-open reduction, endpoint-identity query evidence on both sides, and zero recall regression.

No AWS resources were started. Fault rerun work remains deferred behind representative performance evidence.

## Validation

- `bash -n scripts/spire-aws/summarize-representative-performance.sh scripts/spire-aws/verify-representative-performance-summary.sh`
  - artifact: `artifacts/bash-n-summary-verifier.log`
- `scripts/spire-aws/summarize-representative-performance.sh artifacts/sample-input artifacts/sample-output`
  - artifact: `artifacts/summarize-pooling-latency-sample.log`
  - output: `artifacts/sample-output/representative-pooling-delta-summary.tsv`
- `scripts/spire-aws/verify-representative-performance-summary.sh artifacts/sample-output`
  - artifact: `artifacts/verify-good-pooling-latency-summary.log`
- `scripts/spire-aws/verify-representative-performance-summary.sh artifacts/bad-summary`
  - artifact: `artifacts/verify-bad-p99-delta-summary.log`
  - result: expected exit code `2` after the sample p99 latency delta was set to `0`
- `scripts/spire-aws/preflight-representative-performance.sh`
  - artifact: `artifacts/preflight-representative-performance.log`
- `make -C infra/spire-aws preflight-representative-performance`
  - artifact: `artifacts/make-preflight-representative-performance.log`

## Next

The next non-fault evidence run remains the explicit Graviton `pass-representative-performance` path, which is now gated on representative latency/recall plus pooled-vs-unpooled p50/p95/p99 evidence.
