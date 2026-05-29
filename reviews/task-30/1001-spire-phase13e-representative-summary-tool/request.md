# Review Request: SPIRE Representative Summary Tool

Requester: coder1
Date: 2026-05-27
Head SHA: `c641b6d6051a744302350887dd68a65eca9643a3`
Review focus: verify representative performance result summarization for the next AWS pass.

## Summary

This slice adds a deterministic summarizer for the upcoming representative AWS
performance run.

- `scripts/spire-aws/summarize-representative-performance.sh` reads:
  - `suite-results-representative.jsonl`;
  - `suite-results-representative-pooling.jsonl`.
- It writes:
  - `representative-latency-recall-summary.tsv`;
  - `representative-production-profile-summary.tsv`;
  - `representative-pooling-comparison.tsv`.
- `verify-representative-performance-tunneled` now runs the summarizer after
  the representative and pooling suites finish.

No AWS provisioning or EC2 execution was run for this packet.

## Validation

- `bash -n scripts/spire-aws/summarize-representative-performance.sh scripts/spire-aws/bench.sh scripts/spire-aws/run-pass-with-watchdog.sh`
  - passed.
- `scripts/spire-aws/summarize-representative-performance.sh <sample-input> <packet-artifacts>`
  - passed against saved suite JSONL plus a synthetic disabled/enabled pooling
    input derived from prior correctness results.
- `make -C infra/spire-aws preflight`
  - passed after rerunning outside the sandbox for Terraform provider discovery.

## Why This Matters

The next AWS run should answer the highest-priority Phase 13e questions without
manual scraping:

- representative p50/p95/p99 latency;
- representative recall;
- production read profile counters;
- pooled-vs-unpooled socket-open/connect/total timings.
