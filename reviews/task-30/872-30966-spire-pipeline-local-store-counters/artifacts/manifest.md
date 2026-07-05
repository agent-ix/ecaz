# Artifact Manifest: SPIRE Pipeline Local Store Counters

- head SHA: `aa8e997a5a557d3873d0115883a51d0a32c8b805`
- packet/topic: `30966-spire-pipeline-local-store-counters`
- lane / fixture / storage format / rerank mode: Phase 12.9 CLI counter
  capture extension; no PostgreSQL fixture, storage format, or rerank mode was
  exercised.
- isolated one-index-per-table or shared-table surfaces: not applicable; this
  packet validates CLI/report code only.

## Artifacts

### `cargo-test-ecaz-cli-spire-pipeline.log`

- command: `cargo test -p ecaz-cli spire_pipeline`
- timestamp: `2026-05-12 21:47:11-07:00`
- key result lines:
  - `test commands::bench::spire_pipeline::tests::spire_pipeline_renders_local_store_overlap_counters ... ok`
  - `test cli::tests::cli_parses_spire_pipeline_remote_tuple_transport ... ok`
  - `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 308 filtered out; finished in 0.00s`
- note: the run reported a pre-existing `ecaz` library unused-import warning in
  `src/am/mod.rs`; the CLI tests passed.

### `cargo-check-pg18.log`

- command: `cargo check --no-default-features --features pg18`
- timestamp: `2026-05-12 21:47:15-07:00`
- key result lines:
  - `Finished 'dev' profile [unoptimized + debuginfo] target(s) in 0.12s`
  - command exited with status `0`
- note: the run reported the same pre-existing unused-import warning in
  `src/am/mod.rs`.

### `git-diff-check.log`

- command: `git diff --check aa8e997a^ aa8e997a`
- timestamp: `2026-05-12 21:47:19-07:00`
- key result lines:
  - command exited with status `0`
