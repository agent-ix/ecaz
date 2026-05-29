# Review Request: SPIRE Representative Priority Suite

## Scope

This packet covers commit `287aa038ac97c4902cd7162e81e477a3ff33911d`, which
narrows the next SPIRE AWS performance pass to the highest-priority evidence:
representative latency, representative recall, production read profile metrics,
and pooling A/B.

Fault and resilience runs remain deferred.

## Change Summary

- Added `scripts/spire-aws/suite-representative-priority.json`.
- Added the `representative-priority` tier to `scripts/spire-aws/bench.sh`.
- Changed `verify-representative-performance-tunneled` to run:
  - `load-representative`
  - `register-representative`
  - `smoke-representative`
  - `bench-representative-priority`
  - `bench-representative-pooling`
  - `summarize-representative-performance`
- Left the existing full representative lane intact for later transport/fault
  work.
- Updated the summarizer to prefer
  `suite-results-representative-priority.jsonl`, with fallback to the older
  `suite-results-representative.jsonl`.

## Validation

No AWS provisioning, Terraform apply, EC2 start, PostgreSQL cluster, or SSM
tunnel was used for this packet.

- `jq empty scripts/spire-aws/suite-representative-priority.json scripts/spire-aws/suite-representative-pooling.json`
  - artifact: `artifacts/jq-priority-suites.log`
  - result: exit 0
- `bash -n scripts/spire-aws/bench.sh scripts/spire-aws/summarize-representative-performance.sh`
  - artifact: `artifacts/bash-n-priority-scripts.log`
  - result: exit 0
- `make -C infra/spire-aws -n bench-representative-priority TOPOLOGY=/dev/null ARTIFACT_DIR=reviews/task-30/1004-spire-phase13e-representative-priority-suite/artifacts`
  - artifact: `artifacts/make-n-bench-priority.log`
  - result: exit 0, prints the `bench.sh representative-priority ...` command
- `scripts/spire-aws/summarize-representative-performance.sh artifacts/sample-input artifacts/sample-output`
  - artifact: `artifacts/summarize-priority-sample.log`
  - result: exit 0

The sample summary output confirms the new priority results filename feeds:

- `artifacts/sample-output/representative-latency-recall-summary.tsv`
- `artifacts/sample-output/representative-production-profile-summary.tsv`
- `artifacts/sample-output/representative-pooling-comparison.tsv`
- `artifacts/sample-output/representative-pooling-delta-summary.tsv`

## Next AWS Command When Explicitly Approved

```sh
SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 \
SPIRE_AWS_CONFIRM_PROVISION=yes \
make -C infra/spire-aws \
  ARTIFACT_DIR=reviews/task-30/<next-packet>/artifacts \
  pass-representative-performance
```

That target now excludes fault and transport-sweep work from the first
performance pass.
