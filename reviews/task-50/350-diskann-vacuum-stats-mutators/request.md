# Review Request: DiskANN Vacuum Stats Mutators

Task: 50 unsafe burndown

Commit under review:

- `91575261` - `Centralize DiskANN vacuum stats mutation`

## Summary

This packet applies the shared vacuum stats mutators to DiskANN vacuum paths.

- Replaces DiskANN noop vacuum stats direct writes with `set_index_bulk_delete_summary`.
- Replaces DiskANN bulkdelete stats writes with shared summary and removed-tuple mutators.
- Separates the medoid-refresh metadata flag update from stats result mutation.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1301` to `1299`.
- DiskANN routine no longer has direct `(*stats)` field writes.
- The broadened raw boundary-signature guard has no hits.

See `artifacts/unsafe-count.log`, `artifacts/diskann-vacuum-stats-scan.log`, and `artifacts/raw-boundary-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1299` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm the shared stats mutators preserve DiskANN noop and bulkdelete stats semantics.
- Confirm splitting medoid-refresh metadata update from stats mutation is behavior-preserving.
