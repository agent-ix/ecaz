# Review Request: SPIRE Phase 12c CustomScan Callback Pins

- Code commit: `7d3184e8` (`Cover SPIRE CustomScan callback pins`)
- Scope: Phase 12c test-only coverage slice for CustomScan FFI callback contracts and cost assertion tightening.
- Files changed:
  - `src/am/ec_spire/custom_scan/begin_exec.rs`
  - `src/am/ec_spire/custom_scan/tests.rs`

## What Changed

- Added a small non-FFI helper for the `recheck` callback's V1 EvalPlanQual contract, then pinned it with a Rust unit test.
- Added a unit test that asserts `MarkPosCustomScan` and `RestrPosCustomScan` remain unset while the expected lifecycle callbacks stay wired.
- Replaced three loose CustomScan cost assertions with proportional checks:
  - remote fanout startup and total-cost scaling;
  - output-row scaling without startup-cost movement;
  - projected tuple-width delta matching the tuple-byte cost term.

## File-Size Discipline

This slice stayed inside `src/am/ec_spire/custom_scan/tests.rs`, which is now 535 lines. It does not add to the already-large `src/tests/*` integration files.

## Validation

- `cargo fmt --check` passed.
- `cargo test --no-default-features --features pg18 custom_scan_recheck_returns_true_for_epq_stale_row_contract --no-run` passed, with the existing `src/am/mod.rs` unused-import warning.
- Runtime attempts with `cargo test --no-default-features --features pg18 custom_scan_ -- --nocapture` and `cargo pgrx test pg18 custom_scan_` did not complete because the plain unit-test binary exits before filtered tests run with unresolved PostgreSQL backend symbols (`CacheRegisterRelcacheCallback` / `pg_re_throw`). This is the existing pure-unit-test harness boundary for this crate, not a compile failure from this slice.

## Review Focus

1. Confirm the `custom_scan_recheck_allows_epq_stale_row()` helper is an acceptable minimal testability hook and preserves the callback behavior exactly.
2. Check whether the callback-methods test is enough for the 12c.1.d contract pin, or whether the next slice should add a live planner-refusal fixture.
3. Check the cost ratio bands for fanout scaling; they intentionally allow the current constant and merge terms while catching sign or multiplier regressions that the old `>` checks admitted.
