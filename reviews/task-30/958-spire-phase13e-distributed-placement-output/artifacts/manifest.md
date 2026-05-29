# Artifact Manifest

- head SHA: `13e3bc57b2ce2af7b9e39c2a83a1f3a2171349ef`
- task bucket: `reviews/task-30/958-spire-phase13e-distributed-placement-output`
- timestamp: `2026-05-25T16:32:06Z`
- lane: Task 30 Phase 13e static remote placement / distributed corpus output
- fixture: CLI unit validation only; no PostgreSQL fixture
- storage format: not applicable
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `cargo-test-ecaz-cli-distributed-placement.log`

- command: `script -q -c "cargo test -p ecaz-cli distributed_placement" reviews/task-30/958-spire-phase13e-distributed-placement-output/artifacts/cargo-test-ecaz-cli-distributed-placement.log`
- result: pass
- key lines:
  - `running 7 tests`
  - `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 362 filtered out; finished in 0.00s`
