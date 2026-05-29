# Review Request: DiskANN RaBitQ Benchmark Host Handoff

## Scope

This checkpoint cleans up the Task 60 benchmark packet for execution on the benchmark host. It does not run the 100k/1M benchmarks locally.

Changes under review:

- Updates `benchmarks/task60-diskann-rabitq-format/manifest.md` so dry-run, full-run, and report commands all write durable packet-local logs with `--log-file`.
- Keeps the full benchmark execution on the benchmark host, as requested.
- Adds a final evidence checklist for the benchmark packet:
  - suite run/report logs
  - suite manifest
  - structured run/report result rows
  - recall, latency, storage, and host precheck logs
- Records the 1M shipping decision as a manual packet calculation from storage logs, not as an automated suite comparison.
- Regenerates the dry-run suite manifest and stores `suite-dry-run.log`.

## Validation

Artifacts are under `reviews/task-60/008-benchmark-host-handoff/artifacts/`.

- `suite-audit.log`: `ecaz bench suite audit` passed with 24 steps.
- `benchmarks/task60-diskann-rabitq-format/artifacts/suite-dry-run.log`: dry-run expanded the 24-step suite and wrote the suite manifest.

## Remaining Task 60 Gate

Task 60 still requires benchmark-host execution of the checked-in suite. Completion requires measured 100k/1M recall, latency, and storage evidence, plus the recorded 1M shipping decision.
