# Artifact Manifest: SPIRE Tuple Transport Benchmark Knob

- head SHA: `0aa621526f015c1dbf556a174e42c12e1e6c608a`
- packet/topic: `30964-spire-tuple-transport-bench-knob`
- lane / fixture / storage format / rerank mode: Phase 12.2 typed-vs-JSON
  measurement groundwork; no PostgreSQL fixture, storage format, or rerank mode
  was exercised.
- isolated one-index-per-table or shared-table surfaces: not applicable; this
  packet validates a GUC/CLI benchmark control only.

## Artifacts

### `cargo-test-ecaz-remote-tuple-transport.log`

- command: `cargo test -p ecaz --no-default-features --features pg18 remote_tuple_transport --lib`
- timestamp: `2026-05-12 21:35:04-07:00`
- key result lines:
  - `test am::ec_spire::remote_tuple_transport_tests::remote_tuple_transport_session_override_keeps_capability_gate ... ok`
  - `test am::ec_spire::remote_tuple_transport_tests::remote_tuple_transport_auto_uses_endpoint_default ... ok`
  - `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1694 filtered out; finished in 0.00s`

### `cargo-test-ecaz-cli-spire-pipeline.log`

- command: `cargo test -p ecaz-cli spire_pipeline`
- timestamp: `2026-05-12 21:35:00-07:00`
- key result lines:
  - `test commands::bench::spire_pipeline::tests::spire_pipeline_reports_remote_tuple_transport_override ... ok`
  - `test cli::tests::cli_parses_spire_pipeline_remote_tuple_transport ... ok`
  - `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 308 filtered out; finished in 0.00s`
- note: the run reported a pre-existing `ecaz` library unused-import warning in
  `src/am/mod.rs`; the CLI tests passed.

### `cargo-check-pg18.log`

- command: `cargo check --no-default-features --features pg18`
- timestamp: `2026-05-12 21:35:08-07:00`
- key result lines:
  - `Finished 'dev' profile [unoptimized + debuginfo] target(s) in 0.12s`
  - command exited with status `0`
- note: the run reported the same pre-existing unused-import warning in
  `src/am/mod.rs`.

### `git-diff-check.log`

- command: `git diff --check 0aa62152^ 0aa62152`
- timestamp: `2026-05-12 21:35:13-07:00`
- key result lines:
  - command exited with status `0`
