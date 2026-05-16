# Artifact Manifest

Packet: `30844-spire-insert-v1-feedback-followups`

Head SHA: `4173e385e4fc912e2b8397296752f32960137254`

Timestamp: `2026-05-11 12:26 America/Los_Angeles`

## Artifacts

### `cargo-test-insert-trigger-source-identity-json-roundtrip-lib.log`

- Command: `script -q -e -c "cargo test insert_trigger_source_identity_json_roundtrip --lib" review/30844-spire-insert-v1-feedback-followups/artifacts/cargo-test-insert-trigger-source-identity-json-roundtrip-lib.log`
- Lane / fixture: Rust-side PG18 `pg_test` lane, focused INSERT trigger bytea
  source-identity JSON roundtrip test.
- Storage format / rerank mode: not a recall/rerank benchmark.
- Cluster layout: pgrx PG18 test cluster.
- Isolated one-index-per-table or shared-table surface: isolated test table.
- Result:
  - `test tests::pg_test_ec_spire_insert_trigger_source_identity_json_roundtrip_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1646 filtered out`

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30844-spire-insert-v1-feedback-followups/artifacts/cargo-fmt-check.log`
- Lane / fixture: formatter check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass with the repo's existing stable-rustfmt warnings.

### `git-diff-check.log`

- Command: `script -q -e -c "git diff --check" review/30844-spire-insert-v1-feedback-followups/artifacts/git-diff-check.log`
- Lane / fixture: whitespace check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass.
