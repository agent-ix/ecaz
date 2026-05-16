# Artifact Manifest: 30732 SPIRE Production Remote Index Resolution

Head SHA: `d626c2e664d8dfee4875385cf5fd4d1f61f9efd2`
Code commit under review: `b2c901d9261382ed899942f650a2ba2c5d1008ee`
Packet: `review/30732-spire-production-remote-index-resolution`
Lane: Phase 11 Stage C production compact candidate receive
Fixture: local PG18 pgrx receive-isolation test
Storage format: RaBitQ remote-serving SPIRE index
Rerank mode: compact candidate receive only; no final heap rerank
Surface isolation: shared-table test fixture with per-node simulated receive
outcomes; no AWS/RDS scale claim
Timestamp: 2026-05-10 03:40-03:42 America/Los_Angeles

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Key result: `COMMAND_EXIT_CODE="0"`
- Note: existing stable-rustfmt warnings for unstable
  `imports_granularity` / `group_imports` settings are present.

### `cargo-check-pg18.log`

- Command: `cargo check --no-default-features --features pg18`
- Key result: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.37s`
- Exit: `COMMAND_EXIT_CODE="0"`

### `cargo-pgrx-test-pg18-receive-isolation.log`

- Command:
  `cargo pgrx test pg18 test_ec_spire_prod_receive_isolates_node_failures`
- Key result:
  `test tests::pg_test_ec_spire_prod_receive_isolates_node_failures ... ok`
- Key result:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1539 filtered out; finished in 25.33s`
- Exit: `COMMAND_EXIT_CODE="0"`

### `git-diff-check.log`

- Command: `git diff --check`
- Key result: no whitespace diagnostics
- Exit: `COMMAND_EXIT_CODE="0"`
