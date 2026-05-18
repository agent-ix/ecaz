# Review Request: SPIRE DML Replacement Decision

## Scope

This packet adds the catalog-backed decision layer that turns a classified
ADR-069 DML front-door query into the exact DML CustomScan replacement contract.
It still leaves plan rewriting disabled; the point of this slice is to make the
next executor-replacement packet consume one typed decision row instead of
re-deriving operation, primitive, and fail-closed behavior.

Code commit: `bb7bbbf5cf17ddca424171b5ed91730c54a25346`

Changes:

- Adds `SpireDmlFrontdoorReplacementDecisionRow`.
- Adds `dml_frontdoor_replacement_decision_catalog_row(query)`, using the
  hook-safe catalog/relcache relation context.
- Maps supported shapes to the planned DML CustomScan modes and primitives:
  - `update_non_embedding` -> `coordinator_update_tuple_payload` /
    `ec_spire_forward_coordinator_update_tuple_payload`
  - `delete` -> `coordinator_delete_tuple_payload` /
    `ec_spire_prepare_coordinator_delete_tuple_payload`
  - `pk_select` -> `coordinator_pk_select_tuple_payload` /
    `ec_spire_forward_coordinator_select_tuple_payload`
- Keeps unsupported shapes mapped to `custom_scan_mode = none` and the ADR-069
  planner-error next step, so the future rewrite path can fail closed instead
  of falling through to the coordinator heap path.
- Updates hook observation to record the replacement decision result.
- Adds SQL diagnostic `ec_spire_dml_frontdoor_replacement_sql(sql text)`.
- Fixes actual PostgreSQL UPDATE/DELETE query extraction to use
  `resultRelation` and reject extra `FROM` / join shapes separately.
- Adds PG18 coverage for supported PK SELECT, supported non-embedding UPDATE,
  embedding UPDATE rejection, and `UPDATE ... FROM` rejection.
- Updates the Phase 11 task file with the 30859 milestone.

## Validation

- `cargo test dml_frontdoor --lib`
  - 17 passed, 0 failed, 1648 filtered out.
- `cargo fmt --check`
  - Passed with the existing stable-rustfmt warnings about unstable import
    options.
- `git diff --check`
  - Passed.

Artifacts are recorded in `artifacts/manifest.md`.

## Review Focus

1. Confirm the replacement decision maps each supported v1 shape to the correct
   CustomScan mode and coordinator primitive.
2. Confirm unsupported shapes preserve the fail-closed ADR-069 planner-error
   path, especially embedding UPDATE and `UPDATE ... FROM`.
3. Confirm the UPDATE/DELETE `resultRelation` extraction matches PostgreSQL
   analyzed query trees and does not accidentally admit joined DML.
