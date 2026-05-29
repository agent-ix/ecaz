# SPIRE Phase 13e Endpoint Identity Profile Evidence

This slice strengthens the next representative AWS performance packet without starting AWS. The priority remains pooling, latency, and recall; this adds endpoint identity profile evidence to that same path so the pooling packet also records the identity validation surface.

## Change

- `ecaz bench spire-pipeline` production read profile aggregation now emits:
  - `endpoint_identity_p50`,
  - `endpoint_identity_p95`,
  - `endpoint_identity_query_sum`.
- `scripts/spire-aws/summarize-representative-performance.sh` carries those fields into:
  - `representative-production-profile-summary.tsv`,
  - `representative-pooling-comparison.tsv`,
  - `representative-pooling-delta-summary.tsv`.
- `scripts/spire-aws/verify-representative-performance-summary.sh` now fails the representative pass unless endpoint identity profile fields are present and both disabled and enabled pooling rows show positive endpoint identity query counts.

## Validation

All validation is local. No AWS resources were started.

- `cargo test -p ecaz-cli spire_pipeline_renders_production_read_profile`
  - artifact: `artifacts/cargo-test-spire-pipeline-profile.log`
- `bash -n scripts/spire-aws/summarize-representative-performance.sh scripts/spire-aws/verify-representative-performance-summary.sh`
  - artifact: `artifacts/bash-n-summary-verifier.log`
- `scripts/spire-aws/summarize-representative-performance.sh artifacts/sample-input artifacts/sample-output`
  - artifact: `artifacts/summarize-endpoint-identity-sample.log`
- `scripts/spire-aws/verify-representative-performance-summary.sh artifacts/sample-output`
  - artifact: `artifacts/verify-good-endpoint-summary.log`
- `scripts/spire-aws/verify-representative-performance-summary.sh artifacts/bad-summary`
  - artifact: `artifacts/verify-bad-endpoint-summary.log`
  - result: expected exit code `2` when enabled endpoint identity query count is zero
- `make -C infra/spire-aws preflight`
  - artifact: `artifacts/make-preflight.log`

## Next

The remaining Phase 13e blocker is still the AWS representative performance packet on the established Graviton lane. That pass now has to prove latency p50/p95/p99, recall, pooling socket and latency improvement, zero recall regression, and endpoint identity profiling before it can count.
