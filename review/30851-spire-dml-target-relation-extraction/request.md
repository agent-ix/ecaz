# SPIRE DML Target Relation Extraction

## Scope

This packet adds hook-side target heap relation extraction from analyzed
PostgreSQL `Query` trees. It does not perform relation metadata lookup inside
the planner hook and does not rewrite plans yet.

Changes:

- Adds `dml_frontdoor_target_relation_oid(query)` for the DML front-door path.
- Resolves target heap relation OIDs for:
  - `UPDATE ...`;
  - `DELETE ...`;
  - single-table `SELECT ...`.
- Rejects joined SELECT shapes before any relation metadata lookup.
- Adds PG18 coverage that analyzes real SQL for UPDATE, DELETE, SELECT, and a
  joined SELECT rejection case.
- Updates the Phase 11 tracker with packet `30851`.

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
  - summary: `12 passed; 0 failed; 1648 filtered out`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm the target relation extraction semantics for UPDATE/DELETE vs
  single-table SELECT.
- Confirm rejecting joined SELECT before metadata lookup matches the v1
  fail-closed front-door rule.
- Confirm the next slice should combine target relation extraction plus
  relation-context metadata to classify real planner-hook queries before plan
  replacement.

## Artifacts

- `review/30851-spire-dml-target-relation-extraction/artifacts/manifest.md`
- `review/30851-spire-dml-target-relation-extraction/artifacts/cargo-test-dml-frontdoor-lib.log`
- `review/30851-spire-dml-target-relation-extraction/artifacts/cargo-fmt-check.log`
- `review/30851-spire-dml-target-relation-extraction/artifacts/git-diff-check.log`
