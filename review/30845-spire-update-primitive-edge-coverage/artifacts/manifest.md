# Artifact Manifest

Packet: `30845-spire-update-primitive-edge-coverage`

Head SHA: `93f82ed5ffea5695d1b6ea47a22d70b7dc33a8a8`

Timestamp: `2026-05-11 12:32 America/Los_Angeles`

## Artifacts

### `cargo-test-forward-coordinator-update-lib.log`

- Command: `script -q -e -c "cargo test forward_coordinator_update --lib" review/30845-spire-update-primitive-edge-coverage/artifacts/cargo-test-forward-coordinator-update-lib.log`
- Lane / fixture: Rust-side PG18 `pg_test` lane, focused coordinator UPDATE
  helper tests.
- Storage format / rerank mode: not a recall/rerank benchmark.
- Cluster layout: pgrx PG18 test cluster with loopback libpq connection for
  the remote-owned case.
- Isolated one-index-per-table or shared-table surface: isolated test tables.
- Result:
  - `test tests::pg_test_ec_spire_forward_coordinator_update_local_sql ... ok`
  - `test tests::pg_test_ec_spire_forward_coordinator_update_tuple_payload_sql ... ok`
  - `test tests::pg_test_ec_spire_forward_coordinator_update_missing_placement_sql - should panic ... ok`
  - `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1645 filtered out`

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30845-spire-update-primitive-edge-coverage/artifacts/cargo-fmt-check.log`
- Lane / fixture: formatter check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass with the repo's existing stable-rustfmt warnings.

### `git-diff-check.log`

- Command: `script -q -e -c "git diff --check" review/30845-spire-update-primitive-edge-coverage/artifacts/git-diff-check.log`
- Lane / fixture: whitespace check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass.
