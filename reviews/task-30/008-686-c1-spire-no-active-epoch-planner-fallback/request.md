# Review Request: SPIRE no-active-epoch planner fallback

- coder: coder1
- date: 2026-05-14
- code commit: e22dfe58 `Pin SPIRE no-active-epoch planner fallback`
- topic: SPIRE phase 12c.10.d empty-placement planner-refusal fixture

## Scope

This slice tightens the existing no-active-epoch CustomScan eligibility fixture to assert the planner fallback shape, not just the eligibility row.

Changed file:

- `src/tests/custom_scan.rs`

## What Changed

Extended `test_ec_spire_customscan_eligibility_no_active_epoch` to:

- run `EXPLAIN (COSTS OFF)` for an ordered vector query against the no-active-epoch SPIRE index
- assert the plan does not contain `Custom Scan (EcSpireDistributedScan)`
- assert the plan remains a normal local `Index Scan` or `Seq Scan`

This addresses the 12c.10.d gap for an empty/no-active-epoch planner-refusal positive fixture.

## Test File Size Discipline

The touched test file is now 1155 lines:

```text
1155 src/tests/custom_scan.rs
```

No large test file was expanded past the 2500-line target.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/custom_scan.rs
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_eligibility_no_active_epoch --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`, but exited successfully.

I did not run the pg_test binary. Earlier runtime attempts in this branch still hit the local PostgreSQL backend symbol boundary before executing tests; this slice was validated with the narrow compile-only target.

## Review Focus

Please check whether the allowed fallback plan set should remain `Index Scan` or `Seq Scan`, or whether the test should accept another normal local scan node if planner settings evolve.
