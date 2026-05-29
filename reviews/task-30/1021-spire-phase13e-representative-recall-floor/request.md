# Review Request: SPIRE Representative Recall Floor

Task: `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`

Code commit: `70074aa89`

## Summary

This checkpoint tightens the next Phase 13e AWS representative performance
gate around the user-prioritized recall and pooling-latency proof:

- adds `recall@k >= 0.95` thresholds at `nprobe=32` to the representative
  priority suite for standalone recall and production-read recall;
- adds matching `recall@k >= 0.95` thresholds to both pooled and unpooled
  representative pooling suite rows;
- makes the summary verifier fail closed if those suite thresholds are missing
  or if the summary rows do not meet the recall floor;
- fixes the preflight negative summary checks so they copy suite JSON into the
  bad-summary fixtures, proving the verifier rejects the bad metric rather than
  failing for missing suite files;
- adds a local negative self-check for recall below the representative floor.

No AWS was started. This is local hardening before the next explicit Graviton
representative run.

## Validation

- `jq empty scripts/spire-aws/suite-representative-priority.json scripts/spire-aws/suite-representative-pooling.json`
- `bash -n scripts/spire-aws/preflight-representative-performance.sh scripts/spire-aws/verify-representative-performance-summary.sh`
- `bash scripts/spire-aws/preflight-representative-performance.sh`
- `target/release/ecaz bench suite --config scripts/spire-aws/suite-representative-priority.json --dry-run`
- `target/release/ecaz bench suite --config scripts/spire-aws/suite-representative-pooling.json --dry-run`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-show-stat.log`
- `artifacts/jq-suite-parse.log`
- `artifacts/bash-syntax.log`
- `artifacts/preflight-representative-performance.log`
- `artifacts/suite-priority-dry-run.log`
- `artifacts/suite-pooling-dry-run.log`
