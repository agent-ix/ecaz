# Review Request: SPIRE DML Frontdoor Classify Diagnostic

## Scope

This packet adds a SQL-visible bridge for the ADR-069 DML front-door classifier
without yet changing planner output.

Code commit: `0788cc0154ba59927780d23ac28ac908f2778f29`

Changes:

- Adds `ec_spire_dml_frontdoor_classify_sql(sql text)`.
- The diagnostic:
  - parses and analyzes exactly one SQL statement;
  - extracts the target heap relation from the analyzed `Query`;
  - loads the existing relation context for that heap;
  - invokes the shared DML front-door query classifier;
  - returns target relation OID, relation status, supported flag, operation,
    kind, status, error, hint, and next step.
- Keeps the production planner hook pass-through. This avoids putting the
  SPI-backed relation-context lookup inside `planner_hook` before the replacement
  plan path has a recursion-safe metadata source.
- Extends the existing PG18 query-shape fixture so the SQL diagnostic proves:
  - `SELECT id FROM ... WHERE id = 5` returns `pk_select_by_pk`;
  - read-only CTE-prefixed SELECT returns `unsupported_subquery_shape`.

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

1. Confirm this diagnostic bridge is useful and appropriately scoped before
   hook-side plan replacement.
2. Confirm the single-statement parser/analyzer rejection is the right safety
   default for an operator-facing classification probe.
3. Confirm it is acceptable that relation context still uses the existing SPI
   lookup here, while planner-hook execution remains pass-through until a
   recursion-safe metadata path lands.
