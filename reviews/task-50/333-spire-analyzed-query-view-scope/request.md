# Review Request: SPIRE Analyzed Query View Scope

Task: 50 unsafe burndown

Commit under review:

- `803cfb90` - `Scope analyzed SPIRE DML query views`

## Summary

This packet removes the raw `Query*` return from `storage::query::analyze_single_query` and routes SPIRE DML SQL diagnostics through a scoped analyzed-query view.

- `storage::query::analyze_single_query` now returns an `AnalyzedQuery` wrapper instead of `*mut pg_sys::Query`.
- `dml_frontdoor::with_analyzed_dml_frontdoor_query_view(sql, |view| ...)` owns the PostgreSQL parser/analyzer boundary and scopes the resulting `DmlFrontdoorQueryView`.
- The three SPIRE DML SQL diagnostic functions in `src/lib.rs` now consume that scoped helper and no longer carry local raw-query unsafe blocks.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count: `1347 -> 1345`.
- The broadened boundary-signature guard no longer reports `src/storage/query.rs::analyze_single_query`.
- Remaining guard hits are:
  - `src/am/ec_ivf/vacuum.rs`
  - `src/am/ec_hnsw/shared.rs`
  - `src/am/ec_hnsw/options.rs`

See `artifacts/unsafe-counts-and-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check HEAD~1..HEAD` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1345` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Runtime DML pg tests remain blocked by the PostgreSQL symbol-linkage issue captured in packet 329.

## Reviewer Focus

- Confirm `AnalyzedQuery` plus `with_analyzed_dml_frontdoor_query_view` is the right contract for replacing the safe raw `Query*` return.
- Confirm the SQL diagnostic functions still preserve their prior fail-closed error behavior.
