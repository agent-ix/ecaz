# Artifact Manifest

Packet: `30841-spire-embedding-update-rejection`

Head SHA: `b8d6e4b674276a4fd7e4a58a9f1497f3280839c3`

Timestamp: `2026-05-11 12:05 America/Los_Angeles`

## Artifacts

### `cargo-test-update-rejects-embedding-lib.log`

- Command: `script -q -e -c "cargo test update_rejects_embedding --lib" review/30841-spire-embedding-update-rejection/artifacts/cargo-test-update-rejects-embedding-lib.log`
- Lane / fixture: Rust-side PG18 `pg_test` lane, focused coordinator embedding UPDATE rejection test.
- Storage format / rerank mode: not a recall/rerank benchmark.
- Cluster layout: pgrx PG18 test cluster.
- Isolated one-index-per-table or shared-table surface: isolated test table.
- Result:
  - `test tests::pg_test_ec_spire_update_rejects_embedding_column_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1643 filtered out`

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30841-spire-embedding-update-rejection/artifacts/cargo-fmt-check.log`
- Lane / fixture: formatter check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass with the repo's existing stable-rustfmt warnings.

### `git-diff-check.log`

- Command: `script -q -e -c "git diff --check" review/30841-spire-embedding-update-rejection/artifacts/git-diff-check.log`
- Lane / fixture: whitespace check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass.
