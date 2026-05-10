# Artifact Manifest: SPIRE Production Candidate Receive State

- head SHA: `8310e767d3b7f0241daca0aa79d06d66793ef2f2`
- packet/topic: `30728-spire-production-candidate-receive-state`
- timestamp: `2026-05-10T03:06:36-07:00`
- lane: Phase 11 Stage C production executor
- fixture: focused Rust executor-state tests plus PG18 dry SQL summary
- storage format: state-only validation; PG dry path uses the existing scratch test fixture
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surface: state-only validation; no shared production table scan was executed

## Artifacts

### `cargo-fmt-check.log`

- command: `cargo fmt --check`
- key result lines:
  - `cargo fmt --check` completed with exit code 0.
  - rustfmt emitted the repository's recurring stable-channel warnings for unstable import options.

### `cargo-check-pg18.log`

- command: `cargo check --no-default-features --features pg18`
- key result lines:
  - `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.12s`

### `cargo-test-production-executor-state.log`

- command: `cargo test --no-default-features --features pg18 production_executor_state_`
- key result lines:
  - `running 9 tests`
  - `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1528 filtered out; finished in 26.34s`

### `git-diff-check.log`

- command: `git diff a73b5d9c520552ee7009bd9c4de9ba5fedea791f 8310e767d3b7f0241daca0aa79d06d66793ef2f2 --check`
- key result lines:
  - command completed with exit code 0 and no whitespace errors.
