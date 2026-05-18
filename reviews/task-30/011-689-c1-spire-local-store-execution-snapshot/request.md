# Review Request: SPIRE local-store execution snapshot

- coder: coder1
- date: 2026-05-14
- code commit: 70a1c437 `Add standalone SPIRE local store execution snapshot`
- topic: SPIRE phase 12c.15.c standalone sequential-backend label coverage

## Scope

This slice adds the standalone local-store execution snapshot fixture requested by 12c.15.c.

Changed file:

- `src/tests/scan.rs`

## What Changed

Added `test_ec_spire_scan_local_store_execution_mode_standalone_sql`, which builds a small SPIRE index and directly queries `ec_spire_index_scan_local_store_execution_snapshot`.

The test asserts:

- `local_store_execution_mode = sequential_backend`
- `local_store_parallelism_next_step = async_or_parallel_store_group_executor`

This separates the label contract from the larger placement-snapshot fixture, where the same values are also currently checked.

## Test File Size Discipline

The touched test file is now 1148 lines:

```text
1148 src/tests/scan.rs
```

No large test file was expanded past the 2500-line target.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/scan.rs
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_scan_local_store_execution_mode_standalone_sql --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`, but exited successfully.

I did not run the pg_test binary. Earlier runtime attempts in this branch still hit the local PostgreSQL backend symbol boundary before executing tests; this slice was validated with the narrow compile-only target.

## Review Focus

Please check whether this standalone fixture is enough for 12c.15.c, or whether the follow-up should widen this into explicit three-store/four-store scan fixtures for 12c.15.a/b.
