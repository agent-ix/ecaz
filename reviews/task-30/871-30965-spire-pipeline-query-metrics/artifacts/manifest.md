# Artifact Manifest: SPIRE Pipeline Query Metrics

- head SHA: `f86f690ce927a9e9ccb60ab4b0f14cab4cafaa1d`
- packet/topic: `30965-spire-pipeline-query-metrics`
- lane / fixture / storage format / rerank mode: Phase 12.9 CLI benchmark
  harness extension; no PostgreSQL fixture, storage format, or rerank mode was
  exercised.
- isolated one-index-per-table or shared-table surfaces: not applicable; this
  packet validates CLI/report code only.

## Artifacts

### `cargo-test-ecaz-cli-spire-pipeline.log`

- command: `cargo test -p ecaz-cli spire_pipeline`
- timestamp: `2026-05-12 21:42:53-07:00`
- key result lines:
  - `test commands::bench::spire_pipeline::tests::spire_pipeline_query_matrix_requires_fixed_dimensions ... ok`
  - `test commands::bench::spire_pipeline::tests::spire_pipeline_renders_query_metrics_with_recall ... ok`
  - `test cli::tests::cli_parses_spire_pipeline_remote_tuple_transport ... ok`
  - `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 308 filtered out; finished in 0.00s`
- note: the run reported a pre-existing `ecaz` library unused-import warning in
  `src/am/mod.rs`; the CLI tests passed.

### `cargo-check-pg18.log`

- command: `cargo check --no-default-features --features pg18`
- timestamp: `2026-05-12 21:42:56-07:00`
- key result lines:
  - `Finished 'dev' profile [unoptimized + debuginfo] target(s) in 0.15s`
  - command exited with status `0`
- note: the run reported the same pre-existing unused-import warning in
  `src/am/mod.rs`.

### `git-diff-check.log`

- command: `git diff --check f86f690c^ f86f690c`
- timestamp: `2026-05-12 21:43:00-07:00`
- key result lines:
  - command exited with status `0`
