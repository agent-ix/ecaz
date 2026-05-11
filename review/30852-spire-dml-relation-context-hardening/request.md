# SPIRE DML Relation Context Hardening

## Scope

This packet addresses reviewer feedback from `30850` before the DML executor
replacement slice depends on relation-context metadata.

Changes:

- Rejects heap relations with more than one `ec_spire` index instead of
  silently choosing the lowest-OID index.
- Pins the v1 front-door rule in ADR-069:
  - at most one `ec_spire` index per distributed heap relation;
  - one `bigint` primary-key column;
  - composite and non-`bigint` primary keys are deferred and fail closed.
- Extends PG18 relation-context coverage with an `INCLUDE (source_identity)`
  ec_spire index using `WITH (source_identity = 'include')` and asserts
  `embedding_columns` remains `embedding`.
- Adds PG18 coverage that two `ec_spire` indexes on one heap fail with the
  multi-index v1 error.
- Updates the Phase 11 tracker with packet `30852`.

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
    `test tests::pg_test_ec_spire_dml_frontdoor_target_relation_oid_sql ... ok`
    `test tests::pg_test_ec_spire_dml_frontdoor_relation_context_sql ... ok`
    `test tests::pg_test_ec_spire_dml_frontdoor_rejects_multi_index - should panic ... ok`
  - summary: `13 passed; 0 failed; 1648 filtered out`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm the multi-index fail-closed error is the right v1 behavior.
- Confirm the INCLUDE-column fixture covers the `indnkeyatts` regression risk.
- Confirm ADR-069 now pins the single-index / single-column-bigint-PK rule
  clearly enough for operators.

## Artifacts

- `review/30852-spire-dml-relation-context-hardening/artifacts/manifest.md`
- `review/30852-spire-dml-relation-context-hardening/artifacts/cargo-test-dml-frontdoor-lib.log`
- `review/30852-spire-dml-relation-context-hardening/artifacts/cargo-fmt-check.log`
- `review/30852-spire-dml-relation-context-hardening/artifacts/git-diff-check.log`
