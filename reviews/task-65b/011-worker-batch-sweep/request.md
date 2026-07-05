# Task 65b Packet 011: Worker/Batch Sweep Suite

## Summary

This checkpoint adds the execution-ready `ecaz bench suite` config for Task 65b Slice F/H worker-count and batch-size measurement.

The suite covers:

- real10k worker/batch load matrix: `w1/b1`, `w2/b4`, `w4/b8`, `w4/b16`, `w8/b16`, `w8/b32`
- real100k confirmation matrix: `w4/b16`, `w8/b32`
- recall checks on the serial-equivalent and high-worker real10k candidates
- recall checks on both real100k confirmation candidates
- graph digest and storage checks for selected candidates

Each load step sets both sides of the worker contract:

- `PGOPTIONS` with matching `max_parallel_maintenance_workers` and `max_parallel_workers`
- table reloption `parallel_workers=N`
- DiskANN reloption `parallel_build_batch_size=B`
- `capture_parallel_workers=true`, now supported for DiskANN by packet 010

## Why This Advances Task 65b

Task 65b still needs the Slice F/H corpus evidence: real10k build time, real100k build time, recall deltas, reducer/proposal timing, and worker scaling. This packet makes that run reproducible and reviewable through the required FR-038 suite path before touching the local PG socket again.

## Validation

- `ecaz bench suite audit` passed for 22 steps.
- `ecaz bench suite run --dry-run` expanded all commands into a dry-run manifest.
- The focused DiskANN suite parser test passed.

No PostgreSQL corpus build was run in this checkpoint.

## Evidence

- Suite config: `reviews/task-65b/011-worker-batch-sweep/suite.json`
- Manifest: `reviews/task-65b/011-worker-batch-sweep/artifacts/manifest.md`
- Audit log: `reviews/task-65b/011-worker-batch-sweep/artifacts/suite-audit.log`
- Dry-run log: `reviews/task-65b/011-worker-batch-sweep/artifacts/suite-dry-run.log`
- Dry-run manifest: `reviews/task-65b/011-worker-batch-sweep/artifacts/suite-dry-run-manifest.json`
- Parser test log: `reviews/task-65b/011-worker-batch-sweep/artifacts/cargo-test-diskann-suite-parser.log`

## Review Ask

Please review the suite matrix and dry-run expansion before the actual long corpus execution. This packet does not claim the Task 65b final performance gate; it prepares the exact measurement run that should produce that evidence.
