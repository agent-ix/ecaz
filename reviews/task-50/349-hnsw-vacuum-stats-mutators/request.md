# Review Request: HNSW Vacuum Stats Mutators

Task: 50 unsafe burndown

Commit under review:

- `1fb2c405` - `Centralize HNSW vacuum stats mutation`

## Summary

This packet applies the shared vacuum stats mutators to HNSW vacuum paths.

- Replaces HNSW noop vacuum stats direct writes with `set_index_bulk_delete_summary`.
- Replaces HNSW bulkdelete direct stats writes with shared summary and removed-tuple mutators.
- Keeps PostgreSQL stats pointer ownership and return behavior unchanged.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1303` to `1301`.
- HNSW shared/vacuum no longer have direct `(*stats)` field writes.
- The broadened raw boundary-signature guard has no hits.

See `artifacts/unsafe-count.log`, `artifacts/hnsw-vacuum-stats-scan.log`, and `artifacts/raw-boundary-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1301` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm the shared stats mutators preserve HNSW noop and bulkdelete stats semantics.
- Confirm the `usize` to `u64` conversions are acceptable for the common helper API.
