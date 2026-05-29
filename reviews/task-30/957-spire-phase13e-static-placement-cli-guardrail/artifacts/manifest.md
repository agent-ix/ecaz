# Artifact Manifest

- head SHA: `94ecb3eae9a7faa174517f052520484141b71acf`
- task bucket: `reviews/task-30/957-spire-phase13e-static-placement-cli-guardrail`
- timestamp: `2026-05-25T16:24:01Z`
- lane: Task 30 Phase 13e static remote placement / distributed load guardrail
- fixture: CLI unit validation only; no PostgreSQL fixture
- storage format: not applicable
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `cargo-test-ecaz-cli-distributed-placement-config.log`

- command: `script -q -c "cargo test -p ecaz-cli distributed_placement_config" reviews/task-30/957-spire-phase13e-static-placement-cli-guardrail/artifacts/cargo-test-ecaz-cli-distributed-placement-config.log`
- result: pass
- key lines:
  - `running 5 tests`
  - `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 362 filtered out; finished in 0.00s`
