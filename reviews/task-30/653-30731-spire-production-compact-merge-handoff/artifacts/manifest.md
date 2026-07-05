# Artifact Manifest: SPIRE Production Compact Merge Handoff

- head SHA: `1c0796753dc1224415d5ce0cfd53e2b68799101d`
- packet/topic: `30731-spire-production-compact-merge-handoff`
- timestamp: `2026-05-10T03:30:38-07:00`
- lane: Phase 11 Stage C production executor to Stage D handoff
- fixture: focused production executor Rust tests plus PG18 dry SQL summary
- storage format: state-level validation; ready batches use validated candidate envelopes
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surface: state-level validation; no shared production table scan was executed

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

### `cargo-test-production-executor.log`

- command: `cargo test --no-default-features --features pg18 production_executor_`
- key result lines:
  - `running 11 tests`
  - `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1529 filtered out; finished in 25.11s`

### `git-diff-check.log`

- command: `git diff 95b0fda728b6d5f8462424414f1bfa6d54720fb0 1c0796753dc1224415d5ce0cfd53e2b68799101d --check`
- key result lines:
  - command completed with exit code 0 and no whitespace errors.
