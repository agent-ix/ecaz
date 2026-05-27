# Artifact Manifest

- Head SHA: `c641b6d6051a744302350887dd68a65eca9643a3`
- Task bucket: `reviews/task-30/1001-spire-phase13e-representative-summary-tool`
- Timestamp: `2026-05-27T14:59:59Z`
- Lane: local static/tooling validation only; no AWS provisioning
- Fixture: saved prior suite JSONL plus synthetic pooling A/B copy for parser validation
- Storage format: not applicable
- Rerank mode: not applicable
- Surface isolation: no database or AWS runtime used

## Artifacts

- `preflight.log`
  - Command: `make -C infra/spire-aws preflight`
  - Result: passed after rerunning outside the sandbox for Terraform provider discovery.

- `summarize-sample.log`
  - Command: `scripts/spire-aws/summarize-representative-performance.sh reviews/task-30/1001-spire-phase13e-representative-summary-tool/artifacts/sample-summary-input reviews/task-30/1001-spire-phase13e-representative-summary-tool/artifacts`
  - Result: wrote all three summary TSV files.

- `representative-latency-recall-summary.tsv`
  - Result: includes latency rows, recall rows, and `spire-pipeline` latency/recall rows.

- `representative-production-profile-summary.tsv`
  - Result: includes production read profile socket/connect/candidate/heap/merge/total counters.

- `representative-pooling-comparison.tsv`
  - Result: includes disabled/enabled pooling rows from synthetic parser input.

- `sample-summary-input/suite-results-representative.jsonl`
  - Source: copied from prior correctness suite JSONL to validate summarizer shape.

- `sample-summary-input/suite-results-representative-pooling.jsonl`
  - Source: synthetic disabled/enabled pooling row names generated from the same prior JSONL to validate pooling mode extraction.
