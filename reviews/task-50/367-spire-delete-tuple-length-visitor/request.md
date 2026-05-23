# Task 50 Review Request: SPIRE Delete Tuple Length Visitor

## Summary

This slice removes the SPIRE object-delete local line-pointer read used only to
measure tuple length before `delete_no_compact`.

Changes:

- `src/am/ec_spire/page.rs` now derives object delete tuple length through
  `LockedBufferGuard::visit_tuple_bytes`.
- Removed the delete path's direct `PageGetItemId` / line-pointer read and
  manual bounds check.

The direct `unsafe { ... }` count moved from `1202` to `1201`.

Code commit: `175b62e3cd5c7a635108264fff989a472219321e`

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed. It still reports the known pre-existing SPIRE DML unused-import
  warning in `src/am/mod.rs`.
- `git diff --check` passed.
- Raw-boundary guard scan produced no matches. The `rg` command exits 1 because
  it found no rows.
- SPIRE delete helper scan confirms object deletion now uses the locked-buffer
  tuple visitor for tuple length; line-pointer reads remain centralized in the
  shared visitor.
- Unsafe ledger check passed for `1201` current unsafe rows.

Artifacts are recorded in `artifacts/manifest.md`.
