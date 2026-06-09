# Task 87 Packet 010 Artifact Manifest

- head SHA: `6f830a21aec44422fe068d40afb988fcc1025c76`
- task bucket: `reviews/task-87/`
- packet path: `reviews/task-87/010-measurement-toggles/`
- timestamp: `2026-06-08T20:10:00Z`
- scope: default-on AM CandidateBatch measurement toggles plus suite session-GUC passthrough
- lane / fixture / storage format / rerank mode: unit tests only; no corpus lane; no storage format or rerank mode
- isolated one-index-per-table vs shared-table surfaces: not applicable

## Artifacts

### `cargo-test-hnsw-scan.log`

- command: `cargo test --lib am::ec_hnsw::scan::tests --no-default-features --features pg18`
- result: passed
- key cited lines:
  - `running 74 tests`
  - `test am::ec_hnsw::scan::tests::turboquant_full_lut_payload_batch_matches_scalar_and_caches_scores ... ok`
  - `test result: ok. 74 passed; 0 failed`

### `cargo-test-spire-quantizer.log`

- command: `cargo test --lib am::ec_spire::quantizer::tests --no-default-features --features pg18`
- result: passed
- key cited lines:
  - `running 15 tests`
  - `test am::ec_spire::quantizer::tests::assignment_scorer_batch_matches_scalar_scores ... ok`
  - `test result: ok. 15 passed; 0 failed`

### `cargo-test-cli-suite.log`

- command: `cargo test -p ecaz-cli commands::bench::suite`
- result: passed
- key cited lines:
  - `running 41 tests`
  - `test commands::bench::suite::tests::expands_recall_with_defaults ... ok`
  - `test commands::bench::suite::tests::expands_spire_pipeline_with_production_profile ... ok`
  - `test result: ok. 41 passed; 0 failed`

### `cargo-test-cli-latency-session-gucs.log`

- command: `cargo test -p ecaz-cli commands::bench::latency::tests::parse_session_gucs`
- result: passed
- key cited lines:
  - `running 2 tests`
  - `test commands::bench::latency::tests::parse_session_gucs_accepts_qualified_names ... ok`
  - `test commands::bench::latency::tests::parse_session_gucs_rejects_malformed_entries ... ok`
  - `test result: ok. 2 passed; 0 failed`
