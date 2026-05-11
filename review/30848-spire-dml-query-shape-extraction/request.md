# SPIRE DML Query Shape Extraction

## Scope

This packet advances the `30847` pure DML front-door classifier toward the
reviewer-confirmed planner/CustomScan integration by adding a PostgreSQL
query-tree extraction layer. It does not install a plan-changing planner hook
or execute UPDATE/DELETE/PK SELECT through CustomScan yet.

Changes:

- Adds `classify_dml_frontdoor_query(...)`, which maps a `pg_sys::Query` plus
  table metadata into the existing ADR-069 DML shape classifier.
- Extracts the v1 safety facts needed by the front door:
  - command type for UPDATE / DELETE / PK SELECT;
  - single range-table shape;
  - subquery / CTE / set-operation blockers;
  - RETURNING blocker;
  - bigint primary-key equality predicate, including const and param values;
  - UPDATE target columns and SELECT projected columns.
- Uses `pg_sys::get_opcode(opno) == F_INT8EQ` rather than a missing generated
  int8 equality operator constant.
- Adds focused Rust unit coverage for the new query-layer helpers that can be
  tested without constructing live PostgreSQL lists.
- Updates the DML front-door diagnostic next step from "add classifier" to
  "wire planner hook CustomScan executor."
- Updates the Phase 11 tracker with packet `30848`.

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
    `test tests::pg_test_ec_spire_coordinator_dml_frontdoor_plan_sql ... ok`
  - summary: `9 passed; 0 failed; 1648 filtered out`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm this `pg_sys::Query` extraction boundary is the right next layer
  before installing an actual planner hook.
- Confirm the strict single-table and exact bigint-PK equality interpretation
  matches the v1 hook shape from `30843`.
- Confirm target-column extraction by non-junk `TargetEntry.resno` is the
  right basis for rejecting PK/embedding UPDATEs once the hook feeds relation
  metadata into this context.

## Artifacts

- `review/30848-spire-dml-query-shape-extraction/artifacts/manifest.md`
- `review/30848-spire-dml-query-shape-extraction/artifacts/cargo-test-dml-frontdoor-lib.log`
- `review/30848-spire-dml-query-shape-extraction/artifacts/cargo-fmt-check.log`
- `review/30848-spire-dml-query-shape-extraction/artifacts/git-diff-check.log`
