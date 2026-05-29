# Review Request: DiskANN RaBitQ Benchmark Suite Audit

## Scope

This checkpoint fixes the Task 60 benchmark suite dependency graph before the full 100k/1M measurement run. The previous suite referenced pre-existing 1M staged files under `/var/lib/pgsql/18/datasets/staged-1m/`; `ecaz bench suite audit` could not prove those inputs came from the checked-in suite.

Changes under review:

- Adds an explicit `corpus-prepare` step for the 1M anchor profile, `ec_real_ann_benchmarks_anchor`.
- Moves the 1M `pq_fastscan` and `rabitq` load inputs to the suite-owned staging directory, `/var/lib/pgsql/18/datasets/staged-task60-diskann-rabitq/`.
- Updates the benchmark manifest to document that `ecaz bench suite audit` now verifies fetch -> prepare -> load coverage for the 1M rows.
- Regenerates the dry-run suite manifest.

## Validation

Artifacts are under `reviews/task-60/005-diskann-rabitq-benchmark-suite-audit/artifacts/`.

- `suite-audit.log`: `audit passed: 24 steps`
- `suite-dry-run.log`: dry-run emitted all 24 steps and regenerated `benchmarks/task60-diskann-rabitq-format/artifacts/suite-manifest.json`

No full benchmark execution was run in this checkpoint. This is a suite readiness fix only; Task 60 still needs real 100k/1M recall, latency, and storage measurements on the benchmark host.
