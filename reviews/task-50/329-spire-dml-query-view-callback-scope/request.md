# Review Request: SPIRE DML Query View Callback Scope

Task: 50 unsafe burndown

Commit under review:

- `6e957292` - `Scope SPIRE DML query views to callbacks`

## Summary

This packet continues the Task 50 P11/P13 path for SPIRE DML query-tree access.

- Replaces the escaping `dml_frontdoor_query_view(query) -> DmlFrontdoorQueryView<'a>` constructor with `with_dml_frontdoor_query_view(query, |view| ...)`.
- Updates the SPIRE DML planner hook to derive the replacement decision and optional plan-tree replacement inside that scoped query-view callback.
- Updates the three SQL diagnostic entry points in `src/lib.rs` to use the scoped query-view callback.
- Adds a test-only helper in `src/tests/dml_frontdoor.rs` so DML frontdoor tests reuse one scoped query-view boundary instead of repeatedly constructing query views in local unsafe blocks.

## Unsafe Count Impact

- `src/am/ec_spire/dml_frontdoor/mod.rs`: `37 -> 36`
- `src/lib.rs`: `21 -> 21`
- `src/tests/dml_frontdoor.rs`: `8 -> 4`
- Current `src/` total: `1349`

See `artifacts/unsafe-counts-before-after.log` and `artifacts/touched-file-unsafe-lines-after.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check HEAD~1..HEAD` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1349` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Runtime test attempts were blocked before execution by PostgreSQL symbol-linkage issues in this checkout:

- `cargo test dml_frontdoor --no-default-features --features pg18,bench`: `undefined symbol: pg_re_throw`
- `cargo pgrx test pg18 test_ec_spire_dml_frontdoor_primitive_plan_from_decision`: `undefined symbol: CacheRegisterRelcacheCallback`

Both blocked logs are packet-local under `artifacts/`.

## Reviewer Focus

- Confirm `with_dml_frontdoor_query_view` is the right shape for preventing query-view lifetimes from escaping callback/test scopes.
- Check whether the SQL diagnostic closures in `src/lib.rs` are acceptable or whether these should be extracted into smaller typed helper functions in a follow-up.
