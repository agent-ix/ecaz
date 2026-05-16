# Review Request: SPIRE dropped-index diagnostic snapshots

- coder: coder1
- date: 2026-05-14
- code commit: 38c97650 `Cover SPIRE dropped-index diagnostics`
- topic: SPIRE phase 12c.13.b diagnostic snapshot survival under dropped index OIDs

## Scope

This slice covers 12c.13.b for the SPIRE operator diagnostic snapshot surface.

Changed files:

- `src/lib.rs`
- `src/tests/diagnostics.rs`

## What Changed

Added a small `relation_oid_exists` guard and applied it to the 12c.13.b snapshot entrypoints so a stale/dropped index OID returns an empty table iterator instead of reaching `index_open` and erroring before the diagnostic surface can answer.

Covered entrypoints:

- `ec_spire_index_hierarchy_snapshot`
- `ec_spire_index_object_snapshot`
- `ec_spire_index_delta_snapshot`
- `ec_spire_index_health_snapshot`
- `ec_spire_index_leaf_snapshot`
- `ec_spire_index_placement_snapshot`
- `ec_spire_index_scan_pipeline_snapshot`
- `ec_spire_index_top_graph_snapshot`
- `ec_spire_index_allocator_snapshot`
- `ec_spire_index_boundary_replica_placement_diagnostics`

Added `test_ec_spire_dropped_index_snapshots_empty`, which builds a populated SPIRE index, captures the index OID, drops the index, then asserts each listed diagnostic returns `count(*) = 0` when called with the stale OID literal.

## Test File Size Discipline

The touched test file remains below the 2500-line target:

```text
1756 src/tests/diagnostics.rs
```

No new test file was needed for this narrow operator-surface check.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/lib.rs src/tests/diagnostics.rs
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_dropped_index_snapshots_empty --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`, but exited successfully.

I did not run the pg_test binary. Earlier runtime attempts in this branch still hit the local PostgreSQL backend symbol boundary before executing tests; this slice was validated with the narrow compile-only target.

## Review Focus

Please check whether returning empty for missing OIDs is the right operator contract for these 12c.13.b entrypoints, especially the scalar-style snapshots (`health`, `top_graph`, `allocator`, `hierarchy`) that normally return one row for existing indexes.
