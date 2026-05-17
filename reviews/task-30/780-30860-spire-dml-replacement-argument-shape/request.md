# Review Request: SPIRE DML Replacement Argument Shape

## Scope

This packet extends the DML front-door replacement decision from packet 30859
with the executor argument shape the future DML CustomScan node will need. It
still leaves plan rewriting disabled.

Code commit: `9c73a87b6284a239c4d871f941c704b4d5f4d389`

Changes:

- Extends `SpireDmlFrontdoorReplacementDecisionRow` with:
  - `pk_column`
  - `updated_columns`
  - `projected_columns`
- Refactors relation-backed query classification into a detail helper so the
  replacement decision and classifier use the same extracted target-list facts.
- Extends `ec_spire_dml_frontdoor_replacement_sql(sql text)` to return those
  argument-shape fields.
- Adds PG18 assertions that:
  - PK SELECT replacement reports projected columns `id,title`.
  - non-embedding UPDATE replacement reports PK column `id` and updated column
    `title`.
- Updates the Phase 11 task file with the 30860 milestone.

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

1. Confirm the replacement decision now carries the executor argument shape
   needed by the next DML CustomScan replacement slice.
2. Confirm the refactor keeps classifier and decision extraction aligned rather
   than creating a second independent target-list interpretation.
3. Confirm the SQL diagnostic return-shape change is acceptable before plan
   rewriting is enabled.
