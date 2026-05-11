# Artifact Manifest

Packet: `30843-spire-dml-frontdoor-plan`

Head SHA: `55bfdcc20944c4ca63794eca43aa644cce2b7829`

Timestamp: `2026-05-11 12:19 America/Los_Angeles`

## Artifacts

### `cargo-test-coordinator-dml-frontdoor-plan-lib.log`

- Command: `script -q -e -c "cargo test coordinator_dml_frontdoor_plan --lib" review/30843-spire-dml-frontdoor-plan/artifacts/cargo-test-coordinator-dml-frontdoor-plan-lib.log`
- Lane / fixture: Rust-side PG18 `pg_test` lane, focused coordinator DML
  front-door plan SQL surface test.
- Storage format / rerank mode: not a recall/rerank benchmark.
- Cluster layout: pgrx PG18 test cluster.
- Isolated one-index-per-table or shared-table surface: no table/index fixture;
  SQL status surface only.
- Result:
  - `test tests::pg_test_ec_spire_coordinator_dml_frontdoor_plan_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1645 filtered out`

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30843-spire-dml-frontdoor-plan/artifacts/cargo-fmt-check.log`
- Lane / fixture: formatter check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass with the repo's existing stable-rustfmt warnings.

### `git-diff-check.log`

- Command: `script -q -e -c "git diff --check" review/30843-spire-dml-frontdoor-plan/artifacts/git-diff-check.log`
- Lane / fixture: whitespace check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass.
