# Artifact Manifest

Packet: `30846-spire-pk-select-duplicate-guard`

Head SHA: `cb01f79d3210007400bbce7c06af4ad2c75787b0`

Timestamp: `2026-05-11 12:42 America/Los_Angeles`

## Artifacts

### `cargo-test-forward-coordinator-select-lib.log`

- Command: `script -q -e -c "cargo test forward_coordinator_select --lib" review/30846-spire-pk-select-duplicate-guard/artifacts/cargo-test-forward-coordinator-select-lib.log`
- Lane / fixture: Rust-side PG18 `pg_test` lane, focused coordinator PK SELECT
  helper tests.
- Storage format / rerank mode: not a recall/rerank benchmark.
- Cluster layout: pgrx PG18 test cluster with loopback libpq connection for
  the remote-owned case.
- Isolated one-index-per-table or shared-table surface: isolated test tables.
- Result:
  - `test tests::pg_test_ec_spire_forward_coordinator_select_local_sql ... ok`
  - `test tests::pg_test_ec_spire_forward_coordinator_select_tuple_payload_sql ... ok`
  - `test tests::pg_test_ec_spire_forward_coordinator_select_rejects_multirow_sql - should panic ... ok`
  - `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1646 filtered out`

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30846-spire-pk-select-duplicate-guard/artifacts/cargo-fmt-check.log`
- Lane / fixture: formatter check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass with the repo's existing stable-rustfmt warnings.

### `git-diff-check.log`

- Command: `script -q -e -c "git diff --check" review/30846-spire-pk-select-duplicate-guard/artifacts/git-diff-check.log`
- Lane / fixture: whitespace check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass.
