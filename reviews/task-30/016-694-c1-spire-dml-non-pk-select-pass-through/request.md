# Review Request: SPIRE DML non-PK SELECT pass-through coverage

- coder: coder1
- date: 2026-05-14
- code commit: edbd43aa `Cover DML non-PK SELECT pass-through`
- topic: SPIRE phase 12c.9.a non-PK SELECT pass-through end-to-end

## Scope

This slice adds the missing 12c.9.a end-to-end fixture while avoiding more
growth in the already-large DML frontdoor test file.

Changed files:

- `src/tests/dml_frontdoor_select.rs`
- `src/tests/mod.rs`

## What Changed

Added `test_ec_spire_dml_frontdoor_non_pk_select_passes_through_sql` in a new
focused include file.

The fixture:

- creates a table with a bigint primary key, non-PK `title`, and SPIRE-indexed
  `embedding`
- verifies the relation is SPIRE-fronted through
  `ec_spire_dml_frontdoor_relation_context`
- runs `EXPLAIN (COSTS OFF)` for a `WHERE title LIKE 'keep-%'` query
- asserts the plan is an ordinary PostgreSQL `Seq Scan` or `Index Scan`
- asserts the plan is not `Custom Scan (EcSpireDistributedScan)`
- executes the query and pins the exact returned rows

## Test File Size Discipline

The new file is intentionally small:

```text
67 src/tests/dml_frontdoor_select.rs
2570 src/tests/dml_frontdoor.rs
```

`dml_frontdoor.rs` was already above the 2500-line target, so this slice starts
a separate DML SELECT coverage file instead of adding more tests to the large
legacy include.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/mod.rs src/tests/dml_frontdoor_select.rs
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_dml_frontdoor_non_pk_select_passes_through_sql --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`, but exited
successfully.

I did not run the pg_test binary. Earlier runtime attempts in this branch still
hit the local PostgreSQL backend symbol boundary before executing tests; this
slice was validated with the narrow compile-only target.

## Review Focus

Please check whether asserting ordinary scan shape plus exact returned rows is
sufficient for 12c.9.a, or whether reviewers want an additional diagnostic that
proves the DML frontdoor classifier explicitly returned a non-PK pass-through
reason for this SQL shape.
