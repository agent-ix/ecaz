# Review Request: IVF Page Tuple Access View

## Summary

This slice is structural unsafe cleanup for `src/am/ec_ivf/page.rs`. It consolidates duplicate page tuple reader/writer raw-page handling into a shared `PageTuplePage` view.

The change:

- introduces `PageTuplePage` as the single local owner of line-pointer count, item-id lookup, tuple-byte visiting, required-slot lookup, and exact tuple overwrite;
- makes `PageTupleReader` and `PageTupleWriter` delegate to that shared page view;
- removes the standalone `page_item_id_ref()` and `with_page_line_tuple_bytes()` helpers; and
- keeps read and write call sites on the existing safe reader/writer surface.

This does not reduce the literal `unsafe` count yet; it is a guard/view consolidation pass that narrows the remaining page tuple boundary for follow-on removal.

## Unsafe Burn-Down

- `rg -n "unsafe" src | wc -l`: `2548 -> 2548`
- `rg -n "unsafe" src/am/ec_ivf/page.rs | wc -l`: `48 -> 48`
- `rg -n "unsafe fn" src/am/ec_ivf/page.rs | wc -l`: `18 -> 18`

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_ivf/page.rs` passed with the existing stable-channel import-grouping warnings.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing `src/am/mod.rs` unused-import warning.
- `artifacts/cargo-test-ec-ivf-page-pg18-pgtest-no-run.log`: `cargo test --lib ec_ivf::page --no-default-features --features pg18,pg_test --no-run` passed with the existing Hadamard test-helper dead-code warnings.

