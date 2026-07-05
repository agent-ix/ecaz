# Task 120 Packet 013 Artifact Manifest

- head SHA: `7e86b627db7f25287857019d7e8ce34a4e01ad35`
- task bucket: `reviews/task-120/`
- packet path: `reviews/task-120/013-production-read-profile-counters/`
- timestamp: `2026-06-21T20:48:17Z`
- lane / fixture / storage / rerank:
  - lane: `ecaz-cli bench spire-pipeline` reporting
  - fixture: unit/reporting only; no benchmark data loaded
  - storage format: not applicable
  - rerank mode: not applicable
  - isolated one-index-per-table or shared table: not applicable
- remote/distributed: no EC2 resources were provisioned; this packet exposes
  existing production-read metrics that the upcoming Phase 5 distributed packet
  needs to cite.

## Artifacts

### `cargo-fmt-check.log`

- command: `cargo fmt --check`
- key result: exited 0
- note: the log contains stable-rust warnings for unstable rustfmt options
  (`imports_granularity`, `group_imports`), matching the repository's current
  formatter behavior.

### `cargo-test-ecaz-cli-spire-pipeline.log`

- command: `cargo test -p ecaz-cli spire_pipeline`
- key result line:
  `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 388 filtered out; finished in 0.00s`
