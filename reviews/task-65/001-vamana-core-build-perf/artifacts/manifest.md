# Manifest: Vamana Core Build Perf

- head SHA: `351987249`
- task bucket: `reviews/task-65/001-vamana-core-build-perf/`
- lane: DiskANN build performance / Vamana single-process core
- fixture / storage / rerank: pure Rust build-path compile validation, no corpus fixture
- timestamp: 2026-05-28
- isolated one-index-per-table: not applicable

## Artifacts

- `cargo-check-pg18-lib.log`
  - command: `cargo check -p ecaz --lib --no-default-features --features pg18`
  - key result: `Finished dev profile [unoptimized + debuginfo]`
- `cargo-test-vamana-no-run.log`
  - command: `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::vamana --no-run`
  - key result: `Finished test profile [unoptimized + debuginfo]`; test binary built
- `cargo-test-vamana-run.log`
  - command: `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::vamana -- --nocapture`
  - key result: compile succeeded, then local test binary aborted before executing tests with `dyld: symbol not found in flat namespace '_BufferBlocks'`
