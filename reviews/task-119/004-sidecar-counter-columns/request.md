# Task 119: Sidecar Rerank Counter Columns

## Summary

This checkpoint adds explicit counter columns to `ecaz bench sidecar-rerank` so
Task 119 measurements can report frontier, reranked, sidecar/source-read, and
emitted-row counts per rerank representation.

Previously, the sidecar matrix had `candidate_count_*` and latency/storage
fields, but review had to infer several Task 119 acceptance counters. The new
columns make those counters first-class in the suite JSONL:

- `frontier_p50`, `frontier_p95`
- `reranked_p50`, `reranked_p95`
- `sidecar_reads_p50`, `sidecar_reads_p95`
- `heap_source_reads_p50`, `heap_source_reads_p95`
- `emitted_p50`, `emitted_p95`

For `read_mode=free`, sidecar/source reads are reported as `0`. For DB-backed
read modes, sidecar/source reads equal the fetched sidecar rows per query.

## Code

- Commit: `4614d4c0ef8dbf4b8072aaa60773325f4a74b7f5`
- Changed: `crates/ecaz-cli/src/commands/bench/sidecar_rerank.rs`

## Validation

- `cargo test -p ecaz-cli sidecar -- --nocapture`
  - Log: `reviews/task-119/004-sidecar-counter-columns/artifacts/cargo-test-ecaz-cli-sidecar.log`
  - Result: passed, 10 tests.
- `cargo check -p ecaz-cli`
  - Log: `reviews/task-119/004-sidecar-counter-columns/artifacts/cargo-check-ecaz-cli.log`
  - Result: passed, with the existing `LoadedDistributedPlacementConfig::path`
    dead-code warning.
- `git diff --check -- crates/ecaz-cli/src/commands/bench/sidecar_rerank.rs reviews/task-119/004-sidecar-counter-columns`
  - Result: passed.

`cargo fmt --check` was run and still reports unrelated pre-existing formatting
drift in `src/am/ec_hnsw/scan.rs`; this checkpoint does not touch that file.

## Next Measurement

The next Task 119 benchmark packet should rerun the sidecar matrix with this
binary so every row includes explicit counter fields. A DB-backed read-mode run
is still needed for production-style heap/source-read latency and read counts.
