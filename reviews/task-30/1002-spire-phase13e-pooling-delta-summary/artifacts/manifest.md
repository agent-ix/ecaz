# Artifact Manifest

- Head SHA: `862dbfdb4d264e7fef12a7389bfa7f8e6d2a094f`
- Task bucket: `reviews/task-30/1002-spire-phase13e-pooling-delta-summary`
- Timestamp: `2026-05-27T15:09:13Z`
- Lane: local static/tooling validation only; no AWS provisioning
- Fixture: saved prior suite JSONL plus synthetic pooling A/B copy for parser validation
- Storage format: not applicable
- Rerank mode: not applicable
- Surface isolation: no database or AWS runtime used

## Artifacts

- `preflight.log`
  - Command: `make -C infra/spire-aws preflight`
  - Result: passed after rerunning outside the sandbox for Terraform provider discovery.

- `bash-n.log`
  - Command: `bash -n scripts/spire-aws/summarize-representative-performance.sh`
  - Result: passed.

- `summarize-sample.log`
  - Command: `scripts/spire-aws/summarize-representative-performance.sh reviews/task-30/1002-spire-phase13e-pooling-delta-summary/artifacts/sample-summary-input reviews/task-30/1002-spire-phase13e-pooling-delta-summary/artifacts`
  - Result: wrote all four summary TSV files.

- `representative-latency-recall-summary.tsv`
  - Result: includes latency rows, recall rows, and `spire-pipeline` latency/recall rows.

- `representative-production-profile-summary.tsv`
  - Result: includes production read profile socket/connect/candidate/heap/merge/total counters.

- `representative-pooling-comparison.tsv`
  - Result: includes raw disabled/enabled pooling rows.

- `representative-pooling-delta-summary.tsv`
  - Result: includes joined disabled/enabled rows by nprobe, with socket-open,
    connect p95, total p95, query latency p95, and recall deltas.

- `sample-summary-input/suite-results-representative.jsonl`
  - Source: copied from prior correctness suite JSONL to validate summarizer shape.

- `sample-summary-input/suite-results-representative-pooling.jsonl`
  - Source: synthetic disabled/enabled pooling row names generated from the same prior JSONL to validate pooling mode extraction and delta output.
