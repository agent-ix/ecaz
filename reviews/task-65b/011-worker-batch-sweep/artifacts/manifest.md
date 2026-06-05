# Task 65b Packet 011 Artifact Manifest

- head SHA: `b6779ab07884c441bec222b31f02ae658e493749`
- task bucket: `reviews/task-65b/011-worker-batch-sweep`
- timestamp: `2026-06-05T03:31:52Z`
- lane: m5-local PG18 suite configuration validation
- scope: Slice F/H worker-count and batch-size sweep harness
- fixture: DBpedia real10k and real100k
- profile: `ec_diskann`
- storage format: `pq_fastscan`
- graph params: `graph_degree=32`, `build_list_size=100`, `alpha=1.2`
- index/table isolation: one index per table via unique `task65b_sweep_*` prefixes

## Suite Matrix

The checked-in suite config is `reviews/task-65b/011-worker-batch-sweep/suite.json`.

Real10k load matrix:

- `workers=1`, `parallel_build_batch_size=1`
- `workers=2`, `parallel_build_batch_size=4`
- `workers=4`, `parallel_build_batch_size=8`
- `workers=4`, `parallel_build_batch_size=16`
- `workers=8`, `parallel_build_batch_size=16`
- `workers=8`, `parallel_build_batch_size=32`

Real100k confirmation matrix:

- `workers=4`, `parallel_build_batch_size=16`
- `workers=8`, `parallel_build_batch_size=32`

Each load step sets:

- `capture_parallel_workers=true`
- `PGOPTIONS="-c max_parallel_maintenance_workers=N -c max_parallel_workers=N"`
- table reloption `parallel_workers=N`
- DiskANN reloption `parallel_build_batch_size=B`

The suite also includes recall, graph digest, and storage checks for selected real10k and real100k candidates.

## Validation Commands

- `./target/debug/ecaz bench suite audit --config reviews/task-65b/011-worker-batch-sweep/suite.json > reviews/task-65b/011-worker-batch-sweep/artifacts/suite-audit.log 2>&1`
  - exited 0
  - `audit passed: 22 steps`
- `./target/debug/ecaz bench suite run --config reviews/task-65b/011-worker-batch-sweep/suite.json --dry-run --manifest-output reviews/task-65b/011-worker-batch-sweep/artifacts/suite-dry-run-manifest.json > reviews/task-65b/011-worker-batch-sweep/artifacts/suite-dry-run.log 2>&1`
  - exited 0
  - wrote dry-run manifest and expanded all 22 commands
- `cargo test -p ecaz-cli suite::tests::parses_ec_diskann_build_timing_rows > reviews/task-65b/011-worker-batch-sweep/artifacts/cargo-test-diskann-suite-parser.log 2>&1`
  - exited 0
  - `1 passed; 0 failed; 404 filtered out`

## Artifact Summary

- `suite-audit.log`: suite shape and input artifact audit.
- `suite-dry-run.log`: expanded commands, including `PGOPTIONS`, `parallel_workers`, and `parallel_build_batch_size`.
- `suite-dry-run-manifest.json`: normalized dry-run manifest for all 22 steps.
- `cargo-test-diskann-suite-parser.log`: focused parser evidence for DiskANN build timing rows.

## Notes

This packet intentionally does not run the PostgreSQL corpus build. It avoids another local socket approval gate while landing the exact FR-038 suite config needed for the next measurement execution. The final Task 65b timing, recall, and scaling gates remain unclaimed until this suite is run against PG18 and the resulting artifacts are reviewed.
