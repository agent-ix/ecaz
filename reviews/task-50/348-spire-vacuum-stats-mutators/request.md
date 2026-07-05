# Review Request: SPIRE Vacuum Stats Mutators

Task: 50 unsafe burndown

Commit under review:

- `787e4860` - `Centralize SPIRE vacuum stats mutation`

## Summary

This packet applies the shared vacuum stats mutators to SPIRE vacuum.

- Replaces SPIRE vacuum's direct `IndexBulkDeleteResult` field writes with common helpers.
- Removes the broad explicit unsafe block in `finish_vacuum_stats`.
- Keeps PostgreSQL ownership and return semantics unchanged for the stats pointer.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1304` to `1303`.
- SPIRE vacuum no longer has direct `(*stats)` field writes.
- The broadened raw boundary-signature guard has no hits.

See `artifacts/unsafe-count.log`, `artifacts/spire-vacuum-stats-scan.log`, and `artifacts/raw-boundary-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1303` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm the shared stats mutators preserve SPIRE vacuum stats semantics.
- Confirm removing the broad explicit unsafe block does not obscure any required callback invariant.
