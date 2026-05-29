# Review Request: DiskANN RaBitQ Benchmark Suite

- task: `plan/tasks/60-ec-diskann-rabitq-storage-format.md`
- branch: `task/60-diskann-rabitq`
- topic: `diskann-rabitq-benchmark-suite`
- code checkpoint: benchmark scaffold only

## What Changed

- Added `benchmarks/task60-diskann-rabitq-format/suite.json`.
- Added `benchmarks/task60-diskann-rabitq-format/manifest.md`.
- The suite uses `ecaz bench suite` for paired `pq_fastscan` and `rabitq`
  DiskANN loads, recall sweeps, latency sweeps, storage checks, and explain
  captures at 100k and 1M.

## Review Focus

- Does the suite cover the Task 60 acceptance matrix without creating a custom
  benchmark runner?
- Are the format labels and prefixes clear enough for downstream report
  extraction?
- Are host parity and cache-state fields captured in the durable packet output?

## Validation

- `cargo run -p ecaz-cli -- bench suite run --config benchmarks/task60-diskann-rabitq-format/suite.json --dry-run --manifest-output benchmarks/task60-diskann-rabitq-format/artifacts/suite-manifest.json`
- Dry-run log: `reviews/task-60/002-diskann-rabitq-benchmark-suite/artifacts/suite-dry-run.log`
- Generated suite manifest: `benchmarks/task60-diskann-rabitq-format/artifacts/suite-manifest.json`

The full benchmark run is intentionally not included in this review request;
it requires the PG18 benchmark host and staged DBpedia fixtures.
