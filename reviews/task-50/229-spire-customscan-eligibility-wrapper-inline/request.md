# Review Request: SPIRE CustomScan Eligibility Wrapper Inline

## Summary

Commit `22ec7b81f794cc9a7fade88420d3d69f5b5e1dd2` removes the single-purpose `custom_scan_index_eligibility_row` unsafe wrapper.

The `Result`-returning boundary `custom_scan_index_eligibility_result` is now exported through the AM facade and used directly by:

- the SQL-visible `ec_spire_custom_scan_index_eligibility` wrapper, which maps errors to `pgrx::error!`;
- the CustomScan explain context, which already owns an `IndexRelationGuard` for the relation lifetime.

No safe raw-pointer helper was introduced; callers still acknowledge the live relation contract through the existing `with_live_index_relation!` macro or an explicit guarded unsafe call.

## Unsafe Burndown

- `rg -n 'unsafe' src | wc -l`: `2519 -> 2517`
- Deleted:
  - `unsafe fn custom_scan_index_eligibility_row`
  - its internal `unsafe { custom_scan_index_eligibility_result(...) }` wrapper block

## Validation

See `artifacts/manifest.md`.

- `rustfmt --check src/am/ec_spire/custom_scan/planner.rs src/am/ec_spire/custom_scan/explain.rs src/am/ec_spire/mod.rs src/am/mod.rs`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run`

Known warnings/notes only:

- stable-channel rustfmt import grouping warnings
- direct `rustfmt --check src/lib.rs` traverses an unrelated existing `src/quant/simd.rs` formatting issue, so this packet used `git diff --check` for the small `src/lib.rs` call-site change
- `src/am/mod.rs` unused SPIRE re-export warning
- Hadamard test-helper dead-code warnings
