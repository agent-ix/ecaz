# SPIRE DML Front-Door Shape Classifier

## Scope

This packet starts the reviewer-confirmed planner/CustomScan front-door path
from `30843` with a pure shape-classifier scaffold. It does not install the
planner hook or execute UPDATE/DELETE/PK SELECT through CustomScan yet.

Changes:

- Adds `src/am/ec_spire/dml_frontdoor.rs`, a v1 shape classifier for:
  - non-embedding `UPDATE ... WHERE pk = ...`;
  - `DELETE ... WHERE pk = ...`;
  - `SELECT projection ... WHERE pk = ...`.
- Encodes the fail-closed rules confirmed by the reviewer:
  - single-table only;
  - bigint primary-key equality predicate only;
  - no joins;
  - no subqueries;
  - no `RETURNING`;
  - no primary-key UPDATE;
  - embedding UPDATE returns the ADR-069 rejection message and hint.
- Adds focused Rust unit coverage for supported shapes and unsupported
  shape classes.
- Updates `ec_spire_coordinator_dml_frontdoor_plan()` to name
  `planner_customscan_hook` instead of the earlier hook/view placeholders.
- Updates the Phase 11 tracker with packet `30847`.

## Validation

- `cargo test dml_frontdoor --lib`
  - result: pass.
  - key lines:
    `test am::ec_spire::dml_frontdoor::tests::classifier_accepts_update_delete_and_pk_select_v1_shapes ... ok`
    `test am::ec_spire::dml_frontdoor::tests::classifier_rejects_joins_subqueries_and_returning ... ok`
    `test am::ec_spire::dml_frontdoor::tests::classifier_requires_bigint_pk_equality_predicate ... ok`
    `test am::ec_spire::dml_frontdoor::tests::classifier_rejects_embedding_and_pk_updates ... ok`
    `test am::ec_spire::dml_frontdoor::tests::classifier_rejects_empty_update_or_projection ... ok`
    `test tests::pg_test_ec_spire_coordinator_dml_frontdoor_plan_sql ... ok`
  - summary: `6 passed; 0 failed; 1648 filtered out`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm the classifier's supported/unsupported shape matrix matches the
  `30843` hook-boundary direction.
- Confirm embedding UPDATE should use the ADR-069 rejection message at the
  classifier layer as well as the shared primitive guard.
- Confirm `planner_customscan_hook` is the right diagnostic integration label
  before the actual planner hook lands.

## Artifacts

- `review/30847-spire-dml-frontdoor-shape-classifier/artifacts/manifest.md`
- `review/30847-spire-dml-frontdoor-shape-classifier/artifacts/cargo-test-dml-frontdoor-lib.log`
- `review/30847-spire-dml-frontdoor-shape-classifier/artifacts/cargo-fmt-check.log`
- `review/30847-spire-dml-frontdoor-shape-classifier/artifacts/git-diff-check.log`
