# Artifact Manifest

Packet: `30847-spire-dml-frontdoor-shape-classifier`

Head SHA: `38ee897a26287611169995365bbb25592148d465`

Timestamp: `2026-05-11 12:49 America/Los_Angeles`

## Artifacts

### `cargo-test-dml-frontdoor-lib.log`

- Command: `script -q -e -c "cargo test dml_frontdoor --lib" review/30847-spire-dml-frontdoor-shape-classifier/artifacts/cargo-test-dml-frontdoor-lib.log`
- Lane / fixture: Rust-side unit tests plus PG18 `pg_test` for the DML
  front-door diagnostic SQL surface.
- Storage format / rerank mode: not a recall/rerank benchmark.
- Cluster layout: pgrx PG18 test cluster for the SQL diagnostic fixture.
- Isolated one-index-per-table or shared-table surface: no table/index fixture;
  pure classifier tests plus SQL status surface.
- Result:
  - `test am::ec_spire::dml_frontdoor::tests::classifier_accepts_update_delete_and_pk_select_v1_shapes ... ok`
  - `test am::ec_spire::dml_frontdoor::tests::classifier_rejects_joins_subqueries_and_returning ... ok`
  - `test am::ec_spire::dml_frontdoor::tests::classifier_requires_bigint_pk_equality_predicate ... ok`
  - `test am::ec_spire::dml_frontdoor::tests::classifier_rejects_embedding_and_pk_updates ... ok`
  - `test am::ec_spire::dml_frontdoor::tests::classifier_rejects_empty_update_or_projection ... ok`
  - `test tests::pg_test_ec_spire_coordinator_dml_frontdoor_plan_sql ... ok`
  - `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out`

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30847-spire-dml-frontdoor-shape-classifier/artifacts/cargo-fmt-check.log`
- Lane / fixture: formatter check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass with the repo's existing stable-rustfmt warnings.

### `git-diff-check.log`

- Command: `script -q -e -c "git diff --check" review/30847-spire-dml-frontdoor-shape-classifier/artifacts/git-diff-check.log`
- Lane / fixture: whitespace check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass.
