# Review Request: SPIRE DML Query Helper Boundaries

## Summary

This checkpoint closes the in-flight SPIRE DML frontdoor raw `Query` helper slice after reviewing the soundness-audit feedback. The reviewer was correct that safe helpers accepting raw PostgreSQL pointers are an anti-pattern; this patch restores caller-visible `unsafe fn` contracts for the remaining public DML frontdoor query helpers instead of hiding the live-`Query` precondition in safe signatures.

Code commit: `7072f929cefcdab29380f5d1ed2c81b02df64255`

## Scope

- Marked the raw `pg_sys::Query` public helper boundary unsafe:
  - `dml_frontdoor_replacement_decision_catalog_row`
  - `dml_frontdoor_primitive_plan_expr_catalog_row`
  - `classify_dml_frontdoor_query`
  - `dml_frontdoor_target_relation_oid`
- Added explicit boundary acknowledgements at planner-hook, SQL diagnostic, and pg_test call sites.
- Kept private recursive classifier helpers safe where they operate within an already acknowledged live-query path, avoiding the earlier count explosion from making every internal step unsafe.

## Counts

- `src/am/ec_spire/dml_frontdoor/mod.rs`: `55` unsafe blocks before, `61` after.
- Current packet-local `src/` unsafe ledger: `1939` rows, checked.

This is a soundness-contract restoration slice, not a count-reduction slice. The increase is expected because the raw `Query` precondition now has compile-time call-site acknowledgement again.

## Soundness-Audit Feedback Assessment

All four files under `reviews/task-50/132-helper-soundness-audit/feedback/` were reviewed. The reviewer was correct on the central issue: safe helpers that take raw PostgreSQL pointers and rely only on comments are not defensible closeout evidence. Most listed audit items were already closed in packets 141-162; the remaining confirmed follow-up is the scan/slot guard lifetime `PhantomData` work called out in packet 157 feedback.

## Completion Audit Note

Task 50 is not complete: current ledger output still covers 1939 direct unsafe rows in `src/`, and final closeout still requires residual registration for every remaining unsafe plus hardening/crates/tests/vendor disposition.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check HEAD~1..HEAD`: passed.
- `make unsafe-block-count`: passed.
- `make unsafe-ledger`: generated packet-local ledger.
- `make unsafe-ledger-check`: passed.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
