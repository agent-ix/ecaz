# Artifact Manifest

Packet: `30852-spire-dml-relation-context-hardening`

Head SHA: `d2f13a162c0ea413264211f59937c22fcd1d3cb2`

Timestamp: `2026-05-11 13:38 America/Los_Angeles`

## Artifacts

### `cargo-test-dml-frontdoor-lib.log`

- Command: `script -q -e -c "cargo test dml_frontdoor --lib" review/30852-spire-dml-relation-context-hardening/artifacts/cargo-test-dml-frontdoor-lib.log`
- Lane / fixture: Rust-side classifier/query-layer unit tests plus PG18
  `pg_test` for DML front-door diagnostic SQL surfaces, relation-context
  metadata, target relation extraction, INCLUDE-column hardening, and
  multi-index rejection.
- Storage format / rerank mode: not a recall/rerank benchmark.
- Cluster layout: pgrx PG18 test cluster for SQL diagnostic fixtures.
- Isolated one-index-per-table or shared-table surface: one fixture uses one
  table with one `ec_spire` index plus an INCLUDE column; one negative fixture
  intentionally creates two `ec_spire` indexes on the same heap.
- Result:
  - `test am::ec_spire::dml_frontdoor::tests::classifier_accepts_update_delete_and_pk_select_v1_shapes ... ok`
  - `test am::ec_spire::dml_frontdoor::tests::classifier_rejects_joins_subqueries_and_returning ... ok`
  - `test am::ec_spire::dml_frontdoor::tests::classifier_requires_bigint_pk_equality_predicate ... ok`
  - `test am::ec_spire::dml_frontdoor::tests::classifier_rejects_embedding_and_pk_updates ... ok`
  - `test am::ec_spire::dml_frontdoor::tests::classifier_rejects_empty_update_or_projection ... ok`
  - `test am::ec_spire::dml_frontdoor::tests::query_layer_maps_command_and_subquery_flags ... ok`
  - `test am::ec_spire::dml_frontdoor::tests::query_layer_binds_target_relation_var_to_column_name ... ok`
  - `test am::ec_spire::dml_frontdoor::tests::query_layer_recognizes_bigint_const_and_param_values ... ok`
  - `test tests::pg_test_ec_spire_dml_frontdoor_hook_status_installed_pass_through ... ok`
  - `test tests::pg_test_ec_spire_coordinator_dml_frontdoor_plan_sql ... ok`
  - `test tests::pg_test_ec_spire_dml_frontdoor_target_relation_oid_sql ... ok`
  - `test tests::pg_test_ec_spire_dml_frontdoor_relation_context_sql ... ok`
  - `test tests::pg_test_ec_spire_dml_frontdoor_rejects_multi_index - should panic ... ok`
  - `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out`

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30852-spire-dml-relation-context-hardening/artifacts/cargo-fmt-check.log`
- Lane / fixture: formatter check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass with the repo's existing stable-rustfmt warnings.

### `git-diff-check.log`

- Command: `script -q -e -c "git diff --check" review/30852-spire-dml-relation-context-hardening/artifacts/git-diff-check.log`
- Lane / fixture: whitespace check.
- Storage format / rerank mode: not applicable.
- Cluster layout: not applicable.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result: pass.
