# Review Request: SPIRE DML PK Byte Encoding

## Scope

This packet adds the bigint primary-key byte encoder the future DML CustomScan
executor will use after evaluating a const or parameter PK expression. It does
not change planner path generation or plan rewriting.

Code commit: `8d447bf6f15012edb41692eaf5d8425987dee9a5`

Changes:

- Adds `dml_frontdoor_bigint_pk_value_bytes(i64) -> Vec<u8>`.
- Re-exports the helper through the `ec_spire` and `am` module boundaries for
  the upcoming CustomScan executor path.
- Adds PG18 coverage comparing the helper output to PostgreSQL
  `int8send(...)::bytea` for `0`, positive, negative, `i64::MAX`, and
  `i64::MIN`.
- Updates the Phase 11 task file with the 30864 milestone.

## Validation

- `cargo test dml_frontdoor --lib`
  - 20 passed, 0 failed, 1648 filtered out.
- `cargo fmt --check`
  - Passed with the existing stable-rustfmt warnings about unstable import
    options.
- `git diff --check`
  - Passed.

Artifacts are recorded in `artifacts/manifest.md`.

## Review Focus

1. Confirm the byte encoding matches the placement-directory and primitive
   contract based on `int8send(...)::bytea`.
2. Confirm this helper is an appropriate shared boundary for the upcoming DML
   CustomScan executor mode.
3. Confirm no planner or executor behavior changes in this packet.
