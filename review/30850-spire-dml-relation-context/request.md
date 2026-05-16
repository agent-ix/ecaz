# SPIRE DML Relation Context Metadata

## Scope

This packet adds the relation-metadata bridge that the DML planner hook needs
before it can replace eligible UPDATE/DELETE/PK SELECT plans. It does not
rewrite plans or execute transparent DML yet.

Changes:

- Adds `dml_frontdoor_relation_context_row(heap_relation_oid)`.
- Exposes `ec_spire_dml_frontdoor_relation_context(heap_relation_oid)` with:
  - selected `ec_spire` index OID for the heap relation;
  - whether the table is a DML front-door candidate;
  - v1 bigint primary-key column and type;
  - ordinary heap column count;
  - indexed embedding columns from the `ec_spire` key columns.
- Uses `idx.indnkeyatts` when reading index key columns so INCLUDE columns do
  not appear as embedding/front-door key columns.
- Adds a PG18 fixture that creates a bigint-PK table with an `ec_spire`
  embedding index and verifies the relation context.
- Updates the Phase 11 tracker with packet `30850`.

## Validation

- `cargo test dml_frontdoor --lib`
  - result: pass.
  - key lines:
    `test am::ec_spire::dml_frontdoor::tests::classifier_accepts_update_delete_and_pk_select_v1_shapes ... ok`
    `test am::ec_spire::dml_frontdoor::tests::classifier_rejects_joins_subqueries_and_returning ... ok`
    `test am::ec_spire::dml_frontdoor::tests::classifier_requires_bigint_pk_equality_predicate ... ok`
    `test am::ec_spire::dml_frontdoor::tests::classifier_rejects_embedding_and_pk_updates ... ok`
    `test am::ec_spire::dml_frontdoor::tests::classifier_rejects_empty_update_or_projection ... ok`
    `test am::ec_spire::dml_frontdoor::tests::query_layer_maps_command_and_subquery_flags ... ok`
    `test am::ec_spire::dml_frontdoor::tests::query_layer_binds_target_relation_var_to_column_name ... ok`
    `test am::ec_spire::dml_frontdoor::tests::query_layer_recognizes_bigint_const_and_param_values ... ok`
    `test tests::pg_test_ec_spire_dml_frontdoor_hook_status_installed_pass_through ... ok`
    `test tests::pg_test_ec_spire_coordinator_dml_frontdoor_plan_sql ... ok`
    `test tests::pg_test_ec_spire_dml_frontdoor_relation_context_sql ... ok`
  - summary: `11 passed; 0 failed; 1648 filtered out`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm that the relation-context fields are sufficient for the next
  planner-hook slice to call `classify_dml_frontdoor_query(...)`.
- Confirm `indnkeyatts` is the right boundary for embedding/index key columns.
- Confirm the candidate status should currently mean "has ec_spire index plus
  one bigint primary-key column"; remote-placement enforcement remains for the
  executor/replacement slice.

## Artifacts

- `review/30850-spire-dml-relation-context/artifacts/manifest.md`
- `review/30850-spire-dml-relation-context/artifacts/cargo-test-dml-frontdoor-lib.log`
- `review/30850-spire-dml-relation-context/artifacts/cargo-fmt-check.log`
- `review/30850-spire-dml-relation-context/artifacts/git-diff-check.log`
