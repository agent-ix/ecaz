# Review Request: SPIRE DML Hook Classifier Observation

## Scope

This packet wires the ADR-069 DML front-door planner hook to invoke the shared
query-shape classifier in pass-through mode. The hook now uses the non-SPI
catalog relation-context loader from packet 30855, records the backend-local
last classification, and exposes that observation through
`ec_spire_dml_frontdoor_hook_status()`.

Code commit: `c27faed93e25a32b52d7df4075f1a6a73da4748f`

Changes:

- Has `ec_spire_dml_frontdoor_planner_hook` observe each supported target
  relation query before delegating to the previous hook or `standard_planner`.
- Adds hook-side classification through
  `dml_frontdoor_classify_query_with_catalog_context(...)`, using relcache /
  catalog metadata instead of SPI.
- Extends `ec_spire_dml_frontdoor_hook_status()` with:
  - `query_shape_classifier_invoked_by_hook`
  - `last_classification_supported`
  - `last_classification_kind`
  - `last_classification_status`
- Keeps `plan_rewrite_enabled = false`; this packet does not replace plans or
  execute forwarded DML.
- Extends PG18 coverage so an actual planned
  `SELECT id FROM ... WHERE id = 5` proves the pass-through hook records
  `pk_select_by_pk` / `supported_v1_shape`.
- Updates the Phase 11 task file with the 30856 milestone.

## Validation

- `cargo test dml_frontdoor --lib`
  - 15 passed, 0 failed, 1648 filtered out.
- `cargo fmt --check`
  - Passed with the existing stable-rustfmt warnings about unstable import
    options.
- `git diff --check`
  - Passed.

Artifacts are recorded in `artifacts/manifest.md`.

## Review Focus

1. Confirm invoking the classifier from the planner hook is safe while plan
   rewriting remains disabled.
2. Confirm the hook does not use SPI and only depends on the catalog/relcache
   relation context introduced in packet 30855.
3. Confirm the SQL-visible hook status fields are sufficient for the next
   CustomScan executor replacement slice without implying behavior change.
