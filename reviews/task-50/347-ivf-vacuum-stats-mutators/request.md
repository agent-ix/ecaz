# Review Request: IVF Vacuum Stats Mutators

Task: 50 unsafe burndown

Commit under review:

- `68719357` - `Centralize IVF vacuum stats mutation`

## Summary

This packet centralizes IVF `IndexBulkDeleteResult` mutation through common vacuum helpers.

- Adds common `IndexBulkDeleteResult` mutators backed by a private scoped mutable borrow helper.
- Replaces IVF vacuum `(*stats).tuples_removed`, `num_pages`, `estimated_count`, and `num_index_tuples` writes with those helpers.
- Leaves the returned PostgreSQL stats pointer ownership unchanged.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1305` to `1304`.
- IVF vacuum no longer has direct `(*stats)` field writes.
- The broadened raw boundary-signature guard has no hits.

See `artifacts/unsafe-count.log`, `artifacts/ivf-vacuum-stats-scan.log`, and `artifacts/raw-boundary-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1304` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm the common stats mutators preserve IVF vacuum stats semantics.
- Confirm the private scoped mutable borrow helper is preferable to repeated AM-local `(*stats)` writes.
