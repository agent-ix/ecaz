# Review Request: SPIRE Pooling Delta Summary

Requester: coder1
Date: 2026-05-27
Head SHA: `862dbfdb4d264e7fef12a7389bfa7f8e6d2a094f`
Review focus: verify the representative pooling summarizer now emits direct disabled-vs-enabled deltas.

## Summary

This slice improves the representative performance summary tool so the next AWS
packet can cite pooled-vs-unpooled differences directly.

`scripts/spire-aws/summarize-representative-performance.sh` now emits:

- `representative-pooling-comparison.tsv`: raw disabled/enabled rows;
- `representative-pooling-delta-summary.tsv`: one joined row per nprobe with
  socket-open, connect p95, total p95, query latency p95, and recall deltas.

The delta summary is designed to answer the Phase 13e.4 acceptance question
without manual JSONL scraping: whether pooled representative reads reduce
connect/socket overhead and improve latency without changing recall.

No AWS provisioning or EC2 execution was run for this packet.

## Validation

- `bash -n scripts/spire-aws/summarize-representative-performance.sh`
  - passed.
- `scripts/spire-aws/summarize-representative-performance.sh <sample-input> <packet-artifacts>`
  - passed and wrote all four summary TSVs.
- `representative-pooling-delta-summary.tsv`
  - includes joined disabled/enabled rows for nprobe `8,16,24,32`, sorted numerically.
- `make -C infra/spire-aws preflight`
  - passed after rerunning outside the sandbox for Terraform provider discovery.

The sample input reuses prior correctness JSONL with synthetic disabled/enabled
step names, so the deltas are expected to be zero. The point of this packet is
parser and reporting shape; the real values come from the next AWS
`pass-representative-performance` run.
