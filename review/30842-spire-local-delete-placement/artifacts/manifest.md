# Artifact Manifest

Packet: `30842-spire-local-delete-placement`

Head SHA: `88c3f0cfac3507376fae7836996acf4464561551`

Timestamp: `2026-05-11 12:12 America/Los_Angeles`

## Artifacts

### `cargo-test-prepare-coordinator-delete-lib.log`

- Command: `script -q -e -c "cargo test prepare_coordinator_delete --lib" review/30842-spire-local-delete-placement/artifacts/cargo-test-prepare-coordinator-delete-lib.log`
- Lane / fixture: Rust-side PG18 `pg_test` lane, focused coordinator DELETE helper tests.
- Storage format / rerank mode: not a recall/rerank benchmark.
- Cluster layout: pgrx PG18 test cluster with loopback libpq connection for the remote-owned case.
- Isolated one-index-per-table or shared-table surface: isolated test tables.
- Result:
  - `test tests::pg_test_ec_spire_prepare_coordinator_delete_local_sql ... ok`
  - `test tests::pg_test_ec_spire_prepare_coordinator_delete_tuple_payload_sql ... ok`
  - `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1643 filtered out`

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30842-spire-local-delete-placement/artifacts/cargo-fmt-check.log`
- Lane / fixture: formatter check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass with the repo's existing stable-rustfmt warnings.

### `git-diff-check.log`

- Command: `script -q -e -c "git diff --check" review/30842-spire-local-delete-placement/artifacts/git-diff-check.log`
- Lane / fixture: whitespace check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass.
