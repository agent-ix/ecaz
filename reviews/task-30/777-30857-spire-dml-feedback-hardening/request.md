# Review Request: SPIRE DML Feedback Hardening

## Scope

This packet folds reviewer feedback from packets 30853 and 30854 into the
ADR-069 DML front-door classifier and documentation.

Code commit: `500876c0c602c8ef13a01286f489ed8f3ba1c735`
Artifact head: `52d9cca2d05c63c34c98925d9960feeb4fb7ebfd`

Changes:

- Makes coercion-wrapper walking explicit and bounded for bigint PK predicate
  value extraction.
- Adds a unit regression proving nested PostgreSQL integer coercion wrappers
  still classify as `ParamBigint` when the outer result is `int8`.
- Leaves non-`int8` outer coercion wrappers fail-closed as `Other`.
- Adds the reviewer-requested comment on PG18 single-argument `FuncExpr` list
  extraction.
- Updates ADR-069 to document that
  `ec_spire_dml_frontdoor_classify_sql(sql text)` runs PostgreSQL parse /
  analysis / rewrite first, so standard analyzer errors surface before any
  SPIRE-specific diagnostic row.
- Updates the Phase 11 task tracker with the packet milestone.

## Validation

- `cargo test dml_frontdoor --lib`
  - 16 passed, 0 failed, 1648 filtered out.
- `cargo fmt --check`
  - Passed with the existing stable-rustfmt warnings about unstable import
    options.
- `git diff --check`
  - Passed.

Artifacts are recorded in `artifacts/manifest.md`.

## Review Focus

1. Confirm the bounded recursive wrapper walk handles the intended PostgreSQL
   coercion stack without widening accepted predicate values beyond
   int2/int4/int8 constants and params.
2. Confirm the ADR wording accurately describes the diagnostic behavior without
   implying planner-hook use of the SPI-backed diagnostic path.
3. Confirm the new regression coverage is sufficient for the 30853 P2.
