# SPIRE DML Planner Hook Scaffold

## Scope

This packet installs the ADR-069 DML front-door planner-hook boundary as a
pass-through hook. It does not rewrite plans or execute transparent
UPDATE/DELETE/PK SELECT yet.

Changes:

- Adds `register_dml_frontdoor_planner_hook()`, called from `_PG_init()`.
- Chains any previous PostgreSQL `planner_hook` and delegates to
  `standard_planner` when no previous hook exists.
- Keeps the hook behavior pass-through while the query classifier and relation
  metadata wiring mature.
- Adds `ec_spire_dml_frontdoor_hook_status()` so reviewers/operators can see:
  - the planner hook is installed;
  - the query shape classifier exists;
  - plan rewriting is still disabled.
- Updates `ec_spire_coordinator_dml_frontdoor_plan()` next steps to point at
  relation metadata plus CustomScan executor replacement.
- Updates the Phase 11 tracker with packet `30849`.

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
  - summary: `10 passed; 0 failed; 1648 filtered out`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm the planner-hook chain/delegate behavior is acceptable before the
  plan-rewrite slice.
- Confirm the status surface correctly advertises this as hook-installed but
  plan-rewrite-disabled.
- Confirm the next production slice should bind relation metadata and replace
  eligible UPDATE/DELETE/PK SELECT plans with the CustomScan executor path.

## Artifacts

- `review/30849-spire-dml-planner-hook-scaffold/artifacts/manifest.md`
- `review/30849-spire-dml-planner-hook-scaffold/artifacts/cargo-test-dml-frontdoor-lib.log`
- `review/30849-spire-dml-planner-hook-scaffold/artifacts/cargo-fmt-check.log`
- `review/30849-spire-dml-planner-hook-scaffold/artifacts/git-diff-check.log`
