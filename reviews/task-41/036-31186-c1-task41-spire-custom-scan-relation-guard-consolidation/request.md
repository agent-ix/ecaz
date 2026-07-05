# Review Request: Task 41 SPIRE Custom Scan Relation Guard Consolidation

## Summary

Task 41 relation-guard completeness slice.

The code commit `3bd20ba60e1cd0b9a268b9a22661befdc97d609a` removes two module-local relation wrappers from the SPIRE custom-scan path:

- `OpenIndexRelation` in `src/am/ec_spire/custom_scan/explain.rs`
- `OpenTableRelation` in `src/am/ec_spire/custom_scan/planner.rs`

Call sites now use:

- `IndexRelationGuard::try_access_share` for SPIRE index relation opens
- `HeapRelationGuard::try_access_share` for placement table opens

This addresses the 31175 and 31180 reviewer feedback for the SPIRE custom-scan relation wrappers.

## Baseline Delta

- unsafe baseline entries: `4256 -> 4256`
- `src/am/ec_spire/custom_scan/explain.rs`: removed as a baseline-listed file for this guard pattern
- `src/am/ec_spire/custom_scan/planner.rs`: remains `37`

This slice is baseline-neutral because the removed local guards already had SAFETY comments. The value is structural: fewer module-local PostgreSQL relation wrappers and fewer copies of the close invariant.

See `artifacts/manifest.md` and `artifacts/validation.md`.

## Validation

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

`cargo check` passed with the existing PG18 C-header warnings and the existing unused re-export warning in `src/am/mod.rs`.

## Review Focus

- Confirm the `InvalidOid` behavior in explain remains fail-closed with zero context.
- Confirm planner call sites preserve prior skip/false behavior when relation opens fail.
- Confirm this resolves the `OpenIndexRelation` / `OpenTableRelation` items from the remaining module-local relation guard list.
