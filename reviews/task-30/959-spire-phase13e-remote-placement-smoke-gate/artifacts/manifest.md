# Artifact Manifest

- head SHA: `7b5ea98b9fde3e65bfc4171fbd54367bc86dfbbb`
- task bucket: `reviews/task-30/959-spire-phase13e-remote-placement-smoke-gate`
- timestamp: `2026-05-25T16:35:49Z`
- lane: Task 30 Phase 13e distributed placement smoke gate
- fixture: CLI unit validation only; no PostgreSQL fixture
- storage format: not applicable
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `cargo-test-ecaz-cli-spire-pipeline.log`

- command: `script -q -c "cargo test -p ecaz-cli spire_pipeline" reviews/task-30/959-spire-phase13e-remote-placement-smoke-gate/artifacts/cargo-test-ecaz-cli-spire-pipeline.log`
- result: pass
- key lines:
  - `running 16 tests`
  - `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 355 filtered out; finished in 0.00s`
