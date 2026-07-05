# Artifact Manifest

- head SHA: `bbbc22aceb6dcf46555e43ed72db6dfa7226aed0`
- task bucket: `reviews/task-30/960-spire-phase13e-remote-shard-load-commands`
- timestamp: `2026-05-25T16:43:37Z`
- lane: Task 30 Phase 13e remote shard materialization command surface
- fixture: CLI unit validation only; no PostgreSQL fixture
- storage format: not applicable
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `cargo-test-ecaz-cli-corpus-load.log`

- command: `script -q -c "cargo test -p ecaz-cli commands::corpus::load::tests" reviews/task-30/960-spire-phase13e-remote-shard-load-commands/artifacts/cargo-test-ecaz-cli-corpus-load.log`
- result: pass
- key lines:
  - `running 35 tests`
  - `test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 339 filtered out; finished in 0.01s`
