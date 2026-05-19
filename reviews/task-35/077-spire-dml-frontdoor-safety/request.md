# Task 35 Packet 077: SPIRE DML Frontdoor Safety Comments

## Code Under Review

- Commit: `fe1b305a9e16a811beb3275c445b008e3c9fa62e`
- Files:
  - `src/am/ec_spire/dml_frontdoor/mod.rs`
  - `scripts/unsafe_comment_baseline.txt`

## Scope

This slice burns down the remaining unsafe-comment baseline for the SPIRE DML frontdoor planner module.

The added comments document the safety boundaries for:

- backend-local planner hook installation and hook diagnostic globals;
- relcache callback registration and guarded relation-context loading;
- planner hook chaining, `standard_planner` fallback, and plan-tree replacement;
- guarded heap/index relcache metadata reads;
- PostgreSQL `Query`, `RangeTblEntry`, `TargetEntry`, and expression-list traversal;
- `Datum` and executor parameter decoding for bigint primary-key values.

## Baseline Movement

- Global unsafe-comment baseline: `2138 -> 1979`
- `src/am/ec_spire/dml_frontdoor/mod.rs`: `159 -> 0`

## Validation

- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/unsafe-baseline-report-after.log`: baseline is `1979` entries across `56` files.
- `artifacts/dml-frontdoor-baseline-after.log`: dml_frontdoor entries are `0`.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed.

Known unrelated warnings remain:

- unused `EC_PARALLEL_WORKER_SLOT_CLAIMED` in `src/am/common/parallel.rs`;
- unused SPIRE imports/re-exports in `src/am/mod.rs`.

`cargo fmt --all` emitted the repo's existing stable-rustfmt warnings for unstable rustfmt options. It also touched `hardening/careful/src/lib.rs` and `src/quant/simd.rs`; those unrelated formatting changes were restored before commit.
