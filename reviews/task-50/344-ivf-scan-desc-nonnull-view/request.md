# Review Request: IVF Scan Descriptor NonNull View

Task: 50 unsafe burndown

Commit under review:

- `65778792` - `Use NonNull IVF scan descriptor view`

## Summary

This packet removes the raw IVF scan descriptor view constructor.

- Replaces `IvfScanDescView::from_raw(pg_sys::IndexScanDesc, ...)` with `IvfScanDescView::from_nonnull(NonNull<IndexScanDescData>)`.
- Moves null checking to the AM callback/debug entry points before the descriptor view is constructed.
- Removes caller-side `unsafe { IvfScanDescView::from_raw(...) }` blocks from IVF heap rerank setup, EXPLAIN counters, and debug scan helpers.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1311` to `1308`.
- `IvfScanDescView::from_raw` has no remaining hits.
- The broadened raw boundary-signature guard has no hits.

See `artifacts/unsafe-count.log`, `artifacts/ivf-scan-desc-helper-scan.log`, and `artifacts/raw-boundary-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1308` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm `NonNull<IndexScanDescData>` is the right typed boundary for `IvfScanDescView`.
- Confirm the remaining internal `scan.as_ref()` unsafe is constrained to the view constructor and all call sites now handle null before construction.
