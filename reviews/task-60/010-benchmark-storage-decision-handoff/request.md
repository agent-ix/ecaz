# Review Request: Benchmark Storage Decision Handoff

## Scope

This checkpoint updates the Task 60 benchmark manifest to align the manual 1M shipping decision with the structured result rows emitted by the suite report.

The manifest now instructs the benchmark host runner to calculate:

```text
1 - (rabitq size_bytes / pq_fastscan size_bytes)
```

from `storage_index` rows in `artifacts/results-report.jsonl`, filtered to `access method=ec_diskann`, while retaining the storage logs as the durable source artifacts.

No suite comparison gate or benchmark execution is added here.

## Validation

Artifact: `reviews/task-60/010-benchmark-storage-decision-handoff/artifacts/suite-audit.log`

Result: Task 60 suite audit passed with 24 steps.

## Remaining Task 60 Gate

The benchmark host still needs to execute the full 100k/1M suite and record the measured recall, latency, storage, and shipping decision.
