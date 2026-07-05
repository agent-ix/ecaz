# SPIRE Phase 13e Representative Evidence Gate

This slice hardens the representative AWS performance path around the user's current priority order: pooling, latency, and recall before fault reruns.

## Change

- Added `scripts/spire-aws/verify-representative-performance-summary.sh`.
- Wired `verify-representative-performance-tunneled` to run the verifier immediately after `summarize-representative-performance`.
- Added the `verify-representative-performance-summary` Make target.

The verifier fails closed unless the representative summary directory contains:

- representative latency p50/p95/p99 rows,
- representative recall@k rows,
- production SPIRE pipeline latency plus recall rows,
- production read profile counters including dispatch, socket opens, connect p95, and total p95,
- disabled and enabled pooling rows with socket opens, latency p95, and recall,
- a pooling delta row where socket opens decrease, latency p95 improves, and recall delta is zero.

## Validation

All validation is local harness validation only. No AWS resources were started.

- `bash -n scripts/spire-aws/verify-representative-performance-summary.sh`
  - artifact: `artifacts/bash-n-verifier.log`
- `scripts/spire-aws/verify-representative-performance-summary.sh artifacts/good-summary`
  - artifact: `artifacts/verify-good-summary.log`
  - result: passes with `representative performance summary verified`
- `scripts/spire-aws/verify-representative-performance-summary.sh artifacts/bad-summary`
  - artifact: `artifacts/verify-bad-summary.log`
  - result: expected exit code `2`, rejects flat pooling latency p95
- `make -C infra/spire-aws -n verify-representative-performance-summary ARTIFACT_DIR=artifacts/good-summary`
  - artifact: `artifacts/make-n-verify-summary.log`
- `make -C infra/spire-aws preflight`
  - artifact: `artifacts/make-preflight.log`

## Next

The next AWS command, when explicitly approved, remains:

```bash
SPIRE_AWS_CONFIRM_PROVISION=yes make -C infra/spire-aws ARTIFACT_DIR=reviews/task-30/<new-aws-packet>/artifacts pass-representative-performance
```

That pass now fails if the resulting packet does not prove the prioritized latency, recall, and pooling evidence. Fault/resilience reruns remain deferred.
