# Artifact Manifest: SPIRE Production Candidate Receive Isolation

- head SHA: `1ad44e4264a9f6fdf8c37ebb534fc91b915611bb`
- packet/topic: `30729-spire-production-candidate-receive-isolation`
- timestamp: `2026-05-10T03:18:38-07:00`
- lane: Phase 11 Stage C production compact-candidate receive
- fixture: PG18 loopback fanout with one ready `rabitq` remote plus failed remotes
- storage format: ready remote uses `storage_format = 'rabitq'`
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surface: isolated loopback test table/index for the ready remote; failure remotes use bad request inputs or shadow search-path functions

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

### `cargo-pgrx-test-receive-isolation.log`

- command: `cargo pgrx test pg18 test_ec_spire_prod_receive_isolates_node_failures`
- key result lines:
  - `test tests::pg_test_ec_spire_prod_receive_isolates_node_failures ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1537 filtered out; finished in 25.01s`

### `git-diff-check.log`

- command: `git diff 1961ec9c5b8c1bed11a152ba6930d80c367e340e 1ad44e4264a9f6fdf8c37ebb534fc91b915611bb --check`
- key result lines:
  - command completed with exit code 0 and no whitespace errors.
