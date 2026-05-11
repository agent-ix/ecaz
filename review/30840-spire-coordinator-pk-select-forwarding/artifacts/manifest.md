# Artifact Manifest

Packet: `30840-spire-coordinator-pk-select-forwarding`

Head SHA: `d05b41176fa2974c1cd393bb6a428ef3047272ba`

Timestamp: `2026-05-11 11:57 America/Los_Angeles`

## Artifacts

### `cargo-test-forward-coordinator-select-lib.log`

- Command: `script -q -e -c "cargo test forward_coordinator_select --lib" review/30840-spire-coordinator-pk-select-forwarding/artifacts/cargo-test-forward-coordinator-select-lib.log`
- Lane / fixture: Rust-side PG18 `pg_test` lane, focused coordinator PK SELECT forwarding helper tests.
- Storage format / rerank mode: not a recall/rerank benchmark.
- Cluster layout: pgrx PG18 test cluster with loopback libpq connection.
- Isolated one-index-per-table or shared-table surface: isolated test tables.
- Result:
  - `test tests::pg_test_ec_spire_forward_coordinator_select_local_sql ... ok`
  - `test tests::pg_test_ec_spire_forward_coordinator_select_tuple_payload_sql ... ok`
  - `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1641 filtered out`

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30840-spire-coordinator-pk-select-forwarding/artifacts/cargo-fmt-check.log`
- Lane / fixture: formatter check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass with the repo's existing stable-rustfmt warnings.

### `git-diff-check.log`

- Command: `script -q -e -c "git diff --check" review/30840-spire-coordinator-pk-select-forwarding/artifacts/git-diff-check.log`
- Lane / fixture: whitespace check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass.
