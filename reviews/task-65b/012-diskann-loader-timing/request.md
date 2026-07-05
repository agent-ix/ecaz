# Task 65b: DiskANN Loader Timing

## Summary

This checkpoint exposes the last `ec_diskann` build timing snapshot through a stable SQL helper and teaches `ecaz corpus load` to print the same `[loader] ec_diskann_ambuild_timing ...` row shape that the task 65b suite parser already consumes.

The goal is to make the worker/batch sweep from `reviews/task-65b/011-worker-batch-sweep/suite.json` collect durable DiskANN build timing without relying on backend NOTICE delivery.

## Code Under Review

- Commit: `e2b843774a65906e32198c6bfe44b6e0291dc4f4`
- Branch: `task-65b-diskann-parallel-build`
- Changes:
  - Records the last DiskANN ambuild timing snapshot in process-local atomics.
  - Adds `ec_diskann_last_build_timing()` as a stable SQL table function.
  - Logs DiskANN timing from `ecaz corpus load` after `CREATE INDEX`.
  - Keeps the loader warning-only if the helper is unavailable, matching the existing IVF timing behavior.

## Validation

- `cargo fmt --check`
  - Artifact: `artifacts/cargo-fmt-check.log`
  - Result: passed; rustfmt emitted existing stable-channel warnings for unstable import formatting options.
- `cargo check -p ecaz --lib --no-default-features --features pg18`
  - Artifact: `artifacts/cargo-check-ecaz-pg18.log`
  - Result: passed.
- `cargo check -p ecaz-cli`
  - Artifact: `artifacts/cargo-check-ecaz-cli.log`
  - Result: passed; existing `LoadedDistributedPlacementConfig::path` dead-code warning remains.
- `cargo test -p ecaz-cli parses_ec_diskann_build_timing_rows`
  - Artifact: `artifacts/cargo-test-suite-parser.log`
  - Result: passed, 1 test.

## Not Run

No PG socket corpus load or benchmark sweep was run in this checkpoint. The next task 65b step is to run the packet 011 worker/batch suite and use this loader timing row as the durable timing input.
