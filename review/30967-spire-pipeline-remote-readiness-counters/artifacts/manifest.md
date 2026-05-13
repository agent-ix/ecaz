# Artifact Manifest: SPIRE Pipeline Remote Readiness Counters

- Head SHA: `fbd8582241f8ee0edddb8ae0e453b303705a58ee`
- Packet/topic: `30967-spire-pipeline-remote-readiness-counters`
- Timestamp: `2026-05-13T04:53:46Z`
- Lane / fixture / storage format / rerank mode: non-live `ecaz-cli` report
  wiring; no PostgreSQL fixture; storage format and rerank mode not exercised.
- Surface isolation: not applicable; no one-index-per-table or shared-table
  runtime fixture was started.

## Artifacts

### `git-diff-check.log`

- Command:
  `script -q -c "git diff --check fbd85822^ fbd85822" review/30967-spire-pipeline-remote-readiness-counters/artifacts/git-diff-check.log`
- Result lines:
  - Command exited successfully with no diff-check findings.

### `cargo-test-ecaz-cli-spire-pipeline.log`

- Command:
  `script -q -c "cargo test -p ecaz-cli spire_pipeline" review/30967-spire-pipeline-remote-readiness-counters/artifacts/cargo-test-ecaz-cli-spire-pipeline.log`
- Result lines:
  - `running 12 tests`
  - `test commands::bench::spire_pipeline::tests::spire_pipeline_renders_endpoint_identity_readiness ... ok`
  - `test commands::bench::spire_pipeline::tests::spire_pipeline_renders_degraded_skip_counters ... ok`
  - `test commands::bench::spire_pipeline::tests::spire_pipeline_sql_uses_public_snapshot_contracts ... ok`
  - `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 308 filtered out; finished in 0.00s`

### `cargo-check-pg18.log`

- Command:
  `script -q -c "cargo check --no-default-features --features pg18" review/30967-spire-pipeline-remote-readiness-counters/artifacts/cargo-check-pg18.log`
- Result lines:
  - `Finished 'dev' profile [unoptimized + debuginfo] target(s)`
  - Existing warning: `ecaz` lib has unused imports in `src/am/mod.rs`.
