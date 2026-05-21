# Review Request: SPIRE DML Index Key View

## Summary

This slice extends the SPIRE DML front-door relation metadata views so catalog callers stop reading index key metadata directly.

The change:

- folds heap relation OID and tuple descriptor capture into one `DmlFrontdoorHeapRelationView` boundary,
- adds `DmlFrontdoorIndexRelationView::key_attnum()` for bounded `indkey` reads,
- routes primary-key detection and embedding-column extraction through the index relation view, and
- removes the direct `indkey.values` unsafe reads from both catalog call sites.

## Unsafe Burn-Down

- `rg -n "unsafe" src | wc -l`: `2554 -> 2552`
- `rg -n "unsafe" src/am/ec_spire/dml_frontdoor/mod.rs | wc -l`: `71 -> 69`
- `rg -n "unsafe fn" src/am/ec_spire/dml_frontdoor/mod.rs | wc -l`: `20 -> 20`

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_spire/dml_frontdoor/mod.rs` passed with the existing stable-channel import-grouping warnings.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing `src/am/mod.rs` unused-import warning.
- `artifacts/cargo-test-dml-frontdoor-pg18-pgtest-no-run.log`: `cargo test --lib dml_frontdoor --no-default-features --features pg18,pg_test --no-run` passed with the existing Hadamard test-helper dead-code warnings.

