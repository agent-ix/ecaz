# Artifact Manifest

Packet: `30839-spire-coordinator-delete-forwarding`

Head SHA: `c14774cf86cf886a7c942c51c1f733c0522ec91b`

Timestamp: `2026-05-11 11:46 America/Los_Angeles`

## Artifacts

### `cargo-test-prepare-coordinator-delete-lib.log`

- Command: `script -q -e -c "cargo test prepare_coordinator_delete --lib" review/30839-spire-coordinator-delete-forwarding/artifacts/cargo-test-prepare-coordinator-delete-lib.log`
- Lane / fixture: Rust-side PG18 `pg_test` lane, focused coordinator DELETE prepared-forwarding helper test.
- Storage format / rerank mode: not a recall/rerank benchmark.
- Cluster layout: pgrx PG18 test cluster with loopback libpq connection.
- Isolated one-index-per-table or shared-table surface: isolated test tables.
- Result:
  - `test tests::pg_test_ec_spire_prepare_coordinator_delete_tuple_payload_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1640 filtered out`

### `cargo-test-forward-coordinator-update-lib.log`

- Command: `script -q -e -c "cargo test forward_coordinator_update --lib" review/30839-spire-coordinator-delete-forwarding/artifacts/cargo-test-forward-coordinator-update-lib.log`
- Lane / fixture: Rust-side PG18 `pg_test` lane, focused coordinator UPDATE forwarding helper tests including local `node_id = 0` placement handling.
- Storage format / rerank mode: not a recall/rerank benchmark.
- Cluster layout: pgrx PG18 test cluster with loopback libpq connection.
- Isolated one-index-per-table or shared-table surface: isolated test tables.
- Result:
  - `test tests::pg_test_ec_spire_forward_coordinator_update_local_sql ... ok`
  - `test tests::pg_test_ec_spire_forward_coordinator_update_tuple_payload_sql ... ok`
  - `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1639 filtered out`

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30839-spire-coordinator-delete-forwarding/artifacts/cargo-fmt-check.log`
- Lane / fixture: formatter check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass with the repo's existing stable-rustfmt warnings.

### `git-diff-check.log`

- Command: `script -q -e -c "git diff --check" review/30839-spire-coordinator-delete-forwarding/artifacts/git-diff-check.log`
- Lane / fixture: whitespace check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass.
