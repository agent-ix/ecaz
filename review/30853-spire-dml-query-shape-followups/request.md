# Review Request: SPIRE DML Query Shape Follow-ups

## Scope

This packet closes the actionable `30848` query-shape feedback for the ADR-069
DML front door.

Changes in code commit `af8dc5663e90c66ce58691d644b3c50f51830425`:

- Pin ADR-069's v1 rule that all CTE-prefixed front-door statements, including
  read-only `WITH`, fail closed as `unsupported_subquery_shape`.
- Extend the query extractor to accept PostgreSQL's bigint/integer equality
  variants for v1 bigint-PK predicates:
  - `int8eq`
  - `int84eq`
  - `int82eq`
  - `int48eq`
  - `int28eq`
- Treat int2/int4/int8 constants or params as acceptable bigint-PK predicate
  values only after the predicate has already bound to the target PK column and
  one of the bigint equality variants.
- Add PG18 coverage proving analyzed `WHERE id = 5` classifies as
  `pk_select_by_pk` and CTE-prefixed SELECT rejects as
  `unsupported_subquery_shape`.
- Add a small code comment on the pass-through planner hook clarifying that
  classification and plan replacement land in later packets.

## Validation

- `cargo test dml_frontdoor --lib`
  - 15 passed, 0 failed, 1648 filtered out.
  - Covers the new Rust opcode matrix and PG18 analyzed-query fixture.
- `cargo fmt --check`
  - Passed with the existing stable-rustfmt warnings about unstable import
    options.
- `git diff --check`
  - Passed.

Artifacts are recorded in `artifacts/manifest.md`.

## Review Focus

1. Confirm the accepted equality opcode set is exactly the right v1 surface for
   bigint-PK equality against int2/int4/int8 literals or params.
2. Confirm the CTE rejection wording in ADR-069 is explicit enough for
   operators and application authors.
3. Confirm the test-only re-export boundary is acceptable until the planner hook
   invokes the classifier directly.
