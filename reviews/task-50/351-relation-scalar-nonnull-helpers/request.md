# Review Request: Relation Scalar NonNull Helpers

Task: 50 unsafe burndown

Commit under review:

- `cb50c8ee` - `Add NonNull relation scalar helpers`

## Summary

This packet adds typed relation scalar readers and rolls them into small AM/debug callers.

- Adds a storage-level `RelationHandle` alias plus safe `main_fork_block_count_handle` and `relation_reltuples_handle` helpers.
- Keeps the legacy raw-pointer storage APIs as checked wrappers around the typed helpers.
- Updates IVF admin diagnostics, DiskANN graph diagnostics, and HNSW scan debug block-count reads to pass checked handles.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1299` to `1295`.
- The updated AM/debug callers no longer wrap relation scalar reads in caller-side unsafe blocks.
- The broadened raw boundary-signature guard has no hits.

See `artifacts/unsafe-count.log`, `artifacts/relation-scalar-helper-scan.log`, and `artifacts/raw-boundary-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1295` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm the `RelationHandle` alias avoids reintroducing the safe raw `pg_sys::Relation` signature antipattern.
- Confirm the checked-handle call sites preserve previous null-relation failure behavior.
